//! One streaming pass over `sandbox_perm_table.json` that never builds a tree.
//!
//! The table has two consumers and neither needs its content: the grader's
//! [`SandboxUniverse`] keeps 11 counts, and the stage classifier keeps the
//! `(level id, code, name)` triples it names Reclamation Algorithm nodes with.
//! Both used to parse the whole file into a `serde_json::Value`, which cost
//! 28 to 60 MB of RSS per server at boot (169 MB over the four, production
//! `STEP_RSS`, 2026-10-19) and was held to the end of the load. This walk
//! visits every value once with a serde visitor, allocates only the strings it
//! keeps, and hands both consumers their part.
//!
//! `SANDBOX_VALUE_PARSE=1` restores the two `Value` parses
//! ([`value_parse_switch`]). `tests/sandbox_perm_scan_real_data_test.rs` pins
//! that both paths give the same counts and the same triples on the real data.

use std::collections::HashMap;
use std::fmt;
use std::io::{BufReader, Read};
use std::path::Path;

use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};

use crate::core::gamedata::tables::{DataError, SanitizingReader};
use crate::core::gamedata::types::sandbox_universe::SandboxUniverse;

/// Lowercased level id -> `(code, name)`, the classifier's `node_named` shape.
pub type LevelNamed = HashMap<String, (String, String)>;

/// What the counting pass hands its two consumers.
#[derive(Debug, Default)]
pub struct SandboxPermScan {
    pub universe: SandboxUniverse,
    /// Every object in the file carrying a `LevelId`/`levelId` and a non-empty
    /// `Code`/`code` or `Name`/`name`, first occurrence kept, in the order the
    /// old recursive walk over a `Value` visited them.
    pub level_named: LevelNamed,
}

/// `SANDBOX_VALUE_PARSE=1`: parse the table into a `serde_json::Value` twice,
/// once for the universe and once in the stage classifier, as before.
pub fn value_parse_switch() -> bool {
    crate::utils::env::switched_on("SANDBOX_VALUE_PARSE")
}

/// Scan `data_dir/sandbox_perm_table.json`. Errors as
/// [`crate::core::gamedata::tables::load_table`] does, and reports the same startup step.
pub fn scan_sandbox_perm(data_dir: &Path) -> Result<SandboxPermScan, DataError> {
    const TABLE: &str = "sandbox_perm_table";
    crate::core::startup::step(TABLE);
    let file = std::fs::File::open(data_dir.join(format!("{TABLE}.json"))).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            DataError::Missing {
                table: TABLE.to_owned(),
            }
        } else {
            DataError::Io(e)
        }
    })?;
    scan_reader(SanitizingReader::new(BufReader::new(file))).map_err(|error| DataError::Parse {
        table: TABLE.to_owned(),
        error,
    })
}

/// Scan a JSON document of the table's shape from `reader`.
pub fn scan_reader<R: Read>(reader: R) -> Result<SandboxPermScan, serde_json::Error> {
    let mut de = serde_json::Deserializer::from_reader(reader);
    let mut tally = Tally::default();
    let seen = Walk {
        at: At::Root,
        want_text: false,
        tally: &mut tally,
    }
    .deserialize(&mut de)?;
    // `serde_json::from_reader` rejects trailing content; so does this.
    de.end()?;

    let mut level_named = LevelNamed::new();
    for (level_id, code, name) in seen.named {
        level_named
            .entry(level_id.to_lowercase())
            .or_insert((code, name));
    }
    Ok(SandboxPermScan {
        universe: SandboxUniverse::from_tally(tally.found, &tally.arrays, tally.zones, &tally.maps),
        level_named,
    })
}

/// Where a value sits relative to `Detail.SANDBOX_V2[0].value`, the subtree
/// the universe counts. Everything else is `Other` and only feeds the triples.
#[derive(Clone, Copy, PartialEq, Eq)]
enum At {
    Root,
    Detail,
    SandboxList,
    SandboxEntry,
    SandboxValue,
    ZoneList,
    ZoneEntry,
    MapList,
    MapEntry,
    MapValue,
    Other,
}

/// The universe's raw counts, filled as the walk passes them.
#[derive(Default)]
struct Tally {
    found: bool,
    arrays: HashMap<String, usize>,
    zones: usize,
    maps: Vec<(String, usize)>,
}

/// One `(level id, code, name)` triple, level id not yet lowercased.
type Named = (String, String, String);

/// What a visited value reports to its parent.
#[derive(Default)]
struct Seen {
    /// Triples inside this value, in `Value` walk order.
    named: Vec<Named>,
    /// The value itself when it is a string the parent asked for.
    text: Option<String>,
    /// The element count when the value is an array.
    len: Option<usize>,
    /// `ZoneEntry`/`MapEntry`: the entry's string `key`.
    key: Option<String>,
    /// `MapEntry`/`MapValue`: the length of the `Nodes` array.
    nodes: usize,
}

