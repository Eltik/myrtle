use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::core::gamedata::types::building::Buff;
use crate::core::gamedata::types::operator::Operator;

/// Build a lowercased operator-name → `char_id` lookup, used to resolve
/// named-teammate conditional buffs (the buff text references operators by
/// display name, e.g. "...the same Trading Post as Lappland").
pub fn build_name_to_char(operators: &HashMap<String, Operator>) -> HashMap<String, String> {
    operators
        .iter()
        .map(|(char_id, op)| (op.name.to_lowercase(), char_id.clone()))
        .collect()
}

/// Lowercased faction identifiers (group/nation/team id) for one operator, used
/// as match tags for count-scaling synergies.
pub fn faction_tags_of(op: &Operator) -> Vec<String> {
    let mut tags = Vec::new();
    let mut push = |s: &str| {
        if !s.is_empty() {
            tags.push(s.to_lowercase());
        }
    };
    push(&op.nation_id);
    if let Some(g) = &op.group_id {
        push(g);
    }
    if let Some(t) = &op.team_id {
        push(t);
    }
    tags
}

/// `char_id` -> faction tags, for operators built without per-op game data.
pub fn build_faction_map(operators: &HashMap<String, Operator>) -> HashMap<String, Vec<String>> {
    operators
        .iter()
        .map(|(char_id, op)| (char_id.clone(), faction_tags_of(op)))
        .collect()
}

static RE_FIRST_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").unwrap());

static RE_FIRST_FLOAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)</>").unwrap());

static RE_TAG_KEYWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.kw>(\w+)</>").unwrap());

static RE_LAST_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").unwrap());

static RE_LAST_PCT_INNER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([\d.]+)%").unwrap());

static RE_KW_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<@cc\.kw>(\d+)</>").unwrap());

static RE_VDOWN_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vdown>[+-]?([\d.]+)%?</>").unwrap());

static RE_PER_HOUR_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)%?</>\s*per hour").unwrap());

static RE_ORDER_LIMIT_POS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"order limit\s*<@cc\.vup>\+?(\d+)</>").unwrap());

static RE_ORDER_LIMIT_NEG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"order limit\s*<@cc\.vdown>-?(\d+)</>").unwrap());

static RE_NTH_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").unwrap());

static RE_VUP_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>(\d+)</>").unwrap());

// Buff text uses both "Morale consumed per hour" and "Morale consumed each
// hour" (the latter covers the Penguin Logistics trio - Texas/Lappland/Exusiai
// - among others). Accept either wording so those drains aren't silently lost.
// First keyword after "for each/every" - the thing a count-scaling buff counts.
static RE_COUNT_KEYWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"for (?:each|every).*?<@cc\.kw>([^<]+)</>").unwrap());

// Any <@cc.kw>…</> keyword (multi-word capable), used to parse converter skills.
static RE_KW_ANY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<@cc\.kw>([^<]+)</>").unwrap());

// A <@cc.kw>…</> keyword block whose contents may carry nested markup wrappers,
// e.g. Lemuen's "<@cc.kw><$cc.angel>Exusiai</></>". Non-greedy so it stops at
// the first closing tag; inner tags are stripped with RE_INNER_TAG afterward.
static RE_KW_BLOCK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<@cc\.kw>(.*?)</>").unwrap());
static RE_INNER_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap());

// A faction marker like "<$cc.g.glasgow>" (group), "<$cc.n.…>" (nation), or
// "<$cc.t.…>" (team) - captures the faction token used by match-tag scaling.
static RE_FACTION_TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<\$cc\.[gnt]\.(\w+)>").unwrap());

// Shamare-type self-scaling: "...each Operator increases … by +X%". Requires the
// per-Operator unit AND a percentage, so it won't match factory automation ops
// that scale "per Power Plant" (Weedy) or grant "+N Capacity" (Snegurochka).
static RE_NULLIFY_SELF_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"each Operator increases.*?<@cc\.vup>\+?([\d.]+)%").unwrap());

static RE_MORALE_INCREASE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Morale consumed (?:per|each) hour\s*<@cc\.vdown>\+?([\d.]+)</>").unwrap()
});

static RE_MORALE_DECREASE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Morale consumed (?:per|each) hour\s*<@cc\.vup>-?([\d.]+)</>").unwrap()
});

pub enum BuffResolutionStrategy {
    /// Efficiency field is the bonus %. Value = efficiency as f64.
    DirectEfficiency { value: f64 },

    /// Bonus scales with count of a facility type.
    /// e.g. Automation: +X% per Power Plant, nullifies other ops' productivity.
    /// e.g. Quartz: base +30% trading, plus +2% per recipe type at Factories
    ///      (approximated by the Factory count via `base_pct` + per-facility scaling).
    FacilityCountScaling {
        target_room: String,    // "POWER", "TRADING", "DORMITORY", "MANUFACTURE", etc.
        per_unit_pct: f64,      // e.g. 5.0, 10.0, 15.0
        per_level: bool,        // true for dorm-level scaling ("per level of each Dormitory")
        nullifies_others: bool, // true for Automation buffs
        base_pct: f64,          // always-on efficiency added on top of the scaling part
    },

