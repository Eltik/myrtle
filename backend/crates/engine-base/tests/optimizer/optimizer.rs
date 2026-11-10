use super::*;

#[test]
fn fixed_synergy_squads_are_marked_locked_and_flexible_teams_are_not() {
    // A Shamare/Tequila/Bibeak post is a FIXED squad (they only work together), so
    // it's flagged `locked`. A post of three independent flat traders is flexible.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let locked_for = |roster: &[OperatorBaseProfile]| -> bool {
        let asn =
            compute_optimal_assignment(roster, &trading_post(3), gd.building(), &registry, &drains);
        asn.rooms
            .iter()
            .find(|r| r.room_type == "TRADING")
            .is_some_and(|r| r.locked)
    };

    let shamare_team = vec![
        profile(gd, "char_254_vodfox"), // Shamare (nullifier)
        profile(gd, "char_486_takila"), // Tequila (order value)
        profile(gd, "char_252_bibeak"), // Bibeak (order value)
    ];
    assert!(
        locked_for(&shamare_team),
        "a nullifier synergy squad must be marked locked"
    );

    let flexible_team = vec![
        profile(gd, "char_502_nblade"), // flat +30%
        profile(gd, "char_123_fang"),   // flat
        profile(gd, "char_211_adnach"), // flat
    ];
    assert!(
        !locked_for(&flexible_team),
        "a team of independent flat traders must NOT be locked"
    );

    // Penguin Logistics (Texas needs Lappland) is a fixed synergy -> locked.
    let penguin = vec![
        profile(gd, TEXAS),
        profile(gd, LAPPLAND),
        profile(gd, EXUSIAI),
    ];
    assert!(
        locked_for(&penguin),
        "Texas+Lappland (named-teammate synergy) must be marked locked"
    );
}

#[test]
fn current_assignment_reflects_live_base() {
    // The "current" assignment should mirror the player's actual stationed
    // operators and be no better than the optimizer's peak (which is the best
    // possible arrangement of the same roster).
    let gd = load_game_data();
    let profiles = full_roster(gd);
    // A base where the player has stationed a deliberately mediocre crew (plain
    // flat traders / factory ops), so the optimizer has room to improve.
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("cc", "CONTROL", 5),
            UserRoom {
                slot_id: "tp".into(),
                room_type: "TRADING".into(),
                level: 3,
                current_operators: vec![
                    "char_502_nblade".into(),
                    "char_123_fang".into(),
                    "char_211_adnach".into(),
                ],
                ..Default::default()
            },
            UserRoom {
                slot_id: "mf".into(),
                room_type: "MANUFACTURE".into(),
                level: 3,
                current_operators: vec![
                    "char_496_wildmn".into(), // Wild Mane
                    "char_190_clour".into(),  // Vermeil
                    "char_500_noirc".into(),  // Noir Corne
                ],
                ..Default::default()
            },
            room("d", "DORMITORY", 5),
        ],
    };

    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let current = myrtle_engine_base::assignment::compute_current_assignment(
        &profiles,
        &building,
        gd.building(),
        &registry,
        &drains,
        None,
    );
    let optimal =
        compute_optimal_assignment(&profiles, &building, gd.building(), &registry, &drains);

    let staffed = current
        .rooms
        .iter()
        .filter(|r| r.room_type == "TRADING" || r.room_type == "MANUFACTURE")
        .filter(|r| !r.operators.is_empty())
        .count();
    assert!(
        staffed > 0,
        "current base should have stationed production operators"
    );

    // The optimizer can't do worse than the player's current arrangement. Compare
    // by realized YIELD (LMD-equivalent/day), the optimizer's actual objective,
    // not the raw efficiency-sum, which ignores the gold/EXP split, order value,
    // and per-room soft caps and so can rank a lower-yield base higher.
    let realized_value = |asn: &myrtle_engine_base::types::BaseAssignment| -> f64 {
        let mut flows = myrtle_engine_base::scoring::yield_model::BaseFlows::default();
        for r in &asn.rooms {
            flows.add_room(
                &r.room_type,
                r.formula_type.as_deref(),
                r.level,
                r.total_efficiency,
                r.order_gold,
                r.order_value,
                r.operators.len(),
                r.order_limit,
            );
        }
        flows.total_value()
    };
    let opt_value = realized_value(&optimal);
    let cur_value = realized_value(&current);
    assert!(
        opt_value + 1.0 >= cur_value,
        "optimal yield ({opt_value:.0}) should be >= current yield ({cur_value:.0})"
    );
}

#[test]
#[ignore = "benchmark - run with: cargo test base_grade_perf -- --ignored --nocapture"]
fn base_grade_perf() {
    use std::time::Instant;
    let gd = load_game_data();
    let building = generic_base();
    let profiles = full_roster(gd);
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    // Time the two calls the grade runs.
    let iters = 30;
    let t = Instant::now();
    for _ in 0..iters {
        let _ = compute_optimal_assignment(&profiles, &building, gd.building(), &registry, &drains);
        let _ =
            compute_sustained_assignment(&profiles, &building, gd.building(), &registry, &drains);
    }
    let per = t.elapsed().as_secs_f64() * 1000.0 / f64::from(iters);
    println!(
        "optimizer pipeline: {per:.1} ms/run ({} operators)",
        profiles.len()
    );
}

#[test]
fn optimizes_a_full_base() {
    // Smoke test on a full base with a deep roster: the optimizer should surface
    // the Control Center and fully staff every production room.
    let gd = load_game_data();
    let building = generic_base();
    let profiles = full_roster(gd);

    assert!(
        profiles.len() > 50,
        "the game has a deep pool of base operators, got {}",
        profiles.len()
    );

    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let optimal =
        compute_optimal_assignment(&profiles, &building, gd.building(), &registry, &drains);
    let sustained =
        compute_sustained_assignment(&profiles, &building, gd.building(), &registry, &drains);

    assert!(
        optimal.rooms.iter().any(|r| r.room_type == "CONTROL"),
        "Control Center should be surfaced"
    );
    assert!(optimal.total_production_efficiency > 0.0);
    assert!(sustained.sustained_efficiency > 0.0);

    // Every production room is fully staffed.
    for room in &optimal.rooms {
        if room.room_type == "TRADING" || room.room_type == "MANUFACTURE" {
            let slots = max_stationed(gd, &room.room_type, room.level);
            assert_eq!(
                room.operators.len() as i32,
                slots,
                "{} (L{}) should fill all {slots} slots, got {:?}",
                room.room_type,
                room.level,
                room.operators
            );
        }
    }
}

