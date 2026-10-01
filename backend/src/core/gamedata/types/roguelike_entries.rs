//! Integrated Strategies as rankable entries: each theme, and the items a
//! player collects in it, read off `roguelike_topic_table`.
//!
//! IS numbering is NOT the table's: `rogue_N` is IS N+1 (`rogue_1` is IS2
//! Phantom, `rogue_2` IS3 Mizuki), because IS1 Ceobe's Fungimist never made it
//! into the table. The entries keep the table's ids; the client numbers them.
//!
//! An item is offered when its `Items` type is one a player collects and ranks
//! ([`ITEM_TYPES`]). Relics, tools, Plays, Foldartals and Thoughts must also be
//! listed in the theme's `ArchiveComp`, the in-game collection, which drops the
//! upgrade variants (`rogue_4_relic_legacy_1_a`) and placeholders the `Items`
//! table also carries. Squads, Wraths and Tongbao have no archive list of
//! items, so they are de-duplicated by name instead: the table spells each
//! squad once per difficulty variant (`rogue_4_band_22_1` to `_25`) and each
//! Tongbao once per mint (2,334 `COPPER` rows, 103 names, on EN 2026-10-01).
//!
//! Left out on purpose: tickets (`RECRUIT_TICKET`, `UPGRADE_TICKET`,
//! `CUSTOM_TICKET`), run resources (`HP`, `GOLD`, `POPULATION`, `EXP`,
//! `SHIELD`, `KEY_POINT`, and the like), internal flags (`FEATURE`,
//! `TOTEM_EFFECT`, `COPPER_BUFF`, `VISION`, `CHAOS*`, `DISASTER*`) and dice.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use serde_json::Value;

/// The item types offered, in the order a theme's entries are listed after
/// the theme itself.
const ITEM_TYPES: [&str; 9] = [
    "BAND",
    "RELIC",
    "ACTIVE_TOOL",
    "EXPLORE_TOOL",
    "CAPSULE",
    "TOTEM",
    "FRAGMENT",
    "WRATH",
    "COPPER",
];

/// The item type a theme entry carries.
pub const THEME_TYPE: &str = "THEME";

/// `ArchiveComp` categories whose ids gate the archived types. `Trap` is
/// where `rogue_1` files its six tools; later themes list them under `Relic`.
const ARCHIVE_CATEGORIES: [&str; 5] = ["Relic", "Trap", "Capsule", "Totem", "Fragment"];

/// Types that must appear in the archive to be offered.
const ARCHIVED_TYPES: [&str; 6] = [
    "RELIC",
    "ACTIVE_TOOL",
    "EXPLORE_TOOL",
    "CAPSULE",
    "TOTEM",
    "FRAGMENT",
];

/// One theme or item.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoguelikeEntry {
    pub id: String,
    /// `rogue_N`.
    pub theme_id: String,
    /// The table's `Type_` (`RELIC`, `TOTEM`), or [`THEME_TYPE`].
    pub item_type: String,
    pub name: String,
    /// Asset stems to try for the icon, best first: a theme's home entry
    /// display, an item's `IconId`, its `ItemIconGroupId`, then its id with the
    /// variant suffix cut (`rogue_5_wrath_1_a` -> `rogue_5_wrath_1`).
    pub icon_stems: Vec<String>,
}

/// Every theme and offered item, in catalogue order: themes by their `Sort`,
/// each theme followed by its items grouped by [`ITEM_TYPES`] and ordered by
/// id with numbers compared as numbers.
#[derive(Debug, Clone, Default)]
pub struct RoguelikeEntries {
    pub entries: Vec<RoguelikeEntry>,
    index: HashMap<String, usize>,
}

impl RoguelikeEntries {
    pub fn get(&self, id: &str) -> Option<&RoguelikeEntry> {
        self.index.get(id).map(|&i| &self.entries[i])
    }

