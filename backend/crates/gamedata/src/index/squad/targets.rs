use std::collections::HashMap;
use std::sync::LazyLock;

use myrtle_gamedata_raw::tables::gacha::{MELEE_TAG_ID, PROFESSION_TAG_IDS, RANGED_TAG_ID};
use regex::Regex;
use serde::Serialize;

use super::SquadIndexData;
use super::text::{any_of, lower_plain, plain};
use super::types::{SynergyCond, SynergyTargetKind};

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

pub const POSITION_TAG_IDS: [(&str, i32); 2] = [("MELEE", MELEE_TAG_ID), ("RANGED", RANGED_TAG_ID)];

static RANDOM_UNIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"an allied unit within range (?:gains|receives?)").expect("random unit regex")
});
static PLACEHOLDER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{[^{}]*\}").expect("placeholder regex"));
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

pub fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[must_use]
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

pub struct Vocab {
    /// Class word -> profession code
    classes: Vec<(String, String)>,
    class_re: Option<Regex>,
    /// Lowercase recruitment tag name -> `TagId`.
    tag_ids: HashMap<String, i32>,
    /// Position tag name -> `MELEE` / `RANGED`.
    positions: Vec<(String, &'static str)>,
    position_re: Option<Regex>,
    /// Normalized `handbook_team_table` name -> power id.
    factions: Vec<(String, String)>,
    faction_re: Option<Regex>,
}

fn word_alternation(words: &[String]) -> String {
    let mut sorted: Vec<&String> = words.iter().collect();
    sorted.sort_by_key(|w| std::cmp::Reverse(w.len()));
    sorted
        .iter()
        .map(|w| regex::escape(w))
        .collect::<Vec<_>>()
        .join("|")
}

impl Vocab {
    #[must_use]
    pub fn from_data(data: &SquadIndexData<'_>) -> Self {
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
        let faction_words: Vec<String> = data
            .factions
            .values()
            .filter(|t| !t.power_id.is_empty() && t.power_id != "none")
            .map(|t| t.power_name.clone())
            .filter(|n| !n.is_empty())
            .collect();
        let faction_re = (!faction_words.is_empty())
            .then(|| {
                let names = word_alternation(&faction_words);
                Regex::new(&format!(
                    r"\bfrom\s+'?({names})'?(?:[^\p{{L}}\p{{N}}]|$)|(?:^|[^\p{{L}}\p{{N}}])'?({names})'?\s+(?:[Oo]perators?|[Mm]embers?)\b"
                ))
                .ok()
            })
            .flatten();
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
            faction_re,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Target {
    pub kind: SynergyTargetKind,
    pub value: Option<String>,
    pub powers: Vec<String>,
    pub class: Option<String>,
    pub unseen: bool,
    pub label: String,
}

impl Target {
    const fn new(kind: SynergyTargetKind, value: Option<String>, label: String) -> Self {
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

const RANDOM_ALLY: &[&str] = &[
    "a random ally",
    "random allied unit",
    "random friendly unit",
    "a random allied operator",
];

const ABSENCE: &[&str] = &["no other", "no ", "without", "none of"];

fn absent(lower: &str, at: usize) -> bool {
    let mut start = at.saturating_sub(40);
    while !lower.is_char_boundary(start) {
        start -= 1;
    }
    let window = &lower[start..at];
    let clause = window.rsplit([';', '.']).next().unwrap_or(window);
    ABSENCE.iter().any(|w| clause.contains(w))
}

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

#[allow(clippy::too_many_lines)]
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
        if let Some(t) = bracket_faction(&p, whole.end(), &name, vocab) {
            return Some(t);
        }
        if after.starts_with("operator") {
            return Some(Target::new(K::Unknown, Some(name.clone()), name));
        }
    }
    if let Some(t) = all_x_operators(&p, vocab).or_else(|| unbracketed_faction(&p, vocab)) {
        return Some(t);
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
    let everyone = EVERYONE
        .iter()
        .find(|w| lower.match_indices(*w).any(|(at, _)| !absent(&lower, at)));
    let negated = EVERYONE.iter().any(|w| lower.contains(*w));
    if everyone.is_none() && negated {
        return None;
    }
    everyone.map(|phrase| Target::new(K::Everyone, None, (*phrase).to_string()))
}

fn bracket_faction(p: &str, end: usize, name: &str, vocab: &Vocab) -> Option<Target> {
    let mut powers = vocab.powers(name);
    let first = powers.first().cloned()?;
    let mut label = name.to_string();
    let mut end = end;
    for next in BRACKET_RE.captures_iter(&p[end..]) {
        let at = next.get(0).expect("group 0");
        let between = p[end..end + at.start()].trim().to_lowercase();
        if !matches!(
            between.as_str(),
            "and" | "or" | "," | ", and" | ", or" | "/" | "&"
        ) {
            break;
        }
        let more = vocab.powers(next[1].trim());
        if more.is_empty() {
            break;
        }
        for m in more {
            if !powers.contains(&m) {
                powers.push(m);
            }
        }
        let sep = p[end..end + at.start()].trim();
        let gap = if sep.starts_with(',') { "" } else { " " };
        label = format!("{label}{gap}{sep} {}", next[1].trim());
        end += at.end();
    }
    let mut t = Target::new(SynergyTargetKind::Faction, Some(first), label);
    t.powers = powers;
    Some(t)
}

fn all_x_operators(p: &str, vocab: &Vocab) -> Option<Target> {
    ALL_X_OPS_RE.captures_iter(p).find_map(|c| {
        let name = c[1].trim().to_string();
        let powers = vocab.powers(&name);
        let first = powers.first()?.clone();
        let mut t = Target::new(SynergyTargetKind::Faction, Some(first), name);
        t.powers = powers;
        Some(t)
    })
}

fn unbracketed_faction(p: &str, vocab: &Vocab) -> Option<Target> {
    let re = vocab.faction_re.as_ref()?;
    re.captures_iter(p).find_map(|c| {
        let m = c.get(1).or_else(|| c.get(2))?;
        let powers = vocab.powers(m.as_str());
        let first = powers.first()?.clone();
        let mut t = Target::new(
            SynergyTargetKind::Faction,
            Some(first),
            m.as_str().to_string(),
        );
        t.powers = powers;
        t.unseen = p[..m.start()]
            .trim_end_matches('\'')
            .trim_end()
            .to_lowercase()
            .ends_with("a random");
        Some(t)
    })
}

pub fn parse_faction(raw: &str, vocab: &Vocab) -> Option<Target> {
    let p = PLACEHOLDER_RE.replace_all(&plain(raw), "").into_owned();
    for c in BRACKET_RE.captures_iter(&p) {
        let whole = c.get(0).expect("group 0");
        if p[whole.end()..]
            .trim_start()
            .to_lowercase()
            .starts_with("enem")
        {
            continue;
        }
        if let Some(t) = bracket_faction(&p, whole.end(), c[1].trim(), vocab) {
            return Some(t);
        }
    }
    all_x_operators(&p, vocab).or_else(|| unbracketed_faction(&p, vocab))
}

#[must_use]
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

pub fn parse_need(raw: &str) -> Option<(i32, bool)> {
    let p = plain(raw);
    let c = NEED_RE.captures_iter(&p).find(|c| {
        let after = p[c.get(0).map_or(0, |m| m.end())..]
            .trim_start()
            .to_lowercase();
        !after.starts_with("block") && !after.starts_with("enem")
    })?;
    if let Some(n) = c.get(1) {
        return Some((word_num(n.as_str())?, c.get(2).is_none()));
    }
    let n = c.get(3).or_else(|| c.get(4))?;
    Some((word_num(n.as_str())?, true))
}

#[must_use]
pub fn highest_only(raw: &str) -> bool {
    lower_plain(raw).contains("only the highest")
}
