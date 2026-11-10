use super::*;

#[test]
fn wangs_expedience_counts_effective_power_plants_for_influence() {
    // Wang's Expedience branches on Influence (Trading Posts + Power Plants)
    // against Territory (Factories). Eunectes' "+2 Power Plants" (with
    // Lancet-2 in a plant) raises Influence in the game: a player with one
    // post, one plant and three factories read +7% on the post where we
    // gave the factories +2% (user report 2026-10-14). The branch now reads
    // the effective facility counts.
    use myrtle_engine_base::assignment::compute_current_assignment;
    const WANG: &str = "char_2027_wang";
    const EUNECTES: &str = "char_416_zumama";
    const LANCET: &str = "char_285_medic2";
    const GRAVEL: &str = "char_237_gravel";
    const LIVE_PLANT_OP: &str = "char_1011_lava2";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let base = |cc: &[&str], plant: &str| {
        let mut control = room("cc", "CONTROL", 5);
        control.current_operators = cc.iter().map(|s| (*s).to_string()).collect();
        let mut post = room("tp", "TRADING", 3);
        post.current_operators = vec![TEXAS.into()];
        let mut pp = room("pp", "POWER", 3);
        pp.current_operators = vec![plant.to_string()];
        let mut rooms = vec![control, post, pp];
        for i in 0..3 {
            let mut mf = room(&format!("mf{i}"), "MANUFACTURE", 3);
            mf.current_formula = Some("F_GOLD".into());
            if i == 0 {
                mf.current_operators = vec![GRAVEL.into()];
            }
            rooms.push(mf);
        }
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        let roster: Vec<_> = [WANG, EUNECTES, LANCET, GRAVEL, LIVE_PLANT_OP, TEXAS]
            .iter()
            .map(|id| profile(gd, id))
            .collect();
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        let eff = |slot: &str| {
            asn.rooms
                .iter()
                .find(|r| r.slot_id == slot)
                .map_or(0.0, |r| r.total_efficiency)
        };
        (eff("tp"), eff("mf0"))
    };
    let (post_plain, factory_plain) = base(&[WANG], LIVE_PLANT_OP);
    let (post_base, factory_base) = base(&[], LIVE_PLANT_OP);
    assert!(
        (factory_plain - factory_base - 2.0).abs() < 1e-6 && (post_plain - post_base).abs() < 1e-6,
        "two influence against three territory: the factories take +2 ({factory_base:.0} -> {factory_plain:.0}), the post nothing ({post_base:.0} -> {post_plain:.0})"
    );
    let (post_boosted, factory_boosted) = base(&[WANG, EUNECTES], LANCET);
    let (post_ref, factory_ref) = base(&[EUNECTES], LANCET);
    assert!(
        (post_boosted - post_ref - 7.0).abs() < 1e-6
            && (factory_boosted - factory_ref).abs() < 1e-6,
        "Eunectes' two plants tip Influence over Territory: the post takes +7 ({post_ref:.0} -> {post_boosted:.0}), the factories nothing ({factory_ref:.0} -> {factory_boosted:.0})"
    );
}

/// The Sui Control-Center economy feeds ONE shared Worldly Plight pool that
/// every reader sees whole (base expert, 2026-09-08): Mr. Nothing's 18 dorm
/// points + Ling 15 (above 12) + Dusk 15 (below 12) + Chongyue 25 (five Sui
/// deployed, capped) = 73. Jieyun's converter (5 Plight -> 1 Crystal) copies,
/// it never drains Shu's or Mr. Nothing's read. With no bars synced the
/// morale-conditional grants weigh their steady-state half (58).
#[test]
fn sui_worldly_plight_is_one_shared_pool_read_whole_by_every_consumer() {
    use myrtle_engine_base::assignment::compute_live_assignment;
    use std::collections::HashMap;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let mut rooms = vec![
        room("cc", "CONTROL", 5),
        room("tp", "TRADING", 2),
        room("mf", "MANUFACTURE", 2),
    ];
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let cc = [
        "char_2023_ling",
        "char_2024_chyue",
        "char_2015_dusk",
        "char_2014_nian",
        "char_4119_wanqin",
    ];
    let mut ids: Vec<&str> = cc.to_vec();
    ids.extend([
        "char_4032_provs",
        "char_455_nothin",
        "char_2025_shu",
        "char_4078_bdhkgt",
    ]);
    let fillers: Vec<String> = gd
        .building()
        .chars
        .keys()
        .filter(|id| id.starts_with("char_") && !ids.contains(&id.as_str()))
        .take(18)
        .cloned()
        .collect();
    ids.extend(fillers.iter().map(String::as_str));
    let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
    rooms[0].current_operators = cc.iter().map(|s| (*s).to_string()).collect();
    rooms[1].current_operators = vec!["char_4032_provs".into(), "char_455_nothin".into()];
    rooms[2].current_operators = vec!["char_2025_shu".into(), "char_4078_bdhkgt".into()];
    rooms[2].current_formula = Some("F_GOLD".into());
    for (i, id) in fillers.iter().enumerate() {
        rooms[3 + i / 5].current_operators.push(id.clone());
    }
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let eff = |live: HashMap<String, f64>| {
        let asn = compute_live_assignment(
            &roster,
            &building,
            gd.building(),
            &registry,
            &drains,
            None,
            &live,
        );
        let of = |slot: &str| {
            asn.rooms
                .iter()
                .find(|r| r.slot_id == slot)
                .map_or(0.0, |r| r.total_efficiency)
        };
        (of("tp"), of("mf"))
    };
    let (tp, mf) = eff(HashMap::from([
        ("char_2023_ling".to_string(), 20.0),
        ("char_2015_dusk".to_string(), 8.0),
    ]));
    assert!(
        (tp - 73.0).abs() < 1e-9,
        "Mr. Nothing reads the whole pool: {tp}"
    );
    // Shu floor(73/3) = 24, Jieyun 2 x floor(73/5) = 28.
    assert!(
        (mf - 52.0).abs() < 1e-9,
        "Shu and Jieyun read the same whole pool: {mf}"
    );
    let (tp_half, _) = eff(HashMap::new());
    assert!(
        (tp_half - 58.0).abs() < 1e-9,
        "no bars: Ling and Dusk at half weight: {tp_half}"
    );
}