    /// Bonus scales with teammates' skills matching a pattern.
    /// e.g. "+5% per Standardization skill in same Factory"
    TeammateSkillScaling {
        target_buff_pattern: String, // prefix to match, e.g. "manu_prod_spd"
        per_match_pct: f64,
    },

    /// Mirrors/multiplies based on teammates' total output.
    /// e.g. Heavenly Reward: "+5% per 5% from others, max +25%"
    TeammateOutputMirroring {
        ratio: f64,   // e.g. 1.0 (1:1 mirror) or 0.5
        cap_pct: f64, // max bonus, e.g. 25.0
    },

    /// Provides both efficiency AND order limit change.
    /// e.g. `SilverAsh`: "+20% efficiency, +4 order limit"
    /// e.g. Degenbrecher: "+25% efficiency, -6 order limit"
    EfficiencyWithOrderLimit {
        efficiency: f64,
        order_limit: i32, // positive = adds CAP, negative = removes CAP
    },

    /// A base efficiency that's always applied, plus a bonus efficiency / order
    /// limit that ONLY applies when a specific named operator shares the room.
    /// e.g. Texas "Feud": base 0, +65% only with Lappland.
    /// e.g. Lemuen: base +20%, +25% more only with Exusiai.
    /// e.g. Lappland "Hidden Purpose β": +4 order limit only with Texas.
    ///
    /// `required_char_id` is the resolved `char_id` of the gating teammate, or
    /// None when the named operator couldn't be resolved (the conditional part
    /// then contributes nothing rather than being over-credited).
    ConditionalOnTeammate {
        required_char_id: Option<String>,
        base_efficiency: f64,
        efficiency: f64,
        order_limit: i32,
    },

    /// Scales with total order limit contributions from teammates.
    /// e.g. Degenbrecher E2: "+25% per 5 CAP from teammates, max +100%"
    /// e.g. Jaye E0+1: "+4% per 1 order limit increase from others"
    OrderLimitScaling {
        per_cap_threshold: f64,   // every N CAP
        bonus_per_threshold: f64, // gives this much %
        cap_pct: f64,             // max bonus
    },

    /// Efficiency scales with the number of teammates that match a keyword the
    /// buff names for itself. The keyword is parsed straight from the buff text
    /// ("for each <kw>…"), so this one strategy covers every faction- and
    /// skill-type synergy without hardcoding any faction or operator name:
    ///   - Dorothy: "+5% per Rhine Tech-type skill"      → token "rhine"
    ///   - Mizuki:  "+5% per Standardization Skill"       → token "standardization"
    ///   - Bryophyta:"+5% per Metalwork-type skill"       → token "metalwork"
    ///   - Morgan:  "+20% per Glasgow Gang Operator"      → token "glasgow"
    ///
    /// A teammate matches when the token equals one of its faction ids
    /// (group/nation/team) or the leading word of one of its skill names.
    ///
    /// `bonus_char_id` / `bonus_pct` carry an OPTIONAL named-teammate rider that
    /// some count-scalers tack on (Morgan "Gang Compass": +20% per Glasgow Gang
    /// op, AND +35% more when Siege shares the room). `bonus_pct` is credited only
    /// when that named operator is present; `None` means no rider.
    MatchCountScaling {
        token: String,
        per_match_pct: f64,
        cap_pct: Option<f64>,
        bonus_char_id: Option<String>,
        bonus_pct: f64,
    },

    /// A base efficiency that's always applied, plus a bonus that applies when ANY
    /// operator of a given faction shares the room - the faction analogue of
    /// `ConditionalOnTeammate`. e.g. Morgan "Resolution on Foreign Trade β": base
    /// +30%, +10% more if any Glasgow Gang operator is in the same Trading Post.
    /// `faction_token` is matched against teammates' `match_tags` (e.g. "glasgow").
    ConditionalOnFaction {
        faction_token: String,
        base_efficiency: f64,
        efficiency: f64,
    },

    /// Reclassifies certain skill types as another (Highmore: "all Rhine Lab and
    /// Pinus Sylvestris skills are considered Standardization skills"). While
    /// this operator is in the room, any teammate matching a `from_tokens` tag
    /// also gains the `to_token` tag - so e.g. Rhine operators start counting
    /// for a Standardization scaler. Carries no efficiency of its own.
    SkillTypeConversion {
        from_tokens: Vec<String>,
        to_token: String,
    },

    /// Nullifies every teammate's output, but this operator's own efficiency
    /// scales with the number of teammates present.
    /// e.g. Shamare: "all other Operators' efficiency becomes 0, but each
    /// Operator increases this Operator's efficiency by +45%".
    NullifyTeammatesSelfScaling { per_teammate_pct: f64 },

