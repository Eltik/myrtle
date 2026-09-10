//! The clause data model: every base-skill bonus, no matter how it is worded,
//! reduces at parse time to one or more CLAUSES drawn from a small fixed
//! taxonomy of shapes. The scoring engine (`ledger.rs`) knows only these shapes -
//! adding a new operator or bonus means classifying text into an existing shape
//! (or, rarely, extending the fixed set), never writing new scoring code.
//!
//! Principles carried from the algorithm reference:
//! - NEVER GUESS: a bonus that cannot be cleanly parsed becomes [`ClauseKind::Unresolved`]
//!   and contributes exactly zero - a confidently wrong number is worse than a
//!   visibly incomplete one.
//! - One buff may emit SEVERAL clauses (Hoederer: a flat base plus a base-wide
//!   presence rider; Weedy: facility scaling plus a suppression of teammates).
//! - Weighting/polarity to LMD-equivalents happens exactly once, downstream in
//!   `yield_model` / `room_search_score` - never here.

use std::collections::HashMap;

use crate::core::gamedata::types::building::Buff;

use super::buff_registry::{BuffResolutionStrategy, OrderEffect};

/// What a clause's contribution is denominated in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Metric {
    /// Factory productivity %.
    ManufactureSpeed,
    /// Trading Post order-acquisition %.
    TradingSpeed,
    /// % LMD-per-order, resolved at scoring time from the room's order
    /// shapes and the post's level (`order_mix`). `pure_gold` marks a value
    /// tied to Pure-Gold orders specifically (Proviso), which a
    /// Precious-Metal-shifting suppressor (Shamare) kills.
    OrderValue { pure_gold: bool },
    /// Order/capacity-limit points (sign carries polarity - Degenbrecher's -6).
    CapacityLimit,
    /// Power Plant drone-recovery %.
    DroneRecovery,
    /// Morale recovered per hour. `base_wide` = reaches operators outside the
    /// owner's room (Control-Center "all other facilities" auras).
    MoraleRecovery { base_wide: bool },
    /// Morale drain modifier per hour (replaces the old `morale_drains` side-map;
    /// positive = drains faster).
    MoraleDrainDelta,
    /// A room-wide drain modifier: EVERY occupant of the owner's room drains
    /// `value` more (negative = slower) per hour while the owner is seated
    /// ("Morale consumed per hour of all Operators in the Factory -0.1").
    MoraleDrainAura,
    /// Marker: the holder IGNORES roommates' `MoraleDrainAura` effects on its
    /// own drain in the owner's room type (Waai Fu's Team Spirit). Value is
    /// unused; presence is the whole effect.
    MoraleDrainAuraImmunity,
    /// A Control-Center aura raising every DORMITORY sleeper's recovery rate
    /// ("all Operators in Dormitories recover +0.05 Morale per hour") - the
    /// game's non-stacking clause applies, so consumers take the maximum.
    DormRecoveryAura,
    /// Non-production facility value (clue search, HR contact, ...).
    NonProduction(NonProdKind),
    /// Raises a room type's EFFECTIVE facility count ("Power Plant +1, only
    /// affects facility quantity"). Consumed by the context builder, not summed
    /// into room output.
    FacilityCount(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonProdKind {
    ClueSearch, // MEETING
    HrContact,  // HIRE
    Training,   // TRAINING
    Workshop,   // WORKSHOP
    Other,
}

/// How multiple surviving entries of one metric combine in a room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombineRule {
    /// Ordinary additive stacking.
    Sum,
    /// Resolved jointly through the order-mix model (order VALUE): the room's
    /// whole set of order shapes is priced together against the post's level,
    /// so disjoint-order effects compose and same-kind ones take the strongest.
    OrderMix,
}

impl Metric {
    /// The natural "speed" metric of a room type, for flat efficiency clauses.
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

    pub const fn combine(&self) -> CombineRule {
        match self {
            Self::OrderValue { .. } => CombineRule::OrderMix,
            _ => CombineRule::Sum,
        }
    }
}

