use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::core::gamedata::types::building::Buff;
use crate::core::gamedata::types::operator::Operator;

use super::clause::{PEER_STAGE_FIXED_LIMIT, PEER_STAGE_NET_LIMIT};
use super::pools::ROBOTS_IN_POWER;
use super::util::buff_family;
use crate::core::gamedata::types::consts::GameDataConst;

/// Lowercased display name -> `char_id`, for buff text naming teammates
/// ("...the same Trading Post as Lappland").
pub fn build_name_to_char(operators: &HashMap<String, Operator>) -> HashMap<String, String> {
    operators
        .iter()
        .map(|(char_id, op)| (op.name.to_lowercase(), char_id.clone()))
        .collect()
}

/// Lowercased group/nation/team ids, the match tags for count scalers.
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
    // "Robot" recruitment tag (Lancet-2, Castle-3, ...), counted by Alanna's
    // Operation Platforms and Overclock. No faction token is "robot", so no
    // collision.
    if op.tag_list.iter().any(|s| s == "Robot") {
        push("robot");
    }
    // Race from the handbook ("[Race] Durin"), for "<$cc.tag.durin>Durin
    // Operator" counts (Pozëmka). Lowercased enum name = the markup token.
    if let Some(race) = op
        .profile
        .as_ref()
        .and_then(|p| serde_json::to_value(&p.basic_info.race).ok())
        .and_then(|v| v.as_str().map(str::to_lowercase))
    {
        push(&race);
    }
    // A secondary NATION counts (Texas: lungmen, SubPower siracusa; "all
    // Siracusa Operators" reach her, community-verified). A secondary GROUP
    // does not: Vina Victoria's SubPower {glasgow} isn't counted by Delphine's
    // "Glasgow Gang Operator" (in-game 2026-09-07). MainPower repeats the
    // top-level ids; dedup makes it a no-op.
    let powers = op.main_power.iter().chain(op.sub_power.iter().flatten());
    for nation in powers.filter_map(|p| p.nation_id.as_ref()) {
        let lower = nation.to_lowercase();
        if !lower.is_empty() && !tags.contains(&lower) {
            tags.push(lower);
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

/// First signed % (a CC grant can be a malus: Gnosis's "order acquisition
/// efficiency <vdown>-15%</>").
static RE_FIRST_SIGNED_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.(?:vup|vdown)>([+-]?[\d.]+)%</>").unwrap());

static RE_FIRST_FLOAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)</>").unwrap());

/// `<@cc.vup>+N</>` right before "Morale", not an earlier resource number
/// ("…Worldly Plight<@cc.vup>+5</>").
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

/// Own-room occupant generator: "for every 1 Operators in that Dormitory,
/// <Resource> +N" (Virtuosa).
static RE_POOL_GEN_OWN_OCC: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.vup>1</>\s*Operators? in that (?:Dormitory|Factory|Trading Post|Power Plant|Reception Room|Office|Workshop|Training Room),\s*<\$cc\.([A-Za-z0-9_]+)>[^+]{0,40}?<@cc\.vup>\+([\d.]+)</>",
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

/// Alanna: "productivity +PCT% for every <tag.op> Operation Platform assigned to
/// a Power Plant". Pseudo-pool of Robot-tagged operators in POWER rooms.
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

/// "each"/"every" right before a faction token ("for each <Glasgow Gang>
/// Operator...", Delphine), vs the plural "all <Kjerag> Operators...".
/// Adjacency keeps "each hour" out.
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

fn room_type_from_global_label(label: &str) -> &'static str {
    if label.starts_with("Trading") {
        "TRADING"
    } else {
        "MANUFACTURE"
    }
}

/// The rider a facility-count modifier needs met, read off the deployment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FacilityGate {
    None,
    /// "if Lancet-2 is assigned to a Power Plant" (Eunectes).
    NamedCharInRoom {
        char_id: String,
        room: String,
    },
    /// "if there are no Operation Platforms in other Power Plants" (Greyy the
    /// Lightningbearer): no Robot-tagged operator seated in the target room type.
    NoRobotsInOtherRooms,
}

/// A facility-count modifier's named gate ("if <NAME> is assigned to a <Room>").
static RE_FACILITY_GATE_CHAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"if <@cc\.kw>([^<]+)</> is assigned to (?:a|an|the) <@cc\.kw>([A-Za-z ]+?)</>")
        .unwrap()
});

/// A production-room skill that boosts the room a NAMED operator occupies
/// ("increases the productivity of the Factory <Wild Mane> is assigned to by +5%").
static RE_TARGET_ROOM_BOOST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"increases the productivity of the (Factory|Trading Post) <@cc\.kw>([^<]+)</> is assigned to by <@cc\.vup>\+([\d.]+)%</>",
    )
    .unwrap()
});

/// The first tag token a buff names (`<$cc.tag.durin>` -> "durin").
static RE_TAG_MARKUP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<\$cc\.tag\.([a-z0-9_]+)>").unwrap());

/// "caps at <N>" on a base-wide count.
static RE_CAPS_AT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"caps at <@cc\.kw>(\d+)</>").unwrap());

/// Named-operator-gated CC grant: fires while `char_id` sits in a
/// `target_room`-type room, and lands ON that room.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedCharGrant {
    pub char_id: String,
    /// Game room constant the named operator must be seated in.
    pub target_room: String,
    /// Order/capacity limit added to the room seating the named operator.
    pub order_limit: f64,
    /// Non-production speed % (clue collection etc.), priced in the boosted
    /// facility's own units like `ControlNonProduction`.
    pub nonprod_pct: f64,
}

/// A named-operator room gate inside a CC buff: "if <@cc.kw>NAME</> is
/// assigned to (the Reception Room|a Trading Post|...), <payload>". Captures
/// (name, room label, payload segment up to the next gate or end).
static RE_CC_NAMED_GATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"if <@cc\.kw>([^<]+)</> is assigned to (?:the |a |an )?([A-Za-z' ]+?)\s*,\s*([^;]*)",
    )
    .unwrap()
});

/// The order/capacity-limit payload of a named-gate segment.
static RE_NAMED_GATE_ORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"order limit <@cc\.vup>\+(\d+)</>").unwrap());

/// Waai Fu's Team Spirit: ignores roommates' effects on her own drain.
static RE_DRAIN_AURA_IMMUNITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"ignore</> the effects of any Operators stationed in (?:that|the) [A-Za-z ]+ that would affect the Morale consumption of <@cc\.kw>this Operator",
    )
    .unwrap()
});

/// Non-production CC effects, each in its facility's units: clue speed
/// (Reception), Specialization speed (Training), contact speed (HR Office).
static RE_CC_CLUE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"clue collection speed <@cc\.vup>\+([\d.]+)%</>").unwrap());
static RE_CC_TRAIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Specialization training speed <@cc\.vup>\+([\d.]+)%</>").unwrap()
});
static RE_CC_HIRE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"HR contacting speed <@cc\.vup>\+([\d.]+)%</>").unwrap());
/// A Control-Center morale aura paid per seated operator of a faction:
/// "each <$cc.g.lgd>...</> Operator", "each Operator from <$cc.g.R6>",
/// "for each <$cc.g.karlan>...</> Operator assigned to the Control Center".
static RE_CC_EACH_FACTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)each (?:operator from )?<\$cc\.g\.[a-z0-9]+>").unwrap());

/// A same-room companion gate: "assigned to the Control Center with <op>".
static RE_CC_WITH: LazyLock<Regex> = LazyLock::new(|| {
    // Also Mr. Lee's "assigned together with <Aak> to the Control Center".
    Regex::new(r"assigned (?:to the Control Center with|together with) <@cc\.kw>([^<]+)</>")
        .unwrap()
});

/// Counts SKILLS ("for each Rhine Tech-type skill", "per Standardization
/// Skill"), not operators ("per Glasgow Gang Operator").
static RE_COUNT_SKILLS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:for each|for every|per) [A-Za-z' -]+?[Ss]kills?\b").unwrap());

/// CC global split by PRODUCT (Flametail: "+10% productivity towards Battle
/// Records and -10% productivity towards Precious Metals").
static RE_TOWARDS_PRODUCT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<@cc\.v(?:up|down)>([+-]?[\d.]+)%</> productivity towards <@cc\.kw>([^<]+)</>")
        .unwrap()
});

/// Unknown product -> None, so the split bonus is worth 0, never a guess.
fn formula_for_product(name: &str) -> Option<&'static str> {
    let n = name.to_lowercase();
    if n.contains("battle record") {
        Some("F_EXP")
    } else if n.contains("precious metal") || n.contains("pure gold") {
        Some("F_GOLD")
    } else if n.contains("originium") {
        Some("F_DIAMOND")
    } else {
        None
    }
}

/// A dormitory single-target healer: "restores +X to an(other) Operator in
/// that Dormitory (whose Morale is not full)".
static RE_DORM_SINGLE_TARGET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"to an(?:other)? Operator (?:assigned to|in) (?:that|the) Dorm(?:itory)?").unwrap()
});

