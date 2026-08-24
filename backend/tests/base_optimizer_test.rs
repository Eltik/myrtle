//! End-to-end checks for the infrastructure (base) optimizer against real game
//! data, using generic constructed bases and rosters (no captured user). Focus:
//! named-teammate conditional synergies, Control Center stacking and faction
//! buffs, the staggered rotation, and the high-level grade.

// Test-local `const CHAR: &str = "..."` declarations next to their assertions, and small numeric
// casts on counts, are intentional here - pedantic/nursery lints not worth reshaping test code over.
#![allow(
    clippy::items_after_statements,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines
)]

mod common;

use backend::core::gamedata::types::GameData;
use backend::core::grade::base::assignment::{
    compute_optimal_assignment, compute_optimal_assignment_with_pins, compute_sustained_assignment,
    team_value,
};
use backend::core::grade::base::buff_registry::{
    BuffResolutionStrategy, build_name_to_char, build_registry,
};
use backend::core::grade::base::score::grade_base;
use backend::core::grade::base::types::{OperatorBaseProfile, UserBuilding, UserRoom};
use backend::database::models::roster::RosterEntry;
use common::{load_game_data, max_stationed};

/// A `UserRoom` of `room_type` at `level` with a stable slot id.
fn room(slot: &str, room_type: &str, level: i32) -> UserRoom {
    UserRoom {
        slot_id: slot.into(),
        room_type: room_type.into(),
        level,
        ..Default::default()
    }
}

