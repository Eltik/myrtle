use super::*;

#[test]
fn low_morale_drain_teams_sustain_better_longevity_axis() {
    // The longevity axis: a low-morale-drain factory team holds closer to peak
    // under rotation than a higher-drain team, because its operators rest less and
    // need backups to cover them less often.
    use myrtle_engine_base::assignment::{morale_recovery, sustained_efficiency_of};

    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    // Same room peak, different drain: Vermeil (-0.25/hr) is a far lower-drain
    // factory worker than a neutral/high-drain operator.
    let sustain_fraction = |op: &str| -> f64 {
        let roster = vec![profile(gd, op)];
        let building = UserBuilding {
            drone_capacity: None,
            rooms: vec![UserRoom {
                slot_id: "mf".into(),
                room_type: "MANUFACTURE".into(),
                level: 3,
                current_operators: vec![op.to_string()],
                ..Default::default()
            }],
        };
        let cur = myrtle_engine_base::assignment::compute_current_assignment(
            &roster,
            &building,
            gd.building(),
            &registry,
            &drains,
            None,
        );
        let peak: f64 = cur.rooms.iter().map(|r| r.total_efficiency).sum();
        if peak <= 0.0 {
            return 1.0;
        }
        sustained_efficiency_of(
            &cur,
            &roster,
            &drains,
            morale_recovery(&building),
            &registry,
            gd.building(),
            &std::collections::HashSet::new(),
        ) / peak
    };
    let vermeil = sustain_fraction("char_190_clour"); // -0.25/hr drain
    let neutral = sustain_fraction("char_496_wildmn"); // Wild Mane, neutral drain
    assert!(
        vermeil > neutral,
        "low-drain Vermeil should hold a higher fraction of peak ({vermeil:.3}) than a neutral-drain op ({neutral:.3})"
    );
}

#[test]
fn low_level_dorms_reduce_sustained_output() {
    // Morale recovery scales with dorm level, so the same production setup sustains
    // further below peak when its dormitories are low-level (a 252 reality) than when
    // they are fully developed. The peak (main) staffing is identical either way.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let layout = |dorm_lv: i32| UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("tp", "TRADING", 3),
            room("mf", "MANUFACTURE", 3),
            room("d0", "DORMITORY", dorm_lv),
            room("d1", "DORMITORY", dorm_lv),
        ],
    };
    let roster = full_roster(gd);
    let hi = compute_sustained_assignment(&roster, &layout(5), gd.building(), &registry, &drains);
    let lo = compute_sustained_assignment(&roster, &layout(1), gd.building(), &registry, &drains);
    assert!(
        lo.sustained_efficiency < hi.sustained_efficiency,
        "low-level dorms should sustain below high-level dorms (lo {:.1} vs hi {:.1})",
        lo.sustained_efficiency,
        hi.sustained_efficiency
    );
}

#[test]
fn bellones_rider_fires_with_vigil_resting_in_a_dormitory() {
    // "+30%; when Vigil is in the Base (excluding Assistants and Activity
    // Room users), +10%" (Bellone; Underflow's Envoy To Land is the same
    // shape on Ulpianus). "In the Base" counts a dormitory, unlike the "any
    // Work Area" gates: the rider used to be dropped outright, and a resting
    // partner must count (user-verified in game 2026-10-03).
    use myrtle_engine_base::assignment::compute_current_assignment;
    const BELLONE: &str = "char_4037_demetr";
    const VIGIL: &str = "char_427_vigil";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let post_with = |vigil_in: Option<&str>| {
        let mut post = room("tp", "TRADING", 3);
        post.current_operators = vec![BELLONE.into()];
        let mut dorm = room("d0", "DORMITORY", 5);
        let mut mf = room("mf", "MANUFACTURE", 3);
        mf.current_formula = Some("F_EXP".into());
        match vigil_in {
            Some("d0") => dorm.current_operators = vec![VIGIL.into()],
            Some("mf") => mf.current_operators = vec![VIGIL.into()],
            _ => {}
        }
        let building = UserBuilding {
            drone_capacity: None,
            rooms: vec![post, mf, dorm],
        };
        let roster = vec![profile(gd, BELLONE), profile(gd, VIGIL)];
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "tp")
            .map_or(0.0, |r| r.total_efficiency)
    };
    let without = post_with(None);
    let resting = post_with(Some("d0"));
    let working = post_with(Some("mf"));
    assert!(
        (without - 30.0).abs() < 1e-6,
        "Bellone alone reads her base: {without:.1}"
    );
    assert!(
        (resting - 40.0).abs() < 1e-6,
        "Vigil resting in a dormitory is in the Base: {resting:.1}"
    );
    assert!(
        (working - 40.0).abs() < 1e-6,
        "Vigil working counts too: {working:.1}"
    );
}

#[test]
fn morale_operator_takes_a_spare_cc_seat_and_lifts_sustain() {
    // A global morale-recovery operator (Wiš'adel: workers in other buildings recover
    // faster) should claim a spare Control Center seat instead of being burned as a
    // zero-value production filler, and its base-wide recovery boost must raise the
    // sustained output: the "add a morale operator" the rotation wants.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let layout = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("cc", "CONTROL", 5),
            room("tp", "TRADING", 3),
            room("mf", "MANUFACTURE", 3),
            room("d0", "DORMITORY", 2),
            room("d1", "DORMITORY", 2),
        ],
    };
    let roster = |morale: bool| {
        let mut r = vec![
            profile(gd, "char_103_angel"),
            profile(gd, "char_502_nblade"),
            profile(gd, "char_4032_provs"),
            profile(gd, "char_237_gravel"),
            profile(gd, "char_141_nights"),
        ];
        if morale {
            r.push(profile(gd, "char_1035_wisdel"));
        }
        r
    };

    let with =
        compute_sustained_assignment(&roster(true), &layout, gd.building(), &registry, &drains);
    let without =
        compute_sustained_assignment(&roster(false), &layout, gd.building(), &registry, &drains);

    let cc = with
        .main
        .rooms
        .iter()
        .find(|r| r.room_type == "CONTROL")
        .expect("a control center");
    assert!(
        cc.operators.iter().any(|o| o == "char_1035_wisdel"),
        "the morale operator should take the spare CC seat. Got: {:?}",
        cc.operators
    );
    assert!(
        with.sustained_efficiency > without.sustained_efficiency,
        "the morale operator's recovery boost should raise sustained output ({:.2} vs {:.2})",
        with.sustained_efficiency,
        without.sustained_efficiency
    );
}