/// Pool-scaled Control-Center globals (Sakiko's "all Trading Posts +1% per 8
/// Passion", the Mortis "all Factories +1% (+1% per 20)") parse into
/// `GlobalPoolScaling` and resolve by registry rewrite against settled pool
/// points (floored steps, base always on), exactly like the other
/// deployment-dependent rewrites. Context-free scoring credits the base only.
#[test]
// Exact float equality on purpose: the values come straight from the constants
// the resolver used, not an independent computation.
#[allow(clippy::float_cmp)]
fn global_pool_consumers_resolve_against_settled_points() {
    use myrtle_engine_base::assignment::resolve_global_pool;
    use myrtle_engine_base::registry::{BuffResolutionStrategy, build_registry};
    use std::collections::HashMap;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);

    for (id, want_room, want_base, want_pct, want_per) in [
        ("control_mp_bd&trade[000]", "TRADING", 0.0, 1.0, 8.0),
        ("control_prod_bd_spd[010]", "MANUFACTURE", 1.0, 1.0, 20.0),
        ("control_prod_bd_spd[000]", "MANUFACTURE", 0.5, 0.5, 20.0),
    ] {
        match registry.get(id) {
            Some(BuffResolutionStrategy::GlobalPoolScaling {
                target_room,
                base_pct,
                pct,
                per,
                resource,
            }) => {
                assert_eq!(target_room, want_room, "{id}");
                assert_eq!(*base_pct, want_base, "{id}");
                assert_eq!(*pct, want_pct, "{id}");
                assert_eq!(*per, want_per, "{id}");
                assert_eq!(resource, "bd_mujica", "{id}");
            }
            other => panic!("{id}: expected GlobalPoolScaling, got {other:?}"),
        }
    }

    // 50 Passion: trading global = floor(50/8) x 1 = 6; factory = 1 + floor(50/20) x 1 = 3.
    let points: HashMap<String, f64> = [("bd_mujica".to_string(), 50.0)].into();
    let resolved = resolve_global_pool(&registry, &points);
    assert_eq!(
        resolved.get("control_mp_bd&trade[000]"),
        Some(&BuffResolutionStrategy::GlobalEffect {
            target_room: "TRADING".to_string(),
            bonus_pct: 6.0,
        })
    );
    assert_eq!(
        resolved.get("control_prod_bd_spd[010]"),
        Some(&BuffResolutionStrategy::GlobalEffect {
            target_room: "MANUFACTURE".to_string(),
            bonus_pct: 3.0,
        })
    );
    // No points: the base survives, the pool part is 0 (never guess).
    let dry = resolve_global_pool(&registry, &std::collections::HashMap::new());
    assert_eq!(
        dry.get("control_prod_bd_spd[010]"),
        Some(&BuffResolutionStrategy::GlobalEffect {
            target_room: "MANUFACTURE".to_string(),
            bonus_pct: 1.0,
        })
    );
}

/// The robot power-plant economy is a BUNDLE: Alanna's Operation Platforms only
/// pay when Robot-tagged operators hold Power Plant seats, so the generator
/// pins the roster's robots (capped at the building's real power capacity) and
/// prices her factory skill at the solved payoff. The improvements-level oracle
/// then weighs that feed against the drone specialists the pins displace,
/// a trade that only became priceable once POWER entered `assignment_value`.
#[test]
// Exact float equality is intentional: `10.0` is the literal per-platform bonus
// the bundle resolver is expected to price, not an independent computation.
#[allow(clippy::float_cmp)]
fn alanna_bundle_pins_robots_into_power_seats() {
    use myrtle_engine_base::scoring::pools::candidate_bundles;
    const ALANNA: &str = "char_4178_alanna";
    const CASTLE3: &str = "char_286_cast3";
    const LANCET2: &str = "char_285_medic2";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    let ops: Vec<OperatorBaseProfile> = [ALANNA, CASTLE3, LANCET2]
        .iter()
        .map(|id| profile(gd, id))
        .collect();

    // One L3 plant = one seat: the bundle must cap its pins there even with
    // two robots in the roster.
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("p0", "POWER", 3), room("mf0", "MANUFACTURE", 3)],
    };
    let bundles = candidate_bundles(&ops, &building, gd.building(), &registry);
    let robot_bundle = bundles
        .iter()
        .find(|b| b.pins.iter().any(|(_, rt)| rt == "POWER"))
        .expect("a robot bundle for Alanna");
    assert_eq!(
        robot_bundle.pins.len(),
        1,
        "one power seat -> one pinned robot: {:?}",
        robot_bundle.pins
    );
    let (buff_id, pct) = robot_bundle.overrides.first().expect("consumer override");
    assert!(
        buff_id.starts_with("manu_token_prod_spd"),
        "override targets her platform skill, got {buff_id}"
    );
    assert_eq!(*pct, 10.0, "one robot x +10% per platform at E2");

    // No power plant built -> no robot bundle to offer.
    let no_power = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("mf0", "MANUFACTURE", 3)],
    };
    assert!(
        candidate_bundles(&ops, &no_power, gd.building(), &registry)
            .iter()
            .all(|b| b.pins.iter().all(|(_, rt)| rt != "POWER")),
        "no plants means no robot pins"
    );
}

