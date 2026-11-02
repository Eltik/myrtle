use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use myrtle_gamedata_raw::tables::{
    audio::{
        AudioCategory, AudioSound, EnemyAudio, EnemyAudioCategory, OperatorAudio, RawAudioData,
        RawSoundFxBank,
    },
    voice::LangType,
};
use regex::Regex;

use crate::assets::AssetIndex;

static CHAR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"char_\d+_[a-z0-9]+").expect("valid regex"));
static SK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:skchr|tachr)_([a-z0-9]+)_(\d+)").expect("valid regex"));

fn codename(id: &str) -> Option<&str> {
    id.splitn(3, '_').nth(2)
}

fn playable_sounds(bank: &RawSoundFxBank, assets: &AssetIndex) -> Vec<AudioSound> {
    bank.sounds
        .iter()
        .filter_map(|sound| {
            let urls = assets.resolve_audio(&sound.asset);
            (!urls.is_empty()).then(|| AudioSound {
                asset: sound.asset.clone(),
                urls,
            })
        })
        .collect()
}

pub fn build_operator_audio(
    raw: &RawAudioData,
    op_ids: &HashSet<&str>,
    assets: &AssetIndex,
) -> HashMap<String, Vec<OperatorAudio>> {
    let code2char: HashMap<&str, &str> = op_ids
        .iter()
        .filter_map(|id| codename(id).map(|code| (code, *id)))
        .collect();

    let mut by_char: HashMap<String, Vec<OperatorAudio>> = HashMap::new();

    for bank in &raw.sound_fx_banks {
        let (base_name, lang_suffix) = match bank.name.rsplit_once('@') {
            Some((n, l)) => (n, parse_lang(l)),
            None => (bank.name.as_str(), None),
        };

        let mut skill_id = None;
        let mut skill_slot = None;
        let char_id: Option<&str> = match CHAR_RE.find(base_name) {
            Some(m) if op_ids.contains(m.as_str()) => Some(m.as_str()),
            _ => SK_RE.captures(base_name).and_then(|caps| {
                let cid = code2char.get(&caps[1]).copied()?;
                skill_id = Some(caps[0].to_owned());
                skill_slot = caps[2].parse::<u8>().ok();
                Some(cid)
            }),
        };
        let Some(char_id) = char_id else { continue };

        let sounds = playable_sounds(bank, assets);
        if sounds.is_empty() {
            continue;
        }

        let any_voice = sounds.iter().any(|s| is_voice_asset(&s.asset));
        let parts: Vec<&str> = base_name.split('.').collect();
        let event = parts.get(1).copied().unwrap_or("").to_owned();
        let category = derive_category(&event, &parts, skill_id.is_some(), any_voice);

        let language = lang_suffix.or_else(|| {
            any_voice
                .then(|| sounds.iter().find_map(|s| lang_from_asset(&s.asset)))
                .flatten()
        });

        by_char
            .entry(char_id.to_owned())
            .or_default()
            .push(OperatorAudio {
                bank_name: bank.name.clone(),
                event,
                category,
                skill_id,
                skill_slot,
                language,
                sounds,
            });
    }

    for clips in by_char.values_mut() {
        clips.sort_by(|a, b| a.event.cmp(&b.event).then(a.bank_name.cmp(&b.bank_name)));
    }

    by_char
}

static ENEMY_CODE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^enemy_\d+_([a-z0-9]+)").expect("valid regex"));

