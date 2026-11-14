//! Base-skill bonuses parsed into the clauses `ledger.rs` scores

use std::collections::HashMap;

use myrtle_gamedata_raw::tables::building::Buff;

use crate::registry::{BuffResolutionStrategy, OrderEffect};
use crate::util::buff_family;

/// What a clause's contribution is denominated in
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Metric {
    ManufactureSpeed,
    TradingSpeed,
    OrderValue {
        pure_gold: bool,
    },
    CapacityLimit,
    DroneRecovery,
    MoraleRecovery {
        base_wide: bool,
    },
    /// Morale drain modifier per hour, positive drains faster
    MoraleDrainDelta,
    MoraleDrainAura,
    MoraleDrainAuraImmunity,
    DormRecoveryAura,
    NonProduction(NonProdKind),
    FacilityCount(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NonProdKind {
    ClueSearch, // MEETING
    HrContact,  // HIRE
    Training,   // TRAINING
    Workshop,   // WORKSHOP
    Other,
}

/// How multiple surviving entries of one metric combine in a room
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombineRule {
    Sum,
    OrderMix,
}

impl Metric {
    #[must_use]
    pub fn speed_for_room(room_type: &str) -> Self {
        match room_type {
            "MANUFACTURE" => Self::ManufactureSpeed,
            "TRADING" => Self::TradingSpeed,
            "POWER" => Self::DroneRecovery,
            "MEETING" => Self::NonProduction(NonProdKind::ClueSearch),
            "HIRE" => Self::NonProduction(NonProdKind::HrContact),
            "TRAINING" => Self::NonProduction(NonProdKind::Training),
            "WORKSHOP" => Self::NonProduction(NonProdKind::Workshop),
            _ => Self::NonProduction(NonProdKind::Other),
        }
    }

    #[must_use]
    pub const fn combine(&self) -> CombineRule {
        match self {
            Self::OrderValue { .. } => CombineRule::OrderMix,
            _ => CombineRule::Sum,
        }
    }
}

/// Who a scaling or requirement clause counts or checks
#[derive(Clone, Debug, PartialEq)]
pub enum Subject {
    Tag(String),
    SkillTag(String),
    SkillIdPrefix(String),
    AnyOtherOccupant,
    Chars(Vec<String>),
    PeerMetricAbove { metric: Metric, threshold: f64 },
    CapacityPoints { threshold: f64, above: bool },
}

/// Where a presence requirement looks
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CondScope {
    Room,
    BaseWorkArea,
    BaseAnywhere,
    RoomTypeElsewhere,
}

/// CC global gated on the TARGET room's team
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gate {
    pub tag: String,
    pub required_count: usize,
    pub per_operator: bool,
}

/// Which contributions a suppressor leaves standing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuppressExempt {
    None,
    RoomCountScaledSources,
}

/// What a pool generator's points multiply against
#[derive(Clone, Debug, PartialEq)]
pub enum PoolBasis {
    FunctionalLevels,
    OwnRoomLevel,
    DormOccupants,
    OwnRoomOccupants,
    DeployedTag { tag: String, unit_cap: f64 },
    Flat { weight: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResourceOp {
    Generate {
        resource: String,
        basis: PoolBasis,
    },
    Convert {
        from: String,
        to: String,
        ratio: f64,
    },
    Consume {
        resource: String,
    },
}

/// Peer-scaling stages
pub const PEER_STAGE_MIRROR: u8 = 0;
pub const PEER_STAGE_FIXED_LIMIT: u8 = 1;
pub const PEER_STAGE_LIMIT_CUT: u8 = 2;
pub const PEER_STAGE_NET_LIMIT: u8 = 3;

const SHIFT_FILL_AVERAGE: f64 = 0.5;

#[derive(Clone, Debug, PartialEq)]
pub enum ClauseKind {
    SelfValue,
    TimeRamp {
        first_hour: f64,
        per_hour: f64,
    },
    RoomTypeGlobal {
        target_room: String,
        gate: Option<Gate>,
    },
    ScalingCount {
        subject: Subject,
        include_self: bool,
    },
    ScalingLevelSum {
        room: String,
    },
    ScalingRoomCount {
        room: String,
    },
    ScalingPoolPoints {
        resource: String,
        step: f64,
    },
    RequiresChar {
        chars: Vec<String>,
        scope: CondScope,
    },
    RequiresTag {
        tag: String,
        scope: CondScope,
    },
    RequiresAbsentTag {
        subject: Subject,
    },
    RequiresCountTag {
        tag: String,
        count: usize,
    },
    ResourceConvert(ResourceOp),
    SuppressesOthers {
        metrics: Vec<Metric>,
        exempt: SuppressExempt,
    },
    ExcludesBuffs {
        prefixes: Vec<String>,
    },
    ScalingPeerMetric {
        metric: Metric,
        include_self: bool,
        step: f64,
        stage: u8,
        quantized: bool,
        own_skills_only: bool,
    },
    ScalingRoomOrderLimit,
    GrantsTag {
        from: Vec<String>,
        to: String,
    },
    Unresolved,
    OrderMix(OrderEffect),
}

/// Synthetic `facility_counts` key: hours a crew stays seated
pub const TENURE_HOURS: &str = "TENURE_HOURS";

#[must_use]
pub fn ramp_average(first_hour: f64, per_hour: f64, cap: f64, hours: f64) -> f64 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = hours.round().max(1.0) as usize;
    #[allow(clippy::cast_precision_loss)]
    let total: f64 = (1..=n)
        .map(|k| per_hour.mul_add(k as f64 - 1.0, first_hour).min(cap))
        .sum();
    #[allow(clippy::cast_precision_loss)]
    let avg = total / n as f64;
    avg
}

/// One resolved bonus clause
#[derive(Clone, Debug, PartialEq)]
pub struct Clause {
    pub buff_id: String,
    pub owner_room_type: String,
    pub metric: Metric,
    pub kind: ClauseKind,
    pub value: f64,
    pub cap: Option<f64>,
    pub output_targets: Vec<String>,
    pub non_stacking_family: Option<String>,
}

pub type ClauseSet = Vec<Clause>;

impl Clause {
    fn base(buff_id: &str, buff: &Buff, metric: Metric, kind: ClauseKind, value: f64) -> Self {
        Self {
            buff_id: buff_id.to_string(),
            owner_room_type: buff.room_type.clone(),
            metric,
            kind,
            value,
            cap: None,
            output_targets: buff.targets.clone(),
            non_stacking_family: None,
        }
    }
}

fn cc_non_stacking_family(buff_id: &str, buff: &Buff) -> Option<String> {
    let d = buff.description.to_lowercase();
    let non_stacking = d.contains("only the most effective")
        || d.contains("strongest effect of this type")
        || d.contains("only the strongest");
    non_stacking.then(|| buff_family(buff_id).to_string())
}

/// Legacy [`BuffResolutionStrategy`] -> clauses
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn clauses_from_strategy(
    buff_id: &str,
    buff: &Buff,
    strategy: &BuffResolutionStrategy,
) -> ClauseSet {
    use BuffResolutionStrategy as S;
    let speed = || Metric::speed_for_room(&buff.room_type);
    let mut out: ClauseSet = Vec::new();
    match strategy {
        S::DirectEfficiency { value } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::SelfValue,
                *value,
            ));
        }

        S::FacilityCountScaling {
            target_room,
            per_unit_pct,
            per_level,
            nullifies_others,
            base_pct,
            cap_pct,
        } => {
            if *base_pct != 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::SelfValue,
                    *base_pct,
                ));
            }
            if *per_unit_pct != 0.0 {
                let kind = if *per_level {
                    ClauseKind::ScalingLevelSum {
                        room: target_room.clone(),
                    }
                } else {
                    ClauseKind::ScalingRoomCount {
                        room: target_room.clone(),
                    }
                };
                let mut c = Clause::base(buff_id, buff, speed(), kind, *per_unit_pct);
                c.cap = *cap_pct;
                out.push(c);
            }
            if *nullifies_others {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::SuppressesOthers {
                        metrics: vec![speed()],
                        exempt: SuppressExempt::RoomCountScaledSources,
                    },
                    0.0,
                ));
            }
        }

        S::TeammateSkillScaling {
            target_buff_pattern,
            per_match_pct,
        } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingCount {
                    subject: Subject::SkillIdPrefix(target_buff_pattern.clone()),
                    include_self: false,
                },
                *per_match_pct,
            ));
        }

        S::TeammateOutputMirroring {
            ratio,
            step,
            cap_pct,
        } => {
            let mut c = Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingPeerMetric {
                    metric: speed(),
                    include_self: false,
                    step: *step,
                    stage: PEER_STAGE_MIRROR,
                    quantized: true,
                    own_skills_only: true,
                },
                *ratio,
            );
            c.cap = Some(*cap_pct);
            out.push(c);
        }

        S::OrderDifferenceScaling { per_order_pct } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingRoomOrderLimit,
                per_order_pct * SHIFT_FILL_AVERAGE,
            ));
        }

        S::LimitCutPerPeerEfficiency {
            pct_per_cut,
            cut,
            per_order_pct,
        } => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::CapacityLimit,
                ClauseKind::ScalingPeerMetric {
                    metric: speed(),
                    include_self: false,
                    step: *pct_per_cut,
                    stage: PEER_STAGE_LIMIT_CUT,
                    quantized: true,
                    own_skills_only: false,
                },
                f64::from(*cut),
            ));
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingRoomOrderLimit,
                per_order_pct * SHIFT_FILL_AVERAGE,
            ));
        }

        S::EfficiencyWithOrderLimit {
            efficiency,
            order_limit,
        } => {
            if *efficiency != 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::SelfValue,
                    *efficiency,
                ));
            }
            if *order_limit != 0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    Metric::CapacityLimit,
                    ClauseKind::SelfValue,
                    f64::from(*order_limit),
                ));
            }
        }

        S::ConditionalOnTeammate {
            required_char_id,
            base_efficiency,
            efficiency,
            order_limit,
        } => {
            if *base_efficiency != 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::SelfValue,
                    *base_efficiency,
                ));
            }
            let chars: Vec<String> = required_char_id.clone().into_iter().collect();
            if *efficiency != 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::RequiresChar {
                        chars: chars.clone(),
                        scope: CondScope::Room,
                    },
                    *efficiency,
                ));
            }
            if *order_limit != 0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    Metric::CapacityLimit,
                    ClauseKind::RequiresChar {
                        chars,
                        scope: CondScope::Room,
                    },
                    f64::from(*order_limit),
                ));
            }
        }

        S::ConditionalOnBaseWide {
            required_char_ids,
            base_efficiency,
            bonus_efficiency,
            anywhere,
        } => {
            if *base_efficiency != 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::SelfValue,
                    *base_efficiency,
                ));
            }
            if *bonus_efficiency != 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::RequiresChar {
                        chars: required_char_ids.clone(),
                        scope: if *anywhere {
                            CondScope::BaseAnywhere
                        } else {
                            CondScope::BaseWorkArea
                        },
                    },
                    *bonus_efficiency,
                ));
            }
        }

        S::ConditionalOnRoomPresence {
            required_char_ids,
            required_faction,
            base_efficiency,
            bonus_efficiency,
            ..
        } => {
            if *base_efficiency != 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::SelfValue,
                    *base_efficiency,
                ));
            }
            if *bonus_efficiency != 0.0 {
                if let Some(tag) = required_faction {
                    out.push(Clause::base(
                        buff_id,
                        buff,
                        speed(),
                        ClauseKind::RequiresTag {
                            tag: tag.clone(),
                            scope: CondScope::RoomTypeElsewhere,
                        },
                        *bonus_efficiency,
                    ));
                } else if !required_char_ids.is_empty() {
                    out.push(Clause::base(
                        buff_id,
                        buff,
                        speed(),
                        ClauseKind::RequiresChar {
                            chars: required_char_ids.clone(),
                            scope: CondScope::RoomTypeElsewhere,
                        },
                        *bonus_efficiency,
                    ));
                }
            }
        }

        S::OrderLimitScaling {
            per_cap_threshold,
            bonus_per_threshold,
            cap_pct,
            includes_self,
            stage,
        } => {
            let mut c = Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingPeerMetric {
                    metric: Metric::CapacityLimit,
                    include_self: *includes_self,
                    step: *per_cap_threshold,
                    stage: *stage,
                    quantized: true,
                    own_skills_only: false,
                },
                *bonus_per_threshold,
            );
            c.cap = Some(*cap_pct);
            out.push(c);
        }

        S::CapacityTierScaling {
            threshold,
            low_pct,
            high_pct,
            excludes,
        } => {
            if !excludes.is_empty() {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::ExcludesBuffs {
                        prefixes: excludes.clone(),
                    },
                    0.0,
                ));
            }
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingCount {
                    subject: Subject::CapacityPoints {
                        threshold: f64::from(*threshold),
                        above: false,
                    },
                    include_self: true,
                },
                *low_pct,
            ));
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingCount {
                    subject: Subject::CapacityPoints {
                        threshold: f64::from(*threshold),
                        above: true,
                    },
                    include_self: true,
                },
                *high_pct,
            ));
        }

        S::FacilityCountModifier {
            target_room,
            amount,
            ..
        } => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::FacilityCount(target_room.clone()),
                ClauseKind::SelfValue,
                f64::from(*amount),
            ));
        }

        S::MatchCountScaling {
            token,
            per_match_pct,
            cap_pct,
            bonus_char_id,
            bonus_pct,
            count_skills,
            capacity,
        } => {
            let (subject, include_self) = if *count_skills {
                (Subject::SkillTag(token.clone()), true)
            } else {
                (
                    Subject::Tag(token.clone()),
                    !buff.description.to_lowercase().contains("other"),
                )
            };
            let mut c = Clause::base(
                buff_id,
                buff,
                if *capacity {
                    Metric::CapacityLimit
                } else {
                    speed()
                },
                ClauseKind::ScalingCount {
                    subject,
                    include_self,
                },
                *per_match_pct,
            );
            c.cap = *cap_pct;
            out.push(c);
            if let Some(rider) = bonus_char_id
                && *bonus_pct != 0.0
            {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::RequiresChar {
                        chars: vec![rider.clone()],
                        scope: CondScope::Room,
                    },
                    *bonus_pct,
                ));
            }
        }

        S::ConditionalOnFaction {
            faction_token,
            base_efficiency,
            efficiency,
        } => {
            if *base_efficiency != 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::SelfValue,
                    *base_efficiency,
                ));
            }
            if *efficiency != 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::RequiresTag {
                        tag: faction_token.clone(),
                        scope: CondScope::Room,
                    },
                    *efficiency,
                ));
            }
        }

        S::SkillTypeConversion {
            from_tokens,
            to_token,
        } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::GrantsTag {
                    from: from_tokens.clone(),
                    to: to_token.clone(),
                },
                0.0,
            ));
        }

        S::NullifyTeammatesSelfScaling { per_teammate_pct } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingCount {
                    subject: Subject::AnyOtherOccupant,
                    include_self: false,
                },
                *per_teammate_pct,
            ));
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::SuppressesOthers {
                    metrics: vec![
                        Metric::TradingSpeed,
                        Metric::ManufactureSpeed,
                        Metric::OrderValue { pure_gold: true },
                    ],
                    exempt: SuppressExempt::None,
                },
                0.0,
            ));
        }

        S::OrderValue { effect, pure_gold } => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::OrderValue {
                    pure_gold: *pure_gold,
                },
                ClauseKind::OrderMix(effect.clone()),
                0.0,
            ));
        }

        S::GlobalPoolScaling {
            target_room,
            base_pct,
            ..
        } => {
            if *base_pct != 0.0 {
                let mut c = Clause::base(
                    buff_id,
                    buff,
                    Metric::speed_for_room(target_room),
                    ClauseKind::RoomTypeGlobal {
                        target_room: target_room.clone(),
                        gate: None,
                    },
                    *base_pct,
                );
                c.non_stacking_family = cc_non_stacking_family(buff_id, buff);
                out.push(c);
            }
        }

        S::GlobalEffect {
            target_room,
            bonus_pct,
        }
        | S::GlobalEffectWithCrewTag {
            target_room,
            bonus_pct,
            ..
        } => {
            let mut c = Clause::base(
                buff_id,
                buff,
                Metric::speed_for_room(target_room),
                ClauseKind::RoomTypeGlobal {
                    target_room: target_room.clone(),
                    gate: None,
                },
                *bonus_pct,
            );
            c.non_stacking_family = cc_non_stacking_family(buff_id, buff);
            out.push(c);
        }

        S::ConditionalGlobalEffect {
            target_room,
            faction_token,
            required_count,
            per_operator,
            bonus_pct,
            order_limit,
            ..
        } => {
            let gate = Gate {
                tag: faction_token.clone(),
                required_count: *required_count,
                per_operator: *per_operator,
            };
            let mut c = Clause::base(
                buff_id,
                buff,
                Metric::speed_for_room(target_room),
                ClauseKind::RoomTypeGlobal {
                    target_room: target_room.clone(),
                    gate: Some(gate.clone()),
                },
                *bonus_pct,
            );
            c.non_stacking_family = cc_non_stacking_family(buff_id, buff);
            out.push(c);
            if *order_limit != 0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    Metric::CapacityLimit,
                    ClauseKind::RoomTypeGlobal {
                        target_room: target_room.clone(),
                        gate: Some(gate),
                    },
                    f64::from(*order_limit),
                ));
            }
        }

        S::LayoutCountBranch { .. }
        | S::RoomPresenceGatedGlobal { .. }
        | S::BaseWideMatchCountScaling { .. }
        | S::NamedTargetRoomBoost { .. } => {}

        S::NamedCharRoomGrants { grants } => {
            for g in grants {
                if g.order_limit != 0.0 {
                    out.push(Clause::base(
                        buff_id,
                        buff,
                        Metric::CapacityLimit,
                        ClauseKind::RoomTypeGlobal {
                            target_room: g.target_room.clone(),
                            gate: Some(Gate {
                                tag: g.char_id.clone(),
                                required_count: 1,
                                per_operator: false,
                            }),
                        },
                        g.order_limit,
                    ));
                }
                if g.nonprod_pct != 0.0 {
                    let mut c = Clause::base(
                        buff_id,
                        buff,
                        Metric::speed_for_room(&g.target_room),
                        ClauseKind::RoomTypeGlobal {
                            target_room: g.target_room.clone(),
                            gate: Some(Gate {
                                tag: g.char_id.clone(),
                                required_count: 1,
                                per_operator: false,
                            }),
                        },
                        g.nonprod_pct,
                    );
                    c.non_stacking_family = cc_non_stacking_family(buff_id, buff);
                    out.push(c);
                }
            }
        }

        S::MoraleDrainAuraImmunity => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::MoraleDrainAuraImmunity,
                ClauseKind::SelfValue,
                1.0,
            ));
        }

        S::TagBased {
            tag,
            bonus_pct,
            target_room,
        } => {
            let mut c = Clause::base(
                buff_id,
                buff,
                Metric::speed_for_room(target_room),
                ClauseKind::RoomTypeGlobal {
                    target_room: target_room.clone(),
                    gate: Some(Gate {
                        tag: tag.clone(),
                        required_count: 1,
                        per_operator: true,
                    }),
                },
                *bonus_pct,
            );
            c.non_stacking_family = cc_non_stacking_family(buff_id, buff);
            out.push(c);
        }

        S::MoraleModifier {
            recovery_per_hour,
            base_wide,
            ..
        } => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::MoraleRecovery {
                    base_wide: *base_wide,
                },
                ClauseKind::SelfValue,
                *recovery_per_hour,
            ));
        }

        S::CapacityOnly { order_limit } => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::CapacityLimit,
                ClauseKind::SelfValue,
                f64::from(*order_limit),
            ));
        }

        S::NonProduction { value } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::SelfValue,
                *value,
            ));
        }

        S::ControlNonProduction {
            target_room,
            value,
            same_room_gate,
        } => {
            let kind = match same_room_gate {
                Some(chars) => ClauseKind::RequiresChar {
                    chars: chars.clone(),
                    scope: CondScope::Room,
                },
                None => ClauseKind::SelfValue,
            };
            let mut c = Clause::base(
                buff_id,
                buff,
                Metric::speed_for_room(target_room),
                kind,
                *value,
            );
            if cc_non_stacking_family(buff_id, buff).is_some() {
                c.non_stacking_family = Some(format!("cc_nonprod_{target_room}"));
            }
            out.push(c);
        }

        S::Complex { estimated_pct } => {
            if UNRESOLVED_CARRIES_LEGACY_ESTIMATE && *estimated_pct != 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::SelfValue,
                    *estimated_pct,
                ));
            } else {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::Unresolved,
                    0.0,
                ));
            }
        }

        S::MoraleDecayEfficiency {
            time_averaged_value,
        } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::SelfValue,
                *time_averaged_value,
            ));
        }

        S::TimeRamp {
            first_hour,
            per_hour,
            cap,
        } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::TimeRamp {
                    first_hour: *first_hour,
                    per_hour: *per_hour,
                },
                *cap,
            ));
        }

        S::PoolPayoff { pct } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ResourceConvert(ResourceOp::Consume {
                    resource: PERCEPTION_POOL.to_string(),
                }),
                *pct,
            ));
        }

        S::PoolGenerateBuildingLevels {
            resource,
            per_level,
            cap,
        } => {
            let mut c = Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ResourceConvert(ResourceOp::Generate {
                    resource: resource.clone(),
                    basis: PoolBasis::FunctionalLevels,
                }),
                *per_level,
            );
            c.cap = Some(*cap);
            out.push(c);
        }

        S::RoomPerOperatorGrant {
            speed_pct,
            capacity,
        } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::SuppressesOthers {
                    metrics: vec![speed()],
                    exempt: SuppressExempt::RoomCountScaledSources,
                },
                0.0,
            ));
            if *speed_pct > 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    speed(),
                    ClauseKind::ScalingCount {
                        subject: Subject::AnyOtherOccupant,
                        include_self: true,
                    },
                    *speed_pct,
                ));
            }
            if *capacity > 0.0 {
                out.push(Clause::base(
                    buff_id,
                    buff,
                    Metric::CapacityLimit,
                    ClauseKind::ScalingCount {
                        subject: Subject::AnyOtherOccupant,
                        include_self: true,
                    },
                    *capacity,
                ));
            }
        }

        S::AccountFacts { .. } => {}

        S::PoolGenerateOwnRoomOccupants { resource, per } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ResourceConvert(ResourceOp::Generate {
                    resource: resource.clone(),
                    basis: PoolBasis::OwnRoomOccupants,
                }),
                *per,
            ));
        }

        S::PoolGenerateOwnRoomLevel {
            resource,
            per_level,
        } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ResourceConvert(ResourceOp::Generate {
                    resource: resource.clone(),
                    basis: PoolBasis::OwnRoomLevel,
                }),
                *per_level,
            ));
        }

        S::PoolDormEconomy {
            resource,
            per_occupant,
            per,
            pct,
        } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ResourceConvert(ResourceOp::Generate {
                    resource: resource.clone(),
                    basis: PoolBasis::DormOccupants,
                }),
                *per_occupant,
            ));
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingPoolPoints {
                    resource: resource.clone(),
                    step: *per,
                },
                *pct,
            ));
        }

        S::PoolGenerateDormAndConvert {
            gen_resource,
            per_occupant,
            to_resource,
            from_per,
        } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ResourceConvert(ResourceOp::Generate {
                    resource: gen_resource.clone(),
                    basis: PoolBasis::DormOccupants,
                }),
                *per_occupant,
            ));
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ResourceConvert(ResourceOp::Convert {
                    from: gen_resource.clone(),
                    to: to_resource.clone(),
                    ratio: *from_per,
                }),
                0.0,
            ));
        }

        S::MoraleRoomAura { delta } => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::MoraleDrainAura,
                ClauseKind::SelfValue,
                *delta,
            ));
        }

        S::DormRecoveryAura { rate } => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::DormRecoveryAura,
                ClauseKind::SelfValue,
                *rate,
            ));
        }

        S::PerTeammateEfficiency { per_teammate_pct } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingCount {
                    subject: Subject::AnyOtherOccupant,
                    include_self: false,
                },
                *per_teammate_pct,
            ));
        }

        S::PoolConvert { from, to, from_per } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ResourceConvert(ResourceOp::Convert {
                    from: from.clone(),
                    to: to.clone(),
                    ratio: *from_per,
                }),
                0.0,
            ));
        }

        S::PoolTail {
            base,
            resource,
            pct,
            per,
        } => {
            out.extend(clauses_from_strategy(buff_id, buff, base));
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingPoolPoints {
                    resource: resource.clone(),
                    step: *per,
                },
                *pct,
            ));
        }

        S::PoolPointsScaling { resource, per, pct } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingPoolPoints {
                    resource: resource.clone(),
                    step: *per,
                },
                *pct,
            ));
        }
    }
    out
}

