use serde::{Deserialize, Serialize};

use super::voice::LangType;

// ============================================================================
// Raw types (audio_data.json)
// ============================================================================
//
// `audio_data.json` holds many sections (BgmBanks, Musics, BattleVoice, ...);
// we only need the SoundFX banks, which map battle events to playable assets.
// Every bank is named `<scope>.<EVENT>.<entity>[.<sub>...][@LANG]`, e.g.
// `battle.ON_UNIT_BORN.char_101_sora` or `battle.ON_SKILL_START.skchr_amiya_2`.

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawAudioData {
    // Note the uppercase `FX` in the source key (not `Fx`).
    #[serde(rename = "SoundFXBanks", default)]
    pub sound_fx_banks: Vec<RawSoundFxBank>,
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

// ============================================================================
// Enriched output types
// ============================================================================

/// Coarse semantic label derived from the bank's event/sub-parts/assets so the
/// frontend can group an operator's clips without parsing bank names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioCategory {
    /// `ON_UNIT_BORN` — deployment / spawn sound.
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
pub struct AudioSound {
    /// Original logical asset path from `audio_data.json`.
    pub asset: String,
    /// Playable URLs (`/audio/sound_beta_2/...`), base file plus any variants.
    pub urls: Vec<String>,
}

/// A single operator-linked SoundFX bank, resolved to playable URLs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorAudio {
    /// Full bank name, e.g. `battle.ON_UNIT_BORN.char_101_sora`.
    pub bank_name: String,
    /// Event segment of the bank name, e.g. `ON_UNIT_BORN`.
    pub event: String,
    pub category: AudioCategory,
    /// Skill id when the bank is keyed by `skchr_`/`tachr_`, e.g. `skchr_sora_2`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,
    /// Skill slot (1/2/3) parsed from the skill id, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_slot: Option<u8>,
    /// Language of a voice bark, from the `@LANG` suffix or the `Voice_XX` dir.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangType>,
    pub sounds: Vec<AudioSound>,
}
