//! Who a talent reaches and when, read off its English description.
//!
//! TEXT HEURISTIC, run ONCE per load over the English (EN) tables by `index`,
//! never per request. The talent tables carry no machine-readable target for
//! most squad effects (`blackboard` holds the numbers, not who gets them), so
//! the target and the condition are parsed from the plain description. The
//! result is keyed by stable ids (`index::TextKey`) and applied to whichever
//! server the reader picked, so a CN reader gets EN's reading of the same
//! talent; the targets carry ids (profession code, `handbook_team_table`
//! power id, `GachaTags` tag id), never words, so they fit on any server.
//!
//! The vocabulary that is not words from the data is in this file:
//!
//! * targets, tried in this order: an SP recovery type ("Offensive Recovery",
//!   "Defensive Recovery", "Auto Recovery"); a DP cost bound ("DP Cost no more
//!   than N", "cost N or less DP"); "the next Operator"; a class word (the EN
//!   `GachaTags` names of the profession tags, singular or plural, bracketed or
//!   not); a bracketed name (a recruitment tag name is a Tag, else a faction);
//!   "all X Operators" where X is a `handbook_team_table` name; the Melee and
//!   Ranged tag names; an everyone phrase (`EVERYONE`). A bracket followed by
//!   "enemies" names enemies and is skipped.
//! * conditions: `SQUAD_WORDS` -> Squad, `NEAR_WORDS` -> Near, `SKILL_WORDS` ->
//!   Skill, anything else (when deployed, on the field, no condition) -> Field.
//! * counts: "at least N (more)", "N or more", "there are N".
//! * "only the highest" marks an SP aura that does not stack.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::gacha::PROFESSION_TAG_IDS;
use crate::core::gamedata::types::operator::{Operator, OperatorPosition};

use super::build::Seat;
use super::text::{any_of, lower_plain, plain};
use super::types::{SynergyCond, SynergyTargetKind};

/// Phrases that name every ally. Lowercase.
const EVERYONE: &[&str] = &[
    "all allied units",
    "all allies",
    "all friendly units",
    "all operators",
    "allied operators",
    "friendly units",
    "all units",
    "allied units",
    "nearby allies",
    "surrounding allies",
    "allies within",
    "friendly unit",
];
const SQUAD_WORDS: &[&str] = &["in the squad", "in squad"];
const NEAR_WORDS: &[&str] = &[
    "within range",
    "within attack range",
    "surrounding",
    "adjacent",
    "nearby",
    "tiles",
    "in front of",
    "tile ahead",
];
const SKILL_WORDS: &[&str] = &[
    "skill is active",
    "while this unit's skill",
    "during the skill",
];

/// The recruitment tags that name a position, by `GachaTags` `TagId`. The
/// pairing of tag id to `position` is not in any table (the same residue as
/// `gacha::PROFESSION_TAG_IDS`); the words are each server's `TagName`.
pub const POSITION_TAG_IDS: [(&str, i32); 2] = [("MELEE", 9), ("RANGED", 10)];

/// One allied unit the game picks (Haruka "Fleeting Foam": "an allied unit
/// within range gains a bubble"), only with a granting verb after it.
static RANDOM_UNIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"an allied unit within range (?:gains|receives?)").expect("random unit regex")
});
static BRACKET_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]").expect("bracket regex"));
static ALL_X_OPS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[Aa]ll ([A-Z][A-Za-z'’ -]{2,40}?) Operators").expect("all-x regex")
});
static COST_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)DP Cost (?:of )?(?:no more than|not exceeding|at most) (\d+)|cost (\d+) or less DP|DP Cost (?:of )?(\d+) or (?:less|lower|below)|(\d+) DP or less",
    )
    .expect("cost regex")
});
static NEED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)at least (\d+|one|two|three|four|five|six)( more)?|(\d+|two|three|four|five|six) or more|there are (\d+|two|three|four|five|six) ",
    )
    .expect("need regex")
});

fn word_num(s: &str) -> Option<i32> {
    match s.to_lowercase().as_str() {
        "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        n => n.parse().ok(),
    }
}

/// Lowercase letters and digits only, in any script, the form faction names
/// are compared in.
pub fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Two normalized faction names name the same faction: equal, one a prefix of
/// the other ("abyssalhunter" / "abyssalhunters"), or the same first 12
/// characters ("Ursus Student Self-government Group" vs "...Self-Governing
/// Group", worded differently between a talent and the team table).
pub fn same_faction(a: &str, b: &str) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let head = |s: &str| s.chars().take(12).collect::<String>();
    a == b
        || a.starts_with(b)
        || b.starts_with(a)
        || (a.chars().count() >= 12 && b.chars().count() >= 12 && head(a) == head(b))
}

