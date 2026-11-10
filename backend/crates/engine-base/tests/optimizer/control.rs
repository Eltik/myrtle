use super::*;

#[test]
fn control_center_is_surfaced_in_the_assignment() {
    let gd = load_game_data();

    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            UserRoom {
                slot_id: "slot_cc".into(),
                room_type: "CONTROL".into(),
                level: 3,
                ..Default::default()
            },
            UserRoom {
                slot_id: "slot_mfg".into(),
                room_type: "MANUFACTURE".into(),
                level: 3,
                ..Default::default()
            },
        ],
    };

    // Kal'tsit's "Highest Authority" is a Control Center buff (+factory output).
    let roster = vec![
        profile(gd, "char_003_kalts"),
        profile(gd, EXUSIAI),
        profile(gd, "char_502_nblade"),
    ];

    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);

    let cc = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "CONTROL")
        .expect("Control Center should now appear in the assignment output");
    assert!(
        cc.operators.iter().any(|o| o == "char_003_kalts"),
        "Kal'tsit should be stationed in the Control Center. Got: {:?}",
        cc.operators
    );
}

#[test]
fn faction_gated_cc_buffs_are_conditional_not_flat_global() {
    // Control Center trading buffs split three ways:
    //   - Amiya "all Trading Posts +7%"          -> unconditional GlobalEffect
    //   - Umiri "all Siracusa Operators +5%"      -> per-operator ConditionalGlobalEffect
    //   - SilverAsh "posts w/ 3 Kjerag ops +10%"  -> count-gated ConditionalGlobalEffect
    // Only the first may be credited flat to every post.

    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _drains) = build_registry(&gd.building().buffs, &name_to_char);

    match registry.get("control_tra_spd[000]") {
        Some(BuffResolutionStrategy::GlobalEffect {
            target_room,
            bonus_pct,
        }) => {
            assert_eq!(target_room, "TRADING");
            assert!(
                (*bonus_pct - 7.0).abs() < 0.01,
                "Amiya +7%, got {bonus_pct}"
            );
        }
        _ => panic!("Amiya's CC buff should stay an unconditional GlobalEffect"),
    }
    match registry.get("control_tra_limit&spd2[000]") {
        Some(BuffResolutionStrategy::ConditionalGlobalEffect {
            target_room,
            faction_token,
            per_operator,
            bonus_pct,
            ..
        }) => {
            assert_eq!(target_room, "TRADING");
            assert_eq!(faction_token, "siracusa");
            assert!(*per_operator, "Umiri's bonus is per matching operator");
            assert!((*bonus_pct - 5.0).abs() < 0.01, "+5%, got {bonus_pct}");
        }
        _ => panic!("Umiri's Siracusa CC buff should be a per-operator ConditionalGlobalEffect"),
    }
    match registry.get("control_tra_limit&spd3[000]") {
        Some(BuffResolutionStrategy::ConditionalGlobalEffect {
            faction_token,
            required_count,
            per_operator,
            bonus_pct,
            ..
        }) => {
            assert_eq!(faction_token, "kjerag");
            assert_eq!(*required_count, 3, "post needs 3 Kjerag operators");
            assert!(
                !*per_operator,
                "SilverAsh's bonus is whole-post, count-gated"
            );
            assert!((*bonus_pct - 10.0).abs() < 0.01, "+10%, got {bonus_pct}");
        }
        _ => panic!("SilverAsh's Kjerag CC buff should be a count-gated ConditionalGlobalEffect"),
    }
}

#[test]
fn control_center_same_type_buffs_do_not_stack() {
    // Amiya and Swire both grant the clause-bearing "+X% to all Trading Posts" CC
    // buff ("only the most effective one will take effect... same skill effect"),
    // so two of them must NOT stack; only the strongest applies.
    use myrtle_engine_base::assignment::compute_current_assignment;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    // The global buff a CC team grants, read off an empty trading post.
    let trading_global = |cc: &[&str]| -> f64 {
        let building = UserBuilding {
            drone_capacity: None,
            rooms: vec![
                UserRoom {
                    slot_id: "cc".into(),
                    room_type: "CONTROL".into(),
                    level: 5,
                    current_operators: cc.iter().map(|s| (*s).to_string()).collect(),
                    ..Default::default()
                },
                UserRoom {
                    slot_id: "tp".into(),
                    room_type: "TRADING".into(),
                    level: 3,
                    ..Default::default()
                },
            ],
        };
        let roster: Vec<_> = cc.iter().map(|s| profile(gd, s)).collect();
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.room_type == "TRADING")
            .map_or(0.0, |r| r.total_efficiency)
    };
    const AMIYA: &str = "char_002_amiya";
    const SWIRE: &str = "char_308_swire";
    let amiya = trading_global(&[AMIYA]);
    let swire = trading_global(&[SWIRE]);
    let both = trading_global(&[AMIYA, SWIRE]);
    assert!(amiya > 0.0 && swire > 0.0, "each grants a trading CC buff");
    assert!(
        (both - amiya.max(swire)).abs() < 0.01,
        "same-type CC buffs must not stack: both={both} should equal max({amiya}, {swire})"
    );
    assert!(both < amiya + swire - 0.01, "...and stay below the sum");
}

