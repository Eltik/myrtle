//! Every base-skill bonus parses to one or more CLAUSES from a small fixed set of
//! shapes, the only thing `ledger.rs` scores. A new operator means classifying
//! text into an existing shape, never new scoring code.
//!
//! - NEVER GUESS: unparseable text becomes [`ClauseKind::Unresolved`] and scores 0.
//! - One buff may emit SEVERAL clauses (Hoederer: flat base + base-wide presence
//!   rider; Weedy: facility scaling + teammate suppression).
//! - LMD weighting happens once, downstream in `yield_model` / `room_search_score`.

use std::collections::HashMap;

use crate::core::gamedata::types::building::Buff;

use super::buff_registry::{BuffResolutionStrategy, OrderEffect};
use super::util::buff_family;

/// What a clause's contribution is denominated in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Metric {
    /// Factory productivity %.
    ManufactureSpeed,
    /// Trading Post order-acquisition %.
    TradingSpeed,
    /// % LMD-per-order, priced by `order_mix`. `pure_gold` = tied to Pure-Gold
    /// orders (Proviso), which Shamare's Precious-Metal shift kills.
    OrderValue { pure_gold: bool },
    /// Order/capacity-limit points (sign carries polarity - Degenbrecher's -6).
    CapacityLimit,
    /// Power Plant drone-recovery %.
    DroneRecovery,
    /// Morale/hr. `base_wide` = reaches outside the owner's room (CC "all other
    /// facilities" auras).
    MoraleRecovery { base_wide: bool },
    /// Morale drain modifier per hour (positive = drains faster).
    MoraleDrainDelta,
    /// EVERY occupant of the owner's room drains `value`/hr more (negative =
    /// slower) ("Morale consumed per hour of all Operators in the Factory -0.1").
    MoraleDrainAura,
    /// Marker: holder IGNORES roommates' `MoraleDrainAura` (Waai Fu's Team Spirit).
    MoraleDrainAuraImmunity,
    /// CC aura on every dorm sleeper ("all Operators in Dormitories recover +0.05
    /// Morale per hour"). Non-stacking: consumers take the max.
    DormRecoveryAura,
    /// Non-production facility value (clue search, HR contact, ...).
    NonProduction(NonProdKind),
    /// EFFECTIVE facility count ("Power Plant +1, only affects facility
    /// quantity"). Read by the context builder, not summed into output.
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

/// How multiple surviving entries of one metric combine in a room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombineRule {
    /// Ordinary additive stacking.
    Sum,
    /// Priced jointly by `order_mix`.
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
    /// Leading word of SKILL names only, never faction ("for each Rhine
    /// Tech-type skill in this Factory" counts skills).
    SkillTag(String),
    /// A buff-id prefix matched against teammates' skills ("+5% per
    /// Standardization skill" via id patterns).
    SkillIdPrefix(String),
    /// Any other occupant (Shamare's "each Operator", Bubble's tiers).
    AnyOtherOccupant,
    /// Empty = unresolved name; contributes 0.
    Chars(Vec<String>),
    /// Occupants whose own total of `metric` exceeds `threshold` (Bubble's
    /// high-capacity tier).
    PeerMetricAbove { metric: Metric, threshold: f64 },
    /// Summed capacity POINTS on one side of `threshold` (Bubble's Bigger is
    /// Better!: 1%/point at or below 16, 3%/point above; Bena's +17 alone = 51%,
    /// user-verified 2026-09-10).
    CapacityPoints { threshold: f64, above: bool },
}

/// Where a presence requirement looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CondScope {
    /// The clause owner's own room.
    Room,
    /// Any non-dorm room (Hoederer's "assigned to any Work Area").
    BaseWorkArea,
    /// Anywhere incl. dorms ("when Vigil is in the Base, excluding Assistants and
    /// Activity Room users"). Rewritten by `resolve_base_wide`.
    BaseAnywhere,
    /// A room TYPE elsewhere ("if Kal'tsit is assigned to the Control Center").
    /// Rewritten by `resolve_room_presence`; context-free passes credit 0. The
    /// room type lives on the strategy so this stays `Copy`.
    RoomTypeElsewhere,
}

/// CC global gated on the TARGET room's team ("all Trading Posts with 3 Kjerag
/// Operators", "all Siracusa Operators in Trading Posts").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gate {
    pub tag: String,
    pub required_count: usize,
    /// Per matching occupant; false = whole room once the threshold is met.
    pub per_operator: bool,
}

