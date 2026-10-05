use sqlx::PgPool;

/// Ordered migrations.
///
/// `v001_initial`..`v005_indexes` are the squashed baseline (`pg_dump` of the old
/// v001..v022, split by object type). They keep the old names on purpose: a database
/// that applied the originals already has them in `_migrations` and skips them, so
/// the bodies only run on a fresh database. New changes go after the baseline.
const MIGRATIONS: &[(&str, &str)] = &[
    ("v001_initial", include_str!("v001_initial.sql")),
    ("v002_views", include_str!("v002_views.sql")),
    ("v003_triggers", include_str!("v003_triggers.sql")),
    ("v004_procedures", include_str!("v004_procedures.sql")),
    ("v005_indexes", include_str!("v005_indexes.sql")),
    (
        "v006_cumulative_signin",
        include_str!("v006_cumulative_signin.sql"),
    ),
    (
        "v007_performance_indexes",
        include_str!("v007_performance_indexes.sql"),
    ),
    (
        "v008_medal_ownership",
        include_str!("v008_medal_ownership.sql"),
    ),
    (
        "v009_profile_sync_ts",
        include_str!("v009_profile_sync_ts.sql"),
    ),
    ("v010_base_facts", include_str!("v010_base_facts.sql")),
    (
        "v011_base_grade_components",
        include_str!("v011_base_grade_components.sql"),
    ),
    (
        "v012_game_credentials",
        include_str!("v012_game_credentials.sql"),
    ),
    (
        "v013_checkin_semantics",
        include_str!("v013_checkin_semantics.sql"),
    ),
    ("v014_build_stats", include_str!("v014_build_stats.sql")),
    (
        "v015_investment_levels",
        include_str!("v015_investment_levels.sql"),
    ),
    (
        "v016_release_ledger",
        include_str!("v016_release_ledger.sql"),
    ),
    (
        "v017_release_override_names",
        include_str!("v017_release_override_names.sql"),
    ),
    (
        "v018_release_override_rosters",
        include_str!("v018_release_override_rosters.sql"),
    ),
    (
        "v019_user_status_originite",
        include_str!("v019_user_status_originite.sql"),
    ),
    ("v020_release_plans", include_str!("v020_release_plans.sql")),
    ("v021_i18n", include_str!("v021_i18n.sql")),
    ("v022_source_locale", include_str!("v022_source_locale.sql")),
    (
        "v023_translation_source_snapshot",
        include_str!("v023_translation_source_snapshot.sql"),
    ),
    (
        "v024_user_items_leaderboard_index",
        include_str!("v024_user_items_leaderboard_index.sql"),
    ),
    (
        "v025_user_story_progress",
        include_str!("v025_user_story_progress.sql"),
    ),
    (
        "v026_user_game_story_read",
        include_str!("v026_user_game_story_read.sql"),
    ),
    (
        "v027_game_story_read_semantics",
        include_str!("v027_game_story_read_semantics.sql"),
    ),
    (
        "v028_game_story_read_verdict",
        include_str!("v028_game_story_read_verdict.sql"),
    ),
    ("v029_tier_entities", include_str!("v029_tier_entities.sql")),
    (
        "v030_planner_pins_presets",
        include_str!("v030_planner_pins_presets.sql"),
    ),
];

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS _migrations (
            name VARCHAR(100) PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    ",
    )
    .execute(pool)
    .await?;

    let applied: std::collections::HashSet<String> =
        sqlx::query_scalar("SELECT name FROM _migrations")
            .fetch_all(pool)
            .await?
            .into_iter()
            .collect();

    for (name, sql) in MIGRATIONS {
        if !applied.contains(*name) {
            let mut tx = pool.begin().await?;
            sqlx::raw_sql(sql).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO _migrations (name) VALUES ($1)")
                .bind(name)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            tracing::info!(migration = name, "applied migration");
        }
    }

    Ok(())
}