#[test]
fn viviana_knight_buff_reaches_pinus_knights_like_wild_mane() {
    // Viviana's CC buff "+7% to all Knight Operators in Factories" must reach Wild
    // Mane, who is Pinus Sylvestris (the Knightclub), so the optimizer co-schedules
    // them. A non-Knight factory operator gets nothing from her.
    use myrtle_engine_base::assignment::compute_current_assignment;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let factory_eff = |cc: &[&str], factory: &[&str]| -> f64 {
        let building = UserBuilding {
            drone_capacity: None,
            rooms: vec![
                UserRoom {
                    slot_id: "cc".into(),
                    room_type: "CONTROL".into(),
                    level: 5,
                    current_operators: cc.iter().map(|s| (*s).to_string()).collect(),
                    ..Default::default()
                },
                UserRoom {
                    slot_id: "mf".into(),
                    room_type: "MANUFACTURE".into(),
                    level: 3,
                    current_operators: factory.iter().map(|s| (*s).to_string()).collect(),
                    ..Default::default()
                },
            ],
        };
        let roster: Vec<_> = cc.iter().chain(factory).map(|s| profile(gd, s)).collect();
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.room_type == "MANUFACTURE")
            .map_or(0.0, |r| r.total_efficiency)
    };
    const VIVIANA: &str = "char_4098_vvana";
    const WILD_MANE: &str = "char_496_wildmn";
    const VERMEIL: &str = "char_190_clour"; // Siracusa, not a Knight

    let knight_with = factory_eff(&[VIVIANA], &[WILD_MANE]);
    let knight_without = factory_eff(&[], &[WILD_MANE]);
    assert!(
        knight_with > knight_without + 6.0,
        "Viviana's +7% Knight buff should reach Wild Mane (Pinus): with={knight_with} without={knight_without}"
    );

    let nonknight_with = factory_eff(&[VIVIANA], &[VERMEIL]);
    let nonknight_without = factory_eff(&[], &[VERMEIL]);
    assert!(
        (nonknight_with - nonknight_without).abs() < 1.0,
        "a non-Knight must not get Viviana's bonus: with={nonknight_with} without={nonknight_without}"
    );
}

#[test]
fn faction_gated_cc_bonus_does_not_inflate_a_nonmatching_post() {
    // Regression: a Proviso/Lemuen/Exusiai post under a Control Center staffed by
    // Amiya + SilverAsh + Umiri must read ~87% (Lemuen 45 + Exusiai 35 = 80 speed,
    // plus ONLY Amiya's unconditional +7% global). SilverAsh's "3 Kjerag" +10% and
    // Umiri's "Siracusa" +5% must NOT apply (this team is Kazimierz/Laterano/
    // Lungmen), so it must NOT read the old, inflated 102%.
    use myrtle_engine_base::assignment::compute_current_assignment;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let roster: Vec<_> = [
        "char_002_amiya",
        "char_1045_svash2",
        "char_4186_tmoris",
        "char_4032_provs",
        "char_4193_lemuen",
        "char_103_angel",
    ]
    .iter()
    .map(|s| profile(gd, s))
    .collect();

    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            UserRoom {
                slot_id: "cc".into(),
                room_type: "CONTROL".into(),
                level: 5,
                current_operators: vec![
                    "char_002_amiya".into(),
                    "char_1045_svash2".into(),
                    "char_4186_tmoris".into(),
                ],
                ..Default::default()
            },
            UserRoom {
                slot_id: "tp".into(),
                room_type: "TRADING".into(),
                level: 3,
                current_operators: vec![
                    "char_4032_provs".into(),
                    "char_4193_lemuen".into(),
                    "char_103_angel".into(),
                ],
                ..Default::default()
            },
        ],
    };

    let asn =
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post");

    assert!(
        (tp.total_efficiency - 87.0).abs() < 2.0,
        "non-matching post should read ~87% (80 speed + 7% unconditional CC), got +{:.1}%",
        tp.total_efficiency
    );
    assert!(
        tp.total_efficiency < 95.0,
        "faction-gated CC buffs (SilverAsh +10, Umiri +5) must not inflate this post, got +{:.1}%",
        tp.total_efficiency
    );
}

