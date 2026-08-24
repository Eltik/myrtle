use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::core::gamedata::types::building::Buff;
use crate::core::gamedata::types::operator::Operator;

use super::pools::ROBOTS_IN_POWER;

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
    // Robot-class operators (Lancet-2, Castle-3, Friston-3, ...) carry the
    // "Robot" recruitment tag; the base's robot economies (Alanna's Operation
    // Platforms, Overclock) count them, so expose it as a match tag. No base
    // skill uses a "robot" faction token, so this cannot collide.
    if op.tag_list.iter().any(|s| s == "Robot") {
        push("robot");
    }
    // The multi-power system: RIIC faction tags count SECONDARY affiliations
    // too (Texas: nation lungmen, SubPower siracusa - and the game's "all
    // Siracusa Operators" buffs reach her). MainPower mirrors the top-level
    // ids; the dedup guard makes pushing it a harmless no-op.
    let powers = op.main_power.iter().chain(op.sub_power.iter().flatten());
    for power in powers {
        for id in [&power.nation_id, &power.group_id, &power.team_id]
            .into_iter()
            .flatten()
        {
            let lower = id.to_lowercase();
            if !lower.is_empty() && !tags.contains(&lower) {
                tags.push(lower);
            }
        }
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

/// A `<@cc.vup>+N</>` immediately followed by "Morale" - the actual morale-recovery figure,
/// as opposed to a resource-generation number ("…Worldly Plight<@cc.vup>+5</>") that happens
/// to come first in the text.
static RE_MORALE_RECOVERY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)</>\s*Morale").unwrap());

static RE_TAG_KEYWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.kw>([^<]+)</>").unwrap());

static RE_LAST_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").unwrap());

static RE_LAST_PCT_INNER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([\d.]+)%").unwrap());

static RE_KW_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<@cc\.kw>(\d+)</>").unwrap());

static RE_VDOWN_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vdown>[+-]?([\d.]+)%?</>").unwrap());

/// Layout-derived pool generator: "gain +N <Resource> per level per building
/// ... max CAP" (the resource key is the `$cc.bd_*` term id).
static RE_POOL_GEN_LEVELS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"gain <@cc\.vup>\+?([\d.]+)</>\s*<@cc\.kw><\$cc\.([a-z0-9_]+)>[^<]*</></>\s*per level per building.*max <@cc\.vup>([\d.]+)</>",
    )
    .unwrap()
});

/// Stepped pool consumer: "for every PER <Resource> present, productivity +PCT%".
static RE_POOL_CONSUME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.vup>\+?([\d.]+)</>\s*<@cc\.kw><\$cc\.([a-z0-9_]+)>[^<]*</></>\s*present, productivity <@cc\.vup>\+([\d.]+)%</>",
    )
    .unwrap()
});

/// Stepped pool consumer, reversed word order: "{metric phrase} +PCT% for
/// every PER <Resource>" (the Dungeon Meshi crew's Monster Meal skills).
static RE_POOL_CONSUME_REV: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"<@cc\.vup>\+([\d.]+)%</>[^<]{0,30}for every <@cc\.vup>([\d.]+)</>\s*<\$cc\.([a-z0-9_]+)>",
    )
    .unwrap()
});

/// Own-room-level pool generator: "provide N <Resource> for every level of the
/// current Dormitory" (Senshi).
static RE_POOL_GEN_OWN_ROOM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"provide <@cc\.vup>([\d.]+)</>\s*<\$cc\.([A-Za-z0-9_]+)>.{0,60}for every level of the current",
    )
    .unwrap()
});

/// Stepped pool consumer, direct form: "for every N (points of) <Resource>...,
/// productivity +PCT%" (Chain of Thought, Witchcraft Crystal, Worldly Plight).
static RE_POOL_CONSUME_POINTS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.vup>([\d.]+)</>\s*(?:points? of )?<\$cc\.([A-Za-z0-9_]+)>.{0,60}?[Pp]roductivity <@cc\.vup>\+([\d.]+)%</>",
    )
    .unwrap()
});

/// Mr. Nothing's one-buff dorm economy: "each Operator in the Dormitory grants
/// <Resource>+G, and for every P point <Resource>, grant +PCT% order
/// acquisition efficiency".
static RE_POOL_DORM_ECONOMY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"Operator in the Dormitory grants\s*<\$cc\.([A-Za-z0-9_]+)>[^%]{0,40}?<@cc\.vup>\+([\d.]+)</>, and for every <@cc\.vup>([\d.]+)</>\s*points?<\$cc\.[A-Za-z0-9_]+>.{0,40}?grant <@cc\.vup>\+([\d.]+)%</> order",
    )
    .unwrap()
});

/// A dorm-occupant generator whose points convert onward (Rosmontis'
/// Extrasensory: PI -> Chain of Thought; Musicianship: PI -> Soundless
/// Resonance): "for every 1 operator(s) in the Dormitory/ies, <GenResource>
/// +G ... every P (points of) <GenResource> is converted into/to 1 <To>".
static RE_POOL_GEN_DORM_CONVERT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.(?:kw|vup)>1</>\s*[Oo]perators? in the Dormitor(?:y|ies),\s*<\$cc\.([A-Za-z0-9_]+)>[^+]{0,40}?<@cc\.vup>\+([\d.]+)</>.{0,30}?every <@cc\.vup>([\d.]+)</>\s*(?:points? of )?<\$cc\.[A-Za-z0-9_]+>.{0,60}?converted (?:in)?to <@cc\.vup>1</>\s*(?:points? of )?<\$cc\.([A-Za-z0-9_]+)>",
    )
    .unwrap()
});

/// Alanna's robot counter: "productivity +PCT% for every <tag.op> Operation
/// Platform assigned to a Power Plant" - a pseudo-pool the settlement fills
/// with the count of Robot-tagged operators seated in POWER rooms.
static RE_POOL_CONSUME_ROBOTS_POWER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"productivity <@cc\.vup>\+([\d.]+)%</> for <@cc\.vup>every</>\s*<\$cc\.tag\.op>.{0,60}?assigned to a Power Plant",
    )
    .unwrap()
});

/// A room-wide drain aura: "Morale consumed per hour of all Operators in the
/// <room> -X" / "Morale loss of Operators in the <room> -X per hour".
static RE_MORALE_ROOM_AURA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"Morale (?:consumed per hour of all Operators|loss of Operators) in the [A-Za-z ]{3,20}?\s*<@cc\.vup>(-?[\d.]+)</>",
    )
    .unwrap()
});

/// A pool-scaling tail on a production skill: "plus an additional
/// <Productivity|order acquisition efficiency> +X% for every [N] <bd_ res>".
/// Audited 2026-08-13: captures exactly `manu_prod_spd&limit&bd[000]` and
/// `trade_ord_spd&limit&bd[000]` (the Felvine consumers).
static RE_POOL_TAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:plus an )?additional (?:Productivity|order acquisition efficiency) <@cc\.vup>\+([\d.]+)%</> for every (?:<@cc\.vup>([\d.]+)</>\s*)?<\$cc\.(bd_[A-Za-z0-9_]+)>",
    )
    .unwrap()
});

