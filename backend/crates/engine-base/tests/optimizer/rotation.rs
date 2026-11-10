use super::*;

#[test]
fn rotation_plan_orders_members_by_when_to_swap_and_names_a_backup() {
    // The rotation plan: per production room, the main operators ordered by who
    // needs swapping FIRST (shortest hours), plus an idle backup to rotate in.
    let gd = load_game_data();
    let building = generic_base();
    let profiles = full_roster(gd);

    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let rotation =
        compute_sustained_assignment(&profiles, &building, gd.building(), &registry, &drains);

    assert!(
        rotation.main.rooms.iter().any(|r| r.room_type == "CONTROL"),
        "the main staffing surfaces the Control Center"
    );
    assert!(
        !rotation.rooms.is_empty(),
        "a deep roster has a rotation plan"
    );

    let mains: std::collections::HashSet<&String> = rotation
        .main
        .rooms
        .iter()
        .flat_map(|r| r.operators.iter())
        .collect();
    for room in &rotation.rooms {
        // Members are sorted soonest-to-swap first.
        let hours: Vec<f64> = room.members.iter().map(|m| m.lasts_hours).collect();
        assert!(
            hours.windows(2).all(|w| w[0] <= w[1] + 1e-6),
            "members in {} must be ordered by swap urgency (got {hours:?})",
            room.slot_id
        );
        assert!(
            room.members.iter().all(|m| m.lasts_hours > 0.0),
            "every member has a positive rotation interval"
        );
        // The backup isn't already a main.
        if let Some(b) = &room.backup {
            assert!(
                !mains.contains(b),
                "backup {b} is already working in the main staffing"
            );
        }
    }
}

#[test]
fn staggered_rotation_shares_a_small_bench_across_rooms() {
    // A staggered rotation swaps one operator at a time, so a single versatile
    // filler can back up several rooms: the shared bench is far smaller than a
    // doubled roster (one backup per room), never bigger than the room count.
    let gd = load_game_data();
    let building = generic_base();
    let profiles = full_roster(gd);

    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let rotation =
        compute_sustained_assignment(&profiles, &building, gd.building(), &registry, &drains);

    let production_rooms = rotation
        .rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE" || r.room_type == "TRADING")
        .count();
    assert!(
        production_rooms > 1,
        "the layout has several production rooms"
    );

    // The bench is shared (no duplicates) and covers all rooms with fewer fillers
    // than there are rooms; the same filler is reused, not one reserved per room.
    let unique: std::collections::HashSet<&String> = rotation.shared_bench.iter().collect();
    assert_eq!(
        unique.len(),
        rotation.shared_bench.len(),
        "the shared bench holds distinct fillers"
    );
    assert!(
        !rotation.shared_bench.is_empty() && rotation.shared_bench.len() <= production_rooms,
        "a shared bench ({}) is smaller than one-backup-per-room ({production_rooms})",
        rotation.shared_bench.len()
    );

    // No bench filler is also a main (they come from the idle pool).
    let mains: std::collections::HashSet<&String> = rotation
        .main
        .rooms
        .iter()
        .flat_map(|r| r.operators.iter())
        .collect();
    assert!(
        rotation.shared_bench.iter().all(|b| !mains.contains(b)),
        "bench fillers are idle operators, not mains"
    );
}

#[test]
fn current_preset_shifts_are_distinct() {
    // When the player has a planned rotation (preset shifts), the "current"
    // Shift A and Shift B must reflect those distinct presets, not the same
    // static crew for both.
    use myrtle_engine_base::assignment::{compute_current_assignment, has_preset_shifts};
    let gd = load_game_data();
    let profiles = full_roster(gd);
    // A trading post with two distinct planned shifts.
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![UserRoom {
            slot_id: "tp".into(),
            room_type: "TRADING".into(),
            level: 3,
            preset_shifts: vec![
                vec![TEXAS.into(), LAPPLAND.into(), EXUSIAI.into()],
                vec![
                    "char_502_nblade".into(),
                    "char_123_fang".into(),
                    "char_211_adnach".into(),
                ],
            ],
            ..Default::default()
        }],
    };
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    assert!(
        has_preset_shifts(&building),
        "building should expose the two planned shifts"
    );

    let ops = |shift: Option<usize>| {
        compute_current_assignment(
            &profiles,
            &building,
            gd.building(),
            &registry,
            &drains,
            shift,
        )
        .rooms
        .iter()
        .filter(|r| r.room_type == "TRADING" || r.room_type == "MANUFACTURE")
        .flat_map(|r| r.operators.iter().cloned())
        .collect::<Vec<_>>()
    };
    let a = ops(Some(0));
    let b = ops(Some(1));
    assert!(
        !a.is_empty() && a != b,
        "current shift A and B presets must differ\nA: {a:?}\nB: {b:?}"
    );
}

#[test]
fn staggered_rotation_sustains_near_peak() {
    // A staggered rotation keeps your best operators working almost all the time
    // (swap only the lowest-morale one), so sustained 24/7 output is CLOSE to peak
    // rather than a big drop from averaging weaker whole teams. LOCKED synergy teams can't
    // be staggered one operator at a time, so they sustain a little further below
    // peak (their rest gaps go uncovered), which is why the floor is ~20%.
    let gd = load_game_data();
    let building = generic_base();
    let profiles = full_roster(gd);
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let rot = compute_sustained_assignment(&profiles, &building, gd.building(), &registry, &drains);
    let peak: f64 = rot
        .main
        .rooms
        .iter()
        .filter(|r| r.room_type == "TRADING" || r.room_type == "MANUFACTURE")
        .map(|r| r.total_efficiency)
        .sum();
    assert!(
        rot.sustained_efficiency >= peak * 0.80 && rot.sustained_efficiency <= peak,
        "sustained ({:.1}) should be within ~20% of peak ({:.1})",
        rot.sustained_efficiency,
        peak
    );
}

#[test]
fn texas_is_not_a_standalone_rotation_backup_without_lappland() {
    // Texas's trading value is entirely conditional on Lappland sharing the post
    // (her base is 0). As a staggered rotation backup she is swapped in to cover one
    // resting main, so with no Lappland present she does nothing; she must NOT be
    // chosen as a standalone backup over a real trader.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 3)],
    };
    // Three strong traders staff the post; Texas (no Lappland) and one ordinary
    // trader sit idle. The ordinary trader, not Texas, should be the backup.
    let roster = vec![
        profile(gd, "char_103_angel"),  // Exusiai +35
        profile(gd, "char_4032_provs"), // Proviso (order value)
        profile(gd, "char_502_nblade"), // Yato +30
        profile(gd, "char_185_frncat"), // Mousse +30
        profile(gd, "char_102_texas"),  // Texas (conditional on Lappland)
    ];
    let rot = compute_sustained_assignment(&roster, &building, gd.building(), &registry, &drains);
    let tp = rot
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post rotation");
    assert_ne!(
        tp.backup.as_deref(),
        Some("char_102_texas"),
        "Texas must not be a standalone backup with no Lappland to enable her"
    );
    assert!(
        !rot.shared_bench.iter().any(|b| b == "char_102_texas"),
        "Texas should not sit on the shared rotation bench as a useless filler"
    );
}

#[test]
fn a_locked_synergy_crew_is_relieved_by_the_next_shift_like_any_other() {
    // The rotation the planner recommends relieves a Texas+Lappland post with
    // the next shift's WHOLE crew, so its rest gaps are covered as well as a
    // flexible room's are. Pricing a locked crew's rest at zero coverage made
    // the grade prefer a plain 75-speed post over a 132-speed synergy post the
    // search had chosen (56505800: the player's plain crews out-graded the
    // plan by 4.5%). Sustained value must not depend on the `locked` flag.
    use myrtle_engine_base::assignment::sustained_assignment_value;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("tp", "TRADING", 3),
            room("mf", "MANUFACTURE", 3),
            room("d0", "DORMITORY", 5),
            room("d1", "DORMITORY", 5),
        ],
    };
    let roster = vec![
        profile(gd, TEXAS),
        profile(gd, LAPPLAND),
        profile(gd, EXUSIAI),
        profile(gd, "char_123_fang"),
        profile(gd, "char_133_mm"),
    ];
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let post = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post");
    assert!(
        post.locked && post.operators.contains(&TEXAS.to_string()),
        "precondition: the Penguin synergy squad forms and is locked, got {:?}",
        post.operators
    );
    let value = |a: &myrtle_engine_base::types::BaseAssignment| {
        sustained_assignment_value(a, &roster, &building, gd.building(), &registry, &drains)
    };
    let mut unlocked = asn.clone();
    for r in &mut unlocked.rooms {
        r.locked = false;
    }
    let (locked_v, unlocked_v) = (value(&asn), value(&unlocked));
    assert!(
        locked_v > 0.0 && (locked_v - unlocked_v).abs() < 1e-6,
        "a locked crew sustains exactly like the same crew unlocked: {locked_v:.1} vs {unlocked_v:.1}"
    );
}