#[test]
fn grade_base_scores_stationing_quality() {
    // The grade answers ONE question: does the player have the best base THEY
    // could have? Actual sustained yield / achievable sustained yield, on
    // their own roster and rooms. So: nobody stationed grades 0 (an empty
    // base earns nothing), any real stationing grades in (0, 1], and
    // stationing closer to the optimizer's own pick never grades lower.
    let gd = load_game_data();
    let roster: Vec<RosterEntry> = [
        TEXAS,
        LAPPLAND,
        EXUSIAI,
        "char_496_wildmn",
        "char_190_clour",
        "char_003_kalts",
    ]
    .iter()
    .map(|id| roster_entry(id))
    .collect();

    // A 1-TP / 2-factory base with the given crews stationed, production rooms
    // at the given level. Factories carry real formulas, as every synced base
    // does.
    let building = |tp: &[&str], mf0: &[&str], mf1: &[&str], level: i64| {
        let mut chars = serde_json::Map::new();
        let mut inst_of = std::collections::HashMap::new();
        for (i, id) in tp.iter().chain(mf0).chain(mf1).enumerate() {
            let inst = i as i64 + 1;
            inst_of.insert(*id, inst);
            chars.insert(inst.to_string(), serde_json::json!({ "charId": id }));
        }
        let ids = |crew: &[&str]| -> Vec<i64> { crew.iter().map(|id| inst_of[id]).collect() };
        serde_json::json!({
            "chars": chars,
            "rooms": { "MANUFACTURE": {
                "mf0": { "formulaId": "4" },
                "mf1": { "formulaId": "1" },
            }},
            "roomSlots": {
                "cc": { "roomId": "CONTROL", "level": 5, "state": 2 },
                "tp0": { "roomId": "TRADING", "level": level, "state": 2, "charInstIds": ids(tp) },
                "mf0": { "roomId": "MANUFACTURE", "level": level, "state": 2, "charInstIds": ids(mf0) },
                "mf1": { "roomId": "MANUFACTURE", "level": level, "state": 2, "charInstIds": ids(mf1) },
                "d0": { "roomId": "DORMITORY", "level": 5, "state": 2 },
            }
        })
    };

    // Unstaffed base, rooms already max level: zero utilization, full
    // infrastructure: exactly the 25% infrastructure share, and the stored
    // components say so explicitly.
    let empty = grade_base(&roster, Some(&building(&[], &[], &[], 3)), gd);
    assert!(
        (empty.score - 0.25).abs() < 1e-9,
        "unstaffed but fully built = the infrastructure share alone, got {}",
        empty.score
    );
    assert!(
        empty.utilization.abs() < 1e-9,
        "no stationing, no utilization"
    );
    assert!(
        (empty.infrastructure - 1.0).abs() < 1e-9,
        "rooms already maxed"
    );

    // Bodies with no relevant skills in the wrong rooms...
    let weak = grade_base(
        &roster,
        Some(&building(&["char_003_kalts"], &["char_190_clour"], &[], 3)),
        gd,
    )
    .score;
    // ...vs the trading synergy where it belongs.
    let good = grade_base(
        &roster,
        Some(&building(
            &[TEXAS, LAPPLAND, EXUSIAI],
            &["char_496_wildmn"],
            &["char_190_clour", "char_003_kalts"],
            3,
        )),
        gd,
    )
    .score;
    assert!(
        (0.0..=1.0).contains(&weak) && weak > 0.25,
        "a staffed base earns above the infrastructure floor: {weak}"
    );
    assert!(
        (0.0..=1.0).contains(&good) && good >= weak - 1e-6,
        "better stationing never grades lower (good={good}, weak={weak})"
    );

    // The same stationing on under-leveled rooms: utilization is unchanged
    // (both sides shrink together) but the infrastructure share drops, so the
    // grade must come out strictly lower.
    let good_low = grade_base(
        &roster,
        Some(&building(
            &[TEXAS, LAPPLAND, EXUSIAI],
            &["char_496_wildmn"],
            &["char_190_clour", "char_003_kalts"],
            1,
        )),
        gd,
    )
    .score;
    assert!(
        good_low < good - 1e-6,
        "under-leveled rooms grade lower (L1={good_low}, L3={good})"
    );
}

#[test]
fn a_scoped_proposal_leaves_every_other_room_as_drafted_and_never_worsens_the_room() {
    // "Optimize this room only" must read as exactly that: the search
    // re-seats dormitories and the Control Center on its own and, where the
    // scoped room is not the bottleneck, may hand back a slower crew. The
    // settle pass keeps every out-of-scope room as drafted, books nobody
    // twice, and changes the scoped room only when its own yield rises.
    use myrtle_engine_base::assignment::compute_current_assignment;
    use myrtle_engine_base::assignment::scope::settle_scoped_proposal;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let roster = full_roster(gd);
    let crewed = |slot: &str, room_type: &str, level: i32, crew: &[&str], formula: Option<&str>| {
        let mut r = room(slot, room_type, level);
        r.current_operators = crew.iter().map(|s| (*s).to_string()).collect();
        r.current_formula = formula.map(str::to_string);
        r
    };
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            crewed("cc", "CONTROL", 5, &["char_002_amiya"], None),
            crewed("tp0", "TRADING", 3, &[TEXAS, LAPPLAND, EXUSIAI], None),
            crewed(
                "mf0",
                "MANUFACTURE",
                3,
                &["char_123_fang", "char_133_mm"],
                Some("F_GOLD"),
            ),
            crewed("mf1", "MANUFACTURE", 3, &["char_502_nblade"], Some("F_EXP")),
            crewed(
                "d0",
                "DORMITORY",
                5,
                &["char_124_kroos", "char_212_ansel"],
                None,
            ),
        ],
    };
    let baseline =
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
    // An unpinned full plan: it re-crews every room, the worst case a scoped
    // search could hand the settle pass.
    let full = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let scope: std::collections::HashSet<String> = ["mf1".to_string()].into_iter().collect();
    let settled = settle_scoped_proposal(
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
        &scope,
        &baseline,
        &full,
    );
    let crews = |a: &myrtle_engine_base::types::BaseAssignment| -> std::collections::HashMap<String, std::collections::HashSet<String>> {
        a.rooms
            .iter()
            .map(|r| (r.slot_id.clone(), r.operators.iter().cloned().collect()))
            .collect()
    };
    let settled_crews = crews(&settled);
    for drafted in &building.rooms {
        if drafted.slot_id == "mf1" {
            continue;
        }
        let want: std::collections::HashSet<String> =
            drafted.current_operators.iter().cloned().collect();
        if let Some(got) = settled_crews.get(&drafted.slot_id) {
            assert_eq!(
                got, &want,
                "{} is out of scope and must read as drafted",
                drafted.slot_id
            );
        }
    }
    let mut seen = std::collections::HashSet::new();
    for r in &settled.rooms {
        for o in &r.operators {
            assert!(
                seen.insert(o.clone()),
                "{o} is booked twice in the settled proposal"
            );
        }
    }
    let before = baseline
        .rooms
        .iter()
        .find(|r| r.slot_id == "mf1")
        .expect("drafted mf1");
    let after = settled
        .rooms
        .iter()
        .find(|r| r.slot_id == "mf1")
        .expect("settled mf1");
    assert!(
        after.total_efficiency >= before.total_efficiency - 1e-9,
        "the scoped room never reads worse than its draft: {:.1} -> {:.1}",
        before.total_efficiency,
        after.total_efficiency
    );
}

#[test]
fn hoshigumas_camaraderie_fires_beside_another_lgd_operator() {
    // "When this Operator is assigned together with other L.G.D. Operators to
    // the Control Center, all Factories' productivity +3%" (Hoshiguma the
    // Breacher). The gate is on the center's crew: Swire (group L.G.D.)
    // meets it, Amiya does not, and Swire the Elegant Wit (no group) does
    // not either. It used to be read as "+3% to each L.G.D. operator in
    // Factories" and never fired (user report 2026-10-03).
    use myrtle_engine_base::assignment::compute_current_assignment;
    use myrtle_engine_base::scoring::skill_ledger::LineDisposition;
    const HOSHI: &str = "char_1044_hsgma2";
    const SWIRE: &str = "char_308_swire";
    const SWIRE_ALT: &str = "char_1033_swire2";
    const AMIYA: &str = "char_002_amiya";
    const GRAVEL: &str = "char_237_gravel";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let factory_with_cc = |cc: &[&str]| {
        let mut control = room("cc", "CONTROL", 5);
        control.current_operators = cc.iter().map(|s| (*s).to_string()).collect();
        let mut mf = room("mf", "MANUFACTURE", 3);
        mf.current_formula = Some("F_GOLD".into());
        mf.current_operators = vec![GRAVEL.into()];
        let building = UserBuilding {
            drone_capacity: None,
            rooms: vec![control, mf],
        };
        let roster: Vec<_> = cc
            .iter()
            .copied()
            .chain([GRAVEL])
            .map(|id| profile(gd, id))
            .collect();
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        let mf = asn
            .rooms
            .iter()
            .find(|r| r.slot_id == "mf")
            .expect("the factory");
        let hoshi_line = mf
            .ledger
            .iter()
            .find(|l| l.operator_id == HOSHI && l.buff_id.starts_with("control_token_prod_spd3"))
            .map(|l| (l.disposition, l.speed_pct));
        (mf.total_efficiency, hoshi_line)
    };
    let base = factory_with_cc(&[]).0;
    let (with_swire, line) = factory_with_cc(&[HOSHI, SWIRE]);
    assert!(
        (with_swire - base - 3.0).abs() < 1e-6,
        "Hoshiguma beside Swire grants the factories +3: {base:.1} -> {with_swire:.1}"
    );
    assert!(
        matches!(line, Some((LineDisposition::Contributes, v)) if (v - 3.0).abs() < 1e-6),
        "her line carries the full +3 when the gate is met: {line:?}"
    );
    let (with_amiya, line) = factory_with_cc(&[HOSHI, AMIYA]);
    assert!(
        (with_amiya - base).abs() < 1e-6,
        "Amiya is not L.G.D.: {base:.1} -> {with_amiya:.1}"
    );
    assert!(
        matches!(line, Some((LineDisposition::Inactive, _))),
        "her line reads inactive without another L.G.D. operator: {line:?}"
    );
    let (with_alt, _) = factory_with_cc(&[HOSHI, SWIRE_ALT]);
    assert!(
        (with_alt - base).abs() < 1e-6,
        "Swire the Elegant Wit carries no group and does not count: {base:.1} -> {with_alt:.1}"
    );
}

