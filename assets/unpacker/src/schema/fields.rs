//! A line-based reader for the parts of a `.fbs` that the verifier cannot
//! check: the TYPE of each table field and the underlying type of each enum.
//!
//! The verifier accepts a schema whose field changed type at the same size
//! (`int` to `float`, an enum to `int`, one table to another) and the decoder
//! then reads garbage. `schema-sync` compares these against the stored schema
//! and reports every change; see `assets/schemas/README.md`, "The sync".
//!
//! The generator's output and the stored files are regular: one declaration
//! per line, `table NAME {` or `enum NAME : TYPE {` opening a block, `}`
//! closing it, `name: type;` per field. That is all this reads.

use std::collections::BTreeMap;

/// The declarations of one `.fbs`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Fields {
    /// Table name -> its fields in order, `(name, type text)`.
    pub tables: BTreeMap<String, Vec<(String, String)>>,
    /// Enum name -> its underlying type.
    pub enums: BTreeMap<String, String>,
}

/// Parses `src`. Commented-out lines (`// candidates: [...];`), trailing
/// comments, attributes after the type (`(key)`) and default values are
/// ignored; `root_type` and anything outside a table is skipped.
#[must_use]
pub fn parse(src: &str) -> Fields {
    let mut out = Fields::default();
    let mut table: Option<String> = None;
    for raw in src.lines() {
        let line = raw.split("//").next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("table ") {
            let name = rest.split('{').next().unwrap_or("").trim().to_string();
            let fields = out.tables.entry(name.clone()).or_default();
            // `table X {}` on one line opens and closes the block.
            table = if line.contains('}') {
                None
            } else {
                fields.clear();
                Some(name)
            };
            continue;
        }
        if let Some(rest) = line.strip_prefix("enum ") {
            let head = rest.split('{').next().unwrap_or("");
            if let Some((name, under)) = head.split_once(':') {
                out.enums
                    .insert(name.trim().to_string(), under.trim().to_string());
            }
            table = None;
            continue;
        }
        if line.starts_with('}') {
            table = None;
            continue;
        }
        let Some(name) = &table else { continue };
        let Some((field, ty)) = line.split_once(':') else {
            continue;
        };
        let ty = ty.split(';').next().unwrap_or("");
        let ty = ty.split(['(', '=']).next().unwrap_or("").trim();
        let field = field.trim();
        if field.is_empty() || ty.is_empty() {
            continue;
        }
        if let Some(fields) = out.tables.get_mut(name) {
            fields.push((field.to_string(), ty.split_whitespace().collect()));
        }
    }
    out
}

/// Every type change between `old` and `new` for one stem, as
/// `stem: table.field: <old> -> <new>` and `stem: enum NAME: <old> -> <new>`.
/// Fields and enums present in only one side are not reported: additions and
/// removals are normal patch evolution.
#[must_use]
pub fn type_changes(stem: &str, old: &str, new: &str) -> Vec<String> {
    let (old, new) = (parse(old), parse(new));
    let mut out = Vec::new();
    for (table, old_fields) in &old.tables {
        let Some(new_fields) = new.tables.get(table) else {
            continue;
        };
        for (field, old_ty) in old_fields {
            if let Some((_, new_ty)) = new_fields.iter().find(|(f, _)| f == field)
                && new_ty != old_ty
            {
                out.push(format!("{stem}: {table}.{field}: {old_ty} -> {new_ty}"));
            }
        }
    }
    for (name, old_ty) in &old.enums {
        if let Some(new_ty) = new.enums.get(name)
            && new_ty != old_ty
        {
            out.push(format!("{stem}: enum {name}: {old_ty} -> {new_ty}"));
        }
    }
    out
}