    /// Boosts LMD *per order* (order value) rather than order speed. Because the
    /// trade rate is a fixed 500 LMD per Pure Gold bar, an LMD-per-order boost is
    /// an LMD-per-hour boost of the same proportion (gold-supply permitting). This
    /// is NOT "order acquisition efficiency", so a flat-LMD value (Tequila) survives
    /// a Shamare-style speed-nullifier. `estimated_pct` is the LMD-equivalent value.
    ///
    /// `pure_gold` marks a value that depends on Pure-Gold orders specifically
    /// (Proviso boosts low/"defaulted" Pure-Gold orders). A Shamare-type operator
    /// shifts the post toward higher-yield Precious-Metal orders, so a Pure-Gold
    /// value no longer applies in a Shamare team - which is why Proviso, unlike
    /// Tequila, does NOT benefit from Shamare.
    OrderValue { estimated_pct: f64, pure_gold: bool },

    /// Control Center buff that applies globally to all rooms of a type.
    /// e.g. "all Factories +2%"
    GlobalEffect {
        target_room: String, // "MANUFACTURE", "TRADING"
        bonus_pct: f64,
    },

    /// Control Center buff that boosts a production room ONLY when its team
    /// matches a faction condition - so it is NOT credited flat to every room.
    ///   - per-operator (Umiri): "all <Siracusa> Operators in Trading Posts gain
    ///     +5%" → each matching op adds `bonus_pct` (`faction_token` "siracusa",
    ///     `per_operator` = true, `required_count` = 1).
    ///   - count-gated (SilverAsh): "all Trading Posts with 3 <Kjerag> Operators
    ///     gain +10%" → the whole post gains `bonus_pct` once it holds
    ///     `required_count` of that faction (`per_operator` = false, required 3).
    ///
    /// `faction_token` is matched against teammates' `match_tags`.
    ConditionalGlobalEffect {
        target_room: String,
        faction_token: String,
        required_count: usize,
        per_operator: bool,
        bonus_pct: f64,
    },

    /// Bonus based on operator faction/tag in the affected rooms.
    /// e.g. "all Knight operators in Factories +7%"
    TagBased {
        tag: String, // "knight", "sarkaz", "abyssal", etc.
        bonus_pct: f64,
        target_room: String,
    },

    /// Morale recovery or morale drain modifier (dormitory/control).
    MoraleModifier {
        recovery_per_hour: f64, // positive = recovery, negative = drain
        is_self_only: bool,     // true for single-target, false for AoE
    },

    /// Only affects capacity/order limits, not speed.
    CapacityOnly,

    /// Non-production facilities (workshop, HR, training, reception).
    /// Store the efficiency or parsed value for secondary scoring.
    NonProduction { value: f64 },

    /// Fallback for truly complex buffs we can't cleanly parse.
    /// Stores a conservative estimate.
    Complex { estimated_pct: f64 },

    /// Efficiency changes over the course of a shift based on time/morale.
    /// Stores the time-averaged value over a full 24hr shift.
    MoraleDecayEfficiency { time_averaged_value: f64 },
}