#[test]
fn conditional_cc_buff_is_not_credited_when_its_gate_cannot_be_met() {
    // SilverAsh the Reignfrost's Control Center buff only helps Trading Posts that
    // hold 3 Kjerag operators. With too few Kjerag traders in the roster the gate
    // can never trigger, so he must NOT take a CC seat; a legitimate global op
    // (Amiya: "all Trading Posts +X%") gets it instead.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)],
    };
    let roster = vec![
        profile(gd, "char_1045_svash2"), // SilverAsh the Reignfrost (conditional CC)
        profile(gd, "char_002_amiya"),   // Amiya (unconditional CC global)
        profile(gd, "char_103_angel"),   // Exusiai, trader
        profile(gd, "char_102_texas"),   // Texas, trader
    ];

    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let cc = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "CONTROL")
        .expect("a control center");
    assert!(
        !cc.operators.iter().any(|o| o == "char_1045_svash2"),
        "SilverAsh's unmeetable Kjerag gate must keep him out of the CC. Got: {:?}",
        cc.operators
    );
    assert!(
        cc.operators.iter().any(|o| o == "char_002_amiya"),
        "the legitimate global CC operator should take the seat. Got: {:?}",
        cc.operators
    );
}

#[test]
fn gravel_counts_as_a_knight_for_vivianas_glimmer() {
    // "All Knight Operators assigned to Factories gain productivity +7%"
    // (Viviana). Who a Knight is comes from the game's term glossary
    // (`cc.tag.knight`: Nearl, Blemishine, Whislash, ..., Gravel, Viviana),
    // not from a faction id: Gravel is Kazimierz with no group, and the old
    // Pinus-only alias missed her (user-verified in game 2026-10-03: a
    // Gravel gold factory reads +1.11, our 101 lacked the 7).
    use myrtle_engine_base::assignment::compute_current_assignment;
    const GRAVEL: &str = "char_237_gravel";
    const VIVIANA: &str = "char_4098_vvana";
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
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "mf")
            .map_or(0.0, |r| r.total_efficiency)
    };
    let with = factory_with_cc(&[VIVIANA]);
    let without = factory_with_cc(&[]);
    assert!(
        (with - without - 7.0).abs() < 1e-6,
        "Viviana's +7 reaches Gravel: {without:.1} -> {with:.1}"
    );
}

#[test]
fn dead_conditional_cc_operator_is_dropped_after_assignment() {
    // The roster HAS three Kjerag traders, so SilverAsh's gate passes the roster
    // feasibility check and he is initially seated in the CC. But stronger non-Kjerag
    // traders win the single post's seats, so no post ever holds 3 Kjerag and his +10%
    // never fires. The post-assignment check must drop him and reseat a real op.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)],
    };
    let roster = vec![
        profile(gd, "char_1045_svash2"), // SilverAsh the Reignfrost (CC, Kjerag gate)
        profile(gd, "char_002_amiya"),   // Amiya (unconditional CC global)
        profile(gd, "char_198_blackd"),  // Courier (Kjerag, weak trader)
        profile(gd, "char_199_yak"),     // Matterhorn (Kjerag, weak trader)
        profile(gd, "char_173_slchan"),  // Cliffheart (Kjerag, weak trader)
        profile(gd, "char_103_angel"),   // Exusiai (strong trader)
        profile(gd, "char_502_nblade"),  // Yato (strong trader)
        profile(gd, "char_185_frncat"),  // Mousse (strong trader)
    ];
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let cc = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "CONTROL")
        .expect("a control center");
    assert!(
        !cc.operators.iter().any(|o| o == "char_1045_svash2"),
        "SilverAsh's never-firing Kjerag gate must drop him from the CC. Got: {:?}",
        cc.operators
    );
    assert!(
        cc.operators.iter().any(|o| o == "char_002_amiya"),
        "the contributing global CC operator should hold the seat. Got: {:?}",
        cc.operators
    );
}

