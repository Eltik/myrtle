//! Wire types for `/archives/*`. Every count here is counted from game data;
//! a field the data does not give is absent, never estimated.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::profile::{ArchiveGender, ArchiveProfile, ExamAxis, ExamStep, InfectionState};
use crate::core::gamedata::types::operator::HiddenFaction;

/// Anyone the Archives name: an operator, or an NPC the network places.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchivePerson {
    pub id: String,
    pub name: String,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// 1..=6; `None` for an NPC.
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<u8>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profession: Option<String>,
    pub npc: bool,
    /// The faction the card shows: team, else group, else nation.
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchivePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveHex {
    /// The table's global point index, which `hexLinks` refer to.
    pub index: u32,
    pub x: f64,
    pub y: f64,
}

/// One force (faction) on the territories map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveForce {
    pub id: String,
    pub name: String,
    pub code: String,
    /// `handbook_team_table` level: 0 nation, 1 group, 2 team.
    pub level: i32,
    /// Hex without `#`.
    pub color: String,
    pub card_color: String,
    /// The chart this force opens.
    pub group_id: String,
    pub logo: ArchivePoint,
    pub logo_scale: f64,
    pub card: ArchivePoint,
    pub card_scale: f64,
    pub hexes: Vec<ArchiveHex>,
    /// Members of this force's own block on its chart.
    pub member_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveEdge {
    /// 0..=5; see `core::archives::network` for the geometry.
    pub direction: u8,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_id: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_id: Option<String>,
}

/// One portrait hex on a chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveCell {
    pub char_id: String,
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub outline: bool,
    pub edges: Vec<ArchiveEdge>,
    /// The force block this cell's character is listed in on this chart.
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_id: Option<String>,
}

/// One force's block on a chart: its label, its colour and its roster.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveBlock {
    pub force_id: String,
    pub name: String,
    pub color: String,
    pub pos: ArchivePoint,
    pub focus: ArchivePoint,
    /// False for a guest block, a force that is visiting this chart.
    pub home: bool,
    pub members: Vec<String>,
}

/// A coloured hex with no portrait, which fills out a block's shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveFiller {
    pub id: String,
    pub force_id: String,
    pub color: String,
    pub pos: ArchivePoint,
    pub edges: Vec<ArchiveEdge>,
}

