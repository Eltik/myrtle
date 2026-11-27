use crate::scoring::ledger::{MAX_POOL_ROUNDS, POOL_EPS};
use std::collections::HashMap;

#[test]
fn conversion_chain_relaxes_to_fixed_point() {
    let mut points: HashMap<String, f64> = HashMap::from([("A".into(), 17.0)]);
    let converts = vec![
        ("A".to_string(), "B".to_string(), 5.0),
        ("B".to_string(), "C".to_string(), 3.0),
    ];
    for _ in 0..MAX_POOL_ROUNDS {
        let snapshot = points.clone();
        let mut moved = 0.0f64;
        for (from, to, ratio) in &converts {
            let available = snapshot.get(from).copied().unwrap_or(0.0);
            let converted = (available / ratio).floor();
            if converted > 0.0 {
                *points.entry(from.clone()).or_insert(0.0) -= converted * ratio;
                *points.entry(to.clone()).or_insert(0.0) += converted;
                moved += converted;
            }
        }
        if moved < POOL_EPS {
            break;
        }
    }
    assert!((points["A"] - 2.0).abs() < 1e-9, "A remainder: {points:?}");
    assert!((points["B"] - 0.0).abs() < 1e-9, "B drained: {points:?}");
    assert!((points["C"] - 1.0).abs() < 1e-9, "C settled: {points:?}");
}