pub fn build_registry(
    buffs: &HashMap<String, Buff>,
    // Lowercased operator display name -> char_id, used to resolve named-teammate
    // conditional buffs (e.g. Texas's "...same Trading Post as Lappland").
    name_to_char: &HashMap<String, String>,
) -> (
    HashMap<String, BuffResolutionStrategy>,
    HashMap<String, f64>, // buff_id -> morale drain modifier
) {
    let mut registry: HashMap<String, BuffResolutionStrategy> = HashMap::new();
    let mut morale_drains = HashMap::new();

    for (buff_id, buff) in buffs {
        let prefix = buff_id.split('[').next().unwrap_or(buff_id);
        // "manu_prod_spd&power[000]" → "manu_prod_spd&power"
        // Then check: prefix.contains("&power"), prefix.contains("_variable"), etc.
        let strategy = match buff.room_type.as_str() {
            "WORKSHOP" | "HIRE" | "TRAINING" | "MEETING" => {
                let effiency = if buff.efficiency == 0 {
                    // parse from description
                    0.0
                } else {
                    f64::from(buff.efficiency)
                };
                BuffResolutionStrategy::NonProduction { value: effiency }
            }
            "DORMITORY" => {
                let recovery = parse_first_float(&buff.description).unwrap_or(0.0);
                let desc_lower = buff.description.to_lowercase();
                let is_self_only = desc_lower.contains("self")
                    || desc_lower.contains("oneself")
                    || prefix.contains("_oneself")
                    || prefix.contains("_single");
                BuffResolutionStrategy::MoraleModifier {
                    recovery_per_hour: recovery,
                    is_self_only,
                }
            }
            "CONTROL" => {
                let desc_lower = buff.description.to_lowercase();
                if prefix.contains("_fraction") || prefix.contains("_tag") {
                    let tag = parse_tag_keyword(&buff.description).unwrap_or_default();
                    let bonus = parse_first_pct(&buff.description).unwrap_or(0.0);
                    // Production tag buffs (Viviana: "+7% Knights in Factories")
                    // only reach matching operators, so model them per-operator
                    // like the faction conditionals - this lets the optimizer
                    // co-schedule the buffed operators with the CC operator. Tag
                    // buffs on non-production rooms (e.g. "Elite in Dormitories")
                    // stay as a flat tag bonus.
                    if desc_lower.contains("factor") || desc_lower.contains("trading") {
                        let target_room = if desc_lower.contains("trading") {
                            "TRADING"
                        } else {
                            "MANUFACTURE"
                        };
                        BuffResolutionStrategy::ConditionalGlobalEffect {
                            target_room: target_room.to_string(),
                            faction_token: tag,
                            required_count: 1,
                            per_operator: true,
                            bonus_pct: bonus,
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
                    // Morale gain
                    let recovery = parse_first_float(&buff.description).unwrap_or(0.0);
                    BuffResolutionStrategy::MoraleModifier {
                        recovery_per_hour: recovery,
                        is_self_only: false,
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
                    let bonus = parse_first_pct(&buff.description).unwrap_or(0.0);
                    // Faction-gated global bonuses ("all <Siracusa> Operators…",
                    // "all Trading Posts with 3 <Kjerag> Operators…") must NOT be
                    // credited flat to every room - they depend on each room's team.
                    // The faction token comes from the displayed keyword (Kjerag →
                    // "kjerag", matching operators' nation tag), not the internal
                    // group marker. A "with <N>" clause means the WHOLE post is
                    // gated on holding N of that faction; otherwise it's a per-
                    // operator bonus that each matching operator earns.
                    if let Some(faction_token) = parse_tag_keyword(&buff.description)
                        && !faction_token
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_digit())
                        && buff.description.contains("Operators")
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
                        }
                    } else {
                        // Unconditional global ("all Trading Posts +7%").
                        BuffResolutionStrategy::GlobalEffect {
                            target_room: target_room.to_string(),
                            bonus_pct: bonus,
                        }
                    }
                } else {
                    // Other buffs
                    let value = parse_first_pct(&buff.description).unwrap_or(0.0);
                    BuffResolutionStrategy::Complex {
                        estimated_pct: value,
                    }
                }
            }
            "MANUFACTURE" | "TRADING" | "POWER" => {
                // Named-teammate conditional. Handles both phrasings:
                //   "...same Trading Post as <@cc.kw>Lappland</> … +65%"   (Texas)
                //   "+20%; if <@cc.kw>Exusiai</> … same Trading Post … +25%" (Lemuen)
                // The base efficiency (always applied) is the Efficiency field; the
                // bonus is the % stated alongside the named operator. Faction-count
                // buffs ("for every Glasgow Gang operator…") are excluded.
                if buff.description.contains("same")
                    && !buff.description.contains("for every")
                    && let Some((req_name, name_end)) =
                        find_operator_keyword(&buff.description, name_to_char)
                {
                    let required_char_id = name_to_char.get(&req_name).cloned();
                    let base_efficiency = f64::from(buff.efficiency);
                    // The conditional bonus is the efficiency % stated after the
                    // named operator; for pure-conditional buffs (base 0) fall back
                    // to the first % in the text.
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
                }
                // Faction-gated conditional (Morgan "Resolution on Foreign Trade β":
                // base +30%, +10% more if ANY Glasgow Gang op shares the post). The
                // faction analogue of the named-teammate conditional above: it names
                // a faction ("if a <Glasgow Gang> Operator…same…") rather than one
                // operator. Count-scalers ("for every <faction>…") are excluded -
                // those are handled by MatchCountScaling below.
                else if buff.description.contains("same")
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
                }
                // Shamare-type: nullifies every teammate's output, but self-scales
                // per teammate ("...all other Operators' efficiency becomes 0, but
                // each Operator increases this Operator's efficiency by +45%").
                // The regex requires the per-Operator % so factory automation ops
                // (Weedy "per Power Plant", Snegurochka "+N Capacity") fall through
                // to their facility-scaling handling below.
                else if let Some(cap) = RE_NULLIFY_SELF_PCT.captures(&buff.description) {
                    BuffResolutionStrategy::NullifyTeammatesSelfScaling {
                        per_teammate_pct: cap[1].parse().unwrap_or(0.0),
                    }
                }
                // Skill-type converter (Highmore): "all <X> and <Y> skills are
                // considered <Z> skills". Parsed generically from the keywords.
                else if let Some((from_tokens, to_token)) =
                    parse_skill_conversion(&buff.description)
                {
                    BuffResolutionStrategy::SkillTypeConversion {
                        from_tokens,
                        to_token,
                    }
                }
                // Match-count scaling: "+X% for each <keyword>" where the keyword
                // is a faction or skill type (NOT a number - those are resource
                // mechanics, handled elsewhere). One data-driven strategy for every
                // faction/skill synergy; the token comes straight from the text.
                else if let Some(token) = parse_count_keyword(&buff.description) {
                    let per_match_pct = parse_first_pct(&buff.description).unwrap_or(5.0);
                    let cap_pct = parse_scaling_cap(&buff.description);
                    // Optional named-teammate rider (Morgan "Gang Compass": +35%
                    // more "when in the same Trading Post as Siege"). Credited only
                    // when that operator is present; absent → (None, 0).
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
                    BuffResolutionStrategy::MatchCountScaling {
                        token,
                        per_match_pct,
                        cap_pct,
                        bonus_char_id,
                        bonus_pct,
                    }
                }
                // Order-VALUE trading skills: raise LMD *per order* rather than
                // order speed. Calibrated against the trading-post economy (Pure
                // Gold = 500 LMD/bar, L3 order mix 30/50/20 low/med/high). NOTE these
                // do NOT stack across operators (the team scorer keeps only the
                // strongest); their payoff is realised by pairing the value operator
                // with order-acquisition SPEED, not with a second value operator:
                //   - Proviso "Damages for Breach" (+2 Pure Gold to defaulted
                //     low/med orders, same completion time): avg order 1450→2250
                //     LMD ⇒ ×1.55, i.e. +55% LMD/hour.
                //   - Tequila "+N LMD on non-defaulted high orders": ~+10% in
                //     isolation, but its bonus EXCLUDES the defaulted orders Proviso
                //     boosts, so it adds nothing alongside Proviso.
                //   - Precious-Metal "higher-yield chance" (Tailoring): shifts the
                //     order mix up; gold/hour ≈ flat, value realised via the order
                //     cap ⇒ ~+10%.
                //   - A bare enabler with no payoff (Proviso "Contract Law") ⇒ 0.
                else if buff.room_type == "TRADING"
                    && let Some((est, pure_gold)) = order_value_estimate(&buff.description)
                {
                    BuffResolutionStrategy::OrderValue {
                        estimated_pct: est,
                        pure_gold,
                    }
                }
                // Jaye-style: efficiency scales with the order-limit difference
                // that teammates' efficiency creates ("increases order acquisition
                // efficiency by +X% for every difference of 1 order"). Teammates'
                // efficiency drives the order limit down, so model it as mirroring
                // their output - Texas's +65% pushes Jaye to ~+50%. (This buff has
                // "_limit" in its id but is an EFFICIENCY skill, so it must be
                // caught before the capacity-only check below.)
                else if prefix.contains("_limit_diff")
                    || (buff.room_type == "TRADING"
                        && buff.description.contains("order acquisition efficiency")
                        && buff.description.contains("difference"))
                {
                    let per = parse_first_pct(&buff.description).unwrap_or(4.0);
                    // Jaye's bonus tracks the order-limit DIFFERENCE (how empty the
                    // post is): it peaks right after a collection and decays toward 0
                    // as orders accumulate. Recommend his TIME-AVERAGED value over a
                    // ~12h shift (the post fills from empty toward full), which is
                    // about half the empty-post peak - so the cap is the order-limit-
                    // bounded ~40% peak halved to ~20%.
                    const SHIFT_AVERAGE: f64 = 0.5;
                    BuffResolutionStrategy::TeammateOutputMirroring {
                        ratio: (per / 5.0) * SHIFT_AVERAGE,
                        cap_pct: 40.0 * SHIFT_AVERAGE,
                    }
                }
                // Capacity-only: true order-limit skills with no speed component.
                else if (prefix.contains("_limit") || prefix.contains("limit&"))
                    && !prefix.contains("_spd")
                    && buff.efficiency == 0
                {
                    BuffResolutionStrategy::CapacityOnly
                }
                // Morale-decay dependent: efficiency decreases as morale drops
                else if prefix.contains("_reduce")
                    && buff.description.contains("Morale difference")
                {
                    // Pattern: "+X% base, -Y% per Z morale difference"
                    // Peak is in Efficiency field. Penalty: parse from description.
                    // Over a full shift, avg morale difference = 12
                    // (morale goes from 24 to 0, difference goes from 0 to 24, avg = 12)
                    let peak = f64::from(buff.efficiency);
                    // Parse: "every <@cc.kw>4</> points" → 4, and "-5%" → 5
                    let interval = parse_kw_number(&buff.description).unwrap_or(4.0);
                    let penalty_pct = parse_first_vdown_pct(&buff.description).unwrap_or(5.0);
                    let avg_penalty = (12.0 / interval) * penalty_pct;
                    BuffResolutionStrategy::MoraleDecayEfficiency {
                        time_averaged_value: (peak - avg_penalty).max(0.0),
                    }
                }
                // Morale threshold: activates when morale difference > threshold
                else if prefix.contains("_addition&cost")
                    && buff.description.contains("Morale difference")
                {
                    // Pattern: "+X% when morale difference > T"
                    // Active for (24 - T) / 24 of the shift
                    let bonus = parse_first_pct(&buff.description).unwrap_or(0.0);
                    let threshold = parse_kw_number(&buff.description).unwrap_or(12.0);
                    let active_fraction = (24.0 - threshold) / 24.0;
                    BuffResolutionStrategy::MoraleDecayEfficiency {
                        time_averaged_value: bonus * active_fraction,
                    }
                }
                // Time-ramp: efficiency increases per hour, capped
                else if prefix.contains("_addition") && buff.description.contains("per hour") {
                    // Pattern: "+X% base, +Y% per hour, up to +Z%"
                    let base = f64::from(buff.efficiency); // starting value (may be 0)
                    // let per_hour = parse_first_pct(&buff.description).unwrap_or(0.0);
                    // Note: parse_first_pct will grab the first %, which for [030] is "20%"
                    // We need the "per hour" value specifically. Use a targeted regex.
                    let per_hr = parse_per_hour_pct(&buff.description).unwrap_or(1.0);
                    let cap = parse_last_pct(&buff.description).unwrap_or(25.0);
                    // Time to reach cap from base: (cap - base) / per_hr hours
                    let ramp_hours = if per_hr > 0.0 {
                        (cap - base) / per_hr
                    } else {
                        24.0
                    };
                    // Over a 24hr shift:
                    // - Ramp phase: average = (base + cap) / 2, duration = min(ramp_hours, 24)
                    // - Plateau phase: value = cap, duration = max(24 - ramp_hours, 0)
                    let ramp_duration = ramp_hours.min(24.0);
                    let plateau_duration = (24.0 - ramp_duration).max(0.0);
                    let avg = ((base + cap) / 2.0 * ramp_duration + cap * plateau_duration) / 24.0;
                    BuffResolutionStrategy::MoraleDecayEfficiency {
                        time_averaged_value: avg,
                    }
                }
                // Efficiency + order limit (e.g. "efficiency +25% and order limit -6")
                // Must come BEFORE the generic buff.efficiency > 0 check
                else if prefix.starts_with("trade_ord_spd&limit") {
                    let efficiency = f64::from(buff.efficiency);
                    let order_limit = parse_order_limit(&buff.description).unwrap_or(0);
                    BuffResolutionStrategy::EfficiencyWithOrderLimit {
                        efficiency,
                        order_limit,
                    }
                }
                // Order limit scaling: Degenbrecher's "for every 5 order limit increase... +25%, max +100%"
                else if prefix == "trade_ord_spd_variable3" {
                    let threshold = parse_first_vup_number(&buff.description).unwrap_or(5.0);
                    let bonus = parse_nth_pct(&buff.description, 0).unwrap_or(25.0);
                    let cap = parse_last_pct(&buff.description).unwrap_or(100.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: threshold,
                        bonus_per_threshold: bonus,
                        cap_pct: cap,
                    }
                }
                // Jaye's "Investment Solicitations": "+4% per order limit increase from others"
                else if prefix == "trade_ord_spd_variable" {
                    let per = parse_first_pct(&buff.description).unwrap_or(4.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: 1.0,
                        bonus_per_threshold: per,
                        cap_pct: f64::MAX,
                    }
                }
                // Recipe-type scaling (Quartz "Precise Scheduling"): a base trading
                // efficiency PLUS "+N% per recipe type being processed at Factories".
                // Distinct recipe types are approximated by the Factory count - the
                // base % is the Efficiency field, the per-unit % is the trailing %.
                else if prefix.contains("&formula") && buff.description.contains("recipe type") {
                    let per_unit = parse_last_pct(&buff.description).unwrap_or(2.0);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "MANUFACTURE".to_string(),
                        per_unit_pct: per_unit,
                        per_level: false,
                        nullifies_others: false,
                        base_pct: f64::from(buff.efficiency),
                    }
                }
                // Direct efficiency
                else if buff.efficiency > 0 {
                    BuffResolutionStrategy::DirectEfficiency {
                        value: f64::from(buff.efficiency),
                    }
                }
                // Automation, scales with power plant count
                else if prefix.contains("&power") {
                    let per_unit = parse_first_pct(&buff.description).unwrap_or(5.0);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "POWER".to_string(),
                        per_unit_pct: per_unit,
                        per_level: false,
                        nullifies_others: true,
                        base_pct: 0.0,
                    }
                }
                // Snegurochka-type: nullifies teammates but only grants Capacity
                // (no speed), so it contributes 0 productivity. Modeled as a
                // zero-value automation op so it's never picked for output.
                else if prefix.contains("&manu") {
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "MANUFACTURE".to_string(),
                        per_unit_pct: 0.0,
                        per_level: false,
                        nullifies_others: true,
                        base_pct: 0.0,
                    }
                }
                // Dormitory scaling
                else if prefix.contains("&dorm") {
                    let per_unit = parse_first_pct(&buff.description).unwrap_or(1.0);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "DORMITORY".to_string(),
                        per_unit_pct: per_unit,
                        per_level: true,
                        nullifies_others: false,
                        base_pct: 0.0,
                    }
                }
                // Teammate skill scaling (eg. +5% per Standardization skill)
                else if prefix.contains("_skill_spd") {
                    let per_match = parse_first_pct(&buff.description).unwrap_or(5.0);
                    let keyword = parse_tag_keyword(&buff.description).unwrap_or_default();
                    BuffResolutionStrategy::TeammateSkillScaling {
                        target_buff_pattern: keyword,
                        per_match_pct: per_match,
                    }
                }
                // Output mirroring, eg. Heavenly Reward, Champion's Bearing
                else if prefix.contains("_variable2") {
                    let cap = parse_last_pct(&buff.description).unwrap_or(25.0);
                    let per = parse_first_pct(&buff.description).unwrap_or(5.0);
                    BuffResolutionStrategy::TeammateOutputMirroring {
                        ratio: per,
                        cap_pct: cap,
                    }
                }
                // Building-resource dependent (Marcille's "+1% per Monster Meal",
                // Engineering Robots, Chain of Thought, Witchcraft Crystal, etc.):
                // scales with a special stockpile resource the optimizer can't assume
                // the player has banked, so it's worth 0 in the baseline - crediting
                // the per-unit % (as if one unit were stocked) over-ranks the operator
                // against reliable specialists. Any always-on flat productivity lives
                // in a separate base-skill slot, captured by the `efficiency > 0`
                // branch above.
                else if prefix.contains("_bd") {
                    BuffResolutionStrategy::Complex { estimated_pct: 0.0 }
                }
                // Trading gold-line scaling
                else if prefix.contains("&gold") || prefix.contains("&trade") {
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
                    }
                }
                // Fallback
                else {
                    let est = parse_first_pct(&buff.description).unwrap_or(15.0);
                    BuffResolutionStrategy::Complex { estimated_pct: est }
                }
            }
            _ => BuffResolutionStrategy::CapacityOnly,
        };

        let mut drain = 0.0;
        if let Some(val) = parse_morale_drain_increase(&buff.description) {
            drain += val;
        }
        if let Some(val) = parse_morale_drain_decrease(&buff.description) {
            drain -= val;
        }

        if drain != 0.0 {
            morale_drains.insert(buff_id.clone(), drain);
        }

        registry.insert(buff_id.clone(), strategy);
    }

    (registry, morale_drains)
}

