//! Missing-formula audit: which playable operators the DPS / HPS engine cannot
//! compute.
//!
//! "Playable" is a `char_` id whose profession is not TOKEN or TRAP and whose
//! `IsNotObtainable` is false, read through the backend's own game-data loader.
//! An operator "has a formula" when the engine's table lists it
//! (`engine::supported_operators` for DPS, `engine::supported_healers` for HPS),
//! hand layer included. A table entry whose body is a transpiler fallback stub
//! (returns `None` for every input) still counts as listed; the stubs are named
//! in the output so they are not mistaken for coverage.
//!
//! Exemptions live in `tests/fixtures/dps_formula_exemptions.json`: operators
//! reviewed as dealing no meaningful damage or healing, each with a reason. It
//! starts empty on purpose; add an id only after reading the operator's kit.
//!
//! Run (needs EN game data, prints the report):
//! `cargo test -j 3 --test dps_formula_audit_test -- --ignored --nocapture`

use backend::core::gamedata;
use backend::core::gamedata::types::operator::{Operator, OperatorProfession};
use backend::dps::engine;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// HPS table entries the generator emits as `None` stubs (`HEAL_FALLBACKS` in
/// `src/bin/generate_dps.rs`): listed in the table, never a number.
const HPS_FALLBACK_STUBS: &[&str] = &["AmiyaMedic", "ReedAlter"];

#[derive(Deserialize)]
struct Exemption {
    id: String,
    reason: String,
}

#[derive(Deserialize)]
struct Exemptions {
    exemptions: Vec<Exemption>,
}

fn load_exemptions() -> HashMap<String, String> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dps_formula_exemptions.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let parsed: Exemptions = serde_json::from_str(&text).expect("dps_formula_exemptions.json");
    parsed
        .exemptions
        .into_iter()
        .map(|e| {
            assert!(
                !e.reason.trim().is_empty(),
                "exemption {} has no reason",
                e.id
            );
            (e.id, e.reason)
        })
        .collect()
}

fn is_playable(id: &str, op: &Operator) -> bool {
    id.starts_with("char_")
        && !matches!(
            op.profession,
            OperatorProfession::Token | OperatorProfession::Trap
        )
        && !op.is_not_obtainable
}

/// `rarity -> class -> [name (id)]`, rarity descending.
fn print_grouped(title: &str, rows: &[(&String, &Operator)]) {
    let mut grouped: BTreeMap<std::cmp::Reverse<i16>, BTreeMap<String, Vec<String>>> =
        BTreeMap::new();
    for (id, op) in rows {
        grouped
            .entry(std::cmp::Reverse(op.rarity.to_star_int()))
            .or_default()
            .entry(format!("{:?}", op.profession))
            .or_default()
            .push(format!("{} ({id})", op.name));
    }
    println!("\n=== {title}: {} ===", rows.len());
    for (rarity, classes) in &grouped {
        let n: usize = classes.values().map(Vec::len).sum();
        println!("  {} star: {n}", rarity.0);
        for (class, names) in classes {
            let mut names = names.clone();
            names.sort();
            println!("    {class} ({}): {}", names.len(), names.join(", "));
        }
    }
}

#[test]
#[ignore = "audit report; needs game data"]
fn missing_formula_audit() {
    dotenv::dotenv().ok();
    let data_dir_str = std::env::var("GAME_DATA_DIR")
        .unwrap_or_else(|_| "../assets/output/en/gamedata/excel".into());
    let assets_dir_str =
        std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
    let (game_data, _assets) =
        gamedata::init_game_data(Path::new(&data_dir_str), Path::new(&assets_dir_str))
            .expect("Failed to load game data");

    let dps = engine::supported_operators();
    let hps = engine::supported_healers();
    let exemptions = load_exemptions();

    let mut playable: Vec<(&String, &Operator)> = game_data
        .operators
        .iter()
        .filter(|(id, op)| is_playable(id, op))
        .collect();
    playable.sort_by(|a, b| a.0.cmp(b.0));

    let dps_count = playable
        .iter()
        .filter(|(id, _)| dps.contains_key(*id))
        .count();
    let hps_count = playable
        .iter()
        .filter(|(id, _)| hps.contains_key(*id))
        .count();
    let stubs: Vec<&str> = hps
        .iter()
        .filter(|(_, f)| HPS_FALLBACK_STUBS.contains(&f.class_name.as_str()))
        .map(|(id, _)| id.as_str())
        .collect();

    let neither: Vec<(&String, &Operator)> = playable
        .iter()
        .filter(|(id, _)| !dps.contains_key(*id) && !hps.contains_key(*id))
        .filter(|(id, _)| !exemptions.contains_key(*id))
        .copied()
        .collect();
    let medics_no_hps: Vec<(&String, &Operator)> = playable
        .iter()
        .filter(|(id, op)| op.profession == OperatorProfession::Medic && !hps.contains_key(*id))
        .filter(|(id, _)| !exemptions.contains_key(*id))
        .copied()
        .collect();

    // Table entries that resolve to no playable operator in this data.
    let mut not_playable: Vec<String> = dps
        .keys()
        .chain(hps.keys())
        .filter(|id| {
            game_data
                .operators
                .get(*id)
                .is_none_or(|op| !is_playable(id, op))
        })
        .cloned()
        .collect();
    not_playable.sort();
    not_playable.dedup();

    println!("playable operators: {}", playable.len());
    println!("  with a DPS formula: {dps_count}");
    println!(
        "  with an HPS formula: {hps_count} (of which fallback stubs, no number: {})",
        stubs
            .iter()
            .filter(|id| playable.iter().any(|(p, _)| p.as_str() == **id))
            .count()
    );
    println!(
        "  with neither (after {} exemptions): {}",
        exemptions.len(),
        neither.len()
    );
    println!("table entries not playable here: {not_playable:?}");
    println!("HPS fallback stubs in the table: {stubs:?}");
    print_grouped("NO DPS AND NO HPS FORMULA", &neither);
    print_grouped("MEDICS WITH NO HPS FORMULA", &medics_no_hps);
    if !exemptions.is_empty() {
        println!("\n=== EXEMPT: {} ===", exemptions.len());
        let mut ex: Vec<_> = exemptions.iter().collect();
        ex.sort();
        for (id, reason) in ex {
            println!("  {id}: {reason}");
        }
    }

    // Every exemption must name a playable operator with no formula, or it is stale.
    for id in exemptions.keys() {
        let op = game_data.operators.get(id);
        assert!(
            op.is_some_and(|op| is_playable(id, op)),
            "exemption {id} is not a playable operator"
        );
        assert!(
            !dps.contains_key(id) && !hps.contains_key(id),
            "exemption {id} now has a formula; remove it"
        );
    }
}