/// "each"/"every" QUANTIFYING a faction/tag token ("for each <Glasgow Gang>
/// Operator..."), styled or bare - the singular per-operator conditional form
/// (Delphine), as opposed to the plural "all <Kjerag> Operators..." phrasing.
/// Token-adjacency keeps a temporal "each hour" from ever matching.
static RE_EACH_FACTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:<@cc\.vup>)?(?:each|every)(?:</>)?\s*<\$cc\.(?:g|tag)\.").unwrap()
});

/// Pool-scaled Control-Center globals. Shape A: "for every N <res>, ... all
/// <Rooms>' order efficiency +P%" (Sakiko). Shape B: "of all <Rooms> +B%, with
/// an additional +P% for every N <res>" (Mortis).
static RE_GLOBAL_POOL_A: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.vup>([\d.]+)</>\s*<\$cc\.(bd_[A-Za-z0-9_]+)>.{0,120}?all (Trading Posts|Factories)'? (?:order )?(?:efficiency|productivity)?\s*<@cc\.vup>\+([\d.]+)%</>",
    )
    .unwrap()
});
static RE_GLOBAL_POOL_B: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"of all (Factories|Trading Posts) <@cc\.vup>\+([\d.]+)%</>, with an additional <@cc\.vup>\+([\d.]+)%</> for every <@cc\.vup>([\d.]+)</>\s*<\$cc\.(bd_[A-Za-z0-9_]+)>",
    )
    .unwrap()
});

/// Global-target label -> internal room type ("Trading Posts" / "Factories").
fn room_type_from_global_label(label: &str) -> &'static str {
    if label.starts_with("Trading") {
        "TRADING"
    } else {
        "MANUFACTURE"
    }
}

/// Non-production Control-Center skill effects, each in its own facility's
/// units: clue collection speed (Reception Room), Specialization training
/// speed (Training Room), HR contacting speed (HR Office).
static RE_CC_CLUE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"clue collection speed <@cc\.vup>\+([\d.]+)%</>").unwrap());
static RE_CC_TRAIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Specialization training speed <@cc\.vup>\+([\d.]+)%</>").unwrap()
});
static RE_CC_HIRE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"HR contacting speed <@cc\.vup>\+([\d.]+)%</>").unwrap());
/// A same-room companion gate: "assigned to the Control Center with <op>".
static RE_CC_WITH: LazyLock<Regex> = LazyLock::new(|| {
    // Both word orders: "assigned to the Control Center with <Aak>" and
    // Mr. Lee's "assigned together with <Aak> to the Control Center".
    Regex::new(r"assigned (?:to the Control Center with|together with) <@cc\.kw>([^<]+)</>")
        .unwrap()
});

/// A dormitory single-target healer: "restores +X to an(other) Operator in
/// that Dormitory (whose Morale is not full)".
static RE_DORM_SINGLE_TARGET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"to an(?:other)? Operator (?:assigned to|in) (?:that|the) Dorm(?:itory)?").unwrap()
});

/// The whole-dorm aura figure: "restores +X Morale per hour to all (other)
/// Operators assigned to that Dormitory". Captured explicitly so compound
/// texts classify as the AURA they are - the `contains("self")` heuristic
/// filed both shapes under self-only, hiding them from dorm staffing:
/// - Durin's "self Morale recovered per hour -0.1, but restores +0.2 ... to
///   all Operators". Community-confirmed 2026-08-24: the aura applies to
///   herself fully too (net +0.1), so the self-malus can never turn her
///   negative and needs no field of its own - dorm staff don't drain, so a
///   net-positive rider never changes an outcome.
/// - The "self +0.55, and restores +0.1 ... to all OTHER Operators" family.
///   A dorm self-recovery rider is priced nowhere (resters recover at dorm
///   rate + auras; only the Fiammetta swap path reads self-only rates, gated
///   on its "swap" text), so the others-aura is the whole model-relevant
///   value of these skills.
static RE_DORM_AURA_ALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"restores <@cc\.vup>\+([\d.]+)</> Morale per hour to all (?:other )?Operators")
        .unwrap()
});

/// A CC dorm-recovery aura: "all Operators in Dormitories recover +X Morale per hour".
static RE_DORM_RECOVERY_AURA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"all Operators in Dormitories recover <@cc\.vup>\+([\d.]+)</>\s*Morale per hour")
        .unwrap()
});

/// A per-teammate aura: "other Operators (working) in the <room> have +X%".
static RE_PER_TEAMMATE_EFF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"other Operators (?:working )?in the [A-Za-z ]{3,20} have <@cc\.vup>\+([\d.]+)%</>")
        .unwrap()
});

/// Stepped pool consumer, order-efficiency form: "for every PER <Resource>,
/// +PCT% order (acquisition) efficiency" (the band's Soundless Resonance).
static RE_POOL_CONSUME_ORDER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.vup>([\d.]+)</>\s*<\$cc\.([A-Za-z0-9_]+)>.{0,60}?,\s*<@cc\.vup>\+([\d.]+)%</> order (?:acquisition )?efficiency",
    )
    .unwrap()
});

/// A speed/capacity trade ("productivity -5%, capacity limit +16" - the
/// Craftsmanship family): signed productivity plus a flat capacity grant. The
/// morale-cost rider is captured separately by the drains side-map.
static RE_SPEED_CAPACITY_TRADE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"productivity <@cc\.(vup|vdown)>([+-]?[\d.]+)%</>,\s*capacity limit <@cc\.vup>\+([\d.]+)</>",
    )
    .unwrap()
});

/// A standalone converter: "every F <From> is converted to 1 <To>".
static RE_POOL_CONVERT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"every <@cc\.vup>([\d.]+)</>\s*<\$cc\.([A-Za-z0-9_]+)>.{0,50}?is converted to <@cc\.vup>1</>\s*<\$cc\.([A-Za-z0-9_]+)>",
    )
    .unwrap()
});

static RE_PER_HOUR_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)%?</>\s*per hour").unwrap());

// Factories phrase the queue cap as "capacity limit", trading posts as "order limit" - the
// same mechanic, so accept either so a factory capacity skill (Vermeil's "+8") is counted.
static RE_ORDER_LIMIT_POS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:order|capacity) limit\s*<@cc\.vup>\+?(\d+)</>").unwrap());

static RE_ORDER_LIMIT_NEG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:order|capacity) limit\s*<@cc\.vdown>-?(\d+)</>").unwrap());

static RE_NTH_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").unwrap());

static RE_VUP_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>(\d+)</>").unwrap());

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

// Buff text phrases the drain as "Morale consumed per hour" or "...each hour" (the latter
// covers the Penguin Logistics trio - Texas/Lappland/Exusiai), and some add "...BY +2"
// (Enforcer's reception skill) - so accept "(?:per|each) hour" and an optional "by", else
// those drains are silently lost.
static RE_MORALE_INCREASE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Morale consumed (?:per|each) hour\s*(?:by\s*)?<@cc\.vdown>\+?([\d.]+)</>").unwrap()
});

