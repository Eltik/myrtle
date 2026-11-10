use super::*;

#[test]
fn control_center_clauseless_buffs_stack() {
    // Mon3tr's "+2% all Factories" carries the non-stacking clause, but Sakiko's
    // Precious-Metal productivity does NOT, so Sakiko stacks on top of Mon3tr.
    use myrtle_engine_base::assignment::compute_current_assignment;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let manu_global = |cc: &[&str]| -> f64 {
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
                    ..Default::default()
                },
            ],
        };
        let roster: Vec<_> = cc.iter().map(|s| profile(gd, s)).collect();
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.room_type == "MANUFACTURE")
            .map_or(0.0, |r| r.total_efficiency)
    };
    const MON3TR: &str = "char_4179_monstr";
    const SAKIKO: &str = "char_4182_oblvns";
    let mon = manu_global(&[MON3TR]);
    let both = manu_global(&[MON3TR, SAKIKO]);
    assert!(
        both > mon + 0.001,
        "Sakiko's clause-less buff must stack on Mon3tr ({both} vs {mon})"
    );
}

/// Aroma's "productivity per hour +2%, up to +20%" states its rate AFTER the
/// words "per hour": the ramp is 2 per hour (cap at 10 h), averaged over
/// the 24 h block = 15.8, not the half-rate 11.7.
#[test]
fn aroma_ramp_rate_parses_in_the_trailing_form() {
    let gd = load_game_data();
    let (registry, _) = build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    match registry.get("manu_prod_spd_addition[100]") {
        Some(BuffResolutionStrategy::TimeRamp {
            first_hour,
            per_hour,
            cap,
        }) => {
            // +2% after the first hour, +2% more each hour, to +20%: 12.5 over a
            // 12h shift, 16.25 over 24h, in whole-hour steps (riic.autos agrees).
            assert_eq!((*first_hour, *per_hour, *cap), (2.0, 2.0, 20.0));
            use myrtle_engine_base::scoring::clause::ramp_average;
            assert!((ramp_average(*first_hour, *per_hour, *cap, 12.0) - 12.5).abs() < 1e-9);
            assert!((ramp_average(*first_hour, *per_hour, *cap, 24.0) - 16.25).abs() < 1e-9);
        }
        other => panic!("Aroma's ramp: {other:?}"),
    }
}

#[test]
fn a_time_ramp_is_worth_its_average_over_the_crews_tenure() {
    // Aroma's "Routine Cleaning" (+2%/h up to +20%) restarts with every crew
    // change (2026-10-23, riic.autos parity): a 12h shift averages 12.5, a 24h
    // stay 16.25, and the live base (no tenure given) reads 24h. Her Clean
    // Breeze +25% on Precious Metal is flat either way.
    use myrtle_engine_base::assignment::{
        compute_current_assignment, compute_current_assignment_over,
    };
    const AROMA: &str = "char_446_aroma";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let mut factory = room("mf0", "MANUFACTURE", 3);
    factory.current_formula = Some("F_GOLD".into());
    factory.current_operators = vec![AROMA.into()];
    let building = UserBuilding {
        rooms: vec![factory],
        drone_capacity: None,
    };
    let roster = vec![profile(gd, AROMA)];
    let eff = |asn: &myrtle_engine_base::types::BaseAssignment| {
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "mf0")
            .expect("the factory is scored")
            .total_efficiency
    };
    let live = eff(&compute_current_assignment(
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
        None,
    ));
    let day = eff(&compute_current_assignment_over(
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
        None,
        24.0,
    ));
    let shift = eff(&compute_current_assignment_over(
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
        None,
        12.0,
    ));
    assert!(
        (live - day).abs() < 1e-9,
        "the live base reads a 24h tenure: {live} vs {day}"
    );
    assert!((day - 41.25).abs() < 1e-9, "25 + 16.25 over 24h, got {day}");
    assert!(
        (shift - 37.5).abs() < 1e-9,
        "25 + 12.5 over a 12h shift, got {shift}"
    );
}

// ─── Clause registry goldens: real gamedata buffs reduce to the expected shapes ──────────────