#[test]
fn passion_combo_staffs_the_control_center_and_lifts_factories() {
    use myrtle_engine_base::scoring::pools::candidate_bundles;
    let gd = load_game_data();
    let building = bd_base_243();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    // Combo + production fillers, but no other Control-Center operators (clean commit).
    let combo: std::collections::HashSet<&str> = [SAKIKO, MORTIS, AMORIS, DOLRIS, TMORIS]
        .into_iter()
        .collect();
    let roster: Vec<_> = full_roster(gd)
        .into_iter()
        .filter(|op| combo.contains(op.char_id.as_str()) || !is_control_op(gd, op))
        .collect();

    // The NATIVE bundle machinery offers the quintet as one economy: apply it
    // exactly like the service's oracle trial (overrides + globals + pins).
    let bundles = candidate_bundles(&roster, &building, gd.building(), &registry);
    let bundle = bundles
        .iter()
        .find(|b| b.pins.iter().any(|(id, _)| id == SAKIKO))
        .expect("the Passion economy is offered as a bundle");
    let mut reg = registry.clone();
    for (b, pct) in &bundle.overrides {
        reg.insert(b.clone(), BuffResolutionStrategy::PoolPayoff { pct: *pct });
    }
    for (b, room, pct) in &bundle.globals {
        reg.insert(
            b.clone(),
            BuffResolutionStrategy::GlobalEffect {
                target_room: room.clone(),
                bonus_pct: *pct,
            },
        );
    }

    let with_combo = compute_optimal_assignment_with_pins(
        &roster,
        &building,
        gd.building(),
        &reg,
        &drains,
        &bundle.pins,
    );
    let cc = with_combo
        .rooms
        .iter()
        .find(|r| r.room_type == "CONTROL")
        .expect("a Control Center");
    for id in [SAKIKO, MORTIS, AMORIS, DOLRIS, TMORIS] {
        assert!(
            cc.operators.iter().any(|o| o == id),
            "{id} should staff the Control Center, got {:?}",
            cc.operators
        );
    }

    // The global Precious-Metal bonus reaches the factories: their efficiency is strictly higher
    // than the same plan computed without the economy folded in.
    let baseline = compute_optimal_assignment_with_pins(
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
        &[],
    );
    let factory_eff = |a: &myrtle_engine_base::types::BaseAssignment| -> f64 {
        a.rooms
            .iter()
            .filter(|r| r.room_type == "MANUFACTURE")
            .map(|r| r.total_efficiency)
            .sum()
    };
    assert!(
        factory_eff(&with_combo) > factory_eff(&baseline) + 1.0,
        "the committed combo should lift total factory efficiency ({} vs {})",
        factory_eff(&with_combo),
        factory_eff(&baseline)
    );
}

/// Justice Knight's "'Beep beep, activate!'" from a Power Plant seat boosts
/// the Factory Wild Mane works in by +5%: her Pinus Sylvestris β 25 reads 30
/// while he powers a plant, 25 while he rests.
#[test]
fn justice_knight_in_a_plant_boosts_wild_manes_factory() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const WILD_MANE: &str = "char_496_wildmn";
    const JUSTICE_KNIGHT: &str = "char_4000_jnight";
    let roster: Vec<_> = [WILD_MANE, JUSTICE_KNIGHT]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let eff = |knight_room: &str| {
        let mut rooms = vec![
            room("mf", "MANUFACTURE", 3),
            room("pp", "POWER", 3),
            room("d0", "DORMITORY", 5),
        ];
        rooms[0].current_operators = vec![WILD_MANE.into()];
        rooms[0].current_formula = Some("F_EXP".into());
        rooms
            .iter_mut()
            .find(|r| r.slot_id == knight_room)
            .unwrap()
            .current_operators
            .push(JUSTICE_KNIGHT.into());
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
        (eff("pp") - 30.0).abs() < 1e-9,
        "Justice Knight in the plant: {}",
        eff("pp")
    );
    assert!(
        (eff("d0") - 25.0).abs() < 1e-9,
        "Justice Knight resting: {}",
        eff("d0")
    );
}

/// Morgan, Siege and Proviso in a level-2 post with Delphine in the Control
/// Center (user's in-game check 2026-09-14: +102% beside a +3% innate line):
/// Morgan counts herself and Siege (2 x 20) plus 35 beside Siege, Delphine
/// adds 10 per Glasgow operator in the post (2 x 10) = 95; the game's
/// remaining 7 is a flat trading global from another Control Center seat,
/// and the +3% is the innate per-operator bonus we keep in the yield only.
#[test]
fn morgan_siege_proviso_with_delphine_read_ninety_five() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const MORGAN: &str = "char_154_morgan";
    const SIEGE: &str = "char_112_siege";
    const PROVISO: &str = "char_4032_provs";
    const DELPHINE: &str = "char_4110_delphn";
    let roster: Vec<_> = [MORGAN, SIEGE, PROVISO, DELPHINE]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let mut rooms = vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 2)];
    rooms[0].current_operators = vec![DELPHINE.into()];
    rooms[1].current_operators = vec![MORGAN.into(), SIEGE.into(), PROVISO.into()];
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let asn =
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
    let tp = asn
        .rooms
        .iter()
        .find(|r| r.slot_id == "tp")
        .expect("the post");
    assert!(
        (tp.total_efficiency - 95.0).abs() < 1e-9,
        "Morgan 75 + Delphine 20: {}",
        tp.total_efficiency
    );
    assert!(
        tp.order_value > 80.0,
        "Proviso's value in a level-2 post: {}",
        tp.order_value
    );
}