static RE_MORALE_DECREASE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Morale consumed (?:per|each) hour\s*(?:by\s*)?<@cc\.vup>-?([\d.]+)</>").unwrap()
});

#[derive(Clone, Debug, PartialEq)]
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
        /// Ceiling on the scaled part, when the buff states one ("Max +25%").
        cap_pct: Option<f64>,
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

    /// A base efficiency that's always applied, plus a bonus that applies when one of several named
    /// operators is actively WORKING somewhere in the base ("any Work Area, excluding Assistants") -
    /// not just present in the room, and not while resting in a dormitory.
    /// e.g. Hoederer "Starting From Scratch β": +30%, and +5% more when Ines or W works any Work Area.
    /// The optimizer resolves this to a flat efficiency per assignment once it knows who is actually
    /// deployed in a work area (two-pass), so the scorer treats it as base-only on its own.
    ConditionalOnBaseWide {
        required_char_ids: Vec<String>,
        base_efficiency: f64,
        bonus_efficiency: f64,
    },

    /// Deployment-context gate: the bonus applies only while a named operator (or enough
    /// operators of a faction) is stationed in a specific ROOM TYPE anywhere in the base.
    /// e.g. "if Kal'tsit is assigned to the Control Center, drone recovery +5%" (Roberta),
    /// "and Gummy is in a Trading Post, Battle Record formula productivity +35%",
    /// "if another Laterano Operator is assigned to a Power Plant, +5%".
    /// Like `ConditionalOnBaseWide`, the optimizer collapses this to a flat efficiency via
    /// `resolve_room_presence` once the deployment is known; unresolved contexts credit
    /// the base only (never guess).
    ConditionalOnRoomPresence {
        /// Named-operator form: any of these deployed in `room_type` unlocks the bonus.
        /// Empty when the named operator couldn't be resolved (bonus then stays off).
        required_char_ids: Vec<String>,
        /// Faction form: lowercased faction token; at least `required_count` operators
        /// carrying it must be deployed in `room_type`.
        required_faction: Option<String>,
        /// 2 for the "another <faction> Operator" phrasing - the buff only applies while
        /// its (same-faction) owner works that room type, so "owner + another" = 2 total.
        required_count: usize,
        /// Internal room type ("TRADING", "CONTROL", "TRAINING", "POWER", ...).
        room_type: String,
        base_efficiency: f64,
        bonus_efficiency: f64,
    },

    /// Scales with total order limit contributions from teammates.
    /// e.g. Degenbrecher E2: "+25% per 5 CAP from teammates, max +100%"
    /// e.g. Jaye E0+1: "+4% per 1 order limit increase from others"
    /// e.g. Vermeil E1: "+2% per capacity limit in the factory" (her OWN +8 counts too)
    OrderLimitScaling {
        per_cap_threshold: f64,   // every N CAP
        bonus_per_threshold: f64, // gives this much %
        cap_pct: f64,             // max bonus
        /// True when the operator's OWN capacity counts toward the total it scales on
        /// (Vermeil scales on the whole factory's capacity, including her own +8). False for
        /// "from others/teammates" skills (Jaye, Degenbrecher - whose own -6 must not self-reduce).
        includes_self: bool,
    },

    /// Per-operator capacity-tier scaling (Bubble E1): each operator in the factory - including
    /// this one - gains `low_pct` productivity if its own capacity-limit bonus is at or below
    /// `threshold`, else `high_pct`. The room bonus is the SUM across the team, so it rewards
    /// stacking high-capacity operators.
    CapacityTierScaling {
        threshold: i32,
        low_pct: f64,
        high_pct: f64,
    },

    /// Raises a room type's EFFECTIVE facility count by `amount` ("Power Plant +1, only affects
    /// facility quantity" - Greyy the Lightningbearer E2; Eunectes E2 "+2"). It grants no
    /// productivity itself, but every `FacilityCountScaling` buff that scales per that facility
    /// (factory automation - Weedy/Eunectes/Pudding) reads the boosted count, so it powers those
    /// combos. Resolved by adjusting the facility counts the optimizer scores against.
    FacilityCountModifier { target_room: String, amount: i32 },

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
    /// A Control-Center global whose strength scales with a POOL: "for every 8
    /// Passion, all Trading Posts' order efficiency +1%" (Sakiko), "all
    /// Factories +1%, with an additional +1% for every 20 Passion" (Mortis).
    /// Context-free scoring credits `base_pct` only; the settlement resolves
    /// the pool part by registry rewrite (`resolve_global_pool`) once the
    /// points are known - current view from live seats, bundles from pinned
    /// generators.
    GlobalPoolScaling {
        target_room: String,
        base_pct: f64,
        /// +`pct`% per `per` points of `resource` (floored, stepped counter).
        pct: f64,
        per: f64,
        resource: String,
    },

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
        is_self_only: bool,     // true when only the holder benefits
        /// True only for Control-Center auras reaching OTHER buildings' workers
        /// (Chongyue's "Operators working in other buildings recover +0.05").
        /// False for the CC-room-only `control_mp_cost` family ("all Operators
        /// in the Control Center") and every dormitory skill.
        base_wide: bool,
        /// Dormitory skills only: true for the "restores +X to ANOTHER Operator
        /// in that Dormitory whose Morale is not full" single-target healers
        /// (+0.55-class), false for whole-dorm auras ("to all Operators in
        /// that Dormitory", +0.15-class). Both carry the game's non-stacking
        /// "only the strongest effect of this type" rule within their type.
        single_target: bool,
    },

    /// Only affects the room's capacity/order limit, not speed. `order_limit` is the cap it
    /// adds (e.g. Vermeil's "capacity limit +8") - it contributes no productivity itself, but
    /// feeds operators whose output scales with the room's total order/capacity limit.
    CapacityOnly { order_limit: i32 },

    /// Non-production facilities (workshop, HR, training, reception).
    /// Store the efficiency or parsed value for secondary scoring.
    NonProduction { value: f64 },

    /// A Control-Center skill that boosts a NON-PRODUCTION facility metric
    /// (clue collection, HR contacting, training speed), priced in its OWN
    /// units with zero LMD weight - the objective stays production-pure (both
    /// reference implementations silo or zero these; an LMD conversion would
    /// be an invented weight). Feeds the CC spare-seat tie-break and display.
    /// `same_room_gate`: None = unconditional; Some(chars) = only while one of
    /// the named operators shares the Control Center (empty = the named
    /// operator couldn't be resolved, so the gate never fires - never guess).
    ControlNonProduction {
        /// The boosted facility's room type ("MEETING", "TRAINING", "HIRE").
        target_room: String,
        value: f64,
        same_room_gate: Option<Vec<String>>,
    },

    /// Fallback for truly complex buffs we can't cleanly parse.
    /// Stores a conservative estimate.
    Complex { estimated_pct: f64 },

    /// Efficiency changes over the course of a shift based on time/morale.
    /// Stores the time-averaged value over a full 24hr shift.
    MoraleDecayEfficiency { time_averaged_value: f64 },

    /// Generates points of a named pool resource from the base's LAYOUT: "+N
    /// <Resource> per level per building, max CAP" (Minimalist's Engineering
    /// Robots). The basis is the summed level of the functional facilities, so
    /// the points are known without an assignment.
    PoolGenerateBuildingLevels {
        /// Stable resource key from the `$cc.bd_*` term id, not the display name.
        resource: String,
        per_level: f64,
        cap: f64,
    },

    /// Consumes settled pool points: "+PCT% productivity for every PER
    /// <Resource>". Floored by PER, like every stepped game counter.
    /// A pool-scaling TAIL on an otherwise ordinary production skill:
    /// "...capacity limit +8, Productivity +5%, plus an additional +1% for
    /// every <Felvine>". The base strategy prices the flat parts; the tail
    /// adds a `ScalingPoolPoints` clause on the same buff so the pool
    /// machinery feeds it - no half-parse where the tail silently drops.
    PoolTail {
        base: Box<Self>,
        resource: String,
        /// +`pct`% per `per` points of `resource`.
        pct: f64,
        per: f64,
    },

    PoolPointsScaling {
        resource: String,
        per: f64,
        pct: f64,
    },

    /// Generates pool points from the level of the room the OWNER is seated in
    /// (Senshi: "provide 1 Monster Meal for every level of the current
    /// Dormitory"). Settled at assignment scope, where the seat is known.
    PoolGenerateOwnRoomLevel { resource: String, per_level: f64 },

    /// A one-buff dorm economy (Mr. Nothing): every dorm occupant grants
    /// `per_occupant` points, and the SAME buff consumes them at `pct` per
    /// `per` points.
    PoolDormEconomy {
        resource: String,
        per_occupant: f64,
        per: f64,
        pct: f64,
    },

    /// A one-buff generator + converter (Rosmontis' Extrasensory): dorm
    /// occupants feed `gen_resource`, which converts 1:`from_per` into
    /// `to_resource`.
    PoolGenerateDormAndConvert {
        gen_resource: String,
        per_occupant: f64,
        to_resource: String,
        from_per: f64,
    },

    /// A room-wide morale-drain aura: every occupant of the owner's room
    /// (the owner included) drains `delta` more per hour (negative = slower).
    MoraleRoomAura { delta: f64 },

    /// A Control-Center aura raising dormitory sleepers' recovery
    /// (non-stacking: the strongest applies).
    DormRecoveryAura { rate: f64 },

    /// A per-teammate room aura: every OTHER occupant of the room gains
    /// `per_teammate_pct` of the room's speed metric ("other Operators working
    /// in the Trading Post have +15% order acquisition efficiency"), which the
    /// room total sees as value x (occupants - 1).
    PerTeammateEfficiency { per_teammate_pct: f64 },

    /// A standalone pool converter (Ancient Witchcraft: "every 5 Worldly
    /// Plight is converted to 1 Witchcraft Crystal").
    ///
    /// NOTE: the Control-Center WP/PI generators (Chongyue's Sui counter,
    /// Dusk/Ling's morale-conditional grants) are NOT parsed yet - those buffs
    /// carry a morale aura the CONTROL arm already claims, and a second effect
    /// needs a side-channel like `morale_drains`. Their consumers parse now
    /// and read an unfed pool (zero) until that lands.
    PoolConvert {
        from: String,
        to: String,
        from_per: f64,
    },

    /// A SOLVED resource-pool payoff (the perception-economy integration
    /// seam): the pool solver has already settled how much this consumer's
    /// buff drains, and `pct` is that drain as productivity. Distinct from
    /// `DirectEfficiency` so the scorer knows this is pool output - it rides
    /// the ledger's pool-drain channel, not a parsed flat skill.
    PoolPayoff { pct: f64 },
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
        // Strip the tier suffix: "manu_prod_spd&power[000]" → "manu_prod_spd&power".
        let prefix = buff_id.split('[').next().unwrap_or(buff_id);

        // Morale-drain extraction runs FIRST, before any parse branch can
        // `continue` past it - a buff's cost rider must be captured no matter
        // which strategy family claims its productivity half.
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

        // Facility-count enablers (Greyy the Lightningbearer E2 "Power Plant +1", Eunectes E2
        // "+2") raise a room type's EFFECTIVE facility count - they grant no productivity but
        // power every per-facility automation scaler. Detected room-agnostically by the stock
        // "(only affects facility quantity/count)" clause and the room they name - matching that
        // exact clause (not the "...based on facility count" of Weedy/Eunectes automation buffs)
        // keeps the two apart.
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
            registry.insert(
                buff_id.clone(),
                BuffResolutionStrategy::FacilityCountModifier {
                    target_room: target_room.to_string(),
                    amount,
                },
            );
            continue;
        }

        // Layout-derived pool economies (Minimalist's Engineering Robots): a
        // generator whose points come purely from the base's LAYOUT ("+N per
        // level per building, max CAP") and its stepped consumers ("for every
        // PER <Resource> present, productivity +PCT%"). The resource key is the
        // stable `$cc.bd_*` term id, never the display text. Economies whose
        // points depend on the ASSIGNMENT (dorm occupants, resting operators)
        // stay Unresolved until the assignment-scope pool pass lands.
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
        // Pool consumers are room-scored clauses; a CONTROL-room "+X% per N
        // <pool>" is a GLOBAL that fans out to other rooms (Ave Mujica's
        // "Plentiful Work Experience") and stays with the CC-global machinery.
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
                registry.insert(
                    buff_id.clone(),
                    BuffResolutionStrategy::EfficiencyWithOrderLimit {
                        efficiency: signed,
                        order_limit: c[3].parse().unwrap_or(0),
                    },
                );
                continue;
            }
        }

        let strategy = match buff.room_type.as_str() {
            "MEETING" => {
                // Clue-search speed. Most reception buffs carry it in `efficiency`, but several put
                // (or raise) the % in the description: a time-ramping skill states a higher ceiling
                // ("…by 2% per hour, up to a maximum of 30%" - Ines, which `efficiency` reports as
                // its lower 20% starting value), and solo / Clue-Exchange skills leave `efficiency`
                // 0 entirely (Caper's exchange skill, solo specialists). Credit the sustained
                // ceiling first, then the `efficiency` field, then any plain description %.
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
                // The explicit whole-dorm phrasing WINS over the self heuristic:
                // Durin's compound text mentions "self" but is an aura.
                let aura = RE_DORM_AURA_ALL
                    .captures(&buff.description)
                    .and_then(|c| c[1].parse::<f64>().ok());
                let recovery =
                    aura.unwrap_or_else(|| parse_first_float(&buff.description).unwrap_or(0.0));
                let is_self_only = aura.is_none()
                    && (desc_lower.contains("self")
                        || desc_lower.contains("oneself")
                        || prefix.contains("_oneself"));
                // "restores +X to another Operator in that Dormitory" - one
                // beneficiary, not the whole room (audited: 25 single-target
                // vs 39 whole-dorm captures, disjoint).
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
                // Pool-scaled globals FIRST, so the plain-global branches below
                // don't claim them at their flat base value. Audited 2026-08-13:
                // shape A captures exactly control_mp_bd&trade[000], shape B
                // exactly control_prod_bd_spd[000]/[010].
                if let Some(c) = RE_GLOBAL_POOL_A.captures(&buff.description) {
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
                    // Per-operator faction globals outside the _fraction/_tag id
                    // family (audited: exactly control_bd_spd's "for each
                    // <Blacksteel Worldwide> Operator assigned to Factories,
                    // productivity +5%"; its drain rider rides the side-map).
                    || (buff.description.contains("for each <$cc.g.")
                        && (desc_lower.contains("factor") || desc_lower.contains("trading")))
                {
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
                    // Control-Center morale recovery. Only the "other buildings"
                    // phrasing reaches workers base-wide; the `control_mp_cost`
                    // family ("all Operators in the Control Center") is CC-room-only.
                    //
                    // An aura is credited ONLY when the sentence's subject really
                    // is an aura scope. This family also holds named-partner gates
                    // (Mr. Lee's "together with Aak" +0.25, the Amiya pair skills)
                    // and self-subject texts (Gladiia's Abyssal-conditional self
                    // ±0.5, the self-drain riders) - a flat parse credited all of
                    // them as unconditional room auras, inflating the sustain sim.
                    // Partner-gated and self-subject variants price 0 (never-guess:
                    // the gate/condition isn't resolvable at parse time); self
                    // DRAINS still ride the `parse_morale_loss_increase` side-map.
                    let partner_gated = RE_CC_WITH.is_match(&buff.description);
                    let aura_scope = desc_lower.contains("operators in the control center")
                        || desc_lower.contains("other building");
                    let recovery = if partner_gated || !aura_scope {
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
                    let bonus = parse_first_pct(&buff.description).unwrap_or(0.0);
                    // Faction-gated global bonuses ("all <Siracusa> Operators…",
                    // "all Trading Posts with 3 <Kjerag> Operators…") must NOT be
                    // credited flat to every room - they depend on each room's team.
                    // The faction token comes from the displayed keyword (Kjerag →
                    // "kjerag", matching operators' nation tag), not the internal
                    // group marker. A "with <N>" clause means the WHOLE post is
                    // gated on holding N of that faction; otherwise it's a per-
                    // operator bonus that each matching operator earns.
                    // Both faction phrasings are conditional: the plural "all
                    // <Kjerag> Operators assigned to Trading Posts gain..."
                    // and Delphine's singular "for each <Glasgow Gang>
                    // Operator assigned to the same Trading Post, +10%"
                    // (audited: hers is the ONLY each-singular in this branch).
                    // Missing the singular routed her to TagBased - a flat
                    // half-credit with no condition, so she earned a CC seat
                    // even when no Glasgow member worked a post, and the
                    // dead-weight reselection never checked her.
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
                        }
                    } else {
                        // Unconditional global ("all Trading Posts +7%").
                        BuffResolutionStrategy::GlobalEffect {
                            target_room: target_room.to_string(),
                            bonus_pct: bonus,
                        }
                    }
                } else if let Some((target_room, c)) = [
                    ("MEETING", &RE_CC_CLUE),
                    ("TRAINING", &RE_CC_TRAIN),
                    ("HIRE", &RE_CC_HIRE),
                ]
                .iter()
                .find_map(|(room, rx)| rx.captures(&buff.description).map(|c| (*room, c)))
                    && !buff.description.contains("assigned to the Reception Room")
                {
                    // Non-production CC skill (clue / training / HR), priced in its
                    // own units - see ControlNonProduction. Placed last so every
                    // production branch keeps priority. Audited 2026-08-13: within
                    // CONTROL this captures upMeetingSpeed x2, meeting_spd&bd,
                    // mp&meet_spd (Sakiko same-room gate), meeting&mp_cost x2
                    // (their drain rides the hoisted extraction), train_spd x3,
                    // hire_spd&bd. The Reception-Room guard excludes meeting&ord
                    // x2 - their clue part is gated on Ines elsewhere and their
                    // order-limit part targets Hoederer's post, cross-room
                    // machinery we don't price: whole-buff unresolved beats a
                    // half-parse. control_hire_spd's aggregate-state conditional
                    // self-excludes by phrasing.
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
                    // Other buffs
                    let value = parse_first_pct(&buff.description).unwrap_or(0.0);
                    BuffResolutionStrategy::Complex {
                        estimated_pct: value,
                    }
                }
            }
            "MANUFACTURE" | "TRADING" | "POWER" => {
                // Base-wide named-operator conditional: "+30%, and +5% more when Ines or W is
                // assigned to any Work Area" (Hoederer). The bonus depends on a named operator
                // actively WORKING somewhere in the base, distinguished by the "Work Area" phrasing
                // (vs the same-room "...same Trading Post as <op>"). Carries the base, the bonus %
                // stated after the named list, and the required operators; the optimizer credits the
                // bonus only once it knows who is deployed in a work area.
                let base_wide = buff
                    .description
                    .contains("Work Area")
                    .then(|| find_all_operator_char_ids(&buff.description, name_to_char))
                    .filter(|(ids, _)| !ids.is_empty());
                // Deployment-context gate: the bonus needs a specific operator (or faction)
                // stationed in a specific room TYPE somewhere in the base, not this room.
                // Must precede the base-wide/teammate branches (its "is assigned to a
                // <Room>" phrasing carries no "Work Area"/"same" marker to catch it).
                if let Some(gate) = parse_room_presence_gate(
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
                    }
                }
                // Named-teammate conditional. Handles both phrasings:
                //   "...same Trading Post as <@cc.kw>Lappland</> … +65%"   (Texas)
                //   "+20%; if <@cc.kw>Exusiai</> … same Trading Post … +25%" (Lemuen)
                // The base efficiency (always applied) is the Efficiency field; the
                // bonus is the % stated alongside the named operator. Faction-count
                // buffs ("for every Glasgow Gang operator…") are excluded.
                else if buff.description.contains("same")
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
                    BuffResolutionStrategy::CapacityOnly {
                        order_limit: parse_order_limit(&buff.description).unwrap_or(0),
                    }
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
                    // A targeted regex, not parse_first_pct: the first % in the text is the
                    // [030] tier's base (20%), not the per-hour ramp rate we need here.
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
                        includes_self: false, // "from teammates" - excludes Degenbrecher's own -6
                    }
                }
                // Jaye's "Investment Solicitations": "+4% per order limit increase from others"
                else if prefix == "trade_ord_spd_variable" {
                    let per = parse_first_pct(&buff.description).unwrap_or(4.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: 1.0,
                        bonus_per_threshold: per,
                        cap_pct: f64::MAX,
                        includes_self: false, // "from others"
                    }
                }
                // Vermeil-type: factory productivity scales with the team's capacity-limit
                // boosts ("+X% productivity per capacity limit increase"). The manufacture
                // analogue of Jaye's order-limit scaling - strong in capacity-stacking teams.
                // Counts the WHOLE factory's capacity, including this operator's own (+8).
                else if prefix == "manu_prod_spd_variable" {
                    let per = parse_first_pct(&buff.description).unwrap_or(2.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: 1.0,
                        bonus_per_threshold: per,
                        cap_pct: f64::MAX,
                        includes_self: true,
                    }
                }
                // Bubble E1: per-operator capacity tiers - each operator in the factory gains
                // `low`% if its capacity bonus is <= threshold, else `high`%. Strong when paired
                // with high-capacity operators (Vulcan/Ceobe/Wulfenite etc.).
                else if prefix == "manu_prod_spd_variable3" {
                    let threshold =
                        parse_first_vup_number(&buff.description).unwrap_or(16.0) as i32;
                    let low = parse_nth_pct(&buff.description, 0).unwrap_or(1.0);
                    let high = parse_nth_pct(&buff.description, 1).unwrap_or(3.0);
                    BuffResolutionStrategy::CapacityTierScaling {
                        threshold,
                        low_pct: low,
                        high_pct: high,
                    }
                }
                // Recipe-type scaling (Quartz "Precise Scheduling"): a base trading
                // efficiency PLUS "+N% per recipe type being processed at Factories".
                // Scales on the count of DISTINCT recipe types (the synthetic
                // `MANUFACTURE_RECIPE_TYPES` count - gold + EXP is 2, not the 4-factory
                // count), with the base % from the Efficiency field and the per-unit % the
                // trailing %.
                else if prefix.contains("&formula") && buff.description.contains("recipe type") {
                    let per_unit = parse_last_pct(&buff.description).unwrap_or(2.0);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "MANUFACTURE_RECIPE_TYPES".to_string(),
                        per_unit_pct: per_unit,
                        per_level: false,
                        nullifies_others: false,
                        base_pct: f64::from(buff.efficiency),
                        cap_pct: None,
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
                        cap_pct: None,
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
                        cap_pct: None,
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
                        cap_pct: None,
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
                // Building-resource dependent. Two kinds, both worth 0 in the baseline:
                //   - Unstockable consumables (Marcille's "+1% per Monster Meal",
                //     Engineering Robots, Witchcraft Crystal): can't assume the player
                //     has any banked, and crediting the per-unit % (as if one unit were
                //     stocked) over-ranks the operator against reliable specialists.
                //   - The Perception Information / Chain of Thought / Soundless Resonance
                //     economy (Rosmontis, Ebenholz, ...): a base-wide resource loop fed
                //     by operators resting in dorms (and CC/HR/Training generators). Real
                //     and potentially large, but it spans the whole base, so it needs a
                //     cross-building resource simulation rather than a per-room estimate -
                //     0 until that exists, which is safer than a wrong per-unit guess.
                // Any always-on flat productivity lives in a separate base-skill slot,
                // captured by the `efficiency > 0` branch above.
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
                        cap_pct: None,
                    }
                }
                // Drone-recovery power skill scaling with max Drone capacity (Greyy the
                // Lightningbearer's "+1% Drone recovery rate for every 10 max Drone capacity
                // (Max +25%)"). Scales on the base's ACTUAL max drone capacity - the synthetic
                // `DRONE_CAPACITY` facility count (a 3x L3-plant base holds 235 drones, so the
                // skill reads +23.5%, under its +25% ceiling) - with the stated cap carried.
                else if buff.room_type == "POWER"
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
                }
                // Non-capacity-scaled drone skill with its % only in the description.
                else if buff.room_type == "POWER" && buff.description.contains("Drone recovery") {
                    let value = parse_last_pct(&buff.description)
                        .or_else(|| parse_first_pct(&buff.description))
                        .unwrap_or(0.0);
                    BuffResolutionStrategy::DirectEfficiency { value }
                }
                // Fallback
                else {
                    let est = parse_first_pct(&buff.description).unwrap_or(15.0);
                    BuffResolutionStrategy::Complex { estimated_pct: est }
                }
            }
            _ => BuffResolutionStrategy::CapacityOnly { order_limit: 0 },
        };

        // A pool-scaling tail ("plus an additional +X% for every <res>")
        // composes over whatever the base parse produced, so the flat parts
        // keep their pricing and the tail becomes a ScalingPoolPoints clause.
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
        // Higher-yield order chance (Tailoring etc.). The E0/E1 tier reads "increased
        // slightly"; the promoted (E2) tier drops "slightly" for a stronger shift. Value them
        // apart so an E2 trader (e.g. Bibeak's [010]) outranks the un-promoted "slightly" tier
        // of the same kind of trader (e.g. an E1 Kafka's [001]), while two E2 traders tie.
        let pct = if desc.contains("slightly") { 5.0 } else { 10.0 };
        Some((pct, false))
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

