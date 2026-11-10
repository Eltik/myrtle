#![allow(clippy::float_cmp)] // exact 1.0 sentinels

use crate::assignment::{SUSTAINED_AFK_DAYS, factory_cap_factor, gold_base_capacity};
use myrtle_gamedata_raw::tables::building::{
    BuildingDataFile, ManufactData, ManufactFormula, ManufactPhase,
};

fn test_building_data() -> BuildingDataFile {
    let mut bd = BuildingDataFile {
        manufact_data: ManufactData {
            basic_speed_buff: 0.01,
            phases: vec![
                ManufactPhase {
                    output_capacity: 24,
                    speed: 1.0,
                },
                ManufactPhase {
                    output_capacity: 36,
                    speed: 1.0,
                },
                ManufactPhase {
                    output_capacity: 54,
                    speed: 1.0,
                },
            ],
        },
        ..Default::default()
    };
    bd.manufact_formulas.insert(
        "4".to_string(),
        ManufactFormula {
            cost_point: 4320,
            ..Default::default()
        },
    );
    bd
}

#[test]
fn cap_factor_throttles_high_throughput_low_capacity_gold() {
    let lvl = 3;
    let high_eff = 110.0;
    let bd = test_building_data();
    let no_cap = factory_cap_factor(&bd, Some("F_GOLD"), lvl, high_eff, 0);
    assert!(
        no_cap < 1.0,
        "a +110% gold factory with no capacity should stall, got {no_cap}"
    );
    let with_cap = factory_cap_factor(&bd, Some("F_GOLD"), lvl, high_eff, 40);
    assert!(
        with_cap > no_cap,
        "adding capacity should reduce the stall ({no_cap} -> {with_cap})"
    );
    assert!((with_cap - 1.0).abs() < 1e-9 || with_cap >= no_cap);
}

#[test]
fn cap_factor_is_neutral_for_low_throughput_and_non_gold() {
    let bd = test_building_data();
    assert!((factory_cap_factor(&bd, Some("F_GOLD"), 3, 10.0, 0) - 1.0).abs() < 1e-9);
    assert_eq!(factory_cap_factor(&bd, Some("F_EXP"), 3, 300.0, 0), 1.0);
    assert_eq!(factory_cap_factor(&bd, None, 3, 300.0, 0), 1.0);
    assert!(SUSTAINED_AFK_DAYS > 0.0 && gold_base_capacity(&bd, 3) > gold_base_capacity(&bd, 1));
}