/// A realistic base layout (Control Center + 2 trading posts + 4 factories + 4
/// dormitories) - used instead of any specific captured user.
fn generic_base() -> UserBuilding {
    let mut rooms = vec![room("cc", "CONTROL", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    UserBuilding { rooms }
}

/// Every operator with base skills, at E2-max - a deep, user-independent roster
/// (mirrors the "best possible" baseline the grader builds).
fn full_roster(gd: &GameData) -> Vec<OperatorBaseProfile> {
    gd.building
        .chars
        .keys()
        .filter(|id| id.starts_with("char_"))
        .map(|id| profile(gd, id))
        .collect()
}

/// A minimal E2-max `RosterEntry` for an operator (for the high-level `grade_base`).
fn roster_entry(operator_id: &str) -> RosterEntry {
    RosterEntry {
        user_id: uuid::Uuid::nil(),
        operator_id: operator_id.into(),
        elite: 2,
        level: 90,
        exp: 0,
        potential: 0,
        skill_level: 7,
        favor_point: 0,
        skin_id: None,
        default_skill: None,
        voice_lan: None,
        current_equip: None,
        current_tmpl: None,
        obtained_at: None,
        masteries: serde_json::json!([]),
        modules: serde_json::json!([]),
    }
}

/// Build a fully-elited (E2-max) base profile for an operator by taking the
/// highest-unlock buff in every skill slot.
fn profile(gd: &GameData, char_id: &str) -> OperatorBaseProfile {
    let bc = gd
        .building
        .chars
        .get(char_id)
        .unwrap_or_else(|| panic!("{char_id} missing from building data"));
    let available_buffs: Vec<String> = bc
        .buff_char
        .iter()
        .filter_map(|slot| slot.buff_data.last().map(|e| e.buff_id.clone()))
        .collect();
    let faction_tags = gd
        .operators
        .get(char_id)
        .map(backend::core::grade::base::buff_registry::faction_tags_of)
        .unwrap_or_default();
    let match_tags = backend::core::grade::base::types::compute_match_tags(
        &faction_tags,
        &available_buffs,
        &gd.building,
    );
    OperatorBaseProfile {
        char_id: char_id.to_string(),
        available_buffs,
        faction_tags,
        match_tags,
        rarity: gd
            .operators
            .get(char_id)
            .map_or(0, |o| o.rarity.to_star_int()),
        elite: 2,
    }
}

fn trading_post(level: i32) -> UserBuilding {
    single_room("TRADING", level)
}

fn single_room(room_type: &str, level: i32) -> UserBuilding {
    UserBuilding {
        rooms: vec![UserRoom {
            slot_id: "slot_1".into(),
            room_type: room_type.into(),
            level,
            ..Default::default()
        }],
    }
}

fn trading_efficiency(gd: &GameData, roster: &[OperatorBaseProfile]) -> f64 {
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let asn =
        compute_optimal_assignment(roster, &trading_post(3), &gd.building, &registry, &drains);
    asn.rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .map_or(0.0, |r| r.total_efficiency)
}

/// The trading post's potential LMD/day - reflects BOTH order speed and order
/// value (so Proviso, who adds value not speed, shows up here).
fn trading_lmd(gd: &GameData, roster: &[OperatorBaseProfile]) -> f64 {
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let asn =
        compute_optimal_assignment(roster, &trading_post(3), &gd.building, &registry, &drains);
    asn.rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .map_or(0.0, |r| {
            backend::core::grade::base::yield_model::room_yield(
                &r.room_type,
                r.formula_type.as_deref(),
                r.level,
                r.total_efficiency,
                r.order_value,
            )
            .lmd_per_day
        })
}

fn factory_efficiency(gd: &GameData, roster: &[OperatorBaseProfile]) -> f64 {
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let asn = compute_optimal_assignment(
        roster,
        &single_room("MANUFACTURE", 3),
        &gd.building,
        &registry,
        &drains,
    );
    asn.rooms
        .iter()
        .find(|r| r.room_type == "MANUFACTURE")
        .map_or(0.0, |r| r.total_efficiency)
}

const TEXAS: &str = "char_102_texas";
const EXUSIAI: &str = "char_103_angel";
const LAPPLAND: &str = "char_140_whitew";

#[test]
fn trading_post_finds_penguin_logistics_synergy() {
    let gd = load_game_data();

    // Texas's +65% only triggers with Lappland; Lappland has ~0 standalone
    // efficiency. The two distractors are plain +30% trading operators. A naive
    // optimizer that credits Texas unconditionally would pick
    // Texas + Exusiai + a distractor and drop Lappland. The correct optimum is
    // Texas + Lappland + Exusiai.
    let roster = vec![
        profile(gd, TEXAS),
        profile(gd, EXUSIAI),
        profile(gd, LAPPLAND),
        profile(gd, "char_502_nblade"), // flat +30%
        profile(gd, "char_185_frncat"), // flat +30%
    ];

    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    let asn =
        compute_optimal_assignment(&roster, &trading_post(3), &gd.building, &registry, &drains);

    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post in the assignment");

    let has = |id: &str| tp.operators.iter().any(|o| o == id);

    assert!(
        has(TEXAS),
        "Texas should be selected. Got: {:?}",
        tp.operators
    );
    assert!(
        has(LAPPLAND),
        "Lappland MUST be selected to enable Texas's +65% (this is the synergy fix). Got: {:?}",
        tp.operators
    );
    assert!(
        has(EXUSIAI),
        "Exusiai (+35% flat) should round out the team. Got: {:?}",
        tp.operators
    );
}

#[test]
fn control_center_is_surfaced_in_the_assignment() {
    let gd = load_game_data();

    let building = UserBuilding {
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

    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);

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
fn texas_without_lappland_is_not_overcredited() {
    let gd = load_game_data();

    // Roster has Texas but NOT Lappland, plus two flat +30% operators. Since
    // Texas's +65% is conditional on Lappland (absent here), her Feud buff must
    // contribute 0 - so the two +30% operators should both beat her.
    let roster = vec![
        profile(gd, TEXAS),
        profile(gd, "char_502_nblade"),
        profile(gd, "char_185_frncat"),
    ];

    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    // Single-slot trading post (level 1) forces a choice; Texas (0 without
    // Lappland) must lose to a +30% operator.
    let asn =
        compute_optimal_assignment(&roster, &trading_post(1), &gd.building, &registry, &drains);
    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post");

    assert!(
        !tp.operators.iter().any(|o| o == TEXAS),
        "Texas's conditional +65% must NOT be credited without Lappland. Got: {:?}",
        tp.operators
    );
}

#[test]
fn shamare_nullifies_teammate_output() {
    // Shamare zeroes every teammate's efficiency, so a Shamare room's output
    // depends only on how many bodies fill it - NOT on who those bodies are.
    let gd = load_game_data();
    const SHAMARE: &str = "char_254_vodfox";

    // Strong trading teammates …
    let with_strong = vec![
        profile(gd, SHAMARE),
        profile(gd, EXUSIAI),           // +35%
        profile(gd, "char_4045_heidi"), // +35%
    ];
    // … vs operators with no trading skill at all (pure bodies).
    let with_bodies = vec![
        profile(gd, SHAMARE),
        profile(gd, "char_003_kalts"),  // no trading buff
        profile(gd, "char_180_amgoat"), // no trading buff
    ];

    let strong = trading_efficiency(gd, &with_strong);
    let bodies = trading_efficiency(gd, &with_bodies);

    assert!(
        (strong - bodies).abs() < 1.0,
        "Shamare should zero teammates → identical output regardless of who they are \
         (strong={strong:.1}, bodies={bodies:.1})"
    );
    // And it must NOT be the naive sum (Exusiai 35 + Heidi 35 + Shamare ~90).
    assert!(
        strong < 120.0,
        "teammates' efficiency must be zeroed, got {strong:.1}"
    );
}

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

    // Even though the non-Rhine teammates have *higher* flat efficiency (35 vs
    // 25), Dorothy's faction bonus (+5% × 2 Rhine = +10%) makes the Rhine team
    // competitive/better. The key assertion: the faction bonus is actually applied.
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
    // is NOT a Standardization skill on its own - but Highmore converts Rhine
    // skills into Standardization, so Mizuki should then count Silence.
    let gd = load_game_data();
    const MIZUKI: &str = "char_437_mizuki"; // +5% per Standardization skill
    const HIGHMORE: &str = "char_4066_highmo"; // converts Rhine/Pinus → Standardization
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
fn rotation_plan_orders_members_by_when_to_swap_and_names_a_backup() {
    // The rotation plan: per production room, the main operators ordered by who
    // needs swapping FIRST (shortest hours), plus an idle backup to rotate in.
    let gd = load_game_data();
    let building = generic_base();
    let profiles = full_roster(gd);

    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let rotation =
        compute_sustained_assignment(&profiles, &building, &gd.building, &registry, &drains);

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
    // filler can back up several rooms - the shared bench is far smaller than a
    // doubled roster (one backup per room), never bigger than the room count.
    let gd = load_game_data();
    let building = generic_base();
    let profiles = full_roster(gd);

    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let rotation =
        compute_sustained_assignment(&profiles, &building, &gd.building, &registry, &drains);

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
    // than there are rooms - the same filler is reused, not one reserved per room.
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
fn weedy_is_not_padded_into_a_normal_factory() {
    // Weedy zeroes her teammates' output, so she only belongs in her own
    // automation team. With a strong normal team already filling the beneficial
    // seats, she must NOT be padded into the leftover slot (which would wreck the
    // team). A harmless 0-value filler takes the seat instead.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    // One factory + one power plant (no trading post -> the factory makes EXP,
    // where gold specialists don't apply). The generic +30% pair are the normal
    // team; Texas has no factory skill and is the natural filler.
    let building = UserBuilding {
        rooms: vec![room("mf", "MANUFACTURE", 3), room("pp", "POWER", 3)],
    };
    let roster = vec![
        profile(gd, "char_4141_marcil"), // Marcille, generic +30%
        profile(gd, "char_242_otter"),   // Mayer, generic +30%
        profile(gd, "char_400_weedy"),   // Weedy, nullifying automation
        profile(gd, "char_102_texas"),   // Texas, trading-only (no factory value)
    ];

    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
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
    // a generic +30% who serves any product equally - the generic is freed for the
    // EXP factory that has no dedicated operator. This is the Marcille vs Haze case.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    // Two factories (one Gold to feed the trading post, one EXP) + a trading post.
    let building = UserBuilding {
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

    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
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
fn proviso_is_credited_as_a_strong_gold_trader() {
    // Proviso's Pure-Gold payoff raises LMD per ORDER (value), not order speed -
    // so it shows up in the LMD yield, not the efficiency %. By LMD she should
    // rival or beat a strong flat (speed) trader.
    let gd = load_game_data();
    let proviso = trading_lmd(gd, &[profile(gd, "char_4032_provs")]);
    let flat = trading_lmd(gd, &[profile(gd, "char_502_nblade")]); // +30% flat speed
    assert!(
        proviso >= flat,
        "Proviso's order value should rival a strong flat trader by LMD (proviso {proviso:.0} vs flat {flat:.0})"
    );
    // And her order VALUE must NOT inflate the displayed efficiency %.
    let proviso_eff = trading_efficiency(gd, &[profile(gd, "char_4032_provs")]);
    assert!(
        proviso_eff < 5.0,
        "Proviso's value must not show as order-speed efficiency, got +{proviso_eff:.1}%"
    );
}

#[test]
fn proviso_value_does_not_survive_shamare() {
    // Shamare shifts the post toward Precious-Metal orders, so Proviso's Pure-Gold
    // value no longer applies in her team. A Shamare + Proviso post reads only
    // Shamare's own ~10% value, NOT Proviso's +55% - she is wasted with Shamare and
    // belongs in a fast post instead.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let asn = compute_optimal_assignment(
        &[
            profile(gd, "char_254_vodfox"),
            profile(gd, "char_4032_provs"),
        ],
        &trading_post(3),
        &gd.building,
        &registry,
        &drains,
    );
    let tp = asn.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    assert!(
        tp.order_value < 20.0,
        "Proviso's +55% Pure-Gold value must be nullified by Shamare, got {:.0}",
        tp.order_value
    );
}

#[test]
fn jaye_order_limit_efficiency_is_counted() {
    // Jaye's "+X% per order-limit difference" is an EFFICIENCY skill (its id
    // contains "_limit", but it must not be treated as capacity-only). It still
    // lifts the Texas+Lappland+Jaye team meaningfully above Texas's +65% alone
    // TIME-AVERAGED diff bonus over a shift (~20%, not the empty-post peak).
    let gd = load_game_data();
    let team = vec![
        profile(gd, TEXAS),
        profile(gd, LAPPLAND),
        profile(gd, "char_272_strong"), // Jaye
    ];
    let eff = trading_efficiency(gd, &team);
    assert!(
        eff > 75.0 && eff < 95.0,
        "Texas+Lappland+Jaye should read above Texas's +65% by Jaye's shift-averaged diff bonus (~+85%), got +{eff:.1}%"
    );
}

#[test]
fn jaye_diff_skill_is_bounded_by_the_order_limit() {
    // Jaye's "Street Economics" scales with the order DIFFERENCE, which is bounded
    // by the post's order limit. Teammates like Exusiai/Lemuen add lots of
    // efficiency but NO order limit, so they must NOT pump Jaye past his ceiling:
    // a Jaye/Exusiai/Lemuen post reads ~+120% (Jaye ≈ 40, not the ~60 an
    // un-bounded efficiency-mirror gave, which inflated the post to ~140%).
    let gd = load_game_data();
    let team = vec![
        profile(gd, "char_272_strong"), // Jaye
        profile(gd, EXUSIAI),
        profile(gd, "char_4193_lemuen"),
    ];
    let eff = trading_efficiency(gd, &team);
    assert!(
        (eff - 100.0).abs() < 6.0,
        "Jaye/Exusiai/Lemuen should read ~+100% (Jaye shift-averaged ≈ 20), got +{eff:.1}%"
    );
}

#[test]
fn lemuen_conditional_bonus_with_exusiai_is_counted() {
    // Lemuen's "Amicus": base +20% order efficiency, PLUS +25% more when Exusiai
    // shares the post - phrased "if <Exusiai> is assigned to the same Trading
    // Post" (name BEFORE "same"), unlike Texas's "...same Post as <Lappland>".
    // The +25% bonus was previously dropped, leaving the team at ~+85% (which the
    // user saw as ~92%). With it counted: Lemuen 45 + Exusiai 35 + Quartz 30 = 110.
    let gd = load_game_data();
    let team = vec![
        profile(gd, "char_4193_lemuen"),
        profile(gd, EXUSIAI),
        profile(gd, "char_4063_quartz"),
    ];
    let eff = trading_efficiency(gd, &team);
    assert!(
        eff > 105.0,
        "Lemuen+Exusiai+Quartz should read ~+110% (Lemuen's +25% Exusiai bonus counted), got +{eff:.1}%"
    );

    // The +25% is genuinely conditional: drop Exusiai and Lemuen keeps only her
    // base +20%, so the same two-trader pairing must read materially lower.
    let no_exu = trading_efficiency(
        gd,
        &[
            profile(gd, "char_4193_lemuen"),
            profile(gd, "char_4063_quartz"),
        ],
    );
    assert!(
        eff > no_exu + 20.0,
        "Exusiai should unlock Lemuen's +25% (with {eff:.1}% vs without {no_exu:.1}%)"
    );
}

#[test]
fn quartz_recipe_scaling_counts_distinct_recipe_types_not_factories() {
    // Quartz "Precise Scheduling" (trade_ord_spd&formula[000]): base +30% trading, plus +2% per
    // recipe TYPE being processed at Factories. That scales on the number of distinct formulas the
    // base runs (gold + EXP = 2), NOT the factory count - four gold/EXP factories still process two
    // recipe types. Her buff resolves to a FacilityCountScaling targeting the synthetic
    // MANUFACTURE_RECIPE_TYPES count with a +30% base, so a 2-recipe base reads +34% (30 + 2×2).

    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _drains) = build_registry(&gd.building.buffs, &name_to_char);

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
fn morgan_glasgow_count_carries_a_siege_rider() {
    // Morgan "Gang Compass": +20% per Glasgow Gang op in the post, AND +35% more
    // "when in the same Trading Post as Siege". The faction count is the dominant
    // term (MatchCountScaling), but the named Siege rider must ride along on top.

    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _drains) = build_registry(&gd.building.buffs, &name_to_char);
    let siege_id = name_to_char.get("siege").cloned();

    match registry.get("trade_ord_spd_par[000]") {
        Some(BuffResolutionStrategy::MatchCountScaling {
            token,
            per_match_pct,
            bonus_char_id,
            bonus_pct,
            ..
        }) => {
            assert_eq!(token, "glasgow", "counts Glasgow Gang operators");
            assert!(
                (*per_match_pct - 20.0).abs() < 0.01,
                "per Glasgow op +20%, got {per_match_pct}"
            );
            assert_eq!(
                bonus_char_id.as_deref(),
                siege_id.as_deref(),
                "rider gated on Siege"
            );
            assert!(
                (*bonus_pct - 35.0).abs() < 0.01,
                "Siege rider +35%, got {bonus_pct}"
            );
        }
        _ => panic!("Morgan's Gang Compass should be MatchCountScaling with a Siege rider"),
    }
}

#[test]
fn faction_conditional_credits_base_plus_glasgow_bonus() {
    // Vina Victoria "Resolution on Foreign Trade β": base +30% trading, plus +10%
    // more "if a Glasgow Gang Operator is assigned to the same Trading Post". The
    // faction analogue of a named-teammate conditional - base always on, bonus
    // gated on a faction tag rather than one operator.

    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _drains) = build_registry(&gd.building.buffs, &name_to_char);

    match registry.get("trade_ord_spd&par[001]") {
        Some(BuffResolutionStrategy::ConditionalOnFaction {
            faction_token,
            base_efficiency,
            efficiency,
        }) => {
            assert_eq!(
                faction_token, "glasgow",
                "bonus gated on Glasgow Gang presence"
            );
            assert!(
                (*base_efficiency - 30.0).abs() < 0.01,
                "base +30%, got {base_efficiency}"
            );
            assert!(
                (*efficiency - 10.0).abs() < 0.01,
                "Glasgow bonus +10%, got {efficiency}"
            );
        }
        _ => panic!("Resolution β should be a ConditionalOnFaction (base + Glasgow bonus)"),
    }
}

#[test]
fn morgan_reads_higher_with_siege_in_the_post() {
    // End-to-end: with Siege sharing the post, Morgan's +35% rider fires, so the
    // team must out-produce the same post with Siege swapped for another Glasgow
    // operator (which keeps the faction count but loses the rider).
    let gd = load_game_data();
    const MORGAN: &str = "char_154_morgan";
    const SIEGE: &str = "char_112_siege";
    const INDRA: &str = "char_155_tiger";
    const MATTERHORN: &str = "char_199_yak";

    let with_siege = trading_efficiency(
        gd,
        &[profile(gd, MORGAN), profile(gd, SIEGE), profile(gd, INDRA)],
    );
    let without_siege = trading_efficiency(
        gd,
        &[
            profile(gd, MORGAN),
            profile(gd, MATTERHORN),
            profile(gd, INDRA),
        ],
    );
    assert!(
        with_siege > without_siege,
        "Siege's presence should fire Morgan's +35% rider (with Siege {with_siege:.1}% vs without {without_siege:.1}%)"
    );
}

#[test]
fn faction_gated_cc_buffs_are_conditional_not_flat_global() {
    // Control Center trading buffs split three ways:
    //   - Amiya "all Trading Posts +7%"          → unconditional GlobalEffect
    //   - Umiri "all Siracusa Operators +5%"      → per-operator ConditionalGlobalEffect
    //   - SilverAsh "posts w/ 3 Kjerag ops +10%"  → count-gated ConditionalGlobalEffect
    // Only the first may be credited flat to every post.

    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _drains) = build_registry(&gd.building.buffs, &name_to_char);

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
    // so two of them must NOT stack - only the strongest applies.
    use backend::core::grade::base::assignment::compute_current_assignment;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    // The global buff a CC team grants, read off an empty trading post.
    let trading_global = |cc: &[&str]| -> f64 {
        let building = UserBuilding {
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
            compute_current_assignment(&roster, &building, &gd.building, &registry, &drains, None);
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
fn control_center_clauseless_buffs_stack() {
    // Mon3tr's "+2% all Factories" carries the non-stacking clause, but Sakiko's
    // Precious-Metal productivity does NOT - so Sakiko stacks on top of Mon3tr.
    use backend::core::grade::base::assignment::compute_current_assignment;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    let manu_global = |cc: &[&str]| -> f64 {
        let building = UserBuilding {
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
            compute_current_assignment(&roster, &building, &gd.building, &registry, &drains, None);
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

#[test]
fn shamare_money_printer_forms_despite_a_deep_distractor_pool() {
    // Shamare nullifies teammates' SPEED, so her only useful partners are
    // order-VALUE operators (Tequila, Bibeak) whose LMD-per-order survives. In a
    // deep roster (18 candidates > the 16-candidate cut) those low-speed value
    // operators would normally be dropped; the nullifier value-boost keeps them,
    // so Shamare + Tequila + Bibeak still forms instead of Shamare + dead bodies.
    let gd = load_game_data();
    const SHAMARE: &str = "char_254_vodfox";
    const TEQUILA: &str = "char_486_takila";
    const BIBEAK: &str = "char_252_bibeak";
    // 15 flat trading operators - distractors that crowd the candidate set.
    let distractors = [
        "char_502_nblade",
        "char_123_fang",
        "char_211_adnach",
        "char_240_wyvern",
        "char_110_deepcl",
        "char_141_nights",
        "char_150_snakek",
        "char_185_frncat",
        "char_289_gyuki",
        "char_101_sora",
        "char_103_angel",
        "char_302_glaze",
        "char_4045_heidi",
        "char_1033_swire2",
        "char_4163_rosesa",
    ];
    let mut roster = vec![
        profile(gd, SHAMARE),
        profile(gd, TEQUILA),
        profile(gd, BIBEAK),
    ];
    roster.extend(distractors.iter().map(|d| profile(gd, d)));

    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let asn =
        compute_optimal_assignment(&roster, &trading_post(3), &gd.building, &registry, &drains);
    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post");
    let has = |id: &str| tp.operators.iter().any(|o| o == id);

    assert!(
        has(SHAMARE),
        "Shamare should anchor the post. Got: {:?}",
        tp.operators
    );
    assert!(
        has(TEQUILA) && has(BIBEAK),
        "Shamare's order-value partners (Tequila + Bibeak) must survive the candidate cut. Got: {:?}",
        tp.operators
    );
}

#[test]
fn fixed_synergy_squads_are_marked_locked_and_flexible_teams_are_not() {
    // A Shamare/Tequila/Bibeak post is a FIXED squad (they only work together), so
    // it's flagged `locked`. A post of three independent flat traders is flexible.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    let locked_for = |roster: &[OperatorBaseProfile]| -> bool {
        let asn =
            compute_optimal_assignment(roster, &trading_post(3), &gd.building, &registry, &drains);
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
fn viviana_knight_buff_reaches_pinus_knights_like_wild_mane() {
    // Viviana's CC buff "+7% to all Knight Operators in Factories" must reach Wild
    // Mane, who is Pinus Sylvestris (the Knightclub) - so the optimizer co-schedules
    // them. A non-Knight factory operator gets nothing from her.
    use backend::core::grade::base::assignment::compute_current_assignment;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    let factory_eff = |cc: &[&str], factory: &[&str]| -> f64 {
        let building = UserBuilding {
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
            compute_current_assignment(&roster, &building, &gd.building, &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.room_type == "MANUFACTURE")
            .map_or(0.0, |r| r.total_efficiency)
    };
    const VIVIANA: &str = "char_4098_vvana";
    const WILD_MANE: &str = "char_496_wildmn";
    const VERMEIL: &str = "char_190_clour"; // Siracusa - not a Knight

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
    // Umiri's "Siracusa" +5% must NOT apply - this team is Kazimierz/Laterano/
    // Lungmen - so it must NOT read the old, inflated 102%.
    use backend::core::grade::base::assignment::compute_current_assignment;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

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
        compute_current_assignment(&roster, &building, &gd.building, &registry, &drains, None);
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
fn weedy_is_treated_as_power_scaling_automation() {
    // Weedy nullifies teammates and scales with POWER PLANTS, not teammates. In a
    // base with more power plants she must produce more - proving she's modeled as
    // facility-scaling automation, not as a Shamare-style per-teammate nullifier.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

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
            &UserBuilding { rooms },
            &gd.building,
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
    let building = UserBuilding { rooms };

    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let optimal =
        compute_optimal_assignment(&profiles, &building, &gd.building, &registry, &drains);

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
fn current_preset_shifts_are_distinct() {
    // When the player has a planned rotation (preset shifts), the "current"
    // Shift A and Shift B must reflect those distinct presets - not the same
    // static crew for both.
    use backend::core::grade::base::assignment::{compute_current_assignment, has_preset_shifts};
    let gd = load_game_data();
    let profiles = full_roster(gd);
    // A trading post with two distinct planned shifts.
    let building = UserBuilding {
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
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    assert!(
        has_preset_shifts(&building),
        "building should expose the two planned shifts"
    );

    let ops = |shift: Option<usize>| {
        compute_current_assignment(
            &profiles,
            &building,
            &gd.building,
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
    // - not a big drop from averaging weaker whole teams. LOCKED synergy teams can't
    // be staggered one operator at a time, so they sustain a little further below
    // peak (their rest gaps go uncovered), which is why the floor is ~20%.
    let gd = load_game_data();
    let building = generic_base();
    let profiles = full_roster(gd);
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    let rot = compute_sustained_assignment(&profiles, &building, &gd.building, &registry, &drains);
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
fn low_morale_drain_teams_sustain_better_longevity_axis() {
    // The longevity axis: a low-morale-drain factory team holds closer to peak
    // under rotation than a higher-drain team, because its operators rest less and
    // need backups to cover them less often.
    use backend::core::grade::base::assignment::{morale_recovery, sustained_efficiency_of};

    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    // Same room peak, different drain: Vermeil (-0.25/hr) is a far lower-drain
    // factory worker than a neutral/high-drain operator.
    let sustain_fraction = |op: &str| -> f64 {
        let roster = vec![profile(gd, op)];
        let building = UserBuilding {
            rooms: vec![UserRoom {
                slot_id: "mf".into(),
                room_type: "MANUFACTURE".into(),
                level: 3,
                current_operators: vec![op.to_string()],
                ..Default::default()
            }],
        };
        let cur = backend::core::grade::base::assignment::compute_current_assignment(
            &roster,
            &building,
            &gd.building,
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
            &gd.building,
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
fn current_assignment_reflects_live_base() {
    // The "current" assignment should mirror the player's actual stationed
    // operators and be no better than the optimizer's peak (which is the best
    // possible arrangement of the same roster).
    let gd = load_game_data();
    let profiles = full_roster(gd);
    // A base where the player has stationed a deliberately mediocre crew (plain
    // flat traders / factory ops), so the optimizer has room to improve.
    let building = UserBuilding {
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

    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    let current = backend::core::grade::base::assignment::compute_current_assignment(
        &profiles,
        &building,
        &gd.building,
        &registry,
        &drains,
        None,
    );
    let optimal =
        compute_optimal_assignment(&profiles, &building, &gd.building, &registry, &drains);

    // Current base has production rooms with real operators stationed.
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
    // by realized YIELD (LMD-equivalent/day) - the optimizer's actual objective -
    // not the raw efficiency-sum, which ignores the gold/EXP split, order value,
    // and per-room soft caps and so can rank a lower-yield base higher.
    let realized_value = |asn: &backend::core::grade::base::types::BaseAssignment| -> f64 {
        let mut flows = backend::core::grade::base::yield_model::BaseFlows::default();
        for r in &asn.rooms {
            flows.add_room(
                &r.room_type,
                r.formula_type.as_deref(),
                r.level,
                r.total_efficiency,
                r.order_value,
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
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    // Warm up, then time the full optimizer pipeline (the part the grade runs).
    let iters = 30;
    let t = Instant::now();
    for _ in 0..iters {
        let _ = compute_optimal_assignment(&profiles, &building, &gd.building, &registry, &drains);
        let _ =
            compute_sustained_assignment(&profiles, &building, &gd.building, &registry, &drains);
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

    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    let optimal =
        compute_optimal_assignment(&profiles, &building, &gd.building, &registry, &drains);
    let sustained =
        compute_sustained_assignment(&profiles, &building, &gd.building, &registry, &drains);

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
fn grade_base_produces_a_sane_score() {
    // The high-level scorer: a base with a partial roster should grade in [0, 1]
    // and strictly above 0 (it has production capacity), and a full roster on the
    // same base should score at least as high.
    let gd = load_game_data();
    let building_json = serde_json::json!({
        "roomSlots": {
            "cc": { "roomId": "CONTROL", "level": 5, "state": 2 },
            "tp0": { "roomId": "TRADING", "level": 3, "state": 2 },
            "mf0": { "roomId": "MANUFACTURE", "level": 3, "state": 2 },
            "mf1": { "roomId": "MANUFACTURE", "level": 3, "state": 2 },
            "d0": { "roomId": "DORMITORY", "level": 5, "state": 2 },
        }
    });

    let partial: Vec<RosterEntry> = [
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
    let partial_score = grade_base(&partial, Some(&building_json), gd);
    assert!(
        (0.0..=1.0).contains(&partial_score) && partial_score > 0.0,
        "partial-roster score should be in (0, 1], got {partial_score}"
    );

    let full: Vec<RosterEntry> = gd
        .building
        .chars
        .keys()
        .filter(|id| id.starts_with("char_"))
        .map(|id| roster_entry(id))
        .collect();
    let full_score = grade_base(&full, Some(&building_json), gd);
    assert!(
        (0.0..=1.0).contains(&full_score) && full_score >= partial_score - 1e-6,
        "full-roster score ({full_score}) should be in [0,1] and >= partial ({partial_score})"
    );
}

#[test]
fn conditional_cc_buff_is_not_credited_when_its_gate_cannot_be_met() {
    // SilverAsh the Reignfrost's Control Center buff only helps Trading Posts that
    // hold 3 Kjerag operators. With too few Kjerag traders in the roster the gate
    // can never trigger, so he must NOT take a CC seat - a legitimate global op
    // (Amiya: "all Trading Posts +X%") gets it instead.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    let building = UserBuilding {
        rooms: vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)],
    };
    let roster = vec![
        profile(gd, "char_1045_svash2"), // SilverAsh the Reignfrost (conditional CC)
        profile(gd, "char_002_amiya"),   // Amiya (unconditional CC global)
        profile(gd, "char_103_angel"),   // Exusiai, trader
        profile(gd, "char_102_texas"),   // Texas, trader
    ];

    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
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
fn degenbrecher_is_not_used_in_a_trading_post_without_cap_synergy() {
    // Degenbrecher gives +25% speed but slashes the order limit by 6 ("minimum 1"),
    // and her payoff clause needs teammates who ADD order limit - which trading
    // posts don't field. So she cripples a normal post and must be left out in
    // favour of ordinary traders (the seat stays empty before she's forced in).
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    let building = UserBuilding {
        rooms: vec![room("tp", "TRADING", 3)],
    };
    let roster = vec![
        profile(gd, "char_4116_blkkgt"), // Degenbrecher (order-limit wrecker)
        profile(gd, "char_103_angel"),   // Exusiai, trader
        profile(gd, "char_102_texas"),   // Texas, trader
    ];

    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post");
    assert!(
        !tp.operators.iter().any(|o| o == "char_4116_blkkgt"),
        "Degenbrecher should not be staffed in a normal trading post. Got: {:?}",
        tp.operators
    );
    assert!(
        tp.total_efficiency > 0.0,
        "the trading post should keep its positive efficiency, got {:.1}",
        tp.total_efficiency
    );
}

#[test]
fn cross_formula_reallocation_frees_a_generic_for_exp() {
    // Two generic +30% operators would both pile into the single Gold factory, but
    // the second's gold is unsold (one trading post). The cross-formula pass frees
    // one of them for the empty EXP factory, where it produces real value.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    let building = UserBuilding {
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

    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
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
fn texas_is_not_a_standalone_rotation_backup_without_lappland() {
    // Texas's trading value is entirely conditional on Lappland sharing the post
    // (her base is 0). As a staggered rotation backup she is swapped in to cover one
    // resting main, so with no Lappland present she does nothing - she must NOT be
    // chosen as a standalone backup over a real trader.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let building = UserBuilding {
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
    let rot = compute_sustained_assignment(&roster, &building, &gd.building, &registry, &drains);
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
fn low_level_dorms_reduce_sustained_output() {
    // Morale recovery scales with dorm level, so the same production setup sustains
    // further below peak when its dormitories are low-level (a 252 reality) than when
    // they are fully developed. The peak (main) staffing is identical either way.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let layout = |dorm_lv: i32| UserBuilding {
        rooms: vec![
            room("tp", "TRADING", 3),
            room("mf", "MANUFACTURE", 3),
            room("d0", "DORMITORY", dorm_lv),
            room("d1", "DORMITORY", dorm_lv),
        ],
    };
    let roster = full_roster(gd);
    let hi = compute_sustained_assignment(&roster, &layout(5), &gd.building, &registry, &drains);
    let lo = compute_sustained_assignment(&roster, &layout(1), &gd.building, &registry, &drains);
    assert!(
        lo.sustained_efficiency < hi.sustained_efficiency,
        "low-level dorms should sustain below high-level dorms (lo {:.1} vs hi {:.1})",
        lo.sustained_efficiency,
        hi.sustained_efficiency
    );
}

#[test]
fn dead_conditional_cc_operator_is_dropped_after_assignment() {
    // The roster HAS three Kjerag traders, so SilverAsh's gate passes the roster
    // feasibility check and he is initially seated in the CC. But stronger non-Kjerag
    // traders win the single post's seats, so no post ever holds 3 Kjerag - his +10%
    // never fires. The post-assignment check must drop him and reseat a real op.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let building = UserBuilding {
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
    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
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
fn morale_operator_takes_a_spare_cc_seat_and_lifts_sustain() {
    // A global morale-recovery operator (Wiš'adel: workers in other buildings recover
    // faster) should claim a spare Control Center seat instead of being burned as a
    // zero-value production filler, and its base-wide recovery boost must raise the
    // sustained output - the "add a morale operator" the rotation wants.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let layout = UserBuilding {
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
        compute_sustained_assignment(&roster(true), &layout, &gd.building, &registry, &drains);
    let without =
        compute_sustained_assignment(&roster(false), &layout, &gd.building, &registry, &drains);

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
    // dorms - less capacity - means the rotation can't rest everyone and sustained
    // output is throttled. This is the "you can only rest 20, so you need 3 sets" cap.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let layout = |dorms: usize| {
        let mut rooms = vec![room("tp0", "TRADING", 3), room("tp1", "TRADING", 3)];
        rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
        rooms.extend((0..dorms).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
        UserBuilding { rooms }
    };
    let roster = full_roster(gd);
    let cramped =
        compute_sustained_assignment(&roster, &layout(1), &gd.building, &registry, &drains);
    let roomy = compute_sustained_assignment(&roster, &layout(4), &gd.building, &registry, &drains);
    assert!(
        cramped.sustained_efficiency < roomy.sustained_efficiency,
        "too few dorms should throttle sustained output (cramped {:.1} vs roomy {:.1})",
        cramped.sustained_efficiency,
        roomy.sustained_efficiency
    );
}

#[test]
fn rotation_is_expressed_as_overlapping_sets() {
    // The rotation surfaces as a few overlapping staffings: each set rests one main
    // per room (covered by the backup), consecutive sets share all-but-one operator,
    // and across the sets every main rests in turn - so the whole base is never
    // swapped at once.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let rot = compute_sustained_assignment(
        &full_roster(gd),
        &generic_base(),
        &gd.building,
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

    // The trading post's working crew in each set.
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
    // on morale within a cycle - NOT just the seat count. A factory of low-drain
    // operators (Vermeil/Vulcan-type, ~48h before a swap) needs no rest slots and
    // yields zero sets, while the same factory of neutral-drain operators (~24h)
    // rotates each of them and yields the full three sets.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let building = UserBuilding {
        rooms: vec![room("mf", "MANUFACTURE", 3), room("d0", "DORMITORY", 3)],
    };
    let sets_for = |ids: &[&str]| {
        let roster: Vec<_> = ids.iter().map(|id| profile(gd, id)).collect();
        compute_sustained_assignment(&roster, &building, &gd.building, &registry, &drains)
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
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    // Four factories + one power plant: Weedy's lone automation output (one plant)
    // loses to the strong normal teams, so she sits idle - the exact case where she
    // used to be offered as a backup for EVERY factory at once.
    let mut rooms: Vec<UserRoom> = (0..4)
        .map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3))
        .collect();
    rooms.push(room("p0", "POWER", 3));
    rooms.push(room("d0", "DORMITORY", 3));
    let building = UserBuilding { rooms };
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
    let rot = compute_sustained_assignment(&roster, &building, &gd.building, &registry, &drains);

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
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let rot = compute_sustained_assignment(
        &full_roster(gd),
        &generic_base(),
        &gd.building,
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
fn proviso_and_tequila_order_value_does_not_stack() {
    // Proviso's Pure-Gold value (+gold on low/"defaulted" orders) and Tequila's bonus
    // (+LMD on high orders, which EXCLUDES defaulted orders) target the same orders by
    // disjoint rules, so they do NOT combine. A post with both reads only the stronger
    // operator's value, not the sum.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    // Level-2 post (2 seats) forces both value operators in together.
    let asn = compute_optimal_assignment(
        &[
            profile(gd, "char_4032_provs"),
            profile(gd, "char_486_takila"),
        ],
        &trading_post(2),
        &gd.building,
        &registry,
        &drains,
    );
    let tp = asn.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    assert!(
        (50.0..60.0).contains(&tp.order_value),
        "Proviso + Tequila value must be the stronger one (~55%), not the sum (~65%), got {:.1}",
        tp.order_value
    );
}

#[test]
fn proviso_pairs_with_speed_not_another_value_operator() {
    // Because a second value operator is wasted, Proviso's best partners are the
    // fastest order-acquisition operators. The optimizer must staff Proviso with the
    // two fastest speed traders and leave Tequila out.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let roster = vec![
        profile(gd, "char_4032_provs"), // Proviso (value)
        profile(gd, "char_486_takila"), // Tequila (value)
        profile(gd, "char_103_angel"),  // Exusiai +35 speed
        profile(gd, "char_502_nblade"), // Yato +30 speed
        profile(gd, "char_185_frncat"), // Mousse +30 speed
    ];
    let asn =
        compute_optimal_assignment(&roster, &trading_post(3), &gd.building, &registry, &drains);
    let tp = asn.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    assert!(
        tp.operators.iter().any(|o| o == "char_4032_provs"),
        "Proviso should staff the post. Got: {:?}",
        tp.operators
    );
    assert!(
        !tp.operators.iter().any(|o| o == "char_486_takila"),
        "Tequila adds nothing alongside Proviso and must be left out. Got: {:?}",
        tp.operators
    );
}

#[test]
fn proviso_is_not_staffed_in_a_shamare_post() {
    // Proviso's Pure-Gold value is nullified by Shamare, so she belongs in her OWN
    // post. With two posts and no Texas, the optimizer must NOT put Proviso and
    // Shamare in the same post (the bad "Lappland + Proviso + Shamare" team); Proviso
    // gets her own post, and Shamare's post is staffed with surviving-value/filler ops.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let building = UserBuilding {
        rooms: vec![
            room("t0", "TRADING", 3),
            room("t1", "TRADING", 3),
            room("d", "DORMITORY", 3),
        ],
    };
    let roster = vec![
        profile(gd, "char_140_whitew"), // Lappland (needs Texas, absent)
        profile(gd, "char_4032_provs"), // Proviso
        profile(gd, "char_254_vodfox"), // Shamare
        profile(gd, "char_486_takila"), // Tequila
        profile(gd, "char_003_kalts"),  // body
        profile(gd, "char_180_amgoat"), // body
    ];
    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
    let same_post = asn.rooms.iter().any(|r| {
        r.room_type == "TRADING"
            && r.operators.iter().any(|o| o == "char_4032_provs")
            && r.operators.iter().any(|o| o == "char_254_vodfox")
    });
    assert!(
        !same_post,
        "Proviso must not share a post with Shamare. Rooms: {:?}",
        asn.rooms
            .iter()
            .filter(|r| r.room_type == "TRADING")
            .map(|r| (&r.slot_id, &r.operators))
            .collect::<Vec<_>>()
    );
}

#[test]
fn shift_rotation_forms_three_overlapping_shifts() {
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
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
    let building = UserBuilding { rooms };
    let rot = recommend_shift_rotation(
        &full_roster(gd),
        &building,
        &gd.building,
        &registry,
        &drains,
        &[],
    );

    assert_eq!(rot.shifts.len(), 3, "three shifts");

    let cell =
        |shift: usize, slot: &str| -> &backend::core::grade::base::shift_rotation::ShiftRoom {
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
    // third - either alternating (1&3 / 2) or as a 24h block (1+2 / 3) when a synergy aligns it.
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

    // The HR Office and Reception Room cycle two squads too - the per-login swap.
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
    assert_eq!(
        tp_ids[0][0], tp_ids[0][1],
        "room0 runs one team for shifts 1+2"
    );
    assert_ne!(
        tp_ids[0][1], tp_ids[0][2],
        "room0 swaps its team for shift 3"
    );
    assert_eq!(
        tp_ids[1][1], tp_ids[1][2],
        "room1 runs one team for shifts 2+3"
    );
    assert_ne!(
        tp_ids[1][0], tp_ids[1][1],
        "room1 swaps its team after shift 1"
    );
    assert_eq!(
        tp_ids[0][2], tp_ids[1][0],
        "the wrap team's block spans room0 shift 3 and room1 shift 1"
    );
    let distinct_teams: std::collections::HashSet<&String> = tp_ids.iter().flatten().collect();
    assert_eq!(
        distinct_teams.len(),
        3,
        "a pair of posts runs exactly three teams"
    );
    // Teams working the same shift never share an operator, and a team shares nobody with its
    // replacement - EXCEPT a Fiammetta-sustained operator, who is deliberately pinned to one
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
fn team_value_is_order_independent_and_rewards_staffing() {
    // `team_value` is what the rotation comparison uses to tell whether a player's current team
    // already ties the recommendation (Bryophyta vs a Dorothy-boosted Rhine operator). It must
    // not depend on operator order, and a staffed team must score at least as high as an empty
    // room - otherwise an equivalent team could be misjudged.
    let gd = load_game_data();
    let building = trading_post(3);
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
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
        &gd.building,
        &registry,
        &drains,
    );
    let v_rev = team_value(
        &reversed,
        "TRADING",
        None,
        &roster,
        &building,
        &gd.building,
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
        &gd.building,
        &registry,
        &drains,
    );
    assert!(
        v >= empty,
        "a staffed team must score at least as high as an empty room"
    );
}

#[test]
fn facility_count_enabler_boosts_power_scaling_automation() {
    // Greyy the Lightningbearer E2 raises the EFFECTIVE Power Plant count by 1 ("only affects
    // facility quantity"), so a per-power automation operator (Weedy) in a factory produces MORE
    // when Greyy is on the roster - even though Greyy herself only staffs a power plant.
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let building = UserBuilding {
        rooms: vec![room("f", "MANUFACTURE", 3), room("p0", "POWER", 3)],
    };
    let factory_eff = |roster: &[OperatorBaseProfile]| -> f64 {
        compute_optimal_assignment(roster, &building, &gd.building, &registry, &drains)
            .rooms
            .iter()
            .find(|r| r.room_type == "MANUFACTURE")
            .map_or(0.0, |r| r.total_efficiency)
    };
    let weedy_only = vec![profile(gd, "char_400_weedy")];
    let weedy_greyy = vec![
        profile(gd, "char_400_weedy"),
        profile(gd, "char_1027_greyy2"),
    ];
    let (a, b) = (factory_eff(&weedy_greyy), factory_eff(&weedy_only));
    assert!(
        a > b,
        "Greyy E2's +1 Power Plant should raise Weedy's per-power output ({a} vs {b})"
    );
}

#[test]
fn vermeil_converts_her_own_capacity() {
    // Vermeil's E1 scales factory productivity with the WHOLE factory's capacity limit -
    // including her own +8 from her E0. A solo Vermeil should therefore convert that +8 into a
    // real productivity bonus (~+16% at +2%/cap), not score a flat 0 (the self-exclusion bug).
    let gd = load_game_data();
    let building = UserBuilding {
        rooms: vec![room("mf", "MANUFACTURE", 3)],
    };
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let roster = vec![profile(gd, "char_190_clour")]; // Vermeil (E1+ in game data fixture)
    let solo = team_value(
        &["char_190_clour".into()],
        "MANUFACTURE",
        Some("F_GOLD"),
        &roster,
        &building,
        &gd.building,
        &registry,
        &drains,
    );
    assert!(
        solo > 1.05,
        "a solo Vermeil should convert her own +8 capacity into productivity (>+5%), got {solo:.3}"
    );
}

#[test]
fn optimal_with_empty_pins_is_identical_to_the_plain_optimum() {
    // The pinned path must be a no-op when nothing is pinned, so non-243 / no-economy bases (and
    // every existing caller) get exactly the plain optimum - the economy only ever ADDS pins.
    let gd = load_game_data();
    let mut rooms = vec![room("cc", "CONTROL", 5), room("hr", "HIRE", 3)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("pp{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding { rooms };
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
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));

    let plain = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
    let pinned = compute_optimal_assignment_with_pins(
        &roster,
        &building,
        &gd.building,
        &registry,
        &drains,
        &[],
    );

    let rooms_of = |a: &backend::core::grade::base::types::BaseAssignment| {
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

// ─── BanG Dream "Passion" combo: joint, slot-budgeted Control-Center valuation ──────────────

const SAKIKO: &str = "char_4182_oblvns"; // Plentiful Work Experience: global FACTORY consumer
const MORTIS: &str = "char_4183_mortis"; // Monster of Acting: generates +20 AND global TRADING consumer
const AMORIS: &str = "char_4185_amoris"; // Diligent Worker: generates +10
const DOLRIS: &str = "char_4184_dolris"; // Idol's Aura: generates +1 per dormitory operator
const TMORIS: &str = "char_4186_tmoris"; // Reliable Companion: generates +10

/// A real 243 layout (Control Center + 2 trading + 4 factories + 3 power + dorms) so the
/// resource economy is in play - the whole Passion combo lives in the Control Center.
fn bd_base_243() -> UserBuilding {
    let mut rooms = vec![room("cc", "CONTROL", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("pp{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    UserBuilding { rooms }
}

/// Run the resource-economy solve the way the service does (real registry + drains).
fn is_control_op(gd: &GameData, op: &OperatorBaseProfile) -> bool {
    op.available_buffs.iter().any(|b| {
        gd.building
            .buffs
            .get(b)
            .is_some_and(|buff| buff.room_type == "CONTROL")
    })
}

#[test]
fn passion_combo_staffs_the_control_center_and_lifts_factories() {
    use backend::core::grade::base::pools::candidate_bundles;
    let gd = load_game_data();
    let building = bd_base_243();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
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
    let bundles = candidate_bundles(&roster, &building, &gd.building, &registry);
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
        &gd.building,
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
        &gd.building,
        &registry,
        &drains,
        &[],
    );
    let factory_eff = |a: &backend::core::grade::base::types::BaseAssignment| -> f64 {
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

#[test]
fn shift_rotation_never_double_books_an_operator_within_a_shift() {
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    use std::collections::HashMap;
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    // bd_base_243 has 3 power plants, exercising the widened power-plan exclusion.
    let building = bd_base_243();
    let roster = full_roster(gd);
    let rotation =
        recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &[]);
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
    use backend::core::grade::base::assignment::{
        morale_recovery, morale_sustained_beneficiaries, sustained_efficiency_of,
    };
    use std::collections::HashSet;
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
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
    let main = compute_optimal_assignment(&base, &building, &gd.building, &registry, &drains);
    let none = morale_sustained_beneficiaries(
        &main,
        &base,
        &drains,
        recovery,
        &building,
        &registry,
        &gd.building,
    );
    assert!(
        none.is_empty(),
        "no manager -> no sustained operator, got {none:?}"
    );

    // With Fiammetta owned -> EXACTLY ONE operator is sustained, and crediting it doesn't lower
    // (it raises) the trading post's sustained efficiency.
    let mut with = base;
    with.push(profile(gd, FIAMMETTA));
    let main2 = compute_optimal_assignment(&with, &building, &gd.building, &registry, &drains);
    let benef = morale_sustained_beneficiaries(
        &main2,
        &with,
        &drains,
        recovery,
        &building,
        &registry,
        &gd.building,
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
        &gd.building,
        &HashSet::new(),
    );
    let credited = sustained_efficiency_of(
        &main2,
        &with,
        &drains,
        recovery,
        &registry,
        &gd.building,
        &benef,
    );
    assert!(
        credited >= without,
        "the sustained operator should not reduce sustained output ({credited} vs {without})"
    );
}

// ─── Part C: Reception Room ambience model (rarity + elite + skill, exchange & solo) ─────────

const CAPER: &str = "char_4100_caper"; // +30% clue search, ONLY during Clue Exchange
const VIGIL: &str = "char_427_vigil"; // +25% clue search
const RED: &str = "char_144_red"; // +25% clue search

fn meeting_room(level: i32) -> UserBuilding {
    single_room("MEETING", level)
}

#[test]
fn reception_room_staffs_caper_during_exchange() {
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let building = meeting_room(3);
    // Caper's clue-search skill is 0 outside exchange (its % lives in the description, parsed under
    // the permanent-exchange assumption); without that, he'd be dropped for the two +25% operators.
    let roster: Vec<_> = [CAPER, VIGIL, RED]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
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
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let building = meeting_room(3);
    let roster: Vec<_> = [VIGIL, RED].iter().map(|id| profile(gd, id)).collect();
    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
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
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let building = meeting_room(3);
    // Kazemaru clue-searches at +15% paired but +35% when he's the room's sole worker (no morale
    // cost). With no partner available he's staffed alone, and the room's total reflects his higher
    // SOLO skill, not the paired one.
    const KAZEMARU: &str = "char_4016_kazema";
    let roster = vec![profile(gd, KAZEMARU)];
    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
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
        build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    // Ines's "Shadow Gathering": clue search +20%, then +2%/hr up to a maximum of 30%. Under
    // sustained operation she holds the 30% ceiling - the `Efficiency` field only reports the 20%
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
    // leaves Enforcer out - without the morale penalty he'd be the top pick.
    const ENFORCER: &str = "char_4036_forcer";
    const CHEN: &str = "char_010_chen";
    const FEATER: &str = "char_109_fmout"; // +25% reception, no morale cost
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let building = UserBuilding {
        rooms: vec![room("rc", "MEETING", 3)],
    };
    let roster: Vec<_> = [ENFORCER, CHEN, FEATER]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
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

#[test]
fn factory_formula_assignment_respects_the_players_current_layout() {
    // The optimizer picks the gold/EXP COUNT, but must lay it onto the slots the player ALREADY
    // runs that way - otherwise it flips every factory's formula and the whole comparison reads as
    // "change everything" (the reported "very wrong" plan).
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
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
    let building = UserBuilding { rooms };

    let asn = compute_optimal_assignment(
        &full_roster(gd),
        &building,
        &gd.building,
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

// ─── Preset-based Fiammetta 24/7 sustain in the shift rotation ───────────────────────────────

#[test]
fn preset_24_7_operator_with_fiammetta_is_kept_every_shift_and_flagged() {
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    const PROVISO: &str = "char_4032_provs";
    const FIAMMETTA: &str = "char_300_phenxi"; // swaps morale with another operator
    const QUARTZ: &str = "char_4063_quartz";
    // The player runs Proviso in BOTH saved presets of the trading post (a deliberate 24/7 hold)
    // and owns Fiammetta to sustain her.
    let building = UserBuilding {
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

    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &[]);

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
        // The manager makes that possible only by STAYING in a dormitory -
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
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    const PROVISO: &str = "char_4032_provs";
    const QUARTZ: &str = "char_4063_quartz";
    // Same 24/7 preset, but NO morale-swap manager owned -> she is not held 24/7.
    let building = UserBuilding {
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

    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &[]);

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

// ─── DTO: the rest shift never keeps a main-team operator working (the "24/7" bug) ────────────

#[test]
fn rest_shift_does_not_keep_a_main_team_operator_working() {
    // An operator recommended on the MAIN shifts (1 & 3) whom the player ALSO keeps in their
    // rest-shift preset (i.e. they currently run them 24/7) must NOT be shown working the middle
    // shift via the "≈ yours" overlay - that made a main operator look 24/7. The rest shift should
    // flag them OUT instead.
    use backend::app::services::improvements::shift_rotation_to_dto;
    use backend::core::grade::base::shift_rotation::{Shift, ShiftRoom, ShiftRotation};
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let building = UserBuilding { rooms: vec![] };
    let profiles: Vec<OperatorBaseProfile> = vec![];
    let exp = |slot: &str, rec: &[&str], cur: &[&str]| ShiftRoom {
        slot_id: slot.into(),
        room_type: "MANUFACTURE".into(),
        formula_type: Some("F_EXP".into()),
        recommended: rec.iter().map(|s| (*s).to_string()).collect(),
        current: cur.iter().map(|s| (*s).to_string()).collect(),
        active: true,
        efficiency: None,
        team_id: None,
        team_label: None,
    };
    // "X" is a main operator (recommended in slots a on shifts 1 & 3) the player also keeps on the
    // rest shift. The shared P/Q makes the rest cell match the player's [X,P,Q] team deterministically.
    let main = |idx| Shift {
        index: idx,
        rooms: vec![
            exp("a", &["X", "P", "Q"], &["X", "P", "Q"]),
            exp("b", &["R", "S", "T"], &["R", "S", "T"]),
        ],
    };
    let rest = Shift {
        index: 2,
        rooms: vec![
            exp("a", &["P", "Q", "Z"], &["X", "P", "Q"]),
            exp("b", &["U", "V", "W"], &["R", "S", "T"]),
        ],
    };
    let rotation = ShiftRotation {
        shifts: vec![main(1), rest, main(3)],
        sustained: vec![],
        bench: vec![],
    };

    let dto = shift_rotation_to_dto(&rotation, gd, &profiles, &building, &registry, &drains);
    let s2 = dto
        .shifts
        .iter()
        .find(|s| s.index == 2)
        .expect("rest shift");
    let a = s2.rooms.iter().find(|r| r.slot_id == "a").expect("slot a");
    assert!(
        !a.equivalent && !a.matches,
        "the rest shift must not keep the player's main-operator team as '≈ yours'"
    );
    let shown = if a.equivalent || a.matches {
        &a.current
    } else {
        &a.recommended
    };
    assert!(
        !shown.iter().any(|o| o.operator_id == "X"),
        "main operator X must not be shown working the rest shift, got {shown:?}"
    );
    assert!(
        a.swap_out.iter().any(|o| o.operator_id == "X"),
        "main operator X should be flagged OUT (rest) on the middle shift"
    );
}

#[test]
fn reception_does_not_credit_a_co_op_conditional_without_the_partner() {
    // Vulpisfoglia's +30% only applies "together with Suzuran"; without Suzuran in the room she's a
    // +20% operator and should lose her slot to unconditional higher-skill operators - not be
    // over-credited the conditional and beat them.
    const VULPIS: &str = "char_4026_vulpis"; // +20%, plus +30% ONLY together with Suzuran
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let building = meeting_room(3); // capacity 2
    let roster: Vec<_> = [VULPIS, RED, VIGIL]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
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

#[test]
fn power_plant_is_equivalent_when_drone_recovery_matches() {
    // Two Power Plant specialists with the SAME drone-recovery % (Pudding and Indigo, both +15%)
    // are interchangeable - the diff should read "≈ yours", not a pointless swap.
    use backend::app::services::improvements::shift_rotation_to_dto;
    use backend::core::grade::base::shift_rotation::{Shift, ShiftRoom, ShiftRotation};
    const PUDDING: &str = "char_4004_pudd";
    const INDIGO: &str = "char_469_indigo";
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let profiles: Vec<_> = [PUDDING, INDIGO].iter().map(|id| profile(gd, id)).collect();
    let building = UserBuilding { rooms: vec![] };
    let rotation = ShiftRotation {
        shifts: vec![Shift {
            index: 1,
            rooms: vec![ShiftRoom {
                slot_id: "pp".into(),
                room_type: "POWER".into(),
                formula_type: None,
                recommended: vec![PUDDING.into()],
                current: vec![INDIGO.into()],
                active: true,
                efficiency: None,
                team_id: None,
                team_label: None,
            }],
        }],
        sustained: vec![],
        bench: vec![],
    };
    let dto = shift_rotation_to_dto(&rotation, gd, &profiles, &building, &registry, &drains);
    let pp = &dto.shifts[0].rooms[0];
    assert!(
        pp.equivalent,
        "Pudding and Indigo (both +15% drone recovery) should be equivalent"
    );
    assert!(
        pp.swap_in.is_empty() && pp.swap_out.is_empty(),
        "an equivalent power team suggests no swap"
    );
}

#[test]
fn base_wide_bonus_applies_only_when_the_partner_works_a_work_area() {
    // Hoederer's "+5% when Ines or W is assigned to any Work Area" must be credited only when Ines is
    // actually deployed in a work area - not merely owned, and not while she has nowhere to work.
    // With a Reception Room she works (a Work Area) and the bonus applies; without one (and no
    // production skill of her own) she can't be stationed, so it doesn't.
    const HOEDERER: &str = "char_4088_hodrer";
    const INES: &str = "char_4087_ines";
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let roster: Vec<_> = [HOEDERER, INES, "char_502_nblade", "char_185_frncat"]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let trading_eff = |building: &UserBuilding| -> f64 {
        compute_optimal_assignment(&roster, building, &gd.building, &registry, &drains)
            .rooms
            .iter()
            .filter(|r| r.room_type == "TRADING")
            .map(|r| r.total_efficiency)
            .sum()
    };
    let with_work_area = UserBuilding {
        rooms: vec![
            room("tp", "TRADING", 3),
            room("rr", "MEETING", 3),
            room("d0", "DORMITORY", 5),
        ],
    };
    let without_work_area = UserBuilding {
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 5)],
    };
    let with = trading_eff(&with_work_area);
    let without = trading_eff(&without_work_area);
    assert!(
        with > without + 1.0,
        "Hoederer's +5% should apply only when Ines works a Work Area (with={with}, without={without})"
    );
}

#[test]
fn quartz_value_reflects_distinct_recipe_types_not_factory_count() {
    // A base with FOUR factories but only TWO distinct recipe types (2 gold + 2 EXP) processes two
    // recipe types, so Quartz reads +34% (30 + 2×2), not the +38% a raw 4-factory count would give.
    use backend::core::grade::base::assignment::team_value;
    const QUARTZ: &str = "char_4063_quartz";
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let profiles = vec![profile(gd, QUARTZ)];
    let fac = |slot: &str, formula: &str| UserRoom {
        slot_id: slot.into(),
        room_type: "MANUFACTURE".into(),
        level: 3,
        current_formula: Some(formula.into()),
        ..Default::default()
    };
    let building = UserBuilding {
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
        &gd.building,
        &registry,
        &drains,
    );
    assert!(
        (v - 1.34).abs() < 0.01,
        "Quartz should read +34% (2 recipe types), not +38% (4 factories): got {v}"
    );
}

// ─── Rotation rework: login-rhythm tiling, proactive Fiammetta, alignment, leniency ──────────

#[test]
fn synergy_pairs_stay_co_teamed_and_all_three_teams_are_staffed() {
    // The balanced packer selects whole candidate teams, so a superadditive pair
    // (Texas + Lappland) is never split across teams - while a roster with enough
    // traders staffs all three of the pair-of-posts teams instead of frontloading
    // two and leaving the third hollow.
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let mut rooms = vec![room("cc", "CONTROL", 5), room("d0", "DORMITORY", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..2).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    let building = UserBuilding { rooms };
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
    .filter(|id| gd.building.chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &[]);

    // Collect the distinct trading teams by team_id.
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

#[test]
fn owning_fiammetta_proactively_sustains_the_best_trading_operator() {
    // NO preset evidence at all - owning a morale-swap manager alone makes the rotation
    // recommend holding the highest-gain trading operator (high team output x high morale
    // drain - a Shamare/Proviso-type) at full morale 24/7, pinned to one post every shift.
    // Shamare's skill burns extra morale per hour, so she gains the most from the manager.
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    const SHAMARE: &str = "char_254_vodfox";
    const TEQUILA: &str = "char_486_takila";
    const FIAMMETTA: &str = "char_300_phenxi";
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let building = UserBuilding {
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 5)],
    };
    // Six trading BODIES besides the manager: Fiammetta is reserved into the
    // dorms by the rotation itself now, so she must not double as a filler.
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
    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &[]);
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

#[test]
fn viviana_synergy_flips_the_cc_to_a_block_aligned_with_her_knights() {
    // Viviana's CC buff ("all Knight Operators in Factories +7%") links her to the factory
    // team fielding her Knights: the CC flips to a 24h block (Squad 1 on shifts 1+2) and the
    // Knight team is phased onto the same shifts-1+2 block, so the buff is live while they work.
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    const VIVIANA: &str = "char_4098_vvana";
    const WILD_MANE: &str = "char_496_wildmn"; // Knight with a factory skill
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let mut rooms = vec![room("cc", "CONTROL", 5), room("d0", "DORMITORY", 5)];
    rooms.extend((0..2).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    let building = UserBuilding { rooms };
    let roster: Vec<_> = [
        VIVIANA,
        WILD_MANE,
        "char_36_forget", // generic factory ops fill the other teams
        "char_123_fang",
        "char_133_mm",
        "char_502_nblade",
    ]
    .iter()
    .filter(|id| gd.building.chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &[]);

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
                // A dorm cell is REST, not work - the Knight rightly rests
                // shift 3 in a dormitory now that the rotation shows it.
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

#[test]
fn leniency_marks_a_close_team_equivalent_and_surfaces_the_gap() {
    // A player team within 5% of the recommendation reads "≈ yours" with the small gap
    // surfaced (gap_pct); a team further behind still gets the swap suggestion.
    use backend::app::services::improvements::shift_rotation_to_dto;
    use backend::core::grade::base::shift_rotation::{Shift, ShiftRoom, ShiftRotation};
    const NBLADE: &str = "char_502_nblade"; // +30% flat trading
    const FRNCAT: &str = "char_185_frncat"; // +30% flat trading
    const QUARTZ: &str = "char_4063_quartz"; // +30% (+recipe types, 0 here)
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let building = UserBuilding { rooms: vec![] };
    let profiles: Vec<_> = [EXUSIAI, NBLADE, FRNCAT, QUARTZ, "char_211_adnach"]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let tp = |slot: &str, rec: &[&str], cur: &[&str]| ShiftRoom {
        slot_id: slot.into(),
        room_type: "TRADING".into(),
        formula_type: None,
        recommended: rec.iter().map(|s| (*s).to_string()).collect(),
        current: cur.iter().map(|s| (*s).to_string()).collect(),
        active: true,
        efficiency: None,
        team_id: None,
        team_label: None,
    };
    let rotation = ShiftRotation {
        shifts: vec![Shift {
            index: 1,
            rooms: vec![
                // rec +35% vs yours +30%: a ~3.7% gap -> equivalent, gap surfaced.
                tp("close", &[EXUSIAI], &[FRNCAT]),
                // rec +60% vs yours +30%: far outside the band -> swap suggested.
                tp("far", &[QUARTZ, NBLADE], &["char_211_adnach"]),
            ],
        }],
        sustained: vec![],
        bench: vec![],
    };
    let dto = shift_rotation_to_dto(&rotation, gd, &profiles, &building, &registry, &drains);
    let cell = |slot: &str| {
        dto.shifts[0]
            .rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .unwrap()
    };
    let close = cell("close");
    assert!(
        close.equivalent,
        "a ~3.7% gap is within the 5% leniency band"
    );
    let gap = close.gap_pct.expect("gap surfaced");
    assert!(
        (-5.0..0.0).contains(&gap),
        "gap_pct shows the small deficit, got {gap}"
    );
    let far = cell("far");
    assert!(!far.equivalent, "a ~19% gap is a real swap suggestion");
    assert!(
        far.gap_pct.expect("gap present") < -10.0,
        "the large deficit is surfaced too"
    );
    assert!(!far.swap_in.is_empty(), "the far cell suggests the swap");
}

#[test]
fn drone_capacity_skill_scales_with_the_bases_actual_capacity() {
    // Greyy the Lightningbearer's "+1% Drone recovery rate for every 10 max Drone capacity
    // (Max +25%)" reads the base's REAL drone capacity: 100 base + 45 per L3 Power Plant.
    // On a full 243 (3x L3 plants = 235 drones) that's +23.5% - under the +25% cap, not AT it.
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    const GREYY_ALTER: &str = "char_1027_greyy2";
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let mut rooms = vec![room("d0", "DORMITORY", 5)];
    rooms.extend((0..3).map(|i| room(&format!("p{i}"), "POWER", 3)));
    let building = UserBuilding { rooms };
    let roster = vec![profile(gd, GREYY_ALTER)];
    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &[]);
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

// ─── Clause registry goldens: real gamedata buffs reduce to the expected shapes ──────────────

#[test]
fn clause_registry_goldens_on_real_gamedata() {
    use backend::core::grade::base::clause::{
        ClauseKind, CondScope, Metric, Subject, SuppressExempt, build_clauses,
    };
    let gd = load_game_data();
    let clauses = build_clauses(&gd.building.buffs, &build_name_to_char(&gd.operators));

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

    // Proviso "Damages for Breach": Pure-Gold order VALUE, not speed.
    let proviso = &clauses["trade_ord_against[010]"];
    assert!(
        proviso
            .iter()
            .any(|c| c.metric == Metric::OrderValue { pure_gold: true } && c.value > 40.0),
        "Proviso is Pure-Gold order value: {proviso:?}"
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

/// Full planner run on a REAL captured base (dumped from psql by the session
/// tooling, never committed). With the CP2 shadow live, every room evaluation
/// double-scores through the clause ledger and panics on divergence - so a
/// clean pass here proves ledger equivalence on live data, and the printed
/// summary is the diff baseline for the principled changes (CP3/CP4).
///
/// Run: `BASE_REPRO_DIR`=<dir with building_<uid>.json / roster_<uid>.json> \
///      cargo test `real_base_repro` -- --ignored --nocapture
#[test]
#[ignore = "needs a captured user dump (BASE_REPRO_DIR)"]
fn real_base_repro() {
    use backend::core::grade::base::assignment::compute_current_assignment;
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;

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
            let bc = gd.building.chars.get(&entry.operator_id)?;
            let static_op = gd.operators.get(&entry.operator_id);
            let faction_tags = static_op
                .map(backend::core::grade::base::buff_registry::faction_tags_of)
                .unwrap_or_default();
            let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
            Some(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                rarity,
                &gd.building,
                false,
            ))
        })
        .collect();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, morale_drains) = build_registry(&gd.building.buffs, &name_to_char);

    // NATIVE-FIRST (mirrors the service): every economy is priced from
    // clauses; the morale-swap manager is reserved when the roster owns both
    // a morale-conditional generator and a manager.
    let mut optimal_registry = registry.clone();
    let mut optimal_pins: Vec<(String, String)> = Vec::new();
    if profiles
        .iter()
        .any(|op| backend::core::grade::base::pools::has_morale_conditional_grant(op, &gd.building))
        && let Some(manager) =
            backend::core::grade::base::assignment::morale_swap_enabler(&profiles, &gd.building)
    {
        optimal_pins.push((manager, "DORMITORY".to_string()));
    }

    // Native pool economies, mirroring the service's wiring.
    let native = backend::core::grade::base::pools::plan_optimal_economies(
        &profiles,
        &user_building,
        &gd.building,
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
    let bundles = backend::core::grade::base::pools::candidate_bundles(
        &profiles,
        &user_building,
        &gd.building,
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
        use backend::core::grade::base::assignment::{
            assignment_value, compute_optimal_assignment_with_pins,
        };
        let baseline = compute_optimal_assignment_with_pins(
            &profiles,
            &user_building,
            &gd.building,
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
                &gd.building,
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

    let print_assignment = |label: &str, a: &backend::core::grade::base::types::BaseAssignment| {
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
        &gd.building,
        &registry,
        &morale_drains,
        None,
    );
    print_assignment("CURRENT", &current);

    let optimal = compute_optimal_assignment_with_pins(
        &profiles,
        &user_building,
        &gd.building,
        &optimal_registry,
        &morale_drains,
        &optimal_pins,
    );
    print_assignment("OPTIMAL", &optimal);

    let sustained = compute_sustained_assignment(
        &profiles,
        &user_building,
        &gd.building,
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
        &gd.building,
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
        use backend::core::grade::base::sustain_sim::simulate_rotation;
        let targeted = backend::core::grade::base::buff_registry::targeted_morale_effects(
            &gd.building.buffs,
            &name_to_char,
        );
        let report = simulate_rotation(
            &rotation,
            &profiles,
            &user_building,
            &gd.building,
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

/// NEVER-GUESS diagnostics (CP3): every buff the parser can't reduce to real
/// clauses contributes exactly ZERO and must be VISIBLE - this test prints the
/// full unresolved inventory (the work list for new parsers) and pins the
/// properties the principle demands: unresolved sets carry no value, and the
/// families the goldens prove parseable never regress into the list.
#[test]
fn unresolved_buffs_are_zero_and_inventoried() {
    use backend::core::grade::base::clause::{ClauseKind, build_clauses, unresolved_buffs};
    let gd = load_game_data();
    let clauses = build_clauses(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let unresolved = unresolved_buffs(&clauses);

    let mut by_room: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for id in &unresolved {
        let buff = &gd.building.buffs[*id];
        *by_room.entry(buff.room_type.as_str()).or_default() += 1;
        println!(
            "UNRESOLVED {:<12} {:<40} {}",
            buff.room_type, id, buff.buff_name
        );
        // Never guess: an unresolved buff's clause set carries no scoring
        // value. A morale drain may ride along - it comes from structured
        // side-map data, not from guessing at the productivity text.
        assert!(
            clauses[*id].iter().all(|c| match c.kind {
                ClauseKind::Unresolved => c.value == 0.0,
                _ => c.metric == backend::core::grade::base::clause::Metric::MoraleDrainDelta,
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

/// Deployment-context gates ("if Kal'tsit is assigned to the Control Center, drone
/// recovery +5%") parse into `ConditionalOnRoomPresence` and are credited by registry
/// rewrite exactly when the deployment stations the required operator - or enough
/// operators of the required faction - in the gate's room type. Context-free scoring
/// credits the gated part 0 (never guess).
#[test]
// Exact float equality is intentional: these are deterministic values threaded
// straight from the same constants/formulas the assignment resolver used, not
// results of independent floating-point computation.
#[allow(clippy::float_cmp)]
fn room_presence_gates_resolve_against_the_deployment() {
    use backend::core::grade::base::assignment::resolve_room_presence;
    use backend::core::grade::base::buff_registry::{BuffResolutionStrategy, build_registry};
    use backend::core::grade::base::types::OperatorBaseProfile;
    use std::collections::HashMap;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _) = build_registry(&gd.building.buffs, &name_to_char);

    // Golden parse: the four gate buffs land on the new strategy with the right context.
    let expect = [
        ("manu_formula_spd_P[000]", "TRADING", 35.0),
        ("power_rec_spd_P[000]", "CONTROL", 5.0),
        ("power_rec_spd_P[001]", "TRAINING", 5.0),
        ("power_rec_spd_ext&faction[000]", "POWER", 5.0),
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
}

/// Delphine's "for each Glasgow Gang Operator assigned to the same Trading
/// Post, +10%" is a CONDITIONAL per-operator global, not a flat tag bonus -
/// the singular "each ... Operator" phrasing must route to the same machinery
/// as `SilverAsh`'s plural form. She earns a Control-Center seat only when the
/// plan actually fields Glasgow traders; otherwise the dead-weight reselection
/// evicts her instead of crediting a +10% nobody receives.
#[test]
fn delphine_needs_glasgow_traders_to_earn_her_cc_seat() {
    use backend::core::grade::base::assignment::compute_optimal_assignment_with_pins;
    use backend::core::grade::base::buff_registry::BuffResolutionStrategy;
    const BUFF: &str = "control_tra_limit&spd[010]";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    match registry.get(BUFF) {
        Some(BuffResolutionStrategy::ConditionalGlobalEffect {
            target_room,
            faction_token,
            required_count,
            per_operator,
            bonus_pct,
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

    // The buff's owner, from gamedata.
    let delphine = gd
        .building
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
            &gd.building,
            &registry,
            &drains,
            &[],
        )
    };
    let cc_eff = |p: &backend::core::grade::base::types::BaseAssignment| {
        p.rooms
            .iter()
            .find(|r| r.room_type == "CONTROL")
            .map_or(0.0, |r| r.total_efficiency)
    };
    let tp_eff = |p: &backend::core::grade::base::types::BaseAssignment| {
        p.rooms
            .iter()
            .filter(|r| r.room_type == "TRADING")
            .map(|r| r.total_efficiency)
            .sum::<f64>()
    };

    // Only non-Glasgow traders available: her conditional can never fire.
    // A spare CC seat is fine (leftovers get parked), but she must add ZERO
    // credited value anywhere - previously the flat TagBased misparse sold
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

/// The Control-Center morale family credits an aura ONLY when the text's
/// subject really is an aura scope ("all Operators in the Control Center" /
/// "other buildings"). Named-partner gates (Mr. Lee needs Aak; the Amiya pair
/// skills) and self-subject conditionals (Gladiia's Abyssal-dependent ±0.5)
/// price 0 - a flat parse credited them all as unconditional room auras.
#[test]
fn cc_morale_auras_credit_only_true_aura_scopes() {
    use backend::core::grade::base::buff_registry::BuffResolutionStrategy;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _) = build_registry(&gd.building.buffs, &name_to_char);
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

/// The rotation's Control-Center Squad 2 must run the same dead-weight rule as
/// Squad 1. Squad 1's eviction loop drops a conditional operator whose gate the
/// planned teams never satisfy - but its exclusion list used to stay private, so
/// the Squad-2 leftover greedy re-picked the SAME operator at face value and
/// seated her on the third shift with nobody to receive the bonus (the uid
/// 09525371 report: Delphine in the shift-3 CC, zero Glasgow anywhere).
#[test]
fn rotation_squad2_evicts_conditional_cc_ops_no_team_satisfies() {
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    const BUFF: &str = "control_tra_limit&spd[010]";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let delphine = gd
        .building
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
    // Center - squad 2's leftover fill included.
    let building = UserBuilding {
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
    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &[]);
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
/// her dormitory. One swap therefore takes 24/2 = 12h to recharge - exactly one
/// login - so she can only hold an operator whose drain won't outrun that
/// cadence: drain <= 2.0/hr sustains, faster does not.
#[test]
fn fiammetta_swap_rate_bounds_who_she_can_sustain() {
    use backend::core::grade::base::dorms::{manager_can_sustain, manager_swap_rate};
    const FIAMMETTA: &str = "char_300_phenxi";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _) = build_registry(&gd.building.buffs, &name_to_char);
    let fia = profile(gd, FIAMMETTA);
    let rate = manager_swap_rate(&fia, &registry, &gd.building);
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
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    let (registry2, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let roster = full_roster(gd);
    let building = generic_base();
    let rot = recommend_shift_rotation(
        &roster,
        &building,
        &gd.building,
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

    // A base with no dormitory cannot host her at all - no pin, no sustain.
    use backend::core::grade::base::dorms::morale_manager_pin;
    let no_dorms = UserBuilding {
        rooms: vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)],
    };
    assert!(
        morale_manager_pin(&roster, &no_dorms, &gd.building).is_none(),
        "no dorm, no manager pin - owning her is not enough"
    );
}

/// Dormitory LEVELS decide recovery: the sim rests each operator at the rate
/// of the SPECIFIC dorm they land in (best dorm first), so a 2/5/2-style base
/// with under-leveled dorms genuinely recovers slower than a full-dorm 243 -
/// enough to flip a heavy drainer's verdict. A dorm-skill aura holder seated
/// as permanent staff speeds that dorm up for whoever rests beside them.
#[test]
fn dorm_levels_and_staffed_boosters_shape_recovery() {
    use backend::core::grade::base::shift_rotation::{Shift, ShiftRoom, ShiftRotation};
    use backend::core::grade::base::sustain_sim::{Verdict, simulate_rotation};
    const BODY: &str = "char_102_texas";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    // A real whole-dorm aura holder ("+X/hr to all Operators in that
    // Dormitory"), discovered from the parsed registry.
    use backend::core::grade::base::buff_registry::BuffResolutionStrategy;
    let aura_owner = gd
        .building
        .chars
        .iter()
        .find(|(_, c)| {
            c.buff_char.iter().any(|bc| {
                bc.buff_data.iter().any(|bd| {
                    matches!(
                        registry.get(&bd.buff_id),
                        Some(BuffResolutionStrategy::MoraleModifier {
                            recovery_per_hour,
                            is_self_only: false,
                            single_target: false,
                            base_wide: false,
                        }) if *recovery_per_hour > 0.0
                    ) && gd
                        .building
                        .buffs
                        .get(&bd.buff_id)
                        .is_some_and(|b| b.room_type == "DORMITORY")
                })
            })
        })
        .map(|(id, _)| id.clone())
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
            &gd.building,
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

/// The rotation ROTATES resters into the dormitories explicitly: every shift
/// emits dorm cells, off-duty workers fill them heaviest-drain-first into the
/// best dorm, unseated dorm-skill holders take permanent seats, and capacity
/// is never exceeded.
#[test]
fn rotation_emits_dorm_cells_with_resters_and_staff() {
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let roster = full_roster(gd);
    // Mixed dorm levels: the L5 must fill before the L1s (a 2/5/2 shape).
    let mut rooms = vec![room("cc", "CONTROL", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.push(room("d_hi", "DORMITORY", 5));
    rooms.push(room("d_lo1", "DORMITORY", 1));
    rooms.push(room("d_lo2", "DORMITORY", 1));
    let building = UserBuilding { rooms };
    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &[]);

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
/// clocks in the plan - the same working-drain offset the simulator charges,
/// not a dorm-rest bonus.
#[test]
fn cc_base_wide_recovery_stretches_swap_clocks() {
    use backend::core::grade::base::assignment::compute_sustained_assignment;
    use backend::core::grade::base::buff_registry::BuffResolutionStrategy;
    const BODY: &str = "char_102_texas";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    // A real owner of a base-wide CC recovery aura, from the parsed registry.
    let holder = gd
        .building
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
        rooms: vec![
            room("cc", "CONTROL", 5),
            room("tp", "TRADING", 3),
            room("d0", "DORMITORY", 2),
        ],
    };
    let body_hours = |ids: &[&str]| -> f64 {
        let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        let plan =
            compute_sustained_assignment(&roster, &building, &gd.building, &registry, &drains);
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

/// Waaifu's Team Spirit makes her IMMUNE to teammates' room morale auras
/// (Shu's -0.1/hr factory aura passes her by, in either direction), and
/// Cement's Vlog cuts her drain by 0.25/hr only while the factory produces
/// Battle Records - both now charged by the sustainability sim.
#[test]
fn aura_immunity_and_formula_drain_shape_the_sim() {
    use backend::core::grade::base::buff_registry::targeted_morale_effects;
    use backend::core::grade::base::shift_rotation::{Shift, ShiftRoom, ShiftRotation};
    use backend::core::grade::base::sustain_sim::simulate_rotation;
    const WAAIFU: &str = "char_243_waaifu";
    const CEMENT: &str = "char_328_cammou";
    const SHU: &str = "char_2025_shu"; // owns the -0.1 factory aura
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let targeted = targeted_morale_effects(&gd.building.buffs, &name_to_char);
    let building = UserBuilding {
        rooms: vec![room("mf", "MANUFACTURE", 3), room("d0", "DORMITORY", 2)],
    };
    // A 36h unbroken factory stretch: depletion hour = 24 / effective drain.
    let mk = |crew: Vec<&str>, formula: &str| ShiftRotation {
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
            &gd.building,
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

/// Targeted morale effects between co-seated operators - the Ave Mujica
/// drama: Dolris' rider drains Sakiko +0.1/hr while they share the Control
/// Center, and Mortis' amnesty cancels Sakiko's OWN +0.05 rider. The sim
/// charges each exactly when the pair is co-seated.
#[test]
fn targeted_morale_effects_follow_co_seating() {
    use backend::core::grade::base::buff_registry::targeted_morale_effects;
    use backend::core::grade::base::shift_rotation::{Shift, ShiftRoom, ShiftRotation};
    use backend::core::grade::base::sustain_sim::simulate_rotation;
    const SAKIKO: &str = "char_4182_oblvns";
    const MORTIS: &str = "char_4183_mortis";
    const DOLRIS: &str = "char_4184_dolris";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let targeted = targeted_morale_effects(&gd.building.buffs, &name_to_char);

    let building = UserBuilding {
        rooms: vec![room("cc", "CONTROL", 5), room("d0", "DORMITORY", 2)],
    };
    // A 36h unbroken CC stretch, so depletion hour = 24 / effective drain.
    let mk = |crew: Vec<&str>| ShiftRotation {
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
            &gd.building,
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

/// The Control Center's own recovery auras (`control_mp_cost`: "+0.05/hr to
/// all Operators in the Control Center") offset its workers' drain in the
/// sustainability sim - and ONLY its workers: the aura is room-local, so a
/// trading-post worker in the same rotation gains nothing from it.
#[test]
fn cc_recovery_auras_offset_cc_workers_drain_only() {
    use backend::core::grade::base::shift_rotation::{Shift, ShiftRoom, ShiftRotation};
    use backend::core::grade::base::sustain_sim::simulate_rotation;
    const BODY_CC: &str = "char_102_texas";
    const BODY_TP: &str = "char_140_whitew";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    // A real owner of the CC-room recovery aura, from gamedata.
    let aura_owner = gd
        .building
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
        rooms: vec![
            room("cc", "CONTROL", 5),
            room("tp", "TRADING", 3),
            room("d0", "DORMITORY", 2),
        ],
    };
    // Everyone works ALL THREE shifts (36h unbroken): a neutral 1.0/hr drain
    // depletes at exactly 24h; a -0.05 aura stretches that to 24/0.95 ≈ 25.3h.
    let mk = |cc_crew: Vec<String>| ShiftRotation {
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
            &gd.building,
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

/// Pool-scaled Control-Center globals (Sakiko's "all Trading Posts +1% per 8
/// Passion", the Mortis "all Factories +1% (+1% per 20)") parse into
/// `GlobalPoolScaling` and resolve by registry rewrite against settled pool
/// points - floored steps, base always on - exactly like the other
/// deployment-dependent rewrites. Context-free scoring credits the base only.
#[test]
// Exact float equality is intentional: these are deterministic values threaded
// straight from the same constants/formulas the resolver used, not results of
// independent floating-point computation.
#[allow(clippy::float_cmp)]
fn global_pool_consumers_resolve_against_settled_points() {
    use backend::core::grade::base::assignment::resolve_global_pool;
    use backend::core::grade::base::buff_registry::{BuffResolutionStrategy, build_registry};
    use std::collections::HashMap;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _) = build_registry(&gd.building.buffs, &name_to_char);

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

/// The planner's `lasts_hours` reads the same room-wide drain-aura model the
/// sustainability simulator charges: a trading-post aura carrier stretches
/// every TEAMMATE's swap clock, not just its own - previously the display
/// said "swap Texas at 24h" while the simulator knew she'd last far longer.
#[test]
fn lasts_hours_reads_the_room_drain_aura() {
    use backend::core::grade::base::assignment::compute_sustained_assignment;
    const BODY: &str = "char_102_texas";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    // A real owner of the trading drain-reduction aura, from gamedata.
    let aura_owner = gd
        .building
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
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 2)],
    };
    let hours_of = |ids: &[&str]| -> f64 {
        let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        let plan =
            compute_sustained_assignment(&roster, &building, &gd.building, &registry, &drains);
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

/// The 3-shift rotation runs on the same economy plan as the optimal view: a
/// consumer's `PoolPayoff` override prices into rotation team selection, and a
/// pinned Control-Center generator (Ling for the Sui economy) holds a Squad-1
/// seat - two of the three shifts, the same realistic uptime perception priced
/// the payoff at (all three would be a 36h stretch no morale bar survives) -
/// and is never burned as a production filler.
#[test]
fn rotation_seats_economy_pins_and_credits_payoffs() {
    use backend::core::grade::base::buff_registry::BuffResolutionStrategy;
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    const LING: &str = "char_2023_ling";
    const SHU: &str = "char_2025_shu";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (mut registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    // The solved consumer payoff the pool plans would commit (Shu's factory
    // skill priced at its Sui-economy value).
    registry.insert(
        "manu_prod_spd_bd[300]".to_string(),
        BuffResolutionStrategy::PoolPayoff { pct: 40.0 },
    );
    let roster = full_roster(gd);
    let building = generic_base();
    let pins = vec![(LING.to_string(), "CONTROL".to_string())];
    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &pins);

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
        // A dormitory cell is REST, not a seat spent - the generator rightly
        // rests her off shift there now that the rotation shows it.
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

/// Non-production Control-Center skills (clue / training / HR) parse into
/// `ControlNonProduction` - priced in their OWN units with zero LMD weight (the
/// objective stays production-pure, matching both reference implementations) -
/// and break ties for SPARE CC seats with the reference priority: clue > HR >
/// training. "Only the strongest effect of this type" makes ONE family per
/// boosted metric, across different buff-id prefixes.
#[test]
// Exact float equality is intentional: the expected value is threaded straight
// from the same constant the resolver used, not an independent computation.
#[allow(clippy::float_cmp)]
fn cc_non_production_skills_parse_and_break_spare_seat_ties() {
    use backend::core::grade::base::assignment::fill_remaining_slots;
    use backend::core::grade::base::buff_registry::{BuffResolutionStrategy, build_registry};
    use backend::core::grade::base::clause::build_clauses;
    use backend::core::grade::base::types::OperatorBaseProfile;
    use std::collections::HashSet;

    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _) = build_registry(&gd.building.buffs, &name_to_char);

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
    let clauses = build_clauses(&gd.building.buffs, &name_to_char);
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
        &gd.building,
        &registry,
        &mut assigned,
    );
    assert_eq!(seats, ["clue_op"], "the clue holder wins the spare seat");

    // Display aggregation: "only the strongest effect of this type" means the
    // crew's clue total is the MAX (+15), never the sum (+20); the Sakiko
    // same-room gate only counts while she shares the Control Center.
    use backend::core::grade::base::assignment::cc_non_production_effects;
    let crew_ops = [
        bare("op_a", vec!["control_upMeetingSpeed[100]".to_string()]), // clue +15
        bare("op_b", vec!["control_meeting_spd&bd[000]".to_string()]), // clue +5
        bare("op_c", vec!["control_mp&meet_spd[000]".to_string()]),    // clue +5, needs Sakiko
    ];
    let ids = |v: &[&str]| v.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
    let effects = |crew: &[String]| cc_non_production_effects(crew, &crew_ops, &registry);
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

/// The robot power-plant economy is a BUNDLE: Alanna's Operation Platforms only
/// pay when Robot-tagged operators hold Power Plant seats, so the generator
/// pins the roster's robots (capped at the building's real power capacity) and
/// prices her factory skill at the solved payoff. The improvements-level oracle
/// then weighs that feed against the drone specialists the pins displace -
/// a trade that only became priceable once POWER entered `assignment_value`.
#[test]
// Exact float equality is intentional: `10.0` is the literal per-platform bonus
// the bundle resolver is expected to price, not an independent computation.
#[allow(clippy::float_cmp)]
fn alanna_bundle_pins_robots_into_power_seats() {
    use backend::core::grade::base::pools::candidate_bundles;
    const ALANNA: &str = "char_4178_alanna";
    const CASTLE3: &str = "char_286_cast3";
    const LANCET2: &str = "char_285_medic2";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _) = build_registry(&gd.building.buffs, &name_to_char);
    let ops: Vec<OperatorBaseProfile> = [ALANNA, CASTLE3, LANCET2]
        .iter()
        .map(|id| profile(gd, id))
        .collect();

    // One L3 plant = one seat: the bundle must cap its pins there even with
    // two robots in the roster.
    let building = UserBuilding {
        rooms: vec![room("p0", "POWER", 3), room("mf0", "MANUFACTURE", 3)],
    };
    let bundles = candidate_bundles(&ops, &building, &gd.building, &registry);
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
        rooms: vec![room("mf0", "MANUFACTURE", 3)],
    };
    assert!(
        candidate_bundles(&ops, &no_power, &gd.building, &registry)
            .iter()
            .all(|b| b.pins.iter().all(|(_, rt)| rt != "POWER")),
        "no plants means no robot pins"
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
    use backend::core::grade::base::assignment::assignment_value;
    use backend::core::grade::base::types::RoomAssignment;
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
    // Two plants stack linearly - no coupling, unlike the gold->trade loop.
    let both = assignment_value(&[power(20.0), power(23.5)]);
    assert!(((both - without) - 2175.0).abs() < 1e-9);
}

/// The trading-post ORDER LIMIT baseline scales with the post's level (6/8/10 for
/// L1-L3, gamedata `TradingData.Phases`), so a capacity-slashing operator is
/// throttled against the REAL buffer, not a level-blind constant. Degenbrecher
/// (-6 order limit) on an L3 post leaves 4 of 10 orders (factor 0.4); on an L1
/// post she hits the floor of 1 of 6. The level-blind model scored the L3 case
/// at ~0.17 - over-penalizing her on exactly the posts endgame players run.
#[test]
fn trading_order_cap_scales_with_post_level() {
    const DEGENBRECHER: &str = "char_4116_blkkgt";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let ops = vec![profile(gd, DEGENBRECHER)];
    let team = vec![DEGENBRECHER.to_string()];

    let value_at = |level: i32| {
        let building = UserBuilding {
            rooms: vec![room("tp0", "TRADING", level)],
        };
        team_value(
            &team,
            "TRADING",
            None,
            &ops,
            &building,
            &gd.building,
            &registry,
            &drains,
        )
    };

    // L3: +25% efficiency x (10-6)/10 capacity = 1.25 x 0.4 = 0.5 objective.
    let l3 = value_at(3);
    assert!(
        (l3 - 0.5).abs() < 0.02,
        "L3 post: Degenbrecher throttles to 4/10 of the level's base cap, got {l3:.4}"
    );
    // L1: 6-6 clamps to the floor of 1 -> 1/6 capacity. Strictly worse than L3.
    let l1 = value_at(1);
    assert!(
        (l1 - 1.25 / 6.0).abs() < 0.02,
        "L1 post: the -6 clamps to the 1-order floor of a 6-cap post, got {l1:.4}"
    );
    assert!(
        l3 > l1,
        "a bigger buffer absorbs the slash better: {l3:.4} vs {l1:.4}"
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
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let ops: Vec<OperatorBaseProfile> = [WEEDY, TEXAS, LAPPLAND]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let building = UserBuilding {
        rooms: vec![room("tp0", "TRADING", 3)],
    };
    let with_weedy = team_value(
        &[TEXAS.to_string(), LAPPLAND.to_string(), WEEDY.to_string()],
        "TRADING",
        None,
        &ops,
        &building,
        &gd.building,
        &registry,
        &drains,
    );
    let without = team_value(
        &[TEXAS.to_string(), LAPPLAND.to_string()],
        "TRADING",
        None,
        &ops,
        &building,
        &gd.building,
        &registry,
        &drains,
    );
    // Weedy adds nothing in a Trading Post, but she must not DESTROY the
    // Texas+Lappland synergy either - the pair's value survives her presence.
    assert!(
        with_weedy >= without - 1e-9,
        "Weedy in a trading post must not wipe teammates: with={with_weedy:.4} without={without:.4}"
    );
    assert!(
        without > 1.5,
        "sanity: Texas+Lappland is a strong trading pair ({without:.4})"
    );
}

/// 2/5/2 layout (2 trading posts, 5 factories, 2 power plants): the rotation
/// machinery is layout-generic - groups, gold split, tiling and power squads
/// all derive from the actual rooms - so the second standard endgame layout
/// gets a full 3-shift plan. Pins: the 3-gold/2-EXP split (2 posts' gold
/// demand + 1), the trading login rhythm, alternating power/CC squads, and no
/// operator double-booked within a shift.
#[test]
fn shift_rotation_supports_252_layout() {
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let mut rooms = vec![
        room("cc", "CONTROL", 5),
        room("hr", "HIRE", 3),
        room("rc", "MEETING", 3),
    ];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..5).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..2).map(|i| room(&format!("p{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding { rooms };
    let rot = recommend_shift_rotation(
        &full_roster(gd),
        &building,
        &gd.building,
        &registry,
        &drains,
        &[],
    );

    assert_eq!(rot.shifts.len(), 3, "three shifts");
    let cell =
        |shift: usize, slot: &str| -> &backend::core::grade::base::shift_rotation::ShiftRoom {
            rot.shifts[shift]
                .rooms
                .iter()
                .find(|r| r.slot_id == slot)
                .unwrap_or_else(|| panic!("shift {shift} missing slot {slot}"))
        };

    // Factory split: two L3 posts sell roughly two factories' worth of bars, so
    // the marginal 5th factory goes to EXP (unsold gold is worthless under the
    // coupled gold→trade yield) - the objective picks 2 gold / 3 EXP.
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
    assert_eq!(team_id(0, "tp0"), team_id(1, "tp0"), "tp0 works shifts 1+2");
    assert_ne!(
        team_id(1, "tp0"),
        team_id(2, "tp0"),
        "tp0 swaps for shift 3"
    );
    assert_eq!(team_id(1, "tp1"), team_id(2, "tp1"), "tp1 works shifts 2+3");

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
/// it - so the recommended plan holds up, and the verdict says so honestly.
#[test]
fn recommended_rotation_survives_its_own_morale_sim() {
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    use backend::core::grade::base::sustain_sim::{Verdict, simulate_rotation};
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
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
        let building = UserBuilding { rooms };
        let roster = full_roster(gd);
        let rot =
            recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &[]);
        let report = simulate_rotation(
            &rot,
            &roster,
            &building,
            &gd.building,
            &registry,
            &drains,
            &std::collections::HashMap::new(),
        );
        // With L5 dorms a 24h block is sustainable at exactly baseline drain
        // (24 drained, 12h x 2.0 recovered), and the planner routes operators
        // whose skills drain faster than baseline into single-shift Squad-2
        // seats - so the recommended plan must fully hold up.
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
        // Dorm pressure, not dorm comfort. This asserted exactly 0 until the
        // empty-team backfill landed: the 5-factory case used to leave one
        // factory unstaffed, and an empty room needs no beds - so the old zero
        // was partly an artefact of the bug. Staffing it adds a crew that has
        // to rest somewhere, and 4xL5 dorms (20 beds) run short at peak - one
        // bed for the extra crew, one more for Fiammetta's permanent seat now
        // that the rotation genuinely parks her in a dormitory. The invariant
        // that matters is the verdict above (nobody actually runs dry over the
        // week); overflow is a pressure gauge, so hold it to a small bound.
        assert!(
            report.dorm_overflow <= 2,
            "{factories}-factory dorms are {} beds short at peak",
            report.dorm_overflow
        );
    }
}

/// The simulator flags an UNSUSTAINABLE rhythm: with L1 dorms (+1.6/hr), a 12h
/// rest recovers only ~19 of the 24 points a 24h block drains, so production
/// operators leak ~5 morale per cycle and run dry within the week - the
/// verdict must say "depletes" and name them, not wave the plan through.
#[test]
fn morale_sim_flags_underbuilt_dorms() {
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    use backend::core::grade::base::sustain_sim::{Verdict, simulate_rotation};
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let mut rooms = vec![
        room("cc", "CONTROL", 5),
        room("hr", "HIRE", 3),
        room("rc", "MEETING", 3),
    ];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("p{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 1)));
    let building = UserBuilding { rooms };
    let roster = full_roster(gd);
    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains, &[]);
    let report = simulate_rotation(
        &rot,
        &roster,
        &building,
        &gd.building,
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

/// The first NATIVE pool economy: Minimalist's Engineering Robots. Her E0
/// generator makes +1 robot per functional-facility level (max 64 - exactly a
/// maxed 243's level sum) and her E2 consumer converts every 8 robots into +5%
/// productivity, floored. Generator and consumer travel together, so the pool
/// settles room-locally from the layout alone: on a maxed base she is a
/// +40% solo factory operator; on a smaller base the value floors down. These
/// buffs scored ZERO before (unresolved) - honest, but she was invisible.
#[test]
fn minimalist_engineering_robots_settle_from_the_layout() {
    use backend::core::grade::base::clause::{ClauseKind, build_clauses};
    const MINIMALIST: &str = "char_4054_malist";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    // The parse: generator + stepped consumer, no longer Unresolved.
    let clauses = build_clauses(&gd.building.buffs, &name_to_char);
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
        let building = UserBuilding { rooms };
        team_value(
            &team,
            "MANUFACTURE",
            Some("F_GOLD"),
            &ops,
            &building,
            &gd.building,
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
/// gain exactly +5% over the identical base without him - and with him absent
/// the pool reads zero, never a guess.
#[test]
fn senshi_monster_meals_settle_against_live_seats() {
    use backend::core::grade::base::assignment::compute_current_assignment;
    const SENSHI: &str = "char_4143_sensi";
    const MARCILLE: &str = "char_4141_marcil";
    const CHILCHUCK: &str = "char_4144_chilc";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
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
            rooms: vec![room("cc", "CONTROL", 5), dorm, mf, tp],
        }
    };

    let eff = |asn: &backend::core::grade::base::types::BaseAssignment, slot: &str| {
        asn.rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .map_or_else(|| panic!("missing {slot}"), |r| r.total_efficiency)
    };
    let with =
        compute_current_assignment(&ops, &build(true), &gd.building, &registry, &drains, None);
    let without =
        compute_current_assignment(&ops, &build(false), &gd.building, &registry, &drains, None);

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

/// Dorm-occupant pools settle against live seats end to end:
/// - Mr. Nothing's one-buff trading economy: each resting operator grants a
///   Worldly Plight point and his own skill converts them to order efficiency.
/// - Rosmontis' Extrasensory CHAIN: dorm occupants feed Perception
///   Information, which converts 1:1 into Chain of Thought, which her E2
///   consumer drains at +1% per point - two pools, one fixed-point pass.
/// - A CROSS-OPERATOR chain: Mr. Nothing generates Worldly Plight from the
///   dorms while a factory teammate's converter turns every 5 into a
///   Witchcraft Crystal worth +2% - generation, conversion and consumption in
///   three different rooms.
#[test]
fn dorm_fed_pools_and_conversion_chains_settle() {
    use backend::core::grade::base::assignment::compute_current_assignment;
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
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
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
            rooms: vec![room("cc", "CONTROL", 5), dorm, mf, tp],
        }
    };
    let eff = |asn: &backend::core::grade::base::types::BaseAssignment, slot: &str| {
        asn.rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .map_or_else(|| panic!("missing {slot}"), |r| r.total_efficiency)
    };
    let run = |factory_op: &str, resting: &[&str]| {
        compute_current_assignment(
            &ops,
            &assemble(factory_op, resting),
            &gd.building,
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

    // Cross-operator chain: 4 resting -> 4 WP is below the 5-per-crystal
    // ratio (floor 0), so the witch factory gains nothing yet... but with all
    // seven ops resting? Use the four bodies + swap Rosmontis into the dorm:
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

/// The Control-Center generator SIDE-CHANNEL: Dusk/Ling/Chongyue's CC buffs
/// carry a morale aura (which the parser owns) AND a pool grant - extracted
/// separately by the settlement. Seating the three siblings in the CC with Shu
/// deployed makes 4 deployed Sui: Chongyue grants min(4,5)x5 = 20 Worldly
/// Plight, Dusk and Ling's "above/below 12 morale" branches settle at the 0.5
/// steady-state fraction (+7.5 each) - 35 points, and Shu's factory skill
/// drains them at +1% per 3: exactly +11%.
#[test]
fn sui_worldly_plight_flows_from_the_control_center() {
    use backend::core::grade::base::assignment::compute_current_assignment;
    const CHONGYUE: &str = "char_2024_chyue";
    const DUSK: &str = "char_2015_dusk";
    const LING: &str = "char_2023_ling";
    const SHU: &str = "char_2025_shu";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
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
            rooms: vec![cc, mf, room("d0", "DORMITORY", 5)],
        }
    };
    let factory_eff = |cc_ops: &[&str]| {
        let asn = compute_current_assignment(
            &ops,
            &assemble(cc_ops),
            &gd.building,
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
    use backend::core::grade::base::assignment::compute_current_assignment;
    const ALANNA: &str = "char_4178_alanna";
    const CASTLE3: &str = "char_286_cast3";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
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
            rooms: vec![room("cc", "CONTROL", 5), pp, mf],
        }
    };
    let factory_eff = |robot: bool| {
        let asn = compute_current_assignment(
            &ops,
            &assemble(robot),
            &gd.building,
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
/// The planner solves Senshi's Monster Meals ahead of the search - pinning him
/// into the best dormitory and pricing Marcille's consumer at the settled
/// +5% - so the optimizer SEATS Marcille in the factory over a plain +2%
/// operator it would otherwise prefer, and reserves Senshi's dorm seat. The
/// honesty rule holds throughout: consumers fed by unpinned third parties get
/// no credit.
#[test]
fn optimal_search_seats_the_monster_meal_economy() {
    use backend::core::grade::base::assignment::compute_optimal_assignment_with_pins;
    use backend::core::grade::base::buff_registry::BuffResolutionStrategy;
    use backend::core::grade::base::pools::plan_optimal_economies;
    const SENSHI: &str = "char_4143_sensi";
    const MARCILLE: &str = "char_4141_marcil";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let ops: Vec<OperatorBaseProfile> = [SENSHI, MARCILLE]
        .iter()
        .map(|id| profile(gd, id))
        .collect();

    let mut mf = room("mf0", "MANUFACTURE", 3);
    mf.current_formula = Some("F_GOLD".to_string());
    let building = UserBuilding {
        rooms: vec![room("cc", "CONTROL", 5), mf, room("d0", "DORMITORY", 5)],
    };

    let plan = plan_optimal_economies(&ops, &building, &gd.building, &registry);
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
        &gd.building,
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
/// +12% at E2. The caller then judges the bundle by total yield - the
/// optimizer itself prices the three CC seats it costs.
#[test]
fn sui_bundle_prices_the_cc_economy_as_a_package() {
    use backend::core::grade::base::pools::candidate_bundles;
    const CHONGYUE: &str = "char_2024_chyue";
    const DUSK: &str = "char_2015_dusk";
    const LING: &str = "char_2023_ling";
    const SHU: &str = "char_2025_shu";
    const WITCH: &str = "char_4078_bdhkgt";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _drains) = build_registry(&gd.building.buffs, &name_to_char);
    let ops: Vec<OperatorBaseProfile> = [CHONGYUE, DUSK, LING, SHU, WITCH]
        .iter()
        .map(|id| profile(gd, id))
        .collect();

    let bundles = candidate_bundles(
        &ops,
        &UserBuilding { rooms: Vec::new() },
        &gd.building,
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

/// PARITY HARNESS vs the reference optimizer: rebuild the exact 2/5/2 fixture
/// the reference solved for this roster (2 gold-strategy posts, 3 gold + 2 EXP
/// factories, 2 plants, low dorms) and print OUR optimal seating for the same
/// rooms - the diff against the captured reference answer attributes every
/// divergence to a coverage gap or a judgment difference.
#[test]
#[ignore = "needs a captured user dump (BASE_REPRO_DIR)"]
fn reference_parity_252() {
    use backend::core::grade::base::assignment::compute_optimal_assignment_with_pins;
    let dir = std::env::var("BASE_REPRO_DIR").expect("set BASE_REPRO_DIR");
    let uid = std::env::var("BASE_REPRO_UID").unwrap_or_else(|_| "89153800".into());
    let roster: Vec<RosterEntry> =
        serde_json::from_str(&std::fs::read_to_string(format!("{dir}/roster_{uid}.json")).unwrap())
            .unwrap();
    let gd = load_game_data();
    let profiles: Vec<OperatorBaseProfile> = roster
        .iter()
        .filter_map(|entry| {
            let bc = gd.building.chars.get(&entry.operator_id)?;
            let static_op = gd.operators.get(&entry.operator_id);
            let faction_tags = static_op
                .map(backend::core::grade::base::buff_registry::faction_tags_of)
                .unwrap_or_default();
            let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
            Some(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                rarity,
                &gd.building,
                false,
            ))
        })
        .collect();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

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
    let building = UserBuilding { rooms };

    // Full service wiring: native economies + bundles via the oracle.
    let mut optimal_registry = registry.clone();
    let mut optimal_pins: Vec<(String, String)> = Vec::new();
    let native = backend::core::grade::base::pools::plan_optimal_economies(
        &profiles,
        &building,
        &gd.building,
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
        &gd.building,
        &optimal_registry,
        &drains,
        &optimal_pins,
    );
    for bundle in backend::core::grade::base::pools::candidate_bundles(
        &profiles,
        &building,
        &gd.building,
        &registry,
    ) {
        use backend::core::grade::base::assignment::assignment_value;
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
            &gd.building,
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

/// Morale AURAS flow through the simulator: a teammate whose skill slows the
/// whole room's drain ("-0.1/hr to all Operators in the Trading Post") and a
/// Control-Center aura lifting every dormitory sleeper's recovery both change
/// the week's morale arithmetic. With L2 dorms (+1.7/hr), a 24h block drains
/// 24 but a 12h rest recovers only 20.4 - operators leak and deplete; the
/// room aura cuts the block's drain to 21.6, which the rest covers.
#[test]
fn morale_auras_change_the_sustainability_arithmetic() {
    use backend::core::grade::base::shift_rotation::{Shift, ShiftRoom, ShiftRotation};
    use backend::core::grade::base::sustain_sim::{Verdict, simulate_rotation};
    const AURA: &str = "char_4171_wulfen"; // Warm-Up owner? resolved below
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

    // Find a real owner of the trading drain aura from gamedata - no hardcoded id.
    let aura_owner = gd
        .building
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
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 2)],
    };
    // A hand-built 3-shift rotation: the trio works shifts 1+2 (a 24h block)
    // and rests shift 3 - the exact rhythm the planner emits.
    let mk_rotation = |crew: Vec<String>| ShiftRotation {
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
        &gd.building,
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
        &gd.building,
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

/// A production room must never stand dark while operators who could staff it
/// are still free.
///
/// Regression: rotation teams are enumerated with `require_24h_sustain`, which
/// drops every operator whose morale drain outpaces the bar. On a wide base
/// (5 factories + 2 trading posts) the sustainable pool ran dry, the beam left
/// the third team of the trading group empty, and the tiling stood a Trading
/// Post dark for two of three shifts - while Lappland, Texas and friends sat
/// unused. An unstaffed production room yields nothing at all; a heavy drainer
/// yields at full rate until their bar empties, and morale is recoverable.
#[test]
fn shift_rotation_never_rests_a_production_room_with_candidates_to_spare() {
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);

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
    let building = UserBuilding { rooms };

    let rot = recommend_shift_rotation(
        &full_roster(gd),
        &building,
        &gd.building,
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

    // And the crews must still be genuinely disjoint within a shift - backfilling
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

/// Dump-driven diagnostic: the planner's plain-optimal path AND the rotation
/// on a captured base, printing Control-Center / trading crews and where
/// Delphine-class conditionals land.
/// Run: `BASE_REPRO_DIR=<dir> BASE_REPRO_UID=<uid> cargo test -- planner_probe --ignored --nocapture`
#[test]
#[ignore = "needs a captured user dump (BASE_REPRO_DIR)"]
fn ledger_probe() {
    use backend::core::grade::base::assignment::compute_current_assignment;
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
            let bc = gd.building.chars.get(&entry.operator_id)?;
            let static_op = gd.operators.get(&entry.operator_id);
            let faction_tags = static_op
                .map(backend::core::grade::base::buff_registry::faction_tags_of)
                .unwrap_or_default();
            let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
            Some(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                rarity,
                &gd.building,
                false,
            ))
        })
        .collect();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let asn =
        compute_current_assignment(&profiles, &building, &gd.building, &registry, &drains, None);
    for r in &asn.rooms {
        if let Some(f) = &r.fill {
            println!(
                "FILL {} {} {:?} cap {} fill {:.1}h",
                r.slot_id,
                r.room_type,
                r.formula_type.as_deref().unwrap_or("-"),
                f.capacity,
                f.fill_hours
            );
        }
        if r.room_type != "TRADING" && r.room_type != "CONTROL" {
            continue;
        }
        println!(
            "== {} {} eff {:.1} value {:.1}",
            r.slot_id, r.room_type, r.total_efficiency, r.order_value
        );
        for l in &r.ledger {
            println!(
                "   {:22} {:32} speed {:+7.2} value {:+6.2} {:?}{}",
                l.operator_id,
                l.buff_id,
                l.speed_pct,
                l.value_pct,
                l.disposition,
                if l.from_control_center { "  [CC]" } else { "" }
            );
        }
    }
}

#[test]
#[ignore]
fn planner_probe() {
    use backend::core::grade::base::assignment::compute_optimal_assignment_with_pins;
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
            let bc = gd.building.chars.get(&entry.operator_id)?;
            let static_op = gd.operators.get(&entry.operator_id);
            let faction_tags = static_op
                .map(backend::core::grade::base::buff_registry::faction_tags_of)
                .unwrap_or_default();
            let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
            // IGNORE_PROMOTION=0 plans at real elite levels; default E2-max.
            let ignore_promo = std::env::var("IGNORE_PROMOTION").as_deref() != Ok("0");
            Some(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                rarity,
                &gd.building,
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
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let plan = compute_optimal_assignment_with_pins(
        &profiles,
        &building,
        &gd.building,
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
        backend::core::grade::base::dorms::morale_manager_pin(&profiles, &building, &gd.building)
        && !pins.iter().any(|(id, _)| id == &pin.0)
    {
        pins.push(pin);
    }
    let rot = backend::core::grade::base::shift_rotation::recommend_shift_rotation(
        &profiles,
        &building,
        &gd.building,
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
    let targeted = backend::core::grade::base::buff_registry::targeted_morale_effects(
        &gd.building.buffs,
        &name_to_char,
    );
    let sim = backend::core::grade::base::sustain_sim::simulate_rotation(
        &rot,
        &profiles,
        &building,
        &gd.building,
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

/// The deep-dive skill ledger: marginals by ablation, with coupled skills
/// (Feud + Jaye's mirroring), pair riders, CC globals and capacity-only lines
/// all classified honestly.
#[test]
fn skill_ledger_reports_marginals_and_dispositions() {
    use backend::core::grade::base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let mut building = UserBuilding {
        rooms: vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)],
    };
    building.rooms[0].current_operators = vec!["char_4132_ascln".into()];
    building.rooms[1].current_operators = vec![
        "char_140_whitew".into(),
        "char_102_texas".into(),
        "char_272_strong".into(),
    ];
    let roster: Vec<OperatorBaseProfile> = [
        "char_4132_ascln",
        "char_140_whitew",
        "char_102_texas",
        "char_272_strong",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .collect();
    let asn =
        compute_current_assignment(&roster, &building, &gd.building, &registry, &drains, None);
    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("trading room scored");
    let line = |op: &str, buff: &str| {
        tp.ledger
            .iter()
            .find(|l| l.operator_id == op && l.buff_id == buff)
            .unwrap_or_else(|| panic!("no ledger line for {op} {buff}"))
    };
    use backend::core::grade::base::skill_ledger::LineDisposition;
    // Texas' Feud fires with Lappland present. Its marginal EXCEEDS its face
    // +65 because removing it also collapses Jaye's teammate-mirroring - the
    // ledger reports what the room actually loses.
    let feud = line("char_102_texas", "trade_ord_spd&cost_P[000]");
    assert_eq!(feud.disposition, LineDisposition::Contributes);
    assert!(
        feud.speed_pct >= 65.0 - 1e-6,
        "Feud marginal {} >= 65",
        feud.speed_pct
    );
    // Jaye's own mirroring line contributes.
    let jaye = line("char_272_strong", "trade_ord_limit_diff[000]");
    assert_eq!(jaye.disposition, LineDisposition::Contributes);
    assert!(jaye.speed_pct > 0.0);
    // Lappland's Texas-gated order-limit skill: zero efficiency marginal, but
    // the gate is met and it moves capacity - "capacity", not "inactive".
    let lapp = line("char_140_whitew", "trade_ord_limit&cost_P[001]");
    assert_eq!(lapp.disposition, LineDisposition::CapacityOnly);
    // Ascalon's CC-wide +7% shows on the post as a Control-Center line.
    let cc_line = line("char_4132_ascln", "control_tra_spd[030]");
    assert!(cc_line.from_control_center);
    assert_eq!(cc_line.disposition, LineDisposition::Contributes);
    assert!((cc_line.speed_pct - 7.0).abs() < 1e-6);
}

#[test]
#[ignore]
fn ledger_probe_registry_peek() {
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _) = build_registry(&gd.building.buffs, &name_to_char);
    for id in ["trade_ord_limit&cost_P[001]", "trade_ord_limit&cost_P[000]"] {
        println!("{id} => {:?}", registry.get(id));
    }
}

/// The sim's per-facility totals: a room dark two of three shifts logs 2/3 of
/// the horizon as idle and produces only for its staffed blocks; a fully
/// staffed room logs none.
#[test]
fn sim_facility_totals_count_dark_shifts_as_idle() {
    use backend::core::grade::base::shift_rotation::{Shift, ShiftRoom, ShiftRotation};
    use backend::core::grade::base::sustain_sim::simulate_rotation;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let targeted = backend::core::grade::base::buff_registry::targeted_morale_effects(
        &gd.building.buffs,
        &name_to_char,
    );
    const BODY: &str = "char_103_angel";
    let rotation = ShiftRotation {
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
        rooms: vec![room("tp", "TRADING", 3), room("d0", "DORMITORY", 5)],
    };
    let profiles = vec![profile(gd, BODY)];
    let sim = simulate_rotation(
        &rotation,
        &profiles,
        &building,
        &gd.building,
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

/// Account facts re-price player-state-gated skills the sync cannot read:
/// Lin's Meritocracy (+10% HR speed per purchased recruit slot) prices 0
/// undeclared (never-guess) and 10 x slots once declared. Flat-valued HIRE
/// skills stay untouched.
#[test]
fn account_facts_reprice_per_slot_hr_riders() {
    use backend::core::grade::base::buff_registry::{
        BuffResolutionStrategy, resolve_account_facts,
    };
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _) = build_registry(&gd.building.buffs, &name_to_char);
    let value =
        |reg: &std::collections::HashMap<String, BuffResolutionStrategy>, id: &str| match reg
            .get(id)
        {
            Some(BuffResolutionStrategy::NonProduction { value }) => *value,
            other => panic!("{id}: expected NonProduction, got {other:?}"),
        };
    assert_eq!(value(&registry, "hire_spd_cost&extra[000]"), 0.0);
    let resolved = resolve_account_facts(&registry, &gd.building.buffs, 3);
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
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _) = build_registry(&gd.building.buffs, &name_to_char);
    for id in [
        "hire_spd_cost&extra[000]",
        "hire_spd_cost&clue[000]",
        "hire_spd&clue[100]",
        "hire_spd_bd_n1_n1[100]",
    ] {
        println!("{id} => {:?}", registry.get(id));
    }
}

/// Dorm ambience: a maxed-furniture dorm (comfort 5000) recovers +2.0/hr on
/// top of its level rate, per the game's ComfortManpowerRecoverFactor.
#[test]
fn dorm_comfort_adds_recovery() {
    use backend::core::grade::base::dorms::dorm_list;
    let gd = load_game_data();
    let mut building = UserBuilding {
        rooms: vec![room("d0", "DORMITORY", 5), room("d1", "DORMITORY", 5)],
    };
    building.rooms[0].comfort = 5000;
    let dorms = dorm_list(&building, &gd.building);
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

#[test]
#[ignore]
fn unpriced_buff_audit() {
    use backend::core::grade::base::buff_registry::BuffResolutionStrategy;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _) = build_registry(&gd.building.buffs, &name_to_char);
    let mut unpriced = 0;
    for (id, buff) in &gd.building.buffs {
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
    println!("TOTAL unpriced: {unpriced} of {}", gd.building.buffs.len());
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
    for buff in gd.building.buffs.values() {
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

/// Non-stacking family attribution: with two "+7% all Trading Posts" globals
/// in the Control Center, exactly ONE line claims the +7 and the duplicate
/// reads "covered" - and the CC row's lines sum to the crew's global total.
/// (Pure ablation marginals are tie-blind: removing either copy changes
/// nothing, so nobody would claim the value.)
#[test]
fn ledger_attributes_nonstacking_families_to_one_winner() {
    use backend::core::grade::base::assignment::compute_current_assignment;
    use backend::core::grade::base::skill_ledger::LineDisposition;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    let mut building = UserBuilding {
        rooms: vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)],
    };
    building.rooms[0].current_operators = vec!["char_4132_ascln".into(), "char_308_swire".into()];
    building.rooms[1].current_operators = vec!["char_103_angel".into()];
    let roster: Vec<OperatorBaseProfile> = ["char_4132_ascln", "char_308_swire", "char_103_angel"]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn =
        compute_current_assignment(&roster, &building, &gd.building, &registry, &drains, None);
    let cc = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "CONTROL")
        .expect("cc scored");
    let trading_lines: Vec<_> = cc
        .ledger
        .iter()
        .filter(|l| l.speed_pct > 0.0 || l.disposition == LineDisposition::Covered)
        .collect();
    let winners = trading_lines
        .iter()
        .filter(|l| l.disposition == LineDisposition::Contributes)
        .count();
    let covered = trading_lines
        .iter()
        .filter(|l| l.disposition == LineDisposition::Covered)
        .count();
    assert_eq!(winners, 1, "exactly one +7 claims the family");
    assert_eq!(covered, 1, "the duplicate reads covered");
    let line_sum: f64 = cc.ledger.iter().map(|l| l.speed_pct).sum();
    assert!(
        (line_sum - cc.total_efficiency).abs() < 1e-6,
        "CC lines ({line_sum}) sum to the row total ({})",
        cc.total_efficiency
    );
    // And the trading post sees exactly one +7 [CC] line, one covered.
    let tp = asn.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    let cc_lines: Vec<_> = tp.ledger.iter().filter(|l| l.from_control_center).collect();
    assert_eq!(
        cc_lines
            .iter()
            .filter(|l| l.disposition == LineDisposition::Contributes)
            .count(),
        1
    );
    assert_eq!(
        cc_lines
            .iter()
            .filter(|l| l.disposition == LineDisposition::Covered)
            .count(),
        1
    );
}

/// Durin-class compound texts ("self Morale recovered per hour -0.1, but
/// restores +0.2 Morale per hour to all Operators assigned to that Dormitory")
/// are whole-dorm AURAS, not self-only skills. Community-confirmed 2026-08-24:
/// the aura applies to the holder fully too (net +0.1 on her own bar), and it
/// competes under the game's strongest-effect rule like every other dorm aura
/// (Lumen-class auras do NOT stack either - the model's max-fold is correct).
#[test]
fn durin_compound_text_is_a_whole_dorm_aura() {
    use backend::core::grade::base::dorms::dorm_aura_value;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, _) = build_registry(&gd.building.buffs, &name_to_char);
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
    // Dorm staffing can now see her: she prices at her max-tier aura value.
    let durin = profile(gd, "char_501_durin");
    let v = dorm_aura_value(&durin, &registry, &gd.building);
    assert!((v - 0.25).abs() < 1e-9, "Durin's staffing value, got {v}");
}

/// Umiri's Famiglia Approval buffs OPERATORS, not the post: it stacks with
/// Amiya's post-wide +7% (different effect types), its credit lands on the
/// post holding the Siracusan, and the CC row labels it "per-room" when it
/// fires - "inactive" only when no team satisfies it.
#[test]
fn umiri_stacks_with_amiya_and_labels_per_room() {
    use backend::core::grade::base::assignment::compute_current_assignment;
    use backend::core::grade::base::skill_ledger::LineDisposition;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    const UMIRI: &str = "char_4186_tmoris";
    const UMIRI_BUFF: &str = "control_tra_limit&spd2[000]";
    let build = |tp_crew: Vec<String>| {
        let mut building = UserBuilding {
            rooms: vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)],
        };
        building.rooms[0].current_operators = vec!["char_002_amiya".into(), UMIRI.into()];
        building.rooms[1].current_operators = tp_crew.clone();
        let mut ids: Vec<&str> = vec!["char_002_amiya", UMIRI];
        ids.extend(tp_crew.iter().map(String::as_str));
        let roster: Vec<OperatorBaseProfile> = ids.iter().map(|id| profile(gd, id)).collect();
        compute_current_assignment(&roster, &building, &gd.building, &registry, &drains, None)
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
    // Amiya's +7 is credited too - they STACK (different effect types).
    let amiya_tp = tp
        .ledger
        .iter()
        .find(|l| l.from_control_center && l.buff_id != UMIRI_BUFF && l.speed_pct > 0.0)
        .expect("amiya line on the post");
    assert!((amiya_tp.speed_pct - 7.0).abs() < 1e-6);

    // No Siracusan anywhere: honestly inactive.
    let asn = build(vec!["char_103_angel".into()]);
    let cc = asn.rooms.iter().find(|r| r.room_type == "CONTROL").unwrap();
    let umiri_cc = cc.ledger.iter().find(|l| l.buff_id == UMIRI_BUFF).unwrap();
    assert_eq!(umiri_cc.disposition, LineDisposition::Inactive);

    // Texas counts too - her SubPower is siracusa even though her nation is
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

    // Shamare cancels contributions sourced from her TEAMMATES - including
    // Umiri's per-operator grant to Lappland - while CC-sourced post-wide
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
