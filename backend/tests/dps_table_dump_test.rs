//! Whole-table dump: every DPS and HPS formula operator, not only the ones the
//! Python fixture covers.
//!
//! The parity suites dump the fixture's cases only, so an operator with no
//! Python class (a hand-layer operator such as Angelina the Mellow Wish) or one
//! the grid skips (Muelsyse, CN-only skills) is invisible to a bit-identity
//! check. This test runs every formula operator in game data at every skill
//! (basic attack included) and module the table lists, against three enemies,
//! with legacy defaults, and writes one sorted line per case:
//!
//! `dps|<char_id>|s<skill>|m<module>|<def>|<res>\t<skill_dps>\t<total>\t<average>`
//! `hps|<char_id>|s<skill>|m<module>\t<skill_hps>\t<base_hps>\t<avg_hps>`
//!
//! Run (ignored by default, needs game data):
//! `DPS_TABLE_DUMP=/path/out.tsv cargo test -j 3 --test dps_table_dump_test -- --ignored`
//! Compare two dumps with `cmp`; `diff` names the cases that moved.

// One `match` per case reads better than `map_or_else` with two closures.
#![allow(clippy::option_if_let_else)]

use backend::core::gamedata;
use backend::dps::engine;
use backend::dps::operator_unit::{EnemyStats, OperatorParams};
use std::path::Path;

#[test]
#[ignore = "needs game data; writes DPS_TABLE_DUMP"]
fn dump_every_formula_operator() {
    dotenv::dotenv().ok();
    let data_dir_str = std::env::var("GAME_DATA_DIR")
        .unwrap_or_else(|_| "../assets/output/en/gamedata/excel".into());
    let assets_dir_str =
        std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
    let (game_data, _assets) =
        gamedata::init_game_data(Path::new(&data_dir_str), Path::new(&assets_dir_str))
            .expect("Failed to load game data");

    let enemies = [(0.0, 0.0), (500.0, 30.0), (1000.0, 50.0)];
    let mut dump: Vec<String> = Vec::new();
    let mut missing: Vec<String> = Vec::new();

    for (char_id, formula) in engine::supported_operators() {
        let Some(operator) = game_data.operators.get(char_id) else {
            missing.push(format!("dps|{char_id}"));
            continue;
        };
        let skills = std::iter::once(0).chain(formula.available_skills.iter().copied());
        for skill in skills {
            for module in std::iter::once(0).chain(formula.available_modules.iter().copied()) {
                for &(defense, res) in &enemies {
                    let params = OperatorParams {
                        skill_index: Some(skill),
                        module_index: Some(module),
                        ..Default::default()
                    };
                    let key = format!("dps|{char_id}|s{skill}|m{module}|{defense}|{res}");
                    let enemy = EnemyStats { defense, res };
                    dump.push(match engine::calculate_dps(operator, params, &enemy) {
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

    for (char_id, formula) in engine::supported_healers() {
        let Some(operator) = game_data.operators.get(char_id) else {
            missing.push(format!("hps|{char_id}"));
            continue;
        };
        for &skill in &formula.available_skills {
            for module in std::iter::once(0).chain(formula.available_modules.iter().copied()) {
                let params = OperatorParams {
                    skill_index: Some(skill),
                    module_index: Some(module),
                    ..Default::default()
                };
                let key = format!("hps|{char_id}|s{skill}|m{module}");
                dump.push(match engine::calculate_hps(operator, params) {
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
    missing.sort();
    println!(
        "  {} cases; {} formula operators not in game data: {missing:?}",
        dump.len(),
        missing.len()
    );
    if let Ok(path) = std::env::var("DPS_TABLE_DUMP") {
        std::fs::write(&path, dump.join("\n") + "\n").expect("write DPS_TABLE_DUMP");
        println!("  wrote {} cases to {path}", dump.len());
    }
}
