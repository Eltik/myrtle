//! `roguelike_topic_table.json` read into the subset of its tree the load
//! reads, never the whole tree.
//!
//! The table is 17 MB of JSON on EN and 18.9 MB on CN. The load used to parse
//! it into a full `serde_json::Value` ([`super::roguelike::RoguelikeTopicTableFile`])
//! and hold it until the stage classifier had run, for three readers:
//! [`super::roguelike::RoguelikeGameData::from_table`] (theme maxima and
//! archive id lists), [`super::roguelike_entries::RoguelikeEntries::from_tables`]
//! (tier-list entries) and the stage classifier's mode names (theme names,
//! level triples, node ids, KV banners). All three read through `Value`
//! accessors with their own field precedence (`Name` before `name`, `Type_`
//! before `type`) and accept both the `FlatBuffer` `[{key, value}]` lists and
//! the CN-gamedata dicts, and the classifier's level walk is shape-agnostic.
//!
//! So the subset keeps the `Value` shape: a [`Spec`] per path says what to
//! keep, and everything the spec does not name is walked and dropped. The
//! readers run unchanged on the pruned tree, which is what makes the two
//! paths comparable output for output. What the walk keeps:
//!
//! - `Topics`: whole (5 KB on EN).
//! - per theme in `Details`: `Items` with only their type, name and icon
//!   fields, `ArchiveComp` with only its ids, the lengths of `Endings`,
//!   `Challenges`, `MonthSquad` and `Milestones` (elements replaced by `null`),
//!   each difficulty's `Grade`, `AutoSetKV` and `DetailConst.AutoSetKV`, and
//!   each stage's `Id`, `LevelId`, `Code` and `Name`.
//! - anywhere else: every object carrying a `LevelId`/`levelId`, with its
//!   `LevelId`, `levelId`, `Code`, `code`, `Name` and `name`, and the path to
//!   it, so the classifier's walk finds the same triples in the same order.
//!
//! `ROGUELIKE_VALUE_PARSE=1` restores the full `Value` parse
//! ([`value_parse_switch`]). `tests/roguelike_prune_real_data_test.rs` pins
//! that both paths give the same themes, entries and mode names on the real
//! tables.

use std::fmt;
use std::io::{BufReader, Read};
use std::path::Path;

use serde::Deserialize;
use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};

use super::roguelike::RoguelikeTopicTableFile;
use crate::core::gamedata::tables::{DataError, SanitizingReader};

const TABLE: &str = "roguelike_topic_table";

/// `ROGUELIKE_VALUE_PARSE=1`: parse the whole table into a `Value`, as before.
pub fn value_parse_switch() -> bool {
    crate::utils::env::switched_on("ROGUELIKE_VALUE_PARSE")
}

/// Read `data_dir/roguelike_topic_table.json` pruned to what the load reads.
/// Errors as [`crate::core::gamedata::tables::load_table`] does, and reports
/// the same startup step.
pub fn load_pruned(data_dir: &Path) -> Result<RoguelikeTopicTableFile, DataError> {
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
    prune_reader(SanitizingReader::new(BufReader::new(file))).map_err(|error| DataError::Parse {
        table: TABLE.to_owned(),
        error,
    })
}

/// Prune a JSON document of the table's shape from `reader`, then read the
/// root fields as the `Value` path does (same struct, same aliases).
pub fn prune_reader<R: Read>(reader: R) -> Result<RoguelikeTopicTableFile, serde_json::Error> {
    serde_json::from_value(prune_value_reader(reader)?)
}

/// The pruned tree itself, before the root struct reads it.
pub fn prune_value_reader<R: Read>(reader: R) -> Result<Value, serde_json::Error> {
    let mut de = serde_json::Deserializer::from_reader(reader);
    let kept = Prune(ROOT).deserialize(&mut de)?;
    // `serde_json::from_reader` rejects trailing content; so does this.
    de.end()?;
    Ok(kept.into_value())
}

