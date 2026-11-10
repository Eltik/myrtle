use super::*;

#[test]
fn rhine_lab_faction_synergy_boosts_dorothy() {
    // Dorothy gains +5% per Rhine Lab operator in the factory. A factory of
    // Rhine Lab operators should out-produce the same Dorothy team with
    // non-Rhine teammates of equal base efficiency.
    let gd = load_game_data();
    const DOROTHY: &str = "char_4048_doroth";

    let rhine_team = vec![
        profile(gd, DOROTHY),
        profile(gd, "char_128_plosis"), // Ptilopsis, Rhine, +25%
        profile(gd, "char_108_silent"), // Silence, Rhine, +25%
    ];
    let mixed_team = vec![
        profile(gd, DOROTHY),
        profile(gd, "char_237_gravel"), // non-Rhine, +35%
        profile(gd, "char_159_peacok"), // non-Rhine, +35%
    ];

    let rhine = factory_efficiency(gd, &rhine_team);
    let mixed = factory_efficiency(gd, &mixed_team);

    // The non-Rhine teammates have higher flat efficiency (35 vs 25), but
    // Dorothy's +5% × 2 Rhine = +10% makes the Rhine team competitive. What is
    // asserted is that the faction bonus applies.
    let mixed_no_dorothy = factory_efficiency(
        gd,
        &[
            profile(gd, "char_237_gravel"),
            profile(gd, "char_159_peacok"),
        ],
    );
    let rhine_no_dorothy = factory_efficiency(
        gd,
        &[
            profile(gd, "char_128_plosis"),
            profile(gd, "char_108_silent"),
        ],
    );
    // Dorothy's marginal contribution on the Rhine team includes the faction
    // bonus; on the mixed team it does not.
    let dorothy_gain_rhine = rhine - rhine_no_dorothy;
    let dorothy_gain_mixed = mixed - mixed_no_dorothy;
    assert!(
        dorothy_gain_rhine > dorothy_gain_mixed + 5.0,
        "Dorothy should contribute more among Rhine Lab operators \
         (rhine gain {dorothy_gain_rhine:.1} vs mixed gain {dorothy_gain_mixed:.1})"
    );
}

#[test]
fn highmore_converts_rhine_skills_for_standardization_scaler() {
    // Mizuki scales +5% per Standardization skill. A Rhine operator (Silence)
    // is NOT a Standardization skill on its own, but Highmore converts Rhine
    // skills into Standardization, so Mizuki should then count Silence.
    let gd = load_game_data();
    const MIZUKI: &str = "char_437_mizuki"; // +5% per Standardization skill
    const HIGHMORE: &str = "char_4066_highmo"; // converts Rhine/Pinus -> Standardization
    const SILENCE: &str = "char_108_silent"; // Rhine Tech skill

    let fac = |ids: &[&str]| {
        factory_efficiency(
            gd,
            &ids.iter().map(|id| profile(gd, id)).collect::<Vec<_>>(),
        )
    };

    // Marginal value of adding the Rhine operator to a Mizuki room…
    let with_converter = fac(&[MIZUKI, HIGHMORE, SILENCE]) - fac(&[MIZUKI, HIGHMORE]);
    // …vs the same, but with no converter present (Silence stays "Rhine", uncounted).
    let without_converter = fac(&[MIZUKI, SILENCE]) - fac(&[MIZUKI]);

    assert!(
        with_converter > without_converter + 3.0,
        "Highmore's conversion should let Mizuki count the Rhine operator as a \
         Standardization skill (with converter {with_converter:.1} vs without {without_converter:.1})"
    );
}

#[test]
fn weedy_is_not_padded_into_a_normal_factory() {
    // Weedy zeroes her teammates' output, so she only belongs in her own
    // automation team. With a strong normal team already filling the beneficial
    // seats, she must NOT be padded into the leftover slot (which would wreck the
    // team). A harmless 0-value filler takes the seat instead.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    // One factory + one power plant (no trading post -> the factory makes EXP,
    // where gold specialists don't apply). The generic +30% pair are the normal
    // team; Texas has no factory skill and is the natural filler.
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("mf", "MANUFACTURE", 3), room("pp", "POWER", 3)],
    };
    let roster = vec![
        profile(gd, "char_4141_marcil"), // Marcille, generic +30%
        profile(gd, "char_242_otter"),   // Mayer, generic +30%
        profile(gd, "char_400_weedy"),   // Weedy, nullifying automation
        profile(gd, "char_102_texas"),   // Texas, trading-only (no factory value)
    ];

    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let factory = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "MANUFACTURE")
        .expect("a factory in the assignment");

    assert!(
        !factory.operators.iter().any(|o| o == "char_400_weedy"),
        "Weedy must not be padded into a normal factory (she nullifies teammates). Got: {:?}",
        factory.operators
    );
}

