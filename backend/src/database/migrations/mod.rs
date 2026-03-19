use sqlx::PgPool;

const MIGRATIONS: &[(&str, &str)] = &[
    ("v001_initial", include_str!("v001_initial.sql")),
    ("v002_views", include_str!("v002_views.sql")),
    ("v003_triggers", include_str!("v003_triggers.sql")),
    ("v004_procedures", include_str!("v004_procedures.sql")),
    ("v005_indexes", include_str!("v005_indexes.sql")),
];

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            name VARCHAR(100) PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#,
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
