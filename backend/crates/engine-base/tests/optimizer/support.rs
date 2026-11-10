use super::*;

#[test]
fn reception_room_staffs_caper_during_exchange() {
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let building = meeting_room(3);
    // Caper's clue-search skill is 0 outside exchange (its % lives in the description, parsed under
    // the permanent-exchange assumption); without that, he'd be dropped for the two +25% operators.
    let roster: Vec<_> = [CAPER, VIGIL, RED]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let m = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "MEETING")
        .expect("a reception room");
    assert!(
        m.operators.iter().any(|o| o == CAPER),
        "Caper should be staffed in the Reception Room during exchange, got {:?}",
        m.operators
    );
}

#[test]
fn reception_total_includes_rarity_elite_and_level() {
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let building = meeting_room(3);
    let roster: Vec<_> = [VIGIL, RED].iter().map(|id| profile(gd, id)).collect();
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let m = asn.rooms.iter().find(|r| r.room_type == "MEETING").unwrap();
    // Raw clue skills are 25 + 25 = 50; the total also folds in per-operator rarity + elite (E2
    // +16 each) and the +11 RR-level bonus, so it lands well above the raw skills.
    assert!(
        m.total_efficiency > 60.0,
        "reception total {} should include rarity/elite/level ambience bonuses",
        m.total_efficiency
    );
}

#[test]
fn reception_applies_the_solo_skill_when_an_operator_works_alone() {
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let building = meeting_room(3);
    // Kazemaru clue-searches at +15% paired but +35% when he's the room's sole worker (no morale
    // cost). With no partner available he's staffed alone, and the room's total reflects his higher
    // SOLO skill, not the paired one.
    const KAZEMARU: &str = "char_4016_kazema";
    let roster = vec![profile(gd, KAZEMARU)];
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let m = asn.rooms.iter().find(|r| r.room_type == "MEETING").unwrap();
    assert_eq!(
        m.operators,
        vec![KAZEMARU.to_string()],
        "the lone operator is staffed"
    );
    // Solo skill 35 + rarity (5★ +4) + elite (E2 +16) + RR-level (L3 +11) = 66; the paired skill
    // (15) would only reach ~46, so a total well above 50 proves the solo skill was used.
    assert!(
        m.total_efficiency > 50.0,
        "the room should use Kazemaru's +35% solo skill, got {}",
        m.total_efficiency
    );
}

#[test]
fn ines_reception_skill_uses_the_sustained_time_ceiling() {
    let gd = load_game_data();
    let (registry, _drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    // Ines's "Shadow Gathering": clue search +20%, then +2%/hr up to a maximum of 30%. Under
    // sustained operation she holds the 30% ceiling; the `Efficiency` field only reports the 20%
    // starting value, which would undercredit her.
    let value = match registry.get("meet_spd_hast[000]") {
        Some(BuffResolutionStrategy::NonProduction { value }) => *value,
        _ => panic!("expected a NonProduction reception value for Ines"),
    };
    assert!(
        (value - 30.0).abs() < 1e-9,
        "Ines should be valued at her 30% sustained ceiling, got {value}"
    );
}

#[test]
fn enforcer_loses_to_chen_in_reception_because_he_burns_out() {
    // Enforcer's reception skill is higher (+35% vs Ch'en's/FEater's +25%), but his "+2 Morale
    // consumed per hour" burns him out in ~5h of a 12h shift, so his prorated clue output drops
    // below both sustainable operators. The 2-seat Reception Room fills with Ch'en + FEater and
    // leaves Enforcer out; without the morale penalty he'd be the top pick.
    const ENFORCER: &str = "char_4036_forcer";
    const CHEN: &str = "char_010_chen";
    const FEATER: &str = "char_109_fmout"; // +25% reception, no morale cost
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("rc", "MEETING", 3)],
    };
    let roster: Vec<_> = [ENFORCER, CHEN, FEATER]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let m = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "MEETING")
        .expect("a reception room");
    assert!(
        m.operators.iter().any(|o| o == CHEN),
        "the sustainable Ch'en should be staffed, got {:?}",
        m.operators
    );
    assert!(
        !m.operators.iter().any(|o| o == ENFORCER),
        "Enforcer should be dropped - he burns out before the shift ends, got {:?}",
        m.operators
    );
}

// ─── DTO: the rest shift never keeps a main-team operator working (the "24/7" bug) ────────────

