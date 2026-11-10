use crate::scoring::clause::{Clause, ClauseKind, Metric};
use crate::scoring::ledger::*;

#[test]
fn config_factor_zeroes_mechanical_mismatches_only() {
    let mk = |targets: &[&str]| Clause {
        buff_id: "b".into(),
        owner_room_type: "MANUFACTURE".into(),
        metric: Metric::ManufactureSpeed,
        kind: ClauseKind::SelfValue,
        value: 10.0,
        cap: None,
        output_targets: targets.iter().map(|s| (*s).to_string()).collect(),
        non_stacking_family: None,
    };
    assert!((config_factor(&mk(&[]), Some("F_GOLD")) - 1.0).abs() < 1e-9);
    assert!((config_factor(&mk(&["F_GOLD"]), Some("F_GOLD")) - 1.0).abs() < 1e-9);
    assert!(config_factor(&mk(&["F_EXP"]), Some("F_GOLD")).abs() < 1e-9);
    assert!((config_factor(&mk(&["F_EXP"]), None) - UNCONFIGURED_OUTPUT_FACTOR).abs() < 1e-9);
    assert!((config_factor(&mk(&["F_GOLD", "F_EXP", "F_DIAMOND"]), None) - 1.0).abs() < 1e-9);
    assert!(
        (config_factor(&mk(&["F_GOLD", "F_EXP", "F_DIAMOND"]), Some("F_DIAMOND")) - 1.0).abs()
            < 1e-9
    );
}