#[test]
fn insufficient_dorm_capacity_throttles_sustained_output() {
    // A base can only rest as many operators at once as its dorms hold. With the same
    // production rooms and the same dorm LEVEL (so recovery rate is identical), fewer
    // dorms (less capacity) means the rotation can't rest everyone and sustained
    // output is throttled. This is the "you can only rest 20, so you need 3 sets" cap.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let layout = |dorms: usize| {
        let mut rooms = vec![room("tp0", "TRADING", 3), room("tp1", "TRADING", 3)];
        rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
        rooms.extend((0..dorms).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
        UserBuilding {
            drone_capacity: None,
            rooms,
        }
    };
    let roster = full_roster(gd);
    let cramped =
        compute_sustained_assignment(&roster, &layout(1), gd.building(), &registry, &drains);
    let roomy =
        compute_sustained_assignment(&roster, &layout(4), gd.building(), &registry, &drains);
    assert!(
        cramped.sustained_efficiency < roomy.sustained_efficiency,
        "too few dorms should throttle sustained output (cramped {:.1} vs roomy {:.1})",
        cramped.sustained_efficiency,
        roomy.sustained_efficiency
    );
}

#[test]
fn owning_fiammetta_proactively_sustains_the_best_trading_operator() {
    // NO preset evidence at all: owning a morale-swap manager alone makes the rotation
    // recommend holding the highest-gain trading operator (high team output x high morale
    // drain, a Shamare/Proviso-type) at full morale 24/7, pinned to one post every shift.
    // Shamare's skill burns extra morale per hour, so she gains the most from the manager.
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    const SHAMARE: &str = "char_254_vodfox";
    const TEQUILA: &str = "char_486_takila";
    const FIAMMETTA: &str = "char_300_phenxi";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 5)],
    };
    // Six trading BODIES besides the manager: the rotation reserves Fiammetta into
    // the dorms itself, so she must not double as a filler.
    let roster: Vec<_> = [
        SHAMARE,
        TEQUILA,
        FIAMMETTA,
        EXUSIAI,
        "char_502_nblade",
        "char_185_frncat",
        "char_123_fang",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .collect();
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);
    assert_eq!(
        rot.sustained,
        vec![SHAMARE.to_string()],
        "owning Fiammetta proactively sustains the highest-gain (high-drain) trader"
    );
    for s in &rot.shifts {
        let tp = s
            .rooms
            .iter()
            .find(|r| r.slot_id == "tp")
            .expect("the post");
        assert!(
            tp.active && tp.recommended.iter().any(|o| o == SHAMARE),
            "the sustained operator works every shift (shift {}: {:?})",
            s.index,
            tp.recommended
        );
    }
}

/// Pozëmka's production lines: +5% per Pure Gold factory, plus one line per
/// Durin operator in the base (dormitories included, capped at 4). Two gold
/// factories and three Durins resting = 5 lines = +25; without the Durins +10.
#[test]
fn pozemka_counts_gold_lines_and_resting_durins() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const POZEMKA: &str = "char_4055_bgsnow";
    let durins = ["char_4164_tecno", "char_4054_malist", "char_501_durin"];
    let mut ids = vec![POZEMKA];
    ids.extend(durins);
    let roster: Vec<_> = ids.iter().map(|id| profile(gd, id)).collect();
    let eff = |park: bool| {
        let mut rooms = vec![
            room("tp", "TRADING", 3),
            room("g0", "MANUFACTURE", 3),
            room("g1", "MANUFACTURE", 3),
            room("d0", "DORMITORY", 5),
        ];
        rooms[0].current_operators = vec![POZEMKA.into()];
        rooms[1].current_formula = Some("F_GOLD".into());
        rooms[2].current_formula = Some("F_GOLD".into());
        if park {
            rooms[3].current_operators = durins.iter().map(|s| (*s).to_string()).collect();
        }
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "tp")
            .map_or(0.0, |r| r.total_efficiency)
    };
    assert!(
        (eff(false) - 10.0).abs() < 1e-9,
        "two gold lines: {}",
        eff(false)
    );
    assert!(
        (eff(true) - 25.0).abs() < 1e-9,
        "plus three resting Durins: {}",
        eff(true)
    );
}

/// Virtuosa's "for every 1 Operators in that Dormitory, Soundless Resonance
/// +1" feeds Ebenholz: five in her dormitory = 5 Soundless Resonance on top
/// of his own dorm-fed conversion (user feedback 2026-09-13: her skill was
/// not counted).
#[test]
fn virtuosa_feeds_ebenholz_from_her_own_dormitory() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const EBENHOLZ: &str = "char_4046_ebnhlz";
    const VIRTUOSA: &str = "char_245_cello";
    assert!(
        matches!(
            registry.get("dorm_bd_num[000]"),
            Some(BuffResolutionStrategy::PoolGenerateOwnRoomOccupants { .. })
        ),
        "{:?}",
        registry.get("dorm_bd_num[000]")
    );
    let fillers = [
        "char_124_kroos",
        "char_278_orchid",
        "char_328_cammou",
        "char_282_catap",
    ];
    let mut ids = vec![EBENHOLZ, VIRTUOSA];
    ids.extend(fillers);
    let roster: Vec<_> = ids.iter().map(|id| profile(gd, id)).collect();
    let eff = |with_virtuosa: bool| {
        let mut rooms = vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 5)];
        rooms[0].current_operators = vec![EBENHOLZ.into()];
        rooms[1].current_operators = fillers.iter().map(|s| (*s).to_string()).collect();
        if with_virtuosa {
            rooms[1].current_operators.push(VIRTUOSA.into());
        }
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "tp")
            .map_or(0.0, |r| r.total_efficiency)
    };
    // Ebenholz β: +1% per 2 Soundless Resonance. Four sleepers: 4 PI -> 4 SR
    // = +2. With Virtuosa: 5 sleepers -> 5 SR from him + 5 from her = 10 = +5.
    assert!(
        (eff(false) - 2.0).abs() < 1e-9,
        "without her: {}",
        eff(false)
    );
    assert!((eff(true) - 5.0).abs() < 1e-9, "with her: {}", eff(true));
}

