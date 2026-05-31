//! Human-readable showcase of the base (infrastructure) optimizer, driven by a
//! real user's data (UID 09525371) captured into a JSON fixture.
//!
//! Refresh the fixture from the database with:
//!     cargo run --bin dump-user-fixture -- 09525371
//!
//! Watch the optimizer's output with:
//!     cargo test --test base_optimizer_display_test -- --nocapture
//!
//! It prints the base layout, the single-shift OPTIMAL assignment (per-room
//! operators + efficiency), the two-shift sustained rotation, and the overall
//! base score / letter grade.

mod common;

use backend::core::gamedata::types::GameData;
use backend::core::grade::base::assignment::{
    compute_optimal_assignment, compute_sustained_assignment,
};
use backend::core::grade::base::buff_registry::{build_name_to_char, build_registry};
use backend::core::grade::base::score::grade_base;
use backend::core::grade::base::types::{BaseAssignment, ShiftAssignment, UserBuilding};
use common::{
    build_profiles, load_game_data, load_user_fixture, max_stationed, operator_name,
};

const UID: &str = "09525371";

fn print_assignment(assignment: &BaseAssignment, game_data: &GameData) {
    use backend::core::grade::base::yield_model::{BaseFlows, room_yield};

    let mut flows = BaseFlows::default();
    for room in &assignment.rooms {
        flows.add_room(
            &room.room_type,
            room.formula_type.as_deref(),
            room.level,
            room.total_efficiency,
            room.order_value,
        );

        let formula_label = room.formula_type.as_deref().unwrap_or("");
        let y = room_yield(
            &room.room_type,
            room.formula_type.as_deref(),
            room.level,
            room.total_efficiency,
            room.order_value,
        );
        let yield_label = if y.gold_per_day > 0.0 {
            format!("  →  {:.0} gold bars/day", y.gold_per_day)
        } else if y.exp_per_day > 0.0 {
            format!("  →  {:.0} EXP/day", y.exp_per_day)
        } else if y.lmd_per_day > 0.0 {
            format!("  →  up to {:.0} LMD/day (if gold-supplied)", y.lmd_per_day)
        } else {
            String::new()
        };
        println!(
            "\n  {} (L{}) {formula_label} — +{:.1}% efficiency{yield_label}",
            room.room_type, room.level, room.total_efficiency
        );
        for id in &room.operators {
            println!("    - {} ({id})", operator_name(id, game_data));
        }
        if room.operators.is_empty() {
            println!("    (no beneficial operators)");
        }
    }
    println!(
        "\n  TOTAL PRODUCTION EFFICIENCY: +{:.1}%",
        assignment.total_production_efficiency
    );
    println!(
        "  REALIZED YIELD: {:.0} LMD/day + {:.0} EXP/day  (gold made {:.0}/day, gold sellable {:.0}/day)\n  TOTAL VALUE: {:.0} LMD-equivalent/day",
        flows.realized_lmd(),
        flows.exp,
        flows.gold_produced,
        flows.gold_sell_capacity,
        flows.total_value(),
    );
}

fn print_rotation(sustained: &ShiftAssignment, game_data: &GameData) {
    println!("\n--- SHIFT A ---");
    print_assignment(&sustained.shift_a, game_data);
    println!("\n--- SHIFT B ---");
    print_assignment(&sustained.shift_b, game_data);
    println!(
        "\n  SUSTAINED (avg of both shifts): +{:.1}%",
        sustained.sustained_efficiency
    );
}

fn letter_grade(score: f64) -> &'static str {
    match score {
        s if s >= 0.90 => "S+",
        s if s >= 0.75 => "S",
        s if s >= 0.60 => "A",
        s if s >= 0.45 => "B",
        s if s >= 0.30 => "C",
        s if s >= 0.15 => "D",
        _ => "F",
    }
}

#[test]
fn display_optimal_base_optimization() {
    let game_data = load_game_data();
    let fixture = load_user_fixture(UID);
    let user_building = UserBuilding::from_json(&fixture.building);
    let profiles = build_profiles(&fixture.roster, &game_data);

    let name_to_char = build_name_to_char(&game_data.operators);
    let (registry, morale_drains) = build_registry(&game_data.building.buffs, &name_to_char);

    let optimal = compute_optimal_assignment(
        &profiles,
        &user_building,
        &game_data.building,
        &registry,
        &morale_drains,
    );
    let sustained = compute_sustained_assignment(
        &profiles,
        &user_building,
        &game_data.building,
        &registry,
        &morale_drains,
    );

    // ---------- Display ----------
    println!("\n========================================");
    println!("  BASE OPTIMIZATION REPORT — UID {UID}");
    println!("========================================");
    println!("\nRoster: {} operators with base skills", profiles.len());

    println!("\nBASE LAYOUT:");
    let mut layout: std::collections::BTreeMap<&str, Vec<i32>> = std::collections::BTreeMap::new();
    for room in &user_building.rooms {
        layout.entry(&room.room_type).or_default().push(room.level);
    }
    for (room_type, levels) in &layout {
        let levels_str: Vec<String> = levels.iter().map(|l| format!("L{l}")).collect();
        println!("  {room_type}: {} ({})", levels.len(), levels_str.join(", "));
    }

    println!("\n----------------------------------------");
    println!("  OPTIMAL (single shift / peak)");
    println!("----------------------------------------");
    print_assignment(&optimal, &game_data);

    println!("\n----------------------------------------");
    println!("  SUSTAINED ROTATION (24/7)");
    println!("----------------------------------------");
    print_rotation(&sustained, &game_data);

    let base_score = grade_base(&fixture.roster, Some(&fixture.building), &game_data);
    println!("\n----------------------------------------");
    println!(
        "  Base Score: {base_score:.3}  →  {}",
        letter_grade(base_score)
    );
    println!("----------------------------------------\n");

    // ---------- Sanity assertions ----------
    assert!(!optimal.rooms.is_empty(), "optimal assignment has rooms");
    assert!(
        optimal.rooms.iter().any(|r| r.room_type == "CONTROL"),
        "Control Center should be surfaced in the assignment"
    );
    assert!(
        optimal.total_production_efficiency > 0.0,
        "total production efficiency should be positive"
    );
    assert!((0.0..=1.0).contains(&base_score), "score in [0,1]");

    // With a deep roster (this user has hundreds of operators) every room must
    // be staffed to its full slot count — no empty or short-handed rooms.
    for room in &optimal.rooms {
        let slots = max_stationed(&game_data, &room.room_type, room.level);
        assert_eq!(
            room.operators.len() as i32,
            slots,
            "{} (L{}, {}) should fill all {slots} slots, got {:?}",
            room.room_type,
            room.level,
            room.slot_id,
            room.operators
        );
    }
}