/// The words of one server's tables the classifier reads, built at load.
pub struct Vocab {
    /// Class word -> profession code, from the profession tags' names.
    classes: Vec<(String, String)>,
    class_re: Option<Regex>,
    /// Lowercase recruitment tag name -> `TagId`.
    tag_ids: HashMap<String, i32>,
    /// Position tag name -> `MELEE` / `RANGED`.
    positions: Vec<(String, &'static str)>,
    position_re: Option<Regex>,
    /// Normalized `handbook_team_table` name -> power id.
    factions: Vec<(String, String)>,
}

fn word_alternation(words: &[String]) -> String {
    let mut sorted: Vec<&String> = words.iter().collect();
    // Longest first, so a word that prefixes another never wins.
    sorted.sort_by_key(|w| std::cmp::Reverse(w.len()));
    sorted
        .iter()
        .map(|w| regex::escape(w))
        .collect::<Vec<_>>()
        .join("|")
}

impl Vocab {
    pub fn from_data(data: &GameData) -> Self {
        let tag_name = |id: i32| {
            data.gacha
                .gacha_tags
                .iter()
                .find(|t| t.tag_id == id)
                .map(|t| t.tag_name.clone())
                .filter(|n| !n.is_empty())
        };
        let classes: Vec<(String, String)> = PROFESSION_TAG_IDS
            .iter()
            .filter_map(|(code, id)| Some((tag_name(*id)?, (*code).to_string())))
            .collect();
        let class_words: Vec<String> = classes.iter().map(|(w, _)| w.clone()).collect();
        let class_re = (!class_words.is_empty())
            .then(|| Regex::new(&format!(r"\b({})s?\b", word_alternation(&class_words))).ok())
            .flatten();
        let positions: Vec<(String, &'static str)> = POSITION_TAG_IDS
            .iter()
            .filter_map(|(code, id)| Some((tag_name(*id)?, *code)))
            .collect();
        let position_words: Vec<String> = positions.iter().map(|(w, _)| w.clone()).collect();
        let position_re = (!position_words.is_empty())
            .then(|| {
                Regex::new(&format!(
                    r"\b({}) (?:Operators|allies|units|Operator|ally|unit)\b",
                    word_alternation(&position_words)
                ))
                .ok()
            })
            .flatten();
        let mut factions: Vec<(String, String)> = data
            .factions
            .values()
            .map(|t| (norm(&t.power_name), t.power_id.clone()))
            .filter(|(n, id)| !n.is_empty() && !id.is_empty())
            .collect();
        factions.sort();
        factions.dedup();
        Self {
            classes,
            class_re,
            tag_ids: data
                .gacha
                .gacha_tags
                .iter()
                .filter(|t| !t.tag_name.is_empty())
                .map(|t| (t.tag_name.to_lowercase(), t.tag_id))
                .collect(),
            positions,
            position_re,
            factions,
        }
    }

    fn class_word(&self, word: &str) -> Option<&str> {
        let w = word.trim();
        let w = w.strip_suffix('s').filter(|s| !s.is_empty()).unwrap_or(w);
        self.classes
            .iter()
            .find(|(n, _)| n == w || n == word.trim())
            .map(|(_, p)| p.as_str())
    }

    /// Every power id whose table name is the same faction as `name`, exact
    /// matches first.
    fn powers(&self, name: &str) -> Vec<String> {
        let n = norm(name);
        let mut exact: Vec<String> = self
            .factions
            .iter()
            .filter(|(f, _)| *f == n)
            .map(|(_, id)| id.clone())
            .collect();
        for (f, id) in &self.factions {
            if same_faction(f, &n) && !exact.contains(id) {
                exact.push(id.clone());
            }
        }
        exact
    }
}

/// A parsed target. Every value is an id, so it fits on any server.
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub kind: SynergyTargetKind,
    /// Class: the profession code. Faction: the best-matching power id. Tag:
    /// the `TagId`. SpType: the SP type. Position: `MELEE`/`RANGED`.
    /// CostAtMost: the bound. Otherwise `None`.
    pub value: Option<String>,
    /// Faction: every power id the name matches, which `fits` reads.
    pub powers: Vec<String>,
    /// A class restriction that rides along with an SP-type target ("Sniper
    /// allies that have an Offensive Recovery skill"), as a profession code.
    pub class: Option<String>,
    /// The game picks the receiver ("a random ally"), so the page cannot name
    /// one: the item is a Check with reason Unseen and reaches no seat.
    pub unseen: bool,
    /// The target as the English talent words it.
    pub label: String,
}

impl Target {
    fn new(kind: SynergyTargetKind, value: Option<String>, label: String) -> Self {
        Self {
            kind,
            value,
            powers: Vec::new(),
            class: None,
            unseen: false,
            label,
        }
    }
}

/// Phrases for one ally the game picks itself. Lowercase.
const RANDOM_ALLY: &[&str] = &[
    "a random ally",
    "random allied unit",
    "random friendly unit",
    "a random allied operator",
];

/// Words that, just before a target phrase, turn it into a condition on the
/// holder ("When there are no other friendly units in the surrounding 4
/// tiles, ATK +8%"): the talent then reaches nobody. Lowercase.
const ABSENCE: &[&str] = &["no other", "no ", "without", "none of"];

/// Whether the target phrase at byte `at` of the lowercase `lower` is negated
/// within its clause: one of `ABSENCE` in the 40 characters before it, after
/// the last `;` or `.`.
fn absent(lower: &str, at: usize) -> bool {
    let mut start = at.saturating_sub(40);
    while !lower.is_char_boundary(start) {
        start -= 1;
    }
    let window = &lower[start..at];
    let clause = window.rsplit([';', '.']).next().unwrap_or(window);
    ABSENCE.iter().any(|w| clause.contains(w))
}

/// The text with every bracket's inside blanked unless it is a class word, so
/// "[Lungmen Guard Department]" does not read as Guards.
fn mask_brackets(p: &str, vocab: &Vocab) -> String {
    BRACKET_RE
        .replace_all(p, |c: &regex::Captures<'_>| {
            if vocab.class_word(&c[1]).is_some() {
                c[0].to_string()
            } else {
                "[]".to_string()
            }
        })
        .into_owned()
}

