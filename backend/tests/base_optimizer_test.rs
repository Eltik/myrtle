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
use backend::core::grade::base::perception;
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
        profile(&gd, TEXAS),
        profile(&gd, EXUSIAI),
        profile(&gd, LAPPLAND),
        profile(&gd, "char_502_nblade"), // flat +30%
        profile(&gd, "char_185_frncat"), // flat +30%
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
        profile(&gd, "char_003_kalts"),
        profile(&gd, EXUSIAI),
        profile(&gd, "char_502_nblade"),
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
        profile(&gd, TEXAS),
        profile(&gd, "char_502_nblade"),
        profile(&gd, "char_185_frncat"),
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
        profile(&gd, SHAMARE),
        profile(&gd, EXUSIAI),           // +35%
        profile(&gd, "char_4045_heidi"), // +35%
    ];
    // … vs operators with no trading skill at all (pure bodies).
    let with_bodies = vec![
        profile(&gd, SHAMARE),
        profile(&gd, "char_003_kalts"),  // no trading buff
        profile(&gd, "char_180_amgoat"), // no trading buff
    ];

    let strong = trading_efficiency(&gd, &with_strong);
    let bodies = trading_efficiency(&gd, &with_bodies);

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
        profile(&gd, DOROTHY),
        profile(&gd, "char_128_plosis"), // Ptilopsis, Rhine, +25%
        profile(&gd, "char_108_silent"), // Silence, Rhine, +25%
    ];
    let mixed_team = vec![
        profile(&gd, DOROTHY),
        profile(&gd, "char_237_gravel"), // non-Rhine, +35%
        profile(&gd, "char_159_peacok"), // non-Rhine, +35%
    ];

    let rhine = factory_efficiency(&gd, &rhine_team);
    let mixed = factory_efficiency(&gd, &mixed_team);

    // Even though the non-Rhine teammates have *higher* flat efficiency (35 vs
    // 25), Dorothy's faction bonus (+5% × 2 Rhine = +10%) makes the Rhine team
    // competitive/better. The key assertion: the faction bonus is actually applied.
    let mixed_no_dorothy = factory_efficiency(
        &gd,
        &[
            profile(&gd, "char_237_gravel"),
            profile(&gd, "char_159_peacok"),
        ],
    );
    let rhine_no_dorothy = factory_efficiency(
        &gd,
        &[
            profile(&gd, "char_128_plosis"),
            profile(&gd, "char_108_silent"),
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
            &gd,
            &ids.iter().map(|id| profile(&gd, id)).collect::<Vec<_>>(),
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
    let profiles = full_roster(&gd);

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
    let profiles = full_roster(&gd);

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
        profile(&gd, "char_4141_marcil"), // Marcille, generic +30%
        profile(&gd, "char_242_otter"),   // Mayer, generic +30%
        profile(&gd, "char_400_weedy"),   // Weedy, nullifying automation
        profile(&gd, "char_102_texas"),   // Texas, trading-only (no factory value)
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
        profile(&gd, "char_4141_marcil"), // Marcille, generic +30% (ties Haze)
        profile(&gd, "char_237_gravel"),  // Gravel, Metalwork +35% (Gold only)
        profile(&gd, "char_141_nights"),  // Haze, Metalwork +30% (Gold only)
        profile(&gd, "char_4106_bryota"), // Bryophyta, Metalwork +30% (Gold only)
        profile(&gd, "char_102_texas"),   // a trader to staff the trading post
        profile(&gd, "char_103_angel"),   // Exusiai, trader
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
    let proviso = trading_lmd(&gd, &[profile(&gd, "char_4032_provs")]);
    let flat = trading_lmd(&gd, &[profile(&gd, "char_502_nblade")]); // +30% flat speed
    assert!(
        proviso >= flat,
        "Proviso's order value should rival a strong flat trader by LMD (proviso {proviso:.0} vs flat {flat:.0})"
    );
    // And her order VALUE must NOT inflate the displayed efficiency %.
    let proviso_eff = trading_efficiency(&gd, &[profile(&gd, "char_4032_provs")]);
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
            profile(&gd, "char_254_vodfox"),
            profile(&gd, "char_4032_provs"),
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
        profile(&gd, TEXAS),
        profile(&gd, LAPPLAND),
        profile(&gd, "char_272_strong"), // Jaye
    ];
    let eff = trading_efficiency(&gd, &team);
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
        profile(&gd, "char_272_strong"), // Jaye
        profile(&gd, EXUSIAI),
        profile(&gd, "char_4193_lemuen"),
    ];
    let eff = trading_efficiency(&gd, &team);
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
        profile(&gd, "char_4193_lemuen"),
        profile(&gd, EXUSIAI),
        profile(&gd, "char_4063_quartz"),
    ];
    let eff = trading_efficiency(&gd, &team);
    assert!(
        eff > 105.0,
        "Lemuen+Exusiai+Quartz should read ~+110% (Lemuen's +25% Exusiai bonus counted), got +{eff:.1}%"
    );

    // The +25% is genuinely conditional: drop Exusiai and Lemuen keeps only her
    // base +20%, so the same two-trader pairing must read materially lower.
    let no_exu = trading_efficiency(
        &gd,
        &[
            profile(&gd, "char_4193_lemuen"),
            profile(&gd, "char_4063_quartz"),
        ],
    );
    assert!(
        eff > no_exu + 20.0,
        "Exusiai should unlock Lemuen's +25% (with {eff:.1}% vs without {no_exu:.1}%)"
    );
}

