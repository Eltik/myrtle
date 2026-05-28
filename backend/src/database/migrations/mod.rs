use sqlx::PgPool;

const MIGRATIONS: &[(&str, &str)] = &[
    ("v001_initial", include_str!("v001_initial.sql")),
    ("v002_views", include_str!("v002_views.sql")),
    ("v003_triggers", include_str!("v003_triggers.sql")),
    ("v004_procedures", include_str!("v004_procedures.sql")),
    ("v005_indexes", include_str!("v005_indexes.sql")),
    (
        "v006_operator_notes",
        include_str!("v006_operator_notes.sql"),
    ),
    (
        "v007_tier_list_stats",
        include_str!("v007_tier_list_stats.sql"),
    ),
    ("v008_support_units", include_str!("v008_support_units.sql")),
    ("v009_nick_number", include_str!("v009_nick_number.sql")),
    (
        "v010_leaderboard_snapshots",
        include_str!("v010_leaderboard_snapshots.sql"),
    ),
    (
        "v011_tier_list_visibility",
        include_str!("v011_tier_list_visibility.sql"),
    ),
    (
        "v012_skin_count_split",
        include_str!("v012_skin_count_split.sql"),
    ),
    (
        "v013_widen_tier_text",
        include_str!("v013_widen_tier_text.sql"),
    ),
    (
        "v014_propagate_tier_list_updated_at",
        include_str!("v014_propagate_tier_list_updated_at.sql"),
    ),
    (
        "v015_repair_tier_list_updated_at",
        include_str!("v015_repair_tier_list_updated_at.sql"),
    ),
    (
        "v016_gacha_batch_index",
        include_str!("v016_gacha_batch_index.sql"),
    ),
    (
        "v017_placement_description",
        include_str!("v017_placement_description.sql"),
    ),
    (
        "v018_consolidate_placement_notes",
        include_str!("v018_consolidate_placement_notes.sql"),
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

    for (name, sql) in MIGRATIONS {
        let applied: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = $1)")
                .bind(name)
                .fetch_one(pool)
                .await?;

        if !applied {
            let mut tx = pool.begin().await?;
            sqlx::raw_sql(sql).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO _migrations (name) VALUES ($1)")
                .bind(name)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            println!("Applied migration: {name}");
        }
    }

    Ok(())
}