    /// Reads `Topics` and `Details`, each a `[{key, value}]` list (the
    /// flatbuffer export) or a `{key: value}` dict.
    pub fn from_tables(topics: &Value, details: &Value) -> Self {
        let topics = kv_pairs(topics);
        let details: HashMap<&str, &Value> = kv_pairs(details).into_iter().collect();

        let mut themes: Vec<(i64, &str, &Value)> = topics
            .into_iter()
            .map(|(id, t)| {
                (
                    t.get("Sort").and_then(Value::as_i64).unwrap_or(i64::MAX),
                    id,
                    t,
                )
            })
            .collect();
        themes.sort_by_key(|&(sort, id, _)| (sort, id));

        let mut out = Self::default();
        for (_, theme_id, topic) in themes {
            out.push(theme_entry(theme_id, topic));
            if let Some(detail) = details.get(theme_id) {
                for entry in theme_items(theme_id, detail) {
                    out.push(entry);
                }
            }
        }
        out
    }

    fn push(&mut self, entry: RoguelikeEntry) {
        if self.index.contains_key(&entry.id) {
            return;
        }
        self.index.insert(entry.id.clone(), self.entries.len());
        self.entries.push(entry);
    }
}

/// The theme itself, named by the topic and drawn by its home-screen key
/// visual.
fn theme_entry(theme_id: &str, topic: &Value) -> RoguelikeEntry {
    let display = topic
        .get("HomeEntryDisplayData")
        .and_then(Value::as_array)
        .and_then(|list| list.first())
        .and_then(|d| str_field(d, "DisplayId"))
        .map(str::to_owned);
    RoguelikeEntry {
        id: theme_id.to_owned(),
        theme_id: theme_id.to_owned(),
        item_type: THEME_TYPE.to_owned(),
        name: str_field(topic, "Name").unwrap_or(theme_id).to_owned(),
        icon_stems: display
            .into_iter()
            .chain(std::iter::once(format!("{theme_id}_entry_display_1")))
            .collect(),
    }
}

/// The theme's offered items, by [`ITEM_TYPES`] and then natural id order.
fn theme_items(theme_id: &str, detail: &Value) -> Vec<RoguelikeEntry> {
    let archived = archive_ids(detail);
    let mut items: Vec<(&str, &Value)> = detail.get("Items").map(kv_pairs).unwrap_or_default();
    items.sort_by(|a, b| natural_cmp(a.0, b.0));

    let mut seen_names: HashSet<(&str, &str)> = HashSet::new();
    let mut out: Vec<(usize, RoguelikeEntry)> = Vec::new();
    for (id, item) in items {
        let Some(item_type) = str_field(item, "Type_").or_else(|| str_field(item, "type")) else {
            continue;
        };
        let Some(order) = ITEM_TYPES.iter().position(|t| *t == item_type) else {
            continue;
        };
        let name = str_field(item, "Name").unwrap_or(id);
        let offered = if ARCHIVED_TYPES.contains(&item_type) {
            archived.contains(id)
        } else {
            // No archive to gate on: one entry per name, and squads without
            // their difficulty variants.
            !(item_type == "BAND" && is_band_variant(id)) && seen_names.insert((item_type, name))
        };
        if !offered {
            continue;
        }
        out.push((
            order,
            RoguelikeEntry {
                id: id.to_owned(),
                theme_id: theme_id.to_owned(),
                item_type: item_type.to_owned(),
                name: name.to_owned(),
                icon_stems: item_icon_stems(id, item),
            },
        ));
    }
    // Stable: ids stay in natural order within a type.
    out.sort_by_key(|(order, _)| *order);
    out.into_iter().map(|(_, e)| e).collect()
}

/// See [`RoguelikeEntry::icon_stems`]: `IconId`, `ItemIconGroupId`, then the
/// id with its variant suffix cut, each once.
fn item_icon_stems(id: &str, item: &Value) -> Vec<String> {
    let mut stems: Vec<String> = Vec::new();
    for stem in [
        str_field(item, "IconId"),
        str_field(item, "ItemIconGroupId"),
        Some(strip_variant(id)),
    ]
    .into_iter()
    .flatten()
    {
        if !stems.iter().any(|s| s == stem) {
            stems.push(stem.to_owned());
        }
    }
    stems
}