/// Whole-dorm aura: "restores +X Morale per hour to all (other) Operators
/// assigned to that Dormitory". Explicit so compound texts classify as AURA; the
/// `contains("self")` heuristic filed both shapes as self-only, hiding them from
/// dorm staffing:
/// - Durin's "self Morale recovered per hour -0.1, but restores +0.2 ... to all
///   Operators". Community-confirmed 2026-08-24: the aura covers her too (net
///   +0.1), so the self-malus needs no field.
/// - "self +0.55, and restores +0.1 ... to all OTHER Operators". The self rider
///   is priced nowhere (only the Fiammetta swap path reads self-only rates), so
///   the aura is the whole value.
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

/// Craftsmanship family: "productivity -5%, capacity limit +16". The morale
/// rider goes through the drains side-map.
static RE_SPEED_CAPACITY_TRADE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"[Pp]roductivity <@cc\.(vup|vdown)>([+-]?[\d.]+)%</>(?:,| and)\s*capacity limit <@cc\.(vup|vdown)>([+-]?[\d.]+)</>",
    )
    .unwrap()
});

/// A standalone converter: "every F <From> is converted to 1 <To>".
static RE_POOL_CONVERT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:convert )?every <@cc\.vup>([\d.]+)</>\s*(?:levels? of |points? of )?<\$cc\.([A-Za-z0-9_]+)>.{0,50}?(?:is converted (?:in)?to|to) <@cc\.vup>1</>\s*(?:levels? of |points? of )?<\$cc\.([A-Za-z0-9_]+)>",
    )
    .unwrap()
});

/// Ramp rate, either word order: "+1% per hour" (Ceobe) or "productivity per
/// hour +2%" (Aroma; the first form alone missed it and halved her ramp).
static RE_PER_HOUR_PCT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:<@cc\.vup>\+?([\d.]+)%?</>\s*per hour|per hour <@cc\.vup>\+?([\d.]+)%?</>)")
        .unwrap()
});

/// Nullifier granting to the ROOM per occupant (Snegurochka: "every Operator in
/// that Factory increases that Factory's Productivity by +10% and Capacity limit
/// by +5"; her lower tier has only the capacity half).
static RE_NULLIFY_ROOM_PER_OP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"every Operator in that Factory increases that Factory's (?:Productivity by <@cc\.vup>\+([\d.]+)%</> and )?Capacity limit by <@cc\.vup>\+([\d.]+)</>",
    )
    .unwrap()
});

/// Whisperain: "for every Recruit slot (Default slots do not count), Memory
/// Fragments +10". The sync can't read slots; the player declares them under
/// [`ACCOUNT_FACTS_KEY`].
pub(crate) static RE_SLOT_GRANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every Recruit slot \(Default slots do not count\),\s*<*<\$cc\.(bd_[A-Za-z0-9_]+)>[^+]{0,40}?<@cc\.vup>\+([\d.]+)</>",
    )
    .unwrap()
});

/// Registry key of the synthetic [`BuffResolutionStrategy::AccountFacts`]; never a buff id.
pub const ACCOUNT_FACTS_KEY: &str = "ACCOUNT_FACTS";

// Factory "capacity limit" = trading "order limit" (Vermeil's "+8"). Also the
// long form "capacity limit is increased by +12 when producing Battle Records"
// (Scene's Editing; `Targets` scopes it via the config factor).
static RE_ORDER_LIMIT_POS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:order|capacity) limit(?: is increased by)?\s*<@cc\.vup>\+?(\d+)</>").unwrap()
});

static RE_ORDER_LIMIT_NEG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:order|capacity) limit(?: is (?:reduced|decreased) by| by)?\s*<@cc\.vdown>-?(\d+)</>",
    )
    .unwrap()
});

static RE_NTH_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").unwrap());

static RE_VUP_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>(\d+)</>").unwrap());

// What a count-scaler counts: first keyword after "for each/every".
static RE_COUNT_KEYWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"for (?:each|every).*?<@cc\.kw>([^<]+)</>").unwrap());

// Any <@cc.kw>…</> keyword (multi-word capable), used to parse converter skills.
static RE_KW_ANY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<@cc\.kw>([^<]+)</>").unwrap());

// Keyword block that may nest markup (Lemuen's "<@cc.kw><$cc.angel>Exusiai</></>").
// Non-greedy; RE_INNER_TAG strips the inner tags after.
static RE_KW_BLOCK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<@cc\.kw>(.*?)</>").unwrap());
static RE_INNER_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap());

// "<$cc.g.glasgow>" (group), "<$cc.n.…>" (nation), "<$cc.t.…>" (team).
static RE_FACTION_TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<\$cc\.[gnt]\.(\w+)>").unwrap());

// Shamare: "...each Operator increases … by +X%". Needs per-Operator AND a %, so
// Weedy's "per Power Plant" and Snegurochka's "+N Capacity" don't match.
static RE_NULLIFY_SELF_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"each Operator increases.*?<@cc\.vup>\+?([\d.]+)%").unwrap());

// "per hour" or "each hour" (Penguin Logistics: Texas/Lappland/Exusiai), optional
// "by" (Enforcer's "...BY +2"). Missing either silently drops those drains.
static RE_MORALE_INCREASE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Morale consumed (?:per|each) hour\s*(?:by\s*)?<@cc\.vdown>\+?([\d.]+)</>").unwrap()
});

static RE_MORALE_DECREASE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Morale consumed (?:per|each) hour\s*(?:by\s*)?<@cc\.vup>-?([\d.]+)</>").unwrap()
});