#[test]
fn a_robot_token_is_parked_dead_when_its_gate_and_an_exclusion_gate_both_pay() {
    // The community's "dead Lancet": Eunectes in the Control Center reads
    // Lancet-2 as assigned to a Power Plant whatever his morale (+2 plants),
    // and Greyy the Lightningbearer's "no Operation Platforms in other Power
    // Plants" stops seeing him once he is dead (+1). Parked at zero morale he
    // is worth both counts and forfeits only his own +10% drone recovery. The
    // plan seats him and Greyy in different plants with Eunectes in the
    // center, the rotation keeps him there every shift without rest, and the
    // morale simulation does not report him as running dry (user report
    // 2026-10-04).
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    use myrtle_engine_base::rotation::sustain_sim::simulate_rotation;
    use myrtle_engine_base::scoring::pools::{optimal_with_bundles, search_economy};
    const EUNECTES: &str = "char_416_zumama";
    const LANCET: &str = "char_285_medic2";
    const GREYY_ALT: &str = "char_1027_greyy2";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let roster: Vec<OperatorBaseProfile> = [
        EUNECTES,
        LANCET,
        GREYY_ALT,
        "char_400_weedy",
        "char_4004_pudd",
        "char_002_amiya",
        "char_2027_wang",
        "char_1020_reed2",
        "char_4098_vvana",
        "char_107_liskam",
        "char_1011_lava2",
        "char_427_vigil",
        "char_123_fang",
        "char_133_mm",
        "char_237_gravel",
        "char_141_nights",
        "char_240_wyvern",
        "char_235_jesica",
        TEXAS,
        LAPPLAND,
        EXUSIAI,
        "char_4032_provs",
        "char_486_takila",
        "char_215_mantic",
        "char_290_vigna",
        "char_193_frostl",
        "char_159_peacok",
        "char_1502_crosly",
        "char_1032_excu2",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    let factory = |slot: &str, formula: &str| {
        let mut r = room(slot, "MANUFACTURE", 3);
        r.current_formula = Some(formula.into());
        r
    };
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("cc", "CONTROL", 5),
            room("tp0", "TRADING", 3),
            room("tp1", "TRADING", 3),
            factory("mf0", "F_GOLD"),
            factory("mf1", "F_GOLD"),
            factory("mf2", "F_EXP"),
            room("pp0", "POWER", 3),
            room("pp1", "POWER", 3),
            room("d0", "DORMITORY", 5),
            room("d1", "DORMITORY", 5),
        ],
    };
    let economy = search_economy(&roster, &building, gd.building(), &registry);
    // As the rotation service does: the rotation plans with the economy the
    // bundle trials accepted (registry and pins).
    let accepted = optimal_with_bundles(
        &roster,
        &building,
        gd.building(),
        &registry,
        &economy.registry,
        &drains,
        &economy.pins,
    );
    let plan = &accepted.optimal;
    let room_of = |a: &myrtle_engine_base::types::BaseAssignment, id: &str| {
        a.rooms
            .iter()
            .find(|r| r.operators.iter().any(|o| o == id))
            .map(|r| (r.slot_id.clone(), r.room_type.clone()))
    };
    assert_eq!(
        room_of(plan, EUNECTES).map(|(_, rt)| rt).as_deref(),
        Some("CONTROL"),
        "Eunectes holds the center"
    );
    let lancet = room_of(plan, LANCET).expect("Lancet-2 is seated");
    let greyy = room_of(plan, GREYY_ALT).expect("Greyy is seated");
    assert!(
        lancet.1 == "POWER" && greyy.1 == "POWER" && lancet.0 != greyy.0,
        "the token and the exclusion holder sit in different plants: {lancet:?} vs {greyy:?}"
    );

    let rot = recommend_shift_rotation(
        &roster,
        &building,
        gd.building(),
        &accepted.registry,
        &drains,
        &accepted.pins,
    );
    assert_eq!(
        rot.parked,
        vec![LANCET.to_string()],
        "Lancet-2 is the parked token"
    );
    for shift in &rot.shifts {
        let seat = shift
            .rooms
            .iter()
            .find(|r| r.recommended.iter().any(|o| o == LANCET))
            .map(|r| r.room_type.clone());
        assert_eq!(
            seat.as_deref(),
            Some("POWER"),
            "shift {} keeps the parked token in a plant",
            shift.index
        );
    }
    let report = simulate_rotation(
        &rot,
        &roster,
        &building,
        gd.building(),
        &accepted.registry,
        &drains,
        &std::collections::HashMap::new(),
    );
    assert!(
        !report.depleted.iter().any(|d| d.char_id == LANCET),
        "a parked token is not reported as running dry"
    );
}

#[test]
fn a_small_roster_falls_back_to_the_24h_blocks() {
    // Three teams per production room need `3N` crews; a roster that cannot
    // fill them keeps every room staffed by running the 24h blocks instead,
    // where a team holds its room for two consecutive shifts.
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let roster: Vec<_> = [
        TEXAS,
        LAPPLAND,
        EXUSIAI,
        "char_4193_lemuen",
        "char_272_strong",
        "char_252_bibeak",
        "char_196_sunbr",
        "char_283_midn",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .collect();
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("tp0", "TRADING", 3),
            room("tp1", "TRADING", 3),
            room("d0", "DORMITORY", 5),
        ],
    };
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);
    let crews = |slot: &str| -> Vec<Vec<String>> {
        rot.shifts
            .iter()
            .map(|s| {
                s.rooms
                    .iter()
                    .find(|r| r.slot_id == slot)
                    .map(|r| r.recommended.clone())
                    .unwrap_or_default()
            })
            .collect()
    };
    let (a, b) = (crews("tp0"), crews("tp1"));
    assert!(
        a.iter().chain(b.iter()).all(|c| !c.is_empty()),
        "both posts stay staffed every shift: {a:?} {b:?}"
    );
    let repeats = |c: &Vec<Vec<String>>| (0..3).any(|k| c[k] == c[(k + 1) % 3]);
    assert!(
        repeats(&a) || repeats(&b),
        "eight traders cannot fill six crews, so a team holds its post across a boundary: {a:?} {b:?}"
    );
}