#[test]
fn reception_does_not_credit_a_co_op_conditional_without_the_partner() {
    // Vulpisfoglia's +30% only applies "together with Suzuran"; without Suzuran in the room she's a
    // +20% operator and should lose her slot to unconditional higher-skill operators, not be
    // over-credited the conditional and beat them.
    const VULPIS: &str = "char_4026_vulpis"; // +20%, plus +30% ONLY together with Suzuran
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let building = meeting_room(3); // capacity 2
    let roster: Vec<_> = [VULPIS, RED, VIGIL]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let crew: Vec<&str> = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "MEETING")
        .expect("a reception room")
        .operators
        .iter()
        .map(String::as_str)
        .collect();
    assert!(
        !crew.contains(&VULPIS),
        "Vulpisfoglia's Suzuran-conditional +30% must not be credited without Suzuran: {crew:?}"
    );
    assert!(
        crew.contains(&RED),
        "Projekt Red (+25%, unconditional) should be staffed: {crew:?}"
    );
}

/// Eunectes' "+2 Power Plant count" needs her in the Control Center AND
/// Lancet-2 in a Power Plant (user-verified 2026-09-10: a 1/5/3 read six
/// plants with her in the factory). Weedy's Bionic Seadragon reads +15 per
/// plant: 45 on three real plants, 75 only when both seats are met.
#[test]
fn eunectes_plant_count_needs_her_seat_and_lancet_in_a_plant() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const WEEDY: &str = "char_400_weedy";
    const EUNECTES: &str = "char_416_zumama";
    const LANCET: &str = "char_285_medic2";
    const KROOS: &str = "char_124_kroos";
    let roster: Vec<_> = [WEEDY, EUNECTES, LANCET, KROOS]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let weedy_eff = |eunectes_room: &str, lancet_room: &str| {
        let mut rooms = vec![
            room("cc", "CONTROL", 5),
            room("mf", "MANUFACTURE", 3),
            room("d0", "DORMITORY", 5),
        ];
        rooms.extend((0..3).map(|i| room(&format!("pp{i}"), "POWER", 3)));
        rooms
            .iter_mut()
            .find(|r| r.slot_id == "mf")
            .unwrap()
            .current_operators
            .push(WEEDY.into());
        rooms
            .iter_mut()
            .find(|r| r.slot_id == "mf")
            .unwrap()
            .current_formula = Some("F_GOLD".into());
        for (id, target) in [(EUNECTES, eunectes_room), (LANCET, lancet_room)] {
            rooms
                .iter_mut()
                .find(|r| r.slot_id == target)
                .unwrap()
                .current_operators
                .push(id.into());
        }
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
        (weedy_eff("cc", "pp0") - 75.0).abs() < 1e-9,
        "both seats met: 5 plants: {}",
        weedy_eff("cc", "pp0")
    );
    assert!(
        (weedy_eff("d0", "pp0") - 45.0).abs() < 1e-9,
        "Eunectes resting: 3 plants: {}",
        weedy_eff("d0", "pp0")
    );
    assert!(
        (weedy_eff("cc", "d0") - 45.0).abs() < 1e-9,
        "Lancet-2 resting: 3 plants: {}",
        weedy_eff("cc", "d0")
    );
}

/// Vigil's New City Trade reads the base's Reception Room level: +25 base,
/// +5 per level, capped at 40, so a level-3 room gives 40, none gives 25.
#[test]
fn vigil_reads_the_reception_room_level() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const VIGIL: &str = "char_427_vigil";
    let roster: Vec<_> = [VIGIL].iter().map(|id| profile(gd, id)).collect();
    let eff = |reception: Option<i32>| {
        let mut rooms = vec![room("tp", "TRADING", 3)];
        rooms[0].current_operators = vec![VIGIL.into()];
        if let Some(lv) = reception {
            rooms.push(room("rr", "MEETING", lv));
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
        (eff(Some(3)) - 40.0).abs() < 1e-9,
        "level 3: {}",
        eff(Some(3))
    );
    assert!(
        (eff(Some(1)) - 30.0).abs() < 1e-9,
        "level 1: {}",
        eff(Some(1))
    );
    assert!((eff(None) - 25.0).abs() < 1e-9, "no room: {}", eff(None));
}