/// What to keep of a value.
#[derive(Clone, Copy)]
enum Spec {
    /// Kept whole.
    Full,
    /// Dropped, except the objects carrying a level id inside it (see
    /// [`TRIPLE_KEYS`]) and the path to them.
    Drop,
    /// A placeholder: a string is kept, any other scalar becomes `null`, a
    /// container keeps what [`Spec::Drop`] keeps, or becomes `null`.
    Stub,
    /// An object's listed keys by their spec, every other key as [`Spec::Drop`].
    Fields(&'static [(&'static str, Spec)]),
    /// Every array element and every object value by the inner spec, so the
    /// length and the keys survive.
    Each(&'static Spec),
    /// A `[{key, value}]` list (each element keeps `key` whole and `value` by
    /// the inner spec) or a `{key: value}` dict (each value by the inner spec).
    Kv(&'static Spec),
    /// One element of a [`Spec::Kv`] list.
    KvEntry(&'static Spec),
}

/// The fields `collect_level_named` in the stage classifier reads off an
/// object with a `LevelId`/`levelId`. Kept on such an object wherever it sits.
const TRIPLE_KEYS: [&str; 6] = ["LevelId", "levelId", "Code", "code", "Name", "name"];

const STUB: Spec = Spec::Stub;
/// A list or dict whose length is read and whose content is not.
const SHAPE: Spec = Spec::Each(&STUB);

const STAGE: Spec = Spec::Fields(&[
    ("Id", Spec::Full),
    ("LevelId", Spec::Full),
    ("levelId", Spec::Full),
    ("Code", Spec::Full),
    ("code", Spec::Full),
    ("Name", Spec::Full),
    ("name", Spec::Full),
]);
/// A `Stages` element: `{key, value: stage}`, or the stage itself
/// (`st.get("value").unwrap_or(st)`).
const STAGE_ENTRY: Spec = Spec::Fields(&[
    ("key", Spec::Full),
    ("value", STAGE),
    ("Id", Spec::Full),
    ("LevelId", Spec::Full),
    ("levelId", Spec::Full),
    ("Code", Spec::Full),
    ("code", Spec::Full),
    ("Name", Spec::Full),
    ("name", Spec::Full),
]);
const ITEM: Spec = Spec::Fields(&[
    ("Type_", Spec::Full),
    ("Type", Spec::Full),
    ("type", Spec::Full),
    ("Name", Spec::Full),
    ("IconId", Spec::Full),
    ("ItemIconGroupId", Spec::Full),
]);
const DIFFICULTY: Spec = Spec::Fields(&[("Grade", Spec::Full), ("grade", Spec::Full)]);
/// `ArchiveComp`: category -> inner list -> `[{key, value}]` or `{key: value}`;
/// the keys are the ids and the lengths are the counts.
const ARCHIVE_LIST: Spec = Spec::Kv(&STUB);
const ARCHIVE_CATEGORY: Spec = Spec::Each(&ARCHIVE_LIST);
const ARCHIVE: Spec = Spec::Each(&ARCHIVE_CATEGORY);
const DETAIL: Spec = Spec::Fields(&[
    ("Endings", SHAPE),
    ("endings", SHAPE),
    ("Challenges", SHAPE),
    ("challenges", SHAPE),
    ("MonthSquad", SHAPE),
    ("monthSquad", SHAPE),
    ("Milestones", SHAPE),
    ("milestones", SHAPE),
    ("Items", Spec::Kv(&ITEM)),
    ("items", Spec::Kv(&ITEM)),
    ("Difficulties", Spec::Each(&DIFFICULTY)),
    ("difficulties", Spec::Each(&DIFFICULTY)),
    ("ArchiveComp", ARCHIVE),
    ("archiveComp", ARCHIVE),
    ("AutoSetKV", Spec::Full),
    ("DetailConst", Spec::Fields(&[("AutoSetKV", Spec::Full)])),
    ("Stages", Spec::Each(&STAGE_ENTRY)),
]);
const ROOT: Spec = Spec::Fields(&[
    ("Topics", Spec::Full),
    ("topics", Spec::Full),
    ("Details", Spec::Kv(&DETAIL)),
    ("details", Spec::Kv(&DETAIL)),
]);

impl Spec {
    /// The spec of `key` inside an object kept under `self`, `None` for a key
    /// the spec does not name.
    fn field(self, key: &str) -> Option<Self> {
        match self {
            Self::Fields(list) => list.iter().find(|(k, _)| *k == key).map(|&(_, s)| s),
            Self::KvEntry(inner) => match key {
                "key" => Some(Self::Full),
                "value" => Some(*inner),
                _ => None,
            },
            Self::Each(inner) | Self::Kv(inner) => Some(*inner),
            Self::Full | Self::Drop | Self::Stub => None,
        }
    }
}

/// What a visited value hands its parent.
enum Kept {
    /// Nothing worth keeping (only under [`Spec::Drop`]).
    Nothing,
    /// A scalar under [`Spec::Drop`]: kept only as a triple field.
    Leaf(Value),
    /// A value to keep.
    Tree(Value),
}

impl Kept {
    fn into_value(self) -> Value {
        match self {
            Self::Nothing => Value::Null,
            Self::Leaf(v) | Self::Tree(v) => v,
        }
    }
}

#[derive(Clone, Copy)]
struct Prune(Spec);

impl<'de> DeserializeSeed<'de> for Prune {
    type Value = Kept;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Kept, D::Error> {
        match self.0 {
            Spec::Full => Value::deserialize(deserializer).map(Kept::Tree),
            _ => deserializer.deserialize_any(self),
        }
    }
}

impl Prune {
    /// A scalar: a string kept as is, anything else as `null` (every reader
    /// of a pruned scalar asks for a string).
    fn scalar(self, value: Value) -> Kept {
        match self.0 {
            Spec::Drop => Kept::Leaf(value),
            _ => Kept::Tree(value),
        }
    }

    /// A container whose spec wants a placeholder rather than its shape.
    const fn placeholder(self) -> bool {
        matches!(self.0, Spec::Drop | Spec::Stub)
    }
}

impl<'de> Visitor<'de> for Prune {
    type Value = Kept;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_bool<E>(self, _: bool) -> Result<Kept, E> {
        Ok(self.scalar(Value::Null))
    }
    fn visit_i64<E>(self, _: i64) -> Result<Kept, E> {
        Ok(self.scalar(Value::Null))
    }
    fn visit_u64<E>(self, _: u64) -> Result<Kept, E> {
        Ok(self.scalar(Value::Null))
    }
    fn visit_f64<E>(self, _: f64) -> Result<Kept, E> {
        Ok(self.scalar(Value::Null))
    }
    fn visit_unit<E>(self) -> Result<Kept, E> {
        Ok(self.scalar(Value::Null))
    }
    fn visit_none<E>(self) -> Result<Kept, E> {
        Ok(self.scalar(Value::Null))
    }
    fn visit_str<E>(self, v: &str) -> Result<Kept, E> {
        Ok(self.scalar(Value::String(v.to_owned())))
    }
    fn visit_string<E>(self, v: String) -> Result<Kept, E> {
        Ok(self.scalar(Value::String(v)))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Kept, A::Error> {
        // `Each` and `Kv` keep every element (the length is read); anything
        // else keeps only the elements with a level triple inside.
        let element = match self.0 {
            Spec::Each(inner) => Some(*inner),
            Spec::Kv(inner) => Some(Spec::KvEntry(inner)),
            _ => None,
        };
        let mut out = Vec::new();
        match element {
            Some(spec) => {
                while let Some(child) = seq.next_element_seed(Prune(spec))? {
                    out.push(child.into_value());
                }
            }
            None => {
                while let Some(child) = seq.next_element_seed(Prune(Spec::Drop))? {
                    if let Kept::Tree(v) = child {
                        out.push(v);
                    }
                }
            }
        }
        Ok(self.container(Value::Array(out), element.is_some()))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Kept, A::Error> {
        let shaped = !self.placeholder();
        let mut out = Map::new();
        // Scalars under keys the spec does not name, kept only if the object
        // turns out to carry a level id.
        let mut triple_fields: Vec<(String, Value)> = Vec::new();
        let mut has_level_id = false;
        while let Some(key) = map.next_key::<String>()? {
            // A repeated key replaces the earlier value in a `Value`.
            out.remove(&key);
            triple_fields.retain(|(k, _)| *k != key);
            has_level_id |= key == "LevelId" || key == "levelId";
            let named = if shaped { self.0.field(&key) } else { None };
            match named {
                Some(spec) => {
                    let child = map.next_value_seed(Prune(spec))?;
                    out.insert(key, child.into_value());
                }
                None => match map.next_value_seed(Prune(Spec::Drop))? {
                    Kept::Tree(v) => {
                        out.insert(key, v);
                    }
                    other if TRIPLE_KEYS.contains(&key.as_str()) => {
                        triple_fields.push((key, other.into_value()));
                    }
                    _ => {}
                },
            }
        }
        if has_level_id {
            for (key, value) in triple_fields {
                out.insert(key, value);
            }
        }
        Ok(self.container(Value::Object(out), shaped))
    }
}

impl Prune {
    /// A visited container: kept as is when its spec keeps the shape, else
    /// kept only when something inside it was.
    fn container(self, value: Value, shaped: bool) -> Kept {
        let empty = match &value {
            Value::Array(a) => a.is_empty(),
            Value::Object(m) => m.is_empty(),
            _ => true,
        };
        match self.0 {
            _ if shaped => Kept::Tree(value),
            Spec::Drop if empty => Kept::Nothing,
            Spec::Stub if empty => Kept::Tree(Value::Null),
            _ => Kept::Tree(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::prune_value_reader;

    fn prune(doc: &serde_json::Value) -> serde_json::Value {
        prune_value_reader(doc.to_string().as_bytes()).expect("prune")
    }

    #[test]
    fn details_keep_only_what_the_readers_read() {
        let doc = json!({
            "Topics": [{"key": "rogue_1", "value": {"Name": "Phantom", "Sort": 1, "LineText": "x"}}],
            "Modules": [{"key": "rogue_1", "value": {"big": [1, 2, 3]}}],
            "Details": [{"key": "rogue_1", "value": {
                "Endings": [{"key": "e1", "value": {"Desc": "long"}}, {"key": "e2", "value": {}}],
                "Milestones": [{"Level": 1}, {"Level": 2}, {"Level": 3}],
                "Items": [{"key": "i1", "value": {"Type_": "RELIC", "Name": "Cup", "Description": "long", "IconId": "c"}}],
                "Difficulties": [{"Grade": 3, "Desc": "x"}],
                "ArchiveComp": {"Relic": {"Relic": [{"key": "r1", "value": {"Desc": "long"}}]}},
                "Stages": [{"key": "s1", "value": {"Id": "ro1_n_1_1", "LevelId": "obt/x", "Code": "1-1", "Name": "N", "Description": "long"}}],
                "Choices": [{"key": "c1", "value": {"Title": "long"}}]
            }}]
        });
        assert_eq!(
            prune(&doc),
            json!({
                "Topics": [{"key": "rogue_1", "value": {"Name": "Phantom", "Sort": 1, "LineText": "x"}}],
                "Details": [{"key": "rogue_1", "value": {
                    "Endings": [null, null],
                    "Milestones": [null, null, null],
                    "Items": [{"key": "i1", "value": {"Type_": "RELIC", "Name": "Cup", "IconId": "c"}}],
                    "Difficulties": [{"Grade": 3}],
                    "ArchiveComp": {"Relic": {"Relic": [{"key": "r1", "value": null}]}},
                    "Stages": [{"key": "s1", "value": {"Id": "ro1_n_1_1", "LevelId": "obt/x", "Code": "1-1", "Name": "N"}}]
                }}]
            })
        );
    }

    #[test]
    fn a_level_id_anywhere_keeps_its_triple_and_the_path_to_it() {
        let doc = json!({
            "Details": [{"key": "rogue_9", "value": {
                "Zones": [
                    {"key": "z1", "value": {"Nodes": [{"levelId": "obt/a", "name": "A", "Code": 7, "Desc": "x"}]}},
                    {"key": "z2", "value": {"Name": "no level id here"}}
                ]
            }}],
            "Constant": {"deep": {"LevelId": "obt/b", "Code": "B-1"}}
        });
        assert_eq!(
            prune(&doc),
            json!({
                "Details": [{"key": "rogue_9", "value": {
                    "Zones": [{"value": {"Nodes": [{"levelId": "obt/a", "name": "A", "Code": null}]}}]
                }}],
                "Constant": {"deep": {"LevelId": "obt/b", "Code": "B-1"}}
            })
        );
    }

    #[test]
    fn dict_forms_keep_their_keys_and_a_repeated_key_replaces() {
        let doc = json!({
            "details": {"rogue_1": {
                "items": {"b1": {"type": "BAND", "name": "x"}},
                "endings": {"e1": {"x": 1}, "e2": {"x": 2}}
            }}
        });
        assert_eq!(
            prune(&doc),
            json!({"details": {"rogue_1": {
                "items": {"b1": {"type": "BAND"}},
                "endings": {"e1": null, "e2": null}
            }}})
        );
        let repeated = br#"{"Topics": 1, "Topics": [2]}"#;
        assert_eq!(
            prune_value_reader(&repeated[..]).expect("prune"),
            json!({"Topics": [2]})
        );
    }

    #[test]
    fn trailing_content_is_rejected() {
        assert!(prune_value_reader(&b"{} {}"[..]).is_err());
    }
}