/// The sustained ceiling of a time-ramping reception skill ("…then by 2% per hour, up to a maximum
/// of 30%") - the value it holds during continuous operation. `None` when the skill has no ramp.
fn parse_reception_ceiling(desc: &str) -> Option<f64> {
    // "maximum of" is ASCII, so the lowercased index is a valid byte offset into the original.
    let idx = desc.to_lowercase().find("maximum of")?;
    parse_first_pct_from(desc, idx)
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

/// Every `<@cc.kw>…</>` keyword that resolves to a known operator, as `char_id`s, plus the byte
/// offset just past the LAST one (so the caller can read a bonus % stated after the named list).
/// Handles multi-operator conditions ("when <Ines> or <W> are assigned…").
fn find_all_operator_char_ids(
    desc: &str,
    name_to_char: &HashMap<String, String>,
) -> (Vec<String>, usize) {
    let mut ids = Vec::new();
    let mut last_end = 0;
    for cap in RE_KW_BLOCK.captures_iter(desc) {
        let name = RE_INNER_TAG.replace_all(&cap[1], "").trim().to_lowercase();
        if let Some(char_id) = name_to_char.get(&name) {
            ids.push(char_id.clone());
            if let Some(block) = cap.get(0) {
                last_end = block.end();
            }
        }
    }
    (ids, last_end)
}

/// Room labels as they appear in buff text -> internal room type. A documented
/// text-identifier binding (like the facility-count enabler mapping): gamedata
/// carries no label->room-type table.
fn room_type_from_label(label: &str) -> Option<&'static str> {
    Some(match label {
        "Factory" => "MANUFACTURE",
        "Trading Post" => "TRADING",
        "Control Center" => "CONTROL",
        "Power Plant" => "POWER",
        "Training Room" => "TRAINING",
        "Dormitory" => "DORMITORY",
        "Reception Room" => "MEETING",
        "Office" => "HIRE",
        "Workshop" => "WORKSHOP",
        _ => return None,
    })
}