#[test]
fn quartz_recipe_scaling_is_modeled_as_base_plus_factory_count() {
    // Quartz "Precise Scheduling" (trade_ord_spd&formula[000]): base +30% trading,
    // plus +2% per recipe type being processed at Factories. We approximate recipe
    // types by the Factory count, so her buff must resolve to a FacilityCountScaling
    // targeting MANUFACTURE with a +30% base - not a flat DirectEfficiency that
    // drops the recipe term. In a 5-factory base she then reads +40% (30 + 5×2).
    use backend::core::grade::base::buff_registry::BuffResolutionStrategy;

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
                target_room, "MANUFACTURE",
                "recipe types scale off Factories"
            );
            assert!(!per_level, "scaling is per-facility, not per-level");
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
    use backend::core::grade::base::buff_registry::BuffResolutionStrategy;

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
    use backend::core::grade::base::buff_registry::BuffResolutionStrategy;

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
        &gd,
        &[
            profile(&gd, MORGAN),
            profile(&gd, SIEGE),
            profile(&gd, INDRA),
        ],
    );
    let without_siege = trading_efficiency(
        &gd,
        &[
            profile(&gd, MORGAN),
            profile(&gd, MATTERHORN),
            profile(&gd, INDRA),
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
    use backend::core::grade::base::buff_registry::BuffResolutionStrategy;

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
        let roster: Vec<_> = cc.iter().map(|s| profile(&gd, s)).collect();
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
        let roster: Vec<_> = cc.iter().map(|s| profile(&gd, s)).collect();
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
        profile(&gd, SHAMARE),
        profile(&gd, TEQUILA),
        profile(&gd, BIBEAK),
    ];
    roster.extend(distractors.iter().map(|d| profile(&gd, d)));

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
        profile(&gd, "char_254_vodfox"), // Shamare (nullifier)
        profile(&gd, "char_486_takila"), // Tequila (order value)
        profile(&gd, "char_252_bibeak"), // Bibeak (order value)
    ];
    assert!(
        locked_for(&shamare_team),
        "a nullifier synergy squad must be marked locked"
    );

    let flexible_team = vec![
        profile(&gd, "char_502_nblade"), // flat +30%
        profile(&gd, "char_123_fang"),   // flat
        profile(&gd, "char_211_adnach"), // flat
    ];
    assert!(
        !locked_for(&flexible_team),
        "a team of independent flat traders must NOT be locked"
    );

    // Penguin Logistics (Texas needs Lappland) is a fixed synergy -> locked.
    let penguin = vec![
        profile(&gd, TEXAS),
        profile(&gd, LAPPLAND),
        profile(&gd, EXUSIAI),
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
        let roster: Vec<_> = cc.iter().chain(factory).map(|s| profile(&gd, s)).collect();
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
    .map(|s| profile(&gd, s))
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

    let weedy = vec![profile(&gd, "char_400_weedy")];
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
    let profiles = full_roster(&gd);

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
    let profiles = full_roster(&gd);
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
    let profiles = full_roster(&gd);
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
        let roster = vec![profile(&gd, op)];
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
    let profiles = full_roster(&gd);
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
    let profiles = full_roster(&gd);
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
    let profiles = full_roster(&gd);

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
            let slots = max_stationed(&gd, &room.room_type, room.level);
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
    let partial_score = grade_base(&partial, Some(&building_json), &gd);
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
    let full_score = grade_base(&full, Some(&building_json), &gd);
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
        profile(&gd, "char_1045_svash2"), // SilverAsh the Reignfrost (conditional CC)
        profile(&gd, "char_002_amiya"),   // Amiya (unconditional CC global)
        profile(&gd, "char_103_angel"),   // Exusiai, trader
        profile(&gd, "char_102_texas"),   // Texas, trader
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
        profile(&gd, "char_4116_blkkgt"), // Degenbrecher (order-limit wrecker)
        profile(&gd, "char_103_angel"),   // Exusiai, trader
        profile(&gd, "char_102_texas"),   // Texas, trader
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
        profile(&gd, "char_4141_marcil"), // Marcille, generic +30%
        profile(&gd, "char_242_otter"),   // Mayer, generic +30%
        profile(&gd, "char_103_angel"),   // Exusiai, trader
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
        profile(&gd, "char_103_angel"),  // Exusiai +35
        profile(&gd, "char_4032_provs"), // Proviso (order value)
        profile(&gd, "char_502_nblade"), // Yato +30
        profile(&gd, "char_185_frncat"), // Mousse +30
        profile(&gd, "char_102_texas"),  // Texas (conditional on Lappland)
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
    let roster = full_roster(&gd);
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
        profile(&gd, "char_1045_svash2"), // SilverAsh the Reignfrost (CC, Kjerag gate)
        profile(&gd, "char_002_amiya"),   // Amiya (unconditional CC global)
        profile(&gd, "char_198_blackd"),  // Courier (Kjerag, weak trader)
        profile(&gd, "char_199_yak"),     // Matterhorn (Kjerag, weak trader)
        profile(&gd, "char_173_slchan"),  // Cliffheart (Kjerag, weak trader)
        profile(&gd, "char_103_angel"),   // Exusiai (strong trader)
        profile(&gd, "char_502_nblade"),  // Yato (strong trader)
        profile(&gd, "char_185_frncat"),  // Mousse (strong trader)
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
            profile(&gd, "char_103_angel"),
            profile(&gd, "char_502_nblade"),
            profile(&gd, "char_4032_provs"),
            profile(&gd, "char_237_gravel"),
            profile(&gd, "char_141_nights"),
        ];
        if morale {
            r.push(profile(&gd, "char_1035_wisdel"));
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
    let roster = full_roster(&gd);
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
        &full_roster(&gd),
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
        let roster: Vec<_> = ids.iter().map(|id| profile(&gd, id)).collect();
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
    .map(|c| profile(&gd, c))
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
        &full_roster(&gd),
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
            profile(&gd, "char_4032_provs"),
            profile(&gd, "char_486_takila"),
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
        profile(&gd, "char_4032_provs"), // Proviso (value)
        profile(&gd, "char_486_takila"), // Tequila (value)
        profile(&gd, "char_103_angel"),  // Exusiai +35 speed
        profile(&gd, "char_502_nblade"), // Yato +30 speed
        profile(&gd, "char_185_frncat"), // Mousse +30 speed
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
        profile(&gd, "char_140_whitew"), // Lappland (needs Texas, absent)
        profile(&gd, "char_4032_provs"), // Proviso
        profile(&gd, "char_254_vodfox"), // Shamare
        profile(&gd, "char_486_takila"), // Tequila
        profile(&gd, "char_003_kalts"),  // body
        profile(&gd, "char_180_amgoat"), // body
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
        &full_roster(&gd),
        &building,
        &gd.building,
        &registry,
        &drains,
    );

    assert_eq!(rot.shifts.len(), 3, "three shifts");

    // The Control Center rests its main crew on the MIDDLE shift (work / off / work) so its
    // operators never sit two shifts back-to-back; a backup off-team covers that middle shift,
    // so the room stays staffed all three shifts. Shifts 1 and 3 are the main; shift 2 differs.
    let cc = |shift: usize| -> &backend::core::grade::base::shift_rotation::ShiftRoom {
        rot.shifts[shift]
            .rooms
            .iter()
            .find(|r| r.room_type == "CONTROL")
            .unwrap()
    };
    let cc_active: Vec<bool> = (0..3).map(|k| cc(k).active).collect();
    assert_eq!(
        cc_active,
        vec![true, true, true],
        "CC stays staffed via its off-team"
    );
    assert_eq!(
        cc(0).recommended,
        cc(2).recommended,
        "shifts 1 and 3 run the same main CC crew"
    );
    assert_ne!(
        cc(0).recommended,
        cc(1).recommended,
        "the middle-shift off-team should differ from the main CC crew, got {:?}",
        cc(1).recommended
    );

    // The HR Office and Reception Room ALSO rotate a main + off-team (rest the middle shift),
    // not a single crew kept stationed - so their operators get to rest too.
    for rt in ["HIRE", "MEETING"] {
        let crew = |shift: usize| -> Vec<String> {
            rot.shifts[shift]
                .rooms
                .iter()
                .find(|r| r.room_type == rt)
                .unwrap()
                .recommended
                .clone()
        };
        assert!(!crew(0).is_empty(), "{rt} should be staffed");
        assert_eq!(
            crew(0),
            crew(2),
            "{rt} runs the same main crew shifts 1 and 3"
        );
        assert_ne!(
            crew(0),
            crew(1),
            "{rt} should rotate a different off-team on the middle shift"
        );
    }

    // Every production room runs its MAIN team on shifts 1 & 3 and rests the MIDDLE shift (an
    // off-team covers it) - a clean work / rest / work cycle. The main never works two shifts
    // back-to-back, no operator is in both the main and its off-team, and nobody works all three.
    let tp_team = |shift: usize, slot: &str| -> Vec<String> {
        let mut v = rot.shifts[shift]
            .rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .unwrap()
            .recommended
            .clone();
        v.sort();
        v
    };
    for slot in ["tp0", "tp1"] {
        assert!(!tp_team(0, slot).is_empty(), "{slot} should be staffed");
        assert_eq!(
            tp_team(0, slot),
            tp_team(2, slot),
            "{slot} runs its main team on shifts 1 and 3"
        );
        assert_ne!(
            tp_team(0, slot),
            tp_team(1, slot),
            "{slot} rests its main on the middle shift"
        );
        assert!(
            !tp_team(0, slot)
                .iter()
                .any(|o| tp_team(1, slot).contains(o)),
            "no operator is in both {slot}'s main and its middle-shift off-team"
        );
    }

    // A power plant ALSO works shifts 1 & 3 and rests the middle (its off-team covers, or it rests
    // dark) - so its operators recover too, instead of working two shifts in a row.
    let p0 = |s: usize| -> Vec<String> {
        let mut v = rot.shifts[s]
            .rooms
            .iter()
            .find(|r| r.slot_id == "p0")
            .unwrap()
            .recommended
            .clone();
        v.sort();
        v
    };
    assert!(!p0(0).is_empty(), "the power plant should be staffed");
    assert_eq!(
        p0(0),
        p0(2),
        "the power plant runs its main crew on shifts 1 and 3"
    );
    assert_ne!(
        p0(0),
        p0(1),
        "a distinct off-team (or rest) covers the middle shift"
    );
}