/// Order-value SHAPE, not a %: "+2 gold on orders below 4" depends on the post's
/// level, priced by `order_mix::value_pct` against `OrderRarity`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderEffect {
    /// Proviso's Damages for Breach: orders trading fewer than `below` Pure
    /// Gold ("Defaulted", from Contract Law's text) trade `bonus` more.
    DefaultedGoldBonus { below: u32, bonus: u32 },
    /// Tequila's Investment: orders over `above` Pure Gold pay `lmd` more
    /// (defaulted trades are below anyway).
    HighOrderLmdBonus { above: u32, lmd: u32 },
    /// The Tailoring family: the chance of higher-yield orders is
    /// "increased slightly" (`strong: false`) or "increased" (`strong: true`).
    HigherYieldChance { strong: bool },
    /// A rule with no payoff of its own (Contract Law defines "Defaulted").
    Enabler,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BuffResolutionStrategy {
    /// The buff's Efficiency field, taken as the flat bonus %.
    DirectEfficiency { value: f64 },

    /// e.g. Automation: +X% per Power Plant, nullifies others.
    /// e.g. Quartz: +30% trading, +2% per recipe type at Factories
    ///      (approximated by the Factory count).
    FacilityCountScaling {
        target_room: String,    // "POWER", "TRADING", "DORMITORY", "MANUFACTURE", etc.
        per_unit_pct: f64,      // e.g. 5.0, 10.0, 15.0
        per_level: bool,        // true for dorm-level scaling ("per level of each Dormitory")
        nullifies_others: bool, // true for Automation buffs
        base_pct: f64,          // always-on efficiency added on top of the scaling part
        /// Ceiling on the scaled part, when the buff states one ("Max +25%").
        cap_pct: Option<f64>,
    },

    /// e.g. "+5% per Standardization skill in same Factory"
    TeammateSkillScaling {
        target_buff_pattern: String, // prefix to match, e.g. "manu_prod_spd"
        per_match_pct: f64,
    },

    /// `ratio` % per full `step` % of roommates' OWN skills, up to `cap_pct`.
    /// Heavenly Reward: "+5% per 5% from others, max +25%"; Waai Fu: +5% per 5%,
    /// max +40%, "excluding the additional productivity affected by facility count".
    TeammateOutputMirroring {
        ratio: f64,   // % paid per step, e.g. 5.0
        step: f64,    // % of roommate contribution per payout, e.g. 5.0
        cap_pct: f64, // max bonus, e.g. 25.0
    },

    /// e.g. `SilverAsh`: "+20% efficiency, +4 order limit"
    /// e.g. Degenbrecher: "+25% efficiency, -6 order limit"
    EfficiencyWithOrderLimit {
        efficiency: f64,
        order_limit: i32, // positive = adds CAP, negative = removes CAP
    },

    /// Base efficiency plus a bonus/limit only with a named roommate.
    /// e.g. Texas "Feud": base 0, +65% only with Lappland.
    /// e.g. Lemuen: base +20%, +25% more only with Exusiai.
    /// e.g. Lappland "Hidden Purpose β": +4 order limit only with Texas.
    ///
    /// `required_char_id` None = unresolved name; the conditional part gives 0.
    ConditionalOnTeammate {
        required_char_id: Option<String>,
        base_efficiency: f64,
        efficiency: f64,
        order_limit: i32,
    },

    /// Base plus a bonus while a named operator WORKS anywhere ("any Work Area,
    /// excluding Assistants"), not resting in a dorm. e.g. Hoederer "Starting From
    /// Scratch β": +30%, +5% more when Ines or W works. The optimizer flattens it
    /// per assignment (two-pass); alone the scorer credits the base only.
    ConditionalOnBaseWide {
        required_char_ids: Vec<String>,
        base_efficiency: f64,
        bonus_efficiency: f64,
        /// "When Vigil is in the Base (excluding Assistants and Activity Room
        /// users)" (Bellone, Underflow): a dorm counts. False = "Work Area"
        /// phrasing, where a resting partner doesn't.
        anywhere: bool,
    },

    /// Bonus while a named operator (or enough of a faction) sits in a ROOM TYPE.
    /// e.g. "if Kal'tsit is assigned to the Control Center, drone recovery +5%"
    /// (Roberta), "and Gummy is in a Trading Post, Battle Record formula
    /// productivity +35%", "if another Laterano Operator is assigned to a Power
    /// Plant, +5%". Flattened by `resolve_room_presence`; unresolved = base only.
    ConditionalOnRoomPresence {
        /// Any of these in `room_type` unlocks it. Empty = unresolved, stays off.
        required_char_ids: Vec<String>,
        /// Faction form: lowercased faction token; at least `required_count` operators
        /// carrying it must be deployed in `room_type`.
        required_faction: Option<String>,
        /// 2 for "another <faction> Operator": the same-faction owner works that
        /// room type too, so owner + another = 2.
        required_count: usize,
        /// Internal room type ("TRADING", "CONTROL", "TRAINING", "POWER", ...).
        room_type: String,
        base_efficiency: f64,
        bonus_efficiency: f64,
    },

    /// Jaye's Street Economics: "+X% for every difference of 1 order between the
    /// current number of orders and the maximum", i.e. per EMPTY slot.
    OrderDifferenceScaling { per_order_pct: f64 },

    /// Jaye's Basic Needs (`trade_ord_limit_count`): "reduces the order limit by
    /// 1 for every 10% order acquisition efficiency provided by all other
    /// Operators (to a minimum of 1); furthermore +4% for every 1 order". Rider
    /// pays per FILLED order.
    LimitCutPerPeerEfficiency {
        pct_per_cut: f64,
        cut: i32,
        per_order_pct: f64,
    },

    /// e.g. Degenbrecher E2: "+25% per 5 CAP from teammates, max +100%"
    /// e.g. Swire the Elegant Wit: "+4% per 1 order limit increase from others"
    /// e.g. Vermeil E1: "+2% per capacity limit in the factory" (her OWN +8 counts too)
    OrderLimitScaling {
        per_cap_threshold: f64,   // every N CAP
        bonus_per_threshold: f64, // gives this much %
        cap_pct: f64,             // max bonus
        /// Evaluation stage (`clause::PEER_STAGE_*`): Degenbrecher and Vermeil
        /// read the FIXED limits, Swire reads the limit after Jaye's cut.
        stage: u8,
        /// OWN capacity counts (Vermeil, her own +8). False for "from
        /// others/teammates" (Jaye; Degenbrecher's own -6 must not self-reduce).
        includes_self: bool,
    },

    /// Bubble E1: each occupant, her included, gives `low_pct` if its own capacity
    /// bonus is at or below `threshold`, else `high_pct`, summed over the team.
    CapacityTierScaling {
        threshold: i32,
        low_pct: f64,
        high_pct: f64,
        /// "Does not stack with Recycling and takes priority over it": prefixes of
        /// the named skill's family, resolved by skill NAME. Roommates' clauses
        /// from these don't apply.
        excludes: Vec<String>,
    },

    /// EFFECTIVE facility count +`amount` ("Power Plant +1, only affects facility
    /// quantity": Greyy the Lightningbearer E2; Eunectes E2 "+2"). No productivity
    /// itself; automation (Weedy/Eunectes/Pudding) reads the boosted count.
    FacilityCountModifier {
        target_room: String,
        amount: i32,
        /// Eunectes: Control Center; Greyy: a Power Plant.
        owner_room: String,
        gate: FacilityGate,
    },

    /// Same-tag operators anywhere IN THE BASE, holder included (Nasti's "for
    /// each Rhine Lab Operator in the Base (caps at 5), Precious Metal
    /// productivity +3%"). 0 until the deployment resolves it.
    BaseWideMatchCountScaling {
        token: String,
        per_match_pct: f64,
        cap_count: Option<usize>,
    },

    /// Grant onto the room a NAMED operator occupies (Justice Knight in a Power
    /// Plant: "+5% to the Factory Wild Mane is assigned to"). `active` is set
    /// when the holder sits in `self_room`; then it lands like a per-operator CC
    /// global.
    NamedTargetRoomBoost {
        self_room: String,
        target_char_id: String,
        target_room: String,
        bonus_pct: f64,
        active: bool,
    },

    /// Per teammate matching the keyword in "for each <kw>…", so no faction or
    /// operator name is hardcoded:
    ///   - Dorothy: "+5% per Rhine Tech-type skill"      -> token "rhine"
    ///   - Mizuki:  "+5% per Standardization Skill"       -> token "standardization"
    ///   - Bryophyta:"+5% per Metalwork-type skill"       -> token "metalwork"
    ///   - Morgan:  "+20% per Glasgow Gang Operator"      -> token "glasgow"
    ///
    /// Matches a faction id (group/nation/team) or a skill name's leading word.
    ///
    /// `bonus_char_id` / `bonus_pct`: optional named-roommate rider (Morgan "Gang
    /// Compass": +20% per Glasgow Gang op, +35% more with Siege).
    MatchCountScaling {
        token: String,
        per_match_pct: f64,
        cap_pct: Option<f64>,
        bonus_char_id: Option<String>,
        bonus_pct: f64,
        /// "each <X>-type skill in this Factory": counts SKILLS, the holder's own
        /// included (Dorothy's Rhine Tech β), never faction (Rosmontis is Rhine
        /// Lab with no Rhine Tech skill). False for "per <faction> Operator".
        count_skills: bool,
        /// Pays STORAGE CAPACITY (Astgenne the Lightchaser's "+5 Storage
        /// Capacity for each Rhine Tech-type skill").
        capacity: bool,
    },

    /// Faction analogue of `ConditionalOnTeammate`. e.g. Morgan "Resolution on
    /// Foreign Trade β": +30%, +10% more with any Glasgow Gang roommate.
    ConditionalOnFaction {
        faction_token: String,
        base_efficiency: f64,
        efficiency: f64,
    },

    /// Highmore: "all Rhine Lab and Pinus Sylvestris skills are considered
    /// Standardization skills". Roommates with a `from_tokens` tag gain
    /// `to_token`. No efficiency of its own.
    SkillTypeConversion {
        from_tokens: Vec<String>,
        to_token: String,
    },

    /// e.g. Shamare: "all other Operators' efficiency becomes 0, but each
    /// Operator increases this Operator's efficiency by +45%".
    NullifyTeammatesSelfScaling { per_teammate_pct: f64 },

    /// LMD *per order*, not speed. At a fixed 500 LMD/bar that's the same % of
    /// LMD/hr (gold supply permitting). Not "order acquisition efficiency", so
    /// Tequila survives a Shamare nullifier. `estimated_pct` = LMD-equivalent.
    ///
    /// `pure_gold`: depends on Pure-Gold orders (Proviso's "defaulted" ones).
    /// Shamare shifts to Precious-Metal orders, so Proviso, unlike Tequila,
    /// gains nothing beside her.
    OrderValue {
        effect: OrderEffect,
        pure_gold: bool,
    },

    /// CC global scaling with a POOL: "for every 8 Passion, all Trading Posts'
    /// order efficiency +1%" (Sakiko), "all Factories +1%, with an additional +1%
    /// for every 20 Passion" (Mortis). Context-free = `base_pct` only;
    /// `resolve_global_pool` adds the pool part (live seats or pinned generators).
    GlobalPoolScaling {
        target_room: String,
        base_pct: f64,
        /// +`pct`% per `per` points of `resource` (floored, stepped counter).
        pct: f64,
        per: f64,
        resource: String,
    },

    /// e.g. "all Factories +2%"
    GlobalEffect {
        target_room: String, // "MANUFACTURE", "TRADING"
        bonus_pct: f64,
    },

    /// CC global gated on the CC's own crew: "assigned together with other L.G.D.
    /// Operators to the Control Center, all Factories' productivity +3%"
    /// (Hoshiguma the Breacher). Needs `min_others` OTHER seats with `tag`, by
    /// group id (Swire counts; Swire the Elegant Wit, no group, doesn't).
    /// Resolved by the CC bonus accumulator; alone worth nothing.
    GlobalEffectWithCrewTag {
        target_room: String,
        bonus_pct: f64,
        tag: String,
        min_others: usize,
    },

    /// CC buff on a production room ONLY when its team matches a faction:
    ///   - per-operator (Umiri): "all <Siracusa> Operators in Trading Posts gain
    ///     +5%" -> each matching op adds `bonus_pct` (`faction_token` "siracusa",
    ///     `per_operator` = true, `required_count` = 1).
    ///   - count-gated (SilverAsh): "all Trading Posts with 3 <Kjerag> Operators
    ///     gain +10%" -> the whole post gains `bonus_pct` once it holds
    ///     `required_count` of that faction (`per_operator` = false, required 3).
    ConditionalGlobalEffect {
        target_room: String,
        faction_token: String,
        required_count: usize,
        per_operator: bool,
        bonus_pct: f64,
        /// Per matching operator (Gnosis's "+6 order limit" on Kjerag traders).
        order_limit: i32,
        /// `formula -> pct` (Flametail: +10% Battle Records / -10% Precious
        /// Metals). Empty = `bonus_pct` everywhere; else listed formulas only.
        formula_bonuses: Vec<(String, f64)>,
    },

    /// CC global branching on LAYOUT counts (Wang's Expedience: "if Influence >=
    /// Territory, all Trading Posts +7%; if Territory > Influence, all Factories
    /// +2%"; glossary: Influence = Trading Posts + Power Plants, Territory =
    /// Factories). `resolve_layout_branches` makes it a `GlobalEffect`; else 0.
    LayoutCountBranch {
        a_term: String,
        b_term: String,
        /// Payload when `a >= b`: (target room, %).
        ge_room: String,
        ge_pct: f64,
        /// Payload when `b > a`.
        gt_room: String,
        gt_pct: f64,
    },

    /// CC global gated on another room's DEPLOYMENT (Pudding's Overclock: "if
    /// there are 2 or more Operation Platforms assigned to Power Plants, all
    /// Factories' productivity +2%"). `resolve_room_presence`; else 0.
    RoomPresenceGatedGlobal {
        required_faction: String,
        required_count: usize,
        /// The room type whose crews are counted.
        room_type: String,
        target_room: String,
        bonus_pct: f64,
    },

    /// Wiš'adel's Conspirator: "if Ines is assigned to the Reception Room, clue
    /// collection speed +5%; if Hoederer is assigned to a Trading Post, that
    /// Trading Post's order limit +2". One grant per segment; unresolved names
    /// are dropped.
    NamedCharRoomGrants { grants: Vec<NamedCharGrant> },

    /// "Ignore the effects of any Operators stationed in that <room> that would
    /// affect the Morale consumption of this Operator" (Waai Fu's Team Spirit).
    MoraleDrainAuraImmunity,

    /// e.g. "all Knight operators in Factories +7%"
    TagBased {
        tag: String, // "knight", "sarkaz", "abyssal", etc.
        bonus_pct: f64,
        target_room: String,
    },

    MoraleModifier {
        recovery_per_hour: f64, // positive = recovery, negative = drain
        is_self_only: bool,     // true when only the holder benefits
        /// CC auras reaching OTHER buildings (Chongyue's "Operators working in
        /// other buildings recover +0.05"). False for `control_mp_cost` ("all
        /// Operators in the Control Center") and dorm skills.
        base_wide: bool,
        /// Dorm only: "restores +X to ANOTHER Operator in that Dormitory whose
        /// Morale is not full" (+0.55-class) vs whole-dorm auras (+0.15-class).
        /// Each type is non-stacking within itself.
        single_target: bool,
    },

    /// Capacity only (Vermeil's "capacity limit +8"); feeds limit scalers.
    CapacityOnly { order_limit: i32 },

    /// Non-production facilities (workshop, HR, training, reception); `value`
    /// is the parsed efficiency, scored secondarily.
    NonProduction { value: f64 },

    /// CC boost to a NON-PRODUCTION metric (clues, HR contact, training), in its
    /// OWN units with zero LMD weight: both reference implementations silo or
    /// zero these, and a conversion would be invented. CC spare-seat tie-break
    /// and display only. `same_room_gate`: Some(chars) = only with one of them in
    /// the CC; empty = unresolved, never fires.
    ControlNonProduction {
        /// The boosted facility's room type ("MEETING", "TRAINING", "HIRE").
        target_room: String,
        value: f64,
        same_room_gate: Option<Vec<String>>,
    },

    /// No branch parses the buff; `estimated_pct` is a conservative estimate.
    Complex { estimated_pct: f64 },

    /// Time-averaged over a full 24h shift.
    MoraleDecayEfficiency { time_averaged_value: f64 },

    /// From the LAYOUT: "+N <Resource> per level per building, max CAP"
    /// (Minimalist's Engineering Robots). Needs no assignment.
    PoolGenerateBuildingLevels {
        /// Stable resource key from the `$cc.bd_*` term id, not the display name.
        resource: String,
        per_level: f64,
        cap: f64,
    },

    /// Pool TAIL on an ordinary production skill: "...capacity limit +8,
    /// Productivity +5%, plus an additional +1% for every <Felvine>". `base`
    /// prices the flat parts; the tail adds a `ScalingPoolPoints` clause so it
    /// never silently drops.
    PoolTail {
        base: Box<Self>,
        resource: String,
        /// +`pct`% per `per` points of `resource`.
        pct: f64,
        per: f64,
    },

    /// "+PCT% productivity for every PER <Resource>", floored by PER.
    PoolPointsScaling {
        resource: String,
        per: f64,
        pct: f64,
    },

    /// Senshi: "provide 1 Monster Meal for every level of the current
    /// Dormitory". Settled at assignment scope.
    PoolGenerateOwnRoomLevel { resource: String, per_level: f64 },

    /// Virtuosa: "for every 1 Operators in that Dormitory, Soundless Resonance
    /// +1". Settled at assignment scope.
    PoolGenerateOwnRoomOccupants { resource: String, per: f64 },

    /// Snegurochka's Workflow Optimization: per-occupant grant to the ROOM. The
    /// speed half survives an automation wipe, as both read "that Factory's
    /// productivity" (user-verified 2026-09-10 beside Eunectes and Passenger).
    RoomPerOperatorGrant { speed_pct: f64, capacity: f64 },

    /// Player-declared, under [`ACCOUNT_FACTS_KEY`]: recruit slots beyond the first.
    AccountFacts { open_recruit_slots: u32 },

    /// Mr. Nothing: each dorm occupant grants `per_occupant`; the SAME buff
    /// pays `pct` per `per` points.
    PoolDormEconomy {
        resource: String,
        per_occupant: f64,
        per: f64,
        pct: f64,
    },

    /// Rosmontis' Extrasensory: dorm occupants feed `gen_resource`, converted
    /// 1:`from_per` into `to_resource`.
    PoolGenerateDormAndConvert {
        gen_resource: String,
        per_occupant: f64,
        to_resource: String,
        from_per: f64,
    },

    /// Every occupant, owner included, drains `delta`/hr more (negative = slower).
    MoraleRoomAura { delta: f64 },

    /// CC aura on dorm sleepers' recovery; non-stacking.
    DormRecoveryAura { rate: f64 },

    /// "other Operators working in the Trading Post have +15% order acquisition
    /// efficiency": value x (occupants - 1) on the room.
    PerTeammateEfficiency { per_teammate_pct: f64 },

    /// Ancient Witchcraft: "every 5 Worldly Plight is converted to 1 Witchcraft
    /// Crystal". The CC WP/PI generators (Chongyue, Dusk/Ling) come through the
    /// side-channel in `pools.rs`, since the CONTROL arm claims their morale aura.
    PoolConvert {
        from: String,
        to: String,
        from_per: f64,
    },

    /// SOLVED pool payoff as productivity (perception-economy seam). Not
    /// `DirectEfficiency`: it rides the ledger's pool-drain channel.
    PoolPayoff { pct: f64 },
}