#[test]
fn every_production_room_changes_crew_at_every_rotate_on_a_full_roster() {
    // The game's global Rotate advances every room's preset each 12h and
    // refuses a room whose next formation is its working crew ("No Available
    // Presets", user recording 2026-10-14). A roster that can staff three
    // teams per production room gets one team per room per shift, so every
    // post and factory changes crew at every rotate and the three presets
    // the game offers map one to one onto the three shifts.
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let building = generic_base();
    let rot = recommend_shift_rotation(
        &full_roster(gd),
        &building,
        gd.building(),
        &registry,
        &drains,
        &[],
    );
    for room in building
        .rooms
        .iter()
        .filter(|r| matches!(r.room_type.as_str(), "TRADING" | "MANUFACTURE"))
    {
        let crews: Vec<std::collections::HashSet<String>> = rot
            .shifts
            .iter()
            .map(|s| {
                s.rooms
                    .iter()
                    .find(|r| r.slot_id == room.slot_id)
                    .map(|r| r.recommended.iter().cloned().collect())
                    .unwrap_or_default()
            })
            .collect();
        assert!(
            crews.iter().all(|c| !c.is_empty()),
            "{} is staffed every shift: {crews:?}",
            room.slot_id
        );
        for k in 0..3 {
            let next = &crews[(k + 1) % 3];
            assert!(
                crews[k] != *next,
                "{} changes crew between shift {} and the next: {crews:?}",
                room.slot_id,
                k + 1
            );
        }
    }
}

#[test]
fn presets_set_to_the_recommended_rotation_read_as_its_value() {
    // "i set the shifts exactly how the optimizer suggested but it keeps saying
    // that i havent optimized the exp production" (2026-10-15): the Optimizer tab
    // compared the PEAK single staffing with the shift stationed right now, which
    // a rotating base never equals. Rotation against rotation closes: saving the
    // recommended shifts as presets values exactly what the recommendation
    // values, and every production cell reads as a match.
    use myrtle_engine_base::rotation::shift::{SHIFT_COUNT, recommend_shift_rotation};
    use myrtle_engine_base::rotation::value::{
        presets_value, rotation_value, usable_preset_shifts,
    };
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let roster = full_roster(gd);
    let mut building = generic_base();
    // Two gold and two EXP lines, so both yields are live in the comparison.
    for (i, room) in building
        .rooms
        .iter_mut()
        .filter(|r| r.room_type == "MANUFACTURE")
        .enumerate()
    {
        room.current_formula = Some(if i < 2 { "F_GOLD" } else { "F_EXP" }.into());
    }
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);
    let recommended = rotation_value(&rot, &roster, &building, gd.building(), &registry, &drains)
        .expect("a rotation has shifts");
    assert!(recommended.exp_per_day > 0.0 && recommended.lmd_per_day > 0.0);
    assert!(
        presets_value(&roster, &building, gd.building(), &registry, &drains).is_none(),
        "no presets, no preset value"
    );

    // The player saves every room's crew of each shift as that preset (the
    // game keeps presets per room, the Control Center and dorms included).
    for room in &mut building.rooms {
        room.preset_shifts = rot
            .shifts
            .iter()
            .map(|s| {
                s.rooms
                    .iter()
                    .find(|r| r.slot_id == room.slot_id && r.active)
                    .map(|r| r.recommended.clone())
                    .unwrap_or_default()
            })
            .collect();
    }
    assert_eq!(usable_preset_shifts(&building).len(), SHIFT_COUNT);
    let presets = presets_value(&roster, &building, gd.building(), &registry, &drains)
        .expect("three usable presets")
        .value;
    // Every room is stationed identically shift by shift on both sides, so the
    // two valuations agree exactly.
    assert!(
        (presets.exp_per_day - recommended.exp_per_day).abs() < 1e-6
            && (presets.lmd_per_day - recommended.lmd_per_day).abs() < 1e-6
            && (presets.production_efficiency - recommended.production_efficiency).abs() < 1e-6,
        "presets {presets:?} vs recommended {recommended:?}"
    );
}

#[test]
fn rotation_is_expressed_as_overlapping_sets() {
    // The rotation surfaces as a few overlapping staffings: each set rests one main
    // per room (covered by the backup), consecutive sets share all-but-one operator,
    // and across the sets every main rests in turn, so the whole base is never
    // swapped at once.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let rot = compute_sustained_assignment(
        &full_roster(gd),
        &generic_base(),
        gd.building(),
        &registry,
        &drains,
    );

    assert!(
        !rot.sets.is_empty(),
        "a deep roster yields a multi-set rotation"
    );
    assert!(
        rot.sets.len() <= 3,
        "at most three sets (the 12h-swap cadence)"
    );

    let tp_working: Vec<Vec<String>> = rot
        .sets
        .iter()
        .map(|set| {
            let tp = set
                .rooms
                .iter()
                .find(|r| r.room_type == "TRADING")
                .expect("a trading post in each set");
            assert!(
                tp.resting
                    .as_ref()
                    .is_none_or(|rest| !tp.working.contains(rest)),
                "the resting main is not also working in the same set"
            );
            tp.working.clone()
        })
        .collect();

    let mains = tp_working[0].len();
    assert!(
        tp_working.iter().all(|w| w.len() == mains),
        "every set fully staffs the post"
    );

    // Each set rests a DIFFERENT main (the rotation visits each in turn).
    let rested: Vec<&String> = rot
        .sets
        .iter()
        .filter_map(|s| {
            s.rooms
                .iter()
                .find(|r| r.room_type == "TRADING")
                .and_then(|r| r.resting.as_ref())
        })
        .collect();
    let unique: std::collections::HashSet<&&String> = rested.iter().collect();
    assert_eq!(unique.len(), rested.len(), "each set rests a distinct main");

    // Overlap: consecutive sets share all-but-one working operator.
    for pair in tp_working.windows(2) {
        let a: std::collections::HashSet<&String> = pair[0].iter().collect();
        let b: std::collections::HashSet<&String> = pair[1].iter().collect();
        assert!(
            a.intersection(&b).count() >= mains - 1,
            "consecutive sets share all but one operator"
        );
    }
}

#[test]
fn set_count_tracks_real_rest_demand_not_seat_count() {
    // The number of rotation sets should reflect how many operators actually run low
    // on morale within a cycle, NOT just the seat count. A factory of low-drain
    // operators (Vermeil/Vulcan-type, ~48h before a swap) needs no rest slots and
    // yields zero sets, while the same factory of neutral-drain operators (~24h)
    // rotates each of them and yields the full three sets.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("mf", "MANUFACTURE", 3), room("d0", "DORMITORY", 3)],
    };
    let sets_for = |ids: &[&str]| {
        let roster: Vec<_> = ids.iter().map(|id| profile(gd, id)).collect();
        compute_sustained_assignment(&roster, &building, gd.building(), &registry, &drains)
            .sets
            .len()
    };
    // Low-drain factory operators (each lasts ~48h, beyond the ~36h cycle).
    let low_drain = sets_for(&[
        "char_163_hpsts",  // Vulcan
        "char_452_bstalk", // Beanstalk
        "char_190_clour",  // Vermeil
        "char_485_pallas", // Pallas
        "char_369_bena",   // Bena
    ]);
    // Neutral-drain factory operators (each lasts ~24h, inside the cycle).
    let neutral = sets_for(&[
        "char_242_otter",   // Mayer
        "char_4141_marcil", // Marcille
        "char_4063_quartz", // Quartz
        "char_4041_chnut",  // Chestnut
        "char_496_wildmn",  // Wild Mane
    ]);
    assert_eq!(
        low_drain, 0,
        "a factory of low-drain operators needs no rotation sets"
    );
    assert!(
        neutral > low_drain,
        "neutral-drain operators rotate, so they yield more sets ({neutral}) than low-drain ({low_drain})"
    );
}

#[test]
fn weedy_is_not_suggested_as_a_rotation_backup() {
    // Weedy sets every non-automation teammate's output to 0, so swapping her into a
    // working room as a backup would wreck it. When she is not a main she must not be
    // offered as a backup for any room (and certainly not for several at once).
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    // Four factories + one power plant: Weedy's lone automation output (one plant)
    // loses to the strong normal teams, so she sits idle: the exact case where she
    // used to be offered as a backup for EVERY factory at once.
    let mut rooms: Vec<UserRoom> = (0..4)
        .map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3))
        .collect();
    rooms.push(room("p0", "POWER", 3));
    rooms.push(room("d0", "DORMITORY", 3));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let roster: Vec<_> = [
        "char_242_otter",
        "char_4141_marcil",
        "char_237_gravel",
        "char_141_nights",
        "char_4063_quartz",
        "char_496_wildmn",
        "char_4041_chnut",
        "char_135_halo",
        "char_4066_highmo",
        "char_430_fartth",
        "char_431_ashlok",
        "char_484_robrta",
        "char_400_weedy",
    ]
    .iter()
    .map(|c| profile(gd, c))
    .collect();
    let rot = compute_sustained_assignment(&roster, &building, gd.building(), &registry, &drains);

    // Weedy is not staffing any room as a main here (normal teams out-produce her).
    let is_main = rot
        .main
        .rooms
        .iter()
        .any(|r| r.operators.iter().any(|o| o == "char_400_weedy"));
    assert!(!is_main, "precondition: Weedy is idle, not a main");

    assert!(
        rot.rooms
            .iter()
            .all(|r| r.backup.as_deref() != Some("char_400_weedy")),
        "Weedy must never be a room backup (she nullifies the team)"
    );
    assert!(
        !rot.shared_bench.iter().any(|b| b == "char_400_weedy"),
        "Weedy must not sit on the shared rotation bench"
    );
}

