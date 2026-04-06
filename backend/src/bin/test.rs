/// TODO: Delete later. Used for testing grades.
use std::path::Path;

use backend::core::grade::calculate::calculate_user_grade;
use backend::database::queries::users;
use uuid::Uuid;

async fn resolve_user_id(pool: &sqlx::PgPool, id_str: &str) -> Uuid {
    // If it parses as a UUID, use it directly
    if let Ok(uuid) = id_str.parse::<Uuid>() {
        return uuid;
    }

    // Otherwise treat it as an Arknights UID and look it up
    let profile = users::find_by_uid(pool, id_str)
        .await
        .expect("database error looking up uid")
        .unwrap_or_else(|| panic!("no user found with Arknights UID: {id_str}"));

    tracing::info!(uid = id_str, uuid = %profile.id, "resolved Arknights UID to database UUID");
    profile.id
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
        None => panic!("usage: test --id <uuid or arknights_uid>"),
    };

    let data_dir_str =
        std::env::var("GAME_DATA_DIR").unwrap_or_else(|_| "../assets/output/gamedata/excel".into());
    let assets_dir_str = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output".into());

    tracing::info!("loading game data...");
    let game_data = backend::core::gamedata::init_game_data(
        Path::new(&data_dir_str),
        Path::new(&assets_dir_str),
    )
    .expect("failed to load game data");
    tracing::info!(operators = game_data.operators.len(), "game data loaded");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = backend::database::init(&database_url)
        .await
        .expect("failed to initialize database");

    let user_id = resolve_user_id(&db, &id_str).await;

    let grade = calculate_user_grade(&db, user_id, &game_data)
        .await
        .expect("failed to calculate grade");

    tracing::info!(grade.overall);
    tracing::info!(grade.operator_grade);
    tracing::info!(grade.base_grade);
    tracing::info!(grade.total_score);
}