pub fn build_registry(
    buffs: &HashMap<String, Buff>,
    name_to_char: &HashMap<String, String>,
) -> (
    HashMap<String, BuffResolutionStrategy>,
    HashMap<String, f64>, // buff_id -> morale drain modifier
) {
    let mut registry: HashMap<String, BuffResolutionStrategy> = HashMap::new();
    let mut morale_drains = HashMap::new();
    // "Defaulted trade" threshold from Contract Law ("less than 4"). Without it,
    // payoff skills stay enablers.
    let defaulted_below = defaulted_trade_threshold(buffs);

    for (buff_id, buff) in buffs {
        let prefix = buff_family(buff_id);

        // Drains FIRST, before any branch can `continue` past: the cost rider
        // counts whichever family claims the productivity half.
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

        // Facility-count enablers (Greyy the Lightningbearer E2 "Power Plant +1",
        // Eunectes E2 "+2"), keyed on the exact "(only affects facility
        // quantity/count)" clause, not automation's "...based on facility count".
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
            // An unresolvable named gate makes the modifier unmeetable.
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

        // Layout pool economies (Minimalist's Engineering Robots): "+N per level
        // per building, max CAP" and "for every PER <Resource> present,
        // productivity +PCT%". Keyed on the `$cc.bd_*` term id, not display text.
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
        // A CONTROL "+X% per N <pool>" is a GLOBAL (Ave Mujica's "Plentiful Work
        // Experience"), handled by the CC-global branches.
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
                // Signed: Wulfenite's Go-Getter "+20% and capacity limit -8" nets
                // against her Storage Guru +16 for Vermeil (the game's 93% for
                // Vermeil/Pallas/Wulfenite needs the -8).
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
                // Ramping skills state a higher ceiling in text ("…by 2% per hour, up
                // to a maximum of 30%": Ines, whose `efficiency` is the 20% start), and
                // solo/Clue-Exchange skills leave `efficiency` 0 (Caper). Order:
                // ceiling, `efficiency`, then any plain %.
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
                // Whole-dorm phrasing beats the self heuristic: Durin says "self"
                // but is an aura.
                let aura = RE_DORM_AURA_ALL
                    .captures(&buff.description)
                    .and_then(|c| c[1].parse::<f64>().ok());
                let recovery =
                    aura.unwrap_or_else(|| parse_first_float(&buff.description).unwrap_or(0.0));
                let is_self_only = aura.is_none()
                    && (desc_lower.contains("self")
                        || desc_lower.contains("oneself")
                        || prefix.contains("_oneself"));
                // Audited: 25 single-target vs 39 whole-dorm captures, disjoint.
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
                // FIRST: these texts also hold the plain "all Factories' +X%"
                // the global branches below would take at face value.
                if let Some(branch) = parse_layout_count_branch(&buff.description) {
                    branch
                } else if let Some(gated) = parse_room_presence_gated_global(&buff.description) {
                    gated
                } else if let Some(crew_gated) = parse_crew_tag_global(&buff.description) {
                    crew_gated
                }
                // Before the plain globals for the same reason. Audited 2026-08-13:
                // shape A = exactly control_mp_bd&trade[000], shape B =
                // control_prod_bd_spd[000]/[010].
                else if let Some(c) = RE_GLOBAL_POOL_A.captures(&buff.description) {
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
                    // Per-operator faction globals outside the _fraction/_tag ids
                    // (audited: control_bd_spd's "for each <Blacksteel Worldwide>
                    // Operator assigned to Factories, productivity +5%", and
                    // Flametail's "each <Pinus Sylvestris> Operator assigned to
                    // Factories have +10% ... towards Battle Records", no "for").
                    || (RE_EACH_FACTION.is_match(&buff.description)
                        && (desc_lower.contains("factor") || desc_lower.contains("trading")))
                {
                    let tag = parse_tag_keyword(&buff.description).unwrap_or_default();
                    let bonus = parse_first_pct(&buff.description).unwrap_or(0.0);
                    // Production tag buffs (Viviana: "+7% Knights in Factories")
                    // are per-operator so the optimizer co-schedules the Knights.
                    // Non-production ones ("Elite in Dormitories") stay flat.
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
                    // Only "other buildings" reaches base-wide; `control_mp_cost`
                    // ("all Operators in the Control Center") is CC-only.
                    //
                    // Partner-gated (Mr. Lee's "together with Aak" +0.25, the Amiya
                    // pairs) and self-subject texts (Gladiia's Abyssal-conditional
                    // ±0.5) price 0: a flat parse made them all auras and inflated
                    // the sustain sim. Self DRAINS still go through
                    // `parse_morale_loss_increase`.
                    let partner_gated = RE_CC_WITH.is_match(&buff.description);
                    // "each <faction> Operator increases the Morale of all
                    // Operators in the Control Center by +0.05" (Lungmen Guard,
                    // Ursus students, Kjerag, Alternates, Team Rainbow, Lee's
                    // agency) scales with a seated count nothing resolves yet. Flat,
                    // it credited a crew with none (Lava the Purgatory, no
                    // Alternate, 31010962). 0 until count-scaled.
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
                    // Signed: Gnosis's "-15%" with "+6 order limit" (the only
                    // CONTROL conditional leading with a vdown, audited 2026-09-17).
                    let bonus = parse_first_signed_pct(&buff.description).unwrap_or(0.0);
                    let order_limit = parse_order_limit(&buff.description).unwrap_or(0);
                    // Faction-gated ("all <Siracusa> Operators…", "all Trading Posts
                    // with 3 <Kjerag> Operators…"): never flat. Token from the shown
                    // keyword (Kjerag -> "kjerag", the nation tag), not the group
                    // marker. "with <N>" gates the WHOLE post; otherwise per operator.
                    // Delphine's singular "for each <Glasgow Gang> Operator assigned
                    // to the same Trading Post, +10%" (the only one, audited) used to
                    // fall to TagBased: a flat half-credit that won her a CC seat with
                    // no Glasgow trader, unseen by the dead-weight reselection.
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
                    // Wiš'adel's Conspirator. Must precede the non-production
                    // branch, whose Reception-Room guard leaves these
                    // (`control_meeting&ord` x2) for here.
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
                    // Last, so production branches win. Audited 2026-08-13: captures
                    // upMeetingSpeed x2, meeting_spd&bd, mp&meet_spd (Sakiko gate),
                    // meeting&mp_cost x2, train_spd x3, hire_spd&bd. The Reception
                    // guard leaves meeting&ord x2 to the named-gate parse above.
                    // control_hire_spd's aggregate-state conditional self-excludes
                    // by phrasing.
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
                // Hoederer: "+30%, and +5% more when Ines or W is assigned to any Work
                // Area". "Work Area" separates it from same-room "...same Trading Post
                // as <op>".
                let in_work_area = buff.description.contains("Work Area");
                let in_base_anywhere = buff.description.contains("is in the Base");
                let base_wide = (in_work_area || in_base_anywhere)
                    .then(|| find_all_operator_char_ids(&buff.description, name_to_char))
                    .filter(|(ids, _)| !ids.is_empty());
                // Room-presence gates must precede the base-wide/teammate branches:
                // "is assigned to a <Room>" has no "Work Area"/"same" marker.
                if RE_DRAIN_AURA_IMMUNITY.is_match(&buff.description) {
                    // Waai Fu's Team Spirit. First: no payload for others to misread.
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
                    // Justice Knight's "'Beep beep, activate!'": from a Power
                    // Plant seat, +5% to the Factory Wild Mane works in.
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
                }
                // Named-teammate conditional, both phrasings:
                //   "...same Trading Post as <@cc.kw>Lappland</> … +65%"   (Texas)
                //   "+20%; if <@cc.kw>Exusiai</> … same Trading Post … +25%" (Lemuen)
                // Base = Efficiency field. Faction counts ("for every Glasgow Gang
                // operator…") excluded.
                else if buff.description.contains("same")
                    && !buff.description.contains("for every")
                    && let Some((req_name, name_end)) =
                        find_operator_keyword(&buff.description, name_to_char)
                {
                    let required_char_id = name_to_char.get(&req_name).cloned();
                    let base_efficiency = f64::from(buff.efficiency);
                    // % after the name; base-0 buffs fall back to the first %.
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
                // Faction conditional (Morgan: "if a <Glasgow Gang> Operator…same…").
                // Count-scalers ("for every <faction>…") go to MatchCountScaling.
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
                // Shamare-type. Weedy and Snegurochka fall through to facility
                // scaling (see RE_NULLIFY_SELF_PCT).
                else if let Some(cap) = RE_NULLIFY_SELF_PCT.captures(&buff.description) {
                    BuffResolutionStrategy::NullifyTeammatesSelfScaling {
                        per_teammate_pct: cap[1].parse().unwrap_or(0.0),
                    }
                }
                // Highmore: "all <X> and <Y> skills are considered <Z> skills".
                else if let Some((from_tokens, to_token)) =
                    parse_skill_conversion(&buff.description)
                {
                    BuffResolutionStrategy::SkillTypeConversion {
                        from_tokens,
                        to_token,
                    }
                }
                // "+X% for each <keyword>", keyword a faction or skill type (a number
                // is a resource mechanic, handled elsewhere).
                else if let Some(token) = parse_count_keyword(&buff.description)
                    && plain_text(&buff.description)
                        .to_lowercase()
                        .contains("in the base")
                    // Mantra's "for every facility in the Base with an Elite
                    // Operator assigned" needs seats by slot, which the deployment
                    // map lacks: flat base only, never a guessed count.
                    && !plain_text(&buff.description)
                        .to_lowercase()
                        .contains("every facility")
                {
                    // "for each Rhine Lab Operator in the Base (caps at 5)": holder
                    // included; rate = the % after the count phrase, not the
                    // leading flat %.
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
                    // Morgan "Gang Compass": +35% more "when in the same Trading Post
                    // as Siege". No rider -> (None, 0).
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
                    // "each <X>-type skill" / "per <X> Skill" counts skills by
                    // name; "per <faction> Operator" counts operators by tag.
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
                }
                // Pozëmka: "+5% per Pure Gold Production Line" scales on the gold
                // factories; "for every Durin Operator in the base (caps at 4),
                // another Line" is a base-wide count at that per-line rate, read
                // from the sibling skill's text.
                else if buff.room_type == "TRADING"
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
                    // A TAG ("<$cc.tag.durin>Durin") after "for every 1", which the
                    // keyword parser rejects as numeric.
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
                    // Order VALUE as a SHAPE, priced per post level by `order_mix`.
                    BuffResolutionStrategy::OrderValue { effect, pure_gold }
                }
                // Jaye's Basic Needs (E1, `trade_ord_limit_count`). Id has "_limit":
                // must precede capacity-only. The minimum is on the ROOM total.
                else if prefix.contains("_limit_count") {
                    let pct_per_cut = parse_nth_pct(&buff.description, 0).unwrap_or(10.0);
                    let cut = parse_order_limit(&buff.description).unwrap_or(-1);
                    let per_order_pct = parse_nth_pct(&buff.description, 1).unwrap_or(4.0);
                    BuffResolutionStrategy::LimitCutPerPeerEfficiency {
                        pct_per_cut,
                        cut,
                        per_order_pct,
                    }
                }
                // Jaye's Street Economics (E0). Also "_limit" but an EFFICIENCY
                // skill: must precede capacity-only.
                else if prefix.contains("_limit_diff")
                    || (buff.room_type == "TRADING"
                        && buff.description.contains("order acquisition efficiency")
                        && buff.description.contains("difference"))
                {
                    BuffResolutionStrategy::OrderDifferenceScaling {
                        per_order_pct: parse_first_pct(&buff.description).unwrap_or(4.0),
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
                    // "+X% base, -Y% per Z morale difference"; peak = Efficiency.
                    // Over a shift morale goes 24 -> 0, so the average difference is 12.
                    let peak = f64::from(buff.efficiency);
                    // Parse: "every <@cc.kw>4</> points" -> 4, and "-5%" -> 5
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
                    // Not parse_first_pct: the first % is the [030] tier's 20% base.
                    let per_hr = parse_per_hour_pct(&buff.description).unwrap_or(1.0);
                    let cap = parse_last_pct(&buff.description).unwrap_or(25.0);
                    let ramp_hours = if per_hr > 0.0 {
                        (cap - base) / per_hr
                    } else {
                        24.0
                    };
                    let ramp_duration = ramp_hours.min(24.0);
                    let plateau_duration = (24.0 - ramp_duration).max(0.0);
                    let avg = ((base + cap) / 2.0 * ramp_duration + cap * plateau_duration) / 24.0;
                    BuffResolutionStrategy::MoraleDecayEfficiency {
                        time_averaged_value: avg,
                    }
                }
                // "efficiency +25% and order limit -6". BEFORE the generic
                // efficiency > 0 check.
                else if prefix.starts_with("trade_ord_spd&limit") {
                    let efficiency = f64::from(buff.efficiency);
                    let order_limit = parse_order_limit(&buff.description).unwrap_or(0);
                    BuffResolutionStrategy::EfficiencyWithOrderLimit {
                        efficiency,
                        order_limit,
                    }
                }
                // Degenbrecher: "for every 5 order limit increase... +25%, max +100%"
                else if prefix == "trade_ord_spd_variable3" {
                    let threshold = parse_first_vup_number(&buff.description).unwrap_or(5.0);
                    let bonus = parse_nth_pct(&buff.description, 0).unwrap_or(25.0);
                    let cap = parse_last_pct(&buff.description).unwrap_or(100.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: threshold,
                        bonus_per_threshold: bonus,
                        cap_pct: cap,
                        includes_self: false, // "from teammates" - excludes Degenbrecher's own -6
                        stage: PEER_STAGE_FIXED_LIMIT,
                    }
                }
                // Swire the Elegant Wit: "+4% per order limit increase provided by
                // all other Operators", read AFTER Jaye's cut (in-game: Jaye/Swire/
                // SilverAsh under Gnosis reads 129 = 4% x (10 - 2), not 4% x 10).
                else if prefix == "trade_ord_spd_variable" {
                    let per = parse_first_pct(&buff.description).unwrap_or(4.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: 1.0,
                        bonus_per_threshold: per,
                        cap_pct: f64::MAX,
                        includes_self: false, // "from others"
                        stage: PEER_STAGE_NET_LIMIT,
                    }
                }
                // Vermeil: "+X% productivity per capacity limit increase", over the
                // WHOLE factory's capacity, her own +8 included.
                else if prefix == "manu_prod_spd_variable" {
                    let per = parse_first_pct(&buff.description).unwrap_or(2.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: 1.0,
                        bonus_per_threshold: per,
                        cap_pct: f64::MAX,
                        includes_self: true,
                        stage: PEER_STAGE_FIXED_LIMIT,
                    }
                }
                // Bubble E1 capacity tiers (strong with Vulcan/Ceobe/Wulfenite).
                else if prefix == "manu_prod_spd_variable3" {
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
                }
                // Quartz "Precise Scheduling": base + "+N% per recipe type being
                // processed at Factories", over DISTINCT recipe types
                // (`MANUFACTURE_RECIPE_TYPES`: gold + EXP = 2, not 4 factories).
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
                // Vigil's New City Trade: "+25%, +5% per Reception Room level, up to
                // a maximum of 40%". The ceiling is on the TOTAL, so the scaled cap
                // is ceiling minus base.
                else if prefix.contains("&meet")
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
                } else if buff.efficiency > 0 {
                    BuffResolutionStrategy::DirectEfficiency {
                        value: f64::from(buff.efficiency),
                    }
                }
                // Automation, per Power Plant.
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
                // Snegurochka: per-occupant ROOM grant (+N% top tier, +M capacity).
                else if let Some(c) = RE_NULLIFY_ROOM_PER_OP.captures(&buff.description) {
                    BuffResolutionStrategy::RoomPerOperatorGrant {
                        speed_pct: c
                            .get(1)
                            .and_then(|m| m.as_str().parse().ok())
                            .unwrap_or(0.0),
                        capacity: c[2].parse().unwrap_or(0.0),
                    }
                }
                // Other &manu nullifiers: zero value, never picked for output.
                else if prefix.contains("&manu") {
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
                // Waai Fu, Snowsant: "+5% for every 5% provided by all other
                // Operators assigned to that <room>, up to a maximum of N%". Payout
                // and step are read separately though equal so far.
                else if prefix.contains("_variable2") {
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
                }
                // Unmatched building-resource skills are 0 here: per-unit credit for
                // an unbanked consumable (Monster Meal, Engineering Robots,
                // Witchcraft Crystal) over-ranks them, and the Perception economy
                // is base-wide (the pool settlement prices it). Flat parts live in
                // a separate slot, caught by `efficiency > 0` above.
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
                // Greyy the Lightningbearer: "+1% Drone recovery rate for every 10 max
                // Drone capacity (Max +25%)", on the real `DRONE_CAPACITY` (3x L3
                // plants hold 235 drones: +23.5%).
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
                // Drone skill with its % only in the description.
                else if buff.room_type == "POWER" && buff.description.contains("Drone recovery") {
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

        // "plus an additional +X% for every <res>" wraps the base parse.
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

/// "for each/every <keyword>" -> lowercased leading word ("Rhine Tech-type
/// skill" -> "rhine", "Glasgow Gang Operator" -> "glasgow"). Numeric keywords
/// ("for each 4 gold bars") are resource mechanics: `None`.
fn parse_count_keyword(desc: &str) -> Option<String> {
    let cap = RE_COUNT_KEYWORD.captures(desc)?;
    let token = first_token(&cap[1]);
    (!token.is_empty() && !token.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .then_some(token)
}

/// "all <X> and <Y> skills are considered <Z> skills" -> ([x, y], z).
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

fn parse_scaling_cap(desc: &str) -> Option<f64> {
    let lower = desc.to_lowercase();
    let idx = lower.find("max")?;
    RE_LAST_PCT
        .find_iter(&desc[idx..])
        .next()
        .and_then(|m| RE_LAST_PCT_INNER.captures(m.as_str()))
        .and_then(|c| c[1].parse().ok())
}

/// A description with its `<@cc.kw>…</>` / `<@cc.vup>…</>` markup removed,
/// for shapes whose numbers sit inside keyword spans.
fn plain_text(desc: &str) -> String {
    static RE_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap());
    RE_TAG.replace_all(desc, "").into_owned()
}

/// From the defining text: "if the amount of Pure Gold traded is less than 4, it
/// will be considered a Defaulted trade".
fn defaulted_trade_threshold(buffs: &HashMap<String, Buff>) -> Option<u32> {
    static RE_DEFAULTED_RULE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"Pure Gold traded is less than (\d+), it will be considered a Defaulted trade")
            .unwrap()
    });
    buffs
        .values()
        .filter(|b| b.room_type == "TRADING")
        .find_map(|b| {
            RE_DEFAULTED_RULE
                .captures(&plain_text(&b.description))
                .and_then(|c| c[1].parse().ok())
        })
}

/// `(shape, pure_gold)`. `pure_gold`: Pure-Gold-only value (Proviso), killed by
/// Shamare's shift. Without `defaulted_below` Proviso stays an enabler.
fn order_value_shape(desc: &str, defaulted_below: Option<u32>) -> Option<(OrderEffect, bool)> {
    static RE_HIGH_LMD: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"traded is higher than (\d+)[^.]*?increase the LMD gained by \+(\d+)").unwrap()
    });
    static RE_DEFAULTED_GOLD: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"Defaulted trade, Pure Gold traded \+(\d+)").unwrap());
    let text = plain_text(desc);
    if let Some(c) = RE_HIGH_LMD.captures(&text) {
        let above = c[1].parse().ok()?;
        let lmd = c[2].parse().ok()?;
        Some((OrderEffect::HighOrderLmdBonus { above, lmd }, false))
    } else if let Some(c) = RE_DEFAULTED_GOLD.captures(&text) {
        let bonus = c[1].parse().ok()?;
        let effect = defaulted_below.map_or(OrderEffect::Enabler, |below| {
            OrderEffect::DefaultedGoldBonus { below, bonus }
        });
        Some((effect, true))
    } else if text.contains("higher-yield") {
        // "increased slightly" (α tiers) vs "increased" (β tiers).
        Some((
            OrderEffect::HigherYieldChance {
                strong: !text.contains("slightly"),
            },
            false,
        ))
    } else if text.contains("Pure Gold") || text.contains("Defaulted trade") {
        Some((OrderEffect::Enabler, true)) // Contract Law and kin: rules, no payoff
    } else {
        None
    }
}

