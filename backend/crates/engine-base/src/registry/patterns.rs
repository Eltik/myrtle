use std::sync::LazyLock;

use regex::Regex;

pub(super) static RE_FIRST_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").expect("valid regex"));

pub(super) static RE_FIRST_SIGNED_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.(?:vup|vdown)>([+-]?[\d.]+)%</>").expect("valid regex"));

pub(super) static RE_FIRST_FLOAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)</>").expect("valid regex"));

pub(super) static RE_MORALE_RECOVERY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)</>\s*Morale").expect("valid regex"));

pub(super) static RE_TAG_KEYWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.kw>([^<]+)</>").expect("valid regex"));

pub(super) static RE_LAST_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").expect("valid regex"));

pub(super) static RE_LAST_PCT_INNER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([\d.]+)%").expect("valid regex"));

pub(super) static RE_KW_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.kw>(\d+)</>").expect("valid regex"));

pub(super) static RE_VDOWN_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vdown>[+-]?([\d.]+)%?</>").expect("valid regex"));

pub(super) static RE_POOL_GEN_LEVELS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"gain <@cc\.vup>\+?([\d.]+)</>\s*<@cc\.kw><\$cc\.([a-z0-9_]+)>[^<]*</></>\s*per level per building.*max <@cc\.vup>([\d.]+)</>",
    )
    .expect("valid regex")
});

pub(super) static RE_POOL_CONSUME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.vup>\+?([\d.]+)</>\s*<@cc\.kw><\$cc\.([a-z0-9_]+)>[^<]*</></>\s*present, productivity <@cc\.vup>\+([\d.]+)%</>",
    )
    .expect("valid regex")
});

pub(super) static RE_POOL_CONSUME_REV: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"<@cc\.vup>\+([\d.]+)%</>[^<]{0,30}for every <@cc\.vup>([\d.]+)</>\s*<\$cc\.([a-z0-9_]+)>",
    )
    .expect("valid regex")
});

pub(super) static RE_POOL_GEN_OWN_OCC: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.vup>1</>\s*Operators? in that (?:Dormitory|Factory|Trading Post|Power Plant|Reception Room|Office|Workshop|Training Room),\s*<\$cc\.([A-Za-z0-9_]+)>[^+]{0,40}?<@cc\.vup>\+([\d.]+)</>",
    )
    .expect("valid regex")
});

pub(super) static RE_POOL_GEN_OWN_ROOM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"provide <@cc\.vup>([\d.]+)</>\s*<\$cc\.([A-Za-z0-9_]+)>.{0,60}for every level of the current",
    )
    .expect("valid regex")
});

pub(super) static RE_POOL_CONSUME_POINTS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.vup>([\d.]+)</>\s*(?:points? of )?<\$cc\.([A-Za-z0-9_]+)>.{0,60}?[Pp]roductivity <@cc\.vup>\+([\d.]+)%</>",
    )
    .expect("valid regex")
});

pub(super) static RE_POOL_DORM_ECONOMY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"Operator in the Dormitory grants\s*<\$cc\.([A-Za-z0-9_]+)>[^%]{0,40}?<@cc\.vup>\+([\d.]+)</>, and for every <@cc\.vup>([\d.]+)</>\s*points?<\$cc\.[A-Za-z0-9_]+>.{0,40}?grant <@cc\.vup>\+([\d.]+)%</> order",
    )
    .expect("valid regex")
});

pub(super) static RE_POOL_GEN_DORM_CONVERT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.(?:kw|vup)>1</>\s*[Oo]perators? in the Dormitor(?:y|ies),\s*<\$cc\.([A-Za-z0-9_]+)>[^+]{0,40}?<@cc\.vup>\+([\d.]+)</>.{0,30}?every <@cc\.vup>([\d.]+)</>\s*(?:points? of )?<\$cc\.[A-Za-z0-9_]+>.{0,60}?converted (?:in)?to <@cc\.vup>1</>\s*(?:points? of )?<\$cc\.([A-Za-z0-9_]+)>",
    )
    .expect("valid regex")
});

pub(super) static RE_POOL_CONSUME_ROBOTS_POWER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"productivity <@cc\.vup>\+([\d.]+)%</> for <@cc\.vup>every</>\s*<\$cc\.tag\.op>.{0,60}?assigned to a Power Plant",
    )
    .expect("valid regex")
});

