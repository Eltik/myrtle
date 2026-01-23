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

    // Add community tier list columns to tier_lists table
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tier_lists' AND column_name = 'tier_list_type') THEN
                ALTER TABLE tier_lists ADD COLUMN tier_list_type VARCHAR(20) NOT NULL DEFAULT 'official';
            END IF;
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tier_lists' AND column_name = 'is_deleted') THEN
                ALTER TABLE tier_lists ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT false;
            END IF;
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tier_lists' AND column_name = 'deleted_by') THEN
                ALTER TABLE tier_lists ADD COLUMN deleted_by UUID REFERENCES users(id);
            END IF;
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tier_lists' AND column_name = 'deleted_reason') THEN
                ALTER TABLE tier_lists ADD COLUMN deleted_reason TEXT;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    // Index for tier list type filtering
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tier_lists_type ON tier_lists(tier_list_type)")
        .execute(pool)
        .await?;

    // Index for community tier lists by owner
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_tier_lists_community_owner ON tier_lists(created_by) WHERE tier_list_type = 'community'",
    )
    .execute(pool)
    .await?;

    // Tier list reports table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tier_list_reports (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tier_list_id UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
            reporter_id UUID NOT NULL REFERENCES users(id),
            reason VARCHAR(50) NOT NULL,
            description TEXT,
            status VARCHAR(20) NOT NULL DEFAULT 'pending',
            reviewed_by UUID REFERENCES users(id),
            reviewed_at TIMESTAMPTZ,
            action_taken TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(tier_list_id, reporter_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_tier_list_reports_status ON tier_list_reports(status)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_tier_list_reports_tier_list ON tier_list_reports(tier_list_id)",
    )
    .execute(pool)
    .await?;

    // Leaderboard performance indices
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_server ON users(server)")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_users_total_score ON users (((score->>'totalScore')::FLOAT) DESC NULLS LAST) WHERE score IS NOT NULL AND score != 'null'::jsonb",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_users_composite_score ON users (((score->'grade'->>'compositeScore')::FLOAT) DESC NULLS LAST) WHERE score IS NOT NULL AND score != 'null'::jsonb",
    )
    .execute(pool)
    .await?;

    // Search performance indices
    // Note: pg_trgm extension must be enabled for GIN trigram index
    // Run: CREATE EXTENSION IF NOT EXISTS pg_trgm;

    // Level index for range queries
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_users_level ON users (((data->'status'->>'level')::INT))",
    )
    .execute(pool)
    .await?;

    // Grade index for exact match filtering
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_grade ON users ((score->'grade'->>'grade'))")
        .execute(pool)
        .await?;

    // =========================================================================
    // Gacha Records Tables
    // =========================================================================

    // Individual gacha pull records
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS gacha_records (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            pull_timestamp BIGINT NOT NULL,
            char_id VARCHAR(100) NOT NULL,
            pool_id VARCHAR(100) NOT NULL,
            char_name VARCHAR(255) NOT NULL,
            rarity SMALLINT NOT NULL,
            pool_name VARCHAR(255) NOT NULL,
            gacha_type VARCHAR(50) NOT NULL,
            type_name VARCHAR(100) NOT NULL,
            pull_timestamp_str VARCHAR(50),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            CONSTRAINT unique_user_pull UNIQUE (user_id, pull_timestamp, char_id, pool_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // User gacha settings (opt-out model)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_gacha_settings (
            user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
            store_records BOOLEAN NOT NULL DEFAULT true,
            share_anonymous_stats BOOLEAN NOT NULL DEFAULT true,
            last_sync_at TIMESTAMPTZ,
            last_sync_limited_at BIGINT,
            last_sync_regular_at BIGINT,
            last_sync_special_at BIGINT,
            total_pulls INTEGER NOT NULL DEFAULT 0,
            six_star_count INTEGER NOT NULL DEFAULT 0,
            five_star_count INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Gacha records indexes
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_gacha_records_user_time ON gacha_records(user_id, pull_timestamp DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_gacha_records_char_id ON gacha_records(char_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_gacha_records_rarity ON gacha_records(rarity)")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_gacha_settings_share_stats ON user_gacha_settings(share_anonymous_stats) WHERE share_anonymous_stats = true",
    )
    .execute(pool)
    .await?;

    // Additional indexes for gacha history queries with filters
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_gacha_records_user_rarity ON gacha_records(user_id, rarity)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_gacha_records_user_gacha_type ON gacha_records(user_id, gacha_type)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_gacha_records_user_char ON gacha_records(user_id, char_id)",
    )
    .execute(pool)
    .await?;

    Ok(())
}