#[test]
fn facility_count_enabler_counts_only_from_her_seat() {
    // Greyy the Lightningbearer E2 raises the EFFECTIVE Power Plant count by 1 ("only affects
    // facility quantity"), but only while she is seated in a Power Plant. The core search
    // never seats her there on its own (the plan carries no plant crews), so without a pin
    // Weedy reads the real plants; the improvements oracle offers her seat as a bundle, and
    // with that pin Weedy's per-plant output rises.
    use myrtle_engine_base::assignment::compute_optimal_assignment_with_pins;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("f", "MANUFACTURE", 3), room("p0", "POWER", 3)],
    };
    let roster = vec![
        profile(gd, "char_400_weedy"),
        profile(gd, "char_1027_greyy2"),
    ];
    let factory_eff = |pins: &[(String, String)]| -> f64 {
        compute_optimal_assignment_with_pins(
            &roster,
            &building,
            gd.building(),
            &registry,
            &drains,
            pins,
        )
        .rooms
        .iter()
        .find(|r| r.room_type == "MANUFACTURE")
        .map_or(0.0, |r| r.total_efficiency)
    };
    let unseated = factory_eff(&[]);
    let seated = factory_eff(&[("char_1027_greyy2".to_string(), "POWER".to_string())]);
    assert!(
        (unseated - 15.0).abs() < 1e-9,
        "one real plant without her seat: {unseated}"
    );
    assert!(
        (seated - 30.0).abs() < 1e-9,
        "her plant seat unlocks the second count: {seated}"
    );
}

#[test]
fn optimal_with_empty_pins_is_identical_to_the_plain_optimum() {
    // The pinned path must be a no-op when nothing is pinned, so non-243 / no-economy bases (and
    // every existing caller) get exactly the plain optimum; the economy only ever ADDS pins.
    let gd = load_game_data();
    let mut rooms = vec![room("cc", "CONTROL", 5), room("hr", "HIRE", 3)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("pp{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let roster: Vec<_> = [
        "char_002_amiya",
        "char_003_kalts",
        "char_134_ifrit",
        "char_391_rosmon",
        "char_455_nothin",
        "char_2025_shu",
        "char_473_mberry",
        "char_2023_ling",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .collect();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));

    let plain = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let pinned = compute_optimal_assignment_with_pins(
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
        &[],
    );

    let rooms_of = |a: &myrtle_engine_base::types::BaseAssignment| {
        a.rooms
            .iter()
            .map(|r| (r.room_type.clone(), r.operators.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        rooms_of(&plain),
        rooms_of(&pinned),
        "empty pins must reproduce the plain optimum exactly"
    );
    assert!(
        (plain.total_production_efficiency - pinned.total_production_efficiency).abs() < 1e-9,
        "empty pins must not change production efficiency"
    );
}

#[test]
fn base_wide_bonus_applies_only_when_the_partner_works_a_work_area() {
    // Hoederer's "+5% when Ines or W is assigned to any Work Area" must be credited only when Ines is
    // actually deployed in a work area, not merely owned, and not while she has nowhere to work.
    // With a Reception Room she works (a Work Area) and the bonus applies; without one (and no
    // production skill of her own) she can't be stationed, so it doesn't.
    const HOEDERER: &str = "char_4088_hodrer";
    const INES: &str = "char_4087_ines";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let roster: Vec<_> = [HOEDERER, INES, "char_502_nblade", "char_185_frncat"]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let trading_eff = |building: &UserBuilding| -> f64 {
        compute_optimal_assignment(&roster, building, gd.building(), &registry, &drains)
            .rooms
            .iter()
            .filter(|r| r.room_type == "TRADING")
            .map(|r| r.total_efficiency)
            .sum()
    };
    let with_work_area = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("tp", "TRADING", 3),
            room("rr", "MEETING", 3),
            room("d0", "DORMITORY", 5),
        ],
    };
    let without_work_area = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 5)],
    };
    let with = trading_eff(&with_work_area);
    let without = trading_eff(&without_work_area);
    assert!(
        with > without + 1.0,
        "Hoederer's +5% should apply only when Ines works a Work Area (with={with}, without={without})"
    );
}

// ─── Rotation rework: login-rhythm tiling, proactive Fiammetta, alignment, leniency ──────────

#[test]
fn synergy_pairs_stay_co_teamed_and_all_three_teams_are_staffed() {
    // The balanced packer selects whole candidate teams, so a superadditive pair
    // (Texas + Lappland) is never split across teams, while a roster with enough
    // traders staffs all three of the pair-of-posts teams instead of frontloading
    // two and leaving the third hollow.
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let mut rooms = vec![room("cc", "CONTROL", 5), room("d0", "DORMITORY", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..2).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let roster: Vec<_> = [
        TEXAS,
        LAPPLAND,
        EXUSIAI,
        "char_4063_quartz",
        "char_502_nblade",
        "char_185_frncat",
        "char_211_adnach",
        "char_123_fang",
        "char_133_mm",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);

    let mut teams: std::collections::HashMap<String, std::collections::HashSet<String>> =
        std::collections::HashMap::new();
    for s in &rot.shifts {
        for r in s
            .rooms
            .iter()
            .filter(|r| r.room_type == "TRADING" && r.active)
        {
            if let Some(id) = &r.team_id {
                teams
                    .entry(id.clone())
                    .or_default()
                    .extend(r.recommended.iter().cloned());
            }
        }
    }
    let texas_team = teams.values().find(|ops| ops.contains(TEXAS));
    assert!(
        texas_team.is_some_and(|ops| ops.contains(LAPPLAND)),
        "Texas and Lappland must stay co-teamed (the pair is superadditive): {teams:?}"
    );
    assert!(
        teams.values().filter(|ops| !ops.is_empty()).count() >= 3,
        "a pair of posts with a deep trader pool staffs three distinct teams: {teams:?}"
    );
}

/// Mantra's "+25%; for every facility in the Base with an Elite Operator
/// assigned, +2% (max 10 facilities)" counts FACILITIES, which the deployment
/// map cannot see: she reads her flat 25, never 25 per counted operator
/// (a live plan briefly showed a 202% post built on that mis-parse).
#[test]
fn mantra_keeps_her_flat_base_not_a_per_operator_count() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const MANTRA: &str = "char_4204_mantra";
    let mut rooms = vec![room("tp", "TRADING", 3), room("mf", "MANUFACTURE", 3)];
    rooms[0].current_operators = vec![MANTRA.into()];
    rooms[1].current_operators = vec![EXUSIAI.into()];
    rooms[1].current_formula = Some("F_EXP".into());
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let roster: Vec<_> = [MANTRA, EXUSIAI].iter().map(|id| profile(gd, id)).collect();
    let asn =
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
    let eff = asn
        .rooms
        .iter()
        .find(|r| r.slot_id == "tp")
        .map_or(0.0, |r| r.total_efficiency);
    assert!((eff - 25.0).abs() < 1e-9, "flat base only: {eff}");
}

/// A depleted roommate holds the seat but does not work: Kazemaru's "if no
/// other Operators are working in the Reception Room, +35%" fires beside a
/// zero-morale body (the community's dead-operator seat), not beside a live one.
#[test]
fn kazemaru_solo_gate_fires_beside_a_dead_roommate() {
    use myrtle_engine_base::assignment::compute_live_assignment;
    use std::collections::HashMap;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const KAZEMARU: &str = "char_4016_kazema";
    const BODY: &str = "char_124_kroos";
    let roster: Vec<_> = [KAZEMARU, BODY].iter().map(|id| profile(gd, id)).collect();
    let mut rooms = vec![room("rr", "MEETING", 3)];
    rooms[0].current_operators = vec![KAZEMARU.into(), BODY.into()];
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let clue = |body_morale: f64| {
        let live = HashMap::from([(BODY.to_string(), body_morale)]);
        compute_live_assignment(
            &roster,
            &building,
            gd.building(),
            &registry,
            &drains,
            None,
            &live,
        )
        .rooms
        .iter()
        .find(|r| r.slot_id == "rr")
        .map_or(0.0, |r| r.total_efficiency)
    };
    // Kazemaru 15 (+35 solo) + innate 5 + her 5-star 4 + E2 16; Kroos E2 adds 16 either way.
    assert!(
        (clue(24.0) - 56.0).abs() < 1e-9,
        "a live roommate: {}",
        clue(24.0)
    );
    assert!(
        (clue(0.0) - 91.0).abs() < 1e-9,
        "a dead roommate: {}",
        clue(0.0)
    );
}