#[test]
fn locked_synergy_teams_get_no_individual_backup() {
    // A locked synergy team (its operators depend on each other) can't be staggered
    // one operator at a time without breaking the combo, so the rotation must NOT
    // offer it an individual backup. Flexible rooms still get one. (generic_base's
    // optimal forms locked trading-post synergy teams.)
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let rot = compute_sustained_assignment(
        &full_roster(gd),
        &generic_base(),
        gd.building(),
        &registry,
        &drains,
    );

    let locked_slots: std::collections::HashSet<&String> = rot
        .main
        .rooms
        .iter()
        .filter(|r| r.locked)
        .map(|r| &r.slot_id)
        .collect();
    assert!(
        !locked_slots.is_empty(),
        "precondition: a locked synergy team forms"
    );

    for room in &rot.rooms {
        if locked_slots.contains(&room.slot_id) {
            assert!(
                room.backup.is_none(),
                "locked team {} must not get an individual backup",
                room.slot_id
            );
        }
    }
    assert!(
        rot.rooms.iter().any(|r| r.backup.is_some()),
        "flexible rooms still get a backup"
    );
}

#[test]
fn shift_rotation_forms_three_overlapping_shifts() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    // A 243 layout: 2 trading posts, 4 factories, 3 power plants, CC, dorms.
    let mut rooms = vec![
        room("cc", "CONTROL", 5),
        room("hr", "HIRE", 3),
        room("rc", "MEETING", 3),
    ];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("p{i}"), "POWER", 3)));
    rooms.extend((0..2).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let rot = recommend_shift_rotation(
        &full_roster(gd),
        &building,
        gd.building(),
        &registry,
        &drains,
        &[],
    );

    assert_eq!(rot.shifts.len(), 3, "three shifts");

    let cell = |shift: usize, slot: &str| -> &myrtle_engine_base::rotation::shift::ShiftRoom {
        rot.shifts[shift]
            .rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .unwrap()
    };
    let crew = |shift: usize, slot: &str| -> Vec<String> {
        let mut v = cell(shift, slot).recommended.clone();
        v.sort();
        v
    };

    // The Control Center runs TWO squads: Squad 1 on two shifts, Squad 2 (disjoint) covering the
    // third: either alternating (1&3 / 2) or as a 24h block (1+2 / 3) when a synergy aligns it.
    let cc_slot = "cc";
    let cc_crews: Vec<Vec<String>> = (0..3).map(|k| crew(k, cc_slot)).collect();
    assert!(
        (0..3).all(|k| cell(k, cc_slot).active),
        "CC staffed all three shifts"
    );
    let distinct_cc: std::collections::HashSet<&Vec<String>> = cc_crews.iter().collect();
    assert_eq!(
        distinct_cc.len(),
        2,
        "CC cycles exactly two squads: {cc_crews:?}"
    );
    let (a, b) = (
        &cc_crews[0],
        cc_crews.iter().find(|c| *c != &cc_crews[0]).unwrap(),
    );
    assert!(
        !a.iter().any(|o| b.contains(o)),
        "the two CC squads are disjoint so Squad 1 actually rests"
    );

    // The HR Office and Reception Room cycle two squads too: the per-login swap.
    for slot in ["hr", "rc"] {
        let crews: Vec<Vec<String>> = (0..3).map(|k| crew(k, slot)).collect();
        assert!(!crews[0].is_empty(), "{slot} should be staffed");
        let distinct: std::collections::HashSet<&Vec<String>> = crews.iter().collect();
        assert_eq!(distinct.len(), 2, "{slot} cycles two squads: {crews:?}");
    }

    // The trading-post pair runs the login rhythm: THREE distinct, pairwise-disjoint teams tiled
    // as 24h blocks (room0 = [A, A, B], room1 = [B, C, C]), so each team works two consecutive
    // shifts then rests one, and exactly one of the pair's rooms changes team at each boundary.
    let team_id = |shift: usize, slot: &str| -> String {
        cell(shift, slot)
            .team_id
            .clone()
            .expect("production cells carry a team id")
    };
    let tp_ids: Vec<Vec<String>> = ["tp0", "tp1"]
        .iter()
        .map(|slot| (0..3).map(|k| team_id(k, slot)).collect())
        .collect();
    // Two dormitories hold ten beds for eighteen production seats, so the
    // rotation runs the 24h blocks: the two posts are interchangeable,
    // whichever holds the [A, A, B] block is "early", the other runs [B, C, C].
    let (early, late) = if tp_ids[0][0] == tp_ids[0][1] {
        (0, 1)
    } else {
        (1, 0)
    };
    assert_eq!(
        tp_ids[early][0], tp_ids[early][1],
        "the early post runs one team for shifts 1+2: {tp_ids:?}"
    );
    assert_ne!(
        tp_ids[early][1], tp_ids[early][2],
        "the early post swaps its team for shift 3: {tp_ids:?}"
    );
    assert_eq!(
        tp_ids[late][1], tp_ids[late][2],
        "the late post runs one team for shifts 2+3: {tp_ids:?}"
    );
    assert_eq!(
        tp_ids[early][2], tp_ids[late][0],
        "the wrap team's block spans the early post's shift 3 and the late post's shift 1: {tp_ids:?}"
    );
    let distinct_teams: std::collections::HashSet<&String> = tp_ids.iter().flatten().collect();
    assert_eq!(
        distinct_teams.len(),
        3,
        "a pair of posts runs exactly three teams under the blocks"
    );
    // Teams working the same shift never share an operator, and a team shares nobody with its
    // replacement, EXCEPT a Fiammetta-sustained operator, who is deliberately pinned to one
    // room across every shift (the full roster owns a manager, so one trader gets sustained).
    let sustained: std::collections::HashSet<&String> = rot.sustained.iter().collect();
    let rotating = |c: Vec<String>| -> Vec<String> {
        c.into_iter().filter(|o| !sustained.contains(o)).collect()
    };
    for k in 0..3 {
        let (c0, c1) = (rotating(crew(k, "tp0")), rotating(crew(k, "tp1")));
        assert!(
            !c0.iter().any(|o| c1.contains(o)),
            "shift {k}: concurrent trading teams are disjoint"
        );
    }
    assert!(!crew(0, "tp0").is_empty(), "tp0 staffed");
    assert!(
        !rotating(crew(0, "tp0"))
            .iter()
            .any(|o| rotating(crew(2, "tp0")).contains(o)),
        "a team and its replacement share no rotating operator"
    );

    // A power plant alternates two squads (Squad 1 on shifts 1&3, Squad 2 on shift 2), so its
    // operators recover instead of working two shifts in a row.
    let p0: Vec<Vec<String>> = (0..3).map(|k| crew(k, "p0")).collect();
    assert!(!p0[0].is_empty(), "the power plant should be staffed");
    assert_eq!(p0[0], p0[2], "the plant runs Squad 1 on shifts 1 and 3");
    assert_ne!(
        p0[0], p0[1],
        "Squad 2 (or a dark rest) covers the middle shift"
    );
}

#[test]
fn shift_rotation_never_double_books_an_operator_within_a_shift() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    use std::collections::HashMap;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    // bd_base_243 has 3 power plants, exercising the widened power-plan exclusion.
    let building = bd_base_243();
    let roster = full_roster(gd);
    let rotation =
        recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);
    for shift in &rotation.shifts {
        let mut owner: HashMap<&str, &str> = HashMap::new();
        for room in shift.rooms.iter().filter(|r| r.active) {
            for op in &room.recommended {
                if let Some(prev) = owner.insert(op.as_str(), room.slot_id.as_str()) {
                    panic!(
                        "shift {} double-books {op}: in both {prev} and {}",
                        shift.index, room.slot_id
                    );
                }
            }
        }
    }
}

// ─── Part B: a morale-swap manager (Fiammetta) sustains ONE operator 24/7 ────────────────────

