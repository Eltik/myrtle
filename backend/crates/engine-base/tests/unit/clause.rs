use crate::registry::BuffResolutionStrategy;
use crate::scoring::clause::*;
use myrtle_gamedata_raw::tables::building::Buff;

fn buff(room: &str, targets: &[&str]) -> Buff {
    Buff {
        room_type: room.to_string(),
        targets: targets.iter().map(|s| (*s).to_string()).collect(),
        ..Default::default()
    }
}

#[test]
fn direct_efficiency_maps_to_a_self_clause() {
    let b = buff("TRADING", &[]);
    let set = clauses_from_strategy(
        "t",
        &b,
        &BuffResolutionStrategy::DirectEfficiency { value: 30.0 },
    );
    assert_eq!(set.len(), 1);
    assert_eq!(set[0].kind, ClauseKind::SelfValue);
    assert_eq!(set[0].metric, Metric::TradingSpeed);
    assert!((set[0].value - 30.0).abs() < 1e-9);
}

#[test]
fn automation_emits_scaling_plus_exempt_suppression() {
    let b = buff("MANUFACTURE", &[]);
    let set = clauses_from_strategy(
        "w",
        &b,
        &BuffResolutionStrategy::FacilityCountScaling {
            target_room: "POWER".into(),
            per_unit_pct: 5.0,
            per_level: false,
            nullifies_others: true,
            base_pct: 0.0,
            cap_pct: None,
        },
    );
    assert_eq!(set.len(), 2);
    assert_eq!(
        set[0].kind,
        ClauseKind::ScalingRoomCount {
            room: "POWER".into()
        }
    );
    assert_eq!(
        set[1].kind,
        ClauseKind::SuppressesOthers {
            metrics: vec![Metric::ManufactureSpeed],
            exempt: SuppressExempt::RoomCountScaledSources,
        }
    );
}

#[test]
fn quartz_style_base_plus_scaling_emits_two_clauses() {
    let b = buff("TRADING", &[]);
    let set = clauses_from_strategy(
        "q",
        &b,
        &BuffResolutionStrategy::FacilityCountScaling {
            target_room: "MANUFACTURE_RECIPE_TYPES".into(),
            per_unit_pct: 2.0,
            per_level: false,
            nullifies_others: false,
            base_pct: 30.0,
            cap_pct: None,
        },
    );
    assert_eq!(set.len(), 2);
    assert_eq!(set[0].kind, ClauseKind::SelfValue);
    assert!((set[0].value - 30.0).abs() < 1e-9);
    assert_eq!(
        set[1].kind,
        ClauseKind::ScalingRoomCount {
            room: "MANUFACTURE_RECIPE_TYPES".into()
        }
    );
}

#[test]
fn shamare_emits_per_body_scaling_plus_suppression_with_value_survival() {
    let b = buff("TRADING", &[]);
    let set = clauses_from_strategy(
        "s",
        &b,
        &BuffResolutionStrategy::NullifyTeammatesSelfScaling {
            per_teammate_pct: 45.0,
        },
    );
    assert_eq!(set.len(), 2);
    assert_eq!(
        set[0].kind,
        ClauseKind::ScalingCount {
            subject: Subject::AnyOtherOccupant,
            include_self: false
        }
    );
    let ClauseKind::SuppressesOthers { metrics, exempt } = &set[1].kind else {
        panic!("expected suppression, got {:?}", set[1].kind);
    };
    assert!(metrics.contains(&Metric::TradingSpeed));
    assert!(metrics.contains(&Metric::OrderValue { pure_gold: true }));
    assert!(!metrics.contains(&Metric::OrderValue { pure_gold: false }));
    assert_eq!(*exempt, SuppressExempt::None);
}