/// Iris and Czerny each feed 1 Perception point per level of their OWN
/// dormitory (the feedback sheet's in-game check: 1 to 5, never a flat 5),
/// and their converter skills are converters, not +1.0 morale auras.
#[test]
fn iris_and_czerny_feed_perception_per_own_dormitory_level() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    assert!(
        matches!(
            registry.get("dorm_rec_bd_n1[000]"),
            Some(BuffResolutionStrategy::PoolConvert { .. })
        ),
        "Iris converts: {:?}",
        registry.get("dorm_rec_bd_n1[000]")
    );
    assert!(
        matches!(
            registry.get("dorm_rec_bd_n1[100]"),
            Some(BuffResolutionStrategy::PoolConvert { .. })
        ),
        "Czerny converts: {:?}",
        registry.get("dorm_rec_bd_n1[100]")
    );
    const ROSMONTIS: &str = "char_391_rosmon";
    const IRIS: &str = "char_338_iris";
    const CZERNY: &str = "char_4047_pianst";
    let roster: Vec<_> = [ROSMONTIS, IRIS, CZERNY]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let eff = |dorm_level: i32| {
        let mut rooms = vec![
            room("mf", "MANUFACTURE", 3),
            room("d0", "DORMITORY", dorm_level),
        ];
        rooms[0].current_operators = vec![ROSMONTIS.into()];
        rooms[0].current_formula = Some("F_EXP".into());
        rooms[1].current_operators = vec![IRIS.into(), CZERNY.into()];
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None)
            .rooms
            .iter()
            .find(|r| r.slot_id == "mf")
            .map_or(0.0, |r| r.total_efficiency)
    };
    // Rosmontis β 1:1: two sleepers (2) + Iris L + Czerny L.
    assert!((eff(1) - 4.0).abs() < 1e-9, "level-1 dorm: {}", eff(1));
    assert!((eff(5) - 12.0).abs() < 1e-9, "level-5 dorm: {}", eff(5));
}

/// The Control-Center morale family credits an aura ONLY when the text's
/// subject really is an aura scope ("all Operators in the Control Center" /
/// "other buildings"). Named-partner gates (Mr. Lee needs Aak; the Amiya pair
/// skills) and self-subject conditionals (Gladiia's Abyssal-dependent ±0.5)
/// price 0; a flat parse credited them all as unconditional room auras.
#[test]
fn cc_morale_auras_credit_only_true_aura_scopes() {
    use myrtle_engine_base::registry::BuffResolutionStrategy;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    let recovery = |id: &str| match registry.get(id) {
        Some(BuffResolutionStrategy::MoraleModifier {
            recovery_per_hour,
            base_wide,
            ..
        }) => (*recovery_per_hour, *base_wide),
        other => panic!("{id}: expected MoraleModifier, got {other:?}"),
    };
    // Mr. Lee's "together with Aak" +0.25: gated, prices 0 without crew context.
    assert_eq!(recovery("control_allCost_condChar[000]"), (0.0, false));
    // The Amiya pair skill: partner-gated AND self+Amiya scoped, not a room aura.
    assert_eq!(recovery("control_mp_cost_double[000]"), (0.0, false));
    // Gladiia's Abyssal-conditional SELF ±0.5: not an aura at all.
    assert_eq!(recovery("control_mp_aegir1[000]"), (0.0, false));
    // Her real aura stays: +0.05 to all CC operators, room-local.
    assert_eq!(recovery("control_mp_aegir2[000]"), (0.05, false));
    // The plain +0.05 CC-room family is untouched.
    assert_eq!(recovery("control_mp_cost[000]"), (0.05, false));
    // Base-wide "other buildings" auras keep their unconditional base value.
    assert_eq!(recovery("control_mp_bd_cost_expand[000]"), (0.05, true));
    assert_eq!(recovery("control_mp_expand_double[000]"), (0.1, true));
}