/// Who a scaling or requirement clause counts / checks for.
#[derive(Clone, Debug, PartialEq)]
pub enum Subject {
    /// A faction / skill-type token matched against an operator's `match_tags`
    /// ("glasgow", "standardization", "rhine").
    Tag(String),
    /// A skill-type token matched against the leading word of an operator's
    /// SKILL names only ("rhine" = a Rhine Tech skill), never its faction -
    /// "for each Rhine Tech-type skill in this Factory" counts skills.
    SkillTag(String),
    /// A buff-id prefix matched against teammates' skills ("+5% per
    /// Standardization skill" via id patterns).
    SkillIdPrefix(String),
    /// Every other occupant of the room, regardless of identity (Shamare's
    /// "each Operator", Bubble's per-occupant tiers).
    AnyOtherOccupant,
    /// Specific resolved operators. Empty = the named operator could not be
    /// resolved: the clause then contributes 0 (never guess).
    Chars(Vec<String>),
    /// Occupants whose own total of `metric` exceeds `threshold` (Bubble's
    /// high-capacity tier).
    PeerMetricAbove { metric: Metric, threshold: f64 },
}

/// Where a presence requirement looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CondScope {
    /// The clause owner's own room.
    Room,
    /// Anywhere the operator is actively WORKING (any non-dormitory room of the
    /// deployment - Hoederer's "assigned to any Work Area").
    BaseWorkArea,
    /// A specific room TYPE elsewhere in the base ("if Kal'tsit is assigned to
    /// the Control Center"). The optimizer resolves these by registry rewrite
    /// (`resolve_room_presence`) once the deployment is known; a context-free
    /// scoring pass credits the gated part exactly 0 (never guess). The room
    /// type lives on the strategy, not here, so the scope stays `Copy`.
    RoomTypeElsewhere,
}

/// A occupancy condition attached to a room-type-global clause (Control-Center
/// buffs gated on the TARGET room's team: "all Trading Posts with 3 Kjerag
/// Operators", "all Siracusa Operators in Trading Posts").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gate {
    pub tag: String,
    pub required_count: usize,
    /// True: the value applies once per matching occupant. False: the whole
    /// room gains the value once the count threshold is met.
    pub per_operator: bool,
}

/// Which contributions a suppressor leaves standing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuppressExempt {
    /// Nothing is exempt (Shamare zeroes every listed metric outright).
    None,
    /// Contributions whose source clause scales on a facility count survive
    /// (Weedy automation: "excluding productivity granted based on facility
    /// count").
    RoomCountScaledSources,
}

/// What a pool generator's points multiply against.
#[derive(Clone, Debug, PartialEq)]
pub enum PoolBasis {
    /// The summed level of the base's functional facilities (layout-derived,
    /// settled room-locally wherever the generator's clause is live).
    FunctionalLevels,
    /// The level of the room the GENERATOR is seated in (Senshi's "per level
    /// of the current Dormitory") - settled at assignment scope, where the
    /// seat is known.
    OwnRoomLevel,
    /// The number of operators seated in the base's dormitories (Rosmontis'
    /// Perception Information, Mr. Nothing's Worldly Plight).
    DormOccupants,
    /// Operators outside the dormitories carrying a faction tag, counted up
    /// to `unit_cap` (Chongyue: "+5 per Sui Operator ... (max 5)").
    DeployedTag { tag: String, unit_cap: f64 },
    /// A flat grant scaled by the steady-state fraction of a work block its
    /// morale condition holds (Dusk/Ling's "when own Morale is above/below
    /// 12": at baseline drain a 24-point bar spends half its block on each
    /// side, so both branches settle at 0.5). A documented model, not a guess.
    Flat { weight: f64 },
}

/// A resource-pool operation (the perception economies: Perception Information,
/// Chain of Thought, Worldly Plight, ...).
#[derive(Clone, Debug, PartialEq)]
pub enum ResourceOp {
    /// Adds `value` points to `resource` per unit of `basis`.
    Generate { resource: String, basis: PoolBasis },
    /// Moves points from one pool into another at a ratio.
    Convert {
        from: String,
        to: String,
        ratio: f64,
    },
    /// Drains a pool into the clause's real metric at `value` % per point.
    Consume { resource: String },
}

