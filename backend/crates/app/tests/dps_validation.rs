#![allow(clippy::unwrap_used)]

use myrtle_app::AppError;
use myrtle_app::services::catalog::dps::{
    BatchRequest, CalculateRequest, MAX_BATCH, RankFilters, RankPreset, RankRequest, RequestBuffs,
    RequestCycleModel, RequestSecondary, SupportersQuery, Sweep, SweepAxis,
};
use myrtle_engine_dps::zones::BuffZones;

fn calc(operator_id: &str) -> CalculateRequest {
    serde_json::from_value(serde_json::json!({ "operatorId": operator_id })).unwrap()
}

type Case = (&'static str, fn(&mut CalculateRequest));

fn message(e: AppError) -> String {
    match e {
        AppError::BadRequest(m) => m,
        other => panic!("expected a bad request, got {other:?}"),
    }
}

#[test]
fn a_plain_request_passes() {
    calc("char_2013_cerber").validate().unwrap();
}

#[test]
fn every_integer_bound_is_enforced() {
    let cases: [Case; 10] = [
        ("promotion", |r| r.promotion = Some(3)),
        ("level", |r| r.level = Some(0)),
        ("potential", |r| r.potential = Some(7)),
        ("trust", |r| r.trust = Some(201)),
        ("skillIndex", |r| r.skill_index = Some(4)),
        ("masteryLevel", |r| r.mastery_level = Some(-1)),
        ("skillLevel", |r| r.skill_level = Some(8)),
        ("moduleIndex", |r| r.module_index = Some(-2)),
        ("moduleLevel", |r| r.module_level = Some(4)),
        ("targets", |r| r.targets = Some(51)),
    ];
    for (field, set) in cases {
        let mut req = calc("char_2013_cerber");
        set(&mut req);
        let m = message(req.validate().unwrap_err());
        assert!(m.starts_with(&format!("{field} must be between")), "{m}");
    }
}

#[test]
fn non_finite_and_out_of_range_floats_are_rejected() {
    let mut req = calc("char_2013_cerber");
    req.defense = Some(f64::NAN);
    assert_eq!(
        message(req.validate().unwrap_err()),
        "defense must be a finite number"
    );
    req.defense = Some(100_001.0);
    assert!(message(req.validate().unwrap_err()).starts_with("defense must be between"));
    let mut req = calc("char_2013_cerber");
    req.sp_boost = Some(f32::INFINITY);
    assert_eq!(
        message(req.validate().unwrap_err()),
        "spBoost must be a finite number"
    );
}

#[test]
fn the_aspd_buff_is_capped() {
    let mut req = calc("char_2013_cerber");
    req.buffs = Some(RequestBuffs {
        atk: None,
        flat_atk: None,
        aspd: Some(1_001),
        fragile: None,
    });
    assert!(message(req.validate().unwrap_err()).starts_with("buffs.aspd"));
}

#[test]
fn zone_rules_hold() {
    let with = |zones: BuffZones| {
        let mut req = calc("char_2013_cerber");
        req.zones = Some(zones);
        req.validate()
    };
    let too_many = BuffZones {
        inspire: vec![1.0; 17],
        ..BuffZones::default()
    };
    assert_eq!(
        message(with(too_many).unwrap_err()),
        "zones.inspire takes at most 16 sources"
    );
    let zero_multiplier = BuffZones {
        atk_multiplier: vec![0.0],
        ..BuffZones::default()
    };
    assert_eq!(
        message(with(zero_multiplier).unwrap_err()),
        "zones.atkMultiplier must be greater than 0 (got 0)"
    );
    let mut bad_percent = BuffZones::default();
    bad_percent.def.percent = Some(1.5);
    assert!(message(with(bad_percent).unwrap_err()).starts_with("zones.def.percent"));
    let mut res_steal = BuffZones::default();
    res_steal.res.steal = vec![10_001.0];
    assert!(message(with(res_steal).unwrap_err()).starts_with("zones.res.steal"));
    with(BuffZones::default()).unwrap();
}

#[test]
fn secondary_excludes_targets_and_frame_timing_is_refused() {
    let mut req = calc("char_2013_cerber");
    req.secondary = Some(RequestSecondary {
        def: 0.0,
        res: 0.0,
        count: 2,
    });
    req.validate().unwrap();
    req.targets = Some(2);
    assert!(message(req.validate().unwrap_err()).starts_with("targets and secondary"));

    let mut req = calc("char_2013_cerber");
    req.frame_timing = Some(true);
    assert!(message(req.validate().unwrap_err()).starts_with("frameTiming"));
    req.frame_timing = Some(false);
    req.validate().unwrap();
}

#[test]
fn the_clone_must_be_an_operator_id() {
    let mut req = calc("char_249_mlyss");
    req.clone_operator_id = Some("ela".into());
    assert!(message(req.validate().unwrap_err()).starts_with("cloneOperatorId"));
    req.clone_operator_id = Some(format!("char_{}", "x".repeat(64)));
    assert!(req.validate().is_err());
    req.clone_operator_id = Some("char_4123_ela".into());
    req.validate().unwrap();
}

#[test]
fn batch_bounds_and_index_prefix() {
    let empty = BatchRequest {
        requests: Vec::new(),
        sweep: None,
    };
    assert!(message(empty.validate().unwrap_err()).starts_with("requests must hold"));
    let full = BatchRequest {
        requests: vec![calc("char_2013_cerber"); MAX_BATCH + 1],
        sweep: None,
    };
    assert!(full.validate().is_err());
    let mut bad = calc("char_2013_cerber");
    bad.level = Some(91);
    let one_bad = BatchRequest {
        requests: vec![calc("char_2013_cerber"), bad],
        sweep: None,
    };
    assert!(message(one_bad.validate().unwrap_err()).starts_with("requests[1]: level"));
}

#[test]
fn sweep_points_and_bounds() {
    let sweep = Sweep {
        axis: SweepAxis::Def,
        min: 0.0,
        max: 1_000.0,
        steps: 5,
    };
    assert_eq!(sweep.points(), vec![0.0, 250.0, 500.0, 750.0, 1_000.0]);
    let one = Sweep { steps: 1, ..sweep };
    assert_eq!(one.points(), vec![0.0]);
    let with = |sweep: Sweep| {
        BatchRequest {
            requests: vec![calc("char_2013_cerber")],
            sweep: Some(sweep),
        }
        .validate()
    };
    with(sweep.clone()).unwrap();
    assert!(
        with(Sweep {
            steps: 102,
            ..sweep
        })
        .is_err()
    );
    assert!(with(Sweep { steps: 0, ..sweep }).is_err());
    assert!(
        with(Sweep {
            min: 2_000.0,
            ..sweep
        })
        .is_err()
    );
    let res = Sweep {
        axis: SweepAxis::Res,
        max: 1_001.0,
        ..sweep
    };
    assert!(message(with(res).unwrap_err()).starts_with("sweep.max"));
}

#[test]
fn rank_validation() {
    RankRequest::default().validate().unwrap();
    let limit = RankRequest {
        limit: Some(5_001),
        ..RankRequest::default()
    };
    assert_eq!(
        message(limit.validate().unwrap_err()),
        "limit must be at most 5000"
    );
    let filters = RankRequest {
        filters: Some(RankFilters {
            rarities: Some(vec![6; 17]),
            ..RankFilters::default()
        }),
        ..RankRequest::default()
    };
    assert_eq!(
        message(filters.validate().unwrap_err()),
        "too many filter values"
    );
    let context = RankRequest {
        defense: Some(-1.0),
        ..RankRequest::default()
    };
    assert!(message(context.validate().unwrap_err()).starts_with("defense"));
}

#[test]
fn the_rank_pages_own_contexts_are_on_the_grid() {
    let base = RankRequest::default();
    assert!(base.on_page_grid(), "nothing set");
    for (defense, res) in [
        (0.0, 0.0),
        (1_000.0, 0.0),
        (1_800.0, 20.0),
        (2_500.0, 30.0),
        (5_000.0, 100.0),
    ] {
        let req = RankRequest {
            defense: Some(defense),
            res: Some(res),
            targets: Some(1),
            preset: Some(RankPreset::P6Max),
            cycle_model: Some(RequestCycleModel::V2),
            ..base.clone()
        };
        assert!(req.on_page_grid(), "DEF {defense} RES {res}");
    }
}

#[test]
fn any_other_context_is_off_the_grid() {
    let base = RankRequest::default();
    let off = [
        RankRequest {
            defense: Some(350.0),
            ..base.clone()
        },
        RankRequest {
            defense: Some(300.001),
            ..base.clone()
        },
        RankRequest {
            defense: Some(5_100.0),
            ..base.clone()
        },
        RankRequest {
            res: Some(33.0),
            ..base.clone()
        },
        RankRequest {
            res: Some(105.0),
            ..base.clone()
        },
        RankRequest {
            targets: Some(3),
            ..base.clone()
        },
        RankRequest {
            sp_boost: Some(0.5),
            ..base.clone()
        },
        RankRequest {
            all_cond: Some(true),
            ..base.clone()
        },
        RankRequest {
            zones: Some(BuffZones::default()),
            ..base.clone()
        },
        RankRequest {
            secondary: Some(RequestSecondary {
                def: 0.0,
                res: 0.0,
                count: 1,
            }),
            ..base
        },
    ];
    for req in off {
        assert!(!req.on_page_grid(), "{req:?}");
    }
}

#[test]
fn row_requests_follow_the_preset() {
    let p6 = RankRequest {
        preset: Some(RankPreset::P6Max),
        ..RankRequest::default()
    }
    .row_request("char_2013_cerber", 3, 1);
    assert_eq!(
        (
            p6.promotion,
            p6.level,
            p6.potential,
            p6.mastery_level,
            p6.module_level
        ),
        (Some(2), Some(90), Some(6), Some(3), Some(3))
    );
    let plain = RankRequest {
        preset: Some(RankPreset::Default),
        ..RankRequest::default()
    }
    .row_request("char_2013_cerber", 3, 0);
    assert_eq!(
        (plain.promotion, plain.potential, plain.module_level),
        (None, None, None)
    );
    let unmoduled = RankRequest::default().row_request("char_2013_cerber", 1, 0);
    assert_eq!(
        (unmoduled.potential, unmoduled.module_level),
        (Some(1), None)
    );
}

#[test]
fn the_wire_names_are_camel_case() {
    let body = serde_json::json!({
        "operatorId": "char_2013_cerber",
        "skillIndex": 3,
        "cycleModel": "v2",
        "secondary": { "def": 100.0, "res": 10.0, "count": 2 },
    });
    let req: CalculateRequest = serde_json::from_value(body).unwrap();
    assert_eq!(req.skill_index, Some(3));
    assert_eq!(req.cycle_model, Some(RequestCycleModel::V2));
    let back = serde_json::to_value(&req).unwrap();
    assert!(
        back.get("zones").is_none(),
        "unset phase fields stay out of the hash"
    );
    assert!(back.get("frameTiming").is_none());
    let q: SupportersQuery = serde_json::from_str(r#"{"skillLevel":7}"#).unwrap();
    assert_eq!(q.skill_level, Some(7));
}