/// The first NATIVE pool economy: Minimalist's Engineering Robots. Her E0
/// generator makes +1 robot per functional-facility level (max 64, exactly a
/// maxed 243's level sum) and her E2 consumer converts every 8 robots into +5%
/// productivity, floored. Generator and consumer travel together, so the pool
/// settles room-locally from the layout alone: on a maxed base she is a
/// +40% solo factory operator; on a smaller base the value floors down.
#[test]
fn minimalist_engineering_robots_settle_from_the_layout() {
    use myrtle_engine_base::scoring::clause::{ClauseKind, build_clauses};
    const MINIMALIST: &str = "char_4054_malist";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    // The parse: generator + stepped consumer, no longer Unresolved.
    let clauses = build_clauses(&gd.building().buffs, &name_to_char);
    assert!(
        clauses["manu_constrLv[000]"]
            .iter()
            .any(|c| matches!(&c.kind, ClauseKind::ResourceConvert(_)) && c.cap == Some(64.0)),
        "generator parses with its 64-point cap: {:?}",
        clauses["manu_constrLv[000]"]
    );
    assert!(
        clauses["manu_prod_spd_bd[110]"]
            .iter()
            .any(|c| matches!(&c.kind, ClauseKind::ScalingPoolPoints { step, .. } if (*step - 8.0).abs() < 1e-9)),
        "E2 consumer parses as a stepped pool drain: {:?}",
        clauses["manu_prod_spd_bd[110]"]
    );

    let ops = vec![profile(gd, MINIMALIST)];
    let team = vec![MINIMALIST.to_string()];
    let value_with = |rooms: Vec<UserRoom>| {
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        team_value(
            &team,
            "MANUFACTURE",
            Some("F_GOLD"),
            &ops,
            &building,
            gd.building(),
            &registry,
            &drains,
        )
    };

    // Maxed 243: levels sum to 64 -> floor(64/8) x 5% = +40%.
    let mut maxed = vec![
        room("cc", "CONTROL", 5),
        room("hr", "HIRE", 3),
        room("rc", "MEETING", 3),
        room("ws", "WORKSHOP", 3),
        room("tr", "TRAINING", 3),
    ];
    maxed.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    maxed.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    maxed.extend((0..3).map(|i| room(&format!("p{i}"), "POWER", 3)));
    maxed.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    assert!(
        (value_with(maxed) - 1.40).abs() < 1e-9,
        "maxed base: 64 robots -> +40%"
    );

    // A half-built base: 30 levels -> floor(30/8)=3 -> +15%.
    let small = vec![
        room("cc", "CONTROL", 3),
        room("mf0", "MANUFACTURE", 3),
        room("mf1", "MANUFACTURE", 3),
        room("tp0", "TRADING", 3),
        room("p0", "POWER", 3),
        room("d0", "DORMITORY", 5),
        room("d1", "DORMITORY", 5),
        room("hr", "HIRE", 2),
        room("rc", "MEETING", 3),
    ];
    assert!(
        (value_with(small) - 1.15).abs() < 1e-9,
        "30 functional levels -> floor(30/8)x5 = +15%"
    );
}

/// The first ASSIGNMENT-FED pool: Senshi's Monster Meals. Seated in a dormitory
/// he provides 1 Meal per level of THAT dorm; Marcille (factory, +1%/Meal) and
/// Chilchuck (trading, +1%/Meal) consume the settled points. The live view
/// settles against real seats: with Senshi resting in an L5 dorm both consumers
/// gain exactly +5% over the identical base without him, and with him absent
/// the pool reads zero, never a guess.
#[test]
fn senshi_monster_meals_settle_against_live_seats() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    const SENSHI: &str = "char_4143_sensi";
    const MARCILLE: &str = "char_4141_marcil";
    const CHILCHUCK: &str = "char_4144_chilc";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let ops: Vec<OperatorBaseProfile> = [SENSHI, MARCILLE, CHILCHUCK]
        .iter()
        .map(|id| profile(gd, id))
        .collect();

    let build = |senshi_rests: bool| {
        let mut dorm = room("d0", "DORMITORY", 5);
        if senshi_rests {
            dorm.current_operators = vec![SENSHI.to_string()];
        }
        let mut mf = room("mf0", "MANUFACTURE", 3);
        mf.current_formula = Some("F_GOLD".to_string());
        mf.current_operators = vec![MARCILLE.to_string()];
        let mut tp = room("tp0", "TRADING", 3);
        tp.current_operators = vec![CHILCHUCK.to_string()];
        UserBuilding {
            drone_capacity: None,
            rooms: vec![room("cc", "CONTROL", 5), dorm, mf, tp],
        }
    };

    let eff = |asn: &myrtle_engine_base::types::BaseAssignment, slot: &str| {
        asn.rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .map_or_else(|| panic!("missing {slot}"), |r| r.total_efficiency)
    };
    let with =
        compute_current_assignment(&ops, &build(true), gd.building(), &registry, &drains, None);
    let without =
        compute_current_assignment(&ops, &build(false), gd.building(), &registry, &drains, None);

    assert!(
        (eff(&with, "mf0") - eff(&without, "mf0") - 5.0).abs() < 1e-9,
        "L5 dorm -> 5 Meals -> Marcille +5%: {} vs {}",
        eff(&with, "mf0"),
        eff(&without, "mf0")
    );
    assert!(
        (eff(&with, "tp0") - eff(&without, "tp0") - 5.0).abs() < 1e-9,
        "Chilchuck's post gains the same 5 Meals: {} vs {}",
        eff(&with, "tp0"),
        eff(&without, "tp0")
    );
}

