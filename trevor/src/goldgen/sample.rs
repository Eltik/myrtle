//! Deterministic, stratified sampling of source passages for generation.
//!
//! Single-passage items spread over story category (main, side, record,
//! vignette) and position in the story (first, middle, last third), with the
//! category weights square-rooted so the 8,244 event chunks do not drown the
//! 2,251 main-story ones. Multi-hop items pair two chunks from different
//! stories in which the same named character speaks.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::eval::goldset::Stratum;
use crate::search::store::ChunkStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleItem {
    pub id: String,
    pub stratum: Stratum,
    pub chunk_ids: Vec<String>,
    /// The shared character, for multi-hop items.
    pub entity: Option<String>,
    pub seed: u64,
}

/// Minimum passage size: below this a chunk rarely holds a fact worth asking.
pub const MIN_TOKENS: u32 = 250;
pub const MIN_WORDS: usize = 120;

pub struct Rng(pub u64);
impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        crate::util::splitmix64(&mut self.0)
    }
    pub fn below(&mut self, n: usize) -> usize {
        #[allow(clippy::cast_possible_truncation)]
        let r = (self.next_u64() % n as u64) as usize;
        r
    }
}

fn eligible(store: &ChunkStore, row: usize) -> bool {
    let c = &store.chunks[row];
    c.token_count >= MIN_TOKENS && c.text.split_whitespace().count() >= MIN_WORDS
}

/// Rows grouped by (category, position third).
fn cells(
    store: &ChunkStore,
    category_of: &HashMap<String, String>,
) -> BTreeMap<(String, u8), Vec<usize>> {
    let mut out: BTreeMap<(String, u8), Vec<usize>> = BTreeMap::new();
    let stories: BTreeSet<&str> = store.chunks.iter().map(|c| c.story_id.as_str()).collect();
    for s in stories {
        let rows = store.story_rows(s);
        let n = rows.len();
        let cat = category_of
            .get(s)
            .cloned()
            .unwrap_or_else(|| "unknown".to_owned());
        for (i, &row) in rows.iter().enumerate() {
            if !eligible(store, row) {
                continue;
            }
            #[allow(clippy::cast_possible_truncation)]
            let third = if n <= 1 {
                1
            } else {
                ((i * 3) / n).min(2) as u8
            };
            out.entry((cat.clone(), third)).or_default().push(row);
        }
    }
    out
}

/// `counts`: how many single-passage items to draw per stratum. A chunk is
/// used at most once, and at most two chunks come from any one story.
#[must_use]
pub fn single(
    store: &ChunkStore,
    category_of: &HashMap<String, String>,
    counts: &[(Stratum, usize)],
    round: u32,
    seed: u64,
    exclude: &BTreeSet<String>,
) -> Vec<SampleItem> {
    let cells = cells(store, category_of);
    let mut by_cat: BTreeMap<String, Vec<(u8, Vec<usize>)>> = BTreeMap::new();
    for ((cat, third), rows) in cells {
        by_cat.entry(cat).or_default().push((third, rows));
    }
    let cats: Vec<&String> = by_cat.keys().collect();
    #[allow(clippy::cast_precision_loss)]
    let weights: Vec<f64> = cats
        .iter()
        .map(|c| (by_cat[*c].iter().map(|(_, r)| r.len()).sum::<usize>() as f64).sqrt())
        .collect();
    let total: f64 = weights.iter().sum();
    let mut rng = Rng(seed ^ u64::from(round).wrapping_mul(0x9E37));
    let mut used: BTreeSet<usize> = store
        .chunks
        .iter()
        .enumerate()
        .filter(|(_, c)| exclude.contains(&c.chunk_id))
        .map(|(i, _)| i)
        .collect();
    let mut per_story: HashMap<&str, usize> = HashMap::new();
    for &r in &used {
        *per_story
            .entry(store.chunks[r].story_id.as_str())
            .or_default() += 1;
    }
    let mut out = Vec::new();
    for &(stratum, n) in counts {
        let mut made = 0;
        let mut tries = 0;
        while made < n && tries < n * 200 {
            tries += 1;
            #[allow(clippy::cast_precision_loss)]
            let mut x = (rng.next_u64() % 1_000_000) as f64 / 1_000_000.0 * total;
            let mut ci = 0;
            while ci + 1 < weights.len() && x >= weights[ci] {
                x -= weights[ci];
                ci += 1;
            }
            let thirds = &by_cat[cats[ci]];
            let (_, rows) = &thirds[rng.below(thirds.len())];
            let row = rows[rng.below(rows.len())];
            let story = store.chunks[row].story_id.as_str();
            if used.contains(&row) || per_story.get(story).copied().unwrap_or(0) >= 2 {
                continue;
            }
            used.insert(row);
            *per_story.entry(story).or_default() += 1;
            let id = format!("r{round}-{}-{made:03}", stratum.name());
            out.push(SampleItem {
                seed: super::filters::fnv1a(&id),
                id,
                stratum,
                chunk_ids: vec![store.chunks[row].chunk_id.clone()],
                entity: None,
            });
            made += 1;
        }
    }
    out
}

