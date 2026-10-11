use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use crate::core::gamedata::{
    assets::AssetIndex,
    types::{
        audio::{
            AudioCategory, AudioSound, EnemyAudio, EnemyAudioCategory, OperatorAudio, RawAudioData,
        },
        voice::LangType,
    },
};

/// Direct operator id embedded in a bank name, e.g. `char_101_sora`.
static CHAR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"char_\d+_[a-z0-9]+").unwrap());
/// Skill / talent entity, e.g. `skchr_amiya_2` or `tachr_skadi2_1`. Captures the
/// operator codename and the slot number.
static SK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:skchr|tachr)_([a-z0-9]+)_(\d+)").unwrap());

/// Resolve every operator-linked `SoundFX` bank to playable clips, keyed by char id.
///
/// A bank links to an operator either directly (`char_<id>_<code>`) or by
/// codename via a skill/talent entity (`skchr_<code>_<n>` / `tachr_<code>_<n>`).
/// Codename matching is required because some skill banks use `skchr_<code>`
/// even when the operator's real skill is a shared `skcom_*`.
pub fn build_operator_audio(
    raw: &RawAudioData,
    op_ids: &HashSet<&str>,
    assets: &AssetIndex,
) -> HashMap<String, Vec<OperatorAudio>> {
    // codename (3rd `_`-segment of the char id) -> char id
    let code2char: HashMap<&str, &str> = op_ids
        .iter()
        .filter_map(|id| id.splitn(3, '_').nth(2).map(|code| (code, *id)))
        .collect();

    let mut by_char: HashMap<String, Vec<OperatorAudio>> = HashMap::new();

    for bank in &raw.sound_fx_banks {
        // Split off an optional `@LANG` suffix (battle voice barks).
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

        let mut sounds: Vec<AudioSound> = Vec::new();
        let mut any_voice = false;
        for sound in &bank.sounds {
            let urls = assets.resolve_audio(&sound.asset);
            if urls.is_empty() {
                continue;
            }
            any_voice |= is_voice_asset(&sound.asset);
            sounds.push(AudioSound {
                asset: sound.asset.clone(),
                urls,
            });
        }
        if sounds.is_empty() {
            continue;
        }

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

    // Deterministic ordering: by event, then full bank name.
    for clips in by_char.values_mut() {
        clips.sort_by(|a, b| a.event.cmp(&b.event).then(a.bank_name.cmp(&b.bank_name)));
    }

    by_char
}

/// Enemy codename: the third `_`-segment of a numbered enemy id, e.g. `frstar`
/// in `enemy_1505_frstar` or `hlslp` in `enemy_10092_hlslp_1`.
static ENEMY_CODE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^enemy_\d+_([a-z0-9]+)").unwrap());

/// Resolve every enemy-linked `SoundFX` bank to playable clips, keyed by the
/// handbook enemy id.
///
/// The entity segment of the bank name (`battle.<EVENT>.<entity>[.<ability>]`)
/// links a bank to an enemy three ways, tried in order:
///
/// 1. it IS the enemy id, or the `PrefabKey` an `enemy_database` level gives it
///    (`enemy_8007_eltrsm` plays the `enemy_8007_eltrsm_3` prefab's banks);
/// 2. `projectile_[enemy_]<code>...`: the enemy that fires it is named only in
///    the unit prefab, which is not exported, so the bank links by codename to
///    every enemy sharing it. Codenames an operator also uses (`cqbw`, `faust`)
///    link only with the explicit `projectile_enemy_` prefix;
/// 3. `enemy_<code>...` with a non-numeric code: an enemy-specific buff such as
///    `enemy_sfsui_doom1`, linked by codename the same way.
///
/// The catch-all `battle.ON_UNIT_DEAD.enemy` names no enemy and links to none.
/// `prefab_keys` maps an enemy id to the distinct prefab keys its levels use;
/// `skill_keys` maps it to its `enemy_database` `Skills[].PrefabKey`s
/// (lowercased, `[tag]` dropped), which name a bank's ability when it is a skill.
pub fn build_enemy_audio<'a>(
    raw: &RawAudioData,
    enemy_ids: impl IntoIterator<Item = &'a str>,
    prefab_keys: &HashMap<String, Vec<String>>,
    skill_keys: &HashMap<String, HashSet<String>>,
    op_ids: &HashSet<&str>,
    assets: &AssetIndex,
) -> HashMap<String, Vec<EnemyAudio>> {
    let op_codes: HashSet<&str> = op_ids
        .iter()
        .filter_map(|id| id.splitn(3, '_').nth(2))
        .collect();

    // entity key (own id or prefab key) -> enemy ids, and codename -> enemy ids
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
                .entry(caps.get(1).unwrap().as_str())
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
            // Drop a `[audio]`-style tag before reading the codename.
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

        let sounds: Vec<AudioSound> = bank
            .sounds
            .iter()
            .filter_map(|sound| {
                let urls = assets.resolve_audio(&sound.asset);
                (!urls.is_empty()).then(|| AudioSound {
                    asset: sound.asset.clone(),
                    urls,
                })
            })
            .collect();
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

/// Projectile, spawn and death by the bank's own event; then Skill when the
/// ability names one of the enemy's `Skills[].PrefabKey`; then the basic
/// attack; then Skill by elimination for the remaining `ON_ABILITY*`/
/// `ON_BUFF*` banks, whose abilities (`T.Reborn`, `attack_p1_start`) no table
/// lists. EN 2026-10-11, 5,326 enemy-bank links: Skill 1,521 -> 1,522 (1,149
/// named by a `PrefabKey`, 373 still by elimination), Other 213 -> 212 (the
/// one move is `ON_SPINE_EVENT_TRIGGER.enemy_1537_mhrors.roar`); Attack 2,529,
/// Projectile 832, Death 149 and Spawn 82 unchanged.
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

/// True when the logical asset lives under a voice directory (`Voice*` / `Vox`).
fn is_voice_asset(asset: &str) -> bool {
    let lower = asset.to_ascii_lowercase();
    lower.contains("/voice/")
        || lower.contains("/voice_cn/")
        || lower.contains("/voice_en/")
        || lower.contains("/voice_kr/")
        || lower.contains("/vox/")
}

/// Infer the bark language from the `Voice_XX` directory in the asset path.
fn lang_from_asset(asset: &str) -> Option<LangType> {
    let lower = asset.to_ascii_lowercase();
    if lower.contains("/voice_cn/") {
        Some(LangType::CnMandarin)
    } else if lower.contains("/voice_en/") {
        Some(LangType::En)
    } else if lower.contains("/voice_kr/") {
        Some(LangType::Kr)
    } else if lower.contains("/voice/") {
        // `Voice` is the default-language (JP) bank.
        Some(LangType::Jp)
    } else {
        None
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