/// Viy's Artisan reads the base's Training Room level: +10 per level, capped at
/// 30, so a level-3 room gives 30, level 1 gives 10, none gives 0.
#[test]
fn viy_reads_the_training_room_level() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const VIY: &str = "char_4226_veen";
    let roster: Vec<_> = [VIY].iter().map(|id| profile(gd, id)).collect();
    let eff = |training: Option<i32>| {
        let mut rooms = vec![room("f", "MANUFACTURE", 3)];
        rooms[0].current_operators = vec![VIY.into()];
        if let Some(lv) = training {
            rooms.push(room("tr", "TRAINING", lv));
        }
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "f")
            .map_or(0.0, |r| r.total_efficiency)
    };
    assert!(
        (eff(Some(3)) - 30.0).abs() < 1e-9,
        "level 3: {}",
        eff(Some(3))
    );
    assert!(
        (eff(Some(1)) - 10.0).abs() < 1e-9,
        "level 1: {}",
        eff(Some(1))
    );
    assert!(eff(None).abs() < 1e-9, "no room: {}", eff(None));
}

#[test]
fn dead_lancet_unlocks_both_plant_counts() {
    use myrtle_engine_base::assignment::compute_live_assignment;
    use std::collections::HashMap;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const WEEDY: &str = "char_400_weedy";
    const EUNECTES: &str = "char_416_zumama";
    const LANCET: &str = "char_285_medic2";
    const GREYY: &str = "char_1027_greyy2";
    let roster: Vec<_> = [WEEDY, EUNECTES, LANCET, GREYY]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let mut rooms = vec![room("cc", "CONTROL", 5), room("mf", "MANUFACTURE", 3)];
    rooms.extend((0..3).map(|i| room(&format!("pp{i}"), "POWER", 3)));
    rooms[0].current_operators = vec![EUNECTES.into()];
    rooms[1].current_operators = vec![WEEDY.into()];
    rooms[1].current_formula = Some("F_EXP".into());
    rooms[2].current_operators = vec![GREYY.into()];
    rooms[3].current_operators = vec![LANCET.into()];
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let weedy = |lancet_morale: f64| {
        let live = HashMap::from([(LANCET.to_string(), lancet_morale)]);
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
        .find(|r| r.slot_id == "mf")
        .map_or(0.0, |r| r.total_efficiency)
    };
    assert!(
        (weedy(0.0) - 90.0).abs() < 1e-9,
        "dead Lancet: six plants: {}",
        weedy(0.0)
    );
    assert!(
        (weedy(24.0) - 75.0).abs() < 1e-9,
        "live Lancet blocks Greyy: {}",
        weedy(24.0)
    );
}

/// Whisperain's fragments follow the Office level (0 at HR1, 10 at HR2, 20
/// at HR3), converted 1:1 into Perception Information for Rosmontis.
#[test]
fn whisperain_fragments_follow_the_office_level() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const ROSMONTIS: &str = "char_391_rosmon";
    const WHISPERAIN: &str = "char_436_whispr";
    let roster: Vec<_> = [ROSMONTIS, WHISPERAIN]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let eff = |office_level: i32| {
        let mut rooms = vec![
            room("mf", "MANUFACTURE", 3),
            room("hr", "HIRE", office_level),
        ];
        rooms[0].current_operators = vec![ROSMONTIS.into()];
        rooms[0].current_formula = Some("F_EXP".into());
        rooms[1].current_operators = vec![WHISPERAIN.into()];
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
    assert!((eff(1) - 0.0).abs() < 1e-9, "HR1: {}", eff(1));
    assert!((eff(2) - 10.0).abs() < 1e-9, "HR2: {}", eff(2));
    assert!((eff(3) - 20.0).abs() < 1e-9, "HR3: {}", eff(3));
}