/// Words that mark a generic role rather than one character: "Reunion
/// Member", "Audience Member B", "An Excited Spectator", "Mischievous Kid".
const GENERIC_WORDS: [&str; 89] = [
    "member",
    "officer",
    "soldier",
    "guard",
    "voice",
    "mercenary",
    "citizen",
    "worker",
    "crew",
    "agent",
    "operator",
    "staff",
    "fighter",
    "warrior",
    "caster",
    "child",
    "children",
    "kid",
    "kids",
    "man",
    "woman",
    "men",
    "women",
    "boy",
    "girl",
    "noble",
    "servant",
    "student",
    "clerk",
    "refugee",
    "official",
    "representative",
    "secretary",
    "communicator",
    "villager",
    "merchant",
    "driver",
    "researcher",
    "scientist",
    "nurse",
    "infected",
    "elder",
    "customer",
    "passerby",
    "stranger",
    "figure",
    "youth",
    "cleric",
    "knight",
    "audience",
    "spectator",
    "father",
    "mother",
    "manager",
    "shopkeeper",
    "assassin",
    "vampire",
    "spirit",
    "mysterious",
    "excited",
    "young",
    "old",
    "elderly",
    "masked",
    "hooded",
    "unknown",
    "strange",
    "suspicious",
    "nervous",
    "angry",
    "bishop",
    "priest",
    "guy",
    "lady",
    "boss",
    "chief",
    "leader",
    "director",
    "announcer",
    "narrator",
    "everyone",
    "farmer",
    "tourist",
    "host",
    "prisoner",
    "prisoners",
    "headman",
    "burdenbeast",
    "crossbowman",
];

/// A speaker name that plausibly denotes one specific character. Operator
/// names from the story index always qualify; anything else must look like
/// a proper name.
#[must_use]
pub fn is_character_name(name: &str, operators: &BTreeSet<String>) -> bool {
    let n = name.trim();
    if operators.contains(n) {
        return true;
    }
    if n.is_empty()
        || n.contains('?')
        || n.contains('…')
        || n.starts_with(|c: char| !c.is_uppercase())
    {
        return false;
    }
    let words: Vec<String> = n
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .collect();
    let first = words.first().map_or("", String::as_str);
    let last = words.last().map_or("", String::as_str);
    words.len() <= 3
        && !matches!(first, "a" | "an" | "the")
        && (words.len() <= 1 || last.len() != 1)
        && !words.iter().any(|w| GENERIC_WORDS.contains(&w.as_str()))
}

/// Pairs of chunks from different stories, in different groups, sharing a named speaker who
/// appears in 3 to 150 stories: common enough to connect stories, rare
/// enough to be specific.
#[must_use]
pub fn multi_hop(
    store: &ChunkStore,
    operators: &BTreeSet<String>,
    n: usize,
    round: u32,
    seed: u64,
    exclude: &BTreeSet<String>,
) -> Vec<SampleItem> {
    let mut stories_of: BTreeMap<&str, BTreeMap<&str, Vec<usize>>> = BTreeMap::new();
    for (row, c) in store.chunks.iter().enumerate() {
        if !eligible(store, row) || exclude.contains(&c.chunk_id) {
            continue;
        }
        for s in &c.speakers {
            if is_character_name(s, operators) {
                stories_of
                    .entry(s.as_str())
                    .or_default()
                    .entry(c.story_id.as_str())
                    .or_default()
                    .push(row);
            }
        }
    }
    let entities: Vec<(&str, &BTreeMap<&str, Vec<usize>>)> = stories_of
        .iter()
        .filter(|(_, st)| (3..=150).contains(&st.len()))
        .map(|(e, st)| (*e, st))
        .collect();
    let mut rng = Rng(seed ^ 0x6d75_6c74 ^ u64::from(round).wrapping_mul(0x9E37));
    let mut used: BTreeSet<usize> = BTreeSet::new();
    let mut used_entity: HashMap<&str, usize> = HashMap::new();
    let mut out = Vec::new();
    let mut tries = 0;
    while out.len() < n && tries < n * 500 && !entities.is_empty() {
        tries += 1;
        let (e, st) = entities[rng.below(entities.len())];
        if used_entity.get(e).copied().unwrap_or(0) >= 2 {
            continue;
        }
        let keys: Vec<&&str> = st.keys().collect();
        let (s1, s2) = (keys[rng.below(keys.len())], keys[rng.below(keys.len())]);
        if s1 == s2 {
            continue;
        }
        let (r1s, r2s) = (&st[*s1], &st[*s2]);
        let (a, b) = (r1s[rng.below(r1s.len())], r2s[rng.below(r2s.len())]);
        // Different groups, not just different stories: two chapters of one
        // event share a synopsis, so a question over them is barely two-hop.
        if used.contains(&a)
            || used.contains(&b)
            || store.chunks[a].group_id == store.chunks[b].group_id
        {
            continue;
        }
        used.insert(a);
        used.insert(b);
        *used_entity.entry(e).or_default() += 1;
        let id = format!("r{round}-multi_hop-{:03}", out.len());
        out.push(SampleItem {
            seed: super::filters::fnv1a(&id),
            id,
            stratum: Stratum::MultiHop,
            chunk_ids: vec![
                store.chunks[a].chunk_id.clone(),
                store.chunks[b].chunk_id.clone(),
            ],
            entity: Some(e.to_owned()),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_names_are_not_characters() {
        let ops: BTreeSet<String> = ["Shining".to_owned()].into();
        for n in [
            "Reunion Member",
            "L. G. D. Officer",
            "???",
            "Self-Salvation Corps Fighter",
            "unknown",
            "Audience Member B",
            "An Excited Spectator",
            "Mischievous Kid",
            "Shopkeeper",
            "Father",
        ] {
            assert!(!is_character_name(n, &ops), "{n}");
        }
        for n in [
            "Kal'tsit",
            "Wei Yenwu",
            "Ch'en",
            "W",
            "Uncle Tung",
            "Shining",
        ] {
            assert!(is_character_name(n, &ops), "{n}");
        }
    }
}