/// Deployment-context gates: "if <op> is assigned to the <Room>", "and <op> is
/// in a <Room>", "if another <faction> Operator is assigned to a <Room>".
/// Audited across all gamedata buffs (2026-08-13): within the production room
/// arms the named form captures exactly `manu_formula_spd_P[000]` (Gummy in a
/// Trading Post) and `power_rec_spd_P[000]/[001]` (Kal'tsit in the Control
/// Center, Logos as the Trainer); the faction form captures exactly
/// `power_rec_spd_ext&faction[000]` (another Laterano op in a Power Plant).
fn parse_room_presence_gate(
    desc: &str,
    base_efficiency: f64,
    name_to_char: &HashMap<String, String>,
) -> Option<BuffResolutionStrategy> {
    static RE_GATE_CHAR: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?:[Aa]nd|[Ii]f) <@cc\.kw>([^<]+)</> is (?:assigned to be the Trainer in|assigned to|in) (?:a|an|the) (Factory|Trading Post|Control Center|Power Plant|Training Room|Dormitory|Reception Room|Office|Workshop)",
        )
        .unwrap()
    });
    static RE_GATE_FACTION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"[Ii]f another <\$cc\.g\.([a-z0-9]+)><@cc\.kw>[^<]+</></> Operator is assigned to (?:a|an|the) (Factory|Trading Post|Control Center|Power Plant|Training Room)",
        )
        .unwrap()
    });
    if let Some(c) = RE_GATE_CHAR.captures(desc) {
        let room_type = room_type_from_label(&c[2])?;
        let end = c.get(0)?.end();
        // An unresolvable name leaves the char list empty: the gate then never
        // fires and only the base is credited (never guess).
        return Some(BuffResolutionStrategy::ConditionalOnRoomPresence {
            required_char_ids: name_to_char
                .get(&c[1].to_lowercase())
                .cloned()
                .into_iter()
                .collect(),
            required_faction: None,
            required_count: 1,
            room_type: room_type.to_string(),
            base_efficiency,
            bonus_efficiency: parse_first_pct_from(desc, end).unwrap_or(0.0),
        });
    }
    if let Some(c) = RE_GATE_FACTION.captures(desc) {
        let room_type = room_type_from_label(&c[2])?;
        let end = c.get(0)?.end();
        return Some(BuffResolutionStrategy::ConditionalOnRoomPresence {
            required_char_ids: Vec::new(),
            required_faction: Some(c[1].to_lowercase()),
            // "another <faction> Operator": the buff only matters while its
            // same-faction owner works that room type too, so owner + another
            // = 2 matching operators deployed there.
            required_count: 2,
            room_type: room_type.to_string(),
            base_efficiency,
            bonus_efficiency: parse_first_pct_from(desc, end).unwrap_or(0.0),
        });
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

/// The `(percent, units)` of a per-capacity rate: "+1% Drone recovery rate for every
/// 10 max Drone capacity" → `(1.0, 10.0)`.
fn parse_per_capacity_rate(desc: &str) -> Option<(f64, f64)> {
    static RE_PER_CAPACITY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"for every\s*(?:<[^>]+>)?\s*(\d+)\s*(?:</>)?\s*max Drone capacity").unwrap()
    });
    let per_pct = parse_first_pct(desc)?;
    let units: f64 = RE_PER_CAPACITY
        .captures(desc)?
        .get(1)?
        .as_str()
        .parse()
        .ok()?;
    (units > 0.0).then_some((per_pct, units))
}