/// Field names the triples read, in pairs of (preferred, fallback):
/// `LevelId`/`levelId`, `Code`/`code`, `Name`/`name`.
const FIELDS: [&str; 6] = ["LevelId", "levelId", "Code", "code", "Name", "name"];

/// `map.get(preferred).or_else(|| map.get(fallback)).and_then(as_str)`: the
/// preferred key wins whenever it is present, string or not.
fn pick(preferred: Option<Option<String>>, fallback: Option<Option<String>>) -> Option<String> {
    match preferred {
        Some(v) => v,
        None => fallback.flatten(),
    }
}

struct Walk<'t> {
    at: At,
    want_text: bool,
    tally: &'t mut Tally,
}

impl<'de> DeserializeSeed<'de> for Walk<'_> {
    type Value = Seen;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Seen, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Walk<'_> {
    type Value = Seen;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_bool<E>(self, _: bool) -> Result<Seen, E> {
        Ok(Seen::default())
    }
    fn visit_i64<E>(self, _: i64) -> Result<Seen, E> {
        Ok(Seen::default())
    }
    fn visit_u64<E>(self, _: u64) -> Result<Seen, E> {
        Ok(Seen::default())
    }
    fn visit_f64<E>(self, _: f64) -> Result<Seen, E> {
        Ok(Seen::default())
    }
    fn visit_unit<E>(self) -> Result<Seen, E> {
        Ok(Seen::default())
    }
    fn visit_none<E>(self) -> Result<Seen, E> {
        Ok(Seen::default())
    }

    fn visit_str<E>(self, v: &str) -> Result<Seen, E> {
        Ok(Seen {
            text: self.want_text.then(|| v.to_owned()),
            ..Seen::default()
        })
    }

    fn visit_string<E>(self, v: String) -> Result<Seen, E> {
        Ok(Seen {
            text: self.want_text.then_some(v),
            ..Seen::default()
        })
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Seen, A::Error> {
        let mut named = Vec::new();
        let mut len = 0usize;
        loop {
            let at = match self.at {
                At::SandboxList if len == 0 => At::SandboxEntry,
                At::ZoneList => At::ZoneEntry,
                At::MapList => At::MapEntry,
                _ => At::Other,
            };
            let Some(child) = seq.next_element_seed(Walk {
                at,
                want_text: false,
                tally: &mut *self.tally,
            })?
            else {
                break;
            };
            match self.at {
                // A zone counts unless its key names the tutorial; a keyless
                // or non-object entry counts, as in `SandboxUniverse::build`.
                At::ZoneList if !child.key.as_deref().is_some_and(|k| k.contains("tutorial")) => {
                    self.tally.zones += 1;
                }
                At::MapList => self
                    .tally
                    .maps
                    .push((child.key.unwrap_or_default(), child.nodes)),
                _ => {}
            }
            named.extend(child.named);
            len += 1;
        }
        Ok(Seen {
            named,
            len: Some(len),
            ..Seen::default()
        })
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Seen, A::Error> {
        let mut out = Seen::default();
        let mut fields: [Option<Option<String>>; 6] = Default::default();
        // Children that hold triples, by key: a `Value` object is a `BTreeMap`
        // (no `preserve_order`), so the old walk visited them in key order.
        let mut children: Vec<(String, Vec<Named>)> = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            let slot = FIELDS.iter().position(|f| *f == key);
            let wants_key = matches!(self.at, At::ZoneEntry | At::MapEntry) && key == "key";
            let at = match (self.at, key.as_str()) {
                (At::Root, "Detail") => At::Detail,
                (At::Detail, "SANDBOX_V2") => At::SandboxList,
                (At::SandboxEntry, "value") => {
                    self.tally.found = true;
                    At::SandboxValue
                }
                // A repeated key replaces the earlier value in a `Value`.
                (At::SandboxValue, "ZoneData") => {
                    self.tally.zones = 0;
                    At::ZoneList
                }
                (At::SandboxValue, "MapData") => {
                    self.tally.maps.clear();
                    At::MapList
                }
                (At::MapEntry, "value") => At::MapValue,
                _ => At::Other,
            };
            let child = map.next_value_seed(Walk {
                at,
                want_text: slot.is_some() || wants_key,
                tally: &mut *self.tally,
            })?;
            match (self.at, key.as_str()) {
                (At::SandboxValue, _) => match child.len {
                    Some(n) => {
                        self.tally.arrays.insert(key.clone(), n);
                    }
                    None => {
                        self.tally.arrays.remove(&key);
                    }
                },
                (At::MapEntry, "value") => out.nodes = child.nodes,
                (At::MapValue, "Nodes") => out.nodes = child.len.unwrap_or(0),
                _ => {}
            }
            if wants_key {
                out.key.clone_from(&child.text);
            }
            if let Some(i) = slot {
                fields[i] = Some(child.text);
            }
            if !child.named.is_empty() {
                children.push((key, child.named));
            }
        }

        let [lid, lid_lower, code, code_lower, name, name_lower] = fields;
        if let Some(level_id) = pick(lid, lid_lower) {
            let code = pick(code, code_lower).unwrap_or_default();
            let name = pick(name, name_lower).unwrap_or_default();
            if !code.is_empty() || !name.is_empty() {
                out.named.push((level_id, code, name));
            }
        }
        children.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, named) in children {
            out.named.extend(named);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::scan_reader;
    use crate::core::gamedata::types::sandbox_universe::SandboxUniverse;

    fn both(doc: &serde_json::Value) -> (SandboxUniverse, super::SandboxPermScan) {
        let scan = scan_reader(doc.to_string().as_bytes()).expect("scan");
        (SandboxUniverse::build(doc), scan)
    }

    #[test]
    fn counts_match_the_value_path() {
        let doc = serde_json::json!({ "Detail": { "SANDBOX_V2": [
            { "value": {
                "ZoneData": [ { "key": "z_1_0" }, { "key": "z_1_tutorial" }, 5, { "x": 1 } ],
                "MapData": [
                    { "key": "sandbox_1_main", "value": { "Nodes": [1, 2, 3] } },
                    { "key": "sandbox_1_hunt_1", "value": { "Nodes": [1] } },
                ],
                "CraftItemData": [ {}, {}, {} ],
                "BuildingItemData": [ {} ],
                "FoodData": { "not": "an array" },
                "StageData": [ { "levelId": "obt/sandbox/a", "code": "A-1" } ],
            } },
            { "value": { "StageData": [ {}, {}, {}, {} ] } },
        ] } });
        let (old, scan) = both(&doc);
        assert_eq!(old, scan.universe);
        assert_eq!(scan.universe.max_zones, 3);
        assert_eq!(scan.universe.max_nodes, 3);
        assert_eq!(scan.universe.max_blueprints, 3);
        assert_eq!(scan.universe.max_recipes, 0);
        assert_eq!(scan.universe.max_stages, 1, "only SANDBOX_V2[0] counts");
    }

    #[test]
    fn map_nodes_fall_back_to_one_variant_per_group() {
        let doc = serde_json::json!({ "Detail": { "SANDBOX_V2": [ { "value": {
            "MapData": [
                { "key": "sandbox_1_hunt_1", "value": { "Nodes": [1, 2] } },
                { "key": "sandbox_1_hunt_2", "value": { "Nodes": [1, 2, 3] } },
                { "key": "sandbox_1_rift", "value": { "Nodes": [1] } },
                { "value": { "Nodes": [1] } },
            ],
        } } ] } });
        let (old, scan) = both(&doc);
        assert_eq!(old, scan.universe);
        assert_eq!(scan.universe.max_nodes, 5);
    }

    #[test]
    fn a_table_without_sandbox_v2_counts_nothing() {
        let doc = serde_json::json!({ "Detail": { "OTHER": [] } });
        let (old, scan) = both(&doc);
        assert_eq!(old, scan.universe);
        assert_eq!(scan.universe, SandboxUniverse::default());
    }

    #[test]
    fn triples_follow_key_order_and_field_precedence() {
        // Key order, not file order: "a" is visited before "b", so its code
        // wins the shared (lowercased) level id.
        let doc = serde_json::json!({
            "b": { "levelId": "OBT/X", "code": "B" },
            "a": { "levelId": "obt/x", "code": "A" },
            // A present but non-string `LevelId` shadows `levelId`.
            "c": { "LevelId": null, "levelId": "obt/y", "code": "C" },
            // `Name` wins over `name`; `code` falls back to "".
            "d": { "levelId": "obt/z", "Name": "Zed", "name": "zed" },
            // Neither code nor name: skipped.
            "e": { "levelId": "obt/w" },
            // The parent's own triple precedes its children's.
            "f": { "levelId": "obt/v", "code": "parent", "kid": { "levelId": "obt/v", "code": "child" } },
        });
        let scan = scan_reader(doc.to_string().as_bytes()).expect("scan");
        let get = |k: &str| scan.level_named.get(k).cloned();
        assert_eq!(get("obt/x"), Some(("A".into(), String::new())));
        assert_eq!(get("obt/y"), None);
        assert_eq!(get("obt/z"), Some((String::new(), "Zed".into())));
        assert_eq!(get("obt/w"), None);
        assert_eq!(get("obt/v"), Some(("parent".into(), String::new())));
        assert_eq!(scan.level_named.len(), 3);
    }

    #[test]
    fn trailing_content_is_rejected() {
        assert!(scan_reader(&b"{} {}"[..]).is_err());
    }
}