/// Full planner run on a REAL captured base (dumped from psql by the session
/// tooling, never committed). With the CP2 shadow live, every room evaluation
/// double-scores through the clause ledger and panics on divergence, so a
/// clean pass here proves ledger equivalence on live data, and the printed
/// summary is the diff baseline for the principled changes (CP3/CP4).
///
/// Run: `BASE_REPRO_DIR`=<dir with building_<uid>.json / roster_<uid>.json> \
///      cargo test `real_base_repro` -- --ignored --nocapture
#[test]
#[ignore = "needs a captured user dump (BASE_REPRO_DIR)"]
fn real_base_repro() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;

    let dir = std::env::var("BASE_REPRO_DIR").expect("set BASE_REPRO_DIR to the dump directory");
    let uid = std::env::var("BASE_REPRO_UID").unwrap_or_else(|_| "89153800".into());
    let read = |name: &str| -> serde_json::Value {
        let path = format!("{dir}/{name}_{uid}.json");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {path}: {e}"))
    };

    let gd = load_game_data();
    let building_json = read("building");
    let roster: Vec<RosterEntry> = serde_json::from_value(read("roster")).expect("roster rows");

    let user_building = UserBuilding::from_json(&building_json);
    assert!(!user_building.is_empty(), "captured building parses");

    // Mirror build_base_improvements: roster -> profiles, registry, perception
    // overrides feeding the OPTIMAL registry only.
    let profiles: Vec<OperatorBaseProfile> = roster
        .iter()
        .filter_map(|entry| {
            let bc = gd.building().chars.get(&entry.operator_id)?;
            let static_op = gd.operators().get(&entry.operator_id);
            let faction_tags = static_op
                .map(|op| faction_tags_with_glossary(gd, &entry.operator_id, op))
                .unwrap_or_default();
            let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
            Some(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                rarity,
                gd.building(),
                false,
            ))
        })
        .collect();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, morale_drains) = build_registry(&gd.building().buffs, &name_to_char);

    // NATIVE-FIRST (mirrors the service): every economy is priced from
    // clauses; the morale-swap manager is reserved when the roster owns both
    // a morale-conditional generator and a manager.
    let mut optimal_registry = registry.clone();
    let mut optimal_pins: Vec<(String, String)> = Vec::new();
    if profiles.iter().any(|op| {
        myrtle_engine_base::scoring::pools::has_morale_conditional_grant(op, gd.building())
    }) && let Some(manager) =
        myrtle_engine_base::assignment::morale_swap_enabler(&profiles, gd.building())
    {
        optimal_pins.push((manager, "DORMITORY".to_string()));
    }

    // Native pool economies, mirroring the service's wiring.
    let native = myrtle_engine_base::scoring::pools::plan_optimal_economies(
        &profiles,
        &user_building,
        gd.building(),
        &registry,
    );
    for (bid, pct) in &native.overrides {
        if optimal_registry.get(bid).is_none_or(|s| {
            !matches!(
                s,
                BuffResolutionStrategy::PoolPayoff { .. }
                    | BuffResolutionStrategy::GlobalEffect { .. }
            )
        }) {
            optimal_registry.insert(
                bid.clone(),
                BuffResolutionStrategy::PoolPayoff { pct: *pct },
            );
        }
    }
    optimal_pins.extend(native.pins.iter().cloned());
    println!(
        "== NATIVE ECONOMIES: {} overrides {:?}, pins {:?}",
        native.overrides.len(),
        native.overrides,
        native.pins
    );
    // Joint-seating bundles judged by the same yield oracle the service uses.
    let bundles = myrtle_engine_base::scoring::pools::candidate_bundles(
        &profiles,
        &user_building,
        gd.building(),
        &registry,
    );
    for bundle in &bundles {
        println!(
            "== BUNDLE candidate: overrides {:?}, pins {:?}",
            bundle.overrides, bundle.pins
        );
    }
    let mut oracle_verdicts: Vec<String> = Vec::new();
    {
        use myrtle_engine_base::assignment::{
            assignment_value, compute_optimal_assignment_with_pins,
        };
        let baseline = compute_optimal_assignment_with_pins(
            &profiles,
            &user_building,
            gd.building(),
            &optimal_registry,
            &morale_drains,
            &optimal_pins,
        );
        let mut baseline_value = assignment_value(&baseline.rooms);
        for bundle in &bundles {
            let mut trial_registry = optimal_registry.clone();
            for (bid, pct) in &bundle.overrides {
                let existing = match trial_registry.get(bid) {
                    Some(BuffResolutionStrategy::PoolPayoff { pct: p }) => *p,
                    _ => f64::NEG_INFINITY,
                };
                if *pct > existing {
                    trial_registry.insert(
                        bid.clone(),
                        BuffResolutionStrategy::PoolPayoff { pct: *pct },
                    );
                }
            }
            for (bid, target_room, pct) in &bundle.globals {
                let existing = match trial_registry.get(bid) {
                    Some(BuffResolutionStrategy::GlobalEffect { bonus_pct, .. }) => *bonus_pct,
                    _ => f64::NEG_INFINITY,
                };
                if *pct > existing {
                    trial_registry.insert(
                        bid.clone(),
                        BuffResolutionStrategy::GlobalEffect {
                            target_room: target_room.clone(),
                            bonus_pct: *pct,
                        },
                    );
                }
            }
            let mut trial_pins = optimal_pins.clone();
            trial_pins.extend(bundle.pins.iter().cloned());
            let trial = compute_optimal_assignment_with_pins(
                &profiles,
                &user_building,
                gd.building(),
                &trial_registry,
                &morale_drains,
                &trial_pins,
            );
            let (b, t) = (baseline_value, assignment_value(&trial.rooms));
            let committed = t > b + 1e-9;
            oracle_verdicts.push(format!(
                "bundle {:?}: baseline {b:.1} vs trial {t:.1} -> {}",
                bundle
                    .pins
                    .iter()
                    .map(|(c, _)| c.as_str())
                    .collect::<Vec<_>>(),
                if committed { "COMMIT" } else { "reject" }
            ));
            // Mirror the service: a winning bundle commits its registry and
            // pins, and later bundles compete against the improved plan.
            if committed {
                optimal_registry = trial_registry;
                optimal_pins = trial_pins;
                baseline_value = t;
            }
        }
    }
    for v in &oracle_verdicts {
        println!("== ORACLE: {v}");
    }

    let print_assignment = |label: &str, a: &myrtle_engine_base::types::BaseAssignment| {
        println!("== {label}: total {:.3}", a.total_production_efficiency);
        for r in &a.rooms {
            println!(
                "  {} {} {:?} eff {:.3} value {:.3} ops {:?}",
                r.slot_id,
                r.room_type,
                r.formula_type,
                r.total_efficiency,
                r.order_value,
                r.operators
            );
        }
    };

    let current = compute_current_assignment(
        &profiles,
        &user_building,
        gd.building(),
        &registry,
        &morale_drains,
        None,
    );
    print_assignment("CURRENT", &current);

    let optimal = compute_optimal_assignment_with_pins(
        &profiles,
        &user_building,
        gd.building(),
        &optimal_registry,
        &morale_drains,
        &optimal_pins,
    );
    print_assignment("OPTIMAL", &optimal);

    let sustained = compute_sustained_assignment(
        &profiles,
        &user_building,
        gd.building(),
        &registry,
        &morale_drains,
    );
    print_assignment("SUSTAINED main", &sustained.main);
    println!("sustained_efficiency {:.3}", sustained.sustained_efficiency);

    // Mirror the service: the rotation plans with the economy-aware registry
    // (PoolPayoff overrides) and seats the generator pins every shift.
    let rotation = recommend_shift_rotation(
        &profiles,
        &user_building,
        gd.building(),
        &optimal_registry,
        &morale_drains,
        &optimal_pins,
    );
    println!("== ROTATION: {} shifts", rotation.shifts.len());
    for (i, s) in rotation.shifts.iter().enumerate() {
        for r in s.rooms.iter().filter(|r| r.active) {
            println!(
                "  shift{i} {} {} team {:?} eff {:?}",
                r.slot_id, r.room_type, r.recommended, r.efficiency
            );
        }
    }

    // The rotation's own morale-simulation verdict (targeted pair effects
    // included, mirroring the service).
    {
        use myrtle_engine_base::rotation::sustain_sim::simulate_rotation;
        let targeted = myrtle_engine_base::registry::targeted_morale_effects(
            &gd.building().buffs,
            &name_to_char,
        );
        let report = simulate_rotation(
            &rotation,
            &profiles,
            &user_building,
            gd.building(),
            &registry,
            &morale_drains,
            &targeted,
        );
        println!(
            "== SUSTAINABILITY: {:?} over {:.0}h, dorm_overflow {}",
            report.verdict, report.horizon_hours, report.dorm_overflow
        );
        for d in &report.depleted {
            println!(
                "  depleted {} @{:.0}h in {}",
                d.char_id, d.at_hours, d.slot_id
            );
        }
    }

    // The whole point: every evaluation above ran the ledger shadow without a
    // divergence panic.
    println!("shadow-clean: real base scored identically by both engines");
}