#[test]
fn metalwork_specialists_take_gold_freeing_generics_for_exp() {
    // A Gold factory should be staffed by Metalwork (Gold-only) specialists, not by
    // a generic +30% who serves any product equally; the generic is freed for the
    // EXP factory that has no dedicated operator. This is the Marcille vs Haze case.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    // Two factories (one Gold to feed the trading post, one EXP) + a trading post.
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("mf0", "MANUFACTURE", 3),
            room("mf1", "MANUFACTURE", 3),
            room("tp", "TRADING", 3),
        ],
    };
    // Marcille is listed FIRST to make the guard meaningful regardless of roster
    // order: the Gold factory must still resolve to the Metalwork specialists, with
    // the generic flowing to EXP.
    let roster = vec![
        profile(gd, "char_4141_marcil"), // Marcille, generic +30% (ties Haze)
        profile(gd, "char_237_gravel"),  // Gravel, Metalwork +35% (Gold only)
        profile(gd, "char_141_nights"),  // Haze, Metalwork +30% (Gold only)
        profile(gd, "char_4106_bryota"), // Bryophyta, Metalwork +30% (Gold only)
        profile(gd, "char_102_texas"),   // a trader to staff the trading post
        profile(gd, "char_103_angel"),   // Exusiai, trader
    ];

    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let gold = asn
        .rooms
        .iter()
        .find(|r| r.formula_type.as_deref() == Some("F_GOLD"))
        .expect("a gold factory");
    let exp = asn
        .rooms
        .iter()
        .find(|r| r.formula_type.as_deref() == Some("F_EXP"))
        .expect("an exp factory");

    assert!(
        !gold.operators.iter().any(|o| o == "char_4141_marcil"),
        "the generic Marcille should yield the Gold factory to Metalwork specialists. Gold: {:?}",
        gold.operators
    );
    assert!(
        exp.operators.iter().any(|o| o == "char_4141_marcil"),
        "the freed generic Marcille should staff the EXP factory. EXP: {:?}",
        exp.operators
    );
}

#[test]
fn quartz_recipe_scaling_counts_distinct_recipe_types_not_factories() {
    // Quartz "Precise Scheduling" (trade_ord_spd&formula[000]): base +30% trading, plus +2% per
    // recipe TYPE being processed at Factories. That scales on the number of distinct formulas the
    // base runs (gold + EXP = 2), NOT the factory count: four gold/EXP factories still process two
    // recipe types. Her buff resolves to a FacilityCountScaling targeting the synthetic
    // MANUFACTURE_RECIPE_TYPES count with a +30% base, so a 2-recipe base reads +34% (30 + 2×2).

    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _drains) = build_registry(&gd.building().buffs, &name_to_char);

    let strat = registry
        .get("trade_ord_spd&formula[000]")
        .expect("Quartz's trading buff should be in the registry");

    match strat {
        BuffResolutionStrategy::FacilityCountScaling {
            target_room,
            per_unit_pct,
            base_pct,
            per_level,
            ..
        } => {
            assert_eq!(
                target_room, "MANUFACTURE_RECIPE_TYPES",
                "recipe scaling counts distinct recipe types, not the factory count"
            );
            assert!(!per_level, "scaling is per-recipe-type, not per-level");
            assert!(
                (*base_pct - 30.0).abs() < 0.01,
                "base should be +30%, got {base_pct}"
            );
            assert!(
                (*per_unit_pct - 2.0).abs() < 0.01,
                "per-recipe should be +2%, got {per_unit_pct}"
            );
        }
        _ => panic!("Quartz should be FacilityCountScaling (base + recipe-type scaling)"),
    }
}

#[test]
fn weedy_is_treated_as_power_scaling_automation() {
    // Weedy nullifies teammates and scales with POWER PLANTS, not teammates. In a
    // base with more power plants she must produce more, proving she's modeled as
    // facility-scaling automation, not as a Shamare-style per-teammate nullifier.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let weedy = vec![profile(gd, "char_400_weedy")];
    let factory_with_power = |n: usize| {
        let mut rooms = vec![UserRoom {
            slot_id: "f".into(),
            room_type: "MANUFACTURE".into(),
            level: 3,
            ..Default::default()
        }];
        for i in 0..n {
            rooms.push(UserRoom {
                slot_id: format!("p{i}"),
                room_type: "POWER".into(),
                level: 3,
                ..Default::default()
            });
        }
        let asn = compute_optimal_assignment(
            &weedy,
            &UserBuilding {
                drone_capacity: None,
                rooms,
            },
            gd.building(),
            &registry,
            &drains,
        );
        asn.rooms
            .iter()
            .find(|r| r.room_type == "MANUFACTURE")
            .map_or(0.0, |r| r.total_efficiency)
    };
    assert!(
        factory_with_power(3) > factory_with_power(1),
        "Weedy's output must scale with the number of power plants"
    );
}