/// The match token of a "for each/every <keyword>" count-scaling buff, or `None`
/// if the buff doesn't scale per teammate. The token is the leading word of the
/// keyword, lowercased (e.g. "Rhine Tech-type skill" → "rhine", "Glasgow Gang
/// Operator" → "glasgow"). Numeric keywords ("for each 4 gold bars") are
/// resource mechanics, not teammate counts, and return `None`.
fn parse_count_keyword(desc: &str) -> Option<String> {
    let cap = RE_COUNT_KEYWORD.captures(desc)?;
    let token = first_token(&cap[1]);
    (!token.is_empty() && !token.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .then_some(token)
}

/// Parse a skill-type converter: "all <X> and <Y> skills are considered <Z>
/// skills" → (from = [x, y], to = z). The last keyword is the target type; the
/// earlier ones are the source types. Returns `None` if not a converter.
fn parse_skill_conversion(desc: &str) -> Option<(Vec<String>, String)> {
    if !(desc.contains("considered") && desc.contains("skill")) {
        return None;
    }
    let kws: Vec<String> = RE_KW_ANY
        .captures_iter(desc)
        .map(|c| first_token(&c[1]))
        .filter(|t| !t.is_empty())
        .collect();
    if kws.len() < 2 {
        return None;
    }
    let (to_token, from) = kws.split_last()?;
    Some((from.to_vec(), to_token.clone()))
}

/// Optional "max +X%" cap on a scaling buff.
fn parse_scaling_cap(desc: &str) -> Option<f64> {
    let lower = desc.to_lowercase();
    let idx = lower.find("max")?;
    RE_LAST_PCT
        .find_iter(&desc[idx..])
        .next()
        .and_then(|m| RE_LAST_PCT_INNER.captures(m.as_str()))
        .and_then(|c| c[1].parse().ok())
}

/// LMD-equivalent value of an order-VALUE trading skill, or `None` if the buff
/// isn't one. Calibrated from the trading-post economy (500 LMD per Pure Gold
/// bar; L3 order mix 30/50/20). See the call site for the derivations.
/// `(estimated LMD-equivalent %, pure_gold)`. `pure_gold` is true for values that
/// only apply to Pure-Gold orders (Proviso), which a Shamare-type Precious-Metal
/// shift nullifies; false for flat-LMD (Tequila) and Precious-Metal values.
fn order_value_estimate(desc: &str) -> Option<(f64, bool)> {
    if desc.contains("increase the LMD") {
        Some((10.0, false)) // Tequila-type: flat +N LMD on non-defaulted high orders
    } else if desc.contains("Pure Gold") && desc.contains("traded <@cc.vup>") {
        Some((55.0, true)) // Proviso payoff: +2 Pure Gold per defaulted order
    } else if desc.contains("Precious Metal") || desc.contains("higher-yield") {
        Some((10.0, false)) // higher-yield order chance (Tailoring etc.)
    } else if desc.contains("Pure Gold") || desc.contains("Defaulted trade") {
        Some((0.0, true)) // enabler with no direct payoff (Proviso "Contract Law")
    } else {
        None
    }
}

/// Leading word of a keyword phrase, lowercased ("Rhine Tech-type" → "rhine").
fn first_token(s: &str) -> String {
    s.trim()
        .split([' ', '-'])
        .next()
        .unwrap_or("")
        .to_lowercase()
}

/// Extract first percentage like "+25%" from description markup
fn parse_first_pct(desc: &str) -> Option<f64> {
    RE_FIRST_PCT.captures(desc).and_then(|c| c[1].parse().ok())
}

/// First `<@cc.vup>+N%` efficiency at or after byte offset `start` - used to
/// pick out the conditional bonus that follows a named operator keyword.
fn parse_first_pct_from(desc: &str, start: usize) -> Option<f64> {
    desc.get(start..).and_then(parse_first_pct)
}

/// First `<@cc.kw>…</>` keyword that resolves to a known operator name
/// (stripping nested `<$cc.x>` wrappers). Returns the lowercased name and the
/// byte offset just past that keyword block, so the caller can scan for the
/// conditional bonus % that appears after it. Handles both word orders:
/// "...same Trading Post as <@cc.kw>Lappland</>" and
/// "if <@cc.kw>Exusiai</> is assigned to the same Trading Post".
fn find_operator_keyword(
    desc: &str,
    name_to_char: &HashMap<String, String>,
) -> Option<(String, usize)> {
    for cap in RE_KW_BLOCK.captures_iter(desc) {
        let block = cap.get(0)?;
        let name = RE_INNER_TAG.replace_all(&cap[1], "").trim().to_lowercase();
        if name_to_char.contains_key(&name) {
            return Some((name, block.end()));
        }
    }
    None
}

/// First faction marker (`<$cc.g.glasgow>` etc.) in the text. Returns the token
/// and the byte offset just past it, so the caller can read the bonus % that
/// follows the faction mention.
fn find_faction_token(desc: &str) -> Option<(String, usize)> {
    let m = RE_FACTION_TOKEN.captures(desc)?;
    let token = m[1].to_lowercase();
    Some((token, m.get(0)?.end()))
}

/// Extract first float like "+0.7" from description markup (for morale values)
fn parse_first_float(desc: &str) -> Option<f64> {
    RE_FIRST_FLOAT
        .captures(desc)
        .and_then(|c| c[1].parse().ok())
}

/// Extract tag keyword like "Knight" from <@cc.kw>Knight</>
fn parse_tag_keyword(desc: &str) -> Option<String> {
    RE_TAG_KEYWORD.captures(desc).map(|c| c[1].to_lowercase())
}

/// Extract the last percentage in description (for cap values)
fn parse_last_pct(desc: &str) -> Option<f64> {
    RE_LAST_PCT.find_iter(desc).last().and_then(|m| {
        RE_LAST_PCT_INNER
            .captures(m.as_str())
            .and_then(|c| c[1].parse().ok())
    })
}

/// Parse number from <@cc.kw>4</> pattern
fn parse_kw_number(desc: &str) -> Option<f64> {
    RE_KW_NUMBER.captures(desc).and_then(|c| c[1].parse().ok())
}

/// Parse first negative percentage from <@cc.vdown>-5%</> or <@cc.vdown>+0.25</>
fn parse_first_vdown_pct(desc: &str) -> Option<f64> {
    RE_VDOWN_PCT.captures(desc).and_then(|c| c[1].parse().ok())
}

/// Parse "per hour" percentage: "+2% per hour" or "+1% per hour"
fn parse_per_hour_pct(desc: &str) -> Option<f64> {
    RE_PER_HOUR_PCT
        .captures(desc)
        .and_then(|c| c[1].parse().ok())
}

/// Parse order limit from description.
/// Matches "+4" from <@cc.vup>+4</> or "-6" from <@cc.vdown>-6</>
fn parse_order_limit(desc: &str) -> Option<i32> {
    if let Some(cap) = RE_ORDER_LIMIT_POS.captures(desc) {
        return cap[1].parse::<i32>().ok();
    }
    if let Some(cap) = RE_ORDER_LIMIT_NEG.captures(desc) {
        return cap[1].parse::<i32>().ok().map(|v| -v);
    }
    None
}

/// Parse the Nth <@cc.vup> percentage (0-indexed).
/// Useful when a description has multiple percentage values.
fn parse_nth_pct(desc: &str, n: usize) -> Option<f64> {
    RE_NTH_PCT.find_iter(desc).nth(n).and_then(|m| {
        RE_LAST_PCT_INNER
            .captures(m.as_str())
            .and_then(|c| c[1].parse().ok())
    })
}

/// Parse a plain number from <@cc.vup>5</> (no % sign)
fn parse_first_vup_number(desc: &str) -> Option<f64> {
    RE_VUP_NUMBER.captures(desc).and_then(|c| c[1].parse().ok())
}

/// Parse increased morale drain: "Morale consumed per hour <@cc.vdown>+0.25</>"
pub fn parse_morale_drain_increase(desc: &str) -> Option<f64> {
    RE_MORALE_INCREASE
        .captures(desc)
        .and_then(|c| c[1].parse().ok())
}

/// Parse decreased morale drain: "Morale consumed per hour <@cc.vup>-?([\d.]+)</>"
pub fn parse_morale_drain_decrease(desc: &str) -> Option<f64> {
    RE_MORALE_DECREASE
        .captures(desc)
        .and_then(|c| c[1].parse().ok())
}
