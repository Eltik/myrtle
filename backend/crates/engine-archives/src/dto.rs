use serde::{Deserialize, Serialize};

use super::profile::{ArchiveGender, ArchiveProfile, ExamAxis, ExamStep, InfectionState};
use myrtle_gamedata_raw::tables::operator::HiddenFaction;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePerson {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// 1..=6, `None` for an NPC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profession: Option<String>,
    pub npc: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveHex {
    pub index: u32,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveForce {
    pub id: String,
    pub name: String,
    pub code: String,
    pub level: i32,
    /// Hex color without `#`
    pub color: String,
    pub card_color: String,
    pub group_id: String,
    pub logo: ArchivePoint,
    pub logo_scale: f64,
    pub card: ArchivePoint,
    pub card_scale: f64,
    pub hexes: Vec<ArchiveHex>,
    pub member_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEdge {
    /// 0..=5
    pub direction: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveCell {
    pub char_id: String,
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub outline: bool,
    pub edges: Vec<ArchiveEdge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveBlock {
    pub force_id: String,
    pub name: String,
    pub color: String,
    pub pos: ArchivePoint,
    pub focus: ArchivePoint,
    pub home: bool,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveFiller {
    pub id: String,
    pub force_id: String,
    pub color: String,
    pub pos: ArchivePoint,
    pub edges: Vec<ArchiveEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLine {
    pub a: String,
    pub b: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
pub struct ArchivePair {
    pub a: String,
    pub b: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveHexLink {
    pub from: u32,
    pub to: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveNetwork {
    /// False when this server ships no network tables
    pub imported: bool,
    pub forces: Vec<ArchiveForce>,
    pub force_links: Vec<ArchivePair>,
    pub hex_links: Vec<ArchiveHexLink>,
    pub groups: Vec<ArchiveGroup>,
    pub people: Vec<ArchivePerson>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDossier {
    pub id: String,
    pub name: String,
    pub rarity: u8,
    pub profession: String,
    pub sub_profession_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_profession_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nation: Option<ArchiveFactionRef>,
    pub affiliations: Vec<ArchiveFactionRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender_key: Option<ArchiveGender>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub race: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthplace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_cm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combat_experience: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combat_years: Option<f64>,
    pub infection_state: InfectionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assimilation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assimilation_pct: Option<f64>,
    /// Stories the operator speaks in
    pub story_count: u32,
    /// Operator Record story sets
    pub record_count: u32,
    /// Characters the network links this operator to
    pub link_count: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDossiers {
    pub dossiers: Vec<ArchiveDossier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveFactionRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveNumber {
    pub display: String,
    pub prefix: String,
    pub holders: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction: Option<ArchiveFactionRef>,
    pub faction_holders: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveRecord {
    pub id: String,
    pub name: String,
}

/// One story an operator speaks in
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveAppearance {
    pub story_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub group_id: String,
    pub group_name: String,
    pub lines: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSceneMate {
    pub char_id: String,
    pub stories: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveChartRef {
    pub group_id: String,
    pub group_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_id: Option<String>,
    pub home: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOperator {
    pub id: String,
    pub name: String,
    pub rarity: u8,
    pub profession: String,
    pub sub_profession_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_profession_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<ArchiveNumber>,
    pub factions: Vec<ArchiveFactionRef>,
    pub hidden_factions: Vec<HiddenFaction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<ArchiveProfile>,
    pub records: Vec<ArchiveRecord>,
    pub appearances: Vec<ArchiveAppearance>,
    pub shares_scenes: Vec<ArchiveSceneMate>,
    pub linked: Vec<String>,
    pub charts: Vec<ArchiveChartRef>,
    pub people: Vec<ArchivePerson>,
    pub exam_steps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveCount {
    pub value: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveBucket {
    pub from: f64,
    pub to: f64,
    pub count: u32,
}

/// One numeric field over every file that states it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDistribution {
    pub counted: u32,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    /// Everyone at `min` by name
    pub low: Vec<String>,
    /// Everyone at `max` by name
    pub high: Vec<String>,
    pub buckets: Vec<ArchiveBucket>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveGroupMean {
    pub value: String,
    pub counted: u32,
    pub mean: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveInfectionRow {
    pub value: String,
    pub total: u32,
    pub infected: u32,
    pub clear: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveScaleStep {
    pub key: ExamStep,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveGenderCount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<ArchiveGender>,
    pub value: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveExamColumn {
    pub axis: ExamAxis,
    pub steps: Vec<u32>,
    pub unrecorded: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDayCount {
    pub month: u8,
    pub day: u8,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveHolder {
    pub char_id: String,
    pub number: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePrefix {
    pub prefix: String,
    pub holders: Vec<ArchiveHolder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction: Option<ArchiveFactionRef>,
    pub faction_holders: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDemographics {
    /// # of operator files
    pub total: u32,
    pub race: Vec<ArchiveCount>,
    pub birthplace: Vec<ArchiveCount>,
    pub gender: Vec<ArchiveGenderCount>,
    /// `InfectionState` names as values
    pub infection: Vec<ArchiveCount>,
    pub height: ArchiveDistribution,
    pub combat: ArchiveDistribution,
    pub blood: ArchiveDistribution,
    pub assimilation: ArchiveDistribution,
    pub height_by_race: Vec<ArchiveGroupMean>,
    pub infection_by_birthplace: Vec<ArchiveInfectionRow>,
    pub infection_by_race: Vec<ArchiveInfectionRow>,
    pub min_group: u32,
    pub scale: Vec<ArchiveScaleStep>,
    pub exam: Vec<ArchiveExamColumn>,
    pub birth_months: Vec<u32>,
    pub birth_days: Vec<ArchiveDayCount>,
    pub dated: u32,
    pub manufactured: Vec<ArchiveCount>,
    pub prefixes: Vec<ArchivePrefix>,
    pub people: Vec<ArchivePerson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSecret {
    pub char_id: String,
    pub shown: ArchiveFactionRef,
    pub hidden: ArchiveFactionRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveShownMember {
    pub char_id: String,
    pub shown: ArchiveFactionRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveNationBehind {
    pub nation: ArchiveFactionRef,
    pub members: Vec<ArchiveShownMember>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveHiddenFactions {
    pub secret: Vec<ArchiveSecret>,
    pub nations: Vec<ArchiveNationBehind>,
    pub people: Vec<ArchivePerson>,
}