#[test]
fn gold_exp_split_is_yield_based() {
    // One Trading Post can only sell roughly one factory's worth of gold per day.
    // With four factories, a yield-based optimizer recognizes that extra gold
    // factories are wasted (no post to sell their gold) and switches them to EXP,
    // which is worth more than unsold gold.
    let gd = load_game_data();
    let profiles = full_roster(gd);

    let mut rooms = vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)];
    rooms.extend((0..4).map(|i| room(&format!("f{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("p{i}"), "POWER", 3)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };

    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let optimal =
        compute_optimal_assignment(&profiles, &building, gd.building(), &registry, &drains);

    let count = |f: &str| {
        optimal
            .rooms
            .iter()
            .filter(|r| r.formula_type.as_deref() == Some(f))
            .count()
    };
    assert!(
        count("F_EXP") >= 2,
        "with only one Trading Post, excess gold factories should become EXP \
         (F_GOLD={}, F_EXP={})",
        count("F_GOLD"),
        count("F_EXP")
    );
}

#[test]
fn cross_formula_reallocation_frees_a_generic_for_exp() {
    // Two generic +30% operators would both pile into the single Gold factory, but
    // the second's gold is unsold (one trading post). The cross-formula pass frees
    // one of them for the empty EXP factory, where it produces real value.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("g", "MANUFACTURE", 3),
            room("e", "MANUFACTURE", 3),
            room("tp", "TRADING", 3),
        ],
    };
    let roster = vec![
        profile(gd, "char_4141_marcil"), // Marcille, generic +30%
        profile(gd, "char_242_otter"),   // Mayer, generic +30%
        profile(gd, "char_103_angel"),   // Exusiai, trader
    ];

    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let exp = asn
        .rooms
        .iter()
        .find(|r| r.formula_type.as_deref() == Some("F_EXP"))
        .expect("an exp factory");
    assert!(
        !exp.operators.is_empty(),
        "a surplus generic should be reallocated from Gold to the EXP factory"
    );
}

#[test]
fn bubbles_tiers_take_priority_over_vermeils_recycling() {
    // Bubble's Bigger is Better! "does not stack with Recycling and takes
    // priority over it": with Bubble in the factory, Vermeil's 2% per
    // capacity point does not apply, and the room reads Bubble's 1% per point
    // alone (Bubble +10, Vermeil +8, Click +15 on Battle Records = 33; the
    // game shows +0.36 with the three 1% innates). Without Bubble, Vermeil's
    // Recycling reads the same points at 2% (46). User-verified 2026-09-22.
    use myrtle_engine_base::scoring::skill_ledger::LineDisposition;
    const BUBBLE: &str = "char_381_bubble";
    const VERMEIL: &str = "char_190_clour";
    const CLICK: &str = "char_328_cammou";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let factory = |crew: &[&str]| {
        let mut mf = room("mf", "MANUFACTURE", 3);
        mf.current_formula = Some("F_EXP".into());
        mf.current_operators = crew.iter().map(|s| (*s).to_string()).collect();
        let building = UserBuilding {
            drone_capacity: None,
            rooms: vec![mf],
        };
        let roster: Vec<_> = crew.iter().map(|id| profile(gd, id)).collect();
        let asn = myrtle_engine_base::assignment::compute_current_assignment(
            &roster,
            &building,
            gd.building(),
            &registry,
            &drains,
            None,
        );
        asn.rooms.into_iter().next().expect("the factory")
    };
    let with_bubble = factory(&[BUBBLE, VERMEIL, CLICK]);
    assert!(
        (with_bubble.total_efficiency - 33.0).abs() < 1e-6,
        "Bubble's 1% per point alone: got {:.1}",
        with_bubble.total_efficiency
    );
    let recycling = with_bubble
        .ledger
        .iter()
        .find(|l| l.operator_id == VERMEIL && l.buff_id.starts_with("manu_prod_spd_variable["))
        .expect("Vermeil's Recycling line");
    assert!(
        recycling.disposition == LineDisposition::Covered && recycling.speed_pct.abs() < 1e-9,
        "Recycling reads covered under Bubble, got {:?} {:.1}",
        recycling.disposition,
        recycling.speed_pct
    );
    assert!(
        recycling
            .note
            .as_deref()
            .is_some_and(|n| n.contains("Bigger is Better") && n.contains("priority")),
        "the line names the skill that takes priority: {:?}",
        recycling.note
    );
    let tiers = with_bubble
        .ledger
        .iter()
        .find(|l| l.operator_id == BUBBLE && l.buff_id.starts_with("manu_prod_spd_variable3["))
        .expect("Bubble's Bigger is Better! line");
    assert!(
        (tiers.speed_pct - 33.0).abs() < 1e-6,
        "Bubble's tiers read their own +33, not the -33 of a probe that resurrects Recycling: {:.1}",
        tiers.speed_pct
    );
    let without_bubble = factory(&[VERMEIL, CLICK]);
    assert!(
        (without_bubble.total_efficiency - 46.0).abs() < 1e-6,
        "Vermeil's 2% per point without Bubble: got {:.1}",
        without_bubble.total_efficiency
    );
}

#[test]
fn vermeil_converts_her_own_capacity() {
    // Vermeil's E1 scales factory productivity with the WHOLE factory's capacity limit,
    // including her own +8 from her E0. A solo Vermeil should therefore convert that +8 into a
    // real productivity bonus (~+16% at +2%/cap), not score a flat 0 (the self-exclusion bug).
    let gd = load_game_data();
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("mf", "MANUFACTURE", 3)],
    };
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let roster = vec![profile(gd, "char_190_clour")]; // Vermeil (E1+ in game data fixture)
    let solo = team_value(
        &["char_190_clour".into()],
        "MANUFACTURE",
        Some("F_GOLD"),
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
    );
    assert!(
        solo > 1.05,
        "a solo Vermeil should convert her own +8 capacity into productivity (>+5%), got {solo:.3}"
    );
}

