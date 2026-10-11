//! The Squad Builder analysis contract: what the page sends for one squad and
//! what comes back for its five panels and the DPS calculator hand-off.
//!
//! Every label the page shows is keyed, never worded, here: a `CoverageRow` is
//! its `key`, a `ReadinessFlag` is its `kind` plus the numbers. The frontend
//! owns the sentences through its i18n catalog. The exceptions are game-data
//! strings (an operator's talent name, a faction name, a voice actor), which
//! are the server's own text and come through verbatim.
//!
//! Seat indices are 0-based. Seats 0..12 are the twelve squad seats in order;
//! the support seat is `SUPPORT_SEAT` (12).

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

pub use super::policy::{SQUAD_SIZE, SUPPORT_SEAT};

/// One seat as the page has it built. `potential` is 0-based (0 = potential 1),
/// `skill_index` is 0-based, `skill_level` is 1 to the operator's top rank,
/// `mastery` counts on top of the top rank, `module_level` is 1 to the
/// module's rank count when `module_id` is set. Each upper bound is the
/// operator's own (`rules::SkillRules`, `battle_equip_table` phases); the
/// analysis clamps to them.
#[derive(TS, ToSchema)]
#[ts(export)]
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

/// `POST /squads/analyze`. `members` holds up to twelve entries, `None` for an
/// empty seat; the support is analysed for coverage, economy and readiness and
/// left out of synergy-as-giver and themes, the way challenge runs count a
/// squad. `start_dp` is the stage's or the page's number; `deploy_limit` sizes
/// the "cheapest N" sum when known. `dub` is a voice language type
/// (`EN`, `JP`, `CN_MANDARIN`, `KR`, ...) for the Themes voice-actor row;
/// `None` reads the server's `charword_table.DefaultLangType`.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadAnalyzeRequest {
    pub members: Vec<Option<SquadMemberBuild>>,
    pub support: Option<SquadMemberBuild>,
    pub start_dp: i32,
    pub deploy_limit: Option<i32>,
    pub dub: Option<String>,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SquadAnalysis {
    pub coverage: Vec<CoverageGroup>,
    pub economy: Economy,
    pub synergy: Synergy,
    pub readiness: Readiness,
    pub themes: Themes,
    /// One entry per filled seat (support included), in request order.
    pub handoff: Vec<SeatHandoff>,
    /// Seats whose operator's texts have no English reading (an operator the
    /// EN server does not have yet): rules that read trait, skill or talent
    /// wording skip them, and their talents come back as `Unread` items.
    pub unread_seats: Vec<i32>,
}

// ---------------------------------------------------------------- coverage

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageGroupKey {
    GettingOnTheField,
    HoldingTheLine,
    Damage,
    Control,
    Utility,
}

/// The checklist rows, in display order within their group.
#[derive(TS, ToSchema)]
#[ts(export)]
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

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageGroup {
    pub key: CoverageGroupKey,
    pub rows: Vec<CoverageRow>,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageRow {
    pub key: CoverageRowKey,
    pub ops: Vec<CoverageOp>,
}

/// One operator covering a row, with the source that earned it. An operator
/// appears once per row, by its first source in `trait, skill, talent, module,
/// stats, tag` order.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageOp {
    pub seat: i32,
    pub char_id: String,
    pub support: bool,
    pub why: SourceRef,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// The archetype trait (the operator's `description`).
    Trait,
    /// The chosen skill, at the chosen rank.
    Skill,
    /// A talent, at the build's promotion and potential.
    Talent,
    /// A module effect at the chosen module rank.
    Module,
    /// Base stats at the chosen promotion (block count, cost, redeploy).
    Stats,
    /// A recruitment tag on the operator. The fallback when nothing structural
    /// names the capability.
    Tag,
}

