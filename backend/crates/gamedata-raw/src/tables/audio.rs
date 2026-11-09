use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::voice::{LangType, VoicesTableFile};

#[derive(Debug, Default)]
pub struct AudioTables {
    /// `audio_data`
    pub audio: RawAudioData,
    /// `charword_table`
    pub voices: VoicesTableFile,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawMusic {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub bank: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize)]
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
pub struct AudioSound {
    /// Original logical asset path from `audio_data.json`
    pub asset: String,
    /// Playable URLs (`/audio/sound_beta_2/...`)
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