#[test]
fn clause_registry_goldens_on_real_gamedata() {
    use myrtle_engine_base::scoring::clause::{
        ClauseKind, CondScope, Metric, Subject, SuppressExempt, build_clauses,
    };
    let gd = load_game_data();
    let clauses = build_clauses(&gd.building().buffs, &build_name_to_char(gd.operators()));

    // Texas "Feud": no base, +65% gated on Lappland sharing the post.
    let texas = &clauses["trade_ord_spd&cost_P[000]"];
    assert!(
        texas.iter().any(|c| matches!(
            &c.kind,
            ClauseKind::RequiresChar { chars, scope: CondScope::Room } if !chars.is_empty()
        ) && c.value > 50.0),
        "Texas reduces to a Room-scoped RequiresChar worth +65: {texas:?}"
    );

    // Shamare "Whispers": per-body scaling + suppression that spares flat order value.
    let shamare = &clauses["trade_ord_vodfox[000]"];
    assert!(
        shamare.iter().any(|c| matches!(
            &c.kind,
            ClauseKind::ScalingCount {
                subject: Subject::AnyOtherOccupant,
                ..
            }
        )),
        "Shamare scales per body: {shamare:?}"
    );
    assert!(
        shamare.iter().any(|c| matches!(
            &c.kind,
            ClauseKind::SuppressesOthers { metrics, exempt: SuppressExempt::None }
                if metrics.contains(&Metric::OrderValue { pure_gold: true })
                    && !metrics.contains(&Metric::OrderValue { pure_gold: false })
        )),
        "Shamare suppresses speed + Pure-Gold value but spares flat value: {shamare:?}"
    );

    // Weedy automation: per-Power-Plant scaling + facility-count-exempt suppression.
    let weedy = &clauses["manu_prod_spd&power[010]"];
    assert!(
        weedy.iter().any(|c| matches!(
            &c.kind,
            ClauseKind::ScalingRoomCount { room } if room == "POWER"
        )),
        "Weedy scales per Power Plant: {weedy:?}"
    );
    assert!(
        weedy.iter().any(|c| matches!(
            &c.kind,
            ClauseKind::SuppressesOthers {
                exempt: SuppressExempt::RoomCountScaledSources,
                ..
            }
        )),
        "Weedy's suppression exempts facility-count productivity: {weedy:?}"
    );

    // Quartz "Precise Scheduling": +30 base + +2 per distinct recipe type.
    let quartz = &clauses["trade_ord_spd&formula[000]"];
    assert!(
        quartz
            .iter()
            .any(|c| matches!(c.kind, ClauseKind::SelfValue) && (c.value - 30.0).abs() < 0.01),
        "Quartz base +30: {quartz:?}"
    );
    assert!(
        quartz.iter().any(|c| matches!(
            &c.kind,
            ClauseKind::ScalingRoomCount { room } if room == "MANUFACTURE_RECIPE_TYPES"
        )),
        "Quartz scales per recipe type: {quartz:?}"
    );

    // Hoederer "Starting From Scratch β": +30 base + +5 when Ines/W works any Work Area.
    let hoederer = &clauses["trade_ord_par&per[001]"];
    assert!(
        hoederer
            .iter()
            .any(|c| matches!(c.kind, ClauseKind::SelfValue) && (c.value - 30.0).abs() < 0.01),
        "Hoederer base +30: {hoederer:?}"
    );
    assert!(
        hoederer.iter().any(|c| matches!(
            &c.kind,
            ClauseKind::RequiresChar { chars, scope: CondScope::BaseWorkArea } if chars.len() == 2
        )),
        "Hoederer's rider requires Ines/W in any Work Area: {hoederer:?}"
    );

    // Proviso "Damages for Breach": Pure-Gold order VALUE, not speed, kept as
    // its SHAPE (+2 gold on orders below the game's defaulted threshold of 4)
    // and priced per post level at scoring time.
    let proviso = &clauses["trade_ord_against[010]"];
    assert!(
        proviso
            .iter()
            .any(|c| c.metric == Metric::OrderValue { pure_gold: true }
                && c.kind
                    == ClauseKind::OrderMix(OrderEffect::DefaultedGoldBonus {
                        below: 4,
                        bonus: 2
                    })),
        "Proviso is a Pure-Gold defaulted-order shape: {proviso:?}"
    );

    // Greyy the Lightningbearer: drone skill scales on the base's drone capacity
    // with the stated +25% cap; the E2 skill raises the POWER facility count.
    let greyy_drone = &clauses["power_rec_drone[000]"];
    assert!(
        greyy_drone.iter().any(|c| matches!(
            &c.kind,
            ClauseKind::ScalingRoomCount { room } if room == "DRONE_CAPACITY"
        ) && c.cap == Some(25.0)),
        "Greyy Alter's drone skill scales on DRONE_CAPACITY capped at 25: {greyy_drone:?}"
    );
    let greyy_count = &clauses["power_count[000]"];
    assert!(
        greyy_count
            .iter()
            .any(|c| c.metric == Metric::FacilityCount("POWER".into())),
        "Greyy Alter's E2 raises the POWER facility count: {greyy_count:?}"
    );

    // Vermeil: capacity +8 (limit metric) and morale reduction on the same buff.
    let vermeil = &clauses["manu_prod_limit&cost[0000]"];
    assert!(
        vermeil
            .iter()
            .any(|c| c.metric == Metric::CapacityLimit && (c.value - 8.0).abs() < 0.01),
        "Vermeil's capacity +8 is a CapacityLimit clause: {vermeil:?}"
    );
}

