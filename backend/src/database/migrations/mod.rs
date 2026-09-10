use sqlx::PgPool;

/// Ordered list of migrations.
///
/// `v001_initial`..`v005_indexes` are the squashed baseline schema, generated
/// from `pg_dump` of the original v001..v022 migrations and split by object
/// type (tables, views, triggers, procedures, indexes). They reuse the original
/// migration names on purpose: any database that applied the original
/// migrations already has those names in `_migrations`, so the runner skips them
/// (their bodies only ever run against a fresh database). New changes are added
/// as their own migrations after the baseline (`v006_cumulative_signin`, ...).
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

    // Fetch the set of applied migrations once rather than querying per file.
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