#[test]
fn morale_swap_manager_sustains_one_operator_at_full_uptime() {
    use myrtle_engine_base::assignment::{
        morale_recovery, morale_sustained_beneficiaries, sustained_efficiency_of,
    };
    use std::collections::HashSet;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    // The manager has to STAY in a dormitory for the swap to work, so the
    // fixture needs one for anyone to be sustainable at all.
    let mut building = trading_post(3);
    building.rooms.push(room("d0", "DORMITORY", 2));
    let recovery = morale_recovery(&building);
    const PROVISO: &str = "char_4032_provs";
    const FIAMMETTA: &str = "char_300_phenxi"; // swaps morale with another operator

    // No morale-swap manager owned -> nobody is held at full morale.
    let base: Vec<_> = [PROVISO, EXUSIAI, TEXAS]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let main = compute_optimal_assignment(&base, &building, gd.building(), &registry, &drains);
    let none = morale_sustained_beneficiaries(
        &main,
        &base,
        &drains,
        recovery,
        &building,
        &registry,
        gd.building(),
    );
    assert!(
        none.is_empty(),
        "no manager -> no sustained operator, got {none:?}"
    );

    // With Fiammetta owned -> EXACTLY ONE operator is sustained, and crediting it doesn't lower
    // (it raises) the trading post's sustained efficiency.
    let mut with = base;
    with.push(profile(gd, FIAMMETTA));
    let main2 = compute_optimal_assignment(&with, &building, gd.building(), &registry, &drains);
    let benef = morale_sustained_beneficiaries(
        &main2,
        &with,
        &drains,
        recovery,
        &building,
        &registry,
        gd.building(),
    );
    assert_eq!(
        benef.len(),
        1,
        "one manager sustains exactly one operator, got {benef:?}"
    );
    let without = sustained_efficiency_of(
        &main2,
        &with,
        &drains,
        recovery,
        &registry,
        gd.building(),
        &HashSet::new(),
    );
    let credited = sustained_efficiency_of(
        &main2,
        &with,
        &drains,
        recovery,
        &registry,
        gd.building(),
        &benef,
    );
    assert!(
        credited >= without,
        "the sustained operator should not reduce sustained output ({credited} vs {without})"
    );
}

// ─── Preset-based Fiammetta 24/7 sustain in the shift rotation ───────────────────────────────

#[test]
fn preset_24_7_operator_with_fiammetta_is_kept_every_shift_and_flagged() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const PROVISO: &str = "char_4032_provs";
    const FIAMMETTA: &str = "char_300_phenxi"; // swaps morale with another operator
    const QUARTZ: &str = "char_4063_quartz";
    // The player runs Proviso in BOTH saved presets of the trading post (a deliberate 24/7 hold)
    // and owns Fiammetta to sustain her.
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![UserRoom {
            slot_id: "tp".into(),
            room_type: "TRADING".into(),
            level: 3,
            preset_shifts: vec![
                vec![PROVISO.into(), QUARTZ.into(), EXUSIAI.into()],
                vec![
                    PROVISO.into(),
                    "char_502_nblade".into(),
                    "char_185_frncat".into(),
                ],
            ],
            ..Default::default()
        }],
    };
    // The manager has to STAY in a dormitory for the swap to work.
    let mut building = building;
    building.rooms.push(room("d0", "DORMITORY", 2));
    let roster: Vec<_> = [
        PROVISO,
        FIAMMETTA,
        QUARTZ,
        EXUSIAI,
        "char_502_nblade",
        "char_185_frncat",
        "char_211_adnach",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .collect();

    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);

    assert_eq!(
        rot.sustained,
        vec![PROVISO.to_string()],
        "Proviso (run 24/7 in both presets, with Fiammetta owned) should be flagged sustained"
    );
    // She works the trading post in ALL THREE shifts, including the middle (rest) shift.
    for shift in &rot.shifts {
        let tp = shift
            .rooms
            .iter()
            .find(|r| r.room_type == "TRADING")
            .expect("a trading post each shift");
        assert!(
            tp.active && tp.recommended.iter().any(|o| o == PROVISO),
            "Proviso must keep working on shift {} (got active={}, {:?})",
            shift.index,
            tp.active,
            tp.recommended
        );
        // The manager makes that possible only by STAYING in a dormitory:
        // the rotation reserves her seat itself, with no caller pins.
        const FIAMMETTA: &str = "char_300_phenxi";
        assert!(
            shift.rooms.iter().any(|r| {
                r.room_type == "DORMITORY" && r.recommended.iter().any(|o| o == FIAMMETTA)
            }),
            "shift {}: Fiammetta must hold her dormitory seat while Proviso runs 24/7",
            shift.index
        );
    }
}

#[test]
fn without_a_manager_the_24_7_preset_operator_still_rests_the_middle_shift() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const PROVISO: &str = "char_4032_provs";
    const QUARTZ: &str = "char_4063_quartz";
    // Same 24/7 preset, but NO morale-swap manager owned -> she is not held 24/7.
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![UserRoom {
            slot_id: "tp".into(),
            room_type: "TRADING".into(),
            level: 3,
            preset_shifts: vec![
                vec![PROVISO.into(), QUARTZ.into(), EXUSIAI.into()],
                vec![
                    PROVISO.into(),
                    "char_502_nblade".into(),
                    "char_185_frncat".into(),
                ],
            ],
            ..Default::default()
        }],
    };
    let roster: Vec<_> = [
        PROVISO,
        QUARTZ,
        EXUSIAI,
        "char_502_nblade",
        "char_185_frncat",
        "char_211_adnach",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .collect();

    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);

    assert!(
        rot.sustained.is_empty(),
        "without a manager nobody is sustained, got {:?}",
        rot.sustained
    );
    // Without a manager Proviso rotates like everyone else: her team's 24h block covers at most
    // two shifts, so she must NOT be shown working all three.
    let shifts_working = rot
        .shifts
        .iter()
        .filter(|s| {
            s.rooms
                .iter()
                .any(|r| r.active && r.recommended.iter().any(|o| o == PROVISO))
        })
        .count();
    assert!(
        shifts_working <= 2,
        "Proviso must rest at least one shift without a manager (worked {shifts_working}/3)"
    );
}

/// The community's "perma Tequila combo" (base expert, 2026-09-08): Fiammetta
/// refreshes TEQUILA around the clock in a level-3 post while Shamare, Bibeak
/// and Kafka cycle beside him. The model derives it from the gold coupling:
/// with one gold factory feeding the post, more speed moves no extra bars,
/// so Tequila's per-bar LMD rider (fired by the Tailoring mix) is the seat
/// worth holding 24/7, and the speed anchor rotates.
#[test]
fn gold_starved_post_sustains_tequila_and_rotates_shamare_and_bibeak() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    const SHAMARE: &str = "char_254_vodfox";
    const TEQUILA: &str = "char_486_takila";
    const BIBEAK: &str = "char_252_bibeak";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("tp", "TRADING", 3),
            room("mf", "MANUFACTURE", 3),
            room("d0", "DORMITORY", 5),
        ],
    };
    let roster: Vec<_> = [
        SHAMARE,
        TEQUILA,
        BIBEAK,
        "char_214_kafka",
        "char_300_phenxi",
        EXUSIAI,
        "char_502_nblade",
        "char_185_frncat",
        "char_123_fang",
        "char_124_kroos",
        "char_278_orchid",
        "char_190_clour",
        "char_237_gravel",
        "char_381_bubble",
        "char_328_cammou",
        "char_282_catap",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .collect();
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);
    assert_eq!(
        rot.sustained,
        vec![TEQUILA.to_string()],
        "the gold-bound post holds Tequila 24/7"
    );
    let posts: Vec<Vec<String>> = rot
        .shifts
        .iter()
        .map(|s| {
            s.rooms
                .iter()
                .find(|r| r.slot_id == "tp")
                .map(|r| r.recommended.clone())
                .unwrap_or_default()
        })
        .collect();
    assert!(
        posts.iter().all(|crew| crew.iter().any(|o| o == TEQUILA)),
        "Tequila every shift: {posts:?}"
    );
    assert!(
        posts
            .iter()
            .any(|crew| crew.iter().any(|o| o == SHAMARE) && crew.iter().any(|o| o == BIBEAK)),
        "Shamare and Bibeak combo with him on a block: {posts:?}"
    );
    assert!(
        !posts.iter().all(|crew| crew.iter().any(|o| o == SHAMARE)),
        "the speed anchor rotates, she is not the 24/7 seat: {posts:?}"
    );
}