/// NEVER-GUESS diagnostics (CP3): every buff the parser can't reduce to real
/// clauses contributes exactly ZERO and must be VISIBLE; this test prints the
/// full unresolved inventory (the work list for new parsers) and pins the
/// properties the principle demands: unresolved sets carry no value, and the
/// families the goldens prove parseable never regress into the list.
#[test]
fn unresolved_buffs_are_zero_and_inventoried() {
    use myrtle_engine_base::scoring::clause::{ClauseKind, build_clauses, unresolved_buffs};
    let gd = load_game_data();
    let clauses = build_clauses(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let unresolved = unresolved_buffs(&clauses);

    let mut by_room: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for id in &unresolved {
        let buff = &gd.building().buffs[*id];
        *by_room.entry(buff.room_type.as_str()).or_default() += 1;
        println!(
            "UNRESOLVED {:<12} {:<40} {}",
            buff.room_type, id, buff.buff_name
        );
        // Never guess: an unresolved buff's clause set carries no scoring
        // value. A morale drain may ride along; it comes from structured
        // side-map data, not from guessing at the productivity text.
        assert!(
            clauses[*id].iter().all(|c| match c.kind {
                ClauseKind::Unresolved => c.value == 0.0,
                _ => c.metric == myrtle_engine_base::scoring::clause::Metric::MoraleDrainDelta,
            }),
            "{id}: unresolved buffs must not carry value alongside the marker"
        );
    }
    println!(
        "unresolved: {} of {} buffs {:?}",
        unresolved.len(),
        clauses.len(),
        by_room
    );

    // Families the goldens prove parseable must never fall back to Unresolved.
    for known in [
        "trade_ord_spd&cost_P[000]",  // Texas
        "trade_ord_vodfox[000]",      // Shamare
        "manu_prod_spd&power[010]",   // Weedy
        "trade_ord_spd&formula[000]", // Quartz
        "trade_ord_par&per[001]",     // Hoederer
        "trade_ord_against[010]",     // Proviso
        "power_rec_drone[000]",       // Greyy Alter
        "manu_prod_limit&cost[0000]", // Vermeil
        // Deployment-context gates (ConditionalOnRoomPresence)
        "manu_formula_spd_P[000]",        // Gummy in a Trading Post
        "power_rec_spd_P[000]",           // Kal'tsit in the Control Center
        "power_rec_spd_P[001]",           // Logos as the Trainer
        "power_rec_spd_ext&faction[000]", // another Laterano op in a Power Plant
        "power_rec_spd_ext&tag[000]",     // GALLUS²: another Operation Platform in a Power Plant
        "manu_prod_spd_train&lv[000]",    // Viy: per Training Room level
        // Drain-only buffs: their sole effect (the morale drain) IS captured,
        // so the Unresolved marker would be label pessimism.
        "power_rec_spd&cost[000]",
        "power_rec_spd&cost[010]",
        // Pure side-channel grant (dorm-occupancy Passion gen, natively priced)
        "control_dorm_bd[000]",
        // Non-production CC skills (ControlNonProduction, own units, 0 LMD)
        "control_upMeetingSpeed[000]",
        "control_upMeetingSpeed[100]",
        "control_meeting_spd&bd[000]",
        "control_mp&meet_spd[000]",
        "control_train_spd[010]",
        "control_train_spd[011]",
        "control_train_spd[012]",
        "control_hire_spd&bd[000]",
    ] {
        assert!(
            !unresolved.contains(&known),
            "{known} regressed to Unresolved"
        );
    }
}

#[test]
#[ignore]
fn planner_probe() {
    use myrtle_engine_base::assignment::compute_optimal_assignment_with_pins;
    let gd = load_game_data();
    let dir = std::env::var("BASE_REPRO_DIR").expect("BASE_REPRO_DIR");
    let uid = std::env::var("BASE_REPRO_UID").expect("BASE_REPRO_UID");
    let read = |what: &str| -> serde_json::Value {
        let path = format!("{dir}/{what}_{uid}.json");
        serde_json::from_str(
            &std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}")),
        )
        .expect("valid json")
    };
    let building = UserBuilding::from_json(&read("building"));
    let roster: Vec<RosterEntry> = serde_json::from_value(read("roster")).expect("roster rows");
    let profiles: Vec<OperatorBaseProfile> = roster
        .iter()
        .filter_map(|entry| {
            let bc = gd.building().chars.get(&entry.operator_id)?;
            let static_op = gd.operators().get(&entry.operator_id);
            let faction_tags = static_op
                .map(|op| faction_tags_with_glossary(gd, &entry.operator_id, op))
                .unwrap_or_default();
            let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
            // IGNORE_PROMOTION=0 plans at real elite levels; default E2-max.
            let ignore_promo = std::env::var("IGNORE_PROMOTION").as_deref() != Ok("0");
            Some(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                rarity,
                gd.building(),
                ignore_promo,
            ))
        })
        // GLASGOW_OUT=1 drops the Glasgow Gang, to probe the eviction path.
        .filter(|p| {
            std::env::var("GLASGOW_OUT").as_deref() != Ok("1")
                || ![
                    "char_154_morgan",
                    "char_155_tiger",
                    "char_157_dagda",
                    "char_112_siege",
                ]
                .contains(&p.char_id.as_str())
        })
        .collect();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let plan = compute_optimal_assignment_with_pins(
        &profiles,
        &building,
        gd.building(),
        &registry,
        &drains,
        &[],
    );
    for r in &plan.rooms {
        if r.room_type == "CONTROL" || r.room_type == "TRADING" {
            println!(
                "{} {} eff {:.1} ops {:?}",
                r.slot_id, r.room_type, r.total_efficiency, r.operators
            );
        }
    }
    let delph_in_cc = plan
        .rooms
        .iter()
        .any(|r| r.room_type == "CONTROL" && r.operators.iter().any(|o| o == "char_4110_delphn"));
    let glasgow_in_tp = plan.rooms.iter().any(|r| {
        r.room_type == "TRADING"
            && r.operators.iter().any(|o| {
                [
                    "char_154_morgan",
                    "char_155_tiger",
                    "char_157_dagda",
                    "char_112_siege",
                ]
                .contains(&o.as_str())
            })
    });
    println!("PROBE delphine_in_cc={delph_in_cc} glasgow_in_tp={glasgow_in_tp}");

    // The rotation path, wired EXACTLY like base_planner::rotation: pins for
    // unmodeled rooms' drafted crews plus the morale-swap manager.
    let mut pins: Vec<(String, String)> = Vec::new();
    for room in &building.rooms {
        if ["TRAINING", "WORKSHOP"].contains(&room.room_type.as_str()) {
            for op in &room.current_operators {
                pins.push((op.clone(), room.room_type.clone()));
            }
        }
    }
    if let Some(pin) =
        myrtle_engine_base::rotation::dorms::morale_manager_pin(&profiles, &building, gd.building())
        && !pins.iter().any(|(id, _)| id == &pin.0)
    {
        pins.push(pin);
    }
    let rot = myrtle_engine_base::rotation::shift::recommend_shift_rotation(
        &profiles,
        &building,
        gd.building(),
        &registry,
        &drains,
        &pins,
    );
    for shift in &rot.shifts {
        for r in shift.rooms.iter().filter(|r| r.room_type == "CONTROL") {
            println!(
                "ROT-CC shift{} active={} crew {:?}",
                shift.index, r.active, r.recommended
            );
        }
    }
    for shift in &rot.shifts {
        for r in shift.rooms.iter().filter(|r| r.active) {
            if r.recommended.iter().any(|o| o == "char_4110_delphn") {
                println!(
                    "ROT shift{} {} {} crew {:?}",
                    shift.index, r.slot_id, r.room_type, r.recommended
                );
            }
            if r.room_type == "TRADING" {
                let glas: Vec<&String> = r
                    .recommended
                    .iter()
                    .filter(|o| {
                        [
                            "char_154_morgan",
                            "char_155_tiger",
                            "char_157_dagda",
                            "char_112_siege",
                        ]
                        .contains(&o.as_str())
                    })
                    .collect();
                if !glas.is_empty() {
                    println!(
                        "ROT shift{} {} TRADING glasgow {:?}",
                        shift.index, r.slot_id, glas
                    );
                }
            }
        }
    }
    println!("PROBE rotation bench {:?}", rot.bench);
    let targeted =
        myrtle_engine_base::registry::targeted_morale_effects(&gd.building().buffs, &name_to_char);
    let sim = myrtle_engine_base::rotation::sustain_sim::simulate_rotation(
        &rot,
        &profiles,
        &building,
        gd.building(),
        &registry,
        &drains,
        &targeted,
    );
    for f in &sim.facilities {
        println!(
            "SIM {} {:12} {:6} lmd {:8.0} gold {:6.1} exp {:8.0} idle {:5.1}h",
            f.slot_id,
            f.room_type,
            f.formula_type.as_deref().unwrap_or("-"),
            f.lmd,
            f.gold,
            f.exp,
            f.idle_hours
        );
    }
    println!("PROBE rotation done");
}