#[test]
fn factory_formula_assignment_respects_the_players_current_layout() {
    // The optimizer picks the gold/EXP COUNT, but must lay it onto the slots the player ALREADY
    // runs that way, otherwise it flips every factory's formula and the whole comparison reads as
    // "change everything" (the reported "very wrong" plan).
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let fac = |slot: &str, formula: &str| UserRoom {
        slot_id: slot.into(),
        room_type: "MANUFACTURE".into(),
        level: 3,
        current_formula: Some(formula.into()),
        ..Default::default()
    };
    // 2 trading posts force the split to at least 2 gold; the player runs mf0/mf1 as EXP and
    // mf2/mf3 as GOLD.
    let mut rooms = vec![room("cc", "CONTROL", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.push(fac("mf0", "F_EXP"));
    rooms.push(fac("mf1", "F_EXP"));
    rooms.push(fac("mf2", "F_GOLD"));
    rooms.push(fac("mf3", "F_GOLD"));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };

    let asn = compute_optimal_assignment(
        &full_roster(gd),
        &building,
        gd.building(),
        &registry,
        &drains,
    );
    let formula_of = |slot: &str| -> Option<String> {
        asn.rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .and_then(|r| r.formula_type.clone())
    };
    // The split is >= 2 gold (>= trading posts), so the player's two GOLD slots stay gold.
    assert_eq!(
        formula_of("mf2").as_deref(),
        Some("F_GOLD"),
        "mf2 (your gold slot) should stay gold"
    );
    assert_eq!(
        formula_of("mf3").as_deref(),
        Some("F_GOLD"),
        "mf3 (your gold slot) should stay gold"
    );
}

#[test]
fn quartz_value_reflects_distinct_recipe_types_not_factory_count() {
    // A base with FOUR factories but only TWO distinct recipe types (2 gold + 2 EXP) processes two
    // recipe types, so Quartz reads +34% (30 + 2×2), not the +38% a raw 4-factory count would give.
    use myrtle_engine_base::assignment::team_value;
    const QUARTZ: &str = "char_4063_quartz";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let profiles = vec![profile(gd, QUARTZ)];
    let fac = |slot: &str, formula: &str| UserRoom {
        slot_id: slot.into(),
        room_type: "MANUFACTURE".into(),
        level: 3,
        current_formula: Some(formula.into()),
        ..Default::default()
    };
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("tp", "TRADING", 3),
            fac("m0", "F_GOLD"),
            fac("m1", "F_GOLD"),
            fac("m2", "F_EXP"),
            fac("m3", "F_EXP"),
        ],
    };
    let v = team_value(
        &[QUARTZ.to_string()],
        "TRADING",
        None,
        &profiles,
        &building,
        gd.building(),
        &registry,
        &drains,
    );
    assert!(
        (v - 1.34).abs() < 0.01,
        "Quartz should read +34% (2 recipe types), not +38% (4 factories): got {v}"
    );
}

/// Astgenne the Lightchaser's "+5 Storage Capacity for each Rhine Tech-type
/// skill in that Factory" is CAPACITY, not productivity: her room's speed is
/// her Rhine Tech β plus Dorothy's counts, and her capacity is 5 per Rhine
/// Tech skill (her own, Dorothy's, Silence's = 15).
#[test]
fn astgenne_backpack_is_storage_capacity_not_productivity() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    use myrtle_engine_base::scoring::ledger::op_capacity_limit;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const ASTGENNE: &str = "char_1047_halo2";
    const DOROTHY: &str = "char_4048_doroth";
    const SILENCE2: &str = "char_1031_slent2";
    let mut rooms = vec![room("mf", "MANUFACTURE", 3)];
    rooms[0].current_operators = vec![ASTGENNE.into(), DOROTHY.into(), SILENCE2.into()];
    rooms[0].current_formula = Some("F_GOLD".into());
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let roster: Vec<_> = [ASTGENNE, DOROTHY, SILENCE2]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn =
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
    let eff = asn.rooms[0].total_efficiency;
    // Rhine Tech: Astgenne 25 + Dorothy 25 + Silence 30, Dorothy +5 x 3 skills.
    assert!(
        (eff - 95.0).abs() < 1e-9,
        "no productivity from the backpack: {eff}"
    );
    let present: std::collections::HashSet<String> = [ASTGENNE, DOROTHY, SILENCE2]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let cap = op_capacity_limit(
        &roster[0],
        "MANUFACTURE",
        Some("F_GOLD"),
        &registry,
        gd.building(),
        &present,
    );
    assert!(
        cap >= 5.0,
        "her own Rhine Tech skill is the capacity floor: {cap}"
    );
}