pub fn build_enemy_audio<'a>(
    raw: &RawAudioData,
    enemy_ids: impl IntoIterator<Item = &'a str>,
    prefab_keys: &HashMap<String, Vec<String>>,
    skill_keys: &HashMap<String, HashSet<String>>,
    op_ids: &HashSet<&str>,
    assets: &AssetIndex,
) -> HashMap<String, Vec<EnemyAudio>> {
    let op_codes: HashSet<&str> = op_ids.iter().filter_map(|id| codename(id)).collect();
    let mut by_key: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut by_code: HashMap<&str, Vec<&str>> = HashMap::new();
    for id in enemy_ids {
        let keys = std::iter::once(id).chain(
            prefab_keys
                .get(id)
                .into_iter()
                .flatten()
                .map(String::as_str),
        );
        for key in keys {
            let ids = by_key.entry(key).or_default();
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        if let Some(caps) = ENEMY_CODE_RE.captures(id) {
            by_code
                .entry(caps.get(1).expect("group 1 always participates").as_str())
                .or_default()
                .push(id);
        }
    }

    let mut by_enemy: HashMap<String, Vec<EnemyAudio>> = HashMap::new();

    for bank in &raw.sound_fx_banks {
        let name = bank.name.trim();
        let base_name = name.rsplit_once('@').map_or(name, |(n, _)| n);
        let parts: Vec<&str> = base_name.split('.').collect();
        let (Some(&event), Some(&entity)) = (parts.get(1), parts.get(2)) else {
            continue;
        };

        let (targets, via, is_projectile) = if let Some(ids) = by_key.get(entity) {
            (ids, None, false)
        } else {
            let bare = entity.split('[').next().unwrap_or(entity);
            let (code, explicit, is_projectile) =
                if let Some(rest) = bare.strip_prefix("projectile_") {
                    match rest.strip_prefix("enemy_") {
                        Some(r) => (r.split('_').next(), true, true),
                        None => (rest.split('_').next(), false, true),
                    }
                } else if let Some(rest) = bare.strip_prefix("enemy_") {
                    let code = rest
                        .split('_')
                        .next()
                        .filter(|c| !c.is_empty() && !c.bytes().all(|b| b.is_ascii_digit()));
                    (code, true, false)
                } else {
                    (None, false, false)
                };
            let Some(code) = code else { continue };
            if !explicit && op_codes.contains(code) {
                continue;
            }
            let Some(ids) = by_code.get(code) else {
                continue;
            };
            (ids, Some(entity.to_owned()), is_projectile)
        };

        let sounds = playable_sounds(bank, assets);
        if sounds.is_empty() {
            continue;
        }

        let ability = (parts.len() > 3).then(|| parts[3..].join("."));
        for id in targets {
            let category = derive_enemy_category(
                event,
                ability.as_deref(),
                is_projectile,
                skill_keys.get(*id),
            );
            by_enemy
                .entry((*id).to_owned())
                .or_default()
                .push(EnemyAudio {
                    bank_name: bank.name.trim().to_owned(),
                    event: event.to_owned(),
                    category,
                    ability: ability.clone(),
                    via: via.clone(),
                    sounds: sounds.clone(),
                });
        }
    }

    for clips in by_enemy.values_mut() {
        clips.sort_by(|a, b| a.event.cmp(&b.event).then(a.bank_name.cmp(&b.bank_name)));
    }

    by_enemy
}

fn lang_from_asset(asset: &str) -> Option<LangType> {
    let lower = asset.to_ascii_lowercase();
    if lower.contains("/voice_cn/") {
        Some(LangType::CnMandarin)
    } else if lower.contains("/voice_en/") {
        Some(LangType::En)
    } else if lower.contains("/voice_kr/") {
        Some(LangType::Kr)
    } else if lower.contains("/voice/") {
        Some(LangType::Jp)
    } else {
        None
    }
}

fn is_voice_asset(asset: &str) -> bool {
    let lower = asset.to_ascii_lowercase();
    lower.contains("/voice/")
        || lower.contains("/voice_cn/")
        || lower.contains("/voice_en/")
        || lower.contains("/voice_kr/")
        || lower.contains("/vox/")
}

fn derive_enemy_category(
    event: &str,
    ability: Option<&str>,
    is_projectile: bool,
    skill_keys: Option<&HashSet<String>>,
) -> EnemyAudioCategory {
    let head = ability.and_then(|a| a.split('.').next());
    let basic_attack = head.is_some_and(|a| a == "combat" || a == "attack");
    let named_skill = head
        .zip(skill_keys)
        .is_some_and(|(a, keys)| keys.contains(&a.to_ascii_lowercase()));
    if is_projectile {
        EnemyAudioCategory::Projectile
    } else if event == "ON_UNIT_BORN" || event == "ON_BOSS_ENTER" {
        EnemyAudioCategory::Spawn
    } else if event == "ON_UNIT_DEAD" {
        EnemyAudioCategory::Death
    } else if named_skill && !basic_attack {
        EnemyAudioCategory::Skill
    } else if basic_attack {
        EnemyAudioCategory::Attack
    } else if event.starts_with("ON_ABILITY") || event.starts_with("ON_BUFF") {
        EnemyAudioCategory::Skill
    } else {
        EnemyAudioCategory::Other
    }
}

fn derive_category(
    event: &str,
    parts: &[&str],
    is_skill_entity: bool,
    any_voice: bool,
) -> AudioCategory {
    if any_voice {
        AudioCategory::Voice
    } else if event == "ON_UNIT_BORN" {
        AudioCategory::Deploy
    } else if parts.contains(&"attack") {
        AudioCategory::Attack
    } else if is_skill_entity || event.starts_with("ON_SKILL") {
        AudioCategory::Skill
    } else {
        AudioCategory::Other
    }
}

fn parse_lang(s: &str) -> Option<LangType> {
    Some(match s {
        "JP" => LangType::Jp,
        "CN_MANDARIN" => LangType::CnMandarin,
        "EN" => LangType::En,
        "KR" => LangType::Kr,
        "CN_TOPOLECT" => LangType::CnTopolect,
        "RUS" => LangType::Rus,
        "ITA" => LangType::Ita,
        "GER" => LangType::Ger,
        "FRE" => LangType::Fre,
        "SPA" => LangType::Spa,
        "LINKAGE" => LangType::Linkage,
        _ => return None,
    })
}