/// Extract first float like "+0.7" from description markup (for morale values)
fn parse_first_float(desc: &str) -> Option<f64> {
    RE_FIRST_FLOAT
        .captures(desc)
        .and_then(|c| c[1].parse().ok())
}

/// Base-wide morale recovery from a Control-Center buff. Prefers the figure tied to "Morale"
/// (e.g. "recover <@cc.vup>+0.05</> Morale per hour"); a buff whose only `<@cc.vup>` number is
/// a perception-RESOURCE generation (it mentions a `<$cc.bd_…>` resource like Worldly Plight)
/// recovers no morale at all - that economy is valued separately, not as base-wide morale.
/// Otherwise falls back to the first `<@cc.vup>` figure (buffs phrased without "Morale" nearby).
fn parse_morale_recovery(desc: &str) -> Option<f64> {
    if let Some(c) = RE_MORALE_RECOVERY.captures(desc) {
        return c[1].parse().ok();
    }
    if desc.contains("<$cc.bd_") {
        return Some(0.0);
    }
    parse_first_float(desc)
}

/// Extract the faction/tag token from a `<@cc.kw>…</>` keyword, normalised to match an
/// operator's faction tag (`nation_id`/`group_id`/`team_id`). A dotted acronym like "L.G.D."
/// collapses to its letters ("lgd"), which is how the L.G.D. group is tagged - a plain word
/// match would otherwise drop it and the conditional buff would be credited unconditionally.
/// Anything else takes its leading word ("Blacksteel Worldwide" -> "blacksteel", "Kjerag" ->
/// "kjerag"), matching the existing behaviour.
fn parse_tag_keyword(desc: &str) -> Option<String> {
    let kw = RE_TAG_KEYWORD.captures(desc)?.get(1)?.as_str();
    if kw.contains('.') && kw.chars().all(|c| c.is_ascii_alphabetic() || c == '.') {
        let collapsed: String = kw
            .chars()
            .filter(char::is_ascii_alphabetic)
            .map(|c| c.to_ascii_lowercase())
            .collect();
        if !collapsed.is_empty() {
            return Some(collapsed);
        }
    }
    let first: String = kw
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    (!first.is_empty()).then_some(first)
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

/// A morale effect one operator's buff applies to a DIFFERENT co-seated
/// operator - the Ave Mujica drama riders ("Morale consumed per hour by
/// Sakiko Togawa +0.1" while sharing the Control Center), Mortis' amnesty
/// ("ignores the self Morale loss effect from her own base skill"), and
/// Nian's faction-wide version ("remove any Morale reduction effects from
/// Sui Operators ... that affect themselves").
#[derive(Clone, Debug, PartialEq)]
pub enum TargetedMoraleEffect {
    /// The named target drains `delta` more per hour while the owner shares
    /// the room. `None` target = the name didn't resolve; the effect is inert.
    Rider { target: Option<String>, delta: f64 },
    /// The named target's OWN self-drain-increase riders are negated while
    /// the owner shares the room.
    NegatesOwnLoss { target: Option<String> },
    /// Every co-seated operator carrying the faction tag has their own
    /// self-drain-increase riders negated (the owner included).
    NegatesFactionOwnLoss { faction: String },
    /// The owner ignores every morale effect TEAMMATES project into the room
    /// (Waaifu's Team Spirit: room drain auras don't touch her, in either
    /// direction).
    SelfAuraImmunity,
    /// The owner's drain shifts by `delta` while their room produces one of
    /// `targets` (Cement's Vlog: -0.25 while making Battle Records).
    SelfFormulaDrain { targets: Vec<String>, delta: f64 },
}

static RE_TARGETED_RIDER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"Morale consumed per hour by <@cc\.kw>([^<]+)</>\s*<@cc\.(?:vup|vdown)>\+?([\d.]+)</>",
    )
    .unwrap()
});
static RE_NEGATES_OWN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"ignores? the self Morale loss effect").unwrap());
static RE_NEGATES_FACTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<@cc\.kw>remove</> any Morale reduction effects from <\$cc\.g\.([a-z0-9_]+)>")
        .unwrap()
});
static RE_AURA_IMMUNITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"<@cc\.kw>ignore</> the effects of any Operators stationed in (?:that|the) Factory that would affect the Morale consumption of <@cc\.kw>this Operator",
    )
    .unwrap()
});
static RE_FORMULA_DRAIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"Morale consumed when producing <@cc\.kw>[^<]+</> is reduced by <@cc\.vup>-([\d.]+)</>",
    )
    .unwrap()
});