#[test]
fn perception_pool_credits_rosmontis_on_a_243() {
    // The Rosmontis / Ebenholz "Perception Information" economy: operators resting in
    // dormitories feed a base-wide pool that Rosmontis converts to Chain of Thought and
    // reads for productivity. On a 243 with full dorms (~20 resting), the two core
    // per-resting generators (Rosmontis + Ebenholz) alone give her a sizable bonus.
    let gd = load_game_data();
    let mut rooms = vec![room("cc", "CONTROL", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("pp{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding { rooms };

    let resting = perception::resting_capacity(&building, &gd.building);
    assert_eq!(resting, 20, "4 dorms x 5 slots");

    let roster = vec![
        profile(&gd, "char_391_rosmon"), // Rosmontis (generator + Chain-of-Thought consumer)
        profile(&gd, "char_4046_ebnhlz"), // Ebenholz (generator + Soundless-Resonance consumer)
        profile(&gd, "char_245_cello"),  // Virtuosa (dorm Soundless-Resonance generator)
        profile(&gd, "char_455_nothin"), // Mr. Nothing (Worldly Plight / "Yan" economy)
    ];
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let result = perception::evaluate(&roster, &building, &gd.building, &drains, &registry);
    for c in &result.consumers {
        println!(
            "  {} reads {} -> +{:.0}% (sustained +{:.0}%) [{}]",
            c.char_id, c.resource, c.bonus_pct, c.sustained_pct, c.room_type
        );
    }

    // Rosmontis's Chain of Thought equals the dormitory occupancy (~20) - the guide's
    // "20 PI max" - NOT a shared pool summed with Ebenholz's. So ~+20%, not +40%.
    let rosmontis = result
        .consumers
        .iter()
        .find(|c| c.char_id == "char_391_rosmon");
    assert!(
        rosmontis.is_some_and(|c| c.room_type == "MANUFACTURE" && (c.bonus_pct - 20.0).abs() < 2.0),
        "Rosmontis should read ~+20% Chain of Thought (capped at dorm occupancy), got {:?}",
        rosmontis.map(|c| (c.resource.clone(), c.bonus_pct))
    );
    // Rosmontis is the SOLE Chain-of-Thought source here (she generates it from dorm occupancy
    // and reads it herself), so her pool is co-present with her consumption: the sustained 24/7
    // bonus equals the peak - her own rest reduces her OUTPUT, not the bonus multiplier.
    assert!(
        rosmontis.is_some_and(|c| (c.sustained_pct - c.bonus_pct).abs() < 0.01),
        "a self-feeding consumer's sustained bonus should equal its peak, got {:?}",
        rosmontis.map(|c| (c.bonus_pct, c.sustained_pct))
    );
    // Ebenholz reads her OWN Soundless Resonance (~dorm occupancy / 2 at E2), independent of
    // Rosmontis - around +10-12%, not the +22% a shared pool would give.
    let ebenholz = result
        .consumers
        .iter()
        .find(|c| c.char_id == "char_4046_ebnhlz");
    assert!(
        ebenholz.is_some_and(|c| c.bonus_pct < 16.0),
        "Ebenholz should read her own ~+10-12% SR, not a shared-pool +22%, got {:?}",
        ebenholz.map(|c| c.bonus_pct)
    );
}

#[test]
fn perception_consumer_is_placed_and_credited_on_a_243() {
    // End-to-end: with the Perception override folded into the registry (as the 243
    // improvements path does), the optimizer must place Rosmontis in a Factory and credit
    // her base-wide Chain-of-Thought bonus - beating the generic gold operators she's up
    // against. (Non-243 layouts and `current` keep the unaugmented registry.)
    let gd = load_game_data();
    let mut rooms = vec![room("cc", "CONTROL", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("pp{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding { rooms };

    let roster = vec![
        profile(&gd, "char_391_rosmon"),  // Rosmontis (perception consumer)
        profile(&gd, "char_4046_ebnhlz"), // Ebenholz (feeds the pool)
        profile(&gd, "char_237_gravel"),  // Metalwork +35% (gold)
        profile(&gd, "char_4141_marcil"), // generic +30%
        profile(&gd, "char_159_peacok"),  // generic
        profile(&gd, "char_102_texas"),   // trader
        profile(&gd, "char_103_angel"),   // trader
    ];

    let name_to_char = build_name_to_char(&gd.operators);
    let (mut registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    for (buff_id, pct) in
        perception::evaluate(&roster, &building, &gd.building, &drains, &registry).overrides
    {
        registry.insert(
            buff_id,
            BuffResolutionStrategy::DirectEfficiency { value: pct },
        );
    }

    let asn = compute_optimal_assignment(&roster, &building, &gd.building, &registry, &drains);
    let room = asn
        .rooms
        .iter()
        .find(|r| r.operators.iter().any(|o| o == "char_391_rosmon"));
    assert!(
        room.is_some_and(|r| r.room_type == "MANUFACTURE"),
        "Rosmontis should be placed in a Factory by her Perception bonus, got {:?}",
        room.map(|r| (&r.room_type, r.total_efficiency))
    );
    assert!(
        room.is_some_and(|r| r.total_efficiency >= 40.0),
        "her Factory should carry the ~+40% Chain-of-Thought bonus, got {:?}",
        room.map(|r| r.total_efficiency)
    );
}

#[test]
fn perception_cross_room_support_placement_on_a_243() {
    // Phase 1b: a support generator earns a non-production seat *because* it feeds deployed
    // consumers elsewhere. Mulberry (HR Office) makes Worldly Plight; Shu & Jieyun (Factory)
    // consume it (Jieyun via Witchcraft Crystal). With no Worldly-Plight generator stationed
    // the pool is empty - so the plan must station Mulberry in HR and credit the factories.
    let gd = load_game_data();
    let mut rooms = vec![room("cc", "CONTROL", 5), room("hr", "HIRE", 1)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding { rooms };

    let roster = vec![
        profile(&gd, "char_473_mberry"), // Mulberry  - HR Worldly Plight generator
        profile(&gd, "char_436_whispr"), // Whisperain - HR (Memory Fragment, non-bd -> 0)
        profile(&gd, "char_2025_shu"),   // Shu     - Factory Worldly Plight consumer
        profile(&gd, "char_4078_bdhkgt"), // Jieyun  - Factory WP -> Witchcraft Crystal consumer
    ];

    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let result = perception::evaluate(&roster, &building, &gd.building, &drains, &registry);
    println!("support plan: {:?}", result.support);
    assert!(
        result
            .support
            .iter()
            .any(|(c, r)| c == "char_473_mberry" && r == "HIRE"),
        "Mulberry should be stationed in HR to feed Worldly Plight, got {:?}",
        result.support
    );

    for c in &result.consumers {
        println!(
            "  {} reads {} -> +{:.0}% [{}]",
            c.char_id, c.resource, c.bonus_pct, c.room_type
        );
    }
    let shu = result
        .consumers
        .iter()
        .find(|c| c.char_id == "char_2025_shu");
    let jieyun = result
        .consumers
        .iter()
        .find(|c| c.char_id == "char_4078_bdhkgt");
    assert!(
        shu.is_some_and(|c| c.room_type == "MANUFACTURE" && c.bonus_pct > 5.0),
        "Shu should read a Worldly-Plight bonus enabled by Mulberry, got {shu:?}"
    );
    assert!(
        jieyun.is_some_and(|c| c.room_type == "MANUFACTURE" && c.bonus_pct > 5.0),
        "Jieyun should read a Witchcraft-Crystal bonus, got {jieyun:?}"
    );
}

#[test]
fn perception_support_is_pinned_and_reserved_in_the_optimal_plan() {
    // The economy is wired into the REAL optimizer, not a side panel: its support generators and
    // morale-swap manager are pinned into the rooms that feed the pool and reserved from
    // everything else, so production is optimized around them - never double-booking an operator.
    let gd = load_game_data();
    let mut rooms = vec![room("cc", "CONTROL", 5), room("hr", "HIRE", 3)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("pp{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding { rooms };

    let roster = vec![
        profile(&gd, "char_473_mberry"), // Mulberry - Office Worldly-Plight generator
        profile(&gd, "char_2023_ling"),  // Ling - Control-Center conditional generator
        profile(&gd, "char_2015_dusk"),  // Dusk - Control-Center conditional generator
        profile(&gd, "char_300_phenxi"), // Fiammetta - morale-swap rotation manager
        profile(&gd, "char_455_nothin"), // Mr. Nothing - Trading consumer
        profile(&gd, "char_391_rosmon"), // Rosmontis - Factory consumer
        profile(&gd, "char_2025_shu"),   // Shu - Factory consumer
        // A few strong generic producers so the optimizer has plenty to staff production with.
        profile(&gd, "char_002_amiya"),
        profile(&gd, "char_003_kalts"),
        profile(&gd, "char_134_ifrit"),
    ];

    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let perception = perception::evaluate(&roster, &building, &gd.building, &drains, &registry);

    // Build the same pins the service builds: the support plan + the morale-swap manager.
    let mut pins: Vec<(String, String)> = perception.support.clone();
    if let Some(mgr) = &perception.rotation_manager {
        pins.push((mgr.clone(), "DORMITORY".to_string()));
    }
    assert!(
        perception.rotation_manager.as_deref() == Some("char_300_phenxi"),
        "Fiammetta should be picked up as the rotation manager, got {:?}",
        perception.rotation_manager
    );

    let asn = compute_optimal_assignment_with_pins(
        &roster,
        &building,
        &gd.building,
        &registry,
        &drains,
        &pins,
    );
    let in_room = |id: &str, rt: &str| {
        asn.rooms
            .iter()
            .any(|r| r.room_type == rt && r.operators.iter().any(|o| o == id))
    };
    let in_production = |id: &str| {
        asn.rooms
            .iter()
            .filter(|r| matches!(r.room_type.as_str(), "MANUFACTURE" | "TRADING" | "POWER"))
            .any(|r| r.operators.iter().any(|o| o == id))
    };

    // Control-Center generators are forced into the CC even though they carry no CC production bonus.
    assert!(
        in_room("char_2023_ling", "CONTROL"),
        "Ling should be pinned into the Control Center"
    );
    assert!(
        in_room("char_2015_dusk", "CONTROL"),
        "Dusk should be pinned into the Control Center"
    );
    // The Office generator and the dorm-stationed morale-swap manager show up in their rooms.
    assert!(
        in_room("char_473_mberry", "HIRE"),
        "Mulberry should be pinned into the Office"
    );
    assert!(
        in_room("char_300_phenxi", "DORMITORY"),
        "Fiammetta should be pinned into a dormitory"
    );
    // None of the reserved economy operators may be double-booked into a production room.
    for id in [
        "char_473_mberry",
        "char_2023_ling",
        "char_2015_dusk",
        "char_300_phenxi",
    ] {
        assert!(
            !in_production(id),
            "{id} is reserved and must not appear in a production room"
        );
    }
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
        .map(|id| profile(&gd, id))
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
    let weedy_only = vec![profile(&gd, "char_400_weedy")];
    let weedy_greyy = vec![
        profile(&gd, "char_400_weedy"),
        profile(&gd, "char_1027_greyy2"),
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
    let roster = vec![profile(&gd, "char_190_clour")]; // Vermeil (E1+ in game data fixture)
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
    .map(|id| profile(&gd, id))
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

#[test]
fn perception_conditional_ling_dusk_feed_both_economies_sustainably() {
    // Phase 2 + time-simulation: Ling & Dusk (Control Center) flip which resource they make by
    // morale. The SUSTAINABLE state (a Fiammetta manager holds them at full/high morale) puts
    // each on its high-morale side - one resource EACH, not both on the single best one (that
    // transient peak "only lasts 1 shift" per the guide). So Ling -> Worldly Plight (boosts Mr.
    // Nothing) and Dusk -> Perception Information (which converts to Rosmontis's Chain of
    // Thought, boosting her). Both are planned into the Control Center.
    let gd = load_game_data();
    let mut rooms = vec![room("cc", "CONTROL", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding { rooms };

    let roster = vec![
        profile(&gd, "char_2023_ling"), // Ling - high morale -> Worldly Plight (Yan)
        profile(&gd, "char_2015_dusk"), // Dusk - high morale -> Perception Information (A/B)
        profile(&gd, "char_455_nothin"), // Mr. Nothing - Trading Worldly-Plight gen + consumer
        profile(&gd, "char_391_rosmon"), // Rosmontis - Factory Chain-of-Thought consumer
    ];

    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    let result = perception::evaluate(&roster, &building, &gd.building, &drains, &registry);
    println!("support: {:?}", result.support);
    for c in &result.consumers {
        println!(
            "  {} reads {} -> +{:.0}% [{}]",
            c.char_id, c.resource, c.bonus_pct, c.room_type
        );
    }
    assert!(
        result
            .support
            .iter()
            .any(|(c, r)| c == "char_2023_ling" && r == "CONTROL")
            && result
                .support
                .iter()
                .any(|(c, r)| c == "char_2015_dusk" && r == "CONTROL"),
        "Ling and Dusk should both be planned into the Control Center, got {:?}",
        result.support
    );
    // Ling adds +15 Worldly Plight on top of Mr. Nothing's resting ~20 -> ~+35% (NOT the +50%
    // the transient "both on Worldly Plight" peak would give).
    let mrn = result
        .consumers
        .iter()
        .find(|c| c.char_id == "char_455_nothin");
    assert!(
        mrn.is_some_and(|c| (c.bonus_pct - 35.0).abs() < 4.0),
        "Mr. Nothing should read ~+35% (resting + Ling's Worldly Plight), got {mrn:?}"
    );
    // Dusk's +10 Perception Information converts to Rosmontis's Chain of Thought on top of her
    // own ~20 -> ~+30%.
    let ros = result
        .consumers
        .iter()
        .find(|c| c.char_id == "char_391_rosmon");
    assert!(
        ros.is_some_and(|c| c.bonus_pct > 25.0),
        "Dusk's Perception Information should lift Rosmontis above her solo ~+20%, got {ros:?}"
    );
    // Generator-side uptime: Rosmontis's own ~20 is co-present (full), but Dusk's cross-source
    // +10 only counts while DUSK works, so the sustained value sits between her solo +20% and
    // the +30% peak (it does NOT collapse all the way to +20% - the self part is undiminished).
    assert!(
        ros.is_some_and(|c| c.sustained_pct > 20.0 && c.sustained_pct < c.bonus_pct),
        "Rosmontis's sustained bonus should keep her full self-pool and a scaled share of Dusk's, got {ros:?}"
    );
    // Likewise Mr. Nothing: own ~20 Worldly Plight (co-present) + Ling's +15 scaled by Ling's uptime.
    assert!(
        mrn.is_some_and(|c| c.sustained_pct > 20.0 && c.sustained_pct < c.bonus_pct),
        "Mr. Nothing's sustained bonus should keep his full self-pool and a scaled share of Ling's, got {mrn:?}"
    );
}

#[test]
fn perception_rotation_needs_a_morale_swap_manager() {
    // Phase 3: a Ling/Dusk morale-conditional rotation must be sustained by a Fiammetta-type
    // morale-swap manager. The plan flags this - named when the roster owns one, "needed"
    // when it doesn't. (The full time-averaged morale simulation is deliberately deferred -
    // the peak is the meaningful number; see docs/base-resource-economy.md.)
    let gd = load_game_data();
    let mut rooms = vec![room("cc", "CONTROL", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding { rooms };

    let base = vec![
        profile(&gd, "char_2023_ling"),  // Ling (conditional)
        profile(&gd, "char_2015_dusk"),  // Dusk (conditional)
        profile(&gd, "char_455_nothin"), // Mr. Nothing (Worldly-Plight consumer)
    ];

    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    // No morale-swap operator owned -> the rotation is flagged as needing one.
    let without = perception::evaluate(&base, &building, &gd.building, &drains, &registry);
    assert!(
        without.needs_rotation_manager && without.rotation_manager.is_none(),
        "a Ling/Dusk rotation with no Fiammetta should flag it needs a morale-swap manager"
    );

    // With Fiammetta owned -> she's named as the manager and the flag clears.
    let mut with = base;
    with.push(profile(&gd, "char_300_phenxi")); // Fiammetta (morale-swap manager)
    let result = perception::evaluate(&with, &building, &gd.building, &drains, &registry);
    assert_eq!(
        result.rotation_manager.as_deref(),
        Some("char_300_phenxi"),
        "Fiammetta should be picked as the rotation manager"
    );
    assert!(!result.needs_rotation_manager);
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
fn perception_of(
    gd: &GameData,
    roster: &[OperatorBaseProfile],
    building: &UserBuilding,
) -> perception::PerceptionResult {
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    perception::evaluate(roster, building, &gd.building, &drains, &registry)
}

/// Does this operator carry any Control-Center base skill (so it competes for CC seats)?
fn is_control_op(gd: &GameData, op: &OperatorBaseProfile) -> bool {
    op.available_buffs.iter().any(|b| {
        gd.building
            .buffs
            .get(b)
            .is_some_and(|buff| buff.room_type == "CONTROL")
    })
}

fn mf_override(p: &perception::PerceptionResult) -> Option<f64> {
    p.global_overrides
        .values()
        .find(|(room, _)| room == "MANUFACTURE")
        .map(|(_, pct)| *pct)
}
fn td_override(p: &perception::PerceptionResult) -> Option<f64> {
    p.global_overrides
        .values()
        .find(|(room, _)| room == "TRADING")
        .map(|(_, pct)| *pct)
}

#[test]
fn passion_combo_is_committed_realized_and_counts_the_dual_role_once() {
    let gd = load_game_data();
    let building = bd_base_243();
    // The five combo operators alone: with no OTHER Control-Center operators the ordinary fill is
    // worth ~nothing, so the whole-base combo wins and commits.
    let roster: Vec<_> = [SAKIKO, MORTIS, AMORIS, DOLRIS, TMORIS]
        .iter()
        .map(|id| profile(&gd, id))
        .collect();
    let p = perception_of(&gd, &roster, &building);

    let mf = mf_override(&p).expect("factories should get the committed Passion bonus");
    let td = td_override(&p).expect("trading should get the committed Passion bonus");
    assert!(
        td > mf,
        "trading's 1/8-per-point rate outpaces factories' 1/20: {td} vs {mf}"
    );

    // Realized pool = Monster of Acting +20 (counted ONCE) + Diligent Worker +10 + Reliable
    // Companion +10 + Idol's Aura (one per resting operator). Factories read pool/20, trading
    // pool/8; a double-counted +20 would inflate both.
    let resting = perception::resting_capacity(&building, &gd.building);
    let pool = 20.0 + 10.0 + 10.0 + f64::from(resting);
    assert!(
        (mf - pool / 20.0).abs() < 0.05,
        "factory override {mf} should equal pool/20 = {}",
        pool / 20.0
    );
    assert!(
        (td - pool / 8.0).abs() < 0.05,
        "trading override {td} should equal pool/8 = {}",
        pool / 8.0
    );

    // Pure generators are pinned into the Control Center; the consumers are NOT pinned (the
    // bonus-greedy seats them via their realized bonus), and the dual-role op never takes a pin.
    let pins: std::collections::HashSet<&str> = p
        .support
        .iter()
        .filter(|(_, rt)| rt == "CONTROL")
        .map(|(c, _)| c.as_str())
        .collect();
    for g in [AMORIS, DOLRIS, TMORIS] {
        assert!(
            pins.contains(g),
            "{g} (generator) should be pinned into the Control Center, pins = {pins:?}"
        );
    }
    assert!(
        !pins.contains(SAKIKO) && !pins.contains(MORTIS),
        "consumers are seated by the optimizer, not pinned"
    );
}

#[test]
fn passion_combo_staffs_the_control_center_and_lifts_factories() {
    let gd = load_game_data();
    let building = bd_base_243();
    let name_to_char = build_name_to_char(&gd.operators);
    let (registry, drains) = build_registry(&gd.building.buffs, &name_to_char);
    // Combo + production fillers, but no other Control-Center operators (clean commit).
    let combo: std::collections::HashSet<&str> = [SAKIKO, MORTIS, AMORIS, DOLRIS, TMORIS]
        .into_iter()
        .collect();
    let roster: Vec<_> = full_roster(&gd)
        .into_iter()
        .filter(|op| combo.contains(op.char_id.as_str()) || !is_control_op(&gd, op))
        .collect();
    let p = perception::evaluate(&roster, &building, &gd.building, &drains, &registry);

    // Fold the economy into the registry exactly like the improvements service.
    let mut reg = registry.clone();
    for (b, pct) in &p.overrides {
        reg.insert(
            b.clone(),
            BuffResolutionStrategy::DirectEfficiency { value: *pct },
        );
    }
    for (b, (room, pct)) in &p.global_overrides {
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
        &p.support,
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
fn partial_passion_roster_yields_a_smaller_pool() {
    let gd = load_game_data();
    let building = bd_base_243();
    let full: Vec<_> = [SAKIKO, MORTIS, AMORIS, DOLRIS, TMORIS]
        .iter()
        .map(|id| profile(&gd, id))
        .collect();
    // Drop the dorm generator and one flat generator: a smaller Passion pool.
    let partial: Vec<_> = [SAKIKO, MORTIS, AMORIS]
        .iter()
        .map(|id| profile(&gd, id))
        .collect();

    let full_mf = mf_override(&perception_of(&gd, &full, &building)).expect("full combo commits");
    let part_mf =
        mf_override(&perception_of(&gd, &partial, &building)).expect("partial combo still commits");
    assert!(
        part_mf > 0.0,
        "the partial combo still pays something: {part_mf}"
    );
    assert!(
        part_mf < full_mf,
        "fewer generators -> smaller realized pool: {part_mf} < {full_mf}"
    );
}

#[test]
fn weak_partial_combo_is_not_committed_when_ordinary_buffs_win() {
    let gd = load_game_data();
    let building = bd_base_243();
    // Sakiko + one small generator, but the full roster of ordinary Control-Center buffers competes
    // for the same seats. The combo's lone ~+0.5% to factories loses, so it must NOT commit.
    let drop: std::collections::HashSet<&str> = [MORTIS, DOLRIS, TMORIS].into_iter().collect();
    let roster: Vec<_> = full_roster(&gd)
        .into_iter()
        .filter(|op| !drop.contains(op.char_id.as_str()))
        .collect();
    let p = perception_of(&gd, &roster, &building);
    assert!(
        p.global_overrides.is_empty(),
        "a weak partial combo must lose to the ordinary CC fill and emit no global override, got {:?}",
        p.global_overrides
    );
}

#[test]
fn a_roster_without_passion_ops_has_no_combo_override() {
    let gd = load_game_data();
    let building = bd_base_243();
    let combo: std::collections::HashSet<&str> = [SAKIKO, MORTIS, AMORIS, DOLRIS, TMORIS]
        .into_iter()
        .collect();
    let roster: Vec<_> = full_roster(&gd)
        .into_iter()
        .filter(|op| !combo.contains(op.char_id.as_str()))
        .collect();
    let p = perception_of(&gd, &roster, &building);
    assert!(
        p.global_overrides.is_empty(),
        "no Passion operators -> no Control-Center combo override"
    );
}

// ─── Part A: shift rotation never double-books an operator within one shift ──────────────────

#[test]
fn shift_rotation_never_double_books_an_operator_within_a_shift() {
    use backend::core::grade::base::shift_rotation::recommend_shift_rotation;
    use std::collections::HashMap;
    let gd = load_game_data();
    let (registry, drains) = build_registry(&gd.building.buffs, &build_name_to_char(&gd.operators));
    // bd_base_243 has 3 power plants, exercising the widened power-plan exclusion.
    let building = bd_base_243();
    let roster = full_roster(&gd);
    let rotation = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains);
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
    let building = trading_post(3);
    let recovery = morale_recovery(&building);
    const PROVISO: &str = "char_4032_provs";
    const FIAMMETTA: &str = "char_300_phenxi"; // swaps morale with another operator

    // No morale-swap manager owned -> nobody is held at full morale.
    let base: Vec<_> = [PROVISO, EXUSIAI, TEXAS]
        .iter()
        .map(|id| profile(&gd, id))
        .collect();
    let main = compute_optimal_assignment(&base, &building, &gd.building, &registry, &drains);
    let none = morale_sustained_beneficiaries(&main, &base, &drains, recovery, &gd.building);
    assert!(
        none.is_empty(),
        "no manager -> no sustained operator, got {none:?}"
    );

    // With Fiammetta owned -> EXACTLY ONE operator is sustained, and crediting it doesn't lower
    // (it raises) the trading post's sustained efficiency.
    let mut with = base;
    with.push(profile(&gd, FIAMMETTA));
    let main2 = compute_optimal_assignment(&with, &building, &gd.building, &registry, &drains);
    let benef = morale_sustained_beneficiaries(&main2, &with, &drains, recovery, &gd.building);
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
        .map(|id| profile(&gd, id))
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
    let roster: Vec<_> = [VIGIL, RED].iter().map(|id| profile(&gd, id)).collect();
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
    let roster = vec![profile(&gd, KAZEMARU)];
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
        .map(|id| profile(&gd, id))
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
        &full_roster(&gd),
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
    .map(|id| profile(&gd, id))
    .collect();

    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains);

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
    .map(|id| profile(&gd, id))
    .collect();

    let rot = recommend_shift_rotation(&roster, &building, &gd.building, &registry, &drains);

    assert!(
        rot.sustained.is_empty(),
        "without a manager nobody is sustained, got {:?}",
        rot.sustained
    );
    // The middle (off) shift must NOT keep Proviso working - she rests like everyone else.
    let middle = &rot.shifts[rot.shifts.len() / 2];
    let tp = middle
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post");
    assert!(
        !(tp.active && tp.recommended.iter().any(|o| o == PROVISO)),
        "Proviso should rest the middle shift without a manager (got active={}, {:?})",
        tp.active,
        tp.recommended
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
    };

    let dto = shift_rotation_to_dto(&rotation, &gd, &profiles, &building, &registry, &drains);
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
