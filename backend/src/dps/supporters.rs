//! Supporter presets (docs/dps-refactor.md, phase 2): a curated, code-reviewed
//! table of supporter skills and talents mapped onto buff zones.
//!
//! The table names WHERE a number lives (character, skill or talent, the
//! blackboard key) and WHICH zone it feeds; it never carries the number. Every
//! value is read from the loaded game data at the requested investment
//! (promotion, skill level, potential, module level), so a balance patch moves
//! the presets with no code change.
//!
//! Two conventions, each a modelling choice rather than a measurement:
//! - Inspiration (zone C) is `ratio * supporter ATK`, the supporter's ATK at
//!   max level of the requested promotion plus max trust, the potential ATK
//!   bonus and the module's ATK attribute. The supporter's own percentage
//!   buffs are NOT in it (a TRADE: most of them are conditional on the field).
//! - `damage_scale` keys are multipliers (1.33); the zone takes the fragility
//!   (0.33). A key the game already stores as the fragility (`debuff.damage_scale`
//!   0.3 on Saileach S3) is read as is.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::core::gamedata::types::module::{ModuleTarget, ModuleType};
use crate::core::gamedata::types::operator::{
    Operator, OperatorModule, OperatorPhase, OperatorRarity,
};

use super::operator_data::{OperatorData, uniequip_number};

/// Where a contribution lands. Names follow the request's `zones` fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SupporterZone {
    /// `zones.atkPercent`
    AtkPercent,
    /// `zones.inspire`
    Inspire,
    /// `zones.aspd`
    Aspd,
    /// `zones.spRecovery`
    SpRecovery,
    /// `zones.spAura`
    SpAura,
    /// `zones.fragile`
    Fragile,
    /// `zones.artsFragile`
    ArtsFragile,
    /// `zones.def.percent`
    DefPercent,
    /// `zones.def.flat`
    DefFlat,
    /// `zones.res.percent`
    ResPercent,
    /// `zones.res.flat`
    ResFlat,
}

/// How sources inside one zone combine, from `dps::zones`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Stacking {
    /// Summed with every other source of the zone.
    Additive,
    /// Only the largest source of the zone applies.
    Max,
}

impl SupporterZone {
    pub const fn stacking(self) -> Stacking {
        match self {
            Self::Inspire | Self::SpAura | Self::Fragile | Self::ArtsFragile => Stacking::Max,
            Self::AtkPercent
            | Self::Aspd
            | Self::SpRecovery
            | Self::DefPercent
            | Self::DefFlat
            | Self::ResPercent
            | Self::ResFlat => Stacking::Additive,
        }
    }
}

/// Blackboard value -> zone value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Transform {
    /// The value as stored (`atk` 0.9 -> +90% ATK).
    Value,
    /// Sign flipped: a debuff stored negative (`def` -0.6 -> 60% DEF reduction).
    Negate,
    /// A damage multiplier to its fragility (`damage_scale` 1.33 -> 0.33).
    MinusOne,
    /// A ratio of the supporter's ATK (Inspiration).
    TimesAtk,
}

/// Who receives the effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SupporterTarget {
    /// Every ally on the field (auras).
    AllAllies,
    /// Allies inside the supporter's range.
    AlliesInRange,
    /// One ally, chosen by the rule in `condition`.
    OneAlly,
    /// Enemies inside the range of the effect (debuffs).
    Enemies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// 1-based skill number.
    Skill(u8),
    /// 1-based talent number.
    Talent(u8),
}

pub struct ContributionDef {
    pub zone: SupporterZone,
    pub key: &'static str,
    pub transform: Transform,
}

pub struct SupporterDef {
    pub id: &'static str,
    pub char_id: &'static str,
    pub source: Source,
    /// 1-based position in the operator's uniequip-ordered advanced modules
    /// whose talent override applies at a module level above 0; 0 = none.
    pub module: u8,
    pub contributions: &'static [ContributionDef],
    pub target: SupporterTarget,
    /// Game profession codes the effect is limited to; empty = every class.
    pub professions: &'static [&'static str],
    /// What must hold on the field for the effect to apply, in plain words.
    pub condition: Option<&'static str>,
    /// Blackboard key holding the effect's duration. `None`: the skill's own
    /// duration when it has one.
    pub duration_key: Option<&'static str>,
}