#[test]
fn viviana_synergy_flips_the_cc_to_a_block_aligned_with_her_knights() {
    // Viviana's CC buff ("all Knight Operators in Factories +7%") links her to the factory
    // team fielding her Knights: the CC flips to a 24h block (Squad 1 on shifts 1+2) and the
    // Knight team is phased onto the same shifts-1+2 block, so the buff is live while they work.
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    const VIVIANA: &str = "char_4098_vvana";
    const WILD_MANE: &str = "char_496_wildmn"; // Knight with a factory skill
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let mut rooms = vec![room("cc", "CONTROL", 5), room("d0", "DORMITORY", 5)];
    rooms.extend((0..2).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let roster: Vec<_> = [
        VIVIANA,
        WILD_MANE,
        "char_36_forget", // generic factory ops fill the other teams
        "char_123_fang",
        "char_133_mm",
        "char_502_nblade",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);

    let cc = |k: usize| -> Vec<String> {
        rot.shifts[k]
            .rooms
            .iter()
            .find(|r| r.room_type == "CONTROL")
            .map(|r| r.recommended.clone())
            .unwrap_or_default()
    };
    assert!(
        cc(0).contains(&VIVIANA.to_string()),
        "Viviana anchors CC Squad 1: {:?}",
        cc(0)
    );
    assert_eq!(
        cc(0),
        cc(1),
        "the CC runs a 24h BLOCK (Squad 1 on shifts 1+2) when a synergy links it"
    );
    // Her Knight works the same shifts-1+2 window (the ordinal-0 block of its group).
    let knight_shifts: Vec<usize> = rot
        .shifts
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            s.rooms.iter().any(|r| {
                // A dorm cell is REST, not work: the Knight rests shift 3 in a dormitory.
                r.active
                    && r.room_type != "DORMITORY"
                    && r.recommended.iter().any(|o| o == WILD_MANE)
            })
        })
        .map(|(k, _)| k)
        .collect();
    assert_eq!(
        knight_shifts,
        vec![0, 1],
        "the Knight team is phased onto the shifts-1+2 block, aligned with Viviana"
    );
}