/// Leading word of a keyword phrase, lowercased ("Rhine Tech-type" -> "rhine").
fn first_token(s: &str) -> String {
    s.trim()
        .split([' ', '-'])
        .next()
        .unwrap_or("")
        .to_lowercase()
}

/// "assigned together with other <$cc.g.lgd>L.G.D.</> Operators to the Control Center".
static RE_CC_WITH_OTHER_TAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"assigned together with other <\$cc\.(?:g|tag)\.([A-Za-z0-9_]+)>").unwrap()
});

/// Hoshiguma the Breacher's Camaraderie: "together with other L.G.D. Operators to
/// the Control Center, all Factories' productivity +3%". One other is enough
/// ("Operators" names no count). Parsed BEFORE the per-operator faction branch,
/// which misread it as "+3% to each L.G.D. operator in Factories".
fn parse_crew_tag_global(desc: &str) -> Option<BuffResolutionStrategy> {
    let tag = RE_CC_WITH_OTHER_TAG.captures(desc)?[1].to_lowercase();
    let lower = plain_text(desc).to_lowercase();
    if !lower.contains("to the control center") {
        return None;
    }
    let target_room = if lower.contains("all factories") {
        "MANUFACTURE"
    } else if lower.contains("all trading posts") {
        "TRADING"
    } else {
        return None;
    };
    Some(BuffResolutionStrategy::GlobalEffectWithCrewTag {
        target_room: target_room.to_string(),
        bonus_pct: parse_first_pct(desc)?,
        tag,
        min_others: 1,
    })
}