/// Account facts re-price player-state-gated skills the sync cannot read:
/// Lin's Meritocracy (+10% HR speed per purchased recruit slot) prices 0
/// undeclared (never-guess) and 10 x slots once declared. Flat-valued HIRE
/// skills stay untouched.
#[test]
fn account_facts_reprice_per_slot_hr_riders() {
    use myrtle_engine_base::registry::{BuffResolutionStrategy, resolve_account_facts};
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    let value =
        |reg: &std::collections::HashMap<String, BuffResolutionStrategy>, id: &str| match reg
            .get(id)
        {
            Some(BuffResolutionStrategy::NonProduction { value }) => *value,
            other => panic!("{id}: expected NonProduction, got {other:?}"),
        };
    assert_eq!(value(&registry, "hire_spd_cost&extra[000]"), 0.0);
    let resolved = resolve_account_facts(&registry, &gd.building().buffs, 3);
    assert_eq!(value(&resolved, "hire_spd_cost&extra[000]"), 30.0);
    // A flat HIRE skill with a non-priced slot rider is untouched.
    assert_eq!(
        value(&resolved, "hire_spd_cost&clue[000]"),
        value(&registry, "hire_spd_cost&clue[000]")
    );
}

#[test]
#[ignore]
fn facts_probe_registry_peek() {
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    for id in [
        "hire_spd_cost&extra[000]",
        "hire_spd_cost&clue[000]",
        "hire_spd&clue[100]",
        "hire_spd_bd_n1_n1[100]",
    ] {
        println!("{id} => {:?}", registry.get(id));
    }
}

