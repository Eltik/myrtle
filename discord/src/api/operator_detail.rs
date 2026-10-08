//! `GET /api/operators/{id}`: one operator's full record, for `/collection operator
//! compact:false`.
//!
//! The payload mixes the backend's `camelCase` with the game tables' `PascalCase` (`phases`,
//! `talents`, `trait`, the skill cost lists), so each struct names its own casing. Only what
//! the full card prints is decoded. Shapes and nullability were checked against all 441 EN
//! operators on `api.myrtle.moe`; every field seen null there is an `Option` here.

use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorDetail {
    pub id: String,
    /// The class trait, with markup and possibly `{...}` templates.
    pub description: Option<String>,
    pub item_usage: Option<String>,
    pub item_desc: Option<String>,
    pub item_obtain_approach: Option<String>,
    /// Blackboard (and override text) for the trait; null for 291 of 441.
    #[serde(rename = "trait")]
    pub trait_: Option<Trait>,
    #[serde(default)]
    pub phases: Vec<Phase>,
    #[serde(default)]
    pub skills: Vec<Skill>,
    pub talents: Option<Vec<Talent>>,
    #[serde(default)]
    pub potential_ranks: Vec<PotentialRank>,
    pub favor_key_frames: Option<Vec<KeyFrame>>,
    /// Skill levels 2 to 7, shared by every skill.
    pub all_skill_level_up: Option<Vec<SkillLevelUp>>,
    #[serde(default)]
    pub modules: Vec<Module>,
    pub profile: Option<Profile>,
    pub base_skills: Option<Vec<BaseSkill>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Trait {
    #[serde(default)]
    pub candidates: Vec<TraitCandidate>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TraitCandidate {
    pub blackboard: Option<Vec<BoardEntryPascal>>,
    pub override_description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BoardEntryPascal {
    pub key: String,
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoardEntry {
    pub key: String,
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Phase {
    pub max_level: u32,
    #[serde(default)]
    pub attributes_key_frames: Vec<KeyFrame>,
    pub evolve_cost: Option<Vec<ItemCost>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyFrame {
    pub level: u32,
    pub data: Attributes,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Attributes {
    pub max_hp: f64,
    pub atk: f64,
    pub def: f64,
    pub magic_resistance: f64,
    pub cost: f64,
    pub block_cnt: f64,
    pub base_attack_time: f64,
    pub attack_speed: f64,
    pub respawn_time: f64,
}

/// An item and how many, in the game tables' casing.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemCost {
    pub id: String,
    pub count: u64,
}

/// An item and how many, in the module tables' casing.
#[derive(Debug, Clone, Deserialize)]
pub struct ModuleItemCost {
    pub id: String,
    pub count: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub skill_id: String,
    /// The three mastery levels' costs; empty for skills without masteries.
    pub level_up_cost_cond: Option<Vec<MasteryCost>>,
    #[serde(rename = "static")]
    pub static_: Option<SkillStatic>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MasteryCost {
    pub level_up_cost: Option<Vec<ItemCost>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SkillStatic {
    /// Levels 1 to 7, then M1 to M3 when the skill has masteries (10 or 7 entries).
    #[serde(default)]
    pub levels: Vec<SkillLevel>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillLevel {
    pub name: String,
    pub description: Option<String>,
    /// `MANUAL`, `AUTO` or `PASSIVE`.
    pub skill_type: String,
    /// `NONE`, or `AMMO` for skills that end when their charges run out.
    pub duration_type: Option<String>,
    pub sp_data: SpData,
    pub duration: f64,
    #[serde(default)]
    pub blackboard: Vec<BoardEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpData {
    /// `INCREASE_WITH_TIME`, `INCREASE_WHEN_ATTACK`, `INCREASE_WHEN_TAKEN_DAMAGE`, or
    /// `UNKNOWN_8` (passive skills, which have no SP).
    pub sp_type: String,
    pub sp_cost: u32,
    pub init_sp: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Talent {
    pub candidates: Option<Vec<TalentCandidate>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TalentCandidate {
    pub unlock_condition: UnlockCondition,
    pub required_potential_rank: u32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub blackboard: Option<Vec<BoardEntryPascal>>,
    #[serde(default)]
    pub is_hide_talent: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UnlockCondition {
    /// `PHASE_0`, `PHASE_1`, `PHASE_2`.
    pub phase: String,
    pub level: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PotentialRank {
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SkillLevelUp {
    pub lvl_up_cost: Option<Vec<ItemCost>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub uni_equip_name: String,
    pub type_name1: Option<String>,
    /// `X`, `Y`, `A`, `B`, or `D` (shown as Δ); null on the `INITIAL` (original) entry.
    pub type_name2: Option<String>,
    /// `INITIAL` or `ADVANCED`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Costs per module stage, keyed "1" to "3".
    pub item_cost: Option<BTreeMap<String, Vec<ModuleItemCost>>>,
    pub data: Option<ModuleData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleData {
    #[serde(default)]
    pub phases: Vec<ModulePhase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePhase {
    pub equip_level: u32,
    #[serde(default)]
    pub parts: Vec<ModulePart>,
    #[serde(default)]
    pub attribute_blackboard: Vec<BoardEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePart {
    pub override_trait_data_bundle: Option<CandidateBundle<ModuleTraitCandidate>>,
    pub add_or_override_talent_data_bundle: Option<CandidateBundle<ModuleTalentCandidate>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CandidateBundle<T> {
    pub candidates: Option<Vec<T>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleTraitCandidate {
    /// Appended to the class trait.
    pub additional_description: Option<String>,
    /// Replaces the class trait outright.
    pub override_description: Option<String>,
    #[serde(default)]
    pub blackboard: Vec<BoardEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleTalentCandidate {
    pub name: Option<String>,
    pub upgrade_description: Option<String>,
    #[serde(default)]
    pub blackboard: Vec<BoardEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    /// `gender`, `placeOfBirth`, `race`, `height`, ...; some values are empty strings.
    pub basic_info: Option<HashMap<String, Option<String>>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseSkill {
    pub buff_name: String,
    pub description: Option<String>,
    /// `MANUFACTURE`, `TRADING`, `CONTROL`, `DORMITORY`, ...
    pub room_type: String,
    pub unlock_elite: u32,
    pub unlock_level: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_nulls_seen_in_real_records() {
        // Every field observed null somewhere across the 441 EN records, null at once.
        let detail: OperatorDetail = serde_json::from_str(
            r#"{"id":"char_x","description":null,"itemUsage":null,"itemDesc":null,
                "itemObtainApproach":null,"trait":null,"phases":[{"MaxLevel":30,
                "AttributesKeyFrames":[],"EvolveCost":null}],
                "skills":[{"skillId":"s","levelUpCostCond":null,"static":null}],
                "talents":[{"Candidates":null}],"potentialRanks":[],"favorKeyFrames":null,
                "allSkillLevelUp":null,"modules":[{"uniEquipName":"M","typeName1":"ORIGINAL",
                "typeName2":null,"type":"INITIAL","itemCost":null,"data":null}],
                "profile":null,"baseSkills":null}"#,
        )
        .unwrap();
        assert!(detail.trait_.is_none() && detail.profile.is_none());
        assert_eq!(detail.phases[0].max_level, 30);
        assert!(detail.skills[0].static_.is_none());
    }
}