/// `char_id` -> glossary TAGs. `cc.tag.knight` reads "Includes the following
/// operators / Nearl the Radiant Knight, Nearl, Blemishine, ..., Gravel,
/// Viviana"; curated tags have no character field, so this is the only source
/// for "all Knight Operators" (Viviana).
pub fn glossary_tags(
    consts: &GameDataConst,
    name_to_char: &HashMap<String, String>,
) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for entry in &consts.term_description_dict {
        let Some(tag) = entry.key.strip_prefix("cc.tag.") else {
            continue;
        };
        let desc = plain_text(&entry.value.description);
        let Some((_, names)) = desc.split_once("following") else {
            continue;
        };
        for name in names.split([',', '\n']) {
            let name = name
                .trim()
                .trim_matches(|c| c == '\'' || c == '"' || c == ':')
                .trim()
                .to_lowercase();
            if let Some(id) = name_to_char.get(&name) {
                let tags = out.entry(id.clone()).or_default();
                let tag = tag.to_lowercase();
                if !tags.contains(&tag) {
                    tags.push(tag);
                }
            }
        }
    }
    out
}

/// [`faction_tags_of`] plus the glossary tags the operator is listed under.
pub fn faction_tags_for(
    char_id: &str,
    op: &Operator,
    glossary: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let mut tags = faction_tags_of(op);
    for tag in glossary.get(char_id).into_iter().flatten() {
        if !tags.contains(tag) {
            tags.push(tag.clone());
        }
    }
    tags
}

