#[derive(Debug, Default)]
pub struct AudioTables {}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
#[cfg(feature = "schema")]
use utoipa::ToSchema;

use super::voice::LangType;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawAudioData {
    #[serde(rename = "SoundFXBanks", default)]
    pub sound_fx_banks: Vec<RawSoundFxBank>,
    /// Named tracks
    #[serde(rename = "Musics", default)]
    pub musics: Vec<RawMusic>,
    #[serde(rename = "BgmBanks", default)]
    pub bgm_banks: Vec<RawBgmBank>,
    /// Bank name -> bank name (`battle.ON_GAME_READY.indust` ->
    /// `battle.ON_GAME_READY`).
    #[serde(rename = "BankAlias", default)]
    pub bank_alias: Vec<RawBankAlias>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawMusic {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub bank: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawBgmBank {
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "Intro", default)]
    pub intro: Option<String>,
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

#[derive(Debug, Clone, Default)]
pub struct MusicBanks {
    by_id: HashMap<String, RawMusic>,
    by_bank: HashMap<String, Vec<RawMusic>>,
    banks: HashMap<String, RawBgmBank>,
    alias: HashMap<String, String>,
}

impl MusicBanks {
    #[must_use]
    pub fn from_raw(raw: &RawAudioData) -> Self {
        let mut by_id = HashMap::new();
        let mut by_bank: HashMap<String, Vec<RawMusic>> = HashMap::new();
        for m in &raw.musics {
            by_id.insert(m.id.clone(), m.clone());
            by_bank.entry(m.bank.clone()).or_default().push(m.clone());
        }
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

    #[must_use]
    pub fn music_for_bank(&self, bank: &str) -> Option<&RawMusic> {
        self.by_bank.get(bank)?.first()
    }

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

    #[must_use]
    pub fn bank(&self, name: &str) -> Option<&RawBgmBank> {
        self.banks
            .get(name)
            .or_else(|| self.banks.get(self.alias.get(name)?))
    }

    pub fn musics(&self) -> impl Iterator<Item = &RawMusic> {
        self.by_id.values()
    }

    pub fn banks(&self) -> impl Iterator<Item = &RawBgmBank> {
        self.banks.values()
    }

    pub fn aliases(&self) -> impl Iterator<Item = (&str, &str)> {
        self.alias.iter().map(|(k, v)| (k.as_str(), v.as_str()))
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
    pub asset: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub enum AudioCategory {
    /// `ON_UNIT_BORN` - deployment / spawn sound
    Deploy,
    /// Basic-attack sounds (`.attack` sub-part)
    Attack,
    /// Skill activation / ability sounds (`skchr_`/`tachr_` or `ON_SKILL_*`)
    Skill,
    /// In-battle voice lines (asset resolves under `Voice*`/`Vox`)
    Voice,
    /// Anything else (deaths, buffs, custom triggers, etc.).
    #[default]
    Other,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct AudioSound {
    /// Original logical asset path from `audio_data.json`
    pub asset: String,
    /// Playable URLs (`/audio/sound_beta_2/...`)
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct OperatorAudio {
    /// e.g. `battle.ON_UNIT_BORN.char_101_sora`
    pub bank_name: String,
    /// e.g. `ON_UNIT_BORN`
    pub event: String,
    pub category: AudioCategory,
    /// e.g. `skchr_sora_2`
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,
    /// Skill slot (1/2/3) parsed from skill id
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_slot: Option<u8>,
    /// Language of a voice line from `@LANG` suffix or `Voice_XX` dir
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangType>,
    pub sounds: Vec<AudioSound>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub enum EnemyAudioCategory {
    /// `ON_UNIT_BORN` or `ON_BOSS_ENTER`.
    Spawn,
    /// Basic attack, `combat` (melee) or `attack` (ranged) ability
    Attack,
    /// Any other ability (`IceBurst`, `T.AppearMode`, etc.) or enemy-specific buff (`ON_BUFF_*`)
    Skill,
    /// Projectile bank (`projectile_*`) linked by codename
    Projectile,
    /// `ON_UNIT_DEAD`.
    Death,
    /// Spine events, custom triggers, etc.
    #[default]
    Other,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct EnemyAudio {
    /// e.g. `battle.ON_ABILITY_HIT.enemy_1007_slime.combat`
    pub bank_name: String,
    /// e.g. `ON_ABILITY_HIT`
    pub event: String,
    pub category: EnemyAudioCategory,
    /// e.g. `combat`, `IceBurst` or `T.over_energy`
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability: Option<String>,
    /// Entity segment, when is not enemy itself
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    pub sounds: Vec<AudioSound>,
}