#[test]
fn two_presets_are_valued_as_the_morale_bound_cycle_they_run() {
    // uid 93881052 (2026-10-23): "all operators remain working until one
    // facility hits zero morale, then rotate the whole base", 34h then 8-19h.
    // Two presets: the first works until its first worker runs dry, the second
    // until the first set (resting in the dorm) is back to full, and the figures
    // weight each preset by its hours instead of reading two equal shifts.
    use myrtle_engine_base::assignment::compute_current_assignment_over;
    use myrtle_engine_base::rotation::dorms::dorm_list;
    use myrtle_engine_base::rotation::sustain_sim::{MORALE_MAX, game_morale_drain};
    use myrtle_engine_base::rotation::value::{preset_cycle, presets_value};
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let a = ["char_237_gravel", "char_4106_bryota"];
    let b = ["char_400_weedy", "char_385_finlpp"];
    let mut factory = room("mf0", "MANUFACTURE", 3);
    factory.current_formula = Some("F_GOLD".into());
    factory.current_operators = a.iter().map(|s| (*s).to_string()).collect();
    factory.preset_shifts = vec![
        a.iter().map(|s| (*s).to_string()).collect(),
        b.iter().map(|s| (*s).to_string()).collect(),
    ];
    // The dorm holds whoever is off: the first set while the second works.
    let mut dorm = room("d0", "DORMITORY", 5);
    dorm.current_operators = a.iter().chain(&b).map(|s| (*s).to_string()).collect();
    let building = UserBuilding {
        rooms: vec![factory, dorm],
        drone_capacity: None,
    };
    let roster: Vec<OperatorBaseProfile> = a.iter().chain(&b).map(|id| profile(gd, id)).collect();
    let legs = preset_cycle(&[0, 1], &roster, &building, gd.building(), &drains)
        .expect("two usable presets make a cycle");
    let drain = |ids: &[&str]| {
        ids.iter()
            .map(|id| game_morale_drain(roster.iter().find(|o| o.char_id == *id).unwrap(), &drains))
            .fold(0.0, f64::max)
    };
    let recovery = dorm_list(&building, gd.building())[0].recovery_per_hour;
    let t_a = MORALE_MAX / drain(&a);
    let t_b = (drain(&a) * t_a / recovery).min(MORALE_MAX / drain(&b));
    assert!(
        (legs[0] - t_a).abs() < 1e-6 && (legs[1] - t_b).abs() < 1e-6,
        "legs {legs:?} vs {t_a} / {t_b}"
    );
    assert!(
        legs[0] > legs[1],
        "the first set works longer than the second recovers: {legs:?}"
    );

    let v = presets_value(&roster, &building, gd.building(), &registry, &drains).expect("valued");
    assert_eq!(v.legs_hours, legs);
    let leg_value = |i: usize, h: f64| {
        compute_current_assignment_over(
            &roster,
            &building,
            gd.building(),
            &registry,
            &drains,
            Some(i),
            h,
        )
        .total_production_efficiency
    };
    let expect =
        (leg_value(0, legs[0]) * legs[0] + leg_value(1, legs[1]) * legs[1]) / (legs[0] + legs[1]);
    assert!(
        (v.value.production_efficiency - expect).abs() < 1e-6,
        "{} vs {expect}",
        v.value.production_efficiency
    );
}

#[test]
fn the_optimizer_proposes_a_hand_swapped_cycle_beside_the_rotation() {
    // uid 93881052 (2026-10-23) out-earned our 3 x 12h rotation by 7% with a
    // hand-swapped A/B cycle. The planner now proposes one: the peak plan works
    // until a room runs dry, the best plan of everyone else until the first has
    // recovered, dorms holding the off leg. Valued exactly as a saved pair of
    // presets would be.
    use myrtle_engine_base::rotation::value::plan_cycle;
    use myrtle_engine_base::scoring::pools::optimal_with_bundles;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    // A fifth L5 dorm: the peak plan works 23 operators and every one of them
    // needs a bed while the second leg runs (a 4-dorm base has 20 and gets no
    // cycle proposal).
    let mut building = generic_base();
    building.rooms.push(room("d4", "DORMITORY", 5));
    let roster = full_roster(gd);
    let accepted = optimal_with_bundles(
        &roster,
        &building,
        gd.building(),
        &registry,
        &registry,
        &drains,
        &[],
    );
    assert!(
        plan_cycle(
            &accepted.optimal,
            &roster,
            &generic_base(),
            gd.building(),
            &registry,
            &accepted.registry,
            &drains,
            &accepted.pins,
        )
        .is_none(),
        "20 beds cannot rest 23 workers: no cycle"
    );
    let cycle = plan_cycle(
        &accepted.optimal,
        &roster,
        &building,
        gd.building(),
        &registry,
        &accepted.registry,
        &drains,
        &accepted.pins,
    )
    .expect("a deep roster staffs a second leg");
    assert_eq!(cycle.legs.len(), 2);
    assert!(
        cycle.legs_hours.iter().all(|h| *h > 0.0),
        "{:?}",
        cycle.legs_hours
    );
    // The second leg works nobody the first leg works, and every production
    // room is staffed in both legs.
    let workers = |leg: &[(String, Vec<String>)]| -> std::collections::HashSet<String> {
        leg.iter()
            .filter(|(slot, _)| {
                building
                    .rooms
                    .iter()
                    .any(|r| &r.slot_id == slot && r.room_type != "DORMITORY")
            })
            .flat_map(|(_, ops)| ops.iter().cloned())
            .collect()
    };
    let (a, b) = (workers(&cycle.legs[0]), workers(&cycle.legs[1]));
    assert!(
        a.is_disjoint(&b),
        "the legs share workers: {:?}",
        a.intersection(&b).collect::<Vec<_>>()
    );
    for room in building
        .rooms
        .iter()
        .filter(|r| matches!(r.room_type.as_str(), "TRADING" | "MANUFACTURE"))
    {
        for leg in &cycle.legs {
            let crew = &leg
                .iter()
                .find(|(s, _)| *s == room.slot_id)
                .expect("room listed")
                .1;
            assert!(!crew.is_empty(), "{} is dark in a leg", room.slot_id);
        }
    }
    // The first leg's workers rest in the dorms during the second leg.
    let resting: std::collections::HashSet<String> = cycle.legs[1]
        .iter()
        .filter(|(slot, _)| {
            building
                .rooms
                .iter()
                .any(|r| &r.slot_id == slot && r.room_type == "DORMITORY")
        })
        .flat_map(|(_, ops)| ops.iter().cloned())
        .collect();
    assert!(
        !resting.is_empty() && resting.is_subset(&a),
        "dorms hold the first leg's workers"
    );
    assert!(cycle.value.total_value > 0.0);
}

/// The rotation's Control-Center Squad 2 must run the same dead-weight rule as
/// Squad 1. Squad 1's eviction loop drops a conditional operator whose gate the
/// planned teams never satisfy, but its exclusion list used to stay private, so
/// the Squad-2 leftover greedy re-picked the SAME operator at face value and
/// seated her on the third shift with nobody to receive the bonus (the uid
/// 09525371 report: Delphine in the shift-3 CC, zero Glasgow anywhere).
#[test]
fn rotation_squad2_evicts_conditional_cc_ops_no_team_satisfies() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    const BUFF: &str = "control_tra_limit&spd[010]";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
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

    // The roster OWNS a Glasgow trader (Morgan carries a TRADING buff, so the
    // roster-level feasibility gate passes and Delphine keeps her face-value
    // selection weight), but the base has NO Trading Post: the condition fires
    // in no plannable team, so Delphine may hold a seat in NO shift's Control
    // Center, squad 2's leftover fill included.
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("cc", "CONTROL", 5), room("d0", "DORMITORY", 2)],
    };
    let roster: Vec<OperatorBaseProfile> = [
        "char_103_angel",
        "char_214_kafka",
        "char_4032_provs",
        "char_154_morgan",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .chain(std::iter::once(profile(gd, delphine.as_str())))
    .collect();
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);
    for shift in &rot.shifts {
        for r in shift.rooms.iter().filter(|r| r.room_type == "CONTROL") {
            assert!(
                !r.recommended.contains(&delphine),
                "shift {}: Delphine seated in the CC with no Glasgow member in any team",
                shift.index
            );
        }
    }
}

/// Fiammetta's swap mechanic, as her own skills describe it: "Self-Discipline"
/// recharges her at +2/hr EXCLUSIVELY (no dorm level, aura or ambience helps),
/// and "Communal Suffering" swaps her full bar with the operator assigned into
/// her dormitory. One swap therefore takes 24/2 = 12h to recharge, exactly one
/// login, so she can only hold an operator whose drain won't outrun that
/// cadence: drain <= 2.0/hr sustains, faster does not.
#[test]
fn fiammetta_swap_rate_bounds_who_she_can_sustain() {
    use myrtle_engine_base::rotation::dorms::{manager_can_sustain, manager_swap_rate};
    const FIAMMETTA: &str = "char_300_phenxi";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    let fia = profile(gd, FIAMMETTA);
    let rate = manager_swap_rate(&fia, &registry, gd.building());
    assert!(
        (rate - 2.0).abs() < 1e-9,
        "Self-Discipline parses to +2/hr exclusive self-recovery, got {rate}"
    );
    assert!(
        manager_can_sustain(rate, 1.25),
        "a 1.25/hr drainer survives the 12h between swaps"
    );
    assert!(
        manager_can_sustain(rate, 2.0),
        "2.0/hr spends the bar exactly as she recharges - boundary sustains"
    );
    assert!(
        !manager_can_sustain(rate, 3.0),
        "an Enforcer-class 3.0/hr drainer outruns the swap cadence"
    );
    assert!(
        !manager_can_sustain(0.0, 0.5),
        "no manager, no sustain - rate 0 holds nobody"
    );

    // She has to STAY in the dorm for the swap to work: pinned into the
    // rotation, she holds a dormitory seat in EVERY shift and never appears
    // anywhere else.
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    let (registry2, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let roster = full_roster(gd);
    let building = generic_base();
    let rot = recommend_shift_rotation(
        &roster,
        &building,
        gd.building(),
        &registry2,
        &drains,
        &[(FIAMMETTA.to_string(), "DORMITORY".to_string())],
    );
    for shift in &rot.shifts {
        let in_dorm = shift
            .rooms
            .iter()
            .any(|r| r.room_type == "DORMITORY" && r.recommended.iter().any(|id| id == FIAMMETTA));
        assert!(
            in_dorm,
            "shift {}: Fiammetta must hold her dormitory seat",
            shift.index
        );
        for room in shift.rooms.iter().filter(|r| r.room_type != "DORMITORY") {
            assert!(
                !room.recommended.iter().any(|id| id == FIAMMETTA),
                "shift {}: Fiammetta must never leave the dorm ({})",
                shift.index,
                room.room_type
            );
        }
    }

    // A base with no dormitory cannot host her at all: no pin, no sustain.
    use myrtle_engine_base::rotation::dorms::morale_manager_pin;
    let no_dorms = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)],
    };
    assert!(
        morale_manager_pin(&roster, &no_dorms, gd.building()).is_none(),
        "no dorm, no manager pin - owning her is not enough"
    );
}