/// "(This effect does not stack with Recycling and takes priority over it)"
/// names a SKILL; resolve it to every buff of that name (tiers share the id
/// prefix). Unresolved -> empty, nothing excluded.
fn non_stacking_priority_families(desc: &str, buffs: &HashMap<String, Buff>) -> Vec<String> {
    let plain = plain_text(desc);
    let Some(start) = plain.find("does not stack with ") else {
        return Vec::new();
    };
    let rest = &plain[start + "does not stack with ".len()..];
    let end = rest
        .find(" and takes priority")
        .or_else(|| rest.find(')'))
        .unwrap_or(rest.len());
    let name = rest[..end].trim();
    if name.is_empty() {
        return Vec::new();
    }
    let mut families: Vec<String> = buffs
        .iter()
        .filter(|(_, b)| b.buff_name.eq_ignore_ascii_case(name))
        .map(|(id, _)| buff_family(id).to_string())
        .collect();
    families.sort();
    families.dedup();
    families
}

fn parse_first_pct(desc: &str) -> Option<f64> {
    RE_FIRST_PCT.captures(desc).and_then(|c| c[1].parse().ok())
}

/// Sustained ceiling of a ramping reception skill ("…then by 2% per hour, up to a
/// maximum of 30%").
fn parse_reception_ceiling(desc: &str) -> Option<f64> {
    // ASCII needle, so the lowercased index is valid in the original.
    let idx = desc.to_lowercase().find("maximum of")?;
    parse_first_pct_from(desc, idx)
}

/// First `<@cc.vup>+N%` at or after byte offset `start`.
fn parse_first_pct_from(desc: &str, start: usize) -> Option<f64> {
    desc.get(start..).and_then(parse_first_pct)
}

/// First keyword naming a known operator (nested `<$cc.x>` stripped): lowercased
/// name and the byte offset past the block. Both orders: "...same Trading Post
/// as <@cc.kw>Lappland</>" and "if <@cc.kw>Exusiai</> is assigned to the same
/// Trading Post".
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

/// Every keyword naming a known operator ("when <Ines> or <W> are assigned…"),
/// plus the byte offset past the LAST one.
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

/// Buff-text room label -> room type. Gamedata has no such table.
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

/// "if <NAME> is assigned to <room>, <payload>" segments -> grants. None when no
/// segment has both a resolvable gate and a priced payload (falls through).
/// Unresolved names are dropped.
fn parse_named_char_room_grants(
    desc: &str,
    name_to_char: &HashMap<String, String>,
) -> Option<Vec<NamedCharGrant>> {
    let grants: Vec<NamedCharGrant> = RE_CC_NAMED_GATE
        .captures_iter(desc)
        .filter_map(|c| {
            let char_id = name_to_char.get(&c[1].to_lowercase())?.clone();
            let target_room = room_type_from_label(&c[2])?;
            let payload = &c[3];
            let order_limit = RE_NAMED_GATE_ORD
                .captures(payload)
                .and_then(|o| o[1].parse::<f64>().ok())
                .unwrap_or(0.0);
            let nonprod_pct = RE_CC_CLUE
                .captures(payload)
                .and_then(|n| n[1].parse::<f64>().ok())
                .unwrap_or(0.0);
            (order_limit != 0.0 || nonprod_pct != 0.0).then(|| NamedCharGrant {
                char_id,
                target_room: target_room.to_string(),
                order_limit,
                nonprod_pct,
            })
        })
        .collect();
    (!grants.is_empty()).then_some(grants)
}

/// Layout-counted pool resources from the term glossary: "For every Trading
/// Post and Power Plant, Influence +1" -> `bd_wang_1: [TRADING, POWER]`.
/// Keyed by the resource id as it appears inside buff markup (`bd_*`).
pub fn layout_term_rooms(
    consts: &crate::core::gamedata::types::consts::GameDataConst,
) -> HashMap<String, Vec<String>> {
    static RE_FOR_EVERY: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"For every ([A-Za-z ,]+?), [A-Za-z ]+ \+1").unwrap());
    consts
        .term_description_dict
        .iter()
        .filter_map(|e| {
            let id = e.value.term_id.strip_prefix("cc.")?.to_string();
            let c = RE_FOR_EVERY.captures(&e.value.description)?;
            let rooms: Vec<String> = c[1]
                .split(" and ")
                .flat_map(|part| part.split(", "))
                .filter_map(|label| room_type_from_label(label.trim()))
                .map(str::to_string)
                .collect();
            (!rooms.is_empty()).then_some((id, rooms))
        })
        .collect()
}

