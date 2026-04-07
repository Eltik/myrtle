/// Test bin for displaying base optimization details.
/// Shows which operators the algorithm assigns to each room and the resulting efficiency.
///
/// Usage: cargo run --bin grade-example -- --id <uuid or arknights_uid>
use std::path::Path;

use backend::core::gamedata::types::GameData;
use backend::core::grade::base::assignment::compute_sustained_assignment;
use backend::core::grade::base::{
    assignment::compute_optimal_assignment,
    buff_registry::build_registry,
    score::grade_base,
    types::{OperatorBaseProfile, UserBuilding},
};
use backend::database::models::roster::RosterEntry;
use backend::database::queries::{building, roster, users};
use uuid::Uuid;

async fn resolve_user_id(pool: &sqlx::PgPool, id_str: &str) -> Uuid {
    if let Ok(uuid) = id_str.parse::<Uuid>() {
        return uuid;
    }
    let profile = users::find_by_uid(pool, id_str)
        .await
        .expect("database error looking up uid")
        .unwrap_or_else(|| panic!("no user found with Arknights UID: {id_str}"));
    profile.id
}

fn build_profiles(roster: &[RosterEntry], game_data: &GameData) -> Vec<OperatorBaseProfile> {
    roster
        .iter()
        .filter_map(|entry| {
            let bc = game_data.building.chars.get(&entry.operator_id)?;
            Some(OperatorBaseProfile::build(entry, bc))
        })
        .collect()
}

fn operator_name<'a>(char_id: &'a str, game_data: &'a GameData) -> &'a str {
    game_data
        .operators
        .get(char_id)
        .map(|op| op.name.as_str())
        .unwrap_or(char_id)
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let mut args = std::env::args().skip(1);
    let id_str = match args.next().as_deref() {
        Some("--id") => args.next().expect("--id requires a value"),
        Some(other) => panic!("unknown argument: {other}"),
        None => panic!("usage: grade-example --id <uuid or arknights_uid>"),
    };

    let data_dir_str =
        std::env::var("GAME_DATA_DIR").unwrap_or_else(|_| "../assets/output/gamedata/excel".into());
    let assets_dir_str = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output".into());

    println!("Loading game data...");
    let game_data = backend::core::gamedata::init_game_data(
        Path::new(&data_dir_str),
        Path::new(&assets_dir_str),
    )
    .expect("failed to load game data");
    println!(
        "Loaded {} operators, {} building buffs, {} building chars",
        game_data.operators.len(),
        game_data.building.buffs.len(),
        game_data.building.chars.len(),
    );

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = backend::database::init(&database_url)
        .await
        .expect("failed to initialize database");

    let user_id = resolve_user_id(&db, &id_str).await;

    // Fetch data
    let user_roster = roster::get_roster(&db, user_id)
        .await
        .expect("failed to get roster");
    let building_json = building::get_building(&db, user_id)
        .await
        .expect("failed to get building");

    println!("\n=== ROSTER ===");
    println!("{} operators in roster", user_roster.len());

    // Build profiles and registry
    let profiles = build_profiles(&user_roster, &game_data);
    let (registry, morale_drains) = build_registry(&game_data.building.buffs);
    println!("{} operators with base skills", profiles.len());
    println!("{} buffs classified", registry.len());

    // Parse building
    let building_json = match building_json {
        Some(json) => json,
        None => {
            println!("No building data synced for this user.");
            return;
        }
    };
    let user_building = UserBuilding::from_json(&building_json);
    if user_building.is_empty() {
        println!("Building data is empty.");
        return;
    }

    // Show layout
    println!("\n=== BASE LAYOUT ===");
    let mut layout_counts: std::collections::HashMap<&str, Vec<i32>> =
        std::collections::HashMap::new();
    for room in &user_building.rooms {
        layout_counts
            .entry(&room.room_type)
            .or_default()
            .push(room.level);
    }
    let mut sorted: Vec<_> = layout_counts.iter().collect();
    sorted.sort_by_key(|(name, _)| *name);
    for (room_type, levels) in &sorted {
        let levels_str: Vec<String> = levels.iter().map(|l| format!("L{l}")).collect();
        println!(
            "  {room_type}: {} ({})",
            levels.len(),
            levels_str.join(", ")
        );
    }

    // Run optimal assignment
    println!("\n=== OPTIMAL ASSIGNMENT ===");
    let assignment = compute_optimal_assignment(
        &profiles,
        &user_building,
        &game_data.building,
        &registry,
        &morale_drains,
    );

    for room in &assignment.rooms {
        let ops: Vec<String> = room
            .operators
            .iter()
            .map(|id| {
                let name = operator_name(id, &game_data);
                format!("{name} ({id})")
            })
            .collect();
        let formula_label = room.formula_type.as_deref().unwrap_or("");
        println!(
            "\n  {} (L{}) {formula_label} — +{:.1}% efficiency",
            room.room_type, room.level, room.total_efficiency
        );
        for op in &ops {
            println!("    - {op}");
        }
        if room.operators.is_empty() {
            println!("    (no beneficial operators)");
        }
    }

    println!(
        "\n  TOTAL PRODUCTION EFFICIENCY: +{:.1}%",
        assignment.total_production_efficiency
    );

    // Run the full grade
    println!("\n=== BASE GRADE ===");
    let base_score = grade_base(&user_roster, Some(&building_json), &game_data);
    println!("  Base Score: {base_score:.4} / 1.0");

    let grade = match base_score {
        s if s >= 0.90 => "S+",
        s if s >= 0.75 => "S",
        s if s >= 0.60 => "A",
        s if s >= 0.45 => "B",
        s if s >= 0.30 => "C",
        s if s >= 0.15 => "D",
        _ => "F",
    };
    println!("  Letter Grade: {grade}");

    // Run sustained assignment
    println!("\n=== OPTIMAL ASSIGNMENT (SHIFT A) ===");
    let sustained = compute_sustained_assignment(
        &profiles,
        &user_building,
        &game_data.building,
        &registry,
        &morale_drains,
    );

    for room in &sustained.shift_a.rooms {
        let ops: Vec<String> = room
            .operators
            .iter()
            .map(|id| {
                let name = operator_name(id, &game_data);
                format!("{name} ({id})")
            })
            .collect();
        let formula_label = room.formula_type.as_deref().unwrap_or("");
        println!(
            "\n  {} (L{}) {formula_label} — +{:.1}% efficiency",
            room.room_type, room.level, room.total_efficiency
        );
        for op in &ops {
            println!("    - {op}");
        }
    }
    println!(
        "\n  SHIFT A TOTAL: +{:.1}%",
        sustained.shift_a.total_production_efficiency
    );

    println!("\n=== ROTATION (SHIFT B) ===");
    for room in &sustained.shift_b.rooms {
        let ops: Vec<String> = room
            .operators
            .iter()
            .map(|id| {
                let name = operator_name(id, &game_data);
                format!("{name} ({id})")
            })
            .collect();
        let formula_label = room.formula_type.as_deref().unwrap_or("");
        println!(
            "\n  {} (L{}) {formula_label} — +{:.1}% efficiency",
            room.room_type, room.level, room.total_efficiency
        );
        for op in &ops {
            println!("    - {op}");
        }
        if room.operators.is_empty() {
            println!("    (no beneficial operators)");
        }
    }
    println!(
        "\n  SHIFT B TOTAL: +{:.1}%",
        sustained.shift_b.total_production_efficiency
    );
    println!(
        "\n  SUSTAINED AVERAGE: +{:.1}%",
        sustained.sustained_efficiency
    );
}
