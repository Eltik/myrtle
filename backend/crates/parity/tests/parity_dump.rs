#![allow(clippy::float_cmp, clippy::too_many_lines, clippy::unwrap_used)]

use std::collections::HashMap;
use std::path::Path;

use myrtle_dps_formulas::operator_unit::{
    EnemyStats, OperatorBuffs, OperatorParams, OperatorShred,
};
use myrtle_dps_formulas::tables::load_formulas;
use myrtle_engine_dps::{DpsRequest, calculate_dps};

#[allow(clippy::too_many_arguments)]
fn make_test_key(
    operator_name: &str,
    skill: i32,
    module: i32,
    defense: f64,
    res: f64,
    fragile: f64,
    def_shred_mult: f64,
    def_shred_flat: f64,
    res_shred_mult: f64,
    res_shred_flat: f64,
) -> String {
    let base = format!("{operator_name}_s{skill}_m{module}_{defense:.0}_{res:.0}");
    if fragile != 0.0
        || def_shred_mult != 1.0
        || res_shred_mult != 1.0
        || def_shred_flat != 0.0
        || res_shred_flat != 0.0
    {
        format!(
            "{}_db{}_{}_{}_{}_{}",
            base,
            (fragile * 100.0) as i32,
            (def_shred_mult * 100.0) as i32,
            def_shred_flat as i32,
            (res_shred_mult * 100.0) as i32,
            res_shred_flat as i32,
        )
    } else {
        base
    }
}

#[test]
fn parity_dump() {
    let Ok(dump_path) = std::env::var("DPS_DUMP") else {
        println!("DPS_DUMP unset, skipping");
        return;
    };
    let excel = myrtle_testkit::real_data::excel_dir();
    if !excel.join("character_table.json").exists() {
        println!("no game data at {}, skipping", excel.display());
        return;
    }
    let operators = myrtle_testkit::real_data::dps_operators(&excel).unwrap();

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/expected_dps.json");
    let expected: HashMap<String, f64> =
        serde_json::from_str(&std::fs::read_to_string(fixture).unwrap()).unwrap();
    let formulas = load_formulas();

    let defense_values = [0.0, 300.0, 500.0, 1000.0];
    let res_values = [0.0, 20.0, 30.0, 50.0];
    let debuff_scenarios: &[(f64, f64, f64, f64, f64)] = &[
        (0.0, 1.0, 0.0, 1.0, 0.0),
        (0.3, 1.0, 0.0, 1.0, 0.0),
        (0.0, 0.6, 100.0, 1.0, 0.0),
        (0.0, 1.0, 0.0, 0.7, 15.0),
    ];

    let dump_all = std::env::var("DPS_DUMP_ALL").is_ok_and(|v| v == "1");
    let mut dump: Vec<String> = Vec::new();
    for (char_id, formula) in &formulas {
        let Some(operator) = operators.get(char_id) else {
            continue;
        };
        for &skill in &formula.available_skills {
            if operator.skills.is_empty() || formula.class_name == "Muelsyse" {
                continue;
            }
            if usize::try_from(skill).is_ok_and(|s| s > operator.skills.len()) {
                continue;
            }
            let prefixes = ["uniequip_002_", "uniequip_003_", "uniequip_004_"];
            let all_modules_exist = formula.available_modules.iter().all(|&m| {
                let pos = formula.available_modules.iter().position(|&v| v == m);
                pos.and_then(|p| prefixes.get(p)).is_some_and(|prefix| {
                    operator.modules.iter().any(|om| {
                        om.module
                            .id
                            .as_deref()
                            .is_some_and(|id| id.starts_with(prefix))
                    })
                })
            });
            for module in std::iter::once(0).chain(formula.available_modules.iter().copied()) {
                if module > 0 && !all_modules_exist {
                    continue;
                }
                for &def in &defense_values {
                    for &res in &res_values {
                        for &(fragile, def_mult, def_flat, res_mult, res_flat) in debuff_scenarios {
                            let key = make_test_key(
                                &formula.class_name,
                                skill,
                                module,
                                def,
                                res,
                                fragile,
                                def_mult,
                                def_flat,
                                res_mult,
                                res_flat,
                            );
                            if !dump_all && !expected.contains_key(&key) {
                                continue;
                            }
                            let shred = if def_mult != 1.0
                                || def_flat != 0.0
                                || res_mult != 1.0
                                || res_flat != 0.0
                            {
                                Some(OperatorShred {
                                    def: Some(((1.0 - def_mult) * 100.0).round() as i32),
                                    def_flat: Some(def_flat as i32),
                                    res: Some(((1.0 - res_mult) * 100.0).round() as i32),
                                    res_flat: Some(res_flat as i32),
                                })
                            } else {
                                None
                            };
                            let params = OperatorParams {
                                skill_index: Some(skill),
                                module_index: Some(module),
                                buffs: OperatorBuffs {
                                    fragile: if fragile == 0.0 {
                                        None
                                    } else {
                                        Some(fragile as f32)
                                    },
                                    ..Default::default()
                                },
                                shred,
                                ..Default::default()
                            };
                            let enemy = EnemyStats { defense: def, res };
                            let result = calculate_dps(operator, DpsRequest::from(params), &enemy);
                            dump.push(match &result {
                                Some(r) => format!(
                                    "{key}\t{:?}\t{:?}\t{:?}",
                                    r.skill_dps, r.total_damage, r.average_dps
                                ),
                                None => format!("{key}\tNone"),
                            });
                        }
                    }
                }
            }
        }
    }
    dump.sort();
    std::fs::write(&dump_path, dump.join("\n") + "\n").unwrap();
    println!("  wrote {} cases to {dump_path}", dump.len());
}