/// Every id the theme's archive lists under [`ARCHIVE_CATEGORIES`].
fn archive_ids(detail: &Value) -> HashSet<&str> {
    let mut ids = HashSet::new();
    let Some(archive) = detail.get("ArchiveComp") else {
        return ids;
    };
    for category in ARCHIVE_CATEGORIES {
        let Some(inner) = archive.get(category).and_then(Value::as_object) else {
            continue;
        };
        for list in inner.values() {
            for (id, _) in kv_pairs(list) {
                ids.insert(id);
            }
        }
    }
    ids
}

/// `rogue_4_band_22_7`: a numbered variant of squad 22.
fn is_band_variant(id: &str) -> bool {
    let Some((head, tail)) = id.rsplit_once('_') else {
        return false;
    };
    tail.bytes().all(|b| b.is_ascii_digit())
        && head
            .rsplit_once("_band_")
            .is_some_and(|(_, n)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// The id with a trailing `_x` or `_x_N` variant suffix cut, where `x` is one
/// lowercase letter: `rogue_5_wrath_1_a_3` -> `rogue_5_wrath_1`.
fn strip_variant(id: &str) -> &str {
    let mut rest = id;
    if let Some((head, tail)) = rest.rsplit_once('_')
        && !tail.is_empty()
        && tail.bytes().all(|b| b.is_ascii_digit())
        && head
            .rsplit_once('_')
            .is_some_and(|(_, l)| l.len() == 1 && l.bytes().all(|b| b.is_ascii_lowercase()))
    {
        rest = head;
    }
    match rest.rsplit_once('_') {
        Some((head, l)) if l.len() == 1 && l.bytes().all(|b| b.is_ascii_lowercase()) => head,
        _ => rest,
    }
}

/// Compares ids with digit runs read as numbers, so `band_2` sorts before
/// `band_10`.
fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.as_bytes(), b.as_bytes());
    loop {
        match (a.first(), b.first()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let (va, na) = leading_number(a);
                let (vb, nb) = leading_number(b);
                match va.cmp(&vb) {
                    Ordering::Equal => {
                        a = &a[na..];
                        b = &b[nb..];
                    }
                    other => return other,
                }
            }
            (Some(x), Some(y)) => match x.cmp(y) {
                Ordering::Equal => {
                    a = &a[1..];
                    b = &b[1..];
                }
                other => return other,
            },
        }
    }
}

/// The digit run `s` starts with, as a number (0 past `u64`), and its length.
fn leading_number(s: &[u8]) -> (u64, usize) {
    let len = s.iter().take_while(|c| c.is_ascii_digit()).count();
    let value = std::str::from_utf8(&s[..len])
        .ok()
        .and_then(|d| d.parse().ok())
        .unwrap_or(0);
    (value, len)
}

fn str_field<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str)
}

/// `[{key, value}]` or `{key: value}` as `(key, value)` pairs.
fn kv_pairs(v: &Value) -> Vec<(&str, &Value)> {
    if let Some(list) = v.as_array() {
        list.iter()
            .filter_map(|e| Some((e.get("key")?.as_str()?, e.get("value")?)))
            .collect()
    } else if let Some(map) = v.as_object() {
        map.iter().map(|(k, v)| (k.as_str(), v)).collect()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_suffixes_are_cut() {
        assert_eq!(strip_variant("rogue_5_wrath_1_a"), "rogue_5_wrath_1");
        assert_eq!(strip_variant("rogue_5_wrath_1_a_3"), "rogue_5_wrath_1");
        assert_eq!(
            strip_variant("rogue_5_copper_B_01_a"),
            "rogue_5_copper_B_01"
        );
        assert_eq!(strip_variant("rogue_1_band_1"), "rogue_1_band_1");
        assert!(is_band_variant("rogue_4_band_22_7"));
        assert!(!is_band_variant("rogue_4_band_22"));
    }

    #[test]
    fn ids_sort_by_number() {
        let mut ids = vec!["band_10", "band_2", "band_1"];
        ids.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(ids, ["band_1", "band_2", "band_10"]);
    }
}