const fn c(zone: SupporterZone, key: &'static str, transform: Transform) -> ContributionDef {
    ContributionDef {
        zone,
        key,
        transform,
    }
}

use SupporterTarget::{AllAllies, AlliesInRange, Enemies, OneAlly};
use SupporterZone as Z;
use Transform as T;

/// The table. Every key was checked to exist in EN game data on 2026-10-24
/// (`supporters_resolve` in `tests/dps_phase2_test.rs` keeps it so).
///
/// Considered and left out (docs/dps-refactor.md, Phase 2 results): Ascalon
/// (talent is a damage-over-time she deals, no ally or enemy modifier),
/// Kal'tsit (every buff goes to Mon3tr only), Ebenholz (no ally effect),
/// Steward (talent buffs his own ATK), Laurentina (Specter the Unchained in
/// EN, no ally modifier), Warfarin T1 and Saria T2 (SP on an event, not a
/// rate), Shamare S1 and Suzuran S3 (they scale a talent's Fragile: one
/// blackboard times another, which a one-key contribution cannot carry),
/// Logos T2 `atk_addition` (flat Arts damage per hit, zone E, not buildable).
pub static SUPPORTERS: &[SupporterDef] = &[
    SupporterDef {
        id: "warfarin_s2",
        char_id: "char_171_bldsk",
        source: Source::Skill(2),
        module: 0,
        contributions: &[c(Z::AtkPercent, "atk", T::Value)],
        target: OneAlly,
        professions: &[],
        condition: Some("Warfarin and one random ally in range"),
        duration_key: Some("duration"),
    },
    SupporterDef {
        id: "aak_s3",
        char_id: "char_225_haak",
        source: Source::Skill(3),
        module: 0,
        contributions: &[
            c(Z::AtkPercent, "atk", T::Value),
            c(Z::Aspd, "attack_speed", T::Value),
        ],
        target: OneAlly,
        professions: &[],
        condition: Some("the ally Aak injects, front tile first"),
        duration_key: None,
    },
    SupporterDef {
        id: "sora_s2",
        char_id: "char_101_sora",
        source: Source::Skill(2),
        module: 0,
        contributions: &[c(Z::Inspire, "atk", T::TimesAtk)],
        target: AlliesInRange,
        professions: &[],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "heidi_s1",
        char_id: "char_4045_heidi",
        source: Source::Skill(1),
        module: 0,
        contributions: &[c(Z::Inspire, "atk", T::TimesAtk)],
        target: AlliesInRange,
        professions: &[],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "heidi_t1",
        char_id: "char_4045_heidi",
        source: Source::Talent(1),
        module: 0,
        contributions: &[c(Z::AtkPercent, "atk", T::Value)],
        target: AlliesInRange,
        professions: &[],
        condition: Some("Heidi has Skill 1 equipped"),
        duration_key: None,
    },
    SupporterDef {
        id: "heidi_t2",
        char_id: "char_4045_heidi",
        source: Source::Talent(2),
        module: 0,
        contributions: &[c(Z::SpRecovery, "sp_recovery_per_sec", T::Value)],
        target: AlliesInRange,
        professions: &[],
        condition: Some("allies in range while in Shatterpoint"),
        duration_key: None,
    },
    SupporterDef {
        id: "skadi2_s2",
        char_id: "char_1012_skadi2",
        source: Source::Skill(2),
        module: 0,
        contributions: &[c(Z::Inspire, "atk", T::TimesAtk)],
        target: AlliesInRange,
        professions: &[],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "skadi2_s3",
        char_id: "char_1012_skadi2",
        source: Source::Skill(3),
        module: 0,
        contributions: &[c(Z::Inspire, "atk", T::TimesAtk)],
        target: AlliesInRange,
        professions: &[],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "uika_s1",
        char_id: "char_4184_dolris",
        source: Source::Skill(1),
        module: 0,
        contributions: &[c(Z::Inspire, "attack@atk", T::TimesAtk)],
        target: AlliesInRange,
        professions: &[],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "civilight_s2",
        char_id: "char_4134_cetsyr",
        source: Source::Skill(2),
        module: 0,
        contributions: &[c(Z::Inspire, "attack@atk", T::TimesAtk)],
        target: AlliesInRange,
        professions: &[],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "virtuosa_s3",
        char_id: "char_245_cello",
        source: Source::Skill(3),
        module: 0,
        contributions: &[c(Z::AtkPercent, "cello_s_3[atk].atk", T::Value)],
        target: OneAlly,
        professions: &[],
        condition: Some("the other Operator in range with the highest ATK"),
        duration_key: None,
    },
    SupporterDef {
        id: "pallas_t1",
        char_id: "char_485_pallas",
        source: Source::Talent(1),
        module: 0,
        contributions: &[c(Z::AtkPercent, "peak_performance.atk", T::Value)],
        target: AllAllies,
        professions: &[],
        condition: Some("Minoan Operators, Vigor: HP above peak_performance.hp_ratio"),
        duration_key: None,
    },
    SupporterDef {
        id: "proviso_t1",
        char_id: "char_4032_provs",
        source: Source::Talent(1),
        module: 0,
        contributions: &[c(Z::Aspd, "attack_speed", T::Value)],
        target: OneAlly,
        professions: &[],
        condition: Some("one random Kazimierz Operator"),
        duration_key: None,
    },
    SupporterDef {
        id: "shu_t2_atk",
        char_id: "char_2025_shu",
        source: Source::Talent(2),
        module: 0,
        contributions: &[c(Z::AtkPercent, "atk", T::Value)],
        target: AllAllies,
        professions: &[],
        condition: Some("at least 4 Sui Operators in the squad"),
        duration_key: None,
    },
    SupporterDef {
        id: "shu_t2_aspd",
        char_id: "char_2025_shu",
        source: Source::Talent(2),
        module: 0,
        contributions: &[c(Z::Aspd, "attack_speed", T::Value)],
        target: AllAllies,
        professions: &[],
        condition: Some("at least 3 Operators of one class on the field"),
        duration_key: None,
    },
    SupporterDef {
        id: "shu_s3",
        char_id: "char_2025_shu",
        source: Source::Skill(3),
        module: 0,
        contributions: &[
            c(Z::AtkPercent, "e_atk", T::Value),
            c(Z::Aspd, "e_attack_speed", T::Value),
        ],
        target: AlliesInRange,
        professions: &[],
        condition: Some("ground enemies on sowed tiles"),
        duration_key: None,
    },
    SupporterDef {
        id: "ptilopsis_t1",
        char_id: "char_128_plosis",
        source: Source::Talent(1),
        module: 1,
        contributions: &[c(Z::SpAura, "sp_recovery_per_sec", T::Value)],
        target: AllAllies,
        professions: &[],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "suzuran_t1",
        char_id: "char_358_lisa",
        source: Source::Talent(1),
        module: 1,
        contributions: &[c(Z::SpAura, "sp_recovery_per_sec", T::Value)],
        target: AllAllies,
        professions: &["SUPPORT"],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "mostima_t1",
        char_id: "char_213_mostma",
        source: Source::Talent(1),
        module: 0,
        contributions: &[c(Z::SpAura, "sp_recovery_per_sec", T::Value)],
        target: AllAllies,
        professions: &["CASTER"],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "suzuran_t2",
        char_id: "char_358_lisa",
        source: Source::Talent(2),
        module: 2,
        contributions: &[c(Z::Fragile, "damage_scale", T::MinusOne)],
        target: Enemies,
        professions: &[],
        condition: Some("Slowed enemies in range"),
        duration_key: None,
    },
    SupporterDef {
        id: "pramanix_t1",
        char_id: "char_174_slbell",
        source: Source::Talent(1),
        module: 0,
        contributions: &[c(Z::Fragile, "damage_scale", T::MinusOne)],
        target: Enemies,
        professions: &[],
        condition: Some("enemies in range below hp_ratio of their max HP"),
        duration_key: None,
    },
    SupporterDef {
        id: "shamare_t1",
        char_id: "char_254_vodfox",
        source: Source::Talent(1),
        module: 0,
        contributions: &[c(Z::Fragile, "damage_scale", T::MinusOne)],
        target: Enemies,
        professions: &[],
        condition: Some("enemies in range below hp_ratio of their max HP"),
        duration_key: None,
    },
    SupporterDef {
        id: "gnosis_t1_cold",
        char_id: "char_206_gnosis",
        source: Source::Talent(1),
        module: 1,
        contributions: &[c(Z::Fragile, "damage_scale_cold", T::MinusOne)],
        target: Enemies,
        professions: &[],
        condition: Some("Cold enemies in range"),
        duration_key: None,
    },
    SupporterDef {
        id: "gnosis_t1_frozen",
        char_id: "char_206_gnosis",
        source: Source::Talent(1),
        module: 1,
        contributions: &[c(Z::Fragile, "damage_scale_freeze", T::MinusOne)],
        target: Enemies,
        professions: &[],
        condition: Some("Frozen enemies in range"),
        duration_key: None,
    },
    SupporterDef {
        id: "lucilla_t1",
        char_id: "char_4079_haini",
        source: Source::Talent(1),
        module: 1,
        contributions: &[c(Z::Fragile, "damage_scale", T::MinusOne)],
        target: Enemies,
        professions: &[],
        condition: Some("enemies in range that are not Elite or Leader"),
        duration_key: None,
    },
    SupporterDef {
        id: "astgenne_t2",
        char_id: "char_1047_halo2",
        source: Source::Talent(2),
        module: 0,
        contributions: &[c(Z::Fragile, "halo2_t_1[weak].damage_scale", T::MinusOne)],
        target: Enemies,
        professions: &[],
        condition: Some("enemies in range, before the ramp"),
        duration_key: None,
    },
    SupporterDef {
        id: "astgenne_t2_max",
        char_id: "char_1047_halo2",
        source: Source::Talent(2),
        module: 0,
        contributions: &[c(
            Z::Fragile,
            "halo2_t_1[weak].damage_scale_max",
            T::MinusOne,
        )],
        target: Enemies,
        professions: &[],
        condition: Some("enemies in range for halo2_t_1[weak].interval seconds"),
        duration_key: None,
    },
    SupporterDef {
        id: "raidian_s3",
        char_id: "char_614_acsupo",
        source: Source::Skill(3),
        module: 0,
        contributions: &[c(Z::Fragile, "damage_scale", T::MinusOne)],
        target: Enemies,
        professions: &[],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "saileach_s3",
        char_id: "char_479_sleach",
        source: Source::Skill(3),
        module: 0,
        contributions: &[c(Z::Fragile, "debuff.damage_scale", T::Value)],
        target: Enemies,
        professions: &[],
        condition: Some("enemies around the banner"),
        duration_key: None,
    },
    SupporterDef {
        id: "ela_s3",
        char_id: "char_4123_ela",
        source: Source::Skill(3),
        module: 0,
        contributions: &[c(Z::Fragile, "damage_scale", T::MinusOne)],
        target: Enemies,
        professions: &[],
        condition: Some("enemies around a triggered trap (passive)"),
        duration_key: Some("sluggish"),
    },
    SupporterDef {
        id: "saria_s3",
        char_id: "char_202_demkni",
        source: Source::Skill(3),
        module: 0,
        contributions: &[c(Z::ArtsFragile, "demkni_s_3.damage_scale", T::MinusOne)],
        target: Enemies,
        professions: &[],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "skyfire_t1",
        char_id: "char_166_skfire",
        source: Source::Talent(1),
        module: 1,
        contributions: &[c(Z::ArtsFragile, "damage_scale", T::MinusOne)],
        target: Enemies,
        professions: &[],
        condition: Some("blocked enemies"),
        duration_key: None,
    },
    SupporterDef {
        id: "pramanix_s2",
        char_id: "char_174_slbell",
        source: Source::Skill(2),
        module: 0,
        contributions: &[
            c(Z::DefPercent, "def", T::Negate),
            c(Z::ResPercent, "magic_resistance", T::Negate),
        ],
        target: Enemies,
        professions: &[],
        condition: None,
        duration_key: None,
    },
    SupporterDef {
        id: "shamare_s2",
        char_id: "char_254_vodfox",
        source: Source::Skill(2),
        module: 0,
        contributions: &[c(Z::DefPercent, "def", T::Negate)],
        target: Enemies,
        professions: &[],
        condition: Some("enemies around the Curse Doll"),
        duration_key: None,
    },
    SupporterDef {
        id: "meteor_s1",
        char_id: "char_126_shotst",
        source: Source::Skill(1),
        module: 0,
        contributions: &[c(Z::DefPercent, "def", T::Negate)],
        target: Enemies,
        professions: &[],
        condition: Some("the target of the empowered attack"),
        duration_key: Some("duration"),
    },
    SupporterDef {
        id: "meteorite_s2",
        char_id: "char_219_meteo",
        source: Source::Skill(2),
        module: 0,
        contributions: &[c(Z::DefFlat, "def", T::Negate)],
        target: Enemies,
        professions: &[],
        condition: None,
        duration_key: Some("duration"),
    },
    SupporterDef {
        id: "ifrit_s2",
        char_id: "char_134_ifrit",
        source: Source::Skill(2),
        module: 0,
        contributions: &[c(Z::DefFlat, "def", T::Negate)],
        target: Enemies,
        professions: &[],
        condition: Some("targets of the empowered attack"),
        duration_key: Some("duration"),
    },
    SupporterDef {
        id: "eyjafjalla_s2",
        char_id: "char_180_amgoat",
        source: Source::Skill(2),
        module: 0,
        contributions: &[c(Z::ResPercent, "magic_resistance", T::Negate)],
        target: Enemies,
        professions: &[],
        condition: Some("enemies around the target of the empowered attack"),
        duration_key: Some("duration"),
    },
    SupporterDef {
        id: "logos_t2",
        char_id: "char_4133_logos",
        source: Source::Talent(2),
        module: 0,
        contributions: &[c(Z::ResFlat, "magic_resistance", T::Negate)],
        target: Enemies,
        professions: &[],
        condition: Some("targets of Logos' attacks"),
        duration_key: Some("duration"),
    },
];