/// Where a fact came from, for the chip title and the Economy ladder.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub kind: SourceKind,
    /// Skill, talent or module name; `None` for trait, stats and tag.
    pub name: Option<String>,
    /// 0-based, with `kind == Skill`.
    pub skill_index: Option<i32>,
}

impl SourceRef {
    /// A source with no name: a trait, stats or a tag.
    pub const fn bare(kind: SourceKind) -> Self {
        Self {
            kind,
            name: None,
            skill_index: None,
        }
    }
}

// ----------------------------------------------------------------- economy

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Economy {
    /// The request's `start_dp` plus `start_bonus`.
    pub start_dp: i32,
    pub start_base: i32,
    /// Talents that add DP at the start just by being in the squad.
    pub start_bonus: Vec<EconomyBonus>,
    /// Filled seats, support included, sorted by `cost` ascending then seat.
    pub seats: Vec<EconomySeat>,
    /// Sum of every seat's `cost`.
    pub total: i32,
    /// Sum of the `cheapest_count` cheapest seats.
    pub cheapest: i32,
    pub cheapest_count: i32,
    pub deploy_limit: Option<i32>,
    /// Seats whose `cost` is at or under `start_dp`.
    pub openers: i32,
    pub sources: Vec<DpSource>,
    /// Squad-wide effects that changed a number above (an SP bonus that moved a
    /// skill's `first_ready`, a redeploy cut), so the footer can say so.
    pub counted: Vec<EconomyNote>,
    /// Effects left out of the numbers on purpose, with why.
    pub notes: Vec<EconomyNote>,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyBonus {
    pub seat: i32,
    pub char_id: String,
    pub name: String,
    pub amount: i32,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomySeat {
    pub seat: i32,
    pub char_id: String,
    pub support: bool,
    /// After every cut, rounded the way the game rounds (towards zero).
    pub cost: i32,
    /// The promotion's key-frame cost, before any cut.
    pub base_cost: i32,
    pub cuts: Vec<Cut>,
    /// Seconds after every cut.
    pub respawn: f64,
    pub base_respawn: f64,
    pub respawn_cuts: Vec<Cut>,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutKind {
    /// A potential rank's attribute modifier.
    Potential,
    /// The seat's own talent.
    Talent,
    /// The seat's module attribute blackboard.
    Module,
    /// Another seat's squad talent.
    Squad,
}

/// One change to a cost or a redeploy time. `amount` is in DP or seconds and
/// is negative for a cut; `ratio` is set instead when the source scales
/// (`SilverAsh`'s "all allied units' Redeployment Time -12%" is `ratio: -0.12`).
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cut {
    pub kind: CutKind,
    pub name: Option<String>,
    /// The giver's seat for `kind == Squad`.
    pub from_seat: Option<i32>,
    pub amount: f64,
    pub ratio: Option<f64>,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpSource {
    pub seat: i32,
    pub char_id: String,
    pub support: bool,
    pub source: SourceRef,
    /// DP per activation (skills), per kill (`per_kill`), or per `interval`
    /// seconds while deployed (talents with an interval).
    pub amount: f64,
    pub per_kill: bool,
    /// The trait refunds the full cost on retreat.
    pub refund: bool,
    pub duration: Option<f64>,
    pub interval: Option<f64>,
    pub sp_cost: Option<i32>,
    pub sp_init: Option<i32>,
    /// Seconds from deployment to the first activation for a time-charged
    /// skill, at the build's SP numbers plus any counted squad SP bonus.
    pub first_ready: Option<f64>,
    pub charges_on_attack: bool,
    /// `skillType == AUTO`: fires on its own when charged.
    pub auto: bool,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EconomyNoteKind {
    /// Works once the giver is deployed, so it is not in the starting numbers.
    OnceDeployed,
    /// Depends on something the page cannot see (placement, order, a count).
    Conditional,
    /// Applies to the first deployment only.
    FirstDeploy,
    /// Applies to whoever is deployed next.
    NextOperator,
    /// A squad SP bonus moved a skill's `first_ready`.
    SpBonus,
    /// A squad redeploy cut was applied.
    RedeployCut,
    /// A squad cost cut was applied.
    CostCut,
    /// The trait drains DP while deployed (Merchants).
    Drain,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyNote {
    pub kind: EconomyNoteKind,
    pub seat: i32,
    pub char_id: String,
    pub name: String,
    pub amount: Option<f64>,
}

// ----------------------------------------------------------------- synergy

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Synergy {
    pub items: Vec<SynergyItem>,
    pub counts: SynergyCounts,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SynergyCounts {
    pub check: i32,
    pub squad: i32,
    pub field: i32,
    pub place: i32,
}

/// When a squad effect works.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynergyCond {
    /// From the start of the stage, by being in the squad.
    Squad,
    /// Needs the giver on the field.
    Field,
    /// Needs the giver's skill active.
    Skill,
    /// Depends on placement (range, adjacent tiles).
    Near,
}

/// Which panel section an item lands in.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynergyGroup {
    /// Worth a look: reaches nobody here, or the page cannot tell.
    Check,
    Squad,
    Field,
    Place,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynergyTargetKind {
    Everyone,
    /// `value` is the profession as the game names it (`PIONEER`, ...).
    Class,
    /// `value` is a `handbook_team_table` power id; matched against the
    /// operator's nation, group, team and `SubPower` ids.
    Faction,
    /// `value` is a `GachaTags` tag id (`28`, Robot).
    Tag,
    /// `value` is a skill SP type (`INCREASE_WHEN_ATTACK`, ...): the target is
    /// anyone whose chosen skill charges that way.
    SpType,
    /// `value` is `MELEE` or `RANGED`.
    Position,
    /// Operators whose DP cost is at most `value` (as text).
    CostAtMost,
    /// Whoever is deployed next; no fixed receiver.
    Next,
    /// Named something the page could not resolve; `label` keeps the words.
    Unknown,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynergyTarget {
    pub kind: SynergyTargetKind,
    pub value: Option<String>,
    /// The target in the reader's server's words where its tables name it
    /// (`Class`: the profession tag name; `Faction`: the team table name;
    /// `Tag` and `Position`: the recruitment tag name). Other kinds keep the
    /// English talent's words; the page words those itself from `kind` and
    /// `value`.
    pub label: String,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SynergyReason {
    /// The target is named and nobody else in the squad fits it.
    Nobody,
    /// The target could not be resolved to seats.
    Unknown,
    /// Needs a count of operators in the squad that is not met (`need`).
    NeedsCount,
    /// Reach depends on placement or status, which the page cannot see.
    Unseen,
    /// The talent's text has no English reading yet (see
    /// `SquadAnalysis::unread_seats`), so who it reaches is not known.
    Unread,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyNeed {
    pub count: i32,
    pub have: i32,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyItem {
    pub seat: i32,
    pub char_id: String,
    pub support: bool,
    pub name: String,
    pub source: SourceRef,
    pub cond: SynergyCond,
    pub group: SynergyGroup,
    /// The effect text with rich-text tags kept, for `GameText`.
    pub text: String,
    pub target: SynergyTarget,
    /// Receiving seats. Empty with `to_all` when it reaches everyone else.
    pub to: Vec<i32>,
    pub to_all: bool,
    pub reason: Option<SynergyReason>,
    pub need: Option<SynergyNeed>,
    /// An SP-recovery aura: only the highest of its kind applies.
    pub highest_only: bool,
}

// --------------------------------------------------------------- readiness

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Readiness {
    pub stats: ReadinessStats,
    /// One per filled seat, support included, in seat order.
    pub rows: Vec<ReadinessRow>,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReadinessStats {
    /// Squad seats compared (the support is left out).
    pub seats: i32,
    /// The most common promotion among them, ties to the higher.
    pub usual_phase: i32,
    pub usual_phase_count: i32,
    /// Median level among seats at `usual_phase`.
    pub median_level: Option<i32>,
    /// Seats whose chosen skill is at its top mastery (its `LevelUpCostCond`
    /// count, 3 for every skill that has any on 2026-10-11).
    pub mastery3: i32,
    pub chosen_skills: i32,
    pub modules_on: i32,
    /// Seats whose operator has at least one module to turn on.
    pub modules_possible: i32,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unlock {
    pub phase: i32,
    pub level: i32,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessFlagKind {
    /// Below the squad's usual promotion while the operator can reach it.
    Promotion,
    /// Below the usual promotion's median level by the threshold or more.
    Level,
    /// The chosen skill is not unlocked at this promotion.
    SkillLocked,
    /// The chosen skill rank or mastery needs a promotion or level the build
    /// is not at (`AllSkillLvlup` / `LevelUpCostCond` unlock conditions).
    SkillRankLocked,
    /// The module needs a promotion or level the build is not at.
    ModuleLocked,
    /// A borrowed support is shown but left out of the comparison.
    SupportLeftOut,
}

/// `value` and `reference` carry the numbers the sentence needs: for `Level`
/// the seat's level and the median; for `Promotion` the seat's phase and the
/// usual phase; for the locked kinds the unlock phase and level.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessFlag {
    pub kind: ReadinessFlagKind,
    pub value: Option<i32>,
    pub reference: Option<i32>,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessRow {
    pub seat: i32,
    pub char_id: String,
    pub support: bool,
    pub phase: i32,
    pub level: i32,
    /// The promotion's `maxLevel`.
    pub level_cap: i32,
    /// The highest promotion the rarity allows.
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

// ------------------------------------------------------------------ themes

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Themes {
    pub stats: ThemeStats,
    /// Shared by every squad seat.
    pub labels: Vec<ThemeShare>,
    /// Shared by all but one or two.
    pub close: Vec<ThemeShare>,
    /// Shared by two or more.
    pub shared: Vec<ThemeShare>,
    /// The voice language the `voice` rows used.
    pub dub: String,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeStats {
    pub seats: i32,
    pub total_rarity: i32,
    /// The highest star count any playable obtainable operator has
    /// (`SquadIndex.top_rarity`, 6 on every server on 2026-10-11).
    pub top_rarity: i32,
    /// Seats at `top_rarity`.
    pub top_rarity_seats: i32,
    pub medics: i32,
    pub classes: i32,
    /// How many professions playable operators have (`SquadIndex.professions`).
    pub class_total: i32,
    /// Professions nobody in the squad has, as the game names them.
    pub missing: Vec<String>,
}

#[derive(TS, ToSchema)]
#[ts(export)]
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

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeShare {
    pub kind: ThemeKind,
    pub value: String,
    pub seats: Vec<i32>,
}

// ----------------------------------------------------------------- handoff

/// What the DPS calculator can take from the squad for one seat:
/// the flat buffs its request accepts, from squadmates' talents that reach
/// this seat and work by being in the squad or once deployed. Placement-bound
/// and skill-bound effects, and anything that is not an ATK, flat ATK or ASPD
/// number, are listed in `left_out`.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeatHandoff {
    pub seat: i32,
    pub char_id: String,
    pub buffs: HandoffBuffs,
    pub counted: Vec<HandoffSource>,
    pub left_out: Vec<HandoffLeftOut>,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HandoffBuffs {
    /// ATK as a decimal (0.16 = +16%).
    pub atk: f64,
    pub flat_atk: i32,
    pub aspd: i32,
}

#[derive(TS, ToSchema)]
#[ts(export)]
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

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffLeftOutReason {
    Placement,
    Skill,
    NotAStat,
    Conditional,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffLeftOut {
    pub from_seat: i32,
    pub char_id: String,
    pub name: String,
    pub reason: HandoffLeftOutReason,
}