/// Dormitory LEVELS decide recovery: the sim rests each operator at the rate
/// of the SPECIFIC dorm they land in (best dorm first), so a 2/5/2-style base
/// with under-leveled dorms recovers slower than a full-dorm 243,
/// enough to flip a heavy drainer's verdict. A dorm-skill aura holder seated
/// as permanent staff speeds that dorm up for whoever rests beside them.
#[test]
fn dorm_levels_and_staffed_boosters_shape_recovery() {
    use myrtle_engine_base::rotation::shift::{Shift, ShiftRoom, ShiftRotation};
    use myrtle_engine_base::rotation::sustain_sim::{Verdict, simulate_rotation};
    const BODY: &str = "char_102_texas";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    // A real whole-dorm aura holder ("+X/hr to all Operators in that
    // Dormitory"), discovered from the parsed registry.
    use myrtle_engine_base::registry::BuffResolutionStrategy;
    // The STRONGEST aura owner, so the pick is deterministic (a HashMap walk
    // could land on a +0.1 "all other Operators" tier, which defers the leak
    // by less than the hour this test demands).
    let aura_owner = gd
        .building()
        .chars
        .iter()
        .filter_map(|(id, c)| {
            let best = c
                .buff_char
                .iter()
                .flat_map(|bc| bc.buff_data.iter())
                .filter(|bd| {
                    gd.building()
                        .buffs
                        .get(&bd.buff_id)
                        .is_some_and(|b| b.room_type == "DORMITORY")
                })
                .filter_map(|bd| match registry.get(&bd.buff_id) {
                    Some(BuffResolutionStrategy::MoraleModifier {
                        recovery_per_hour,
                        is_self_only: false,
                        single_target: false,
                        base_wide: false,
                    }) if *recovery_per_hour > 0.0 => Some(*recovery_per_hour),
                    _ => None,
                })
                .fold(0.0_f64, f64::max);
            (best > 0.0).then_some((id.clone(), best))
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then_with(|| b.0.cmp(&a.0)))
        .map(|(id, _)| id)
        .expect("some operator owns a whole-dorm recovery aura");

    // Trading duty with a drain rider so the cycle leaks: Texas (+0.25/hr =
    // 1.25) works shifts 1+2 and rests shift 3. Spend 30/cycle; a dorm
    // recovers 12h x rate. L5 (2.0/hr = 24) nearly covers it; L1 (1.6/hr =
    // 19.2) leaks ~10.8/cycle and depletes within the week.
    let dorm_cell = |slot: &str, crew: Vec<String>| ShiftRoom {
        slot_id: slot.into(),
        room_type: "DORMITORY".into(),
        formula_type: None,
        recommended: crew,
        current: Vec::new(),
        active: true,
        efficiency: None,
        team_id: None,
        team_label: None,
    };
    let mk = |staff: Vec<String>| ShiftRotation {
        parked: Vec::new(),
        shifts: (1..=3)
            .map(|index| Shift {
                index,
                rooms: vec![
                    ShiftRoom {
                        slot_id: "tp".into(),
                        room_type: "TRADING".into(),
                        formula_type: None,
                        recommended: if index <= 2 {
                            vec![BODY.to_string()]
                        } else {
                            Vec::new()
                        },
                        current: Vec::new(),
                        active: index <= 2,
                        efficiency: None,
                        team_id: None,
                        team_label: None,
                    },
                    dorm_cell("d0", staff.clone()),
                ],
            })
            .collect(),
        sustained: Vec::new(),
        bench: Vec::new(),
    };
    let building_with = |dorm_level: i32| UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("tp", "TRADING", 3),
            room("d0", "DORMITORY", dorm_level),
        ],
    };
    let outcome = |dorm_level: i32, staff: Vec<String>| -> (Verdict, f64) {
        let mut ids = vec![BODY.to_string()];
        ids.extend(staff.iter().cloned());
        let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        let report = simulate_rotation(
            &mk(staff),
            &roster,
            &building_with(dorm_level),
            gd.building(),
            &registry,
            &drains,
            &std::collections::HashMap::new(),
        );
        let first = report
            .depleted
            .first()
            .map_or(f64::INFINITY, |d| d.at_hours);
        (report.verdict, first)
    };

    // Level 1 dorm leaks; level 5 holds the same rhythm.
    let (v_low, low_at) = outcome(1, Vec::new());
    let (v_high, _) = outcome(5, Vec::new());
    assert_eq!(
        v_low,
        Verdict::Depletes,
        "an L1 dorm can't cover a 1.25 drainer"
    );
    assert_eq!(
        v_high,
        Verdict::HoldsUp,
        "an L5 dorm covers the same rhythm"
    );

    // A staffed whole-dorm aura holder speeds the LOW dorm: their +X/hr on
    // top of the L1 rate shrinks the leak, so depletion comes strictly later.
    let (_, staffed_at) = outcome(1, vec![aura_owner]);
    assert!(
        staffed_at > low_at + 1.0,
        "a permanent aura resident must defer the L1 dorm's depletion \
         ({staffed_at}h vs {low_at}h)"
    );
}

/// Waaifu's Team Spirit makes her IMMUNE to teammates' room morale auras
/// (Shu's -0.1/hr factory aura passes her by, in either direction), and
/// Cement's Vlog cuts her drain by 0.25/hr only while the factory produces
/// Battle Records. The sustainability sim charges both.
#[test]
fn aura_immunity_and_formula_drain_shape_the_sim() {
    use myrtle_engine_base::registry::targeted_morale_effects;
    use myrtle_engine_base::rotation::shift::{Shift, ShiftRoom, ShiftRotation};
    use myrtle_engine_base::rotation::sustain_sim::simulate_rotation;
    const WAAIFU: &str = "char_243_waaifu";
    const CEMENT: &str = "char_328_cammou";
    const SHU: &str = "char_2025_shu"; // owns the -0.1 factory aura
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let targeted = targeted_morale_effects(&gd.building().buffs, &name_to_char);
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("mf", "MANUFACTURE", 3), room("d0", "DORMITORY", 2)],
    };
    // A 36h unbroken factory stretch: depletion hour = 24 / effective drain.
    let mk = |crew: Vec<&str>, formula: &str| ShiftRotation {
        parked: Vec::new(),
        shifts: (1..=3)
            .map(|index| Shift {
                index,
                rooms: vec![ShiftRoom {
                    slot_id: "mf".into(),
                    room_type: "MANUFACTURE".into(),
                    formula_type: Some(formula.to_string()),
                    recommended: crew.iter().map(|s| (*s).to_string()).collect(),
                    current: Vec::new(),
                    active: true,
                    efficiency: None,
                    team_id: None,
                    team_label: None,
                }],
            })
            .collect(),
        sustained: Vec::new(),
        bench: Vec::new(),
    };
    let at = |crew: Vec<&str>, formula: &str, who: &str| -> f64 {
        let roster: Vec<OperatorBaseProfile> = crew.iter().map(|id| profile(gd, id)).collect();
        let report = simulate_rotation(
            &mk(crew, formula),
            &roster,
            &building,
            gd.building(),
            &registry,
            &drains,
            &targeted,
        );
        report
            .depleted
            .iter()
            .find(|d| d.char_id == who)
            .map(|d| d.at_hours)
            .expect("a 36h stretch depletes")
    };

    // Vlog: -0.25 only while producing Battle Records (F_EXP).
    let exp = at(vec![CEMENT], "F_EXP", CEMENT); // 24/0.75 = 32h
    let gold = at(vec![CEMENT], "F_GOLD", CEMENT); // 24h
    assert!(
        exp > gold + 4.0,
        "Vlog only pays off on Battle Records ({exp}h vs {gold}h)"
    );

    // Team Spirit: Shu's -0.1 aura stretches HER OWN clock but not Waaifu's.
    let shu_at = at(vec![WAAIFU, SHU], "F_GOLD", SHU); // 24/0.9 ≈ 26.7h
    let waaifu_at = at(vec![WAAIFU, SHU], "F_GOLD", WAAIFU); // immune: 24h
    assert!(
        shu_at > waaifu_at + 1.0,
        "the aura helps Shu but passes immune Waaifu by ({shu_at}h vs {waaifu_at}h)"
    );
    assert!(
        (waaifu_at - 24.0).abs() < 0.5,
        "Waaifu drains at the plain rate despite the aura ({waaifu_at}h)"
    );
}

