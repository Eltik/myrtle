use std::collections::HashMap;

use myrtle_gamedata_raw::tables::building::Buff;

use super::util::buff_family;
use crate::scoring::clause::{PEER_STAGE_FIXED_LIMIT, PEER_STAGE_NET_LIMIT};
use crate::scoring::pools::ROBOTS_IN_POWER;

mod facts;
mod morale;
mod parse;
mod patterns;
mod tags;

pub use self::facts::{ACCOUNT_FACTS_KEY, resolve_account_facts};
pub use self::morale::{
    TargetedMoraleEffect, has_targeted_morale_effect, parse_morale_drain_decrease,
    parse_morale_drain_increase, parse_morale_loss_increase, targeted_morale_effects,
};
use self::parse::{
    defaulted_trade_threshold, find_all_operator_char_ids, find_faction_token,
    find_operator_keyword, formula_for_product, non_stacking_priority_families, order_value_shape,
    parse_count_keyword, parse_crew_tag_global, parse_first_float, parse_first_pct,
    parse_first_pct_from, parse_first_signed_pct, parse_first_vdown_pct, parse_first_vup_number,
    parse_kw_number, parse_last_pct, parse_layout_count_branch, parse_morale_recovery,
    parse_named_char_room_grants, parse_nth_pct, parse_order_limit, parse_per_capacity_rate,
    parse_per_hour_pct, parse_reception_ceiling, parse_room_presence_gate,
    parse_room_presence_gated_global, parse_scaling_cap, parse_skill_conversion, parse_tag_keyword,
    plain_text, room_type_from_global_label, room_type_from_label,
};
pub(crate) use self::patterns::RE_SLOT_GRANT;
use self::patterns::{
    RE_CAPS_AT, RE_CC_CLUE, RE_CC_EACH_FACTION, RE_CC_HIRE, RE_CC_TRAIN, RE_CC_WITH,
    RE_COUNT_SKILLS, RE_DORM_AURA_ALL, RE_DORM_RECOVERY_AURA, RE_DORM_SINGLE_TARGET,
    RE_DRAIN_AURA_IMMUNITY, RE_EACH_FACTION, RE_FACILITY_GATE_CHAR, RE_GLOBAL_POOL_A,
    RE_GLOBAL_POOL_B, RE_MORALE_ROOM_AURA, RE_NULLIFY_ROOM_PER_OP, RE_NULLIFY_SELF_PCT,
    RE_PER_TEAMMATE_EFF, RE_POOL_CONSUME, RE_POOL_CONSUME_ORDER, RE_POOL_CONSUME_POINTS,
    RE_POOL_CONSUME_REV, RE_POOL_CONSUME_ROBOTS_POWER, RE_POOL_CONVERT, RE_POOL_DORM_ECONOMY,
    RE_POOL_GEN_DORM_CONVERT, RE_POOL_GEN_LEVELS, RE_POOL_GEN_OWN_OCC, RE_POOL_GEN_OWN_ROOM,
    RE_POOL_TAIL, RE_SPEED_CAPACITY_TRADE, RE_TAG_MARKUP, RE_TARGET_ROOM_BOOST, RE_TOWARDS_PRODUCT,
    RE_VUP_NUMBER,
};
pub use self::tags::{
    build_faction_map, build_name_to_char, faction_tags_for, faction_tags_of, glossary_tags,
};

/// The rider a facility-count modifier needs met
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FacilityGate {
    None,
    NamedCharInRoom { char_id: String, room: String },
    NoRobotsInOtherRooms,
}

/// Named-operator-gated CC grant
#[derive(Debug, Clone, PartialEq)]
pub struct NamedCharGrant {
    pub char_id: String,
    pub target_room: String,
    pub order_limit: f64,
    pub nonprod_pct: f64,
}

