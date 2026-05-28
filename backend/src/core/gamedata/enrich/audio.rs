use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use crate::core::gamedata::{
    assets::AssetIndex,
    types::{
        audio::{AudioCategory, AudioSound, OperatorAudio, RawAudioData},
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

        // Resolve the owning operator + optional skill metadata.
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

        // Resolve each sound asset to URLs, dropping anything that doesn't exist.
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