/// The Control Center's own recovery auras (`control_mp_cost`: "+0.05/hr to
/// all Operators in the Control Center") offset its workers' drain in the
/// sustainability sim, and ONLY its workers: the aura is room-local, so a
/// trading-post worker in the same rotation gains nothing from it.
#[test]
fn cc_recovery_auras_offset_cc_workers_drain_only() {
    use myrtle_engine_base::rotation::shift::{Shift, ShiftRoom, ShiftRotation};
    use myrtle_engine_base::rotation::sustain_sim::simulate_rotation;
    const BODY_CC: &str = "char_102_texas";
    const BODY_TP: &str = "char_140_whitew";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    // A real owner of the CC-room recovery aura, from gamedata.
    let aura_owner = gd
        .building()
        .chars
        .iter()
        .find(|(_, c)| {
            c.buff_char.iter().any(|bc| {
                bc.buff_data
                    .iter()
                    .any(|bd| bd.buff_id.starts_with("control_mp_cost["))
            })
        })
        .map(|(id, _)| id.clone())
        .expect("some operator owns the CC recovery aura");

    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("cc", "CONTROL", 5),
            room("tp", "TRADING", 3),
            room("d0", "DORMITORY", 2),
        ],
    };
    // Everyone works ALL THREE shifts (36h unbroken): a neutral 1.0/hr drain
    // depletes at exactly 24h; a -0.05 aura stretches that to 24/0.95 ≈ 25.3h.
    let mk = |cc_crew: Vec<String>| ShiftRotation {
        parked: Vec::new(),
        shifts: (1..=3)
            .map(|index| Shift {
                index,
                rooms: vec![
                    ShiftRoom {
                        slot_id: "cc".into(),
                        room_type: "CONTROL".into(),
                        formula_type: None,
                        recommended: cc_crew.clone(),
                        current: Vec::new(),
                        active: true,
                        efficiency: None,
                        team_id: None,
                        team_label: None,
                    },
                    ShiftRoom {
                        slot_id: "tp".into(),
                        room_type: "TRADING".into(),
                        formula_type: None,
                        recommended: vec![BODY_TP.to_string()],
                        current: Vec::new(),
                        active: true,
                        efficiency: None,
                        team_id: None,
                        team_label: None,
                    },
                ],
            })
            .collect(),
        sustained: Vec::new(),
        bench: Vec::new(),
    };
    let depleted_at = |rotation: &ShiftRotation, ids: &[&str], who: &str| -> f64 {
        let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        let report = simulate_rotation(
            rotation,
            &roster,
            &building,
            gd.building(),
            &registry,
            &drains,
            &std::collections::HashMap::new(),
        );
        report
            .depleted
            .iter()
            .find(|d| d.char_id == who)
            .map(|d| d.at_hours)
            .expect("a 36h stretch depletes a neutral operator")
    };

    let without = mk(vec![BODY_CC.to_string()]);
    let with_aura = mk(vec![BODY_CC.to_string(), aura_owner.clone()]);
    let ids_with = [BODY_CC, BODY_TP, aura_owner.as_str()];
    let cc_before = depleted_at(&without, &[BODY_CC, BODY_TP], BODY_CC);
    let cc_after = depleted_at(&with_aura, &ids_with, BODY_CC);
    assert!(
        cc_after > cc_before + 0.5,
        "the CC aura must stretch a CC worker's bar ({cc_after}h vs {cc_before}h)"
    );
    // Room-local: the trading worker's clock must not move.
    let tp_before = depleted_at(&without, &[BODY_CC, BODY_TP], BODY_TP);
    let tp_after = depleted_at(&with_aura, &ids_with, BODY_TP);
    assert!(
        (tp_after - tp_before).abs() < 1e-6,
        "a CC-room aura must not reach a trading post ({tp_after}h vs {tp_before}h)"
    );
}

/// The planner's `lasts_hours` reads the same room-wide drain-aura model the
/// sustainability simulator charges: a trading-post aura carrier stretches
/// every TEAMMATE's swap clock, not just its own; previously the display
/// said "swap Texas at 24h" while the simulator knew she'd last far longer.
#[test]
fn lasts_hours_reads_the_room_drain_aura() {
    use myrtle_engine_base::assignment::compute_sustained_assignment;
    const BODY: &str = "char_102_texas";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    // A real owner of the trading drain-reduction aura, from gamedata.
    let aura_owner = gd
        .building()
        .chars
        .iter()
        .find(|(_, c)| {
            c.buff_char.iter().any(|bc| {
                bc.buff_data
                    .iter()
                    .any(|bd| bd.buff_id.starts_with("trade_cost["))
            })
        })
        .map(|(id, _)| id.clone())
        .expect("some operator owns the trading drain aura");

    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 2)],
    };
    let hours_of = |ids: &[&str]| -> f64 {
        let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        let plan =
            compute_sustained_assignment(&roster, &building, gd.building(), &registry, &drains);
        plan.rooms
            .iter()
            .flat_map(|r| r.members.iter())
            .find(|m| m.operator == BODY)
            .map(|m| m.lasts_hours)
            .expect("the body operator is in the rotation plan")
    };
    let without = hours_of(&[BODY, "char_140_whitew"]);
    let with_aura = hours_of(&[BODY, "char_140_whitew", aura_owner.as_str()]);
    assert!(
        with_aura > without,
        "the drain-reduction aura must stretch a teammate's swap clock \
         ({with_aura}h with vs {without}h without)"
    );
}