/// Flametail's "each Pinus Sylvestris Operator assigned to Factories have
/// +10% productivity towards Battle Records and -10% towards Precious
/// Metals" is a product-split faction global: three Pinus knights making
/// Battle Records read +30 from her seat, making Pure Gold -30.
#[test]
fn flametail_buffs_pinus_operators_by_product() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const FLAMETAIL: &str = "char_420_flamtl";
    let knights = ["char_430_fartth", "char_431_ashlok", "char_496_wildmn"];
    let eff_for = |formula: &str| {
        let mut rooms = vec![room("cc", "CONTROL", 5), room("mf", "MANUFACTURE", 3)];
        rooms[0].current_operators = vec![FLAMETAIL.into()];
        rooms[1].current_operators = knights.iter().map(|s| (*s).to_string()).collect();
        rooms[1].current_formula = Some(formula.into());
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        let mut ids = vec![FLAMETAIL];
        ids.extend(knights);
        let roster: Vec<_> = ids.iter().map(|id| profile(gd, id)).collect();
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "mf")
            .map_or(0.0, |r| r.total_efficiency)
    };
    assert!(
        (eff_for("F_EXP") - 105.0).abs() < 1e-9,
        "75 + 3 x 10 on Battle Records: {}",
        eff_for("F_EXP")
    );
    assert!(
        (eff_for("F_GOLD") - 45.0).abs() < 1e-9,
        "75 - 3 x 10 on Precious Metals: {}",
        eff_for("F_GOLD")
    );
}

/// Nasti's "for each Rhine Lab Operator in the Base (caps at 5), Precious
/// Metal productivity +3%" counts herself: three Rhine Lab operators in the
/// factory = +9, on top of the Rhine Tech skills (25 + 25 + 30) and Dorothy's
/// three-skill count (15) = 104 (user-verified 106 with Wang's +2).
#[test]
fn nasti_counts_every_rhine_lab_operator_in_the_base_including_herself() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const NASTI: &str = "char_4212_nasti";
    const DOROTHY: &str = "char_4048_doroth";
    const SILENCE2: &str = "char_1031_slent2";
    let mut rooms = vec![room("mf", "MANUFACTURE", 3)];
    rooms[0].current_operators = vec![NASTI.into(), DOROTHY.into(), SILENCE2.into()];
    rooms[0].current_formula = Some("F_GOLD".into());
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let roster: Vec<_> = [NASTI, DOROTHY, SILENCE2]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn =
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
    let eff = asn.rooms[0].total_efficiency;
    assert!(
        (eff - 104.0).abs() < 1e-9,
        "80 + Dorothy 15 + Nasti 3 x 3: {eff}"
    );
}

/// Bubble's Bigger is Better! pays per capacity POINT, tiered by each
/// occupant's own bonus (user-verified 2026-09-10: Bubble 10, Ceobe 8, Bena
/// 17 read +0.76 in-game = 10 + 8 + 51 + Ceobe's ramp - Bena's 20 + Wang 2).
#[test]
fn bubble_pays_per_capacity_point_by_tier() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const BUBBLE: &str = "char_381_bubble";
    const CEOBE: &str = "char_2013_cerber";
    const BENA: &str = "char_369_bena";
    let mut rooms = vec![room("mf", "MANUFACTURE", 3)];
    rooms[0].current_operators = vec![BUBBLE.into(), CEOBE.into(), BENA.into()];
    rooms[0].current_formula = Some("F_EXP".into());
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let roster: Vec<_> = [BUBBLE, CEOBE, BENA]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn =
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
    let eff = asn.rooms[0].total_efficiency;
    // 69 from Bubble + Ceobe's 24h-averaged ramp (24.5) - Bena's 20.
    assert!((73.0..=74.0).contains(&eff), "per-point tiers: {eff}");
}

/// Snegurochka's Workflow Optimization grants the ROOM +10% per occupant
/// (and +5 capacity), which survives an automation wipe like the plant
/// grants: beside Eunectes on three plants the room reads 30 + 20.
#[test]
fn snegurochka_room_grant_survives_the_automation_wipe() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const SNEG: &str = "char_4208_wintim";
    const EUNECTES: &str = "char_416_zumama";
    let roster: Vec<_> = [SNEG, EUNECTES].iter().map(|id| profile(gd, id)).collect();
    let eff = |crew: &[&str]| {
        let mut rooms = vec![room("mf", "MANUFACTURE", 3)];
        rooms.extend((0..3).map(|i| room(&format!("pp{i}"), "POWER", 3)));
        rooms[0].current_operators = crew.iter().map(|s| (*s).to_string()).collect();
        rooms[0].current_formula = Some("F_EXP".into());
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "mf")
            .map_or(0.0, |r| r.total_efficiency)
    };
    assert!(
        (eff(&[SNEG]) - 10.0).abs() < 1e-9,
        "alone: {}",
        eff(&[SNEG])
    );
    assert!(
        (eff(&[SNEG, EUNECTES]) - 50.0).abs() < 1e-9,
        "beside Eunectes: {}",
        eff(&[SNEG, EUNECTES])
    );
}

