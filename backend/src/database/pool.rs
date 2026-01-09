use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await
}

pub async fn init_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            uid VARCHAR(255) NOT NULL UNIQUE,
            server VARCHAR(50) NOT NULL,
            data JSONB NOT NULL DEFAULT '{}',
            settings JSONB NOT NULL DEFAULT '{}',
            role VARCHAR(50) NOT NULL DEFAULT 'user',
            score JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#,
    )
    .execute(pool)
    .await?;

    // Tier lists table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tier_lists (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            slug VARCHAR(100) NOT NULL UNIQUE,
            description TEXT,
            is_active BOOLEAN NOT NULL DEFAULT true,
            created_by UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Tiers table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tiers (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tier_list_id UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
            name VARCHAR(50) NOT NULL,
            display_order INT NOT NULL,
            color VARCHAR(7),
            description TEXT,
            UNIQUE(tier_list_id, display_order)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Tier placements table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tier_placements (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tier_id UUID NOT NULL REFERENCES tiers(id) ON DELETE CASCADE,
            operator_id VARCHAR(100) NOT NULL,
            sub_order INT NOT NULL DEFAULT 0,
            notes TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(tier_id, operator_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_tier_placements_operator ON tier_placements(operator_id)",
    )
    .execute(pool)
    .await?;

    // Tier list versions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tier_list_versions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tier_list_id UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
            version INT NOT NULL,
            snapshot JSONB NOT NULL,
            changelog TEXT NOT NULL,
            change_summary VARCHAR(500),
            published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            published_by UUID REFERENCES users(id),
            UNIQUE(tier_list_id, version)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_versions_tier_list ON tier_list_versions(tier_list_id, version DESC)",
    )
    .execute(pool)
    .await?;

    // Tier change log table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tier_change_log (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tier_list_id UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
            version_id UUID REFERENCES tier_list_versions(id),
            change_type VARCHAR(50) NOT NULL,
            operator_id VARCHAR(100),
            old_tier_id UUID,
            new_tier_id UUID,
            old_value JSONB,
            new_value JSONB,
            reason TEXT,
            changed_by UUID REFERENCES users(id),
            changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_change_log_operator ON tier_change_log(operator_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_change_log_time ON tier_change_log(changed_at DESC)",
    )
    .execute(pool)
    .await?;

    // Tier list permissions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tier_list_permissions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tier_list_id UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            permission VARCHAR(50) NOT NULL,
            granted_by UUID REFERENCES users(id),
            granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(tier_list_id, user_id, permission)
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
