//! One-off: dump a user's building + roster to a JSON test fixture.
//!
//!   cargo run --bin dump-user-fixture -- 09525371
//!
//! Writes tests/fixtures/user_<uid>.json containing the raw building JSON and
//! the full roster, so the base-optimizer tests can run against real data
//! without touching the database.

use anyhow::{Context, Result};
use backend::database::queries::{building, roster, users};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let uid = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "09525371".to_string());

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    let profile = users::find_by_uid(&pool, &uid)
        .await?
        .with_context(|| format!("no user with uid {uid}"))?;

    let building = building::get_building(&pool, profile.id)
        .await?
        .context("user has no building data")?;
    let roster = roster::get_roster(&pool, profile.id).await?;

    let fixture = serde_json::json!({
        "uid": uid,
        "user_id": profile.id,
        "building": building,
        "roster": roster,
    });

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("user_{uid}.json"));
    std::fs::write(&path, serde_json::to_string_pretty(&fixture)?)?;

    println!(
        "wrote {} ({} operators) -> {}",
        uid,
        roster.len(),
        path.display()
    );
    Ok(())
}