/// The fixed shape taxonomy. Everything the scorer knows how to do lives here.
#[derive(Clone, Debug, PartialEq)]
pub enum ClauseKind {
    /// Flat bonus to the owner's own room. Resolves against nothing.
    SelfValue,
    /// Fans out once per room of `target_room` (Control-Center globals). An
    /// optional `gate` conditions each target room on its own occupants.
    RoomTypeGlobal {
        target_room: String,
        gate: Option<Gate>,
    },
    /// `value` applies once per matching entity present in the owner's room.
    ScalingCount {
        subject: Subject,
        include_self: bool,
    },
    /// `value` applies once per level, summed across every built room of a type
    /// (dormitory-level scaling).
    ScalingLevelSum { room: String },
    /// `value` applies once per unit of a room-count-like quantity: real room
    /// counts ("per Power Plant") and the synthetic counts the context builder
    /// provides (`MANUFACTURE_RECIPE_TYPES`, `DRONE_CAPACITY`).
    ScalingRoomCount { room: String },
    /// `value` applies once per `step` settled points of a named pool resource,
    /// floored ("+5% per 8 Engineering Robots"). Points come from the room's
    /// live generator clauses (layout-derived pools) or, later, the
    /// assignment-scope pool settlement.
    ScalingPoolPoints { resource: String, step: f64 },
    /// Binary gate: full `value` when one of `chars` is present in scope.
    /// Empty `chars` never fires (unresolvable name - never guess).
    RequiresChar {
        chars: Vec<String>,
        scope: CondScope,
    },
    /// Binary gate on any occupant carrying a faction tag.
    RequiresTag { tag: String, scope: CondScope },
    /// Full value only when nothing matching is present (reception solo skills:
    /// "if no other Operators are working").
    RequiresAbsentTag { subject: Subject },
    /// Binary threshold gate: full value once `count`+ matches are present in
    /// the owner's room.
    RequiresCountTag { tag: String, count: usize },
    /// Pool operation, settled by the ledger's fixed-point pool pass.
    ResourceConvert(ResourceOp),
    /// Zeroes every OTHER occupant's contribution of the listed metrics in the
    /// owner's room, leaving the owner untouched. Runs strictly last.
    SuppressesOthers {
        metrics: Vec<Metric>,
        exempt: SuppressExempt,
    },
    /// The owner's bonus scales off a total that roommates are already
    /// contributing - resolved by delta-relaxation because it can be circular.
    /// `step` quantizes the basis ("per 5 CAP" -> floor(total/5)).
    ScalingPeerMetric {
        metric: Metric,
        include_self: bool,
        step: f64,
    },
    /// While the owner is present, occupants matching a `from` tag also carry
    /// the `to` tag (Highmore's skill-type conversion). Affects other clauses'
    /// subject resolution; contributes no value itself.
    GrantsTag { from: Vec<String>, to: String },
    /// The parse failed. Contributes exactly zero and is surfaced in the
    /// diagnostics list so coverage gaps get real parsers, not guesses.
    Unresolved,
    /// An order-VALUE shape (Proviso, Tequila, Tailoring), resolved at
    /// scoring time against the post's order rarity together with every
    /// other order shape in the room - `order_mix::value_pct`. The clause's
    /// own `value` is unused.
    OrderMix(OrderEffect),
}

/// One resolved bonus clause.
#[derive(Clone, Debug, PartialEq)]
pub struct Clause {
    pub buff_id: String,
    /// Room type the OWNER must occupy for this clause to be live at all.
    pub owner_room_type: String,
    pub metric: Metric,
    pub kind: ClauseKind,
    /// Per-unit / per-match / flat magnitude; sign already carries polarity.
    pub value: f64,
    /// Ceiling on the scaled part when the text states one ("Max +25%").
    pub cap: Option<f64>,
    /// Product formulas this clause is restricted to (from `Buff.targets`,
    /// e.g. `F_GOLD`) - drives the three-tier configuration discount. Empty =
    /// generic, always full value.
    pub output_targets: Vec<String>,
    /// Set when the game's non-stacking clause applies ("only the strongest
    /// effect of this type"): entries sharing a family keep only the strongest.
    pub non_stacking_family: Option<String>,
}

pub type ClauseSet = Vec<Clause>;