/// Deployment-context gates ("if Kal'tsit is assigned to the Control Center, drone
/// recovery +5%") parse into `ConditionalOnRoomPresence` and are credited by registry
/// rewrite exactly when the deployment stations the required operator (or enough
/// operators of the required faction) in the gate's room type. Context-free scoring
/// credits the gated part 0 (never guess).
#[test]
// Exact float equality on purpose: the values come straight from the constants
// the assignment resolver used, not an independent computation.
#[allow(clippy::float_cmp)]
fn room_presence_gates_resolve_against_the_deployment() {
    use myrtle_engine_base::assignment::resolve_room_presence;
    use myrtle_engine_base::registry::{BuffResolutionStrategy, build_registry};
    use myrtle_engine_base::types::OperatorBaseProfile;
    use std::collections::HashMap;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);

    // Golden parse: the four gate buffs land on the new strategy with the right context.
    let expect = [
        ("manu_formula_spd_P[000]", "TRADING", 35.0),
        ("power_rec_spd_P[000]", "CONTROL", 5.0),
        ("power_rec_spd_P[001]", "TRAINING", 5.0),
        ("power_rec_spd_ext&faction[000]", "POWER", 5.0),
        ("power_rec_spd_ext&tag[000]", "POWER", 5.0),
    ];
    for (id, want_room, want_bonus) in expect {
        match registry.get(id) {
            Some(BuffResolutionStrategy::ConditionalOnRoomPresence {
                required_char_ids,
                required_faction,
                room_type,
                base_efficiency,
                bonus_efficiency,
                ..
            }) => {
                assert_eq!(room_type, want_room, "{id}: wrong gate room type");
                assert_eq!(*bonus_efficiency, want_bonus, "{id}: wrong bonus");
                assert_eq!(
                    *base_efficiency, 0.0,
                    "{id}: gates in this family are pure-bonus"
                );
                assert!(
                    !required_char_ids.is_empty() || required_faction.is_some(),
                    "{id}: gate must name an operator or a faction"
                );
            }
            other => panic!("{id}: expected ConditionalOnRoomPresence, got {other:?}"),
        }
    }

    // Named gate: Kal'tsit seated in the Control Center unlocks the +5, anywhere else doesn't.
    let kaltsit = name_to_char["kal'tsit"].clone();
    let gate_id = "power_rec_spd_P[000]";
    for (room, want) in [("CONTROL", 5.0), ("MANUFACTURE", 0.0)] {
        let stationed: HashMap<String, String> = [(kaltsit.clone(), room.to_string())].into();
        let resolved = resolve_room_presence(&registry, &stationed, &[]);
        assert_eq!(
            resolved.get(gate_id),
            Some(&BuffResolutionStrategy::DirectEfficiency { value: want }),
            "Kal'tsit in {room}"
        );
    }

    // Faction gate: "another Laterano Operator in a Power Plant" needs TWO Laterano
    // operators deployed there (the owner works one too); one alone stays locked.
    let lat = |id: &str| OperatorBaseProfile {
        char_id: id.to_string(),
        available_buffs: vec![],
        faction_tags: vec!["laterano".to_string()],
        match_tags: vec![],
        rarity: 6,
        elite: 2,
    };
    let ops = [lat("char_a"), lat("char_b")];
    let gate_id = "power_rec_spd_ext&faction[000]";
    let one: HashMap<String, String> = [("char_a".to_string(), "POWER".to_string())].into();
    let two: HashMap<String, String> = [
        ("char_a".to_string(), "POWER".to_string()),
        ("char_b".to_string(), "POWER".to_string()),
    ]
    .into();
    let resolved_one = resolve_room_presence(&registry, &one, &ops);
    let resolved_two = resolve_room_presence(&registry, &two, &ops);
    assert_eq!(
        resolved_one.get(gate_id),
        Some(&BuffResolutionStrategy::DirectEfficiency { value: 0.0 }),
        "one Laterano operator alone must not unlock 'another Laterano'"
    );
    assert_eq!(
        resolved_two.get(gate_id),
        Some(&BuffResolutionStrategy::DirectEfficiency { value: 5.0 }),
        "two Laterano operators in Power Plants unlock the bonus"
    );

    // Tag gate: GALLUS²'s "another Operation Platform in a Power Plant" reads the
    // glossary tag (`cc.tag.op`), so Lancet-2 beside her in the plants unlocks it.
    let gallus = profile(gd, "char_4227_gallus");
    let lancet = profile(gd, "char_285_medic2");
    assert!(
        gallus.faction_tags.iter().any(|t| t == "op")
            && lancet.faction_tags.iter().any(|t| t == "op"),
        "both robots carry the glossary tag"
    );
    let robots = [gallus, lancet];
    let gate_id = "power_rec_spd_ext&tag[000]";
    let alone: HashMap<String, String> =
        [("char_4227_gallus".to_string(), "POWER".to_string())].into();
    let pair: HashMap<String, String> = [
        ("char_4227_gallus".to_string(), "POWER".to_string()),
        ("char_285_medic2".to_string(), "POWER".to_string()),
    ]
    .into();
    assert_eq!(
        resolve_room_presence(&registry, &alone, &robots).get(gate_id),
        Some(&BuffResolutionStrategy::DirectEfficiency { value: 0.0 }),
        "GALLUS² alone must not unlock 'another Operation Platform'"
    );
    assert_eq!(
        resolve_room_presence(&registry, &pair, &robots).get(gate_id),
        Some(&BuffResolutionStrategy::DirectEfficiency { value: 5.0 }),
        "a second Operation Platform in a Power Plant unlocks the bonus"
    );
}