/// A link the game draws between two characters on a chart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveLine {
    pub a: String,
    pub b: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveGroup {
    pub id: String,
    pub name: String,
    pub min_scale: f64,
    pub max_scale: f64,
    pub cells: Vec<ArchiveCell>,
    pub blocks: Vec<ArchiveBlock>,
    pub fillers: Vec<ArchiveFiller>,
    pub lines: Vec<ArchiveLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchivePair {
    pub a: String,
    pub b: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveHexLink {
    pub from: u32,
    pub to: u32,
}

/// The network: territories, every chart and everyone on them.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveNetwork {
    /// False when this server ships no network tables.
    pub imported: bool,
    pub forces: Vec<ArchiveForce>,
    pub force_links: Vec<ArchivePair>,
    pub hex_links: Vec<ArchiveHexLink>,
    pub groups: Vec<ArchiveGroup>,
    pub people: Vec<ArchivePerson>,
}

/// One row of the Dossiers table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveDossier {
    pub id: String,
    pub name: String,
    pub rarity: u8,
    pub profession: String,
    pub sub_profession_id: String,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_profession_name: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction_id: Option<String>,
    /// The operator's nation, and its group and team (in that order), as the
    /// character table lists them; the Dossiers filters read these.
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nation: Option<ArchiveFactionRef>,
    pub affiliations: Vec<ArchiveFactionRef>,
    /// The file's gender word as written, and the gender it reads as.
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender_key: Option<ArchiveGender>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub race: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthplace: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_cm: Option<f64>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combat_experience: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combat_years: Option<f64>,
    pub infection_state: InfectionState,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assimilation: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assimilation_pct: Option<f64>,
    /// Stories the operator speaks in.
    pub story_count: u32,
    /// Operator Record story sets.
    pub record_count: u32,
    /// Characters the network links this operator to.
    pub link_count: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveDossiers {
    pub dossiers: Vec<ArchiveDossier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveFactionRef {
    pub id: String,
    pub name: String,
}

/// A personnel number's letters, and who else carries them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveNumber {
    pub display: String,
    pub prefix: String,
    /// Characters (operators and NPCs) whose number starts with `prefix`.
    pub holders: u32,
    /// The faction most of them show, when one does.
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction: Option<ArchiveFactionRef>,
    /// How many holders show `faction`.
    pub faction_holders: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveRecord {
    pub id: String,
    pub name: String,
}

/// One story an operator speaks in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveAppearance {
    pub story_id: String,
    pub name: String,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub group_id: String,
    pub group_name: String,
    /// Weighted lines attributed to the operator in this story.
    pub lines: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveSceneMate {
    pub char_id: String,
    /// Stories both speak in.
    pub stories: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveChartRef {
    pub group_id: String,
    pub group_name: String,
    /// The block the operator sits in on that chart.
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_id: Option<String>,
    pub home: bool,
}

/// The personnel file drawer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveOperator {
    pub id: String,
    pub name: String,
    pub rarity: u8,
    pub profession: String,
    pub sub_profession_id: String,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_profession_name: Option<String>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<ArchiveNumber>,
    /// What the card shows: nation, group and team, broadest first.
    pub factions: Vec<ArchiveFactionRef>,
    /// `SubPower` factions the card does not show. A spoiler: the page shows
    /// them only with spoilers enabled.
    pub hidden_factions: Vec<HiddenFaction>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<ArchiveProfile>,
    pub records: Vec<ArchiveRecord>,
    pub appearances: Vec<ArchiveAppearance>,
    pub shares_scenes: Vec<ArchiveSceneMate>,
    pub linked: Vec<String>,
    pub charts: Vec<ArchiveChartRef>,
    /// Everyone `sharesScenes` and `linked` name.
    pub people: Vec<ArchivePerson>,
    /// Steps on the exam scale: a rating's `step` runs 0 to this minus one.
    pub exam_steps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveCount {
    pub value: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveBucket {
    pub from: f64,
    pub to: f64,
    pub count: u32,
}

/// One numeric field over every file that states it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveDistribution {
    pub counted: u32,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    /// Everyone at `min`, by name.
    pub low: Vec<String>,
    /// Everyone at `max`, by name.
    pub high: Vec<String>,
    pub buckets: Vec<ArchiveBucket>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveGroupMean {
    pub value: String,
    pub counted: u32,
    pub mean: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveInfectionRow {
    pub value: String,
    pub total: u32,
    pub infected: u32,
    pub clear: u32,
}

/// One step of the exam scale, lowest first: its key and the word this
/// server's files print for it (the most common spelling). `word` is absent
/// when no file on this server gives the step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveScaleStep {
    pub key: ExamStep,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word: Option<String>,
}

/// One gender bar. `key` is set when the word reads as a gender, and
/// `value` is then this server's most common spelling of it (女 and 女性
/// both count under JP's word); otherwise `value` is the file's own word
/// (`Unknown`, 断罪) and `key` is absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveGenderCount {
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<ArchiveGender>,
    pub value: String,
    pub count: u32,
}

/// One exam column: how many files give each scale step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveExamColumn {
    pub axis: ExamAxis,
    /// One count per `scale` step, lowest first.
    pub steps: Vec<u32>,
    /// Files with no single readable rating in this column.
    pub unrecorded: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveDayCount {
    pub month: u8,
    pub day: u8,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveHolder {
    pub char_id: String,
    pub number: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchivePrefix {
    pub prefix: String,
    pub holders: Vec<ArchiveHolder>,
    #[ts(optional)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction: Option<ArchiveFactionRef>,
    pub faction_holders: u32,
}

/// By the Numbers.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveDemographics {
    /// Operator files counted.
    pub total: u32,
    pub race: Vec<ArchiveCount>,
    pub birthplace: Vec<ArchiveCount>,
    pub gender: Vec<ArchiveGenderCount>,
    /// `InfectionState` names as values.
    pub infection: Vec<ArchiveCount>,
    pub height: ArchiveDistribution,
    pub combat: ArchiveDistribution,
    pub blood: ArchiveDistribution,
    pub assimilation: ArchiveDistribution,
    pub height_by_race: Vec<ArchiveGroupMean>,
    pub infection_by_birthplace: Vec<ArchiveInfectionRow>,
    pub infection_by_race: Vec<ArchiveInfectionRow>,
    /// Fewest files behind a per-group figure (height by race, infection by
    /// birthplace or race); smaller groups are left out.
    pub min_group: u32,
    /// The exam's rating steps, lowest first.
    pub scale: Vec<ArchiveScaleStep>,
    pub exam: Vec<ArchiveExamColumn>,
    /// Twelve counts, January first.
    pub birth_months: Vec<u32>,
    /// Exact days given by two or more files, most first.
    pub birth_days: Vec<ArchiveDayCount>,
    /// Files whose birthday reads as a date.
    pub dated: u32,
    pub manufactured: Vec<ArchiveCount>,
    pub prefixes: Vec<ArchivePrefix>,
    /// Everyone the distributions and prefixes name.
    pub people: Vec<ArchivePerson>,
}

/// An operator counted in a faction their card does not show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveSecret {
    pub char_id: String,
    pub shown: ArchiveFactionRef,
    pub hidden: ArchiveFactionRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveShownMember {
    pub char_id: String,
    pub shown: ArchiveFactionRef,
}

/// A nation, and the operators whose card shows a group or team inside it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveNationBehind {
    pub nation: ArchiveFactionRef,
    pub members: Vec<ArchiveShownMember>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveHiddenFactions {
    pub secret: Vec<ArchiveSecret>,
    pub nations: Vec<ArchiveNationBehind>,
    pub people: Vec<ArchivePerson>,
}