/// The rotation ROTATES resters into the dormitories explicitly: every shift
/// emits dorm cells, off-duty workers fill them heaviest-drain-first into the
/// best dorm, unseated dorm-skill holders take permanent seats, and capacity
/// is never exceeded.
#[test]
fn rotation_emits_dorm_cells_with_resters_and_staff() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let roster = full_roster(gd);
    // Mixed dorm levels: the L5 must fill before the L1s (a 2/5/2 shape).
    let mut rooms = vec![room("cc", "CONTROL", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.push(room("d_hi", "DORMITORY", 5));
    rooms.push(room("d_lo1", "DORMITORY", 1));
    rooms.push(room("d_lo2", "DORMITORY", 1));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);

    let mut saw_rester = false;
    for shift in &rot.shifts {
        let dorm_cells: Vec<_> = shift
            .rooms
            .iter()
            .filter(|r| r.room_type == "DORMITORY")
            .collect();
        assert_eq!(dorm_cells.len(), 3, "every dorm appears in every shift");
        // Workers this shift, for the no-double-booking check.
        let working: std::collections::HashSet<&String> = shift
            .rooms
            .iter()
            .filter(|r| r.active && r.room_type != "DORMITORY")
            .flat_map(|r| r.recommended.iter())
            .collect();
        for cell in &dorm_cells {
            // Dorm seats are 5 at every level (gamedata: MaxStationedNum).
            let cap = 5usize;
            assert!(
                cell.recommended.len() <= cap,
                "dorm {} holds {} > {cap} seats",
                cell.slot_id,
                cell.recommended.len()
            );
            for id in &cell.recommended {
                assert!(
                    !working.contains(id),
                    "{id} both works and rests in shift {}",
                    shift.index
                );
                saw_rester = true;
            }
        }
        // The best dorm fills first: the low dorms only hold anyone when the
        // L5 is full.
        let by_slot: std::collections::HashMap<&str, usize> = dorm_cells
            .iter()
            .map(|c| (c.slot_id.as_str(), c.recommended.len()))
            .collect();
        let cap = 5;
        if by_slot.get("d_lo1").copied().unwrap_or(0) > 0
            || by_slot.get("d_lo2").copied().unwrap_or(0) > 0
        {
            assert_eq!(
                by_slot.get("d_hi").copied().unwrap_or(0),
                cap,
                "low-level dorms must not fill before the L5 is full"
            );
        }
    }
    assert!(saw_rester, "off-duty operators appear in dorm cells");
}

/// A base-wide Control-Center recovery aura (Chongyue-type: "+0.05/hr to
/// Operators working in other buildings") stretches PRODUCTION members' swap
/// clocks in the plan: the same working-drain offset the simulator charges,
/// not a dorm-rest bonus.
#[test]
fn cc_base_wide_recovery_stretches_swap_clocks() {
    use myrtle_engine_base::assignment::compute_sustained_assignment;
    use myrtle_engine_base::registry::BuffResolutionStrategy;
    const BODY: &str = "char_102_texas";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    // A real owner of a base-wide CC recovery aura, from the parsed registry.
    let holder = gd
        .building()
        .chars
        .iter()
        .find(|(_, c)| {
            c.buff_char.iter().any(|bc| {
                bc.buff_data.iter().any(|bd| {
                    matches!(
                        registry.get(&bd.buff_id),
                        Some(BuffResolutionStrategy::MoraleModifier {
                            recovery_per_hour,
                            base_wide: true,
                            ..
                        }) if *recovery_per_hour > 0.0
                    )
                })
            })
        })
        .map(|(id, _)| id.clone())
        .expect("some operator owns a base-wide CC recovery aura");

    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("cc", "CONTROL", 5),
            room("tp", "TRADING", 3),
            room("d0", "DORMITORY", 2),
        ],
    };
    let body_hours = |ids: &[&str]| -> f64 {
        let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        let plan =
            compute_sustained_assignment(&roster, &building, gd.building(), &registry, &drains);
        plan.rooms
            .iter()
            .flat_map(|r| r.members.iter())
            .find(|m| m.operator == BODY)
            .map(|m| m.lasts_hours)
            .expect("the body operator rotates in the plan")
    };
    let without = body_hours(&[BODY, "char_140_whitew"]);
    let with_holder = body_hours(&[BODY, "char_140_whitew", holder.as_str()]);
    assert!(
        with_holder > without,
        "the CC's base-wide recovery must stretch a trading member's clock \
         ({with_holder}h vs {without}h)"
    );
}

/// The 3-shift rotation runs on the same economy plan as the optimal view: a
/// consumer's `PoolPayoff` override prices into rotation team selection, and a
/// pinned Control-Center generator (Ling for the Sui economy) holds a Squad-1
/// seat for two of the three shifts, the same realistic uptime perception priced
/// the payoff at (all three would be a 36h stretch no morale bar survives),
/// and is never burned as a production filler.
#[test]
fn rotation_seats_economy_pins_and_credits_payoffs() {
    use myrtle_engine_base::registry::BuffResolutionStrategy;
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    const LING: &str = "char_2023_ling";
    const SHU: &str = "char_2025_shu";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (mut registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    // The solved consumer payoff the pool plans would commit (Shu's factory
    // skill priced at its Sui-economy value).
    registry.insert(
        "manu_prod_spd_bd[300]".to_string(),
        BuffResolutionStrategy::PoolPayoff { pct: 40.0 },
    );
    let roster = full_roster(gd);
    let building = generic_base();
    let pins = vec![(LING.to_string(), "CONTROL".to_string())];
    let rot =
        recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &pins);

    let ling_cc_shifts = rot
        .shifts
        .iter()
        .filter(|s| {
            s.rooms.iter().any(|r| {
                r.room_type == "CONTROL" && r.active && r.recommended.iter().any(|id| id == LING)
            })
        })
        .count();
    assert_eq!(
        ling_cc_shifts, 2,
        "the pinned generator holds the Squad-1 share: 2 of 3 shifts"
    );
    for shift in &rot.shifts {
        // A dormitory cell is REST, not a seat spent: the generator rests off shift there.
        for room in shift
            .rooms
            .iter()
            .filter(|r| r.room_type != "CONTROL" && r.room_type != "DORMITORY")
        {
            assert!(
                !room.recommended.iter().any(|id| id == LING),
                "shift {}: the pinned generator must not be spent as a {} filler",
                shift.index,
                room.room_type
            );
        }
    }
    // The payoff-priced consumer earns a factory seat somewhere in the rotation.
    assert!(
        rot.shifts.iter().any(|s| {
            s.rooms
                .iter()
                .any(|r| r.room_type == "MANUFACTURE" && r.recommended.iter().any(|id| id == SHU))
        }),
        "a +40% PoolPayoff consumer must be worth a factory seat"
    );
}

