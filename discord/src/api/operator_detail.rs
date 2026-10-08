//! `GET /api/operators/{id}`: one operator's full record, for `/collection operator`.
//!
//! The payload mixes the backend's `camelCase` with the game tables' `PascalCase` (`phases`,
//! `talents`, `trait`, the skill cost lists, the summons' skill slots), so each struct names
//! its own casing. Only what the operator pages print is decoded. Shapes and nullability were
//! checked against all 441 EN operators on `api.myrtle.moe` (2026-10-08); every field seen null
//! there is an `Option` here.

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
    /// The operator's summons and deployables, each a full character record of its own.
    #[serde(default)]
    pub drones: Vec<Token>,
    pub handbook: Option<Handbook>,
    pub base_skills: Option<Vec<BaseSkill>>,
}

impl OperatorDetail {
    /// The highest level index any skill has: 9 (M3) for skills with masteries, else 6.
    #[must_use]
    pub fn top_skill_level(&self) -> usize {
        self.skills
            .iter()
            .filter_map(|s| s.static_.as_ref().map(|st| st.levels.len()))
            .max()
            .unwrap_or(0)
            .saturating_sub(1)
    }

    /// The modules that are real upgrades (not the `INITIAL` "original" placeholder).
    pub fn advanced_modules(&self) -> impl Iterator<Item = &Module> {
        self.modules.iter().filter(|m| m.kind == "ADVANCED")
    }
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
    /// The attack range at this promotion, a key of `/api/static/ranges`.
    pub range_id: Option<String>,
    #[serde(default)]
    pub attributes_key_frames: Vec<KeyFrame>,
    /// Promotion cost into this phase, LMD included; null on E0.
    pub evolve_cost: Option<Vec<ItemCost>>,
    /// LMD and EXP to level from 1 to this phase's cap.
    pub level_up_cost: Option<Vec<ItemCost>>,
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

/// An item and how many, in the module and handbook tables' casing.
#[derive(Debug, Clone, Deserialize)]
pub struct ModuleItemCost {
    pub id: String,
    pub count: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub skill_id: String,
    /// The summon this skill deploys, a `drones` id (Ling's three, W's box).
    pub override_token_key: Option<String>,
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
    pub skill_id: String,
    /// Shared icon id, when the skill borrows another's icon.
    pub icon_id: Option<String>,
    /// Levels 1 to 7, then M1 to M3 when the skill has masteries (10 or 7 entries).
    #[serde(default)]
    pub levels: Vec<SkillLevel>,
}

impl SkillStatic {
    /// The id `/api/skill-icon/{id}` serves this skill's icon under.
    #[must_use]
    pub fn icon(&self) -> &str {
        self.icon_id.as_deref().unwrap_or(&self.skill_id)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillLevel {
    pub name: String,
    pub description: Option<String>,
    /// The skill's own attack range, when it replaces the operator's.
    pub range_id: Option<String>,
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
    /// A range the talent grants (Ch'en's, Mostima's); 88 of 1,853 candidates have one.
    pub range_id: Option<String>,
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

impl UnlockCondition {
    /// The promotion as a number: `PHASE_2` -> 2.
    #[must_use]
    pub fn elite(&self) -> usize {
        phase_number(&self.phase)
    }
}

/// `PHASE_2` -> 2; anything unexpected reads as 0.
#[must_use]
pub fn phase_number(phase: &str) -> usize {
    phase
        .strip_prefix("PHASE_")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
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
    pub uni_equip_id: String,
    pub uni_equip_name: String,
    pub type_name1: Option<String>,
    /// `X`, `Y`, `A`, `B`, or `D` (shown as Δ); null on the `INITIAL` (original) entry.
    pub type_name2: Option<String>,
    /// `INITIAL` or `ADVANCED`.
    #[serde(rename = "type")]
    pub kind: String,
    /// `PHASE_2` for every advanced module.
    pub unlock_evolve_phase: String,
    pub unlock_level: u32,
    /// Trust needed, in the game's raw points (not percent).
    pub unlock_favor_point: u32,
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
    /// Stat bonuses for the operator's summons, per summon id.
    #[serde(default)]
    pub token_attribute_blackboard: Vec<TokenBoard>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenBoard {
    /// A `drones` id.
    pub key: String,
    #[serde(default)]
    pub value: Vec<BoardEntry>,
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
    /// A range the module gives the operator.
    pub range_id: Option<String>,
    #[serde(default)]
    pub blackboard: Vec<BoardEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleTalentCandidate {
    pub name: Option<String>,
    pub upgrade_description: Option<String>,
    /// A range the upgraded talent gives the operator.
    pub range_id: Option<String>,
    #[serde(default)]
    pub blackboard: Vec<BoardEntry>,
}

/// A summon or deployable (`drones[]`): Ling's dragons, Phantom's clone, W's box.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub id: String,
    pub name: String,
    /// The token's trait text.
    pub description: Option<String>,
    #[serde(rename = "trait")]
    pub trait_: Option<Trait>,
    #[serde(default)]
    pub phases: Vec<Phase>,
    /// One slot per skill of the owner, in the owner's skill order; `SkillId` is null for a
    /// slot where the token has no skill (53 of 182).
    #[serde(default)]
    pub skills: Vec<TokenSkill>,
    #[serde(default)]
    pub talents: Vec<Talent>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TokenSkill {
    pub skill_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Handbook {
    #[serde(default)]
    pub story_text_audio: Vec<HandbookSection>,
}

/// One archive entry: "Basic Info", "Profile", "Archive File 1", "Promotion Record"...
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookSection {
    pub story_title: String,
    #[serde(default)]
    pub stories: Vec<HandbookStory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandbookStory {
    pub story_text: String,
    /// `DIRECT`, `FAVOR` (trust, `unLockParam` in percent) or `AWAKE` (`"2;1"`: E2 Lv.1).
    pub unlock_type: String,
    #[serde(rename = "unLockParam")]
    pub unlock_param: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseSkill {
    pub buff_name: String,
    pub description: Option<String>,
    /// `MANUFACTURE`, `TRADING`, `CONTROL`, `DORMITORY`, ...
    pub room_type: String,
    /// Icon stem under `textures/spritepack/building_ui_buff_skills_h1_0/`.
    pub skill_icon: Option<String>,
    pub unlock_elite: u32,
    pub unlock_level: u32,
}

/// One attack range from `GET /api/static/ranges`: tiles relative to the operator, who faces
/// right (`col` grows forward, `row` grows up).
#[derive(Debug, Clone, Deserialize)]
pub struct Range {
    #[serde(default)]
    pub grids: Vec<GridCell>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub struct GridCell {
    pub row: i32,
    pub col: i32,
}

/// One skill from `GET /api/static/skills`, the table a summon's skills are looked up in.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSkill {
    pub skill_id: String,
    pub icon_id: Option<String>,
    /// The icon's asset path; null where the game ships no icon (90 of 747 summon skills).
    pub image: Option<String>,
    #[serde(default)]
    pub levels: Vec<SkillLevel>,
}

impl TableSkill {
    /// The id `/api/skill-icon/{id}` serves this skill's icon under.
    #[must_use]
    pub fn icon(&self) -> &str {
        self.icon_id.as_deref().unwrap_or(&self.skill_id)
    }
}

/// `GET /api/skins/{id}`: the operator's outfits, keyed by skin id.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinData {
    #[serde(default)]
    pub char_skins: HashMap<String, Skin>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    /// `char_2023_ling#1` (E0 art), `char_2023_ling#2` (E2 art), `char_2023_ling@nian#9`.
    pub skin_id: String,
    pub avatar_id: Option<String>,
    pub display_skin: Option<DisplaySkin>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplaySkin {
    pub skin_name: Option<String>,
    pub skin_group_name: Option<String>,
    pub drawer_list: Option<Vec<String>>,
    pub designer_list: Option<Vec<String>>,
    /// The outfit's description, with `<color name=...>` markup on 371 of them.
    pub content: Option<String>,
    pub obtain_approach: Option<String>,
    /// Release time, unix seconds; 0 on the default outfits.
    #[serde(default)]
    pub get_time: i64,
}

/// `GET /api/voices/{id}`: voice lines keyed by word id.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceData {
    #[serde(default)]
    pub char_words: HashMap<String, VoiceLine>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceLine {
    /// The voice set: the operator id for the base set, with a skin or dialect suffix for
    /// the others (`char_2023_ling_CN_TOPOLECT`, `char_2023_ling@nian#12`).
    pub word_key: String,
    pub voice_title: String,
    pub voice_text: Option<String>,
    pub voice_index: i64,
    /// `HOME_SHOW`, `BATTLE_SKILL_1`, `BIRTHDAY`, ...
    pub place_type: String,
}

/// One entry of `handbookStageData` in `GET /api/static/handbook`: an operator's Paradox
/// Simulation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParadoxEntry {
    /// The stage's code, `mem_ling_1`; the stage index lists it under this code.
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub unlock_param: Vec<ParadoxUnlock>,
    #[serde(default)]
    pub reward_item: Vec<ModuleItemCost>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParadoxUnlock {
    /// `AWAKE` (promotion) or `FAVOR` (trust).
    pub unlock_type: String,
    pub unlock_param1: Option<String>,
}

/// The parts of `GET /api/stages/{id}/detail` the Paradox page prints.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParadoxStage {
    #[serde(default)]
    pub enemies: HashMap<String, StageEnemy>,
    pub level_data: Option<ParadoxLevel>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageEnemy {
    pub enemy_index: Option<String>,
    pub name: String,
    /// `NORMAL`, `ELITE` or `BOSS`.
    pub enemy_level: Option<String>,
    pub stats: Option<EnemyLevels>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnemyLevels {
    #[serde(default)]
    pub levels: Vec<serde::de::IgnoredAny>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParadoxLevel {
    pub enemy_db_refs: Option<Vec<StageEnemyRef>>,
    pub waves: Option<Vec<Wave>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageEnemyRef {
    pub id: String,
    #[serde(default)]
    pub level: u32,
    /// A level-local variant's own fields. 17 refs across the 296 Paradox stages name an
    /// enemy the detail's `enemies` map lacks; 15 of them carry their name here.
    pub overwritten_data: Option<EnemyOverwrite>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyOverwrite {
    pub name: Option<Overwritten>,
    pub level_type: Option<Overwritten>,
    /// The handbook enemy the variant is built on.
    pub prefab_key: Option<Overwritten>,
}

/// A level file's optional value: used only when `m_defined`.
#[derive(Debug, Clone, Deserialize)]
pub struct Overwritten {
    #[serde(default)]
    pub m_defined: bool,
    pub m_value: Option<serde_json::Value>,
}

impl Overwritten {
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        self.m_defined
            .then(|| self.m_value.as_ref()?.as_str())
            .flatten()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Wave {
    #[serde(default)]
    pub fragments: Vec<Fragment>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fragment {
    #[serde(default)]
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    /// `"SPAWN"` in current level files, `0` in older ones.
    pub action_type: serde_json::Value,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub count: u32,
}

impl Action {
    #[must_use]
    pub fn is_spawn(&self) -> bool {
        self.action_type.as_str() == Some("SPAWN") || self.action_type.as_u64() == Some(0)
    }
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
                "RangeId":"1-1","AttributesKeyFrames":[],"EvolveCost":null,"LevelUpCost":null}],
                "skills":[{"skillId":"s","overrideTokenKey":null,"levelUpCostCond":null,
                "static":null}],
                "talents":[{"Candidates":null}],"potentialRanks":[],"favorKeyFrames":null,
                "allSkillLevelUp":null,"modules":[{"uniEquipId":"uniequip_001_x",
                "uniEquipName":"M","typeName1":"ORIGINAL","typeName2":null,"type":"INITIAL",
                "unlockEvolvePhase":"PHASE_0","unlockLevel":0,"unlockFavorPoint":0,
                "itemCost":null,"data":null}],
                "drones":[{"id":"token_x","name":"T","description":null,"trait":null,
                "phases":[],"skills":[{"SkillId":null}],"talents":[]}],
                "handbook":{"storyTextAudio":[]},"baseSkills":null}"#,
        )
        .unwrap();
        assert!(detail.trait_.is_none() && detail.handbook.is_some());
        assert_eq!(detail.phases[0].max_level, 30);
        assert!(detail.skills[0].static_.is_none());
        assert!(detail.drones[0].skills[0].skill_id.is_none());
        assert_eq!(detail.top_skill_level(), 0);
        assert_eq!(detail.advanced_modules().count(), 0);
    }

    #[test]
    fn spawn_actions_in_both_encodings() {
        let actions: Vec<Action> = serde_json::from_str(
            r#"[{"actionType":"SPAWN","key":"e","count":2},{"actionType":0,"key":"e","count":1},
                {"actionType":"STORY","key":"","count":0},{"actionType":4}]"#,
        )
        .unwrap();
        let spawns: Vec<bool> = actions.iter().map(Action::is_spawn).collect();
        assert_eq!(spawns, vec![true, true, false, false]);
        assert_eq!(phase_number("PHASE_2"), 2);
        assert_eq!(phase_number("junk"), 0);
    }
}