/// Wang's Expedience: "if <A> is greater than or equal to <B>, all <room> +X%; if
/// <B> is greater than <A>, all <room> +Y%". A and B map to `$cc.bd_*` ids by
/// order of appearance.
fn parse_layout_count_branch(desc: &str) -> Option<BuffResolutionStrategy> {
    static RE_BRANCH: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"if ([A-Za-z]+) is greater than or equal to ([A-Za-z]+), all (Trading Posts|Factories)' (?:order efficiency|productivity) \+([\d.]+)%; if ([A-Za-z]+) is greater than ([A-Za-z]+), all (Trading Posts|Factories)' (?:order efficiency|productivity) \+([\d.]+)%",
        )
        .unwrap()
    });
    static RE_TERM_NAME: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"<\$cc\.(bd_[a-z0-9_]+)><@cc\.kw>([^<]+)</>").unwrap());
    let text = plain_text(desc);
    let c = RE_BRANCH.captures(&text)?;
    if c[1] != c[6] || c[2] != c[5] {
        return None;
    }
    let ids: HashMap<String, String> = RE_TERM_NAME
        .captures_iter(desc)
        .map(|m| (m[2].to_string(), m[1].to_string()))
        .collect();
    let a_term = ids.get(&c[1])?.clone();
    let b_term = ids.get(&c[2])?.clone();
    let room_of = |label: &str| room_type_from_global_label(label).to_string();
    Some(BuffResolutionStrategy::LayoutCountBranch {
        a_term,
        b_term,
        ge_room: room_of(&c[3]),
        ge_pct: c[4].parse().ok()?,
        gt_room: room_of(&c[7]),
        gt_pct: c[8].parse().ok()?,
    })
}

/// Pudding's Overclock: "if there are N or more Operation Platforms assigned to
/// <room>s, all <room> +X%". Operation Platforms = `robot` tag.
fn parse_room_presence_gated_global(desc: &str) -> Option<BuffResolutionStrategy> {
    static RE_GATE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"if there are (\d+) or more Operation Platforms assigned to ([A-Za-z ]+?)s?, all (Trading Posts|Factories)' (?:order efficiency|productivity) \+([\d.]+)%",
        )
        .unwrap()
    });
    let text = plain_text(desc);
    let c = RE_GATE.captures(&text)?;
    Some(BuffResolutionStrategy::RoomPresenceGatedGlobal {
        required_faction: "robot".to_string(),
        required_count: c[1].parse().ok()?,
        room_type: room_type_from_label(&c[2])?.to_string(),
        target_room: room_type_from_global_label(&c[3]).to_string(),
        bonus_pct: c[4].parse().ok()?,
    })
}

/// "if <op> is assigned to the <Room>", "and <op> is in a <Room>", "if another
/// <faction> Operator is assigned to a <Room>". Audited 2026-08-13 in the
/// production arms: named = exactly `manu_formula_spd_P[000]` (Gummy, Trading
/// Post) and `power_rec_spd_P[000]/[001]` (Kal'tsit in CC, Logos as Trainer);
/// faction = exactly `power_rec_spd_ext&faction[000]` (Laterano, Power Plant).
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
        // Unresolved name -> empty list: base only.
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
            // "another": owner + another.
            required_count: 2,
            room_type: room_type.to_string(),
            base_efficiency,
            bonus_efficiency: parse_first_pct_from(desc, end).unwrap_or(0.0),
        });
    }
    None
}

/// First faction marker (`<$cc.g.glasgow>` etc.) and the byte offset past it.
fn find_faction_token(desc: &str) -> Option<(String, usize)> {
    let m = RE_FACTION_TOKEN.captures(desc)?;
    let token = m[1].to_lowercase();
    Some((token, m.get(0)?.end()))
}

/// The `(percent, units)` of a per-capacity rate: "+1% Drone recovery rate for every
/// 10 max Drone capacity" -> `(1.0, 10.0)`.
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

fn parse_first_float(desc: &str) -> Option<f64> {
    RE_FIRST_FLOAT
        .captures(desc)
        .and_then(|c| c[1].parse().ok())
}

/// Prefers the figure tied to "Morale" ("recover <@cc.vup>+0.05</> Morale per
/// hour"). If the only number is a `<$cc.bd_…>` resource grant (Worldly Plight),
/// no morale. Else the first `<@cc.vup>` figure.
fn parse_morale_recovery(desc: &str) -> Option<f64> {
    if let Some(c) = RE_MORALE_RECOVERY.captures(desc) {
        return c[1].parse().ok();
    }
    if desc.contains("<$cc.bd_") {
        return Some(0.0);
    }
    parse_first_float(desc)
}

/// Keyword -> faction tag. "L.G.D." collapses to "lgd" (how the group is tagged;
/// a word match dropped it and credited the conditional unconditionally).
/// Otherwise the leading word ("Blacksteel Worldwide" -> "blacksteel").
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

fn parse_last_pct(desc: &str) -> Option<f64> {
    RE_LAST_PCT.find_iter(desc).last().and_then(|m| {
        RE_LAST_PCT_INNER
            .captures(m.as_str())
            .and_then(|c| c[1].parse().ok())
    })
}

fn parse_kw_number(desc: &str) -> Option<f64> {
    RE_KW_NUMBER.captures(desc).and_then(|c| c[1].parse().ok())
}

/// Parse first negative percentage from <@cc.vdown>-5%</> or <@cc.vdown>+0.25</>
fn parse_first_vdown_pct(desc: &str) -> Option<f64> {
    RE_VDOWN_PCT.captures(desc).and_then(|c| c[1].parse().ok())
}

fn parse_per_hour_pct(desc: &str) -> Option<f64> {
    RE_PER_HOUR_PCT.captures(desc).and_then(|c| {
        c.get(1)
            .or_else(|| c.get(2))
            .and_then(|m| m.as_str().parse().ok())
    })
}

fn parse_first_signed_pct(desc: &str) -> Option<f64> {
    RE_FIRST_SIGNED_PCT
        .captures(desc)
        .and_then(|c| c[1].parse().ok())
}

fn parse_order_limit(desc: &str) -> Option<i32> {
    if let Some(cap) = RE_ORDER_LIMIT_POS.captures(desc) {
        return cap[1].parse::<i32>().ok();
    }
    if let Some(cap) = RE_ORDER_LIMIT_NEG.captures(desc) {
        return cap[1].parse::<i32>().ok().map(|v| -v);
    }
    None
}

/// The Nth `<@cc.vup>` percentage, 0-indexed
fn parse_nth_pct(desc: &str, n: usize) -> Option<f64> {
    RE_NTH_PCT.find_iter(desc).nth(n).and_then(|m| {
        RE_LAST_PCT_INNER
            .captures(m.as_str())
            .and_then(|c| c[1].parse().ok())
    })
}

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

/// Morale effect on a DIFFERENT co-seated operator: Ave Mujica riders ("Morale
/// consumed per hour by Sakiko Togawa +0.1" in the same CC), Mortis' amnesty
/// ("ignores the self Morale loss effect from her own base skill"), Nian's
/// faction version ("remove any Morale reduction effects from Sui Operators ...
/// that affect themselves").
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
    /// Ignores every morale effect TEAMMATES project, either direction (Waai Fu).
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

/// Lets the unresolved inventory drop the marker on a buff this side-channel prices.
pub fn has_targeted_morale_effect(desc: &str) -> bool {
    RE_TARGETED_RIDER.is_match(desc)
        || RE_NEGATES_OWN.is_match(desc)
        || RE_NEGATES_FACTION.is_match(desc)
        || RE_AURA_IMMUNITY.is_match(desc)
        || RE_FORMULA_DRAIN.is_match(desc)
}

/// Audited 2026-08-13: riders = `control_mp&meet_spd`[000] (+0.05 on Sakiko) and
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
            // Targets entry (F_EXP for Battle Records), no name mapping.
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

/// Lin's Meritocracy: "+10% HR contacting speed for every Recruit slot other than
/// the initial slot". The sync can't read slots: 0 until the player declares them.
static RE_HR_PER_SLOT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:<@cc\.vup>)?\+([\d.]+)%(?:</>)? HR contacting speed for every Recruit slot")
        .unwrap()
});

/// Re-priced registry copy for player-declared facts; today `open_recruit_slots`
/// (0-3 beyond the first). Audited 2026-08-23: `RE_HR_PER_SLOT` captures exactly
/// `hire_spd_cost&extra[000]` (Lin, 0 + 10/slot); other slot-gated riders (clue
/// odds, drains, resource points) aren't priced value and stay untouched.
pub fn resolve_account_facts(
    registry: &HashMap<String, BuffResolutionStrategy>,
    buffs: &HashMap<String, Buff>,
    open_recruit_slots: u32,
) -> HashMap<String, BuffResolutionStrategy> {
    let mut out = registry.clone();
    out.insert(
        ACCOUNT_FACTS_KEY.to_string(),
        BuffResolutionStrategy::AccountFacts { open_recruit_slots },
    );
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

/// "self Morale loss per hour <@cc.vdown>+N</>". Companion-gated forms ("when
/// assigned together with <op>, ... Morale loss +N") are SKIPPED: flat, they
/// fabricate depletion warnings. Audited 2026-08-13: exactly `control_bd_spd`[000],
/// `control_mp_cost&bd_up`[000] (Chongyue +0.5), `control_mp_cost&bd2`[010]
/// (+0.5), `control_mp_cost&bd3`[000] (Sakiko +0.05; its trailing "when Passion
/// is 40 or higher" holds at any committed plan's steady state).
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