/// Delphine's "for each Glasgow Gang Operator assigned to the same Trading
/// Post, +10%" is a CONDITIONAL per-operator global, not a flat tag bonus:
/// the singular "each ... Operator" phrasing must route to the same machinery
/// as `SilverAsh`'s plural form. She earns a Control-Center seat only when the
/// plan actually fields Glasgow traders; otherwise the dead-weight reselection
/// evicts her instead of crediting a +10% nobody receives.
#[test]
fn delphine_needs_glasgow_traders_to_earn_her_cc_seat() {
    use myrtle_engine_base::assignment::compute_optimal_assignment_with_pins;
    use myrtle_engine_base::registry::BuffResolutionStrategy;
    const BUFF: &str = "control_tra_limit&spd[010]";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    match registry.get(BUFF) {
        Some(BuffResolutionStrategy::ConditionalGlobalEffect {
            target_room,
            faction_token,
            required_count,
            per_operator,
            bonus_pct,
            ..
        }) => {
            assert_eq!(target_room, "TRADING");
            assert_eq!(faction_token, "glasgow");
            assert_eq!(*required_count, 1);
            assert!(
                *per_operator,
                "each Glasgow member earns the +10% separately"
            );
            assert!(
                (*bonus_pct - 10.0).abs() < 1e-9,
                "the conditional carries a +10% bonus, got {bonus_pct}"
            );
        }
        other => panic!("expected ConditionalGlobalEffect, got {other:?}"),
    }

    let delphine = gd
        .building()
        .chars
        .iter()
        .find(|(_, c)| {
            c.buff_char
                .iter()
                .any(|bc| bc.buff_data.iter().any(|bd| bd.buff_id == BUFF))
        })
        .map(|(id, _)| id.clone())
        .expect("some operator owns Delphine's skill");

    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("cc", "CONTROL", 5),
            room("tp", "TRADING", 3),
            room("d0", "DORMITORY", 2),
        ],
    };
    let plan = |ids: &[&str], with_delphine: bool| {
        let mut roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        if with_delphine {
            roster.push(profile(gd, delphine.as_str()));
        }
        compute_optimal_assignment_with_pins(
            &roster,
            &building,
            gd.building(),
            &registry,
            &drains,
            &[],
        )
    };
    let cc_eff = |p: &myrtle_engine_base::types::BaseAssignment| {
        p.rooms
            .iter()
            .find(|r| r.room_type == "CONTROL")
            .map_or(0.0, |r| r.total_efficiency)
    };
    let tp_eff = |p: &myrtle_engine_base::types::BaseAssignment| {
        p.rooms
            .iter()
            .filter(|r| r.room_type == "TRADING")
            .map(|r| r.total_efficiency)
            .sum::<f64>()
    };

    // Only non-Glasgow traders available: her conditional can never fire.
    // A spare CC seat is fine (leftovers get parked), but she must add ZERO
    // credited value anywhere; previously the flat TagBased misparse sold
    // her +10% at half credit with nobody to receive it.
    let neutrals = ["char_103_angel", "char_214_kafka", "char_4032_provs"];
    let without_her = plan(&neutrals, false);
    let with_her = plan(&neutrals, true);
    assert!(
        cc_eff(&with_her).abs() < 1e-9,
        "no Glasgow fielded: her Control-Center row must carry no credit"
    );
    assert!(
        (tp_eff(&with_her) - tp_eff(&without_her)).abs() < 1e-9,
        "no Glasgow fielded: the posts must not gain from her either ({} vs {})",
        tp_eff(&with_her),
        tp_eff(&without_her)
    );

    // With Texas and Lappland fielded, her conditional is REAL: each Glasgow
    // member in the post earns +10%, credited to that post's efficiency.
    let glasgow = ["char_154_morgan", "char_157_dagda", "char_103_angel"];
    let g_without = plan(&glasgow, false);
    let g_with = plan(&glasgow, true);
    let fielded = g_with
        .rooms
        .iter()
        .filter(|r| r.room_type == "TRADING")
        .flat_map(|r| r.operators.iter())
        .filter(|o| *o == "char_154_morgan" || *o == "char_157_dagda")
        .count();
    if fielded > 0 {
        assert!(
            tp_eff(&g_with) >= tp_eff(&g_without) + 10.0 - 1e-9,
            "with {fielded} Glasgow trader(s) fielded, the post must gain her credit \
             ({} vs {})",
            tp_eff(&g_with),
            tp_eff(&g_without)
        );
    }
}