impl Clause {
    /// A bare clause with the fields every emission shares; callers override
    /// the rest by struct update.
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

/// The non-stacking family for a Control-Center global, when the buff text
/// carries the game's "only the strongest of this type applies" clause. Family
/// key = the buff-id prefix (same family across tiers/carriers).
fn cc_non_stacking_family(buff_id: &str, buff: &Buff) -> Option<String> {
    let d = buff.description.to_lowercase();
    let non_stacking = d.contains("only the most effective")
        || d.contains("strongest effect of this type")
        || d.contains("only the strongest");
    non_stacking.then(|| buff_id.split('[').next().unwrap_or(buff_id).to_string())
}

/// Map one legacy [`BuffResolutionStrategy`] to its clause set - the migration
/// adapter that guarantees the clause model can express everything the old
/// taxonomy could. Dies once `build_clauses` parses buff text directly.
#[allow(clippy::too_many_lines)]
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
                // The automation survival rule, made declarative: teammates'
                // speed is zeroed EXCEPT contributions that scale on facility
                // counts ("excluding productivity granted based on facility
                // count") - so automation operators stack with each other and
                // with facility-count scalers like Purestream.
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

        S::TeammateOutputMirroring { ratio, cap_pct } => {
            let mut c = Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingPeerMetric {
                    metric: speed(),
                    include_self: false,
                    step: 1.0,
                },
                *ratio,
            );
            c.cap = Some(*cap_pct);
            out.push(c);
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
            // An unresolved name emits an empty chars list, which never fires.
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
                        scope: CondScope::BaseWorkArea,
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
            // The gated part scores 0 without deployment context; the optimizer
            // credits it by rewriting the strategy (resolve_room_presence).
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
        } => {
            let mut c = Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingPeerMetric {
                    metric: Metric::CapacityLimit,
                    include_self: *includes_self,
                    step: *per_cap_threshold,
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
        } => {
            // Decompose "low below the threshold, high above" into
            // all x low + above-threshold x (high - low), keeping both halves
            // inside the ScalingCount shape.
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingCount {
                    subject: Subject::AnyOtherOccupant,
                    include_self: true,
                },
                *low_pct,
            ));
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingCount {
                    subject: Subject::PeerMetricAbove {
                        metric: Metric::CapacityLimit,
                        threshold: f64::from(*threshold),
                    },
                    include_self: true,
                },
                *high_pct - *low_pct,
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
            // Skill counts include the holder's own skill (Dorothy's Rhine
            // Tech β counts toward her per-Rhine-Tech-skill bonus). Operator
            // counts "in the same room" include the holder too when they
            // carry the tag (Morgan's Gang Compass reads +20% for herself and
            // +20% for Siege, user-verified 2026-09-10) unless the text says
            // "other".
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
            // Shamare shifts the post toward Precious-Metal orders: teammates'
            // SPEED dies, flat/Precious-Metal order value survives, Pure-Gold
            // value (Proviso) dies with the Pure-Gold orders themselves.
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
            // Context-free scoring credits the always-on base only; the pool
            // part resolves by registry rewrite (resolve_global_pool) once the
            // settled points are known.
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
            ..
        } => {
            let mut c = Clause::base(
                buff_id,
                buff,
                Metric::speed_for_room(target_room),
                ClauseKind::RoomTypeGlobal {
                    target_room: target_room.clone(),
                    gate: Some(Gate {
                        tag: faction_token.clone(),
                        required_count: *required_count,
                        per_operator: *per_operator,
                    }),
                },
                *bonus_pct,
            );
            c.non_stacking_family = cc_non_stacking_family(buff_id, buff);
            out.push(c);
        }

        // Resolved by registry rewrite before scoring (`resolve_layout_branches`,
        // `resolve_room_presence`); context-free they contribute exactly 0.
        S::LayoutCountBranch { .. }
        | S::RoomPresenceGatedGlobal { .. }
        | S::BaseWideMatchCountScaling { .. }
        | S::NamedTargetRoomBoost { .. } => {}

        S::NamedCharRoomGrants { grants } => {
            // Each grant lands on the room seating the named operator, gated
            // on that operator (the char id doubles as the gate token - the
            // occupancy matcher accepts char ids alongside faction tags). The
            // live values ride the CC-condition machinery; these clauses give
            // the taxonomy an honest record of both payloads.
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
            // `base_wide` is parsed, not inferred from the room: only the
            // "other buildings" Control-Center auras reach outside workers;
            // the `control_mp_cost` family and dorm skills stay room-local.
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
            // The metric belongs to the BOOSTED facility, not the CC the owner
            // sits in. Zero LMD weight - it feeds the CC spare-seat tie-break
            // and display only.
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
            // "Only the strongest effect of this type takes place" is one
            // family per boosted METRIC across all CC skills - upMeetingSpeed
            // +25% and meeting_spd&bd +5% share it despite different buff-id
            // prefixes, so the prefix-keyed family would be wrong here.
            if cc_non_stacking_family(buff_id, buff).is_some() {
                c.non_stacking_family = Some(format!("cc_nonprod_{target_room}"));
            }
            out.push(c);
        }

        // NEVER GUESS: the legacy estimate dies at the clause boundary. The
        // adapter still carries the estimate through until CP3 flips it off -
        // see `UNRESOLVED_CARRIES_LEGACY_ESTIMATE`.
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
            // Time-averaging over the shift is a parse-time transform; the
            // clause itself is a plain flat value.
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::SelfValue,
                *time_averaged_value,
            ));
        }

        // A solved pool payoff drains the perception pool into the room's
        // speed metric - pool output on the ledger's drain channel, never a
        // fake flat skill.
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

        // A layout-derived pool generator: its points exist wherever the owner
        // is seated, computed from the building alone. Carried as a Generate
        // clause the ledger's room-local settlement reads (value = points per
        // functional level, cap = the stated maximum).
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

        // An own-room-level generator (Senshi's Monster Meals): points depend
        // on WHERE the owner sits, so the assignment-scope settlement resolves
        // it; the room-local slice skips it.
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

        // Mr. Nothing's one-buff dorm economy: a dorm-occupant generator AND a
        // stepped consumer of the same pool.
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

        // Rosmontis' Extrasensory: a dorm-occupant generator whose points
        // convert onward into a second pool her other skills consume.
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

        // Room-wide drain aura: everyone in the owner's room drains delta more
        // (negative = slower) - consumed by the sustainability simulator.
        S::MoraleRoomAura { delta } => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::MoraleDrainAura,
                ClauseKind::SelfValue,
                *delta,
            ));
        }

        // CC dorm-recovery aura (non-stacking) - also a simulator consumer.
        S::DormRecoveryAura { rate } => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::DormRecoveryAura,
                ClauseKind::SelfValue,
                *rate,
            ));
        }

        // A per-teammate room aura: the room total gains value x (occupants-1),
        // exactly a count over the other occupants.
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

        // A standalone converter (Ancient Witchcraft: WP/5 -> Witchcraft
        // Crystal), relaxed by the settlement's fixed-point rounds.
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

        // A stepped pool consumer: +pct per `per` settled points, floored.
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