#[test]
fn hoederer_emits_base_plus_base_wide_requirement() {
    let b = buff("TRADING", &[]);
    let set = clauses_from_strategy(
        "h",
        &b,
        &BuffResolutionStrategy::ConditionalOnBaseWide {
            required_char_ids: vec!["char_4087_ines".into(), "char_113_cqbw".into()],
            base_efficiency: 30.0,
            bonus_efficiency: 5.0,
            anywhere: false,
        },
    );
    assert_eq!(set.len(), 2);
    assert_eq!(set[0].kind, ClauseKind::SelfValue);
    assert_eq!(
        set[1].kind,
        ClauseKind::RequiresChar {
            chars: vec!["char_4087_ines".into(), "char_113_cqbw".into()],
            scope: CondScope::BaseWorkArea,
        }
    );
}

#[test]
fn unresolved_teammate_name_emits_a_gate_that_never_fires() {
    let b = buff("TRADING", &[]);
    let set = clauses_from_strategy(
        "x",
        &b,
        &BuffResolutionStrategy::ConditionalOnTeammate {
            required_char_id: None,
            base_efficiency: 20.0,
            efficiency: 25.0,
            order_limit: 0,
        },
    );
    assert_eq!(set.len(), 2);
    let ClauseKind::RequiresChar { chars, .. } = &set[1].kind else {
        panic!("expected RequiresChar");
    };
    assert!(chars.is_empty(), "unresolved names must never fire");
}

#[test]
fn bubble_tiers_decompose_into_two_counts() {
    let b = buff("MANUFACTURE", &[]);
    let set = clauses_from_strategy(
        "b",
        &b,
        &BuffResolutionStrategy::CapacityTierScaling {
            threshold: 4,
            low_pct: 1.0,
            high_pct: 3.0,
            excludes: vec![],
        },
    );
    assert_eq!(set.len(), 2);
    assert!(
        (set[0].value - 1.0).abs() < 1e-9,
        "low per point at or below"
    );
    assert!((set[1].value - 3.0).abs() < 1e-9, "high per point above");
    assert!(matches!(
        &set[0].kind,
        ClauseKind::ScalingCount {
            subject: Subject::CapacityPoints { above: false, .. },
            include_self: true
        }
    ));
    assert!(matches!(
        &set[1].kind,
        ClauseKind::ScalingCount {
            subject: Subject::CapacityPoints { above: true, .. },
            include_self: true
        }
    ));
}

#[test]
fn degenbrecher_negative_capacity_carries_sign() {
    let b = buff("TRADING", &[]);
    let set = clauses_from_strategy(
        "d",
        &b,
        &BuffResolutionStrategy::EfficiencyWithOrderLimit {
            efficiency: 25.0,
            order_limit: -6,
        },
    );
    assert_eq!(set.len(), 2);
    assert_eq!(set[1].metric, Metric::CapacityLimit);
    assert!((set[1].value - (-6.0)).abs() < 1e-9);
}

#[test]
fn order_value_metric_combines_through_the_order_mix() {
    assert_eq!(
        Metric::OrderValue { pure_gold: false }.combine(),
        CombineRule::OrderMix
    );
    assert_eq!(Metric::TradingSpeed.combine(), CombineRule::Sum);
}

#[test]
fn formula_targets_flow_into_output_targets() {
    let b = buff("MANUFACTURE", &["F_GOLD"]);
    let set = clauses_from_strategy(
        "g",
        &b,
        &BuffResolutionStrategy::DirectEfficiency { value: 30.0 },
    );
    assert_eq!(set[0].output_targets, vec!["F_GOLD".to_string()]);
}

#[test]
fn a_time_ramp_averages_over_the_tenure_in_whole_hours() {
    assert!((ramp_average(2.0, 2.0, 20.0, 12.0) - 12.5).abs() < 1e-9);
    assert!((ramp_average(2.0, 2.0, 20.0, 24.0) - 16.25).abs() < 1e-9);
    assert!((ramp_average(2.0, 2.0, 20.0, 1.0) - 2.0).abs() < 1e-9);
    assert!((ramp_average(15.0, 2.0, 25.0, 24.0) - 23.75).abs() < 1e-9);
    assert!((ramp_average(2.0, 2.0, 20.0, 1000.0) - 20.0).abs() < 0.1);
    assert!((ramp_average(2.0, 2.0, 20.0, 0.0) - 2.0).abs() < 1e-9);
}