/// True when the buff carries any targeted morale effect - used by the
/// unresolved-inventory relabel: a buff whose whole effect the targeted
/// side-channel prices needs no Unresolved marker.
pub fn has_targeted_morale_effect(desc: &str) -> bool {
    RE_TARGETED_RIDER.is_match(desc)
        || RE_NEGATES_OWN.is_match(desc)
        || RE_NEGATES_FACTION.is_match(desc)
        || RE_AURA_IMMUNITY.is_match(desc)
        || RE_FORMULA_DRAIN.is_match(desc)
}

/// Extract every targeted morale effect from the buff table. Audited
/// 2026-08-13: riders = `control_mp&meet_spd`[000] (+0.05 on Sakiko) and
/// `control_dorm_rec2`[000] (+0.1 on Sakiko); own-loss negation =
/// `control_mp_cost_reset`[000] (Mortis, companion-gated on Sakiko); faction
/// negation = `control_facCostReset`[000] (Sui).
pub fn targeted_morale_effects(
    buffs: &HashMap<String, Buff>,
    name_to_char: &HashMap<String, String>,
) -> HashMap<String, TargetedMoraleEffect> {
    let mut out = HashMap::new();
    for (buff_id, buff) in buffs {
        if let Some(c) = RE_TARGETED_RIDER.captures(&buff.description) {
            out.insert(
                buff_id.clone(),
                TargetedMoraleEffect::Rider {
                    target: name_to_char.get(&c[1].to_lowercase()).cloned(),
                    delta: c[2].parse().unwrap_or(0.0),
                },
            );
        } else if RE_NEGATES_OWN.is_match(&buff.description) {
            // The protected companion is the "assigned ... with <op>" name.
            let target = RE_CC_WITH
                .captures(&buff.description)
                .and_then(|w| name_to_char.get(&w[1].to_lowercase()).cloned());
            out.insert(
                buff_id.clone(),
                TargetedMoraleEffect::NegatesOwnLoss { target },
            );
        } else if let Some(c) = RE_NEGATES_FACTION.captures(&buff.description) {
            out.insert(
                buff_id.clone(),
                TargetedMoraleEffect::NegatesFactionOwnLoss {
                    faction: c[1].to_lowercase(),
                },
            );
        } else if RE_AURA_IMMUNITY.is_match(&buff.description) {
            out.insert(buff_id.clone(), TargetedMoraleEffect::SelfAuraImmunity);
        } else if let Some(c) = RE_FORMULA_DRAIN.captures(&buff.description) {
            // The produced good is the buff's Targets entry (F_EXP for
            // Battle Records) - no product-name mapping needed.
            out.insert(
                buff_id.clone(),
                TargetedMoraleEffect::SelfFormulaDrain {
                    targets: buff.targets.clone(),
                    delta: -c[1].parse().unwrap_or(0.0),
                },
            );
        }
    }
    out
}