/// 2/5/2 layout (2 trading posts, 5 factories, 2 power plants): the rotation
/// machinery is layout-generic (groups, gold split, tiling and power squads
/// all derive from the actual rooms), so the second standard endgame layout
/// gets a full 3-shift plan. Pins: the 3-gold/2-EXP split (2 posts' gold
/// demand + 1), the trading login rhythm, alternating power/CC squads, and no
/// operator double-booked within a shift.
#[test]
fn shift_rotation_supports_252_layout() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let mut rooms = vec![
        room("cc", "CONTROL", 5),
        room("hr", "HIRE", 3),
        room("rc", "MEETING", 3),
    ];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..5).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..2).map(|i| room(&format!("p{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let rot = recommend_shift_rotation(
        &full_roster(gd),
        &building,
        gd.building(),
        &registry,
        &drains,
        &[],
    );

    assert_eq!(rot.shifts.len(), 3, "three shifts");
    let cell = |shift: usize, slot: &str| -> &myrtle_engine_base::rotation::shift::ShiftRoom {
        rot.shifts[shift]
            .rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .unwrap_or_else(|| panic!("shift {shift} missing slot {slot}"))
    };

    // Factory split: two L3 posts sell roughly two factories' worth of bars, so
    // the marginal 5th factory goes to EXP (unsold gold is worthless under the
    // coupled gold->trade yield), so the objective picks 2 gold / 3 EXP.
    for shift in 0..3 {
        let mut gold = 0;
        let mut exp = 0;
        for i in 0..5 {
            match cell(shift, &format!("mf{i}")).formula_type.as_deref() {
                Some("F_GOLD") => gold += 1,
                Some("F_EXP") => exp += 1,
                other => panic!("mf{i} shift {shift}: unexpected formula {other:?}"),
            }
        }
        assert_eq!((gold, exp), (2, 3), "shift {shift}: 2 gold / 3 EXP");
    }

    // Trading pair keeps the login rhythm: 24h blocks, one swap per boundary.
    let team_id = |shift: usize, slot: &str| -> String {
        cell(shift, slot)
            .team_id
            .clone()
            .expect("production cells carry a team id")
    };
    // Twenty-one production seats against twenty beds: the 252 runs the 24h
    // blocks. Whichever post holds the shifts-1+2 block is "early"; the other
    // runs 2+3.
    let (early, late) = if team_id(0, "tp0") == team_id(1, "tp0") {
        ("tp0", "tp1")
    } else {
        ("tp1", "tp0")
    };
    assert_eq!(
        team_id(0, early),
        team_id(1, early),
        "{early} works shifts 1+2"
    );
    assert_ne!(
        team_id(1, early),
        team_id(2, early),
        "{early} swaps for shift 3"
    );
    assert_eq!(
        team_id(1, late),
        team_id(2, late),
        "{late} works shifts 2+3"
    );

    // Both plants and the CC stay staffed every shift, cycling two squads.
    for slot in ["p0", "p1", "cc"] {
        let crews: Vec<Vec<String>> = (0..3)
            .map(|k| {
                let mut v = cell(k, slot).recommended.clone();
                v.sort();
                v
            })
            .collect();
        assert!(
            (0..3).all(|k| cell(k, slot).active && !crews[k].is_empty()),
            "{slot} staffed all shifts: {crews:?}"
        );
        let distinct: std::collections::HashSet<&Vec<String>> = crews.iter().collect();
        assert_eq!(distinct.len(), 2, "{slot} cycles two squads: {crews:?}");
    }

    // No operator is double-booked within any shift.
    for (k, shift) in rot.shifts.iter().enumerate() {
        let mut seen = std::collections::HashSet::new();
        for r in shift.rooms.iter().filter(|r| r.active) {
            for op in &r.recommended {
                assert!(
                    seen.insert(op.clone()),
                    "shift {k}: {op} appears in two rooms"
                );
            }
        }
    }
}

/// The recommended rotation must survive its own morale simulation: game-true
/// rates (1/hr drain, level-scaled dorm recovery) over a week of the login
/// rhythm. A 24h work block drains the full bar and a 12h L5-dorm rest refills
/// it, so the recommended plan holds up and the verdict says so.
#[test]
fn recommended_rotation_survives_its_own_morale_sim() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    use myrtle_engine_base::rotation::sustain_sim::{Verdict, simulate_rotation};
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    for factories in [4usize, 5] {
        let mut rooms = vec![
            room("cc", "CONTROL", 5),
            room("hr", "HIRE", 3),
            room("rc", "MEETING", 3),
        ];
        rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
        rooms.extend((0..factories).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
        rooms.extend((0..(7 - factories)).map(|i| room(&format!("p{i}"), "POWER", 3)));
        rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        let roster = full_roster(gd);
        let rot =
            recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);
        let report = simulate_rotation(
            &rot,
            &roster,
            &building,
            gd.building(),
            &registry,
            &drains,
            &std::collections::HashMap::new(),
        );
        // With L5 dorms a 24h block is sustainable at exactly baseline drain
        // (24 drained, 12h x 2.0 recovered), and the planner routes operators
        // whose skills drain faster than baseline into single-shift Squad-2
        // seats, so the recommended plan must fully hold up.
        assert_eq!(
            report.verdict,
            Verdict::HoldsUp,
            "{factories}-factory rotation depletes: {:?} (dorm overflow {})",
            report
                .depleted
                .iter()
                .map(|d| format!("{} @{:.0}h in {}", d.char_id, d.at_hours, d.slot_id))
                .collect::<Vec<_>>(),
            report.dorm_overflow,
        );
        // Dorm pressure, not dorm comfort. The invariant that matters is the verdict above
        // (nobody runs dry over the week); overflow is a pressure gauge, so hold it to a
        // small bound. 4xL5 dorms (20 beds) run short at peak: one bed for the crew the
        // empty-team backfill now staffs (an empty room needs no beds, so the old 0 was
        // partly a bug artefact), one for Fiammetta's permanent dorm seat, and a third
        // since the order-mix model: Shamare's printer out-values Proviso's team, so
        // Fiammetta pins HER 24/7 and her post cycles two fresh bodies every shift.
        assert!(
            report.dorm_overflow <= 3,
            "{factories}-factory dorms are {} beds short at peak",
            report.dorm_overflow
        );
    }
}

/// A production room must never stand dark while operators who could staff it
/// are still free.
///
/// Regression: rotation teams are enumerated with `require_24h_sustain`, which
/// drops every operator whose morale drain outpaces the bar. On a wide base
/// (5 factories + 2 trading posts) the sustainable pool ran dry, the beam left
/// the third team of the trading group empty, and the tiling stood a Trading
/// Post dark for two of three shifts while Lappland, Texas and friends sat
/// unused. An unstaffed production room yields nothing at all; a heavy drainer
/// yields at full rate until their bar empties, and morale is recoverable.
#[test]
fn shift_rotation_never_rests_a_production_room_with_candidates_to_spare() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    // The reported layout: 5 factories, 2 trading posts, 3 power plants, plus the
    // support rooms. 5 factories split gold/EXP gives three production groups,
    // which is what exhausts the shared sustainable pool.
    let mut rooms = vec![
        room("cc", "CONTROL", 5),
        room("hr", "HIRE", 3),
        room("rc", "MEETING", 3),
        room("ws", "WORKSHOP", 3),
        room("tr", "TRAINING", 3),
    ];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..5).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("p{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };

    let rot = recommend_shift_rotation(
        &full_roster(gd),
        &building,
        gd.building(),
        &registry,
        &drains,
        &[],
    );

    let mut dark: Vec<String> = Vec::new();
    for shift in &rot.shifts {
        for r in &shift.rooms {
            if (r.room_type == "TRADING" || r.room_type == "MANUFACTURE")
                && (!r.active || r.recommended.is_empty())
            {
                dark.push(format!("shift {} {}", shift.index, r.slot_id));
            }
        }
    }
    assert!(
        dark.is_empty(),
        "production rooms left unstaffed against a full roster: {dark:?}"
    );

    // And the crews must still be disjoint within a shift; backfilling
    // must not double-book someone already working elsewhere that shift.
    for shift in &rot.shifts {
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for r in shift.rooms.iter().filter(|r| r.active) {
            for op in &r.recommended {
                assert!(
                    seen.insert(op.as_str()),
                    "{op} double-booked in shift {}",
                    shift.index
                );
            }
        }
    }
}

/// The sim's per-facility totals: a room dark two of three shifts logs 2/3 of
/// the horizon as idle and produces only for its staffed blocks; a fully
/// staffed room logs none.
#[test]
fn sim_facility_totals_count_dark_shifts_as_idle() {
    use myrtle_engine_base::rotation::shift::{Shift, ShiftRoom, ShiftRotation};
    use myrtle_engine_base::rotation::sustain_sim::simulate_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let targeted =
        myrtle_engine_base::registry::targeted_morale_effects(&gd.building().buffs, &name_to_char);
    const BODY: &str = "char_103_angel";
    let rotation = ShiftRotation {
        parked: Vec::new(),
        shifts: (1..=3)
            .map(|index| Shift {
                index,
                rooms: vec![ShiftRoom {
                    slot_id: "tp".into(),
                    room_type: "TRADING".into(),
                    formula_type: None,
                    recommended: if index == 1 {
                        vec![BODY.to_string()]
                    } else {
                        Vec::new()
                    },
                    current: Vec::new(),
                    active: index == 1,
                    efficiency: Some(35.0),
                    team_id: None,
                    team_label: None,
                }],
            })
            .collect(),
        sustained: Vec::new(),
        bench: Vec::new(),
    };
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 5)],
    };
    let profiles = vec![profile(gd, BODY)];
    let sim = simulate_rotation(
        &rotation,
        &profiles,
        &building,
        gd.building(),
        &registry,
        &drains,
        &targeted,
    );
    let tp = sim
        .facilities
        .iter()
        .find(|f| f.slot_id == "tp")
        .expect("trading post accounted");
    // Dark two of three shifts: every block whose shift isn't shift 1 is
    // idle. The horizon isn't an exact multiple of the 3-shift cycle, so
    // count blocks rather than assuming a clean 2/3.
    let total_blocks = (sim.horizon_hours / 12.0) as usize;
    let staffed_blocks = total_blocks.div_ceil(3);
    #[allow(clippy::cast_precision_loss)]
    let expected_idle = (total_blocks - staffed_blocks) as f64 * 12.0;
    assert!(
        (tp.idle_hours - expected_idle).abs() < 1e-6,
        "idle {} vs expected {expected_idle}",
        tp.idle_hours
    );
    // Produces only while staffed, never zero, and no phantom other resources.
    assert!(tp.lmd > 0.0);
    assert!(tp.gold.abs() < 1e-9 && tp.exp.abs() < 1e-9);
}