/// The reception row carries the full model (the feedback sheet's RR tab):
/// Kazemaru E2 alone = 50 skill + 5 innate + 4 (5-star) + 16 (E2) = 75;
/// beside a live Vigil E2 the solo gate drops and Vigil adds 25 + 5 + 16 = 86;
/// beside a DEAD Vigil the solo gate holds and Vigil keeps her ambience = 96.
#[test]
fn reception_row_carries_ambience_innate_and_dead_seat_rules() {
    use myrtle_engine_base::assignment::compute_live_assignment;
    use std::collections::HashMap;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const KAZEMARU: &str = "char_4016_kazema";
    const VIGIL: &str = "char_427_vigil";
    let roster: Vec<_> = [KAZEMARU, VIGIL].iter().map(|id| profile(gd, id)).collect();
    let clue = |crew: &[&str], live: HashMap<String, f64>| {
        let mut rooms = vec![room("rr", "MEETING", 3)];
        rooms[0].current_operators = crew.iter().map(|s| (*s).to_string()).collect();
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
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
    assert!(
        (clue(&[KAZEMARU], HashMap::new()) - 75.0).abs() < 1e-9,
        "alone: {}",
        clue(&[KAZEMARU], HashMap::new())
    );
    assert!(
        (clue(&[KAZEMARU, VIGIL], HashMap::new()) - 86.0).abs() < 1e-9,
        "live Vigil: {}",
        clue(&[KAZEMARU, VIGIL], HashMap::new())
    );
    let dead = HashMap::from([(VIGIL.to_string(), 0.0)]);
    assert!(
        (clue(&[KAZEMARU, VIGIL], dead.clone()) - 96.0).abs() < 1e-9,
        "dead Vigil: {}",
        clue(&[KAZEMARU, VIGIL], dead)
    );
}

#[test]
fn pinned_support_seats_report_their_own_figure() {
    // A support seat pinned for its pool grant still reports the crew's own
    // room figure: Whisperain pinned into the Office shows her +20 HR speed,
    // not 0 (live report 2026-09-14).
    use myrtle_engine_base::assignment::compute_optimal_assignment_with_pins;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const WHISPERAIN: &str = "char_436_whispr";
    const ROSMONTIS: &str = "char_391_rosmon";
    let roster: Vec<_> = [ROSMONTIS, WHISPERAIN, EXUSIAI]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("mf", "MANUFACTURE", 3), room("hr", "HIRE", 3)],
    };
    let pins = vec![(WHISPERAIN.to_string(), "HIRE".to_string())];
    let asn = compute_optimal_assignment_with_pins(
        &roster,
        &building,
        gd.building(),
        &registry,
        &drains,
        &pins,
    );
    let hr = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "HIRE")
        .expect("the pinned Office row");
    assert!(
        hr.operators.iter().any(|o| o == WHISPERAIN),
        "Whisperain seated: {:?}",
        hr.operators
    );
    assert!(
        (hr.total_efficiency - 20.0).abs() < 1e-9,
        "her HR speed shows: {}",
        hr.total_efficiency
    );
}

/// A Control-Center bench seat prefers an operator who brings the room
/// something over a blank: Blaze the Igniting Spark (a +5% training skill
/// in the Control Center, drone skills elsewhere) outranks a level-1 E0 Lee
/// whose only skill needs Aak, even though Lee costs nothing elsewhere
/// (user feedback 2026-09-15: "he is E0 so he is useless").
#[test]
fn bench_seat_prefers_a_valued_operator_over_a_blank() {
    use myrtle_engine_base::assignment::fill_remaining_slots;
    use std::collections::HashSet;
    let gd = load_game_data();
    let (registry, _) = build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const BLAZE2: &str = "char_1040_blaze2";
    const LEE: &str = "char_322_lmlee";
    let mut lee = profile(gd, LEE);
    // E0 level 1: only his pair skill with Aak is unlocked.
    lee.available_buffs
        .retain(|b| b.starts_with("control_allCost_condChar"));
    let roster = vec![lee, profile(gd, BLAZE2)];
    let mut slots: Vec<String> = Vec::new();
    let mut assigned: HashSet<String> = HashSet::new();
    let added = fill_remaining_slots(
        &mut slots,
        1,
        "CONTROL",
        &roster,
        gd.building(),
        &registry,
        &mut assigned,
    );
    assert_eq!(
        added,
        vec![BLAZE2.to_string()],
        "the valued operator takes the seat: {added:?}"
    );
}

