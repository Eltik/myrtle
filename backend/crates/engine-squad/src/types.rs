use serde::{Deserialize, Serialize};

pub use myrtle_gamedata::index::squad::types::{SynergyCond, SynergyTargetKind};

pub use crate::policy::{SQUAD_SIZE, SUPPORT_SEAT};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadMemberBuild {
    pub char_id: String,
    pub phase: i32,
    pub level: i32,
    pub potential: i32,
    pub skill_index: i32,
    pub skill_level: i32,
    pub mastery: i32,
    pub module_id: Option<String>,
    pub module_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadAnalyzeRequest {
    pub members: Vec<Option<SquadMemberBuild>>,
    pub support: Option<SquadMemberBuild>,
    pub start_dp: i32,
    pub deploy_limit: Option<i32>,
    pub dub: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SquadAnalysis {
    pub coverage: Vec<CoverageGroup>,
    pub economy: Economy,
    pub synergy: Synergy,
    pub readiness: Readiness,
    pub themes: Themes,
    pub handoff: Vec<SeatHandoff>,
    /// Squad seats whose operator's texts have no English reading
    pub unread_seats: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageGroupKey {
    GettingOnTheField,
    HoldingTheLine,
    Damage,
    Control,
    Utility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageRowKey {
    DpGen,
    Openers,
    FastRedeploy,
    Block3,
    HealSingle,
    HealArea,
    HealElemental,
    Physical,
    Arts,
    TrueElemental,
    AntiAir,
    Multi,
    Cc,
    Shift,
    Debuff,
    Invisible,
    Summons,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageGroup {
    pub key: CoverageGroupKey,
    pub rows: Vec<CoverageRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageRow {
    pub key: CoverageRowKey,
    pub ops: Vec<CoverageOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageOp {
    pub seat: i32,
    pub char_id: String,
    pub support: bool,
    pub why: SourceRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Trait,
    Skill,
    Talent,
    Module,
    Stats,
    Tag,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub kind: SourceKind,
    pub name: Option<String>,
    pub skill_index: Option<i32>,
}

impl SourceRef {
    #[must_use]
    pub const fn bare(kind: SourceKind) -> Self {
        Self {
            kind,
            name: None,
            skill_index: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Economy {
    pub start_dp: i32,
    pub start_base: i32,
    pub start_bonus: Vec<EconomyBonus>,
    pub seats: Vec<EconomySeat>,
    pub total: i32,
    /// Sum of the `cheapest_count` seats
    pub cheapest: i32,
    pub cheapest_count: i32,
    pub deploy_limit: Option<i32>,
    pub dp_lower_limit: i32,
    /// Seats whose `cost` is at or under `start_dp - dp_lower_limit`
    pub openers: i32,
    pub sources: Vec<DpSource>,
    pub counted: Vec<EconomyNote>,
    pub notes: Vec<EconomyNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyBonus {
    pub seat: i32,
    pub char_id: String,
    pub name: String,
    pub amount: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomySeat {
    pub seat: i32,
    pub char_id: String,
    pub support: bool,
    pub cost: i32,
    pub base_cost: i32,
    pub cuts: Vec<Cut>,
    pub respawn: f64,
    pub base_respawn: f64,
    pub respawn_cuts: Vec<Cut>,
    pub first_respawn: Option<f64>,
    pub first_respawn_cuts: Vec<Cut>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutKind {
    Potential,
    Talent,
    Module,
    Trait,
    Squad,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cut {
    pub kind: CutKind,
    pub name: Option<String>,
    pub from_seat: Option<i32>,
    pub amount: f64,
    pub ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpSource {
    pub seat: i32,
    pub char_id: String,
    pub support: bool,
    pub source: SourceRef,
    /// DP per activation, per kill (`per_kill`) or per `interval`
    pub amount: f64,
    pub per_kill: bool,
    pub refund: bool,
    pub duration: Option<f64>,
    pub interval: Option<f64>,
    pub sp_cost: Option<i32>,
    pub sp_init: Option<i32>,
    /// Seconds from deployment to first activation
    pub first_ready: Option<f64>,
    pub charges_on_attack: bool,
    /// `skillType == AUTO`
    pub auto: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EconomyNoteKind {
    OnceDeployed,
    Conditional,
    FirstDeploy,
    NextOperator,
    SpBonus,
    RedeployCut,
    CostCut,
    Drain,
    SkillActive,
    DpFloor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteStat {
    /// DP Cost
    Cost,
    /// Redeploy time
    Respawn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyNote {
    pub kind: EconomyNoteKind,
    pub seat: i32,
    pub char_id: String,
    pub name: String,
    pub amount: Option<f64>,
    pub stat: Option<NoteStat>,
    pub ratio: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Synergy {
    pub items: Vec<SynergyItem>,
    pub counts: SynergyCounts,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SynergyCounts {
    pub check: i32,
    pub squad: i32,
    pub field: i32,
    pub place: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynergyGroup {
    Check,
    Squad,
    Field,
    Place,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynergyTarget {
    pub kind: SynergyTargetKind,
    pub value: Option<String>,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynergyReason {
    Nobody,
    Unknown,
    NeedsCount,
    Unseen,
    Unread,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyNeed {
    pub count: i32,
    pub have: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyItem {
    pub seat: i32,
    pub char_id: String,
    pub support: bool,
    pub name: String,
    pub source: SourceRef,
    pub cond: SynergyCond,
    pub group: SynergyGroup,
    pub text: String,
    pub target: SynergyTarget,
    pub to: Vec<i32>,
    pub to_all: bool,
    pub reason: Option<SynergyReason>,
    pub need: Option<SynergyNeed>,
    pub highest_only: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Readiness {
    pub stats: ReadinessStats,
    pub rows: Vec<ReadinessRow>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReadinessStats {
    pub seats: i32,
    pub usual_phase: i32,
    pub usual_phase_count: i32,
    pub median_level: Option<i32>,
    pub mastery3: i32,
    pub chosen_skills: i32,
    pub modules_on: i32,
    pub modules_possible: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unlock {
    pub phase: i32,
    pub level: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessFlagKind {
    Promotion,
    Level,
    SkillLocked,
    SkillRankLocked,
    ModuleLocked,
    SupportLeftOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessFlag {
    pub kind: ReadinessFlagKind,
    pub value: Option<i32>,
    pub reference: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessRow {
    pub seat: i32,
    pub char_id: String,
    pub support: bool,
    pub phase: i32,
    pub level: i32,
    /// Promotion's `maxLevel`
    pub level_cap: i32,
    /// Highest promotion based on rarity
    pub max_phase: i32,
    pub potential: i32,
    pub skill_index: i32,
    pub skill_level: i32,
    pub mastery: i32,
    pub skill_unlock: Option<Unlock>,
    pub module_id: Option<String>,
    pub module_level: i32,
    pub module_unlock: Option<Unlock>,
    pub flags: Vec<ReadinessFlag>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Themes {
    pub stats: ThemeStats,
    pub labels: Vec<ThemeShare>,
    pub close: Vec<ThemeShare>,
    pub shared: Vec<ThemeShare>,
    pub dub: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeStats {
    pub seats: i32,
    pub total_rarity: i32,
    pub top_rarity: i32,
    pub top_rarity_seats: i32,
    pub medics: i32,
    pub classes: i32,
    pub class_total: i32,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeKind {
    Gender,
    Race,
    Birthplace,
    Faction,
    Class,
    Branch,
    Position,
    Rarity,
    Artist,
    Voice,
    Birthday,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeShare {
    pub kind: ThemeKind,
    pub value: String,
    pub seats: Vec<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeatHandoff {
    pub seat: i32,
    pub char_id: String,
    pub buffs: HandoffBuffs,
    pub counted: Vec<HandoffSource>,
    pub left_out: Vec<HandoffLeftOut>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HandoffBuffs {
    pub atk: f64,
    pub flat_atk: i32,
    pub aspd: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffSource {
    pub from_seat: i32,
    pub char_id: String,
    pub name: String,
    pub cond: SynergyCond,
    pub atk: f64,
    pub flat_atk: i32,
    pub aspd: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffLeftOutReason {
    Placement,
    Skill,
    NotAStat,
    Conditional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffLeftOut {
    pub from_seat: i32,
    pub char_id: String,
    pub name: String,
    pub reason: HandoffLeftOutReason,
}