/// PARITY HARNESS vs the reference optimizer: rebuild the exact 2/5/2 fixture
/// the reference solved for this roster (2 gold-strategy posts, 3 gold + 2 EXP
/// factories, 2 plants, low dorms) and print OUR optimal seating for the same
/// rooms; the diff against the captured reference answer attributes every
/// divergence to a coverage gap or a judgment difference.
#[test]
#[ignore = "needs a captured user dump (BASE_REPRO_DIR)"]
fn reference_parity_252() {
    use myrtle_engine_base::assignment::compute_optimal_assignment_with_pins;
    let dir = std::env::var("BASE_REPRO_DIR").expect("set BASE_REPRO_DIR");
    let uid = std::env::var("BASE_REPRO_UID").unwrap_or_else(|_| "89153800".into());
    let roster: Vec<RosterEntry> =
        serde_json::from_str(&std::fs::read_to_string(format!("{dir}/roster_{uid}.json")).unwrap())
            .unwrap();
    let gd = load_game_data();
    let profiles: Vec<OperatorBaseProfile> = roster
        .iter()
        .filter_map(|entry| {
            let bc = gd.building().chars.get(&entry.operator_id)?;
            let static_op = gd.operators().get(&entry.operator_id);
            let faction_tags = static_op
                .map(|op| faction_tags_with_glossary(gd, &entry.operator_id, op))
                .unwrap_or_default();
            let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
            Some(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                rarity,
                gd.building(),
                false,
            ))
        })
        .collect();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    // The reference probe's exact layout: CC5, reception/office/workshop/training L3,
    // dorms L2/L2/L1/L1, 2 gold-strategy posts L3, 3 gold + 2 EXP factories L3, 2 plants L3.
    let mut rooms = vec![
        room("cc", "CONTROL", 5),
        room("rc", "MEETING", 3),
        room("hr", "HIRE", 3),
        room("ws", "WORKSHOP", 3),
        room("tr", "TRAINING", 3),
        room("d0", "DORMITORY", 2),
        room("d1", "DORMITORY", 2),
        room("d2", "DORMITORY", 1),
        room("d3", "DORMITORY", 1),
    ];
    rooms.extend((1..=2).map(|i| room(&format!("production_{i}"), "TRADING", 3)));
    for i in 3..=7 {
        let mut mf = room(&format!("production_{i}"), "MANUFACTURE", 3);
        mf.current_formula = Some(if i <= 5 { "F_GOLD" } else { "F_EXP" }.to_string());
        rooms.push(mf);
    }
    rooms.extend((8..=9).map(|i| room(&format!("production_{i}"), "POWER", 3)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };

    // Full service wiring: native economies + bundles via the oracle.
    let mut optimal_registry = registry.clone();
    let mut optimal_pins: Vec<(String, String)> = Vec::new();
    let native = myrtle_engine_base::scoring::pools::plan_optimal_economies(
        &profiles,
        &building,
        gd.building(),
        &registry,
    );
    for (bid, pct) in &native.overrides {
        optimal_registry.insert(
            bid.clone(),
            BuffResolutionStrategy::PoolPayoff { pct: *pct },
        );
    }
    optimal_pins.extend(native.pins.iter().cloned());
    let mut best = compute_optimal_assignment_with_pins(
        &profiles,
        &building,
        gd.building(),
        &optimal_registry,
        &drains,
        &optimal_pins,
    );
    for bundle in myrtle_engine_base::scoring::pools::candidate_bundles(
        &profiles,
        &building,
        gd.building(),
        &registry,
    ) {
        use myrtle_engine_base::assignment::assignment_value;
        let mut tr = optimal_registry.clone();
        for (bid, pct) in &bundle.overrides {
            tr.insert(
                bid.clone(),
                BuffResolutionStrategy::PoolPayoff { pct: *pct },
            );
        }
        let mut tp = optimal_pins.clone();
        tp.extend(bundle.pins.iter().cloned());
        let trial = compute_optimal_assignment_with_pins(
            &profiles,
            &building,
            gd.building(),
            &tr,
            &drains,
            &tp,
        );
        if assignment_value(&trial.rooms) > assignment_value(&best.rooms) + 1e-9 {
            best = trial;
        }
    }
    for r in &best.rooms {
        println!(
            "OURS {} {} {:?} eff {:.1} ops {:?}",
            r.slot_id, r.room_type, r.formula_type, r.total_efficiency, r.operators
        );
    }
}

/// Umiri's Famiglia Approval buffs OPERATORS, not the post: it stacks with
/// Amiya's post-wide +7% (different effect types), its credit lands on the
/// post holding the Siracusan, and the CC row labels it "per-room" when it
/// fires, "inactive" only when no team satisfies it.
#[test]
fn umiri_stacks_with_amiya_and_labels_per_room() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    use myrtle_engine_base::scoring::skill_ledger::LineDisposition;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    const UMIRI: &str = "char_4186_tmoris";
    const UMIRI_BUFF: &str = "control_tra_limit&spd2[000]";
    let build = |tp_crew: Vec<String>| {
        let mut building = UserBuilding {
            drone_capacity: None,
            rooms: vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)],
        };
        building.rooms[0].current_operators = vec!["char_002_amiya".into(), UMIRI.into()];
        building.rooms[1].current_operators = tp_crew.clone();
        let mut ids: Vec<&str> = vec!["char_002_amiya", UMIRI];
        ids.extend(tp_crew.iter().map(String::as_str));
        let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None)
    };

    // Lappland (Siracusa) in the post: Umiri fires.
    let asn = build(vec!["char_140_whitew".into(), "char_103_angel".into()]);
    let cc = asn.rooms.iter().find(|r| r.room_type == "CONTROL").unwrap();
    let umiri_cc = cc
        .ledger
        .iter()
        .find(|l| l.buff_id == UMIRI_BUFF)
        .expect("umiri line on cc row");
    assert_eq!(umiri_cc.disposition, LineDisposition::PerRoom);
    let tp = asn.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    let umiri_tp = tp
        .ledger
        .iter()
        .find(|l| l.buff_id == UMIRI_BUFF && l.from_control_center)
        .expect("umiri line on the post");
    assert_eq!(umiri_tp.disposition, LineDisposition::Contributes);
    assert!(
        (umiri_tp.speed_pct - 5.0).abs() < 1e-6,
        "one Siracusan = +5, got {}",
        umiri_tp.speed_pct
    );
    // Amiya's +7 is credited too: they STACK (different effect types).
    let amiya_tp = tp
        .ledger
        .iter()
        .find(|l| l.from_control_center && l.buff_id != UMIRI_BUFF && l.speed_pct > 0.0)
        .expect("amiya line on the post");
    assert!((amiya_tp.speed_pct - 7.0).abs() < 1e-6);

    // No Siracusan anywhere: inactive.
    let asn = build(vec!["char_103_angel".into()]);
    let cc = asn.rooms.iter().find(|r| r.room_type == "CONTROL").unwrap();
    let umiri_cc = cc.ledger.iter().find(|l| l.buff_id == UMIRI_BUFF).unwrap();
    assert_eq!(umiri_cc.disposition, LineDisposition::Inactive);

    // Texas counts too: her SubPower is siracusa even though her nation is
    // lungmen (the game's multi-affiliation system). Two Siracusans = +10.
    let asn = build(vec!["char_140_whitew".into(), "char_102_texas".into()]);
    let tp = asn.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    let umiri_tp = tp
        .ledger
        .iter()
        .find(|l| l.buff_id == UMIRI_BUFF && l.from_control_center)
        .expect("umiri line");
    assert!(
        (umiri_tp.speed_pct - 10.0).abs() < 1e-6,
        "two Siracusans = +10, got {}",
        umiri_tp.speed_pct
    );

    // Shamare cancels contributions sourced from her TEAMMATES (including
    // Umiri's per-operator grant to Lappland), while CC-sourced post-wide
    // globals (Amiya's +7) survive.
    let asn = build(vec!["char_254_vodfox".into(), "char_140_whitew".into()]);
    let tp = asn.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    let umiri_tp = tp
        .ledger
        .iter()
        .find(|l| l.buff_id == UMIRI_BUFF && l.from_control_center)
        .expect("umiri line");
    assert!(
        umiri_tp.speed_pct.abs() < 1e-6,
        "Shamare cancels the per-operator grant, got {}",
        umiri_tp.speed_pct
    );
    let amiya_tp = tp
        .ledger
        .iter()
        .find(|l| l.from_control_center && l.buff_id != UMIRI_BUFF && l.speed_pct > 0.0)
        .expect("amiya survives shamare");
    assert!((amiya_tp.speed_pct - 7.0).abs() < 1e-6);
}

