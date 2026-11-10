#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::path::Path;

use myrtle_dps_formulas::operator_unit::OperatorParams;
use myrtle_dps_formulas::tables::load_heal_formulas;
use myrtle_engine_dps::calculate_hps;

#[test]
fn hps_dump() {
    let Ok(dump_path) = std::env::var("HPS_DUMP") else {
        println!("HPS_DUMP unset, skipping");
        return;
    };
    let excel = myrtle_testkit::real_data::excel_dir();
    if !excel.join("character_table.json").exists() {
        println!("no game data at {}, skipping", excel.display());
        return;
    }
    let operators = myrtle_testkit::real_data::dps_operators(&excel).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/expected_hps.json");
    let expected: HashMap<String, serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(fixture).unwrap()).unwrap();
    let dump_all = std::env::var("HPS_DUMP_ALL").is_ok_and(|v| v == "1");

    let mut dump: Vec<String> = Vec::new();
    for (char_id, formula) in &load_heal_formulas() {
        let Some(operator) = operators.get(char_id) else {
            continue;
        };
        if operator.skills.is_empty() {
            continue;
        }
        for &skill in &formula.available_skills {
            for module in std::iter::once(0).chain(formula.available_modules.iter().copied()) {
                let key = format!("{}_s{skill}_m{module}", formula.class_name);
                if !dump_all && !expected.contains_key(&key) {
                    continue;
                }
                let params = OperatorParams {
                    skill_index: Some(skill),
                    module_index: Some(module),
                    ..Default::default()
                };
                dump.push(match calculate_hps(operator, params) {
                    Some(r) => format!(
                        "{key}\t{:?}\t{:?}\t{:?}",
                        r.skill_hps, r.base_hps, r.avg_hps
                    ),
                    None => format!("{key}\tNone"),
                });
            }
        }
    }
    dump.sort();
    std::fs::write(&dump_path, dump.join("\n") + "\n").unwrap();
    println!("  wrote {} cases to {dump_path}", dump.len());
}