/// Which contributions a suppressor leaves standing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuppressExempt {
    /// Nothing is exempt (Shamare zeroes every listed metric outright).
    None,
    /// Facility-count-scaled sources survive (Weedy: "excluding productivity
    /// granted based on facility count").
    RoomCountScaledSources,
}

/// What a pool generator's points multiply against.
#[derive(Clone, Debug, PartialEq)]
pub enum PoolBasis {
    /// Summed functional-facility levels (layout-derived, settled room-locally).
    FunctionalLevels,
    /// The GENERATOR's room level (Senshi's "per level of the current
    /// Dormitory"). Settled at assignment scope.
    OwnRoomLevel,
    /// Dorm occupants (Rosmontis' Perception Information, Mr. Nothing's Worldly Plight).
    DormOccupants,
    /// Occupants of the GENERATOR's room (Virtuosa's "for every 1 Operators in
    /// that Dormitory, Soundless Resonance +1").
    OwnRoomOccupants,
    /// Operators outside the dormitories carrying a faction tag, counted up
    /// to `unit_cap` (Chongyue: "+5 per Sui Operator ... (max 5)").
    DeployedTag { tag: String, unit_cap: f64 },
    /// Flat grant times the fraction of a block its morale condition holds
    /// (Dusk/Ling's "when own Morale is above/below 12": a 24-point bar at
    /// baseline drain spends half on each side, so 0.5). Documented model.
    Flat { weight: f64 },
}

/// Perception Information, Chain of Thought, Worldly Plight, ...
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

/// Peer-scaling stages, see [`ClauseKind::ScalingPeerMetric`]. Mirrors of a
/// roommate's output read the settled room (stage 0); limit readers read the
/// fixed capacity; Jaye's cut reads the efficiency those produce; the
/// net-limit reader sees the cut.
pub const PEER_STAGE_MIRROR: u8 = 0;
pub const PEER_STAGE_FIXED_LIMIT: u8 = 1;
pub const PEER_STAGE_LIMIT_CUT: u8 = 2;
pub const PEER_STAGE_NET_LIMIT: u8 = 3;

/// Jaye: Street Economics pays per EMPTY slot, Basic Needs per FILLED order, and
/// a post fills from empty between collections, so each averages half the limit
/// and both (E1+) exactly the full limit, as the game shows. Model for E0, exact E1+.
const SHIFT_FILL_AVERAGE: f64 = 0.5;

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
    /// Per unit of a room count ("per Power Plant") or a synthetic count
    /// (`MANUFACTURE_RECIPE_TYPES`, `DRONE_CAPACITY`).
    ScalingRoomCount { room: String },
    /// Per `step` settled pool points, floored ("+5% per 8 Engineering Robots").
    ScalingPoolPoints { resource: String, step: f64 },
    /// Full `value` when one of `chars` is in scope. Empty never fires.
    RequiresChar {
        chars: Vec<String>,
        scope: CondScope,
    },
    /// Binary gate on any occupant carrying a faction tag.
    RequiresTag { tag: String, scope: CondScope },
    /// Only when nothing matches ("if no other Operators are working").
    RequiresAbsentTag { subject: Subject },
    /// Full value once `count`+ matches are in the owner's room.
    RequiresCountTag { tag: String, count: usize },
    /// Pool operation, settled by the ledger's fixed-point pool pass.
    ResourceConvert(ResourceOp),
    /// Zeroes every OTHER occupant's listed metrics. Runs strictly last.
    SuppressesOthers {
        metrics: Vec<Metric>,
        exempt: SuppressExempt,
    },
    /// "Does not stack with Recycling and takes priority over it": roommates'
    /// clauses of these families are dropped before pricing.
    ExcludesBuffs { prefixes: Vec<String> },
    /// Scales off what roommates already contribute. `quantized`: "per 5 CAP" ->
    /// floor(max(0, total) / 5); otherwise a continuous mirror. A reader sees
    /// only EARLIER stages (fixed values and CC grants are stage 0), so chains
    /// resolve as the game does, not to a mutual fixed point. Measured in-game
    /// (six Kjerag/Gnosis posts, community sheet Annex 1): Degenbrecher's per-5
    /// reads fixed limits (1), Jaye's cut reads efficiency so far (2), Swire's
    /// per-point reads the limit after Jaye (3). Same-stage readers relax to a
    /// fixed point (genuinely circular).
    ScalingPeerMetric {
        metric: Metric,
        include_self: bool,
        step: f64,
        stage: u8,
        quantized: bool,
        /// Roommates' OWN skills only, no facility-count parts or CC grants:
        /// "per X provided by all other Operators assigned to that room
        /// (excluding ... facility count)". Kjerag limit readers read the full total.
        own_skills_only: bool,
    },
    /// Per order of the FINAL limit (base + every capacity delta, floored at 1).
    /// Trading only.
    ScalingRoomOrderLimit,
    /// With the owner present, `from`-tagged occupants also carry `to`
    /// (Highmore's skill-type conversion). No value of its own.
    GrantsTag { from: Vec<String>, to: String },
    /// Parse failed: zero, and listed in diagnostics so the gap gets a real parser.
    Unresolved,
    /// Order-VALUE shape (Proviso, Tequila, Tailoring), priced with the room's
    /// other shapes by `order_mix::value_pct`. `value` is unused.
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
    /// `Buff.targets` (e.g. `F_GOLD`), for the configuration discount. Empty = generic.
    pub output_targets: Vec<String>,
    /// "Only the strongest effect of this type": a family keeps its strongest.
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