/// Automation suppression is CLAUSE-GATED to the room its buff applies in:
/// Weedy's factory automation must not wipe a Trading Post team she's merely
/// being evaluated in (the legacy engine triggered on carrying the buff at all).
#[test]
fn automation_wipe_is_room_gated() {
    const WEEDY: &str = "char_400_weedy";
    const TEXAS: &str = "char_102_texas";
    const LAPPLAND: &str = "char_140_whitew";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let ops: Vec<OperatorBaseProfile> = [WEEDY, TEXAS, LAPPLAND]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("tp0", "TRADING", 3)],
    };
    let with_weedy = team_value(
        &[TEXAS.to_string(), LAPPLAND.to_string(), WEEDY.to_string()],
        "TRADING",
        None,
        &ops,
        &building,
        gd.building(),
        &registry,
        &drains,
    );
    let without = team_value(
        &[TEXAS.to_string(), LAPPLAND.to_string()],
        "TRADING",
        None,
        &ops,
        &building,
        gd.building(),
        &registry,
        &drains,
    );
    // Weedy adds nothing in a Trading Post, but she must not DESTROY the
    // Texas+Lappland synergy either; the pair's value survives her presence.
    assert!(
        with_weedy >= without - 1e-9,
        "Weedy in a trading post must not wipe teammates: with={with_weedy:.4} without={without:.4}"
    );
    assert!(
        without > 1.5,
        "sanity: Texas+Lappland is a strong trading pair ({without:.4})"
    );
}

/// Capacity trades are signed: Wulfenite's Go-Getter ("+20% and capacity
/// limit -8") nets against her Storage Guru +16 in Vermeil's Recycling basis.
/// The game's own numbers: Vermeil/Pallas/Wulfenite = 93%, Vermeil/Scene/
/// Pallas = 96% ramping to 106% (Scene averages ~105% over a shift).
#[test]
fn signed_capacity_reproduces_the_games_vermeil_teams() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    assert!(
        matches!(
            registry.get("manu_prod_spd&limit&cost[300]"),
            Some(BuffResolutionStrategy::EfficiencyWithOrderLimit { efficiency, order_limit: -8 }) if (*efficiency - 20.0).abs() < 1e-9
        ),
        "Go-Getter is +20% with capacity -8: {:?}",
        registry.get("manu_prod_spd&limit&cost[300]")
    );
    let team = |crew: &[&str]| -> f64 {
        let mut building = UserBuilding {
            drone_capacity: None,
            rooms: vec![room("mf", "MANUFACTURE", 3)],
        };
        building.rooms[0].current_operators = crew.iter().map(|s| (*s).to_string()).collect();
        building.rooms[0].current_formula = Some("F_EXP".into());
        let roster: Vec<OperatorBaseProfile> = crew.iter().map(|id| profile(gd, id)).collect();
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "mf")
            .expect("factory")
            .total_efficiency
    };
    let vpw = team(&["char_190_clour", "char_485_pallas", "char_4171_wulfen"]);
    assert!(
        (vpw - 93.0).abs() < 1e-6,
        "Vermeil/Pallas/Wulfenite = 93 in-game, got {vpw}"
    );
    let vsp = team(&["char_190_clour", "char_336_folivo", "char_485_pallas"]);
    assert!(
        vsp > vpw && (vsp - (56.0 + 25.0 + 23.75)).abs() < 1e-6,
        "Vermeil/Scene/Pallas ~105 (Scene's ramp 23.75 over 24h in whole hours), got {vsp}"
    );
}

/// "+5% for each Rhine Tech-type skill in this Factory" counts SKILLS by
/// name, the holder's own included: Silence's Rhine Tech γ and Dorothy's own
/// Rhine Tech β make +10 (as in-game). Rosmontis is Rhine Lab by faction but
/// carries no Rhine Tech skill, so she adds nothing to the count.
#[test]
fn dorothy_counts_rhine_tech_skills_including_her_own() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let line = |crew: &[&str]| -> f64 {
        let mut building = UserBuilding {
            drone_capacity: None,
            rooms: vec![room("mf", "MANUFACTURE", 3)],
        };
        building.rooms[0].current_operators = crew.iter().map(|s| (*s).to_string()).collect();
        building.rooms[0].current_formula = Some("F_EXP".into());
        let roster: Vec<OperatorBaseProfile> = crew.iter().map(|id| profile(gd, id)).collect();
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms[0]
            .ledger
            .iter()
            .find(|l| l.buff_id == "manu_skill_spd1[010]")
            .map_or(0.0, |l| l.speed_pct)
    };
    let with_silence = line(&["char_391_rosmon", "char_1031_slent2", "char_4048_doroth"]);
    assert!(
        (with_silence - 10.0).abs() < 1e-6,
        "Silence's + her own Rhine Tech = +10, got {with_silence}"
    );
    let alone = line(&["char_4048_doroth"]);
    assert!(
        (alone - 5.0).abs() < 1e-6,
        "her own Rhine Tech β alone = +5, got {alone}"
    );
}