pub(super) static RE_MORALE_ROOM_AURA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"Morale (?:consumed per hour of all Operators|loss of Operators) in the [A-Za-z ]{3,20}?\s*<@cc\.vup>(-?[\d.]+)</>",
    )
    .expect("valid regex")
});

pub(super) static RE_POOL_TAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:plus an )?additional (?:Productivity|order acquisition efficiency) <@cc\.vup>\+([\d.]+)%</> for every (?:<@cc\.vup>([\d.]+)</>\s*)?<\$cc\.(bd_[A-Za-z0-9_]+)>",
    )
    .expect("valid regex")
});

pub(super) static RE_EACH_FACTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:<@cc\.vup>)?(?:each|every)(?:</>)?\s*<\$cc\.(?:g|tag)\.").expect("valid regex")
});

pub(super) static RE_GLOBAL_POOL_A: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.vup>([\d.]+)</>\s*<\$cc\.(bd_[A-Za-z0-9_]+)>.{0,120}?all (Trading Posts|Factories)'? (?:order )?(?:efficiency|productivity)?\s*<@cc\.vup>\+([\d.]+)%</>",
    )
    .expect("valid regex")
});
pub(super) static RE_GLOBAL_POOL_B: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"of all (Factories|Trading Posts) <@cc\.vup>\+([\d.]+)%</>, with an additional <@cc\.vup>\+([\d.]+)%</> for every <@cc\.vup>([\d.]+)</>\s*<\$cc\.(bd_[A-Za-z0-9_]+)>",
    )
    .expect("valid regex")
});

pub(super) static RE_FACILITY_GATE_CHAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"if <@cc\.kw>([^<]+)</> is assigned to (?:a|an|the) <@cc\.kw>([A-Za-z ]+?)</>")
        .expect("valid regex")
});

pub(super) static RE_TARGET_ROOM_BOOST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"increases the productivity of the (Factory|Trading Post) <@cc\.kw>([^<]+)</> is assigned to by <@cc\.vup>\+([\d.]+)%</>",
    )
    .expect("valid regex")
});

pub(super) static RE_TAG_MARKUP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<\$cc\.tag\.([a-z0-9_]+)>").expect("valid regex"));

pub(super) static RE_CAPS_AT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"caps at <@cc\.kw>(\d+)</>").expect("valid regex"));

/// Captures (name, room label, payload segment)
pub(super) static RE_CC_NAMED_GATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"if <@cc\.kw>([^<]+)</> is assigned to (?:the |a |an )?([A-Za-z' ]+?)\s*,\s*([^;]*)",
    )
    .expect("valid regex")
});

pub(super) static RE_NAMED_GATE_ORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"order limit <@cc\.vup>\+(\d+)</>").expect("valid regex"));

pub(super) static RE_DRAIN_AURA_IMMUNITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"ignore</> the effects of any Operators stationed in (?:that|the) [A-Za-z ]+ that would affect the Morale consumption of <@cc\.kw>this Operator",
    )
    .expect("valid regex")
});

/// Non-production CC effects, each in its facility's units
pub(super) static RE_CC_CLUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"clue collection speed <@cc\.vup>\+([\d.]+)%</>").expect("valid regex")
});
pub(super) static RE_CC_TRAIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Specialization training speed <@cc\.vup>\+([\d.]+)%</>").expect("valid regex")
});
pub(super) static RE_CC_HIRE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"HR contacting speed <@cc\.vup>\+([\d.]+)%</>").expect("valid regex")
});
pub(super) static RE_CC_EACH_FACTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)each (?:operator from )?<\$cc\.g\.[a-z0-9]+>").expect("valid regex")
});

pub(super) static RE_CC_WITH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"assigned (?:to the Control Center with|together with) <@cc\.kw>([^<]+)</>")
        .expect("valid regex")
});

pub(super) static RE_COUNT_SKILLS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:for each|for every|per) [A-Za-z' -]+?[Ss]kills?\b").expect("valid regex")
});

pub(super) static RE_TOWARDS_PRODUCT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<@cc\.v(?:up|down)>([+-]?[\d.]+)%</> productivity towards <@cc\.kw>([^<]+)</>")
        .expect("valid regex")
});

pub(super) static RE_DORM_SINGLE_TARGET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"to an(?:other)? Operator (?:assigned to|in) (?:that|the) Dorm(?:itory)?")
        .expect("valid regex")
});