/// The simulator flags an UNSUSTAINABLE rhythm: with L1 dorms (+1.6/hr), a 12h
/// rest recovers only ~19 of the 24 points a 24h block drains, so production
/// operators leak ~5 morale per cycle and run dry within the week; the
/// verdict must say "depletes" and name them, not wave the plan through.
#[test]
fn morale_sim_flags_underbuilt_dorms() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    use myrtle_engine_base::rotation::sustain_sim::{Verdict, simulate_rotation};
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let mut rooms = vec![
        room("cc", "CONTROL", 5),
        room("hr", "HIRE", 3),
        room("rc", "MEETING", 3),
    ];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("p{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 1)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let roster = full_roster(gd);
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);
    let report = simulate_rotation(
        &rot,
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
        &std::collections::HashMap::new(),
    );
    assert_eq!(
        report.verdict,
        Verdict::Depletes,
        "L1 dorms cannot sustain 24h blocks"
    );
    // The leak is SYSTEMIC (many baseline operators bleed ~5 morale per cycle),
    // not one exotic heavy-drainer; and nobody can empty a full bar inside a
    // single 12h shift (that would need >2.0/hr drain, which no skill reaches).
    assert!(
        report.depleted.len() >= 5,
        "underbuilt dorms leak across the whole rotation: {:?}",
        report
            .depleted
            .iter()
            .map(|d| d.at_hours)
            .collect::<Vec<_>>()
    );
    assert!(
        report.depleted.iter().all(|d| d.at_hours > 12.0),
        "no operator empties a full bar within one shift: {:?}",
        report
            .depleted
            .iter()
            .map(|d| d.at_hours)
            .collect::<Vec<_>>()
    );
}

/// Morale AURAS flow through the simulator: a teammate whose skill slows the
/// whole room's drain ("-0.1/hr to all Operators in the Trading Post") and a
/// Control-Center aura lifting every dormitory sleeper's recovery both change
/// the week's morale arithmetic. With L2 dorms (+1.7/hr), a 24h block drains
/// 24 but a 12h rest recovers only 20.4: operators leak and deplete. The
/// room aura cuts the block's drain to 21.6, still short, so fewer deplete
/// and later.
#[test]
fn morale_auras_change_the_sustainability_arithmetic() {
    use myrtle_engine_base::rotation::shift::{Shift, ShiftRoom, ShiftRotation};
    use myrtle_engine_base::rotation::sustain_sim::{Verdict, simulate_rotation};
    const AURA: &str = "char_4171_wulfen"; // unused; the owner is looked up below
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    // Find a real owner of the trading drain aura from gamedata, no hardcoded id.
    let aura_owner = gd
        .building()
        .chars
        .iter()
        .find(|(_, c)| {
            c.buff_char.iter().any(|bc| {
                bc.buff_data
                    .iter()
                    .any(|bd| bd.buff_id.starts_with("trade_cost["))
            })
        })
        .map(|(id, _)| id.clone())
        .expect("some operator owns the trading drain aura");
    let _ = AURA;
    // Two neutral bodies + the aura carrier; everyone at E2-max via profile().
    let bodies = ["char_102_texas", "char_140_whitew"];
    let mut ids: Vec<String> = bodies.iter().map(|s| (*s).to_string()).collect();
    ids.push(aura_owner);
    let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();

    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 2)],
    };
    // A hand-built 3-shift rotation: the trio works shifts 1+2 (a 24h block)
    // and rests shift 3, the exact rhythm the planner emits.
    let mk_rotation = |crew: Vec<String>| ShiftRotation {
        parked: Vec::new(),
        shifts: (1..=3)
            .map(|index| Shift {
                index,
                rooms: vec![ShiftRoom {
                    slot_id: "tp".into(),
                    room_type: "TRADING".into(),
                    formula_type: None,
                    recommended: if index <= 2 { crew.clone() } else { Vec::new() },
                    current: Vec::new(),
                    active: index <= 2,
                    efficiency: None,
                    team_id: None,
                    team_label: None,
                }],
            })
            .collect(),
        sustained: Vec::new(),
        bench: Vec::new(),
    };

    // Without the aura carrier: 24h at 1.0/hr vs 12h at L2's +1.7/hr -> leak -> depletes.
    let without = simulate_rotation(
        &mk_rotation(bodies.iter().map(|s| (*s).to_string()).collect()),
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
        &std::collections::HashMap::new(),
    );
    assert_eq!(
        without.verdict,
        Verdict::Depletes,
        "L2 dorms can't cover a full-drain 24h block"
    );

    // With the aura carrier in the room: everyone drains 0.9/hr -> 21.6/block,
    // covered by 12h x 1.7 = 20.4... still short; but the leak shrinks enough
    // that nobody empties WITHIN the week that previously did. The precise pin:
    // depletion count strictly decreases and first depletion happens later.
    let with = simulate_rotation(
        &mk_rotation(ids.clone()),
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
        &std::collections::HashMap::new(),
    );
    let first_without = without.depleted.first().map_or(0.0, |d| d.at_hours);
    let first_with = with.depleted.first().map_or(f64::INFINITY, |d| d.at_hours);
    assert!(
        first_with > first_without + 12.0,
        "the room aura defers depletion by shifts: {first_without} -> {first_with}"
    );
}