/// A Battle Records factory with Vermeil's capacity comp on the roster
/// (Vermeil Recycling 2% per capacity point, Scene Editing α +12, Pallas
/// Domain of Wisdom +8, Junkman +8: 56 + Time-Lapse 24 + Plan to Victory 25
/// = 105) beside a deep pool of flat +25/+30 hands must field the comp. The
/// candidate cut ranks by optimistic bound and the bound counts speed only,
/// so Scene ranked at her averaged ramp (24.0) behind every flat 25 and the
/// comp was never enumerated: a reporter's optimizer fielded
/// Mizuki/Jessica/Vanilla (90) over Vermeil/Scene/Pallas (105) on 2026-09-19.
#[test]
fn vermeil_capacity_comp_survives_a_deep_flat_pool() {
    const VERMEIL: &str = "char_190_clour";
    const SCENE: &str = "char_336_folivo";
    const PALLAS: &str = "char_485_pallas";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let roster: Vec<_> = [
        VERMEIL,
        SCENE,
        PALLAS,
        "char_437_mizuki",
        "char_235_jesica",
        "char_240_wyvern",
        "char_242_otter",
        "char_4141_marcil",
        "char_1031_slent2",
        "char_496_wildmn",
        "char_484_robrta",
        "char_431_ashlok",
        "char_430_fartth",
        "char_4212_nasti",
        "char_4066_highmo",
        "char_4048_doroth",
        "char_210_stward",
        "char_181_flower",
        "char_135_halo",
        "char_128_plosis",
        "char_108_silent",
        "char_1047_halo2",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    assert!(
        roster.len() >= 20,
        "the pool is in the game data: {}",
        roster.len()
    );
    let mut building = single_room("MANUFACTURE", 3);
    building.rooms[0].current_formula = Some("F_EXP".into());
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let factory = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "MANUFACTURE")
        .expect("the factory is staffed");
    let mut crew = factory.operators.clone();
    crew.sort();
    let mut want = vec![VERMEIL.to_string(), SCENE.to_string(), PALLAS.to_string()];
    want.sort();
    assert_eq!(
        crew, want,
        "Vermeil/Scene/Pallas at 105 beats every flat crew; fielded {crew:?} at {:.1}",
        factory.total_efficiency
    );
}

/// A scoped planner run freezes the rooms outside its scope: their crews and
/// recipes stay exactly as drafted, their operators are unavailable to the
/// scoped room, and they still feed the yield the scoped room is planned
/// beside. Held only as (operator, room type) pins, "optimize this room
/// only" re-crewed every other room, flipped a factory's recipe and left the
/// frozen crews unseated (31010962, 2026-09-20).
#[test]
fn a_frozen_room_keeps_its_crew_and_recipe_while_the_scoped_room_is_planned() {
    const VERMEIL: &str = "char_190_clour";
    const SCENE: &str = "char_336_folivo";
    const PALLAS: &str = "char_485_pallas";
    const NASTI: &str = "char_4212_nasti";
    const DOROTHY: &str = "char_4048_doroth";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let roster: Vec<_> = [
        VERMEIL,
        SCENE,
        PALLAS,
        NASTI,
        DOROTHY,
        "char_123_fang",
        "char_133_mm",
        "char_502_nblade",
        "char_124_kroos",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    assert!(
        roster.len() >= 8,
        "the roster is in the game data: {}",
        roster.len()
    );
    // The frozen factory runs Nasti/Dorothy on gold, drafted by the player;
    // the scoped factory is empty and wants Battle Records.
    let mut frozen = room("mf_frozen", "MANUFACTURE", 3);
    frozen.current_formula = Some("F_GOLD".into());
    frozen.current_operators = vec![NASTI.into(), DOROTHY.into(), "char_123_fang".into()];
    frozen.frozen = true;
    let mut open = room("mf_open", "MANUFACTURE", 3);
    open.current_formula = Some("F_EXP".into());
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![frozen, open, room("tp", "TRADING", 3)],
    };
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let by_slot = |slot: &str| {
        asn.rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .unwrap_or_else(|| panic!("{slot} is in the plan"))
    };
    let kept = by_slot("mf_frozen");
    assert_eq!(
        kept.operators,
        vec![
            NASTI.to_string(),
            DOROTHY.to_string(),
            "char_123_fang".to_string()
        ],
        "the frozen crew is kept verbatim"
    );
    assert_eq!(
        kept.formula_type.as_deref(),
        Some("F_GOLD"),
        "the frozen recipe is kept"
    );
    assert!(
        kept.total_efficiency > 0.0,
        "the frozen room is scored as drafted"
    );
    let planned = by_slot("mf_open");
    assert!(
        planned
            .operators
            .iter()
            .all(|o| !kept.operators.contains(o)),
        "the scoped room cannot take a frozen operator, got {:?}",
        planned.operators
    );
    assert!(
        planned.operators.iter().any(|o| o == VERMEIL),
        "the scoped room is planned from the free pool, got {:?}",
        planned.operators
    );
    assert_eq!(
        planned.formula_type.as_deref(),
        Some("F_EXP"),
        "one post is fed by the frozen gold factory, so the scoped room stays on Battle Records"
    );
}