/// A strongest-only Control-Center skill is worth only what it adds beyond
/// the seated crew: Lee's Worldly Insight (clue +25%, "only the strongest
/// effect of this type") beside a crew already covering clue +25% buys no
/// bench seat.
#[test]
fn a_covered_strongest_only_clue_skill_buys_no_bench_seat() {
    use myrtle_engine_base::assignment::{cc_seated_coverage, cc_spare_seat_value};
    use myrtle_engine_base::scoring::clause::NonProdKind;
    use std::collections::HashMap;
    const LEE: &str = "char_322_lmlee";
    let gd = load_game_data();
    let (registry, _drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let lee = profile(gd, LEE);
    let uncovered = cc_spare_seat_value(&lee, gd.building(), &registry, &HashMap::new());
    assert!(
        uncovered > 0.0,
        "Lee's clue skill has bench value on an empty crew: {uncovered}"
    );
    let mut covered = HashMap::new();
    covered.insert(NonProdKind::ClueSearch, 25.0);
    assert!(
        cc_spare_seat_value(&lee, gd.building(), &registry, &covered).abs() < 1e-9,
        "covered by an equal or stronger clue skill, Lee adds nothing"
    );
    // Lee himself seated sets the bar his own kind of skill is measured against.
    let seated = cc_seated_coverage(
        &[LEE.to_string()],
        std::slice::from_ref(&lee),
        gd.building(),
        &registry,
    );
    assert!(
        seated
            .get(&NonProdKind::ClueSearch)
            .is_some_and(|v| (*v - 25.0).abs() < 1e-9),
        "seated coverage reads Lee's clue +25: {seated:?}"
    );
}

#[test]
fn drone_capacity_skill_scales_with_the_bases_actual_capacity() {
    // Greyy the Lightningbearer's "+1% Drone recovery rate for every 10 max Drone capacity
    // (Max +25%)" reads the base's REAL drone capacity. A fixture names no floorplan
    // slot, so it reads as the fully built floorplan: 235 drones (`InitMaxLabor` plus
    // every slot's `ProvideLabor`), +23.5%, under the +25% cap, not AT it.
    use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
    const GREYY_ALTER: &str = "char_1027_greyy2";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let mut rooms = vec![room("d0", "DORMITORY", 5)];
    rooms.extend((0..3).map(|i| room(&format!("p{i}"), "POWER", 3)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let roster = vec![profile(gd, GREYY_ALTER)];
    let rot = recommend_shift_rotation(&roster, &building, gd.building(), &registry, &drains, &[]);
    let cell = rot.shifts[0]
        .rooms
        .iter()
        .find(|r| r.room_type == "POWER" && r.recommended.iter().any(|o| o == GREYY_ALTER))
        .expect("Greyy Alter staffs a plant");
    let eff = cell.efficiency.expect("power cells carry efficiency");
    assert!(
        (eff - 23.5).abs() < 0.1,
        "3x L3 plants = 235 drones -> +23.5% (not the +25% cap), got {eff}"
    );
}

#[test]
fn a_synced_drone_capacity_overrides_the_floorplan_rule() {
    // The capacity used to be 100 plus a per-Power-Plant table, which read a base at
    // 235 drones as 190 (+19%, report 2026-10-23). The synced `status.labor.maxValue`
    // is the figure the game shows; when the base carries it, the skill reads it.
    use myrtle_engine_base::assignment::drone_capacity;
    let gd = load_game_data();
    let mut building = UserBuilding {
        rooms: vec![room("p0", "POWER", 3)],
        drone_capacity: None,
    };
    assert_eq!(
        drone_capacity(&building, gd.building()),
        235,
        "fixture = full floorplan"
    );
    building.drone_capacity = Some(190);
    assert_eq!(drone_capacity(&building, gd.building()), 190);
    // A base that names real slots reads the slots it has built.
    let real = UserBuilding {
        rooms: vec![room("slot_34", "CONTROL", 5), room("slot_7", "POWER", 3)],
        drone_capacity: None,
    };
    let built = drone_capacity(&real, gd.building());
    assert!(
        built > 0 && built < 235,
        "two built slots read between 0 and 235, got {built}"
    );
}

/// POWER value is in the objective: a Power Plant operator's drone-recovery %
/// converts to LMD/day through gamedata's Labor economy (`LaborRecoverTime` 360 s
/// => 240 drones/day; one drone = `ManufactReduceTimeUnit` 180 s of gold-factory
/// progress = 500 x 180/4320 LMD), so +20% recovery = exactly 1000 LMD/day.
/// This lets the optimizer price seating a drone specialist against displacing
/// one (the Alanna robot-displacement trade) instead of treating POWER as free.
#[test]
fn power_drone_recovery_is_priced_in_the_objective() {
    use myrtle_engine_base::assignment::assignment_value;
    use myrtle_engine_base::types::RoomAssignment;
    let power = |eff: f64| RoomAssignment {
        slot_id: "slot_p".into(),
        room_type: "POWER".into(),
        level: 3,
        total_efficiency: eff,
        ..Default::default()
    };
    let without = assignment_value(&[]);
    let with = assignment_value(&[power(20.0)]);
    assert!(
        ((with - without) - 1000.0).abs() < 1e-9,
        "+20% drone recovery must be worth exactly 1000 LMD/day, got {}",
        with - without
    );
    // Two plants stack linearly: no coupling, unlike the gold->trade loop.
    let both = assignment_value(&[power(20.0), power(23.5)]);
    assert!(((both - without) - 2175.0).abs() < 1e-9);
}

/// Trainer hints rank by the declared class: class-agnostic skills count for
/// everyone, class-specific ones only for their class, composition-scaled
/// texts are skipped rather than guessed.
#[test]
fn trainer_hints_respect_the_declared_class() {
    // Exercised through the public evaluate path indirectly; here just pin the
    // gamedata phrasings the parser relies on.
    let gd = load_game_data();
    let mut agnostic = 0;
    let mut specific = 0;
    let mut scaled = 0;
    for buff in gd.building().buffs.values() {
        if buff.room_type != "TRAINING" {
            continue;
        }
        let d = &buff.description;
        if !d.contains("Specialization training speed") {
            continue;
        }
        if d.contains("for each") {
            scaled += 1;
        } else if [
            "Vanguard",
            "Guard",
            "Defender",
            "Sniper",
            "Caster",
            "Medic",
            "Supporter",
            "Specialist",
        ]
        .iter()
        .any(|c| d.contains(c))
        {
            specific += 1;
        } else {
            agnostic += 1;
        }
    }
    assert!(agnostic > 0, "class-agnostic trainer skills exist");
    assert!(specific > 0, "class-specific trainer skills exist");
    assert!(
        scaled > 0,
        "composition-scaled trainer skills exist (skipped)"
    );
}

/// Secondary affiliations: a `SubPower` NATION counts (Texas is Siracusa for
/// Umiri, community-verified) but a `SubPower` GROUP does not (Vina Victoria
/// carries {glasgow} there, and the game's "Glasgow Gang Operator" checks
/// don't count her, verified in-game 2026-09-07).
#[test]
fn subpower_counts_nations_but_not_groups() {
    let gd = load_game_data();
    let texas = profile(gd, "char_102_texas");
    assert!(
        texas.faction_tags.iter().any(|t| t == "siracusa"),
        "Texas: {:?}",
        texas.faction_tags
    );
    let vina = profile(gd, "char_1019_siege2");
    assert!(
        !vina.match_tags.iter().any(|t| t == "glasgow"),
        "Vina Victoria is not Glasgow: {:?}",
        vina.match_tags
    );
    let siege = profile(gd, "char_112_siege");
    assert!(
        siege.match_tags.iter().any(|t| t == "glasgow"),
        "Siege is: {:?}",
        siege.match_tags
    );
}

/// When a Worldly Plight team is recommended, the Office seat goes to
/// Mulberry ("for every Recruit slot, Worldly Plight +10": +20 at a level-3
/// Office), not to a plain HR-speed operator: the points feed Mr. Nothing's
/// +1% per point and Shu's +1% per 3, which outweigh Provence's HR speed.
#[test]
fn a_worldly_plight_team_seats_mulberry_in_the_office() {
    const NOTHING: &str = "char_455_nothin";
    const SHU: &str = "char_2025_shu";
    const JIEYUN: &str = "char_4078_bdhkgt";
    const MULBERRY: &str = "char_473_mberry";
    const PROVENCE: &str = "char_145_prove";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let mut roster: Vec<_> = [NOTHING, SHU, JIEYUN, MULBERRY, PROVENCE, "char_2024_chyue"]
        .iter()
        .filter(|id| gd.building().chars.contains_key(**id))
        .map(|id| profile(gd, id))
        .collect();
    for id in [
        "char_123_fang",
        "char_133_mm",
        "char_502_nblade",
        "char_124_kroos",
        "char_120_hibisc",
        "char_211_adnach",
        "char_212_ansel",
        "char_36_forget",
        "char_121_lava",
        "char_210_stward",
    ] {
        if gd.building().chars.contains_key(id) {
            roster.push(profile(gd, id));
        }
    }
    let mut rooms = vec![
        room("cc", "CONTROL", 5),
        room("tp", "TRADING", 3),
        room("hr", "HIRE", 3),
    ];
    rooms.extend((0..2).map(|i| {
        let mut r = room(&format!("mf{i}"), "MANUFACTURE", 3);
        r.current_formula = Some("F_GOLD".into());
        r
    }));
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
    let seated = |id: &str| {
        accepted
            .optimal
            .rooms
            .iter()
            .any(|r| r.operators.iter().any(|o| o == id))
    };
    assert!(seated(NOTHING), "Mr. Nothing sells at the post");
    let office = accepted
        .optimal
        .rooms
        .iter()
        .find(|r| r.room_type == "HIRE")
        .expect("the Office is staffed");
    assert_eq!(
        office.operators,
        vec![MULBERRY.to_string()],
        "Mulberry feeds the Worldly Plight team from the Office"
    );
}