/// Dorm ambience: a maxed-furniture dorm (comfort 5000) recovers +2.0/hr on
/// top of its level rate, per the game's `ComfortManpowerRecoverFactor`.
#[test]
fn dorm_comfort_adds_recovery() {
    use myrtle_engine_base::rotation::dorms::dorm_list;
    let gd = load_game_data();
    let mut building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("d0", "DORMITORY", 5), room("d1", "DORMITORY", 5)],
    };
    building.rooms[0].comfort = 5000;
    let dorms = dorm_list(&building, gd.building());
    let rate = |slot: &str| {
        dorms
            .iter()
            .find(|d| d.slot_id == slot)
            .expect("dorm listed")
            .recovery_per_hour
    };
    assert!((rate("d1") - 2.0).abs() < 1e-9, "bare L5 dorm is 2.0/hr");
    assert!(
        (rate("d0") - 4.0).abs() < 1e-9,
        "maxed ambience adds +2.0/hr, got {}",
        rate("d0")
    );
    // Best-first ordering now prefers the furnished dorm.
    assert_eq!(dorms[0].slot_id, "d0");
}

/// Waai Fu's Team Spirit: she IGNORES roommates' drain auras in a Factory.
/// The registry prices it as a real morale effect (never "unmodeled"), and
/// the rotation planner exempts her from the room's shared drain aura.
#[test]
fn waai_fu_ignores_factory_drain_auras() {
    use myrtle_engine_base::assignment::has_drain_aura_immunity;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    assert!(
        matches!(
            registry.get("manu_cost_all[000]"),
            Some(BuffResolutionStrategy::MoraleDrainAuraImmunity)
        ),
        "Team Spirit parses as drain-aura immunity, got {:?}",
        registry.get("manu_cost_all[000]")
    );
    let waaifu = profile(gd, "char_243_waaifu");
    assert!(
        has_drain_aura_immunity(&waaifu, "MANUFACTURE", &registry, gd.building()),
        "immunity detected in factories"
    );
    assert!(
        !has_drain_aura_immunity(&waaifu, "TRADING", &registry, gd.building()),
        "her skill is factory-scoped"
    );
}

/// Rosmontis' dorm-occupancy pool must rank her ABOVE zero before the scorer
/// ever sees a team: the optimistic bound assumes full dormitories and
/// follows her Perception -> Chain of Thought conversion.
#[test]
fn rosmontis_ranks_on_full_dormitories() {
    use myrtle_engine_base::scoring::ledger::op_optimistic_bound;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    let rosmontis = profile(gd, "char_391_rosmon");
    let mut counts = std::collections::HashMap::new();
    counts.insert("DORMITORY".to_string(), 4usize);
    let bound = op_optimistic_bound(
        &rosmontis,
        "MANUFACTURE",
        Some("F_EXP"),
        &registry,
        gd.building(),
        &counts,
        20,
        3,
        &[],
    );
    // Four top-level dorms = 20 seats -> 20 Perception -> 20 Chain -> +20%.
    assert!(
        (bound - 20.0).abs() < 1e-6,
        "Rosmontis' bound at four full dorms, got {bound}"
    );
}

/// The live settlement reads a morale-conditional grant off the seated
/// operator's REAL bar when the sync carries it: Dusk's "when self morale is
/// above 12, Perception Information +10" is the full 10 while she is above
/// 12 (what the game's counter shows), nothing once below, and only the
/// steady-state half when her bar is unknown.
#[test]
fn live_settlement_reads_dusks_grant_off_her_real_morale() {
    use myrtle_engine_base::assignment::compute_live_assignment;
    use std::collections::HashMap;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const ROSMONTIS: &str = "char_391_rosmon";
    const DUSK: &str = "char_2015_dusk";
    let mut rooms = vec![room("mf", "MANUFACTURE", 3), room("cc", "CONTROL", 5)];
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let mut ids: Vec<&str> = vec![ROSMONTIS, DUSK];
    ids.extend(
        gd.building()
            .chars
            .keys()
            .filter(|id| id.starts_with("char_") && *id != ROSMONTIS && *id != DUSK)
            .take(20)
            .map(String::as_str),
    );
    let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
    rooms[0].current_operators = vec![ROSMONTIS.to_string()];
    rooms[0].current_formula = Some("F_EXP".to_string());
    rooms[1].current_operators = vec![DUSK.to_string()];
    for (i, id) in ids[2..].iter().enumerate() {
        rooms[2 + i / 5].current_operators.push((*id).to_string());
    }
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let factory = |live: HashMap<String, f64>| {
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
        .into_iter()
        .find(|r| r.slot_id == "mf")
        .map_or(0.0, |r| r.total_efficiency)
    };
    let unknown = factory(HashMap::new());
    let above = factory(HashMap::from([(DUSK.to_string(), 20.0)]));
    let below = factory(HashMap::from([(DUSK.to_string(), 8.0)]));
    assert!(
        (above - below - 10.0).abs() < 1e-9,
        "above {above} vs below {below}: her 10 is all or nothing"
    );
    assert!(
        (unknown - below - 5.0).abs() < 1e-9,
        "unknown {unknown} vs below {below}: the steady-state half"
    );
}