/// Order-value shape, not a %
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderEffect {
    DefaultedGoldBonus { below: u32, bonus: u32 },
    HighOrderLmdBonus { above: u32, lmd: u32 },
    HigherYieldChance { strong: bool },
    Enabler,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BuffResolutionStrategy {
    /// The buff's Efficiency field, taken as the flat bonus %
    DirectEfficiency {
        value: f64,
    },

    FacilityCountScaling {
        target_room: String,
        per_unit_pct: f64,
        per_level: bool,
        nullifies_others: bool,
        base_pct: f64,
        cap_pct: Option<f64>,
    },

    TeammateSkillScaling {
        target_buff_pattern: String,
        per_match_pct: f64,
    },

    TeammateOutputMirroring {
        ratio: f64,
        step: f64,
        cap_pct: f64,
    },

    EfficiencyWithOrderLimit {
        efficiency: f64,
        order_limit: i32, // negative removes CAP
    },

    ConditionalOnTeammate {
        required_char_id: Option<String>,
        base_efficiency: f64,
        efficiency: f64,
        order_limit: i32,
    },

    ConditionalOnBaseWide {
        required_char_ids: Vec<String>,
        base_efficiency: f64,
        bonus_efficiency: f64,
        anywhere: bool,
    },

    ConditionalOnRoomPresence {
        required_char_ids: Vec<String>,
        required_faction: Option<String>,
        required_count: usize,
        room_type: String,
        base_efficiency: f64,
        bonus_efficiency: f64,
    },

    OrderDifferenceScaling {
        per_order_pct: f64,
    },

    LimitCutPerPeerEfficiency {
        pct_per_cut: f64,
        cut: i32,
        per_order_pct: f64,
    },

    OrderLimitScaling {
        per_cap_threshold: f64,
        bonus_per_threshold: f64,
        cap_pct: f64,
        stage: u8,
        includes_self: bool,
    },

    CapacityTierScaling {
        threshold: i32,
        low_pct: f64,
        high_pct: f64,
        excludes: Vec<String>,
    },

    FacilityCountModifier {
        target_room: String,
        amount: i32,
        owner_room: String,
        gate: FacilityGate,
    },

    BaseWideMatchCountScaling {
        token: String,
        per_match_pct: f64,
        cap_count: Option<usize>,
    },

    NamedTargetRoomBoost {
        self_room: String,
        target_char_id: String,
        target_room: String,
        bonus_pct: f64,
        active: bool,
    },

    MatchCountScaling {
        token: String,
        per_match_pct: f64,
        cap_pct: Option<f64>,
        bonus_char_id: Option<String>,
        bonus_pct: f64,
        count_skills: bool,
        capacity: bool,
    },

    ConditionalOnFaction {
        faction_token: String,
        base_efficiency: f64,
        efficiency: f64,
    },

    SkillTypeConversion {
        from_tokens: Vec<String>,
        to_token: String,
    },

    NullifyTeammatesSelfScaling {
        per_teammate_pct: f64,
    },

    OrderValue {
        effect: OrderEffect,
        pure_gold: bool,
    },

    GlobalPoolScaling {
        target_room: String,
        base_pct: f64,
        pct: f64,
        per: f64,
        resource: String,
    },

    GlobalEffect {
        target_room: String,
        bonus_pct: f64,
    },

    GlobalEffectWithCrewTag {
        target_room: String,
        bonus_pct: f64,
        tag: String,
        min_others: usize,
    },

    ConditionalGlobalEffect {
        target_room: String,
        faction_token: String,
        required_count: usize,
        per_operator: bool,
        bonus_pct: f64,
        order_limit: i32,
        formula_bonuses: Vec<(String, f64)>,
    },

    LayoutCountBranch {
        a_term: String,
        b_term: String,
        /// Payload when `a >= b`
        ge_room: String,
        ge_pct: f64,
        /// Payload when `b > a`
        gt_room: String,
        gt_pct: f64,
    },

    RoomPresenceGatedGlobal {
        required_faction: String,
        required_count: usize,
        room_type: String,
        target_room: String,
        bonus_pct: f64,
    },

    NamedCharRoomGrants {
        grants: Vec<NamedCharGrant>,
    },

    MoraleDrainAuraImmunity,

    TagBased {
        tag: String,
        bonus_pct: f64,
        target_room: String,
    },

    MoraleModifier {
        recovery_per_hour: f64, // negative = drain
        is_self_only: bool,
        base_wide: bool,
        single_target: bool,
    },

    CapacityOnly {
        order_limit: i32,
    },

    NonProduction {
        value: f64,
    },

    ControlNonProduction {
        target_room: String,
        value: f64,
        same_room_gate: Option<Vec<String>>,
    },

    /// No branch parses the buff
    Complex {
        estimated_pct: f64,
    },

    /// Time-averaged over a full 24h shift
    MoraleDecayEfficiency {
        time_averaged_value: f64,
    },

    TimeRamp {
        first_hour: f64,
        per_hour: f64,
        cap: f64,
    },

    PoolGenerateBuildingLevels {
        resource: String,
        per_level: f64,
        cap: f64,
    },

    PoolTail {
        base: Box<Self>,
        resource: String,
        pct: f64,
        per: f64,
    },

    PoolPointsScaling {
        resource: String,
        per: f64,
        pct: f64,
    },

    PoolGenerateOwnRoomLevel {
        resource: String,
        per_level: f64,
    },

    PoolGenerateOwnRoomOccupants {
        resource: String,
        per: f64,
    },

    RoomPerOperatorGrant {
        speed_pct: f64,
        capacity: f64,
    },

    /// Player-declared recruit slots beyond the first
    AccountFacts {
        open_recruit_slots: u32,
    },

    PoolDormEconomy {
        resource: String,
        per_occupant: f64,
        per: f64,
        pct: f64,
    },

    PoolGenerateDormAndConvert {
        gen_resource: String,
        per_occupant: f64,
        to_resource: String,
        from_per: f64,
    },

    MoraleRoomAura {
        delta: f64,
    },

    DormRecoveryAura {
        rate: f64,
    },

    PerTeammateEfficiency {
        per_teammate_pct: f64,
    },

    PoolConvert {
        from: String,
        to: String,
        from_per: f64,
    },

    PoolPayoff {
        pct: f64,
    },
}

pub fn build_registry(
    buffs: &HashMap<String, Buff>,
    name_to_char: &HashMap<String, String>,
) -> (
    HashMap<String, BuffResolutionStrategy>,
    HashMap<String, f64>, // buff_id -> morale drain
) {
    let mut registry: HashMap<String, BuffResolutionStrategy> = HashMap::new();
    let mut morale_drains = HashMap::new();
    let defaulted_below = defaulted_trade_threshold(buffs);

    for (buff_id, buff) in buffs {
        let prefix = buff_family(buff_id);

        let mut drain = 0.0;
        if let Some(val) = parse_morale_drain_increase(&buff.description) {
            drain += val;
        }
        if let Some(val) = parse_morale_loss_increase(&buff.description) {
            drain += val;
        }
        if let Some(val) = parse_morale_drain_decrease(&buff.description) {
            drain -= val;
        }
        if drain != 0.0 {
            morale_drains.insert(buff_id.clone(), drain);
        }

        let desc_l = buff.description.to_lowercase();
        if desc_l.contains("only affects facility quantity")
            || desc_l.contains("only affects the facility count")
        {
            let target_room = if desc_l.contains("power plant") {
                "POWER"
            } else if desc_l.contains("trading post") {
                "TRADING"
            } else {
                "MANUFACTURE"
            };
            let amount = parse_first_float(&buff.description).unwrap_or(1.0) as i32;
            let gate = if let Some(c) = RE_FACILITY_GATE_CHAR.captures(&buff.description) {
                match (
                    name_to_char.get(&c[1].to_lowercase()),
                    room_type_from_label(c[2].trim()),
                ) {
                    (Some(id), Some(room)) => FacilityGate::NamedCharInRoom {
                        char_id: id.clone(),
                        room: room.to_string(),
                    },
                    _ => continue,
                }
            } else if buff.description.contains("no <$cc.tag.op>") {
                FacilityGate::NoRobotsInOtherRooms
            } else {
                FacilityGate::None
            };
            registry.insert(
                buff_id.clone(),
                BuffResolutionStrategy::FacilityCountModifier {
                    target_room: target_room.to_string(),
                    amount,
                    owner_room: buff.room_type.clone(),
                    gate,
                },
            );
            continue;
        }

        if let Some(c) = RE_POOL_GEN_LEVELS.captures(&buff.description) {
            registry.insert(
                buff_id.clone(),
                BuffResolutionStrategy::PoolGenerateBuildingLevels {
                    resource: c[2].to_string(),
                    per_level: c[1].parse().unwrap_or(0.0),
                    cap: c[3].parse().unwrap_or(f64::INFINITY),
                },
            );
            continue;
        }
        if buff.room_type != "CONTROL" {
            if let Some(c) = RE_POOL_CONSUME.captures(&buff.description) {
                registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::PoolPointsScaling {
                        resource: c[2].to_string(),
                        per: c[1].parse().unwrap_or(1.0),
                        pct: c[3].parse().unwrap_or(0.0),
                    },
                );
                continue;
            }
            if let Some(c) = RE_POOL_CONSUME_REV.captures(&buff.description) {
                registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::PoolPointsScaling {
                        resource: c[3].to_string(),
                        per: c[2].parse().unwrap_or(1.0),
                        pct: c[1].parse().unwrap_or(0.0),
                    },
                );
                continue;
            }
            if let Some(c) = RE_POOL_CONSUME_POINTS.captures(&buff.description) {
                registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::PoolPointsScaling {
                        resource: c[2].to_string(),
                        per: c[1].parse().unwrap_or(1.0),
                        pct: c[3].parse().unwrap_or(0.0),
                    },
                );
                continue;
            }
            if let Some(c) = RE_POOL_CONSUME_ROBOTS_POWER.captures(&buff.description) {
                registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::PoolPointsScaling {
                        resource: ROBOTS_IN_POWER.to_string(),
                        per: 1.0,
                        pct: c[1].parse().unwrap_or(0.0),
                    },
                );
                continue;
            }
        }
        if let Some(c) = RE_POOL_GEN_OWN_ROOM.captures(&buff.description) {
            registry.insert(
                buff_id.clone(),
                BuffResolutionStrategy::PoolGenerateOwnRoomLevel {
                    resource: c[2].to_string(),
                    per_level: c[1].parse().unwrap_or(0.0),
                },
            );
            continue;
        }
        if let Some(c) = RE_POOL_GEN_OWN_OCC.captures(&buff.description) {
            registry.insert(
                buff_id.clone(),
                BuffResolutionStrategy::PoolGenerateOwnRoomOccupants {
                    resource: c[1].to_string(),
                    per: c[2].parse().unwrap_or(0.0),
                },
            );
            continue;
        }
        if let Some(c) = RE_POOL_GEN_DORM_CONVERT.captures(&buff.description) {
            registry.insert(
                buff_id.clone(),
                BuffResolutionStrategy::PoolGenerateDormAndConvert {
                    gen_resource: c[1].to_string(),
                    per_occupant: c[2].parse().unwrap_or(0.0),
                    from_per: c[3].parse().unwrap_or(1.0),
                    to_resource: c[4].to_string(),
                },
            );
            continue;
        }
        if let Some(c) = RE_POOL_DORM_ECONOMY.captures(&buff.description) {
            registry.insert(
                buff_id.clone(),
                BuffResolutionStrategy::PoolDormEconomy {
                    resource: c[1].to_string(),
                    per_occupant: c[2].parse().unwrap_or(0.0),
                    per: c[3].parse().unwrap_or(1.0),
                    pct: c[4].parse().unwrap_or(0.0),
                },
            );
            continue;
        }
        if let Some(c) = RE_POOL_CONVERT.captures(&buff.description) {
            registry.insert(
                buff_id.clone(),
                BuffResolutionStrategy::PoolConvert {
                    from: c[2].to_string(),
                    from_per: c[1].parse().unwrap_or(1.0),
                    to: c[3].to_string(),
                },
            );
            continue;
        }
        if let Some(c) = RE_MORALE_ROOM_AURA.captures(&buff.description) {
            registry.insert(
                buff_id.clone(),
                BuffResolutionStrategy::MoraleRoomAura {
                    delta: c[1].parse().unwrap_or(0.0),
                },
            );
            continue;
        }
        if let Some(c) = RE_DORM_RECOVERY_AURA.captures(&buff.description) {
            registry.insert(
                buff_id.clone(),
                BuffResolutionStrategy::DormRecoveryAura {
                    rate: c[1].parse().unwrap_or(0.0),
                },
            );
            continue;
        }
        if buff.room_type != "CONTROL" {
            if let Some(c) = RE_PER_TEAMMATE_EFF.captures(&buff.description) {
                registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::PerTeammateEfficiency {
                        per_teammate_pct: c[1].parse().unwrap_or(0.0),
                    },
                );
                continue;
            }
            if let Some(c) = RE_POOL_CONSUME_ORDER.captures(&buff.description) {
                registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::PoolPointsScaling {
                        resource: c[2].to_string(),
                        per: c[1].parse().unwrap_or(1.0),
                        pct: c[3].parse().unwrap_or(0.0),
                    },
                );
                continue;
            }
            if let Some(c) = RE_SPEED_CAPACITY_TRADE.captures(&buff.description) {
                let magnitude: f64 = c[2].trim_start_matches('+').parse().unwrap_or(0.0);
                let signed = if &c[1] == "vdown" {
                    -magnitude.abs()
                } else {
                    magnitude
                };
                let cap_magnitude: i32 = c[4].trim_start_matches('+').parse().unwrap_or(0);
                let cap_signed = if &c[3] == "vdown" {
                    -cap_magnitude.abs()
                } else {
                    cap_magnitude
                };
                registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::EfficiencyWithOrderLimit {
                        efficiency: signed,
                        order_limit: cap_signed,
                    },
                );
                continue;
            }
        }

        let strategy = match buff.room_type.as_str() {
            "MEETING" => {
                let value = parse_reception_ceiling(&buff.description)
                    .or_else(|| (buff.efficiency != 0).then(|| f64::from(buff.efficiency)))
                    .or_else(|| parse_first_pct(&buff.description))
                    .unwrap_or(0.0);
                BuffResolutionStrategy::NonProduction { value }
            }
            "WORKSHOP" | "HIRE" | "TRAINING" => BuffResolutionStrategy::NonProduction {
                value: f64::from(buff.efficiency),
            },
            "DORMITORY" => {
                let desc_lower = buff.description.to_lowercase();
                let aura = RE_DORM_AURA_ALL
                    .captures(&buff.description)
                    .and_then(|c| c[1].parse::<f64>().ok());
                let recovery =
                    aura.unwrap_or_else(|| parse_first_float(&buff.description).unwrap_or(0.0));
                let is_self_only = aura.is_none()
                    && (desc_lower.contains("self")
                        || desc_lower.contains("oneself")
                        || prefix.contains("_oneself"));
                let single_target = RE_DORM_SINGLE_TARGET.is_match(&buff.description);
                BuffResolutionStrategy::MoraleModifier {
                    recovery_per_hour: recovery,
                    is_self_only: is_self_only && !single_target,
                    base_wide: false,
                    single_target,
                }
            }
            "CONTROL" => {
                let desc_lower = buff.description.to_lowercase();
                if let Some(branch) = parse_layout_count_branch(&buff.description) {
                    branch
                } else if let Some(gated) = parse_room_presence_gated_global(&buff.description) {
                    gated
                } else if let Some(crew_gated) = parse_crew_tag_global(&buff.description) {
                    crew_gated
                } else if let Some(c) = RE_GLOBAL_POOL_A.captures(&buff.description) {
                    BuffResolutionStrategy::GlobalPoolScaling {
                        target_room: room_type_from_global_label(&c[3]).to_string(),
                        base_pct: 0.0,
                        pct: c[4].parse().unwrap_or(0.0),
                        per: c[1].parse().unwrap_or(1.0),
                        resource: c[2].to_string(),
                    }
                } else if let Some(c) = RE_GLOBAL_POOL_B.captures(&buff.description) {
                    BuffResolutionStrategy::GlobalPoolScaling {
                        target_room: room_type_from_global_label(&c[1]).to_string(),
                        base_pct: c[2].parse().unwrap_or(0.0),
                        pct: c[3].parse().unwrap_or(0.0),
                        per: c[4].parse().unwrap_or(1.0),
                        resource: c[5].to_string(),
                    }
                } else if prefix.contains("_fraction")
                    || prefix.contains("_tag")
                    || (RE_EACH_FACTION.is_match(&buff.description)
                        && (desc_lower.contains("factor") || desc_lower.contains("trading")))
                {
                    let tag = parse_tag_keyword(&buff.description).unwrap_or_default();
                    let bonus = parse_first_pct(&buff.description).unwrap_or(0.0);
                    if desc_lower.contains("factor") || desc_lower.contains("trading") {
                        let target_room = if desc_lower.contains("trading") {
                            "TRADING"
                        } else {
                            "MANUFACTURE"
                        };
                        let formula_bonuses: Vec<(String, f64)> = RE_TOWARDS_PRODUCT
                            .captures_iter(&buff.description)
                            .filter_map(|c| {
                                Some((
                                    formula_for_product(&c[2])?.to_string(),
                                    c[1].parse::<f64>().ok()?,
                                ))
                            })
                            .collect();
                        BuffResolutionStrategy::ConditionalGlobalEffect {
                            target_room: target_room.to_string(),
                            faction_token: tag,
                            required_count: 1,
                            per_operator: true,
                            bonus_pct: bonus,
                            order_limit: 0,
                            formula_bonuses,
                        }
                    } else {
                        BuffResolutionStrategy::TagBased {
                            tag,
                            bonus_pct: bonus,
                            target_room: "MANUFACTURE".to_string(),
                        }
                    }
                } else if prefix.contains("_mp_")
                    || prefix.contains("_cost")
                    || prefix.contains("allCost")
                {
                    let partner_gated = RE_CC_WITH.is_match(&buff.description);
                    let faction_counted = RE_CC_EACH_FACTION.is_match(&buff.description);
                    let aura_scope = desc_lower.contains("operators in the control center")
                        || desc_lower.contains("other building");
                    let recovery = if partner_gated || faction_counted || !aura_scope {
                        0.0
                    } else {
                        parse_morale_recovery(&buff.description).unwrap_or(0.0)
                    };
                    BuffResolutionStrategy::MoraleModifier {
                        recovery_per_hour: recovery,
                        is_self_only: false,
                        base_wide: desc_lower.contains("other building"),
                        single_target: false,
                    }
                } else if prefix.contains("_prod_")
                    || prefix.contains("_tra_")
                    || prefix.contains("_trade_")
                {
                    let target_room = if prefix.contains("_prod_") {
                        "MANUFACTURE"
                    } else {
                        "TRADING"
                    };
                    let bonus = parse_first_signed_pct(&buff.description).unwrap_or(0.0);
                    let order_limit = parse_order_limit(&buff.description).unwrap_or(0);
                    if let Some(faction_token) = parse_tag_keyword(&buff.description)
                        && !faction_token
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_digit())
                        && (buff.description.contains("Operators")
                            || RE_EACH_FACTION.is_match(&buff.description))
                    {
                        let required_count = RE_VUP_NUMBER
                            .captures(&buff.description)
                            .and_then(|c| c[1].parse::<usize>().ok());
                        BuffResolutionStrategy::ConditionalGlobalEffect {
                            target_room: target_room.to_string(),
                            faction_token,
                            required_count: required_count.unwrap_or(1),
                            per_operator: required_count.is_none(),
                            bonus_pct: bonus,
                            order_limit,
                            formula_bonuses: Vec::new(),
                        }
                    } else {
                        BuffResolutionStrategy::GlobalEffect {
                            target_room: target_room.to_string(),
                            bonus_pct: bonus,
                        }
                    }
                } else if let Some(grants) =
                    parse_named_char_room_grants(&buff.description, name_to_char)
                {
                    BuffResolutionStrategy::NamedCharRoomGrants { grants }
                } else if let Some((target_room, c)) = [
                    ("MEETING", &RE_CC_CLUE),
                    ("TRAINING", &RE_CC_TRAIN),
                    ("HIRE", &RE_CC_HIRE),
                ]
                .iter()
                .find_map(|(room, rx)| rx.captures(&buff.description).map(|c| (*room, c)))
                    && !buff.description.contains("assigned to the Reception Room")
                {
                    let same_room_gate = RE_CC_WITH.captures(&buff.description).map(|w| {
                        name_to_char
                            .get(&w[1].to_lowercase())
                            .cloned()
                            .into_iter()
                            .collect()
                    });
                    BuffResolutionStrategy::ControlNonProduction {
                        target_room: target_room.to_string(),
                        value: c[1].parse().unwrap_or(0.0),
                        same_room_gate,
                    }
                } else {
                    let value = parse_first_pct(&buff.description).unwrap_or(0.0);
                    BuffResolutionStrategy::Complex {
                        estimated_pct: value,
                    }
                }
            }
            "MANUFACTURE" | "TRADING" | "POWER" => {
                let in_work_area = buff.description.contains("Work Area");
                let in_base_anywhere = buff.description.contains("is in the Base");
                let base_wide = (in_work_area || in_base_anywhere)
                    .then(|| find_all_operator_char_ids(&buff.description, name_to_char))
                    .filter(|(ids, _)| !ids.is_empty());
                if RE_DRAIN_AURA_IMMUNITY.is_match(&buff.description) {
                    BuffResolutionStrategy::MoraleDrainAuraImmunity
                } else if let Some((target_room, target_char_id, bonus_pct)) = RE_TARGET_ROOM_BOOST
                    .captures(&buff.description)
                    .and_then(|c| {
                        Some((
                            room_type_from_label(&c[1])?.to_string(),
                            name_to_char.get(&c[2].to_lowercase())?.clone(),
                            c[3].parse::<f64>().ok()?,
                        ))
                    })
                {
                    BuffResolutionStrategy::NamedTargetRoomBoost {
                        self_room: buff.room_type.clone(),
                        target_char_id,
                        target_room,
                        bonus_pct,
                        active: false,
                    }
                } else if let Some(gate) = parse_room_presence_gate(
                    &buff.description,
                    f64::from(buff.efficiency),
                    name_to_char,
                ) {
                    gate
                } else if let Some((required_char_ids, name_end)) = base_wide {
                    let base_efficiency = f64::from(buff.efficiency);
                    let bonus_efficiency =
                        parse_first_pct_from(&buff.description, name_end).unwrap_or(0.0);
                    BuffResolutionStrategy::ConditionalOnBaseWide {
                        required_char_ids,
                        base_efficiency,
                        bonus_efficiency,
                        anywhere: !in_work_area,
                    }
                } else if buff.description.contains("same")
                    && !buff.description.contains("for every")
                    && let Some((req_name, name_end)) =
                        find_operator_keyword(&buff.description, name_to_char)
                {
                    let required_char_id = name_to_char.get(&req_name).cloned();
                    let base_efficiency = f64::from(buff.efficiency);
                    let efficiency = parse_first_pct_from(&buff.description, name_end)
                        .or_else(|| {
                            (base_efficiency == 0.0)
                                .then(|| parse_first_pct(&buff.description))
                                .flatten()
                        })
                        .unwrap_or(0.0);
                    let order_limit = parse_order_limit(&buff.description).unwrap_or(0);
                    BuffResolutionStrategy::ConditionalOnTeammate {
                        required_char_id,
                        base_efficiency,
                        efficiency,
                        order_limit,
                    }
                } else if buff.description.contains("same")
                    && !buff.description.contains("for every")
                    && !buff.description.contains("for each")
                    && let Some((faction_token, token_end)) = find_faction_token(&buff.description)
                {
                    let base_efficiency = f64::from(buff.efficiency);
                    let efficiency = parse_first_pct_from(&buff.description, token_end)
                        .or_else(|| {
                            (base_efficiency == 0.0)
                                .then(|| parse_first_pct(&buff.description))
                                .flatten()
                        })
                        .unwrap_or(0.0);
                    BuffResolutionStrategy::ConditionalOnFaction {
                        faction_token,
                        base_efficiency,
                        efficiency,
                    }
                } else if let Some(cap) = RE_NULLIFY_SELF_PCT.captures(&buff.description) {
                    BuffResolutionStrategy::NullifyTeammatesSelfScaling {
                        per_teammate_pct: cap[1].parse().unwrap_or(0.0),
                    }
                } else if let Some((from_tokens, to_token)) =
                    parse_skill_conversion(&buff.description)
                {
                    BuffResolutionStrategy::SkillTypeConversion {
                        from_tokens,
                        to_token,
                    }
                } else if let Some(token) = parse_count_keyword(&buff.description)
                    && plain_text(&buff.description)
                        .to_lowercase()
                        .contains("in the base")
                    && !plain_text(&buff.description)
                        .to_lowercase()
                        .contains("every facility")
                {
                    let count_at = buff
                        .description
                        .find("for each")
                        .or_else(|| buff.description.find("for every"))
                        .unwrap_or(0);
                    BuffResolutionStrategy::BaseWideMatchCountScaling {
                        token,
                        per_match_pct: parse_first_pct_from(&buff.description, count_at)
                            .unwrap_or(0.0),
                        cap_count: RE_CAPS_AT
                            .captures(&buff.description)
                            .and_then(|c| c[1].parse::<usize>().ok()),
                    }
                } else if let Some(token) = parse_count_keyword(&buff.description) {
                    let per_match_pct = parse_first_pct(&buff.description).unwrap_or(5.0);
                    let cap_pct = parse_scaling_cap(&buff.description);
                    let (bonus_char_id, bonus_pct) = buff
                        .description
                        .contains("same")
                        .then(|| find_operator_keyword(&buff.description, name_to_char))
                        .flatten()
                        .map_or((None, 0.0), |(name, name_end)| {
                            (
                                name_to_char.get(&name).cloned(),
                                parse_first_pct_from(&buff.description, name_end).unwrap_or(0.0),
                            )
                        });
                    let plain = plain_text(&buff.description);
                    let count_skills = RE_COUNT_SKILLS.is_match(&plain);
                    let capacity = plain.to_lowercase().contains("capacity");
                    BuffResolutionStrategy::MatchCountScaling {
                        token,
                        per_match_pct,
                        cap_pct,
                        bonus_char_id,
                        bonus_pct,
                        count_skills,
                        capacity,
                    }
                } else if buff.room_type == "TRADING"
                    && plain_text(&buff.description).contains("Pure Gold Production Line")
                {
                    let per_line = buffs
                        .values()
                        .filter(|b| b.room_type == "TRADING")
                        .filter(|b| {
                            let t = plain_text(&b.description);
                            t.contains("Pure Gold Production Line") && t.contains('%')
                        })
                        .filter_map(|b| parse_first_pct(&b.description))
                        .fold(0.0, f64::max);
                    let counted = RE_TAG_MARKUP
                        .captures(&buff.description)
                        .map(|c| c[1].to_string())
                        .or_else(|| parse_count_keyword(&buff.description));
                    if plain_text(&buff.description)
                        .to_lowercase()
                        .contains("in the base")
                        && let Some(token) = counted
                    {
                        BuffResolutionStrategy::BaseWideMatchCountScaling {
                            token,
                            per_match_pct: per_line,
                            cap_count: RE_CAPS_AT
                                .captures(&buff.description)
                                .and_then(|c| c[1].parse::<usize>().ok()),
                        }
                    } else {
                        BuffResolutionStrategy::FacilityCountScaling {
                            target_room: super::assignment::GOLD_LINES.to_string(),
                            per_unit_pct: per_line,
                            per_level: false,
                            nullifies_others: false,
                            base_pct: 0.0,
                            cap_pct: None,
                        }
                    }
                } else if buff.room_type == "TRADING"
                    && let Some((effect, pure_gold)) =
                        order_value_shape(&buff.description, defaulted_below)
                {
                    BuffResolutionStrategy::OrderValue { effect, pure_gold }
                } else if prefix.contains("_limit_count") {
                    let pct_per_cut = parse_nth_pct(&buff.description, 0).unwrap_or(10.0);
                    let cut = parse_order_limit(&buff.description).unwrap_or(-1);
                    let per_order_pct = parse_nth_pct(&buff.description, 1).unwrap_or(4.0);
                    BuffResolutionStrategy::LimitCutPerPeerEfficiency {
                        pct_per_cut,
                        cut,
                        per_order_pct,
                    }
                } else if prefix.contains("_limit_diff")
                    || (buff.room_type == "TRADING"
                        && buff.description.contains("order acquisition efficiency")
                        && buff.description.contains("difference"))
                {
                    BuffResolutionStrategy::OrderDifferenceScaling {
                        per_order_pct: parse_first_pct(&buff.description).unwrap_or(4.0),
                    }
                } else if (prefix.contains("_limit") || prefix.contains("limit&"))
                    && !prefix.contains("_spd")
                    && buff.efficiency == 0
                {
                    BuffResolutionStrategy::CapacityOnly {
                        order_limit: parse_order_limit(&buff.description).unwrap_or(0),
                    }
                } else if prefix.contains("_reduce")
                    && buff.description.contains("Morale difference")
                {
                    let peak = f64::from(buff.efficiency);
                    let interval = parse_kw_number(&buff.description).unwrap_or(4.0);
                    let penalty_pct = parse_first_vdown_pct(&buff.description).unwrap_or(5.0);
                    let avg_penalty = (12.0 / interval) * penalty_pct;
                    BuffResolutionStrategy::MoraleDecayEfficiency {
                        time_averaged_value: (peak - avg_penalty).max(0.0),
                    }
                } else if prefix.contains("_addition&cost")
                    && buff.description.contains("Morale difference")
                {
                    let bonus = parse_first_pct(&buff.description).unwrap_or(0.0);
                    let threshold = parse_kw_number(&buff.description).unwrap_or(12.0);
                    let active_fraction = (24.0 - threshold) / 24.0;
                    BuffResolutionStrategy::MoraleDecayEfficiency {
                        time_averaged_value: bonus * active_fraction,
                    }
                } else if prefix.contains("_addition") && buff.description.contains("per hour") {
                    let base = f64::from(buff.efficiency);
                    let per_hr = parse_per_hour_pct(&buff.description).unwrap_or(1.0);
                    let cap = parse_last_pct(&buff.description).unwrap_or(25.0);
                    let first_hour = if base > 0.0 { base } else { per_hr };
                    BuffResolutionStrategy::TimeRamp {
                        first_hour,
                        per_hour: per_hr,
                        cap,
                    }
                } else if prefix.starts_with("trade_ord_spd&limit") {
                    let efficiency = f64::from(buff.efficiency);
                    let order_limit = parse_order_limit(&buff.description).unwrap_or(0);
                    BuffResolutionStrategy::EfficiencyWithOrderLimit {
                        efficiency,
                        order_limit,
                    }
                } else if prefix == "trade_ord_spd_variable3" {
                    let threshold = parse_first_vup_number(&buff.description).unwrap_or(5.0);
                    let bonus = parse_nth_pct(&buff.description, 0).unwrap_or(25.0);
                    let cap = parse_last_pct(&buff.description).unwrap_or(100.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: threshold,
                        bonus_per_threshold: bonus,
                        cap_pct: cap,
                        includes_self: false, // "from teammates"
                        stage: PEER_STAGE_FIXED_LIMIT,
                    }
                } else if prefix == "trade_ord_spd_variable" {
                    let per = parse_first_pct(&buff.description).unwrap_or(4.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: 1.0,
                        bonus_per_threshold: per,
                        cap_pct: f64::MAX,
                        includes_self: false, // "from others"
                        stage: PEER_STAGE_NET_LIMIT,
                    }
                } else if prefix == "manu_prod_spd_variable" {
                    let per = parse_first_pct(&buff.description).unwrap_or(2.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: 1.0,
                        bonus_per_threshold: per,
                        cap_pct: f64::MAX,
                        includes_self: true,
                        stage: PEER_STAGE_FIXED_LIMIT,
                    }
                } else if prefix == "manu_prod_spd_variable3" {
                    let threshold =
                        parse_first_vup_number(&buff.description).unwrap_or(16.0) as i32;
                    let low = parse_nth_pct(&buff.description, 0).unwrap_or(1.0);
                    let high = parse_nth_pct(&buff.description, 1).unwrap_or(3.0);
                    BuffResolutionStrategy::CapacityTierScaling {
                        threshold,
                        low_pct: low,
                        high_pct: high,
                        excludes: non_stacking_priority_families(&buff.description, buffs),
                    }
                } else if prefix.contains("&formula") && buff.description.contains("recipe type") {
                    let per_unit = parse_last_pct(&buff.description).unwrap_or(2.0);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "MANUFACTURE_RECIPE_TYPES".to_string(),
                        per_unit_pct: per_unit,
                        per_level: false,
                        nullifies_others: false,
                        base_pct: f64::from(buff.efficiency),
                        cap_pct: None,
                    }
                } else if prefix.contains("&meet")
                    && buff.description.contains("per Reception Room level")
                {
                    let base = f64::from(buff.efficiency);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: super::assignment::MEETING_LEVEL.to_string(),
                        per_unit_pct: parse_nth_pct(&buff.description, 1).unwrap_or(0.0),
                        per_level: false,
                        nullifies_others: false,
                        base_pct: base,
                        cap_pct: parse_last_pct(&buff.description).map(|c| (c - base).max(0.0)),
                    }
                } else if prefix.contains("&lv")
                    && buff
                        .description
                        .contains("for every level of each Training Room")
                {
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: super::assignment::TRAINING_LEVEL.to_string(),
                        per_unit_pct: parse_first_pct(&buff.description).unwrap_or(0.0),
                        per_level: false,
                        nullifies_others: false,
                        base_pct: f64::from(buff.efficiency),
                        cap_pct: parse_last_pct(&buff.description),
                    }
                } else if buff.efficiency > 0 {
                    BuffResolutionStrategy::DirectEfficiency {
                        value: f64::from(buff.efficiency),
                    }
                } else if prefix.contains("&power") {
                    let per_unit = parse_first_pct(&buff.description).unwrap_or(5.0);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "POWER".to_string(),
                        per_unit_pct: per_unit,
                        per_level: false,
                        nullifies_others: true,
                        base_pct: 0.0,
                        cap_pct: None,
                    }
                } else if let Some(c) = RE_NULLIFY_ROOM_PER_OP.captures(&buff.description) {
                    BuffResolutionStrategy::RoomPerOperatorGrant {
                        speed_pct: c
                            .get(1)
                            .and_then(|m| m.as_str().parse().ok())
                            .unwrap_or(0.0),
                        capacity: c[2].parse().unwrap_or(0.0),
                    }
                } else if prefix.contains("&manu") {
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "MANUFACTURE".to_string(),
                        per_unit_pct: 0.0,
                        per_level: false,
                        nullifies_others: true,
                        base_pct: 0.0,
                        cap_pct: None,
                    }
                } else if prefix.contains("&dorm") {
                    let per_unit = parse_first_pct(&buff.description).unwrap_or(1.0);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "DORMITORY".to_string(),
                        per_unit_pct: per_unit,
                        per_level: true,
                        nullifies_others: false,
                        base_pct: 0.0,
                        cap_pct: None,
                    }
                } else if prefix.contains("_skill_spd") {
                    let per_match = parse_first_pct(&buff.description).unwrap_or(5.0);
                    let keyword = parse_tag_keyword(&buff.description).unwrap_or_default();
                    BuffResolutionStrategy::TeammateSkillScaling {
                        target_buff_pattern: keyword,
                        per_match_pct: per_match,
                    }
                } else if prefix.contains("_variable2") {
                    let cap = parse_last_pct(&buff.description).unwrap_or(25.0);
                    let per = parse_first_pct(&buff.description).unwrap_or(5.0);
                    let step = parse_nth_pct(&buff.description, 1)
                        .filter(|s| *s > 0.0)
                        .unwrap_or(per);
                    BuffResolutionStrategy::TeammateOutputMirroring {
                        ratio: per,
                        step,
                        cap_pct: cap,
                    }
                } else if prefix.contains("_bd") {
                    BuffResolutionStrategy::Complex { estimated_pct: 0.0 }
                } else if prefix.contains("&gold") || prefix.contains("&trade") {
                    let per_unit = parse_first_pct(&buff.description).unwrap_or(5.0);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: if prefix.contains("&gold") {
                            "MANUFACTURE"
                        } else {
                            "TRADING"
                        }
                        .to_string(),
                        per_unit_pct: per_unit,
                        per_level: false,
                        nullifies_others: false,
                        base_pct: 0.0,
                        cap_pct: None,
                    }
                } else if buff.room_type == "POWER"
                    && buff.description.contains("Drone recovery")
                    && buff.description.contains("max Drone capacity")
                    && let Some((per_pct, per_units)) = parse_per_capacity_rate(&buff.description)
                {
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "DRONE_CAPACITY".to_string(),
                        per_unit_pct: per_pct / per_units,
                        per_level: false,
                        nullifies_others: false,
                        base_pct: f64::from(buff.efficiency),
                        cap_pct: parse_last_pct(&buff.description),
                    }
                } else if buff.room_type == "POWER" && buff.description.contains("Drone recovery") {
                    let value = parse_last_pct(&buff.description)
                        .or_else(|| parse_first_pct(&buff.description))
                        .unwrap_or(0.0);
                    BuffResolutionStrategy::DirectEfficiency { value }
                } else {
                    let est = parse_first_pct(&buff.description).unwrap_or(15.0);
                    BuffResolutionStrategy::Complex { estimated_pct: est }
                }
            }
            _ => BuffResolutionStrategy::CapacityOnly { order_limit: 0 },
        };

        let strategy = match RE_POOL_TAIL.captures(&buff.description) {
            Some(c) if buff.room_type == "MANUFACTURE" || buff.room_type == "TRADING" => {
                BuffResolutionStrategy::PoolTail {
                    base: Box::new(strategy),
                    resource: c[3].to_string(),
                    pct: c[1].parse().unwrap_or(0.0),
                    per: c
                        .get(2)
                        .and_then(|m| m.as_str().parse().ok())
                        .unwrap_or(1.0),
                }
            }
            _ => strategy,
        };
        registry.insert(buff_id.clone(), strategy);
    }

    (registry, morale_drains)
}