pub(super) static RE_DORM_AURA_ALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"restores <@cc\.vup>\+([\d.]+)</> Morale per hour to all (?:other )?Operators")
        .expect("valid regex")
});

pub(super) static RE_DORM_RECOVERY_AURA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"all Operators in Dormitories recover <@cc\.vup>\+([\d.]+)</>\s*Morale per hour")
        .expect("valid regex")
});

pub(super) static RE_PER_TEAMMATE_EFF: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"other Operators (?:working )?in the [A-Za-z ]{3,20} have <@cc\.vup>\+([\d.]+)%</>")
        .expect("valid regex")
});

pub(super) static RE_POOL_CONSUME_ORDER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every <@cc\.vup>([\d.]+)</>\s*<\$cc\.([A-Za-z0-9_]+)>.{0,60}?,\s*<@cc\.vup>\+([\d.]+)%</> order (?:acquisition )?efficiency",
    )
    .expect("valid regex")
});

pub(super) static RE_SPEED_CAPACITY_TRADE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"[Pp]roductivity <@cc\.(vup|vdown)>([+-]?[\d.]+)%</>(?:,| and)\s*capacity limit <@cc\.(vup|vdown)>([+-]?[\d.]+)</>",
    )
    .expect("valid regex")
});

pub(super) static RE_POOL_CONVERT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:convert )?every <@cc\.vup>([\d.]+)</>\s*(?:levels? of |points? of )?<\$cc\.([A-Za-z0-9_]+)>.{0,50}?(?:is converted (?:in)?to|to) <@cc\.vup>1</>\s*(?:levels? of |points? of )?<\$cc\.([A-Za-z0-9_]+)>",
    )
    .expect("valid regex")
});

pub(super) static RE_PER_HOUR_PCT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:<@cc\.vup>\+?([\d.]+)%?</>\s*per hour|per hour <@cc\.vup>\+?([\d.]+)%?</>)")
        .expect("valid regex")
});

pub(super) static RE_NULLIFY_ROOM_PER_OP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"every Operator in that Factory increases that Factory's (?:Productivity by <@cc\.vup>\+([\d.]+)%</> and )?Capacity limit by <@cc\.vup>\+([\d.]+)</>",
    )
    .expect("valid regex")
});

pub static RE_SLOT_GRANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"for every Recruit slot \(Default slots do not count\),\s*<*<\$cc\.(bd_[A-Za-z0-9_]+)>[^+]{0,40}?<@cc\.vup>\+([\d.]+)</>",
    )
    .expect("valid regex")
});

pub(super) static RE_ORDER_LIMIT_POS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:order|capacity) limit(?: is increased by)?\s*<@cc\.vup>\+?(\d+)</>")
        .expect("valid regex")
});

pub(super) static RE_ORDER_LIMIT_NEG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:order|capacity) limit(?: is (?:reduced|decreased) by| by)?\s*<@cc\.vdown>-?(\d+)</>",
    )
    .expect("valid regex")
});

pub(super) static RE_NTH_PCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").expect("valid regex"));

pub(super) static RE_VUP_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.vup>(\d+)</>").expect("valid regex"));

pub(super) static RE_COUNT_KEYWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"for (?:each|every).*?<@cc\.kw>([^<]+)</>").expect("valid regex"));

pub(super) static RE_KW_ANY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.kw>([^<]+)</>").expect("valid regex"));

pub(super) static RE_KW_BLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.kw>(.*?)</>").expect("valid regex"));
pub(super) static RE_INNER_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[^>]*>").expect("valid regex"));

pub(super) static RE_FACTION_TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<\$cc\.[gnt]\.(\w+)>").expect("valid regex"));

pub(super) static RE_NULLIFY_SELF_PCT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"each Operator increases.*?<@cc\.vup>\+?([\d.]+)%").expect("valid regex")
});

pub(super) static RE_MORALE_INCREASE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Morale consumed (?:per|each) hour\s*(?:by\s*)?<@cc\.vdown>\+?([\d.]+)</>")
        .expect("valid regex")
});

pub(super) static RE_MORALE_DECREASE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Morale consumed (?:per|each) hour\s*(?:by\s*)?<@cc\.vup>-?([\d.]+)</>")
        .expect("valid regex")
});

pub(super) static RE_CC_WITH_OTHER_TAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"assigned together with other <\$cc\.(?:g|tag)\.([A-Za-z0-9_]+)>")
        .expect("valid regex")
});