/// The resource id of the perception-information economy's pool (Rosmontis /
/// Ebenholz / Mr. Nothing). The only pool today; native generator/converter
/// clauses will join it when the perception parser is absorbed.
pub const PERCEPTION_POOL: &str = "PERCEPTION_INFO";

/// NEVER GUESS (flipped at CP3): an unparsed buff contributes exactly ZERO and
/// is surfaced through [`unresolved_buffs`] diagnostics instead of being scored
/// as a made-up flat estimate. A wrong guess silently corrupts every ranking it
/// touches; a zero shows up as a coverage gap that gets a real parser. (True
/// was the CP2 shadow-migration setting, proving ledger==legacy bit-exactly.)
pub const UNRESOLVED_CARRIES_LEGACY_ESTIMATE: bool = false;

/// Build the clause registry for every buff. Currently the guaranteed-equivalent
/// path through the legacy strategy parser; converts branch-by-branch to direct
/// text parsing as the migration proceeds.
pub fn build_clauses(
    buffs: &HashMap<String, Buff>,
    name_to_char: &HashMap<String, String>,
) -> HashMap<String, ClauseSet> {
    let (registry, morale_drains) = super::buff_registry::build_registry(buffs, name_to_char);
    let mut out: HashMap<String, ClauseSet> = HashMap::new();
    for (buff_id, strategy) in &registry {
        let Some(buff) = buffs.get(buff_id) else {
            continue;
        };
        let mut set = clauses_from_strategy(buff_id, buff, strategy);
        // A buff whose ONLY clause is Unresolved but whose effect IS captured
        // by a side-channel is fully priced - the drain map (the
        // `power_rec_spd&cost` family: "reduces the Morale consumed each hour
        // by -0.52" and nothing else) or the pool-grant scan (Dolris' "Idol's
        // Aura": purely a dorm-occupancy Passion grant). Drop the marker so
        // the unresolved inventory lists real work, not label pessimism.
        if !set.is_empty()
            && set.iter().all(|c| matches!(c.kind, ClauseKind::Unresolved))
            && (morale_drains.get(buff_id).is_some_and(|d| *d != 0.0)
                || super::pools::has_side_channel_grant(&buff.description)
                || super::buff_registry::has_targeted_morale_effect(&buff.description))
        {
            set.clear();
        }
        // The legacy side-map of morale drains becomes MoraleDrainDelta clauses
        // on the same buff, so one registry carries everything.
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

/// Diagnostics: every buff whose clause set contains an [`ClauseKind::Unresolved`]
/// entry - the honest coverage-gap list that replaces silent guessing.
pub fn unresolved_buffs(clauses: &HashMap<String, ClauseSet>) -> Vec<&str> {
    let mut ids: Vec<&str> = clauses
        .iter()
        .filter(|(_, set)| set.iter().any(|c| matches!(c.kind, ClauseKind::Unresolved)))
        .map(|(id, _)| id.as_str())
        .collect();
    ids.sort_unstable();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buff(room: &str, targets: &[&str]) -> Buff {
        Buff {
            room_type: room.to_string(),
            targets: targets.iter().map(|s| (*s).to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn direct_efficiency_maps_to_a_self_clause() {
        let b = buff("TRADING", &[]);
        let set = clauses_from_strategy(
            "t",
            &b,
            &BuffResolutionStrategy::DirectEfficiency { value: 30.0 },
        );
        assert_eq!(set.len(), 1);
        assert_eq!(set[0].kind, ClauseKind::SelfValue);
        assert_eq!(set[0].metric, Metric::TradingSpeed);
        assert!((set[0].value - 30.0).abs() < 1e-9);
    }

    #[test]
    fn automation_emits_scaling_plus_exempt_suppression() {
        // Weedy: +5% per Power Plant, teammates' productivity zeroed EXCEPT
        // facility-count-granted productivity.
        let b = buff("MANUFACTURE", &[]);
        let set = clauses_from_strategy(
            "w",
            &b,
            &BuffResolutionStrategy::FacilityCountScaling {
                target_room: "POWER".into(),
                per_unit_pct: 5.0,
                per_level: false,
                nullifies_others: true,
                base_pct: 0.0,
                cap_pct: None,
            },
        );
        assert_eq!(set.len(), 2);
        assert_eq!(
            set[0].kind,
            ClauseKind::ScalingRoomCount {
                room: "POWER".into()
            }
        );
        assert_eq!(
            set[1].kind,
            ClauseKind::SuppressesOthers {
                metrics: vec![Metric::ManufactureSpeed],
                exempt: SuppressExempt::RoomCountScaledSources,
            }
        );
    }

    #[test]
    fn quartz_style_base_plus_scaling_emits_two_clauses() {
        let b = buff("TRADING", &[]);
        let set = clauses_from_strategy(
            "q",
            &b,
            &BuffResolutionStrategy::FacilityCountScaling {
                target_room: "MANUFACTURE_RECIPE_TYPES".into(),
                per_unit_pct: 2.0,
                per_level: false,
                nullifies_others: false,
                base_pct: 30.0,
                cap_pct: None,
            },
        );
        assert_eq!(set.len(), 2);
        assert_eq!(set[0].kind, ClauseKind::SelfValue);
        assert!((set[0].value - 30.0).abs() < 1e-9);
        assert_eq!(
            set[1].kind,
            ClauseKind::ScalingRoomCount {
                room: "MANUFACTURE_RECIPE_TYPES".into()
            }
        );
    }

    #[test]
    fn shamare_emits_per_body_scaling_plus_suppression_with_value_survival() {
        let b = buff("TRADING", &[]);
        let set = clauses_from_strategy(
            "s",
            &b,
            &BuffResolutionStrategy::NullifyTeammatesSelfScaling {
                per_teammate_pct: 45.0,
            },
        );
        assert_eq!(set.len(), 2);
        assert_eq!(
            set[0].kind,
            ClauseKind::ScalingCount {
                subject: Subject::AnyOtherOccupant,
                include_self: false
            }
        );
        let ClauseKind::SuppressesOthers { metrics, exempt } = &set[1].kind else {
            panic!("expected suppression, got {:?}", set[1].kind);
        };
        // Speed dies; Pure-Gold order value dies with the orders; flat/PM value
        // survives because it is NOT in the suppressed list.
        assert!(metrics.contains(&Metric::TradingSpeed));
        assert!(metrics.contains(&Metric::OrderValue { pure_gold: true }));
        assert!(!metrics.contains(&Metric::OrderValue { pure_gold: false }));
        assert_eq!(*exempt, SuppressExempt::None);
    }

    #[test]
    fn hoederer_emits_base_plus_base_wide_requirement() {
        let b = buff("TRADING", &[]);
        let set = clauses_from_strategy(
            "h",
            &b,
            &BuffResolutionStrategy::ConditionalOnBaseWide {
                required_char_ids: vec!["char_4087_ines".into(), "char_113_cqbw".into()],
                base_efficiency: 30.0,
                bonus_efficiency: 5.0,
            },
        );
        assert_eq!(set.len(), 2);
        assert_eq!(set[0].kind, ClauseKind::SelfValue);
        assert_eq!(
            set[1].kind,
            ClauseKind::RequiresChar {
                chars: vec!["char_4087_ines".into(), "char_113_cqbw".into()],
                scope: CondScope::BaseWorkArea,
            }
        );
    }

    #[test]
    fn unresolved_teammate_name_emits_a_gate_that_never_fires() {
        let b = buff("TRADING", &[]);
        let set = clauses_from_strategy(
            "x",
            &b,
            &BuffResolutionStrategy::ConditionalOnTeammate {
                required_char_id: None,
                base_efficiency: 20.0,
                efficiency: 25.0,
                order_limit: 0,
            },
        );
        assert_eq!(set.len(), 2);
        let ClauseKind::RequiresChar { chars, .. } = &set[1].kind else {
            panic!("expected RequiresChar");
        };
        assert!(chars.is_empty(), "unresolved names must never fire");
    }

    #[test]
    fn bubble_tiers_decompose_into_two_counts() {
        let b = buff("MANUFACTURE", &[]);
        let set = clauses_from_strategy(
            "b",
            &b,
            &BuffResolutionStrategy::CapacityTierScaling {
                threshold: 4,
                low_pct: 1.0,
                high_pct: 3.0,
            },
        );
        assert_eq!(set.len(), 2);
        assert!((set[0].value - 1.0).abs() < 1e-9, "all occupants x low");
        assert!(
            (set[1].value - 2.0).abs() < 1e-9,
            "above-threshold x (high-low)"
        );
        assert!(matches!(
            &set[1].kind,
            ClauseKind::ScalingCount {
                subject: Subject::PeerMetricAbove { .. },
                include_self: true
            }
        ));
    }

    #[test]
    fn degenbrecher_negative_capacity_carries_sign() {
        let b = buff("TRADING", &[]);
        let set = clauses_from_strategy(
            "d",
            &b,
            &BuffResolutionStrategy::EfficiencyWithOrderLimit {
                efficiency: 25.0,
                order_limit: -6,
            },
        );
        assert_eq!(set.len(), 2);
        assert_eq!(set[1].metric, Metric::CapacityLimit);
        assert!((set[1].value - (-6.0)).abs() < 1e-9);
    }

    #[test]
    fn order_value_metric_combines_through_the_order_mix() {
        assert_eq!(
            Metric::OrderValue { pure_gold: false }.combine(),
            CombineRule::OrderMix
        );
        assert_eq!(Metric::TradingSpeed.combine(), CombineRule::Sum);
    }

    #[test]
    fn formula_targets_flow_into_output_targets() {
        let b = buff("MANUFACTURE", &["F_GOLD"]);
        let set = clauses_from_strategy(
            "g",
            &b,
            &BuffResolutionStrategy::DirectEfficiency { value: 30.0 },
        );
        assert_eq!(set[0].output_targets, vec!["F_GOLD".to_string()]);
    }
}