/// The Control-Center generator SIDE-CHANNEL: Dusk/Ling/Chongyue's CC buffs
/// carry a morale aura (which the parser owns) AND a pool grant, extracted
/// separately by the settlement. Seating the three siblings in the CC with Shu
/// deployed makes 4 deployed Sui: Chongyue grants min(4,5)x5 = 20 Worldly
/// Plight, Dusk and Ling's "above/below 12 morale" branches settle at the 0.5
/// steady-state fraction (+7.5 each): 35 points, and Shu's factory skill
/// drains them at +1% per 3: exactly +11%.
#[test]
fn sui_worldly_plight_flows_from_the_control_center() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    const CHONGYUE: &str = "char_2024_chyue";
    const DUSK: &str = "char_2015_dusk";
    const LING: &str = "char_2023_ling";
    const SHU: &str = "char_2025_shu";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let ops: Vec<OperatorBaseProfile> = [CHONGYUE, DUSK, LING, SHU]
        .iter()
        .map(|id| profile(gd, id))
        .collect();

    let assemble = |cc_ops: &[&str]| {
        let mut cc = room("cc", "CONTROL", 5);
        cc.current_operators = cc_ops.iter().map(|s| (*s).to_string()).collect();
        let mut mf = room("mf0", "MANUFACTURE", 3);
        mf.current_formula = Some("F_GOLD".to_string());
        mf.current_operators = vec![SHU.to_string()];
        UserBuilding {
            drone_capacity: None,
            rooms: vec![cc, mf, room("d0", "DORMITORY", 5)],
        }
    };
    let factory_eff = |cc_ops: &[&str]| {
        let asn = compute_current_assignment(
            &ops,
            &assemble(cc_ops),
            gd.building(),
            &registry,
            &drains,
            None,
        );
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "mf0")
            .map(|r| r.total_efficiency)
            .unwrap()
    };

    let with_cc = factory_eff(&[CHONGYUE, DUSK, LING]);
    let alone = factory_eff(&[]);
    assert!(
        (with_cc - alone - 11.0).abs() < 1e-9,
        "35 Worldly Plight -> floor(35/3) = +11% on Shu's factory ({with_cc} vs {alone})"
    );
}

/// Alanna's Operation Platforms: a Robot-tagged operator seated in a Power
/// Plant is her pseudo-pool. Castle-3 manning a plant is worth +10% Precious
/// Metal productivity on her gold factory (E2); pull the robot and it's zero.
#[test]
fn alanna_counts_robots_in_power_plants() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    const ALANNA: &str = "char_4178_alanna";
    const CASTLE3: &str = "char_286_cast3";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let ops: Vec<OperatorBaseProfile> =
        [ALANNA, CASTLE3].iter().map(|id| profile(gd, id)).collect();

    let assemble = |robot_manned: bool| {
        let mut pp = room("p0", "POWER", 3);
        if robot_manned {
            pp.current_operators = vec![CASTLE3.to_string()];
        }
        let mut mf = room("mf0", "MANUFACTURE", 3);
        mf.current_formula = Some("F_GOLD".to_string());
        mf.current_operators = vec![ALANNA.to_string()];
        UserBuilding {
            drone_capacity: None,
            rooms: vec![room("cc", "CONTROL", 5), pp, mf],
        }
    };
    let factory_eff = |robot: bool| {
        let asn = compute_current_assignment(
            &ops,
            &assemble(robot),
            gd.building(),
            &registry,
            &drains,
            None,
        );
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "mf0")
            .map(|r| r.total_efficiency)
            .unwrap()
    };
    assert!(
        (factory_eff(true) - factory_eff(false) - 10.0).abs() < 1e-9,
        "one Operation Platform -> +10% at E2 ({} vs {})",
        factory_eff(true),
        factory_eff(false)
    );
}

/// SEAT INCENTIVES: the optimal search itself now values the pool economies.
/// The planner solves Senshi's Monster Meals ahead of the search, pinning him
/// into the best dormitory and pricing Marcille's consumer at the settled
/// +5%, so the optimizer SEATS Marcille in the factory over a plain +2%
/// operator it would otherwise prefer, and reserves Senshi's dorm seat. The
/// honesty rule holds throughout: consumers fed by unpinned third parties get
/// no credit.
#[test]
fn optimal_search_seats_the_monster_meal_economy() {
    use myrtle_engine_base::assignment::compute_optimal_assignment_with_pins;
    use myrtle_engine_base::registry::BuffResolutionStrategy;
    use myrtle_engine_base::scoring::pools::plan_optimal_economies;
    const SENSHI: &str = "char_4143_sensi";
    const MARCILLE: &str = "char_4141_marcil";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let ops: Vec<OperatorBaseProfile> = [SENSHI, MARCILLE]
        .iter()
        .map(|id| profile(gd, id))
        .collect();

    let mut mf = room("mf0", "MANUFACTURE", 3);
    mf.current_formula = Some("F_GOLD".to_string());
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("cc", "CONTROL", 5), mf, room("d0", "DORMITORY", 5)],
    };

    let plan = plan_optimal_economies(&ops, &building, gd.building(), &registry);
    assert!(
        plan.pins
            .iter()
            .any(|(id, room)| id == SENSHI && room == "DORMITORY"),
        "Senshi is pinned into the dormitory: {:?}",
        plan.pins
    );
    let marcille_override = plan
        .overrides
        .iter()
        .find(|(bid, _)| bid == "manu_prod_spd_bd[400]")
        .map(|(_, pct)| *pct);
    assert_eq!(
        marcille_override,
        Some(5.0),
        "Marcille's consumer is priced at the L5 dorm's 5 Meals: {:?}",
        plan.overrides
    );

    // Apply the plan the way the improvements service does and let the
    // optimizer search: Marcille must be seated with her solved value.
    let mut optimal_registry = registry;
    for (bid, pct) in &plan.overrides {
        optimal_registry.insert(
            bid.clone(),
            BuffResolutionStrategy::PoolPayoff { pct: *pct },
        );
    }
    let asn = compute_optimal_assignment_with_pins(
        &ops,
        &building,
        gd.building(),
        &optimal_registry,
        &drains,
        &plan.pins,
    );
    let factory = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "MANUFACTURE")
        .unwrap();
    assert!(
        factory.operators.iter().any(|o| o == MARCILLE),
        "the search seats Marcille for her solved economy value: {:?}",
        factory.operators
    );
    assert!(
        factory.total_efficiency >= 5.0 - 1e-9,
        "her settled +5% is realized in the optimal view: {}",
        factory.total_efficiency
    );
    let dorm_pin_realized = asn
        .rooms
        .iter()
        .any(|r| r.room_type == "DORMITORY" && r.operators.iter().any(|o| o == SENSHI));
    assert!(
        dorm_pin_realized,
        "Senshi's reserved dorm seat appears in the plan: {:?}",
        asn.rooms
            .iter()
            .map(|r| (&r.room_type, &r.operators))
            .collect::<Vec<_>>()
    );
}