/// The investment every supporter is resolved at. Level is always the
/// promotion's max level and trust is always max.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SupporterInvestment {
    /// 0-2, clamped to the operator's highest promotion.
    pub promotion: i32,
    /// 1-10 (8-10 = M1-M3), clamped to the promotion's highest rank.
    pub skill_level: i32,
    /// 1-6.
    pub potential: i32,
    /// 0 = no module, 1-3.
    pub module_level: i32,
}

impl Default for SupporterInvestment {
    fn default() -> Self {
        Self {
            promotion: 2,
            skill_level: 10,
            potential: 1,
            module_level: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ZoneContribution {
    pub zone: SupporterZone,
    pub stacking: Stacking,
    /// The blackboard key the value is read from.
    pub key: String,
    pub transform: Transform,
    /// The blackboard value at the resolved investment.
    pub raw: f64,
    /// What goes into the zone (fractions for percentages, flat for flat).
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SupporterScope {
    pub target: SupporterTarget,
    /// Game profession codes (`SUPPORT`, `CASTER`, ...); empty = every class.
    pub professions: Vec<String>,
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SupporterEntry {
    pub id: String,
    pub char_id: String,
    /// The operator's name in the loaded game data.
    pub name: String,
    /// `skill` or `talent`.
    pub source_kind: String,
    /// 1-based skill or talent number.
    pub source_index: i32,
    /// The skill or talent name in the loaded game data.
    pub source_name: String,
    pub contributions: Vec<ZoneContribution>,
    /// Seconds, when the effect has a duration.
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    pub scope: SupporterScope,
    /// The ATK an Inspiration ratio multiplies. Present for Inspiration only.
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supporter_atk: Option<f64>,
    /// The module whose talent override was applied, when one was.
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_id: Option<String>,
    /// The investment after clamping to what the operator has.
    pub investment: SupporterInvestment,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UnresolvedSupporter {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SupportersResponse {
    pub entries: Vec<SupporterEntry>,
    /// Table entries the loaded game data cannot resolve at this investment.
    pub unresolved: Vec<UnresolvedSupporter>,
}

const fn max_promotion(rarity: &OperatorRarity) -> i32 {
    match rarity {
        OperatorRarity::OneStar | OperatorRarity::TwoStar => 0,
        OperatorRarity::ThreeStar => 1,
        _ => 2,
    }
}

const fn phase_number(phase: &OperatorPhase) -> i32 {
    match phase {
        OperatorPhase::Elite0 => 0,
        OperatorPhase::Elite1 => 1,
        OperatorPhase::Elite2 => 2,
    }
}

/// `PHASE_2` -> 2, the module tables' string form.
fn phase_str_number(phase: &str) -> i32 {
    phase
        .strip_prefix("PHASE_")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

/// A resolved blackboard: (key, value) pairs plus the source's name.
struct Resolved {
    name: String,
    board: Vec<(String, f64)>,
    duration: Option<f64>,
    module_id: Option<String>,
}

fn advanced_modules(operator: &Operator) -> Vec<&OperatorModule> {
    let mut mods: Vec<_> = operator
        .modules
        .iter()
        .filter(|m| m.module.module_type == ModuleType::Advanced)
        .collect();
    mods.sort_by_key(|m| uniequip_number(m));
    mods
}

fn resolve_skill(operator: &Operator, n: u8, inv: SupporterInvestment) -> Result<Resolved, String> {
    let skill = operator
        .skills
        .get(usize::from(n) - 1)
        .ok_or_else(|| format!("no skill {n}"))?;
    if let Some(unlock) = &skill.unlock_cond
        && phase_number(&unlock.phase) > inv.promotion
    {
        return Err(format!(
            "skill {n} is locked at promotion {}",
            inv.promotion
        ));
    }
    let levels = &skill
        .static_data
        .as_ref()
        .ok_or_else(|| format!("skill {n} has no static data"))?
        .levels;
    let level = levels
        .get((inv.skill_level as usize).saturating_sub(1))
        .or_else(|| levels.last())
        .ok_or_else(|| format!("skill {n} has no levels"))?;
    Ok(Resolved {
        name: level.name.clone(),
        board: level
            .blackboard
            .iter()
            .map(|b| (b.key.clone(), b.value))
            .collect(),
        duration: (level.duration > 0.0).then_some(level.duration),
        module_id: None,
    })
}

fn resolve_talent(
    operator: &Operator,
    n: u8,
    module_pos: u8,
    inv: SupporterInvestment,
    needed: &[&str],
) -> Result<Resolved, String> {
    let index = usize::from(n) - 1;
    let pot_rank = inv.potential - 1;
    let base = operator
        .talents
        .get(index)
        .and_then(|t| {
            t.candidates.iter().rev().find(|c| {
                phase_number(&c.unlock_condition.phase) <= inv.promotion
                    && c.required_potential_rank <= pot_rank
            })
        })
        .ok_or_else(|| format!("talent {n} is locked at this investment"))?;
    let mut resolved = Resolved {
        name: base.name.clone().unwrap_or_default(),
        board: base
            .blackboard
            .iter()
            .map(|b| (b.key.clone(), b.value))
            .collect(),
        duration: None,
        module_id: None,
    };

    if module_pos == 0 || inv.module_level == 0 || inv.promotion < 2 {
        return Ok(resolved);
    }
    let modules = advanced_modules(operator);
    let Some(module) = modules.get(usize::from(module_pos) - 1) else {
        return Ok(resolved);
    };
    let Some(phase) = module
        .data
        .phases
        .iter()
        .find(|p| p.equip_level == inv.module_level)
    else {
        return Ok(resolved);
    };
    let candidates: Vec<_> = phase
        .parts
        .iter()
        .filter(|p| p.target == ModuleTarget::Talent || p.target == ModuleTarget::TalentDataOnly)
        .flat_map(|p| {
            p.add_or_override_talent_data_bundle
                .candidates
                .as_deref()
                .unwrap_or(&[])
        })
        .filter(|c| {
            c.talent_index == i32::from(n) - 1
                && c.required_potential_rank <= pot_rank
                && phase_str_number(&c.unlock_condition.phase) <= inv.promotion
        })
        .collect();
    // The override for the base talent shares its prefab key; a module that
    // adds candidates under other keys (Ptilopsis `11`, Gnosis `10_root`) is
    // taken only when it is the one carrying every key the entry reads.
    let pick = candidates
        .iter()
        .rev()
        .find(|c| c.prefab_key.is_some() && c.prefab_key == base.prefab_key)
        .or_else(|| {
            candidates.iter().rev().find(|c| {
                needed
                    .iter()
                    .all(|k| c.blackboard.iter().any(|b| b.key == *k))
            })
        });
    if let Some(over) = pick {
        resolved.board = over
            .blackboard
            .iter()
            .map(|b| (b.key.clone(), b.value))
            .collect();
        resolved.module_id = Some(module.module.uni_equip_id.clone());
    }
    Ok(resolved)
}

/// The supporter's ATK for Inspiration: max level of the promotion, max trust,
/// the potential ATK bonus, the module's ATK attribute at the module level.
fn supporter_atk(data: &OperatorData, inv: SupporterInvestment, module_id: Option<&str>) -> f64 {
    let base = match inv.promotion {
        0 => data.atk.e0.max,
        1 => data.atk.e1.max,
        _ => data.atk.e2.max,
    };
    let potential = if data.atk_potential.required_potential > 0
        && inv.potential >= data.atk_potential.required_potential
    {
        data.atk_potential.value
    } else {
        0.0
    };
    let module = module_id.map_or(0.0, |id| {
        data.atk_module
            .iter()
            .filter(|m| m.module_id == id && m.level == f64::from(inv.module_level))
            .map(|m| m.value as f64)
            .sum()
    });
    f64::from(base) + data.atk_trust + potential + module
}

/// Resolve one table entry against one operator's data.
pub fn resolve(
    def: &SupporterDef,
    data: &OperatorData,
    requested: SupporterInvestment,
) -> Result<SupporterEntry, String> {
    let operator = &data.data;
    let promotion = requested
        .promotion
        .clamp(0, max_promotion(&operator.rarity));
    let inv = SupporterInvestment {
        promotion,
        skill_level: requested
            .skill_level
            .clamp(1, [4, 7, 10][promotion as usize]),
        potential: requested.potential.clamp(1, 6),
        module_level: if promotion == 2 {
            requested.module_level.clamp(0, 3)
        } else {
            0
        },
    };
    let needed: Vec<&str> = def.contributions.iter().map(|c| c.key).collect();
    let (kind, index, resolved) = match def.source {
        Source::Skill(n) => ("skill", n, resolve_skill(operator, n, inv)?),
        Source::Talent(n) => (
            "talent",
            n,
            resolve_talent(operator, n, def.module, inv, &needed)?,
        ),
    };
    let lookup = |key: &str| {
        resolved
            .board
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| *v)
    };

    // An override module's ATK attribute only counts when the module itself is
    // the one applied; for Inspiration the entry's module is used regardless.
    let atk_module = resolved.module_id.clone().or_else(|| {
        (def.module > 0 && inv.module_level > 0)
            .then(|| {
                advanced_modules(operator)
                    .get(usize::from(def.module) - 1)
                    .map(|m| m.module.uni_equip_id.clone())
            })
            .flatten()
    });
    let mut supporter_atk_value = None;
    let mut contributions = Vec::with_capacity(def.contributions.len());
    for cdef in def.contributions {
        let raw = lookup(cdef.key)
            .ok_or_else(|| format!("{kind} {index} has no blackboard key `{}`", cdef.key))?;
        let value = match cdef.transform {
            Transform::Value => raw,
            Transform::Negate => -raw,
            Transform::MinusOne => raw - 1.0,
            Transform::TimesAtk => {
                let atk = supporter_atk(data, inv, atk_module.as_deref());
                supporter_atk_value = Some(atk);
                raw * atk
            }
        };
        contributions.push(ZoneContribution {
            zone: cdef.zone,
            stacking: cdef.zone.stacking(),
            key: cdef.key.to_owned(),
            transform: cdef.transform,
            raw,
            value,
        });
    }

    let duration = def
        .duration_key
        .and_then(lookup)
        .or(resolved.duration)
        .filter(|d| *d > 0.0);

    Ok(SupporterEntry {
        id: def.id.to_owned(),
        char_id: def.char_id.to_owned(),
        name: operator.name.clone(),
        source_kind: kind.to_owned(),
        source_index: i32::from(index),
        source_name: resolved.name,
        contributions,
        duration,
        scope: SupporterScope {
            target: def.target,
            professions: def.professions.iter().map(|p| (*p).to_owned()).collect(),
            condition: def.condition.map(str::to_owned),
        },
        supporter_atk: supporter_atk_value,
        module_id: resolved.module_id,
        investment: inv,
    })
}