/// Parse the target of a talent description. `None` when it names only
/// enemies or nothing.
#[allow(clippy::too_many_lines)] // one rule per target kind, in match order
pub fn parse_target(raw: &str, vocab: &Vocab) -> Option<Target> {
    use SynergyTargetKind as K;
    let p = plain(raw);
    let lower = p.to_lowercase();
    let masked = mask_brackets(&p, vocab);
    let class_match = vocab.class_re.as_ref().and_then(|re| re.find(&masked));
    let first_class = class_match.and_then(|m| {
        let word = m.as_str();
        vocab
            .class_word(word)
            .map(|v| (v.to_string(), word.to_string(), m.start()))
    });

    for (phrase, sp) in [
        ("offensive recovery", "INCREASE_WHEN_ATTACK"),
        ("defensive recovery", "INCREASE_WHEN_TAKEN_DAMAGE"),
        ("auto recovery", "INCREASE_WITH_TIME"),
    ] {
        if lower.contains(phrase) {
            let label = first_class
                .as_ref()
                .map_or_else(|| phrase.to_string(), |(_, w, _)| format!("{w} ({phrase})"));
            let mut t = Target::new(K::SpType, Some(sp.to_string()), label);
            t.class = first_class.map(|(v, _, _)| v);
            return Some(t);
        }
    }
    if let Some(c) = COST_RE.captures(&p) {
        let n = (1..=4)
            .find_map(|i| c.get(i))
            .map(|m| m.as_str().to_string());
        return Some(Target::new(K::CostAtMost, n, c[0].to_string()));
    }
    if lower.contains("the next operator") {
        return Some(Target::new(K::Next, None, "the next Operator".into()));
    }
    let random = RANDOM_ALLY
        .iter()
        .find(|w| lower.contains(*w))
        .map(|w| (*w).to_string())
        .or_else(|| RANDOM_UNIT_RE.find(&lower).map(|m| m.as_str().to_string()));
    if let Some(label) = random {
        let mut t = Target::new(K::Everyone, None, label);
        t.unseen = true;
        return Some(t);
    }
    let masked_lower = masked.to_lowercase();
    let first_class = first_class.filter(|(_, _, at)| !absent(&masked_lower, *at));
    if let Some((value, word, _)) = first_class {
        return Some(Target::new(K::Class, Some(value), word));
    }
    for c in BRACKET_RE.captures_iter(&p) {
        let whole = c.get(0).expect("group 0");
        let after = p[whole.end()..].trim_start().to_lowercase();
        if after.starts_with("enem") {
            continue;
        }
        let name = c[1].trim().to_string();
        if let Some((_, pos)) = vocab.positions.iter().find(|(w, _)| *w == name) {
            return Some(Target::new(K::Position, Some((*pos).to_string()), name));
        }
        if let Some(id) = vocab.tag_ids.get(&name.to_lowercase()) {
            return Some(Target::new(K::Tag, Some(id.to_string()), name));
        }
        let powers = vocab.powers(&name);
        if let Some(first) = powers.first() {
            let mut t = Target::new(K::Faction, Some(first.clone()), name);
            t.powers = powers;
            return Some(t);
        }
        // An unresolved bracket names allies only when "Operator" follows it;
        // otherwise it is a status ("[Arts Fragility]"), an enemy kind
        // ("[Infected Creatures]") or a game mode, and is skipped.
        if after.starts_with("operator") {
            return Some(Target::new(K::Unknown, Some(name.clone()), name));
        }
    }
    for c in ALL_X_OPS_RE.captures_iter(&p) {
        let name = c[1].trim().to_string();
        let powers = vocab.powers(&name);
        if let Some(first) = powers.first() {
            let mut t = Target::new(K::Faction, Some(first.clone()), name);
            t.powers = powers;
            return Some(t);
        }
    }
    if let Some(c) = vocab.position_re.as_ref().and_then(|re| {
        re.captures(&p)
            .filter(|c| !absent(&lower, c.get(0).map_or(0, |m| m.start())))
    }) {
        let pos = vocab
            .positions
            .iter()
            .find(|(w, _)| *w == c[1])
            .map(|(_, p)| (*p).to_string());
        return Some(Target::new(K::Position, pos, c[0].to_string()));
    }
    // The first everyone phrase that is not negated ("no other friendly
    // units" is Ceobe's own condition, not a grant).
    let everyone = EVERYONE
        .iter()
        .find(|w| lower.match_indices(*w).any(|(at, _)| !absent(&lower, at)));
    let negated = EVERYONE.iter().any(|w| lower.contains(*w));
    if everyone.is_none() && negated {
        return None;
    }
    everyone.map(|phrase| Target::new(K::Everyone, None, (*phrase).to_string()))
}