/// Per-recruit-slot HR speed: "+10% HR contacting speed for every Recruit
/// slot other than the initial slot" (Lin's Meritocracy). The slot count is
/// ACCOUNT state the sync cannot read, so the base parse prices the rider 0
/// (never-guess) and this resolver re-prices it when the player declares the
/// fact.
static RE_HR_PER_SLOT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:<@cc\.vup>)?\+([\d.]+)%(?:</>)? HR contacting speed for every Recruit slot")
        .unwrap()
});

/// Registry rewrite for player-declared account facts (the established
/// resolve-* pattern: never mutate, return a re-priced copy). Currently one
/// fact exists: `open_recruit_slots` - recruit slots purchased beyond the
/// initial one (0-3) - which prices the per-slot HR-speed riders that
/// otherwise resolve to 0. Audited 2026-08-23: `RE_HR_PER_SLOT` captures
/// exactly `hire_spd_cost&extra[000]` (Lin, flat 0 + 10/slot); the other
/// slot-gated buffs' riders are clue likelihoods, drains, or resource points
/// - none of them the buff's PRICED value - and stay untouched.
pub fn resolve_account_facts(
    registry: &HashMap<String, BuffResolutionStrategy>,
    buffs: &HashMap<String, Buff>,
    open_recruit_slots: u32,
) -> HashMap<String, BuffResolutionStrategy> {
    let mut out = registry.clone();
    for (buff_id, strategy) in registry {
        let Some(buff) = buffs.get(buff_id) else {
            continue;
        };
        if let (BuffResolutionStrategy::NonProduction { value }, Some(c)) =
            (strategy, RE_HR_PER_SLOT.captures(&buff.description))
        {
            let per_slot: f64 = c[1].parse().unwrap_or(0.0);
            out.insert(
                buff_id.clone(),
                BuffResolutionStrategy::NonProduction {
                    value: value + per_slot * f64::from(open_recruit_slots),
                },
            );
        }
    }
    out
}

/// The alternative self-drain phrasing: "self Morale loss per hour
/// <@cc.vdown>+N</>". Companion-gated forms ("when assigned together with
/// <op>, ... Morale loss +N") are SKIPPED - capturing them flat would charge
/// drain the operator only pays alongside a partner, fabricating depletion
/// warnings. Audited 2026-08-13: captures exactly `control_bd_spd`[000],
/// `control_mp_cost&bd_up`[000] (Chongyue +0.5), `control_mp_cost&bd2`[010]
/// (+0.5), and `control_mp_cost&bd3`[000] (Sakiko +0.05, whose TRAILING
/// "when Passion is 40 or higher" condition holds at any committed plan's
/// steady-state pool - the flat capture is the steady-state model).
pub fn parse_morale_loss_increase(desc: &str) -> Option<f64> {
    static RE_MORALE_LOSS: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"[Ss]elf Morale loss per hour <@cc\.vdown>\+([\d.]+)</>").unwrap()
    });
    let c = RE_MORALE_LOSS.captures(desc)?;
    let start = c.get(0)?.start();
    let mut from = start.saturating_sub(120);
    while from > 0 && !desc.is_char_boundary(from) {
        from -= 1;
    }
    if desc[from..start].contains("together with") {
        return None;
    }
    c[1].parse().ok()
}