/// An automation leader's wipe spares facility-scaled partners, so the
/// automation team admits them: Weedy beside Purestream (+20% per Trading
/// Post) reads more than Weedy beside Eunectes when the base has two posts
/// and few plants. And when the greedy room order has already seated the
/// partner in a normal room, she is poached whenever the base as a whole
/// gains (00980819: Rosmontis+Purestream 116 and Weedy/Eunectes 77, where
/// Weedy/Purestream 117 leaves Rosmontis a partner).
#[test]
fn weedy_takes_a_facility_scaled_partner_over_a_second_nullifier() {
    const WEEDY: &str = "char_400_weedy";
    const EUNECTES: &str = "char_416_zumama";
    const PURESTREAM: &str = "char_385_finlpp";
    const NASTI: &str = "char_4212_nasti";
    const DOROTHY: &str = "char_4048_doroth";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let purestream = PURESTREAM.to_string();
    // Traders crew the posts, so the gold the factories make is sold and a
    // factory hand is never worth more in an empty post.
    let roster: Vec<_> = [
        WEEDY,
        EUNECTES,
        PURESTREAM,
        NASTI,
        DOROTHY,
        "char_123_fang",
        "char_133_mm",
        "char_102_texas",
        "char_140_whitew",
        "char_103_angel",
        "char_4193_lemuen",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    assert!(
        roster.len() >= 6,
        "the roster is in the game data: {}",
        roster.len()
    );
    let mut rooms = Vec::new();
    for i in 0..2 {
        let mut r = room(&format!("mf{i}"), "MANUFACTURE", 2);
        r.current_formula = Some("F_GOLD".into());
        rooms.push(r);
    }
    rooms.push(room("tp0", "TRADING", 3));
    rooms.push(room("tp1", "TRADING", 2));
    rooms.push(room("pp0", "POWER", 3));
    rooms.push(room("pp1", "POWER", 3));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let weedy_room = asn
        .rooms
        .iter()
        .find(|r| r.operators.iter().any(|o| o == WEEDY))
        .expect("Weedy leads an automation factory");
    assert!(
        weedy_room.operators.contains(&purestream),
        "Purestream's per-post skill survives Weedy's wipe and beats Eunectes here, got {:?}",
        weedy_room.operators
    );
}

/// A player who runs no gold factory at all sells from stock: their posts
/// earn AND every factory makes EXP. A plan that switched their lone
/// factory to gold fed a post that was already fed and threw the EXP away;
/// the 2026-09-22 census found 27 such bases graded at half their own
/// yield. Zero gold is tried only where every factory carries a recorded
/// non-gold recipe; a base with no recipes (a draft, a fixture) or any gold
/// factory is fed the ordinary way.
#[test]
fn a_base_that_runs_no_gold_keeps_its_exp_factory() {
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let roster: Vec<_> = [
        "char_235_jesica",
        "char_502_nblade",
        "char_123_fang",
        "char_121_lava",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .collect();
    let plan = |recipe: Option<&str>| {
        let mut mf = room("mf", "MANUFACTURE", 1);
        mf.current_formula = recipe.map(str::to_string);
        let building = UserBuilding {
            drone_capacity: None,
            rooms: vec![room("tp", "TRADING", 1), mf],
        };
        let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
        asn.rooms
            .iter()
            .find(|r| r.room_type == "MANUFACTURE")
            .and_then(|r| r.formula_type.clone())
    };
    assert_eq!(
        plan(Some("F_EXP")).as_deref(),
        Some("F_EXP"),
        "a recorded EXP-only base keeps EXP"
    );
    assert_eq!(
        plan(None).as_deref(),
        Some("F_GOLD"),
        "a base with no recorded recipe feeds its post"
    );
    assert_eq!(
        plan(Some("F_GOLD")).as_deref(),
        Some("F_GOLD"),
        "a base running gold keeps feeding its post"
    );
}

/// Control for the golden above: the same comp with a pool shallow enough
/// that nobody is cut, and the comp alone. Both must read the comp at its
/// ledger value, which isolates the candidate cut as the cause.
#[test]
fn vermeil_capacity_comp_control_shallow_pool_and_alone() {
    const VERMEIL: &str = "char_190_clour";
    const SCENE: &str = "char_336_folivo";
    const PALLAS: &str = "char_485_pallas";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let run = |ids: &[&str]| {
        let roster: Vec<_> = ids.iter().map(|id| profile(gd, id)).collect();
        let mut building = single_room("MANUFACTURE", 3);
        building.rooms[0].current_formula = Some("F_EXP".into());
        let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
        let f = asn
            .rooms
            .iter()
            .find(|r| r.room_type == "MANUFACTURE")
            .unwrap();
        let mut crew = f.operators.clone();
        crew.sort();
        (crew, f.total_efficiency)
    };
    let (alone, alone_eff) = run(&[VERMEIL, SCENE, PALLAS]);
    eprintln!("ALONE {alone:?} {alone_eff:.1}");
    let (shallow, shallow_eff) = run(&[
        VERMEIL,
        SCENE,
        PALLAS,
        "char_437_mizuki",
        "char_235_jesica",
        "char_240_wyvern",
        "char_242_otter",
        "char_4141_marcil",
        "char_1031_slent2",
        "char_4048_doroth",
        "char_484_robrta",
        "char_431_ashlok",
        "char_430_fartth",
    ]);
    eprintln!("SHALLOW {shallow:?} {shallow_eff:.1}");
    let mut want = vec![VERMEIL.to_string(), SCENE.to_string(), PALLAS.to_string()];
    want.sort();
    assert_eq!(
        shallow, want,
        "with nobody cut the comp is found at {shallow_eff:.1}"
    );
}