/// When the effect works, from the description's wording.
pub fn parse_cond(raw: &str) -> SynergyCond {
    let lower = lower_plain(raw);
    if any_of(&lower, SQUAD_WORDS) {
        SynergyCond::Squad
    } else if any_of(&lower, NEAR_WORDS) {
        SynergyCond::Near
    } else if any_of(&lower, SKILL_WORDS) {
        SynergyCond::Skill
    } else {
        SynergyCond::Field
    }
}

/// A count the text requires: `(count, counts_giver)`. "at least one more"
/// counts others only; "4 [X] Operators in the squad" counts the giver when the
/// giver fits.
pub fn parse_need(raw: &str) -> Option<(i32, bool)> {
    let p = plain(raw);
    let c = NEED_RE.captures(&p)?;
    if let Some(n) = c.get(1) {
        return Some((word_num(n.as_str())?, c.get(2).is_none()));
    }
    let n = c.get(3).or_else(|| c.get(4))?;
    Some((word_num(n.as_str())?, true))
}

pub fn highest_only(raw: &str) -> bool {
    lower_plain(raw).contains("only the highest")
}

/// Every power id an operator belongs to: `nation_id`, `group_id`, `team_id`
/// and every `SubPower` id (the game counts those for faction effects, see
/// `Operator::sub_power`).
pub fn power_ids(op: &Operator) -> impl Iterator<Item = &str> {
    let main = [
        Some(&op.nation_id),
        op.group_id.as_ref(),
        op.team_id.as_ref(),
    ];
    let sub = op.sub_power.iter().flatten().flat_map(|p| {
        [
            p.nation_id.as_ref(),
            p.group_id.as_ref(),
            p.team_id.as_ref(),
        ]
    });
    main.into_iter()
        .chain(sub)
        .flatten()
        .map(String::as_str)
        .filter(|id| !id.is_empty())
}

/// Whether `seat` fits `target`. `cost` is the seat's DP cost for the
/// `CostAtMost` kind; `tag_ids` are the seat's recruitment tag ids on the
/// reader's server. `Everyone` fits anyone; `Next` and `Unknown` fit nobody.
pub fn fits(target: &Target, seat: &Seat<'_>, cost: i32, tag_ids: &[i32]) -> bool {
    let op = seat.op;
    let class_ok = target
        .class
        .as_deref()
        .is_none_or(|c| op.profession.to_raw_str() == c);
    let v = target.value.as_deref().unwrap_or("");
    match target.kind {
        SynergyTargetKind::Everyone => true,
        SynergyTargetKind::Class => op.profession.to_raw_str() == v,
        SynergyTargetKind::Faction => power_ids(op).any(|id| target.powers.iter().any(|p| p == id)),
        SynergyTargetKind::Tag => v.parse::<i32>().is_ok_and(|id| tag_ids.contains(&id)),
        SynergyTargetKind::SpType => {
            class_ok
                && seat
                    .skill
                    .as_ref()
                    .is_some_and(|s| s.level.sp_data.sp_type == v)
        }
        SynergyTargetKind::Position => match op.position {
            OperatorPosition::Melee => v == "MELEE",
            OperatorPosition::Ranged => v == "RANGED",
            OperatorPosition::All => true,
            _ => false,
        },
        SynergyTargetKind::CostAtMost => v.parse::<i32>().is_ok_and(|n| cost <= n),
        SynergyTargetKind::Next | SynergyTargetKind::Unknown => false,
    }
}