#[test]
#[ignore]
fn unpriced_buff_audit() {
    use myrtle_engine_base::registry::BuffResolutionStrategy;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    let mut unpriced = 0;
    for (id, buff) in &gd.building().buffs {
        match registry.get(id) {
            Some(BuffResolutionStrategy::Complex { estimated_pct }) => {
                unpriced += 1;
                let desc: String = buff.description.chars().take(120).collect();
                println!(
                    "COMPLEX {id} [{}] est {estimated_pct}: {desc}",
                    buff.room_type
                );
            }
            None => {
                unpriced += 1;
                println!("MISSING {id} [{}]", buff.room_type);
            }
            _ => {}
        }
    }
    println!(
        "TOTAL unpriced: {unpriced} of {}",
        gd.building().buffs.len()
    );
}

/// Wiš'adel's Conspirator: named-operator CC gates. "If Hoederer is assigned
/// to a Trading Post, that Trading Post's order limit +2" lands on the post
/// seating him (+1 at the α tier); "if Ines is assigned to the Reception
/// Room, clue collection speed +5%" is priced in the reception's own units.
#[test]
fn wisadel_conspirator_named_char_grants() {
    use myrtle_engine_base::assignment::{cc_non_production_effects, compute_current_assignment};
    use myrtle_engine_base::registry::NamedCharGrant;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    const WISADEL: &str = "char_1035_wisdel";
    const HOEDERER: &str = "char_4088_hodrer";
    const INES: &str = "char_4087_ines";

    // Parse: both payload segments resolve, names to char ids.
    match registry.get("control_meeting&ord[001]") {
        Some(BuffResolutionStrategy::NamedCharRoomGrants { grants }) => {
            assert!(
                grants.contains(&NamedCharGrant {
                    char_id: INES.into(),
                    target_room: "MEETING".into(),
                    order_limit: 0.0,
                    nonprod_pct: 5.0,
                }),
                "Ines clue grant, got {grants:?}"
            );
            assert!(
                grants.contains(&NamedCharGrant {
                    char_id: HOEDERER.into(),
                    target_room: "TRADING".into(),
                    order_limit: 2.0,
                    nonprod_pct: 0.0,
                }),
                "Hoederer order-limit grant, got {grants:?}"
            );
        }
        other => panic!("expected NamedCharRoomGrants, got {other:?}"),
    }

    // Behavior: the post seating Hoederer gains +2 order limit while Wiš'adel
    // holds the CC, and only that post.
    let build = |cc_crew: Vec<String>| {
        let mut building = UserBuilding {
            drone_capacity: None,
            rooms: vec![
                room("cc", "CONTROL", 5),
                room("tp1", "TRADING", 3),
                room("tp2", "TRADING", 3),
            ],
        };
        building.rooms[0].current_operators = cc_crew.clone();
        building.rooms[1].current_operators = vec![HOEDERER.into()];
        building.rooms[2].current_operators = vec!["char_103_angel".into()];
        let mut ids: Vec<&str> = vec![HOEDERER, "char_103_angel"];
        ids.extend(cc_crew.iter().map(String::as_str));
        let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None)
    };
    let capacity = |asn: &myrtle_engine_base::types::BaseAssignment, slot: &str| {
        asn.rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .and_then(|r| r.fill.as_ref())
            .map(|f| f.capacity)
            .expect("trading fill")
    };
    let with = build(vec![WISADEL.into()]);
    let without = build(vec![]);
    assert_eq!(
        capacity(&with, "tp1") - capacity(&without, "tp1"),
        2,
        "Hoederer's post gains +2 order limit under Wiš'adel"
    );
    assert_eq!(
        capacity(&with, "tp2"),
        capacity(&without, "tp2"),
        "a post without Hoederer gains nothing"
    );

    // The clue payload fires only while Ines sits in a Reception Room.
    let wisadel = profile(gd, WISADEL);
    let ops = vec![wisadel];
    let meeting_room = |ops_in: Vec<String>| myrtle_engine_base::types::RoomAssignment {
        room_type: "MEETING".into(),
        operators: ops_in,
        ..Default::default()
    };
    let crew = vec![WISADEL.to_string()];
    assert_eq!(
        cc_non_production_effects(&crew, &ops, &registry, &[meeting_room(vec![INES.into()])]),
        [("MEETING".to_string(), 5.0)],
        "clue +5% while Ines holds the reception"
    );
    assert_eq!(
        cc_non_production_effects(&crew, &ops, &registry, &[meeting_room(vec![])]),
        [],
        "no Ines, no clue payload"
    );
}

/// The registry is a function of the tables alone: `BaseData::new` parses it
/// once and every context shares that parse instead of repeating it.
#[test]
fn base_data_parses_the_registry_once_and_contexts_share_it() {
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    assert!(!registry.is_empty());
    assert_eq!(**gd.registry(), registry);
    assert_eq!(**gd.morale_drains(), drains);
    assert_eq!(
        *gd.glossary(),
        myrtle_engine_base::registry::glossary_tags(gd.consts(), &name_to_char)
    );

    let ctx = myrtle_engine_base::BaseContext::build(&[], gd, false);
    assert!(std::sync::Arc::ptr_eq(&ctx.registry, gd.registry()));
    assert!(std::sync::Arc::ptr_eq(
        &ctx.morale_drains,
        gd.morale_drains()
    ));
}