/// JOINT-SEATING: the Sui bundle prices the whole Control-Center economy as a
/// package. With Chongyue, Dusk and Ling pinned into the CC (3 Sui kin ->
/// Chongyue grants 15, the conditional branches add 7.5 each = 30 Worldly
/// Plight), Shu's factory skill solves to floor(30/3) = +10%, and the
/// witch's self-owned converter chain reaches floor(30/5) = 6 Crystals ->
/// +12% at E2. The caller then judges the bundle by total yield; the
/// optimizer itself prices the three CC seats it costs.
#[test]
fn sui_bundle_prices_the_cc_economy_as_a_package() {
    use myrtle_engine_base::scoring::pools::candidate_bundles;
    const CHONGYUE: &str = "char_2024_chyue";
    const DUSK: &str = "char_2015_dusk";
    const LING: &str = "char_2023_ling";
    const SHU: &str = "char_2025_shu";
    const WITCH: &str = "char_4078_bdhkgt";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _drains) = build_registry(&gd.building().buffs, &name_to_char);
    let ops: Vec<OperatorBaseProfile> = [CHONGYUE, DUSK, LING, SHU, WITCH]
        .iter()
        .map(|id| profile(gd, id))
        .collect();

    let bundles = candidate_bundles(
        &ops,
        &UserBuilding {
            drone_capacity: None,
            rooms: Vec::new(),
        },
        gd.building(),
        &registry,
    );
    assert_eq!(bundles.len(), 1, "one Sui bundle: {bundles:?}");
    let b = &bundles[0];
    for id in [CHONGYUE, DUSK, LING] {
        assert!(
            b.pins.iter().any(|(c, r)| c == id && r == "CONTROL"),
            "{id} pinned to the CC: {:?}",
            b.pins
        );
    }
    let get = |bid: &str| b.overrides.iter().find(|(x, _)| x == bid).map(|(_, p)| *p);
    assert_eq!(
        get("manu_prod_spd_bd[300]"),
        Some(10.0),
        "Shu: 30 WP -> +10%: {:?}",
        b.overrides
    );
    assert_eq!(
        get("manu_prod_spd_bd[201]"),
        Some(12.0),
        "witch chain: 6 Crystals -> +12%: {:?}",
        b.overrides
    );
}