/// Waai Fu's Cooperative Will pays +5% per FULL 5% her roommates' own
/// skills add, up to 40%. Beside Tragodia (Battle Records +35%) that is 35,
/// so the room reads 70. Priced per point of the room total it read the cap
/// (40) and, with a Control Center adding 2, a room the game shows at 72
/// came out at 77 (31010962).
#[test]
fn waai_fu_mirrors_her_roommates_per_full_five_percent() {
    const TRAGODIA: &str = "char_1042_phatm2";
    const WAAIFU: &str = "char_243_waaifu";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    let mirror = registry.get("manu_prod_spd_variable2[000]");
    assert!(
        matches!(
            mirror,
            Some(BuffResolutionStrategy::TeammateOutputMirroring { ratio, step, cap_pct })
                if (*ratio - 5.0).abs() < 1e-9 && (*step - 5.0).abs() < 1e-9 && (*cap_pct - 40.0).abs() < 1e-9
        ),
        "Cooperative Will parses as 5 per 5 up to 40, got {mirror:?}"
    );
    let tragodia = profile(gd, TRAGODIA);
    assert_eq!(tragodia.char_id, TRAGODIA, "Tragodia is in the game data");
    let roster = vec![tragodia, profile(gd, WAAIFU)];
    let eff = factory_efficiency(gd, &roster);
    assert!(
        (eff - 70.0).abs() < 1e-6,
        "Tragodia 35 + Waai Fu's mirror of it 35 = 70, got {eff}"
    );
}

/// The grade's 100% is never below what the Optimizer tab shows: copying
/// the tab's plan reads at most 100%, and a base is never held to a bar the
/// tab itself cannot reach.
#[test]
fn the_grade_ceiling_is_at_least_the_tab_optimal() {
    use myrtle_engine_base::assignment::sustained_assignment_value;
    use myrtle_engine_base::scoring::pools::{optimal_with_bundles, search_economy};
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let profiles = full_roster(gd);
    let building = generic_base();
    let economy = search_economy(&profiles, &building, gd.building(), &registry);
    let accepted = optimal_with_bundles(
        &profiles,
        &building,
        gd.building(),
        &registry,
        &economy.registry,
        &drains,
        &economy.pins,
    );
    let tab = sustained_assignment_value(
        &accepted.optimal,
        &profiles,
        &building,
        gd.building(),
        &registry,
        &drains,
    );
    // Station the tab's plan as the player's base and grade it: at most 100%.
    let ids: Vec<&str> = profiles.iter().map(|p| p.char_id.as_str()).collect();
    let inst_of = |id: &str| -> i64 { ids.iter().position(|x| *x == id).unwrap() as i64 + 1 };
    let mut chars = serde_json::Map::new();
    for id in &ids {
        chars.insert(inst_of(id).to_string(), serde_json::json!({ "charId": id }));
    }
    let mut slots = serde_json::Map::new();
    for r in &building.rooms {
        let crew: Vec<i64> = accepted
            .optimal
            .rooms
            .iter()
            .find(|a| a.slot_id == r.slot_id)
            .map(|a| a.operators.iter().map(|o| inst_of(o)).collect())
            .unwrap_or_default();
        slots.insert(
            r.slot_id.clone(),
            serde_json::json!({ "roomId": r.room_type, "level": r.level, "state": 2, "charInstIds": crew }),
        );
    }
    let mut factories = serde_json::Map::new();
    for a in &accepted.optimal.rooms {
        if a.room_type == "MANUFACTURE" {
            let formula_id = match a.formula_type.as_deref() {
                Some("F_GOLD") => "4",
                Some("F_DIAMOND") => "13",
                _ => "1",
            };
            factories.insert(
                a.slot_id.clone(),
                serde_json::json!({ "formulaId": formula_id }),
            );
        }
    }
    let json = serde_json::json!({
        "chars": chars,
        "rooms": { "MANUFACTURE": factories },
        "roomSlots": slots
    });
    let roster: Vec<RosterEntry> = ids.iter().map(|id| roster_entry(id)).collect();
    let g = grade_base(&roster, Some(&json), gd);
    assert!(
        g.utilization <= 1.0 + 1e-9 && g.utilization > 0.9,
        "the tab's plan grades near 100%, got {:.3} (tab sustained {tab:.0})",
        g.utilization
    );
}

/// A proposal is re-slotted onto the slots the player already runs the same
/// crews in: among rooms of one type and level the slots are interchangeable,
/// and a permuted proposal read as "every room changed" to a player who had
/// copied it (00980819, 2026-09-22).
#[test]
fn a_permuted_proposal_is_aligned_to_the_players_slots() {
    use myrtle_engine_base::assignment::align_rooms_to_current;
    use myrtle_engine_base::types::{BaseAssignment, RoomAssignment};
    let crew_a = vec!["a1".to_string(), "a2".to_string()];
    let crew_b = vec!["b1".to_string(), "b2".to_string()];
    let crew_c = vec!["c1".to_string()];
    let mut building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("mf0", "MANUFACTURE", 3),
            room("mf1", "MANUFACTURE", 3),
            room("mf2", "MANUFACTURE", 2),
        ],
    };
    building.rooms[0].current_operators = crew_a.clone();
    building.rooms[1].current_operators = crew_b.clone();
    building.rooms[2].current_operators = crew_c.clone();
    let mk = |slot: &str, level: i32, ops: &[String], formula: &str| RoomAssignment {
        slot_id: slot.into(),
        room_type: "MANUFACTURE".into(),
        level,
        formula_type: Some(formula.into()),
        operators: ops.to_vec(),
        ..Default::default()
    };
    let mut proposal = BaseAssignment {
        rooms: vec![
            mk("mf0", 3, &crew_b, "F_EXP"),
            mk("mf1", 3, &crew_a, "F_GOLD"),
            mk("mf2", 2, &crew_c, "F_GOLD"),
        ],
        total_production_efficiency: 0.0,
        bench: Vec::new(),
    };
    align_rooms_to_current(&building, &mut proposal);
    let slot_of = |ops: &[String]| {
        proposal
            .rooms
            .iter()
            .find(|r| r.operators == ops)
            .map(|r| (r.slot_id.clone(), r.formula_type.clone()))
    };
    assert_eq!(
        slot_of(&crew_a),
        Some(("mf0".into(), Some("F_GOLD".into()))),
        "crew A keeps its slot, recipe travels with it"
    );
    assert_eq!(slot_of(&crew_b), Some(("mf1".into(), Some("F_EXP".into()))));
    assert_eq!(
        slot_of(&crew_c),
        Some(("mf2".into(), Some("F_GOLD".into()))),
        "a level-2 room never trades with level-3 ones"
    );
}

#[test]
fn team_value_is_order_independent_and_rewards_staffing() {
    // `team_value` is what the rotation comparison uses to tell whether a player's current team
    // already ties the recommendation (Bryophyta vs a Dorothy-boosted Rhine operator). It must
    // not depend on operator order, and a staffed team must score at least as high as an empty
    // room, otherwise an equivalent team could be misjudged.
    let gd = load_game_data();
    let building = trading_post(3);
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let roster: Vec<_> = ["char_134_ifrit", "char_002_amiya", "char_003_kalts"]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let team: Vec<String> = vec!["char_134_ifrit".into(), "char_002_amiya".into()];
    let reversed: Vec<String> = team.iter().rev().cloned().collect();
    let v = team_value(
        &team,
        "TRADING",
        None,
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
    );
    let v_rev = team_value(
        &reversed,
        "TRADING",
        None,
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
    );
    assert!(
        (v - v_rev).abs() < 1e-9,
        "team_value must be order-independent, got {v} vs {v_rev}"
    );
    let empty = team_value(
        &[],
        "TRADING",
        None,
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
    );
    assert!(
        v >= empty,
        "a staffed team must score at least as high as an empty room"
    );
}