pub const PERCEPTION_POOL: &str = "PERCEPTION_INFO";

pub const UNRESOLVED_CARRIES_LEGACY_ESTIMATE: bool = false;

#[must_use]
pub fn build_clauses(
    buffs: &HashMap<String, Buff>,
    name_to_char: &HashMap<String, String>,
) -> HashMap<String, ClauseSet> {
    let (registry, morale_drains) = crate::registry::build_registry(buffs, name_to_char);
    let mut out: HashMap<String, ClauseSet> = HashMap::new();
    for (buff_id, strategy) in &registry {
        let Some(buff) = buffs.get(buff_id) else {
            continue;
        };
        let mut set = clauses_from_strategy(buff_id, buff, strategy);
        if !set.is_empty()
            && set.iter().all(|c| matches!(c.kind, ClauseKind::Unresolved))
            && (morale_drains.get(buff_id).is_some_and(|d| *d != 0.0)
                || crate::scoring::pools::has_side_channel_grant(&buff.description)
                || crate::registry::has_targeted_morale_effect(&buff.description))
        {
            set.clear();
        }
        if let Some(drain) = morale_drains.get(buff_id)
            && *drain != 0.0
        {
            set.push(Clause {
                buff_id: buff_id.clone(),
                owner_room_type: buff.room_type.clone(),
                metric: Metric::MoraleDrainDelta,
                kind: ClauseKind::SelfValue,
                value: *drain,
                cap: None,
                output_targets: buff.targets.clone(),
                non_stacking_family: None,
            });
        }
        out.insert(buff_id.clone(), set);
    }
    out
}

/// Buffs with an [`ClauseKind::Unresolved`] clause
#[must_use]
pub fn unresolved_buffs(clauses: &HashMap<String, ClauseSet>) -> Vec<&str> {
    let mut ids: Vec<&str> = clauses
        .iter()
        .filter(|(_, set)| set.iter().any(|c| matches!(c.kind, ClauseKind::Unresolved)))
        .map(|(id, _)| id.as_str())
        .collect();
    ids.sort_unstable();
    ids
}