/// Non-production Control-Center skills (clue / training / HR) parse into
/// `ControlNonProduction`, priced in their OWN units with zero LMD weight (the
/// objective stays production-pure, matching both reference implementations),
/// and break ties for SPARE CC seats with the reference priority: clue > HR >
/// training. "Only the strongest effect of this type" makes ONE family per
/// boosted metric, across different buff-id prefixes.
#[test]
// Exact float equality is intentional: the expected value is threaded straight
// from the same constant the resolver used, not an independent computation.
#[allow(clippy::float_cmp)]
fn cc_non_production_skills_parse_and_break_spare_seat_ties() {
    use myrtle_engine_base::assignment::fill_remaining_slots;
    use myrtle_engine_base::registry::{BuffResolutionStrategy, build_registry};
    use myrtle_engine_base::scoring::clause::build_clauses;
    use myrtle_engine_base::types::OperatorBaseProfile;
    use std::collections::HashSet;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);

    // Golden parse: room target, value, and gating.
    for (id, want_room, want_value, gated) in [
        ("control_upMeetingSpeed[000]", "MEETING", 25.0, false),
        ("control_meeting_spd&bd[000]", "MEETING", 5.0, false),
        ("control_mp&meet_spd[000]", "MEETING", 5.0, true), // with Sakiko
        ("control_train_spd[010]", "TRAINING", 5.0, false),
        ("control_hire_spd&bd[000]", "HIRE", 10.0, false),
    ] {
        match registry.get(id) {
            Some(BuffResolutionStrategy::ControlNonProduction {
                target_room,
                value,
                same_room_gate,
            }) => {
                assert_eq!(target_room, want_room, "{id}");
                assert_eq!(*value, want_value, "{id}");
                assert_eq!(same_room_gate.is_some(), gated, "{id}: gating");
                if gated {
                    assert!(
                        !same_room_gate.as_ref().unwrap().is_empty(),
                        "{id}: the named companion must resolve"
                    );
                }
            }
            other => panic!("{id}: expected ControlNonProduction, got {other:?}"),
        }
    }

    // Non-stacking family spans buff-id prefixes: +25% and +5% clue speed are
    // the same "type", so they must share one family.
    let clauses = build_clauses(&gd.building().buffs, &name_to_char);
    let family = |id: &str| {
        clauses[id]
            .first()
            .and_then(|c| c.non_stacking_family.clone())
    };
    assert_eq!(
        family("control_upMeetingSpeed[000]"),
        family("control_meeting_spd&bd[000]"),
        "clue-speed CC skills share one non-stacking family"
    );
    assert!(family("control_upMeetingSpeed[000]").is_some());

    // Spare-seat tie-break: with equal opportunity cost, the clue holder gets
    // the free CONTROL seat over a skill-less benchwarmer.
    let bare = |id: &str, buffs: Vec<String>| OperatorBaseProfile {
        char_id: id.to_string(),
        available_buffs: buffs,
        faction_tags: vec![],
        match_tags: vec![],
        rarity: 4,
        elite: 1,
    };
    let ops = [
        bare("plain_op", vec![]),
        bare("clue_op", vec!["control_upMeetingSpeed[000]".to_string()]),
    ];
    let mut seats: Vec<String> = Vec::new();
    let mut assigned: HashSet<String> = HashSet::new();
    fill_remaining_slots(
        &mut seats,
        1,
        "CONTROL",
        &ops,
        gd.building(),
        &registry,
        &mut assigned,
    );
    assert_eq!(seats, ["clue_op"], "the clue holder wins the spare seat");

    // Display aggregation: "only the strongest effect of this type" means the
    // crew's clue total is the MAX (+15), never the sum (+20); the Sakiko
    // same-room gate only counts while she shares the Control Center.
    use myrtle_engine_base::assignment::cc_non_production_effects;
    let crew_ops = [
        bare("op_a", vec!["control_upMeetingSpeed[100]".to_string()]), // clue +15
        bare("op_b", vec!["control_meeting_spd&bd[000]".to_string()]), // clue +5
        bare("op_c", vec!["control_mp&meet_spd[000]".to_string()]),    // clue +5, needs Sakiko
    ];
    let ids = |v: &[&str]| v.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
    let effects = |crew: &[String]| cc_non_production_effects(crew, &crew_ops, &registry, &[]);
    assert_eq!(
        effects(&ids(&["op_a", "op_b"])),
        [("MEETING".to_string(), 15.0)],
        "non-stacking: max, not sum"
    );
    assert_eq!(effects(&ids(&["op_b"])), [("MEETING".to_string(), 5.0)]);
    assert_eq!(effects(&ids(&["op_c"])), [], "gated with no Sakiko in crew");
    let sakiko = name_to_char["sakiko togawa"].clone();
    assert_eq!(
        effects(&[vec!["op_c".to_string(), sakiko]].concat()),
        [("MEETING".to_string(), 5.0)],
        "gate fires once Sakiko shares the CC"
    );
}

/// A gated Control-Center grant says on its own line how far it reaches:
/// Viviana's "+7% to Knight operators in Factories" reads "1 fielded" when a
/// Knight works a factory and "no team fields one" otherwise, so a seat
/// held for one Knight is not read as a seat held for nothing (31010962).
#[test]
fn a_gated_cc_grant_line_says_how_many_it_reaches() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    const VIVIANA: &str = "char_4098_vvana";
    const FARTOOTH: &str = "char_430_fartth";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let roster: Vec<_> = [VIVIANA, FARTOOTH, "char_123_fang"]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let note_for = |factory_crew: Vec<&str>| -> String {
        let mut cc = room("cc", "CONTROL", 5);
        cc.current_operators = vec![VIVIANA.into()];
        let mut mf = room("mf", "MANUFACTURE", 3);
        mf.current_formula = Some("F_GOLD".into());
        mf.current_operators = factory_crew.iter().map(|s| (*s).to_string()).collect();
        let building = UserBuilding {
            drone_capacity: None,
            rooms: vec![cc, mf],
        };
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        let cc = asn
            .rooms
            .iter()
            .find(|r| r.room_type == "CONTROL")
            .expect("the CC is scored");
        cc.ledger
            .iter()
            .find(|l| {
                l.operator_id == VIVIANA && l.note.as_deref().is_some_and(|n| n.contains("Knight"))
            })
            .and_then(|l| l.note.clone())
            .unwrap_or_else(|| panic!("Viviana's gated line carries a note: {:?}", cc.ledger))
    };
    let with = note_for(vec![FARTOOTH, "char_123_fang"]);
    assert!(
        with.contains("1 fielded"),
        "a Knight in the factory: {with}"
    );
    let without = note_for(vec!["char_123_fang"]);
    assert!(
        without.contains("no team fields one"),
        "no Knight anywhere: {without}"
    );
}