/// Durin-class compound texts ("self Morale recovered per hour -0.1, but
/// restores +0.2 Morale per hour to all Operators assigned to that Dormitory")
/// are whole-dorm AURAS, not self-only skills. Community-confirmed 2026-08-24:
/// the aura applies to the holder fully too (net +0.1 on her own bar), and it
/// competes under the game's strongest-effect rule like every other dorm aura
/// (Lumen-class auras do NOT stack either; the model's max-fold is correct).
#[test]
fn durin_compound_text_is_a_whole_dorm_aura() {
    use myrtle_engine_base::rotation::dorms::dorm_aura_value;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    // Both tiers of the shared family (six owners: Durin, Hellagur, Glaze,
    // Bagpipe, ...) parse as auras at the ALL-Operators figure, never at the
    // self-malus the "self" heuristic used to file them under.
    for (id, want) in [
        ("dorm_rec_all&oneself[000]", 0.2),
        ("dorm_rec_all&oneself[001]", 0.25),
        // The sibling shape: "self +0.55, and restores +0.1 Morale per hour
        // to all OTHER Operators" (Hellagur/Bagpipe/Mint/Fartooth tiers). The
        // self rider is priced nowhere, so the others-aura is the skill's
        // whole model-relevant value.
        ("dorm_rec_all&oneself[021]", 0.1),
    ] {
        match registry.get(id) {
            Some(BuffResolutionStrategy::MoraleModifier {
                recovery_per_hour,
                is_self_only,
                single_target,
                ..
            }) => {
                assert!(
                    (recovery_per_hour - want).abs() < 1e-9,
                    "{id}: aura value {recovery_per_hour}, want {want}"
                );
                assert!(!is_self_only, "{id}: an aura, not a self-only skill");
                assert!(!single_target, "{id}: whole-dorm, not single-target");
            }
            other => panic!("{id}: expected MoraleModifier, got {other:?}"),
        }
    }
    // Dorm staffing sees her: she prices at her max-tier aura value.
    let durin = profile(gd, "char_501_durin");
    let v = dorm_aura_value(&durin, &registry, gd.building());
    assert!((v - 0.25).abs() < 1e-9, "Durin's staffing value, got {v}");
}

/// A Control-Center morale aura paid per seated operator of a faction
/// ("each Alternate Operator increases the Morale of all Operators in the
/// Control Center by +0.05") is not a flat aura: priced flat it seated Lava
/// the Purgatory beside no Alternate (31010962). Until it is count-scaled
/// it prices 0, like the partner-gated kind; the plain aura still reads.
#[test]
fn a_per_faction_cc_morale_aura_is_not_priced_flat() {
    let gd = load_game_data();
    let (registry, _) = build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    for id in [
        "control_mp_cost&faction[900]",
        "control_mp_cost&faction[000]",
        "control_mp_cost&faction[030]",
        "control_mp_cost&faction2[000]",
    ] {
        let Some(strategy) = registry.get(id) else {
            continue;
        };
        assert!(
            matches!(
                strategy,
                BuffResolutionStrategy::MoraleModifier { recovery_per_hour, .. } if *recovery_per_hour == 0.0
            ),
            "{id} prices no flat recovery, got {strategy:?}"
        );
    }
    // Viviana's plain "+0.05 to all Operators in the Control Center" still reads.
    assert!(
        matches!(
            registry.get("control_mp_cost[010]"),
            Some(BuffResolutionStrategy::MoraleModifier { recovery_per_hour, .. }) if (*recovery_per_hour - 0.05).abs() < 1e-9
        ),
        "a plain CC aura keeps its rate, got {:?}",
        registry.get("control_mp_cost[010]")
    );
}

/// Targeted morale effects between co-seated operators, the Ave Mujica
/// drama: Dolris' rider drains Sakiko +0.1/hr while they share the Control
/// Center, and Mortis' amnesty cancels Sakiko's OWN +0.05 rider. The sim
/// charges each exactly when the pair is co-seated.
#[test]
fn targeted_morale_effects_follow_co_seating() {
    use myrtle_engine_base::registry::targeted_morale_effects;
    use myrtle_engine_base::rotation::shift::{Shift, ShiftRoom, ShiftRotation};
    use myrtle_engine_base::rotation::sustain_sim::simulate_rotation;
    const SAKIKO: &str = "char_4182_oblvns";
    const MORTIS: &str = "char_4183_mortis";
    const DOLRIS: &str = "char_4184_dolris";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let targeted = targeted_morale_effects(&gd.building().buffs, &name_to_char);

    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("cc", "CONTROL", 5), room("d0", "DORMITORY", 2)],
    };
    // A 36h unbroken CC stretch, so depletion hour = 24 / effective drain.
    let mk = |crew: Vec<&str>| ShiftRotation {
        parked: Vec::new(),
        shifts: (1..=3)
            .map(|index| Shift {
                index,
                rooms: vec![ShiftRoom {
                    slot_id: "cc".into(),
                    room_type: "CONTROL".into(),
                    formula_type: None,
                    recommended: crew.iter().map(|s| (*s).to_string()).collect(),
                    current: Vec::new(),
                    active: true,
                    efficiency: None,
                    team_id: None,
                    team_label: None,
                }],
            })
            .collect(),
        sustained: Vec::new(),
        bench: Vec::new(),
    };
    let sakiko_at = |crew: Vec<&str>| -> f64 {
        let roster: Vec<OperatorBaseProfile> = crew.iter().map(|id| profile(gd, id)).collect();
        let report = simulate_rotation(
            &mk(crew),
            &roster,
            &building,
            gd.building(),
            &registry,
            &drains,
            &targeted,
        );
        report
            .depleted
            .iter()
            .find(|d| d.char_id == SAKIKO)
            .map(|d| d.at_hours)
            .expect("a 36h stretch depletes Sakiko")
    };

    const AMORIS: &str = "char_4185_amoris";
    let alone = sakiko_at(vec![SAKIKO]); // own +0.05 -> 24/1.05
    let with_mortis = sakiko_at(vec![SAKIKO, MORTIS]); // amnesty -> 24/1.0
    let with_dolris = sakiko_at(vec![SAKIKO, DOLRIS]); // +0.1 rider -> 24/1.15
    let with_amoris = sakiko_at(vec![SAKIKO, AMORIS]); // +0.05 rider -> 24/1.10
    assert!(
        with_amoris < alone - 0.5,
        "Amoris' rider drains Sakiko faster ({with_amoris}h vs {alone}h)"
    );
    assert!(
        (alone - 24.0 / 1.05).abs() < 0.1,
        "alone: her own rider drains 1.05/hr (got {alone}h)"
    );
    assert!(
        with_mortis > alone + 0.5,
        "Mortis' amnesty cancels her own rider ({with_mortis}h vs {alone}h)"
    );
    assert!(
        with_dolris < alone - 0.5,
        "Dolris' rider drains her faster ({with_dolris}h vs {alone}h)"
    );
}
