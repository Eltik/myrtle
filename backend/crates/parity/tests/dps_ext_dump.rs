#![allow(clippy::unwrap_used, clippy::cast_possible_truncation)]

use std::fmt::Write as _;
use std::sync::Arc;

use myrtle_dps_formulas::operator_data::OperatorData;
use myrtle_dps_formulas::operator_unit::{EnemyStats, OperatorParams};
use myrtle_dps_formulas::tables::supported_operators;
use myrtle_engine_dps::zones::BuffZones;
use myrtle_engine_dps::{CycleModel, DpsRequest, SecondaryTarget, calculate_dps_with};
use serde_json::Value;

#[test]
fn dps_ext_dump() {
    let Ok(out) = std::env::var("DPS_EXT_DUMP") else {
        println!("DPS_EXT_DUMP unset, skipping");
        return;
    };
    let sc: Value = serde_json::from_slice(
        &std::fs::read(std::env::var("DPS_EXT_SCENARIOS").unwrap()).unwrap(),
    )
    .unwrap();
    let excel = myrtle_testkit::real_data::excel_dir();
    let operators = myrtle_testkit::real_data::dps_operators(&excel).unwrap();
    let zones: Vec<Option<BuffZones>> = sc["zones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|z| serde_json::from_value(z.clone()).unwrap())
        .collect();
    let secondary: Vec<Option<SecondaryTarget>> = sc["secondary"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (!s.is_null()).then(|| SecondaryTarget {
                defense: s["defense"].as_f64().unwrap(),
                res: s["res"].as_f64().unwrap(),
                count: s["count"].as_i64().unwrap() as i32,
            })
        })
        .collect();
    let enemies: Vec<(f64, f64)> = sc["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| (e[0].as_f64().unwrap(), e[1].as_f64().unwrap()))
        .collect();
    let targets: Vec<i32> = sc["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_i64().unwrap() as i32)
        .collect();
    let mut ops: Vec<_> = supported_operators().iter().collect();
    ops.sort_by(|a, b| a.0.cmp(b.0));
    let mut s = String::new();
    for (id, f) in ops {
        let Some(op) = operators.get(id) else {
            continue;
        };
        if op.skills.is_empty() {
            continue;
        }
        let data = Arc::new(OperatorData::new(op.clone()));
        for &skill in &f.available_skills {
            if usize::try_from(skill).is_ok_and(|k| k > op.skills.len()) {
                continue;
            }
            for (zi, z) in zones.iter().enumerate() {
                for (si, sec) in secondary.iter().enumerate() {
                    for (ci, cm) in [CycleModel::Legacy, CycleModel::V2].into_iter().enumerate() {
                        for &(d, r) in &enemies {
                            for &t in &targets {
                                let request = DpsRequest {
                                    params: OperatorParams {
                                        skill_index: Some(skill),
                                        targets: Some(t),
                                        ..Default::default()
                                    },
                                    zones: z.clone(),
                                    secondary: *sec,
                                    cycle_model: cm,
                                    clone_operator: None,
                                };
                                let key = format!("{id}_s{skill}_z{zi}_x{si}_c{ci}_d{d}_r{r}_t{t}");
                                let enemy = EnemyStats { defense: d, res: r };
                                match calculate_dps_with(Arc::clone(&data), request, &enemy) {
                                    Some(x) => writeln!(
                                        s,
                                        "{key}\t{:016x}\t{:016x}\t{:016x}",
                                        x.skill_dps.to_bits(),
                                        x.total_damage.to_bits(),
                                        x.average_dps.to_bits()
                                    )
                                    .unwrap(),
                                    None => writeln!(s, "{key}\tNone").unwrap(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    std::fs::write(out, s).unwrap();
}