/// Wang's Expedience branches on LAYOUT counts the glossary defines
/// (Influence = Trading Posts + Power Plants, Territory = Factories): a
/// 2/5/3 base ties 5 >= 5 and takes the trading branch (+7% posts); a 2/5/2
/// base has Territory ahead and takes the factory branch (+2%). Either way
/// he is "strongest effect of the same type" with Amiya's post global and
/// Kal'tsit's factory global, so beside both he is dead weight.
#[test]
fn wang_branches_on_layout_counts_and_is_covered_by_amiya_and_kaltsit() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    use myrtle_engine_base::scoring::skill_ledger::LineDisposition;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    const WANG: &str = "char_2027_wang";
    const WANG_BUFF: &str = "control_prod_tra_spd[000]";
    assert_eq!(
        gd.building()
            .layout_terms
            .get("bd_wang_1")
            .map(Vec::as_slice),
        Some(&["TRADING".to_string(), "POWER".to_string()][..]),
        "Influence counts posts and plants"
    );
    assert_eq!(
        gd.building()
            .layout_terms
            .get("bd_wang_2")
            .map(Vec::as_slice),
        Some(&["MANUFACTURE".to_string()][..]),
        "Territory counts factories"
    );
    assert!(
        matches!(
            registry.get(WANG_BUFF),
            Some(BuffResolutionStrategy::LayoutCountBranch { ge_room, ge_pct, gt_room, gt_pct, .. })
                if ge_room == "TRADING" && (*ge_pct - 7.0).abs() < 1e-9
                    && gt_room == "MANUFACTURE" && (*gt_pct - 2.0).abs() < 1e-9
        ),
        "Wang parses as a layout branch: {:?}",
        registry.get(WANG_BUFF)
    );

    let build = |posts: usize, factories: usize, plants: usize, cc: &[&str]| {
        let mut rooms = vec![room("cc", "CONTROL", 5)];
        rooms.extend((0..posts).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
        rooms.extend((0..factories).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
        rooms.extend((0..plants).map(|i| room(&format!("p{i}"), "POWER", 3)));
        let mut building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        building.rooms[0].current_operators = cc.iter().map(|s| (*s).to_string()).collect();
        // One body in the first post and factory so both rooms score.
        building.rooms[1].current_operators = vec!["char_103_angel".into()];
        building.rooms[1 + posts].current_operators = vec!["char_285_medic2".into()];
        building.rooms[1 + posts].current_formula = Some("F_EXP".into());
        let mut ids: Vec<&str> = vec!["char_103_angel", "char_285_medic2"];
        ids.extend(cc);
        let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None)
    };
    let cc_line = |asn: &myrtle_engine_base::types::BaseAssignment, slot: &str| {
        asn.rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .and_then(|r| {
                r.ledger
                    .iter()
                    .find(|l| l.buff_id == WANG_BUFF && l.from_control_center)
            })
            .map(|l| (l.speed_pct, l.disposition))
    };

    // 2/5/3: Influence 5 >= Territory 5 -> the posts get +7, factories nothing.
    let a = build(2, 5, 3, &[WANG]);
    assert!(
        matches!(cc_line(&a, "tp0"), Some((v, _)) if (v - 7.0).abs() < 1e-9),
        "2/5/3 posts: {:?}",
        cc_line(&a, "tp0")
    );
    assert!(
        cc_line(&a, "mf0").is_none_or(|(v, _)| v.abs() < 1e-9),
        "2/5/3 factories: {:?}",
        cc_line(&a, "mf0")
    );
    // 2/5/2: Territory 5 > Influence 4 -> the factories get +2, posts nothing.
    let b = build(2, 5, 2, &[WANG]);
    assert!(
        matches!(cc_line(&b, "mf0"), Some((v, _)) if (v - 2.0).abs() < 1e-9),
        "2/5/2 factories: {:?}",
        cc_line(&b, "mf0")
    );
    assert!(
        cc_line(&b, "tp0").is_none_or(|(v, _)| v.abs() < 1e-9),
        "2/5/2 posts: {:?}",
        cc_line(&b, "tp0")
    );
    // Beside Amiya (+7% posts) the trading branch is a covered duplicate.
    let c = build(2, 5, 3, &["char_002_amiya", WANG]);
    let cc_row = c
        .rooms
        .iter()
        .find(|r| r.room_type == "CONTROL")
        .expect("cc row");
    let wang_row = cc_row
        .ledger
        .iter()
        .find(|l| l.buff_id == WANG_BUFF)
        .expect("wang line");
    assert!(
        wang_row.speed_pct.abs() < 1e-9 && wang_row.disposition == LineDisposition::Covered,
        "Wang beside Amiya is covered: {:?}",
        (wang_row.speed_pct, &wang_row.disposition)
    );
}

/// Pudding's Overclock is gated on the DEPLOYMENT: "2 or more Operation
/// Platforms assigned to Power Plants": with two Robot-tagged operators in
/// the plants the factories get +2% (Kal'tsit's family, non-stacking); with
/// one robot the skill is inactive, not a flat +2.
#[test]
fn pudding_overclock_needs_two_robots_in_power_plants() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    const PUDDING: &str = "char_4004_pudd";
    const PUDDING_BUFF: &str = "control_token_prod_spd[000]";
    assert!(
        matches!(
            registry.get(PUDDING_BUFF),
            Some(BuffResolutionStrategy::RoomPresenceGatedGlobal { required_faction, required_count: 2, room_type, target_room, bonus_pct })
                if required_faction == "robot" && room_type == "POWER" && target_room == "MANUFACTURE" && (*bonus_pct - 2.0).abs() < 1e-9
        ),
        "Overclock parses as a deployment-gated global: {:?}",
        registry.get(PUDDING_BUFF)
    );
    let build = |plants: &[&str]| {
        let mut building = UserBuilding {
            drone_capacity: None,
            rooms: vec![
                room("cc", "CONTROL", 5),
                room("mf", "MANUFACTURE", 3),
                room("p0", "POWER", 3),
                room("p1", "POWER", 3),
            ],
        };
        building.rooms[0].current_operators = vec![PUDDING.into()];
        building.rooms[1].current_operators = vec!["char_103_angel".into()];
        building.rooms[1].current_formula = Some("F_EXP".into());
        for (i, id) in plants.iter().enumerate() {
            building.rooms[2 + i].current_operators = vec![(*id).to_string()];
        }
        let mut ids: Vec<&str> = vec![PUDDING, "char_103_angel"];
        ids.extend(plants);
        let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "mf")
            .and_then(|r| {
                r.ledger
                    .iter()
                    .find(|l| l.buff_id == PUDDING_BUFF && l.from_control_center)
                    .map(|l| l.speed_pct)
            })
            .unwrap_or(0.0)
    };
    // Lancet-2 and Castle-3 are Robot-tagged Operation Platforms.
    assert!(
        (build(&["char_285_medic2", "char_286_cast3"]) - 2.0).abs() < 1e-9,
        "two robots fire the gate"
    );
    assert!(
        build(&["char_285_medic2"]).abs() < 1e-9,
        "one robot does not"
    );
}