/// Every table whose fields SHIFTED slot from `old` to `new`, as
/// `stem: table slot N: <old field> -> <new field>`: a field inserted before
/// the end, a field removed, or two fields swapped. A FlatBuffers slot is the
/// field's position, so a buffer written with `old` and read with `new` hands
/// every field from slot N on its neighbour's value. The verifier accepts that
/// whenever the shifted slots hold defaults, which the buffer leaves out.
///
/// Fields appended at the end, fields dropped from the end, and a rename at the
/// same slot (the rest unchanged) are not shifts and are not reported.
#[must_use]
pub fn slot_shifts(stem: &str, old: &str, new: &str) -> Vec<String> {
    let (old, new) = (parse(old), parse(new));
    let mut out = Vec::new();
    for (table, old_fields) in &old.tables {
        let Some(new_fields) = new.tables.get(table) else {
            continue;
        };
        let n = old_fields.len().min(new_fields.len());
        let Some(i) = (0..n).find(|&i| old_fields[i].0 != new_fields[i].0) else {
            continue;
        };
        // A rename keeps every later field in place and its old name is gone;
        // `[a, b] -> [a, x, b]` keeps nothing in place after slot 1 but moves
        // `b`, so it is an insertion.
        let renamed = old_fields.len() <= new_fields.len()
            && old_fields[i + 1..]
                .iter()
                .zip(&new_fields[i + 1..])
                .all(|(a, b)| a.0 == b.0)
            && !new_fields.iter().any(|(f, _)| *f == old_fields[i].0);
        if renamed {
            continue;
        }
        out.push(format!(
            "{stem}: {table} slot {i}: {} -> {}",
            old_fields[i].0, new_fields[i].0
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_shifts_report_insertions_not_appends_or_renames() {
        let opts = "table clz_Torappu_LevelData_Options {\n    steeringEnabled: bool;\n    \
                    isTrainingLevel: bool;\n    isHardTrainingLevel: bool;\n}\n";
        assert!(slot_shifts("prts___levels", opts, opts).is_empty());
        let appended = opts.replace(
            "    isHardTrainingLevel: bool;\n",
            "    isHardTrainingLevel: bool;\n    maxPlayTime: float;\n",
        );
        assert!(slot_shifts("prts___levels", opts, &appended).is_empty());
        assert!(slot_shifts("prts___levels", &appended, opts).is_empty());
        let renamed = opts.replace("steeringEnabled", "steering");
        assert!(slot_shifts("prts___levels", opts, &renamed).is_empty());
        // The 2.7.71 insertion that EN, JP and KR level files were read with.
        let inserted = opts.replace(
            "    isTrainingLevel: bool;\n",
            "    reachableCheckIgnoreStartTile: bool;\n    isTrainingLevel: bool;\n",
        );
        assert_eq!(
            slot_shifts("prts___levels", opts, &inserted),
            vec![
                "prts___levels: clz_Torappu_LevelData_Options slot 1: \
                 isTrainingLevel -> reachableCheckIgnoreStartTile"
                    .to_string()
            ]
        );
        // The same pair read the other way round is a removal: also a shift.
        assert_eq!(slot_shifts("prts___levels", &inserted, opts).len(), 1);
        // An insertion before the last field is not a rename.
        let before_last = opts.replace(
            "    isHardTrainingLevel: bool;\n",
            "    isPredefinedCardsSelectable: bool;\n    isHardTrainingLevel: bool;\n",
        );
        assert_eq!(slot_shifts("prts___levels", opts, &before_last).len(), 1);
    }

    const ITEM: &str = "// Generated by unpacker schema-gen\n\
enum enum__Torappu_ItemRarity : int {\n    TIER_1 = 0,\n    TIER_2 = 1,\n}\n\n\
table clz_Torappu_ItemData {\n    itemId: string;\n    sortId: int;\n    rarity: enum__Torappu_ItemRarity;\n    \
buildingProductList: [clz_Torappu_BuildingProduct];\n    // candidates: [clz_Torappu_TalentData]; \n    \
professionIds: [enum__Torappu_ProfessionID]; // __prts__check_required__\n}\n\n\
table dict__string__clz_Torappu_ItemData {\n    key: string(key);\n    value: clz_Torappu_ItemData;\n}\n\n\
table WHAT_THE_FUCK_IS_HG_DOING {}\n\n\
root_type clz_Torappu_ItemTable;\n";

    fn pairs(v: &[(&str, &str)]) -> Vec<(String, String)> {
        v.iter()
            .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
            .collect()
    }

    #[test]
    fn parses_tables_enums_vectors_and_skips_comments() {
        let f = parse(ITEM);
        assert_eq!(
            f.tables["clz_Torappu_ItemData"],
            pairs(&[
                ("itemId", "string"),
                ("sortId", "int"),
                ("rarity", "enum__Torappu_ItemRarity"),
                ("buildingProductList", "[clz_Torappu_BuildingProduct]"),
                ("professionIds", "[enum__Torappu_ProfessionID]"),
            ])
        );
        assert_eq!(
            f.tables["dict__string__clz_Torappu_ItemData"],
            pairs(&[("key", "string"), ("value", "clz_Torappu_ItemData")])
        );
        assert!(f.tables["WHAT_THE_FUCK_IS_HG_DOING"].is_empty());
        assert_eq!(f.tables.len(), 3);
        assert_eq!(f.enums["enum__Torappu_ItemRarity"], "int");
        assert_eq!(f.enums.len(), 1);
    }

    #[test]
    fn reports_type_changes_only() {
        assert!(type_changes("item_table", ITEM, ITEM).is_empty());
        let new = ITEM
            .replace("sortId: int;", "sortId: float;")
            .replace("ItemRarity : int", "ItemRarity : short")
            .replace("    itemId: string;\n", "")
            .replace(
                "    rarity: enum__Torappu_ItemRarity;\n",
                "    rarity: enum__Torappu_ItemRarity;\n    added: long;\n",
            )
            .replace("[clz_Torappu_BuildingProduct]", "[clz_Torappu_Other]");
        assert_eq!(
            type_changes("item_table", ITEM, &new),
            vec![
                "item_table: clz_Torappu_ItemData.sortId: int -> float".to_string(),
                "item_table: clz_Torappu_ItemData.buildingProductList: \
                 [clz_Torappu_BuildingProduct] -> [clz_Torappu_Other]"
                    .to_string(),
                "item_table: enum enum__Torappu_ItemRarity: int -> short".to_string(),
            ]
        );
        // Un-commenting a field is an addition, not a change.
        let uncommented = ITEM.replace("// candidates:", "candidates:");
        assert!(type_changes("item_table", ITEM, &uncommented).is_empty());
        assert!(type_changes("item_table", &uncommented, ITEM).is_empty());
    }
}