/// Greyy the Lightningbearer's "+1 Power Plant (only affects facility
/// quantity)" counts only in the shifts he works a plant: Weedy (+15% per
/// plant), Eunectes (+10%) and Passenger (+5%) read 30 less in the shift
/// that rests him. The rotation priced every shift against the optimal
/// seats' counts, so a shift with three plain plants still claimed his
/// fourth (44947595: Weedy/Eunectes at 136 in both shifts).
#[test]
fn a_rested_facility_enabler_is_not_counted_in_that_shift() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    const WEEDY: &str = "char_400_weedy";
    const EUNECTES: &str = "char_416_zumama";
    const GREYY2: &str = "char_1027_greyy2";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    // Eight operators: too few for three full factory crews, so the rotation
    // runs the 24h blocks, where Weedy's team works two shifts and Greyy's
    // alternating squad rests one of them.
    let mut roster: Vec<_> = [
        WEEDY,
        EUNECTES,
        GREYY2,
        "char_1011_lava2",
        "char_107_liskam",
        "char_472_pasngr",
        "char_1040_blaze2",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .collect();
    // Enough plain hands that the factory's rest team never borrows Greyy
    // as a filler before the power plan can seat him.
    // Too few hands for three full factory crews: the rotation runs the 24h
    // blocks, where Weedy's team works two shifts and Greyy rests one of them.
    for id in ["char_123_fang"] {
        if gd.building().chars.contains_key(id) {
            roster.push(profile(gd, id));
        }
    }
    let mut rooms = vec![room("mf0", "MANUFACTURE", 3)];
    rooms[0].current_formula = Some("F_GOLD".into());
    rooms.extend((0..3).map(|i| room(&format!("pp{i}"), "POWER", 3)));
    rooms.extend((0..2).map(|i| room(&format!("d{i}"), "DORMITORY", 3)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);
    // Every production cell is priced against ITS shift's seats: the cell's
    // figure equals the live scoring of the same stationing (Greyy's fourth
    // plant counts only in a shift he works one), and the cells differ when
    // his squad rests.
    use myrtle_engine_base::assignment::compute_current_assignment;
    let mut with = Vec::new();
    let mut without = Vec::new();
    for shift in &rot.shifts {
        let mut stationed = building.clone();
        for r in &mut stationed.rooms {
            r.current_operators = shift
                .rooms
                .iter()
                .find(|c| c.slot_id == r.slot_id && c.active)
                .map(|c| c.recommended.clone())
                .unwrap_or_default();
        }
        let live = compute_current_assignment(
            &roster,
            &stationed,
            gd.building(),
            &registry,
            &drains,
            None,
        );
        let factory = shift
            .rooms
            .iter()
            .find(|c| c.slot_id == "mf0" && c.active)
            .expect("the factory works every shift");
        let live_eff = live
            .rooms
            .iter()
            .find(|r| r.slot_id == "mf0")
            .map_or(0.0, |r| r.total_efficiency);
        let cell_eff = factory.efficiency.unwrap_or(0.0);
        assert!(
            (cell_eff - live_eff).abs() < 1e-6,
            "shift {}: the cell reads {cell_eff} but the same stationing scores {live_eff}",
            shift.index
        );
        let greyy_working = shift.rooms.iter().any(|r| {
            r.room_type == "POWER" && r.active && r.recommended.iter().any(|o| o == GREYY2)
        });
        if factory.recommended.iter().any(|o| o == WEEDY) {
            if greyy_working {
                with.push(cell_eff);
            } else {
                without.push(cell_eff);
            }
        }
    }
    if let (Some(with), Some(without)) = (with.first(), without.first()) {
        assert!(
            (with - without - 30.0).abs() < 1e-6,
            "four plants with Greyy, three without: 30 apart, got {with} vs {without}"
        );
    }
}

/// A player who runs saved preset shifts is graded on their average, not on
/// whichever shift was on duty at sync: a rotation's shifts are each weaker
/// than the sustained optimum on purpose, so one shift of a three-shift plan
/// read 88% on 00980819 whose plan as a whole read 94%.
#[test]
fn preset_shifts_are_graded_on_their_average() {
    const TEXAS: &str = "char_102_texas";
    const LAPPLAND: &str = "char_140_whitew";
    const EXUSIAI: &str = "char_103_angel";
    const PLAIN: [&str; 3] = ["char_123_fang", "char_133_mm", "char_502_nblade"];
    let gd = load_game_data();
    let ids = [TEXAS, LAPPLAND, EXUSIAI, PLAIN[0], PLAIN[1], PLAIN[2]];
    let roster: Vec<RosterEntry> = ids.iter().map(|id| roster_entry(id)).collect();
    let inst_of = |id: &str| -> i64 { ids.iter().position(|x| *x == id).unwrap() as i64 + 1 };
    let building = |stationed: &[&str], presets: Option<(&[&str], &[&str])>| {
        let mut chars = serde_json::Map::new();
        for id in ids {
            chars.insert(inst_of(id).to_string(), serde_json::json!({ "charId": id }));
        }
        let insts = |crew: &[&str]| -> Vec<i64> { crew.iter().map(|id| inst_of(id)).collect() };
        let mut tp = serde_json::json!({});
        if let Some((a, b)) = presets {
            tp["presetQueue"] = serde_json::json!([insts(a), insts(b)]);
        }
        serde_json::json!({
            "chars": chars,
            "rooms": { "TRADING": { "tp0": tp } },
            "roomSlots": {
                "tp0": { "roomId": "TRADING", "level": 3, "state": 2, "charInstIds": insts(stationed) },
                "d0": { "roomId": "DORMITORY", "level": 5, "state": 2 },
            }
        })
    };
    let strong = [TEXAS, LAPPLAND, EXUSIAI];
    let weak = PLAIN;
    let g_strong = grade_base(&roster, Some(&building(&strong, None)), gd);
    let g_weak = grade_base(&roster, Some(&building(&weak, None)), gd);
    let g_both = grade_base(
        &roster,
        Some(&building(&strong, Some((&strong, &weak)))),
        gd,
    );
    assert!(
        g_weak.utilization < g_both.utilization && g_both.utilization < g_strong.utilization,
        "presets grade between their shifts: weak {:.3} < both {:.3} < strong {:.3}",
        g_weak.utilization,
        g_both.utilization,
        g_strong.utilization
    );
}

/// The rotation prices each trading post at ITS level and picks the posts'
/// teams by the coupled yield the peak search uses: on a gold-starved base
/// with a level-3 and a level-2 post, Proviso (whose "+2 gold below 4"
/// pays on every order a level-2 post draws) sells at the level-2 post and
/// the Shamare squad at the level-3 one, in every shift that fields them.
/// Planned as one group at the lower level and ranked by raw score, shift 1
/// fielded Texas/Lappland at level 3 and the Laterano pair at level 2 with
/// Proviso and Shamare benched (00980819, 2026-09-21).
#[test]
fn the_rotation_sells_proviso_at_the_lower_post_and_shamare_at_the_higher() {
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    const SHAMARE: &str = "char_254_vodfox";
    const TEQUILA: &str = "char_486_takila";
    const BIBEAK: &str = "char_252_bibeak";
    const PROVISO: &str = "char_4032_provs";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let mut roster: Vec<_> = [
        SHAMARE,
        TEQUILA,
        BIBEAK,
        PROVISO,
        "char_102_texas",
        "char_140_whitew",
        "char_103_angel",
        "char_4193_lemuen",
        "char_4088_hodrer",
        "char_4063_quartz",
        "char_427_vigil",
        "char_4046_ebnhlz",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    assert!(
        roster.len() >= 10,
        "the traders are in the game data: {}",
        roster.len()
    );
    let hands: Vec<_> = [
        "char_123_fang",
        "char_133_mm",
        "char_502_nblade",
        "char_124_kroos",
        "char_211_adnach",
        "char_212_ansel",
        "char_210_stward",
        "char_120_hibisc",
        "char_121_lava",
        "char_36_forget",
        "char_4212_nasti",
        "char_4048_doroth",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    roster.extend(hands);
    // Three gold factories outsell two posts: with gold to spare, Proviso's
    // bonus bars are sold and her value shape decides the post, as on the
    // reporter's base.
    let mut rooms = vec![room("tp3", "TRADING", 3), room("tp2", "TRADING", 2)];
    for i in 0..3 {
        let mut r = room(&format!("mf{i}"), "MANUFACTURE", 3);
        r.current_formula = Some("F_GOLD".into());
        rooms.push(r);
    }
    rooms.extend((0..2).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);
    let posts: Vec<(usize, String, Vec<String>)> = rot
        .shifts
        .iter()
        .flat_map(|s| {
            s.rooms
                .iter()
                .filter(|r| r.room_type == "TRADING")
                .map(move |r| (s.index, r.slot_id.clone(), r.recommended.clone()))
        })
        .collect();
    let at = |slot: &str, id: &str| {
        posts
            .iter()
            .any(|(_, s, crew)| s == slot && crew.iter().any(|o| o == id))
    };
    assert!(
        !at("tp3", PROVISO),
        "Proviso never sells at the level-3 post: {posts:?}"
    );
    assert!(
        !at("tp2", SHAMARE),
        "the Shamare squad never sits at the level-2 post: {posts:?}"
    );
    assert!(
        at("tp2", PROVISO) && at("tp3", SHAMARE),
        "both sell where they pay most: {posts:?}"
    );
}