/// Viviana's "+7% to Knight operators in Factories" is worth a Control
/// Center seat only once the Knights sit in a factory, and the Knights are
/// worth that factory only with her seated: three plain +25 Knights read 96
/// with her against 75, and a roster whose other factory hands are weaker
/// should field the trio with her in the Control Center. The greedy seat
/// picker never took her (a discounted guess against flat bonuses); a
/// bundle trial pins her and keeps the plan when the base gains.
#[test]
fn a_knight_trio_is_assembled_with_viviana_in_the_control_center() {
    const VIVIANA: &str = "char_4098_vvana";
    const KNIGHTS: [&str; 3] = ["char_430_fartth", "char_431_ashlok", "char_496_wildmn"];
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let mut roster: Vec<_> = [
        VIVIANA,
        KNIGHTS[0],
        KNIGHTS[1],
        KNIGHTS[2],
        "char_003_kalts",
        "char_002_amiya",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .collect();
    // Weaker generic hands: +15% (E0 tier) operators, and two order traders.
    for id in [
        "char_123_fang",
        "char_133_mm",
        "char_502_nblade",
        "char_124_kroos",
        "char_102_texas",
        "char_140_whitew",
    ] {
        if gd.building().chars.contains_key(id) {
            roster.push(profile_at(gd, id, 0));
        }
    }
    let mut rooms = vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)];
    let mut mf = room("mf0", "MANUFACTURE", 3);
    mf.current_formula = Some("F_GOLD".into());
    rooms.push(mf);
    rooms.extend((0..2).map(|i| room(&format!("d{i}"), "DORMITORY", 3)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let economy = myrtle_engine_base::scoring::pools::search_economy(
        &roster,
        &building,
        gd.building(),
        &registry,
    );
    let accepted = myrtle_engine_base::scoring::pools::optimal_with_bundles(
        &roster,
        &building,
        gd.building(),
        &registry,
        &economy.registry,
        &drains,
        &economy.pins,
    );
    let cc = accepted
        .optimal
        .rooms
        .iter()
        .find(|r| r.room_type == "CONTROL")
        .expect("a CC");
    let factory = accepted
        .optimal
        .rooms
        .iter()
        .find(|r| r.room_type == "MANUFACTURE")
        .expect("a factory");
    assert!(
        cc.operators.iter().any(|o| o == VIVIANA),
        "Viviana sits in the Control Center, got {:?}",
        cc.operators
    );
    let knights = factory
        .operators
        .iter()
        .filter(|o| KNIGHTS.contains(&o.as_str()))
        .count();
    assert_eq!(
        knights, 3,
        "the Knights share the factory, got {:?} at {:.1}",
        factory.operators, factory.total_efficiency
    );
    assert!(
        factory.total_efficiency >= 96.0,
        "three Knights at +32 each: {:.1}",
        factory.total_efficiency
    );
}

/// A spare Control-Center seat goes to a base-wide morale aura first, then
/// HR or training speed, and to clue speed last whatever its size: Lee's
/// +25% clue collection is worth less than Umiri's +10% HR contact, and a
/// morale aura ranks ahead of both (user feedback 2026-09-19).
#[test]
fn a_spare_cc_seat_ranks_morale_and_hr_ahead_of_clue_speed() {
    use myrtle_engine_base::assignment::fill_remaining_slots;
    const LEE: &str = "char_322_lmlee";
    const UMIRI: &str = "char_4210_yumiri";
    const CHONGYUE: &str = "char_2024_chyue";
    let gd = load_game_data();
    let (registry, _) = build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let umiri = gd
        .operators()
        .iter()
        .find(|(_, o)| o.name == "Umiri Yahata")
        .map_or_else(|| UMIRI.to_string(), |(id, _)| id.clone());
    let roster: Vec<_> = [LEE, umiri.as_str(), CHONGYUE]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let mut slots = Vec::new();
    let mut assigned = std::collections::HashSet::new();
    fill_remaining_slots(
        &mut slots,
        2,
        "CONTROL",
        &roster,
        gd.building(),
        &registry,
        &mut assigned,
    );
    assert_eq!(
        slots,
        vec![CHONGYUE.to_string(), umiri],
        "morale aura, then HR speed; Lee's clue speed waits"
    );
}