/// Dusk's Control-Center rider ("when self morale is above 12, Perception
/// Information +10") feeds the SAME pool Rosmontis' dorm count fills; the
/// base expert (2026-09-08): Dusk, Iris, Czerny and Whisperain all stack into
/// it from their own resources, no split, no cap; the community's high-end
/// sheet reads Perception Info 34 = 20 dorm + Dusk 10 + Iris 2 + Czerny 2.
/// The optimizer offers that seating as a shared-pool bundle pinning Dusk
/// into the Control Center, priced at her steady-state grant (10 x the half
/// of her bar spent above 12) on top of Rosmontis' own 20.
#[test]
fn dusk_feeds_rosmontis_pool_through_a_control_center_pin() {
    use myrtle_engine_base::scoring::pools::{candidate_bundles, plan_optimal_economies};
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    const ROSMONTIS: &str = "char_391_rosmon";
    const DUSK: &str = "char_2015_dusk";
    const MANIFEST: &str = "manu_prod_spd_bd[010]";
    let mut rooms = vec![room("mf", "MANUFACTURE", 3), room("cc", "CONTROL", 5)];
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let mut ids: Vec<&str> = vec![ROSMONTIS, DUSK];
    ids.extend(
        gd.building()
            .chars
            .keys()
            .filter(|id| id.starts_with("char_") && *id != ROSMONTIS && *id != DUSK)
            .take(40)
            .map(String::as_str),
    );
    let profiles: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();

    // The native plan never forces Dusk's seat: Rosmontis alone is her 20.
    let native = plan_optimal_economies(&profiles, &building, gd.building(), &registry);
    let own = native
        .overrides
        .iter()
        .find(|(b, _)| b == MANIFEST)
        .map(|(_, v)| *v);
    assert_eq!(own, Some(20.0), "own origins only: {:?}", native.overrides);
    assert!(
        !native.pins.iter().any(|(id, _)| id == DUSK),
        "Dusk is a bundle, not a forced pin"
    );

    let bundles = candidate_bundles(&profiles, &building, gd.building(), &registry);
    let shared = bundles
        .iter()
        .find(|b| {
            b.pins.iter().any(|(id, r)| id == DUSK && r == "CONTROL")
                && b.overrides
                    .iter()
                    .any(|(b, v)| b == MANIFEST && (*v - 25.0).abs() < 1e-9)
        })
        .unwrap_or_else(|| {
            panic!(
                "a bundle pinning Dusk into the Control Center at 20 + 5: {:?}",
                bundles
                    .iter()
                    .map(|b| (&b.pins, &b.overrides))
                    .collect::<Vec<_>>()
            )
        });
    assert_eq!(shared.pins.len(), 1, "Dusk alone: {:?}", shared.pins);
}

/// Pins are seats: the Mujica five (a Passion bundle) plus Dusk and Ling (a
/// perception bundle) are seven Control-Center pins for a five-seat room. A
/// bundle that cannot be seated is not tried, so the optimal never overfills
/// the Control Center (it used to propose seven, which the planner's draft
/// validation then rejected).
#[test]
fn accepted_bundles_never_overfill_the_control_center() {
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let mut roster: Vec<_> = [
        "char_2015_dusk",
        "char_2023_ling",
        "char_391_rosmon",
        "char_4186_tmoris",
        "char_4185_amoris",
        "char_4184_dolris",
        "char_4183_mortis",
        "char_4182_oblvns",
        "char_400_weedy",
        "char_416_zumama",
        "char_472_pasngr",
        "char_190_clour",
        "char_381_bubble",
        "char_485_pallas",
        "char_4032_provs",
        "char_332_archet",
        "char_427_vigil",
        "char_123_fang",
        "char_133_mm",
        "char_502_nblade",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    roster.dedup_by(|a, b| a.char_id == b.char_id);
    let mut rooms = vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)];
    rooms.extend((0..3).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..2).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
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
    assert!(
        cc.operators.len() <= 5,
        "a level-5 Control Center seats 5, the plan seats {}: {:?}",
        cc.operators.len(),
        cc.operators
    );
    let cc_pins = accepted
        .pins
        .iter()
        .filter(|(_, rt)| rt == "CONTROL")
        .count();
    assert!(
        cc_pins <= 5,
        "accepted Control-Center pins {cc_pins} exceed the seats: {:?}",
        accepted.pins
    );
}

/// The base-count bundle offers the oracle the seats a player locks by hand:
/// Pozëmka's Durins pinned into the dormitories.
#[test]
fn base_count_bundle_pins_durins_into_the_dormitories() {
    use myrtle_engine_base::scoring::pools::candidate_bundles;
    let gd = load_game_data();
    let (registry, _) = build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let ids = [
        "char_4055_bgsnow",
        "char_4164_tecno",
        "char_4054_malist",
        "char_501_durin",
    ];
    let roster: Vec<_> = ids.iter().map(|id| profile(gd, id)).collect();
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 5)],
    };
    let bundles = candidate_bundles(&roster, &building, gd.building(), &registry);
    let durin_bundle = bundles.iter().find(|b| {
        b.pins
            .iter()
            .any(|(id, r)| id == "char_4164_tecno" && r == "DORMITORY")
    });
    let pins = durin_bundle.map_or(0, |b| b.pins.len());
    assert_eq!(
        pins,
        3,
        "three Durins parked for the count: {:?}",
        bundles.iter().map(|b| &b.pins).collect::<Vec<_>>()
    );
}