/// For a CC global saying "only the strongest of this type applies". Key = the
/// buff-id prefix, shared across tiers and carriers.
fn cc_non_stacking_family(buff_id: &str, buff: &Buff) -> Option<String> {
    let d = buff.description.to_lowercase();
    let non_stacking = d.contains("only the most effective")
        || d.contains("strongest effect of this type")
        || d.contains("only the strongest");
    non_stacking.then(|| buff_family(buff_id).to_string())
}

/// Legacy [`BuffResolutionStrategy`] -> clauses. Migration adapter; dies once
/// `build_clauses` parses buff text directly.
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
                // Teammates' speed zeroed EXCEPT facility-count parts ("excluding
                // productivity granted based on facility count"), so automation
                // ops stack with each other and with scalers like Purestream.
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

        // "+5% for every 5% provided by all other Operators assigned to that
        // Factory (excluding the additional productivity affected by facility
        // count), up to a maximum of 40%" (Waai Fu; Snowsant at a post). Per FULL
        // step of roommates' own skills; facility parts and CC grants excluded.
        // Per point it read 5x and always capped (31010962: Tragodia's 35 became
        // 40 in a room the game shows at 72 with Wang's 2).
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

        // Jaye's Street Economics: +X% per empty order slot of the FINAL
        // limit, shift-averaged (see `SHIFT_FILL_AVERAGE`).
        S::OrderDifferenceScaling { per_order_pct } => {
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::ScalingRoomOrderLimit,
                per_order_pct * SHIFT_FILL_AVERAGE,
            ));
        }

        // Jaye's Basic Needs: the cut reads settled roommate efficiency (Gnosis's
        // malus, Degenbrecher's scaled part), floored per 10%; the rider pays
        // per filled order, shift-averaged.
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
            // Unresolved name -> empty list, never fires.
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
            // Gated part is 0 until resolve_room_presence rewrites the strategy.
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
            // Per capacity POINT: low rate at or below the threshold, high above.
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
            // Skill counts include the holder's own (Dorothy's Rhine Tech β counts
            // for her own bonus). "In the same room" operator counts include a
            // tagged holder unless the text says "other" (Morgan's Gang Compass:
            // +20% herself, +20% Siege, user-verified 2026-09-10).
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
            // Shamare shifts to Precious-Metal orders: teammates' SPEED dies,
            // flat/Precious-Metal value survives, Pure-Gold value (Proviso) dies.
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
            // Base only; resolve_global_pool rewrites the pool part once settled.
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

        // Gate lives in the CC bonus accumulator, where the crew is known.
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
            // Same gate for Gnosis's "+6 order limit" per Kjerag trader.
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

        // Rewritten by `resolve_layout_branches` / `resolve_room_presence`;
        // context-free they give 0.
        S::LayoutCountBranch { .. }
        | S::RoomPresenceGatedGlobal { .. }
        | S::BaseWideMatchCountScaling { .. }
        | S::NamedTargetRoomBoost { .. } => {}

        S::NamedCharRoomGrants { grants } => {
            // Gated on the named operator's room (char id doubles as the gate
            // token). Live values ride the CC conditions; these clauses just
            // record both payloads.
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
            // Parsed, not inferred: only "other buildings" CC auras reach outside;
            // `control_mp_cost` and dorm skills stay room-local.
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
            // Metric of the BOOSTED facility, not the CC. Zero LMD weight: CC
            // spare-seat tie-break and display only.
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
            // One family per boosted METRIC, not per prefix: upMeetingSpeed +25%
            // and meeting_spd&bd +5% share it.
            if cc_non_stacking_family(buff_id, buff).is_some() {
                c.non_stacking_family = Some(format!("cc_nonprod_{target_room}"));
            }
            out.push(c);
        }

        // NEVER GUESS. The legacy estimate rides through until CP3 flips
        // `UNRESOLVED_CARRIES_LEGACY_ESTIMATE` off.
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
            // Shift averaging happened at parse time.
            out.push(Clause::base(
                buff_id,
                buff,
                speed(),
                ClauseKind::SelfValue,
                *time_averaged_value,
            ));
        }

        // Pool output on the drain channel, never a fake flat skill.
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

        // Layout-derived: points from the building alone, settled room-locally
        // (value = points per functional level, cap = stated max).
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

        // Snegurochka: automation-style wipe plus per-occupant speed/capacity
        // grants with facility provenance.
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

        // Account facts carry no clause of their own.
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

        // Senshi's Monster Meals: depends on WHERE the owner sits, so settled at
        // assignment scope, not room-locally.
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

        // Mr. Nothing: generator AND stepped consumer of one pool, one buff.
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

        // Rosmontis' Extrasensory: points convert into a second pool her other
        // skills consume.
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

        // Read by the sustainability simulator.
        S::MoraleRoomAura { delta } => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::MoraleDrainAura,
                ClauseKind::SelfValue,
                *delta,
            ));
        }

        // Non-stacking; read by the simulator.
        S::DormRecoveryAura { rate } => {
            out.push(Clause::base(
                buff_id,
                buff,
                Metric::DormRecoveryAura,
                ClauseKind::SelfValue,
                *rate,
            ));
        }

        // value x (occupants - 1): a count over the other occupants.
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

        // Ancient Witchcraft: WP/5 -> Witchcraft Crystal.
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

