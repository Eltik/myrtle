use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::voice::LangType;

// ============================================================================
// Raw types (audio_data.json)
// ============================================================================
//
// `audio_data.json` holds many sections (BgmBanks, Musics, BattleVoice, ...);
// two of them are read here. The SoundFX banks map battle events to playable
// assets: every bank is named `<scope>.<EVENT>.<entity>[.<sub>...][@LANG]`,
// e.g. `battle.ON_UNIT_BORN.char_101_sora` or
// `battle.ON_SKILL_START.skchr_amiya_2`. `Musics` + `BgmBanks` + `BankAlias`
// are the SOUNDTRACK, which the Archives serve per story group and per
// archived track; see [`MusicBanks`].

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawAudioData {
    // uppercase FX is the source's spelling, not a typo
    #[serde(rename = "SoundFXBanks", default)]
    pub sound_fx_banks: Vec<RawSoundFxBank>,
    /// The named tracks: 208 on EN, each pointing at a bank by name.
    #[serde(rename = "Musics", default)]
    pub musics: Vec<RawMusic>,
    /// The music banks: 509 on EN, a loop clip and usually an intro. 3 of
    /// them carry no loop at all.
    #[serde(rename = "BgmBanks", default)]
    pub bgm_banks: Vec<RawBgmBank>,
    /// Bank name -> bank name: 215 EN rows that point a variant at the bank
    /// that actually holds the clips (`battle.ON_GAME_READY.indust` ->
    /// `battle.ON_GAME_READY`).
    #[serde(rename = "BankAlias", default)]
    pub bank_alias: Vec<RawBankAlias>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawMusic {
    pub id: String,
    /// The track's title. 101 of the 208 EN rows write a single space rather
    /// than an empty string, which is why the title is trimmed before use.
    #[serde(default)]
    pub name: String,
    pub bank: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawBgmBank {
    #[serde(rename = "Name", default)]
    pub name: String,
    /// The one-shot lead-in, absent on 134 of the 509 EN banks.
    #[serde(rename = "Intro", default)]
    pub intro: Option<String>,
    /// The looping body. The trailing underscore is the source's spelling.
    #[serde(rename = "Loop_", alias = "Loop", default)]
    pub loop_: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawBankAlias {
    #[serde(rename = "key", alias = "Key", default)]
    pub key: String,
    #[serde(rename = "value", alias = "Value", default)]
    pub value: String,
}

/// The music table as the story index reads it: tracks by id, tracks by bank,
/// banks by name and the alias redirect.
///
/// Kept on `GameData` because the Archives need it long after
/// `audio_data.json` is dropped, and it is SMALL: 208 tracks, 509 banks and
/// 215 aliases on EN.
#[derive(Debug, Clone, Default)]
pub struct MusicBanks {
    by_id: std::collections::HashMap<String, RawMusic>,
    by_bank: std::collections::HashMap<String, Vec<RawMusic>>,
    banks: std::collections::HashMap<String, RawBgmBank>,
    alias: std::collections::HashMap<String, String>,
}

impl MusicBanks {
    #[must_use]
    pub fn from_raw(raw: &RawAudioData) -> Self {
        let mut by_id = std::collections::HashMap::new();
        let mut by_bank: std::collections::HashMap<String, Vec<RawMusic>> =
            std::collections::HashMap::new();
        for m in &raw.musics {
            by_id.insert(m.id.clone(), m.clone());
            by_bank.entry(m.bank.clone()).or_default().push(m.clone());
        }
        // A named track beats an unnamed one and the id breaks the tie, so
        // the row a bank answers with is the same on every build.
        for rows in by_bank.values_mut() {
            rows.sort_by(|a, b| {
                a.name
                    .trim()
                    .is_empty()
                    .cmp(&b.name.trim().is_empty())
                    .then_with(|| a.id.cmp(&b.id))
            });
        }
        Self {
            by_id,
            by_bank,
            banks: raw
                .bgm_banks
                .iter()
                .map(|b| (b.name.clone(), b.clone()))
                .collect(),
            alias: raw
                .bank_alias
                .iter()
                .map(|a| (a.key.clone(), a.value.clone()))
                .collect(),
        }
    }

    #[must_use]
    pub fn music(&self, id: &str) -> Option<&RawMusic> {
        self.by_id.get(id)
    }

    /// The track a bank plays, the named one when several share the bank
    /// (5 EN banks do, each an `music_3in1bg_*` duplicate of a titled row).
    #[must_use]
    pub fn music_for_bank(&self, bank: &str) -> Option<&RawMusic> {
        self.by_bank.get(bank)?.first()
    }

    /// The first track whose bank starts with `prefix`, in id order: the way
    /// `act27side.day` and `act44side.theme1` are reached, since those banks
    /// carry a suffix the group id does not.
    #[must_use]
    pub fn music_by_bank_prefix(&self, prefix: &str) -> Option<&RawMusic> {
        let mut best: Option<&RawMusic> = None;
        for (bank, rows) in &self.by_bank {
            if !bank.starts_with(prefix) {
                continue;
            }
            for m in rows {
                if best.is_none_or(|b| {
                    (m.name.trim().is_empty(), &m.id) < (b.name.trim().is_empty(), &b.id)
                }) {
                    best = Some(m);
                }
            }
        }
        best
    }

    /// The bank by name, following the alias when the name is one.
    #[must_use]
    pub fn bank(&self, name: &str) -> Option<&RawBgmBank> {
        self.banks
            .get(name)
            .or_else(|| self.banks.get(self.alias.get(name)?))
    }

    #[must_use]
    pub fn counts(&self) -> (usize, usize, usize) {
        (self.by_id.len(), self.banks.len(), self.alias.len())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawSoundFxBank {
    pub name: String,
    #[serde(default)]
    pub sounds: Vec<RawSound>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawSound {
    /// Logical asset path, e.g. `Audio/Sound_Beta_2/Battle/b_char/b_char_kong`.
    pub asset: String,
}

/// Coarse semantic label derived from the bank's event/sub-parts/assets so the
/// frontend can group an operator's clips without parsing bank names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum AudioCategory {
    /// `ON_UNIT_BORN` - deployment / spawn sound.
    Deploy,
    /// Basic-attack sounds (`.attack` sub-part).
    Attack,
    /// Skill activation / ability sounds (keyed by `skchr_`/`tachr_` or
    /// `ON_SKILL_*`).
    Skill,
    /// In-battle voice bark (asset resolves under `Voice*`/`Vox`).
    Voice,
    /// Anything else (deaths, buffs, custom triggers, ...).
    #[default]
    Other,
}

/// One resolved sound entry within a bank. A single asset can map to several
/// on-disk files (weighted random variations, e.g. `b_char_kong`, `_1`, `_2`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AudioSound {
    /// Original logical asset path from `audio_data.json`.
    pub asset: String,
    /// Playable URLs (`/audio/sound_beta_2/...`), base file plus any variants.
    pub urls: Vec<String>,
}

/// A single operator-linked `SoundFX` bank, resolved to playable URLs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct OperatorAudio {
    /// Full bank name, e.g. `battle.ON_UNIT_BORN.char_101_sora`.
    pub bank_name: String,
    /// Event segment of the bank name, e.g. `ON_UNIT_BORN`.
    pub event: String,
    pub category: AudioCategory,
    /// Skill id when the bank is keyed by `skchr_`/`tachr_`, e.g. `skchr_sora_2`.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,
    /// Skill slot (1/2/3) parsed from the skill id, when present.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_slot: Option<u8>,
    /// Language of a voice bark, from the `@LANG` suffix or the `Voice_XX` dir.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangType>,
    pub sounds: Vec<AudioSound>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW: &str = r#"{
        "Musics": [
            {"Id": "music_act13side_0", "Name": "The Grand Knight Territory", "Bank": "sys.ON_ACTIVITY_LOADED.act13side"},
            {"Id": "music_3in1bg_act13side", "Name": " ", "Bank": "sys.ON_ACTIVITY_LOADED.act13side"},
            {"Id": "music_3in1bg_main1", "Name": " ", "Bank": "battle.ON_GAME_READY.escape"},
            {"Id": "music_bg_volcano_day", "Name": "Misty Memory", "Bank": "sys.ON_ACTIVITY_LOADED.act27side.day"}
        ],
        "BgmBanks": [
            {"Name": "sys.ON_ACTIVITY_LOADED.act13side", "Intro": "Audio/Sound_Beta_2/Music/act13side/m_sys_kazimierz2_intro", "Loop_": "Audio/Sound_Beta_2/Music/act13side/m_sys_kazimierz2_loop"},
            {"Name": "battle.ON_GAME_READY", "Loop_": "Audio/Sound_Beta_2/Music/beta1_180603/m_bat_indust_loop"},
            {"Name": "sys.ON_ACTIVITY_LOADED.act27side.day", "Loop_": "Audio/Sound_Beta_2/Music/act27side/m_sys_volcano_day_loop"}
        ],
        "BankAlias": [{"key": "battle.ON_GAME_READY.escape", "value": "battle.ON_GAME_READY"}]
    }"#;

    #[test]
    fn a_bank_resolves_through_its_alias() {
        let raw: RawAudioData = serde_json::from_str(RAW).unwrap();
        let banks = MusicBanks::from_raw(&raw);
        assert_eq!(banks.counts(), (4, 3, 1));
        let m = banks.music("music_3in1bg_main1").unwrap();
        assert_eq!(m.bank, "battle.ON_GAME_READY.escape");
        assert!(m.name.trim().is_empty());
        let bank = banks.bank(&m.bank).unwrap();
        assert_eq!(
            bank.loop_.as_deref(),
            Some("Audio/Sound_Beta_2/Music/beta1_180603/m_bat_indust_loop")
        );
        assert!(bank.intro.is_none());
        assert!(banks.bank("battle.ON_NOTHING").is_none());
    }

    #[test]
    fn the_named_row_wins_a_shared_bank() {
        let raw: RawAudioData = serde_json::from_str(RAW).unwrap();
        let banks = MusicBanks::from_raw(&raw);
        let m = banks
            .music_for_bank("sys.ON_ACTIVITY_LOADED.act13side")
            .unwrap();
        assert_eq!(m.id, "music_act13side_0");
        assert_eq!(m.name, "The Grand Knight Territory");
        let intro = banks.bank(&m.bank).unwrap().intro.as_deref();
        assert_eq!(
            intro,
            Some("Audio/Sound_Beta_2/Music/act13side/m_sys_kazimierz2_intro")
        );
    }

    #[test]
    fn a_suffixed_bank_is_reached_by_prefix() {
        let raw: RawAudioData = serde_json::from_str(RAW).unwrap();
        let banks = MusicBanks::from_raw(&raw);
        assert!(
            banks
                .music_for_bank("sys.ON_ACTIVITY_LOADED.act27side")
                .is_none()
        );
        let m = banks
            .music_by_bank_prefix("sys.ON_ACTIVITY_LOADED.act27side.")
            .unwrap();
        assert_eq!(m.id, "music_bg_volcano_day");
    }
}