/// Dorm-occupant pools settle against live seats end to end:
/// - Mr. Nothing's one-buff trading economy: each resting operator grants a
///   Worldly Plight point and his own skill converts them to order efficiency.
/// - Rosmontis' Extrasensory CHAIN: dorm occupants feed Perception
///   Information, which converts 1:1 into Chain of Thought, which her E2
///   consumer drains at +1% per point: two pools, one fixed-point pass.
/// - A CROSS-OPERATOR chain: Mr. Nothing generates Worldly Plight from the
///   dorms while a factory teammate's converter turns every 5 into a
///   Witchcraft Crystal worth +2%: generation, conversion and consumption in
///   three different rooms.
#[test]
fn dorm_fed_pools_and_conversion_chains_settle() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    const MR_NOTHING: &str = "char_455_nothin";
    const ROSMONTIS: &str = "char_391_rosmon";
    const WITCH: &str = "char_4078_bdhkgt";
    const BODIES: [&str; 4] = [
        "char_002_amiya",
        "char_123_fang",
        "char_240_wyvern",
        "char_192_falco",
    ];
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let mut ids: Vec<&str> = vec![MR_NOTHING, ROSMONTIS, WITCH];
    ids.extend(BODIES);
    let ops: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();

    let assemble = |factory_op: &str, resting: &[&str]| {
        let mut dorm = room("d0", "DORMITORY", 5);
        dorm.current_operators = resting.iter().map(|s| (*s).to_string()).collect();
        let mut mf = room("mf0", "MANUFACTURE", 3);
        mf.current_formula = Some("F_GOLD".to_string());
        mf.current_operators = vec![factory_op.to_string()];
        let mut tp = room("tp0", "TRADING", 3);
        tp.current_operators = vec![MR_NOTHING.to_string()];
        UserBuilding {
            drone_capacity: None,
            rooms: vec![room("cc", "CONTROL", 5), dorm, mf, tp],
        }
    };
    let eff = |asn: &myrtle_engine_base::types::BaseAssignment, slot: &str| {
        asn.rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .map_or_else(|| panic!("missing {slot}"), |r| r.total_efficiency)
    };
    let run = |factory_op: &str, resting: &[&str]| {
        compute_current_assignment(
            &ops,
            &assemble(factory_op, resting),
            gd.building(),
            &registry,
            &drains,
            None,
        )
    };

    // Mr. Nothing: 4 resting operators -> 4 Worldly Plight -> +4% on his post.
    let with4 = run(ROSMONTIS, &BODIES);
    let with0 = run(ROSMONTIS, &[]);
    assert!(
        (eff(&with4, "tp0") - eff(&with0, "tp0") - 4.0).abs() < 1e-9,
        "Mr. Nothing: 4 dorm occupants -> +4% order efficiency ({} vs {})",
        eff(&with4, "tp0"),
        eff(&with0, "tp0")
    );

    // Rosmontis: 4 dorm occupants -> 4 PI -> 4 Chain of Thought -> +4% (E2,
    // +1% per point) on her factory.
    assert!(
        (eff(&with4, "mf0") - eff(&with0, "mf0") - 4.0).abs() < 1e-9,
        "Rosmontis chain: 4 occupants -> +4% productivity ({} vs {})",
        eff(&with4, "mf0"),
        eff(&with0, "mf0")
    );

    // Cross-operator chain: 4 resting -> 4 WP is below the 5-per-crystal ratio
    // (floor 0), so the witch factory gains nothing. Swap Rosmontis into the dorm:
    // 5 resting -> 5 WP -> 1 Witchcraft Crystal -> +2% on the witch's factory.
    let mut five: Vec<&str> = BODIES.to_vec();
    five.push(ROSMONTIS);
    let witch5 = run(WITCH, &five);
    let witch_base4 = run(WITCH, &BODIES);
    assert!(
        (eff(&witch5, "mf0") - eff(&witch_base4, "mf0") - 2.0).abs() < 1e-9,
        "5 WP -> 1 crystal -> +2% ({} vs {})",
        eff(&witch5, "mf0"),
        eff(&witch_base4, "mf0")
    );
}

/// A dorm-fed pool with a co-feeder: Rosmontis' Chain of Thought draws on
/// her own Perception generation AND Ebenholz's Musicianship. The native
/// plan prices her on HER origin points (the search may not seat Ebenholz),
/// and a shared-pool bundle offers the oracle the alternative: pin Ebenholz
/// into a post and price every consumer of the pool at the full total.
#[test]
fn shared_dorm_pool_prices_own_origins_and_bundles_the_cofeeder() {
    use myrtle_engine_base::scoring::pools::{candidate_bundles, plan_optimal_economies};
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    const ROSMONTIS: &str = "char_391_rosmon";
    const EBENHOLZ: &str = "char_4046_ebnhlz";
    const MANIFEST: &str = "manu_prod_spd_bd[010]";
    let mut rooms = vec![room("mf", "MANUFACTURE", 3), room("tp", "TRADING", 3)];
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    // Deep roster: the projected dorm occupancy fills all 20 beds.
    let mut ids: Vec<&str> = vec![ROSMONTIS, EBENHOLZ];
    ids.extend(
        gd.building()
            .chars
            .keys()
            .filter(|id| id.starts_with("char_") && *id != ROSMONTIS && *id != EBENHOLZ)
            .take(40)
            .map(String::as_str),
    );
    let profiles: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();

    let native = plan_optimal_economies(&profiles, &building, gd.building(), &registry);
    let own = native
        .overrides
        .iter()
        .find(|(b, _)| b == MANIFEST)
        .map(|(_, v)| *v);
    assert_eq!(
        own,
        Some(20.0),
        "Rosmontis priced on her own 20 dorm points: {:?}",
        native.overrides
    );

    let bundles = candidate_bundles(&profiles, &building, gd.building(), &registry);
    // Exactly Ebenholz's seat: the filler roster may carry other co-feeders
    // (Dusk), whose larger subsets are bundles of their own.
    let shared = bundles
        .iter()
        .find(|b| b.pins == vec![(EBENHOLZ.to_string(), "TRADING".to_string())])
        .expect("a bundle pinning Ebenholz alone into a Trading Post");
    let full = shared
        .overrides
        .iter()
        .find(|(b, _)| b == MANIFEST)
        .map(|(_, v)| *v);
    assert_eq!(
        full,
        Some(40.0),
        "with Ebenholz seated the pool is 40: {:?}",
        shared.overrides
    );
}