/// Rosmontis / Ebenholz / Mr. Nothing's pool.
pub const PERCEPTION_POOL: &str = "PERCEPTION_INFO";

/// NEVER GUESS (flipped at CP3): an unparsed buff scores ZERO and shows up in
/// [`unresolved_buffs`]. A wrong guess silently corrupts every ranking; a zero is a
/// visible gap. True was the CP2 shadow setting that proved ledger == legacy.
pub const UNRESOLVED_CARRIES_LEGACY_ESTIMATE: bool = false;

/// Goes through the legacy strategy parser for now; converts to direct text
/// parsing branch by branch.
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
        // Only-Unresolved but priced by a side channel: the drain map
        // (`power_rec_spd&cost`: "reduces the Morale consumed each hour by
        // -0.52", nothing else) or the pool-grant scan (Dolris' "Idol's Aura", a
        // dorm-occupancy Passion grant). Drop the marker so the list is real work.
        if !set.is_empty()
            && set.iter().all(|c| matches!(c.kind, ClauseKind::Unresolved))
            && (morale_drains.get(buff_id).is_some_and(|d| *d != 0.0)
                || super::pools::has_side_channel_grant(&buff.description)
                || super::buff_registry::has_targeted_morale_effect(&buff.description))
        {
            set.clear();
        }
        // Fold the legacy drain side-map in so one registry carries everything.
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

/// Buffs with an [`ClauseKind::Unresolved`] clause: the coverage-gap list.
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
        // Speed and Pure-Gold value die; flat/PM value isn't in the list.
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
                anywhere: false,
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
                excludes: vec![],
            },
        );
        assert_eq!(set.len(), 2);
        assert!(
            (set[0].value - 1.0).abs() < 1e-9,
            "low per point at or below"
        );
        assert!((set[1].value - 3.0).abs() < 1e-9, "high per point above");
        assert!(matches!(
            &set[0].kind,
            ClauseKind::ScalingCount {
                subject: Subject::CapacityPoints { above: false, .. },
                include_self: true
            }
        ));
        assert!(matches!(
            &set[1].kind,
            ClauseKind::ScalingCount {
                subject: Subject::CapacityPoints { above: true, .. },
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
