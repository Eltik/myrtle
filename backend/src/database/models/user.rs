use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub uid: String,
    pub server: String,
    pub data: serde_json::Value,
    pub settings: serde_json::Value,
    pub role: String,
    #[serde(default)]
    pub score: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// For inserting new users (without id/created_at)
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub uid: String,
    pub server: String,
    pub data: serde_json::Value,
    pub score: serde_json::Value,
}

impl User {
    pub async fn create(pool: &PgPool, input: CreateUser) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO users (uid, server, data, score)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(&input.uid)
        .bind(&input.server)
        .bind(&input.data)
        .bind(&input.score)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_uid(pool: &PgPool, uid: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE uid = $1")
            .bind(uid)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_all(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    pub async fn update_data(
        pool: &PgPool,
        id: Uuid,
        data: serde_json::Value,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE users
            SET data = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(&data)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_by_server(pool: &PgPool, server: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE server = $1")
            .bind(server)
            .fetch_all(pool)
            .await
    }

    pub async fn search_by_nickname(
        pool: &PgPool,
        nickname: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE data->'status'->>'nickName' ILIKE $1")
            .bind(format!("%{nickname}%"))
            .fetch_all(pool)
            .await
    }

    /// Update user settings
    pub async fn update_settings(
        pool: &PgPool,
        id: Uuid,
        settings: serde_json::Value,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE users
            SET settings = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(&settings)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// Update user role
    pub async fn update_role(pool: &PgPool, id: Uuid, role: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE users
            SET role = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(role)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// Update score
    pub async fn update_score(
        pool: &PgPool,
        id: Uuid,
        score: &serde_json::Value,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE users
            SET score = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(score)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// Find users for leaderboard with dynamic sorting
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `sort_expression` - SQL expression for ORDER BY clause
    /// * `order` - Sort order ("ASC" or "DESC")
    /// * `server` - Optional server filter
    /// * `limit` - Maximum number of results
    /// * `offset` - Pagination offset
    pub async fn find_for_leaderboard(
        pool: &PgPool,
        sort_expression: &str,
        order: &str,
        server: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT * FROM users
            WHERE score IS NOT NULL
              AND score != 'null'::jsonb
              AND ($1::VARCHAR IS NULL OR server = $1)
              AND (settings->>'publicProfile' IS NULL OR (settings->>'publicProfile')::BOOLEAN = true)
            ORDER BY {} {} NULLS LAST
            LIMIT $2 OFFSET $3
            "#,
            sort_expression, order
        );

        sqlx::query_as::<_, Self>(&query)
            .bind(server)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    /// Count users for leaderboard pagination
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `server` - Optional server filter
    pub async fn count_for_leaderboard(
        pool: &PgPool,
        server: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM users
            WHERE score IS NOT NULL
              AND score != 'null'::jsonb
              AND ($1::VARCHAR IS NULL OR server = $1)
              AND (settings->>'publicProfile' IS NULL OR (settings->>'publicProfile')::BOOLEAN = true)
            "#,
        )
        .bind(server)
        .fetch_one(pool)
        .await
    }
}
