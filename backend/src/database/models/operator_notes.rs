use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

// ============================================================================
// Core Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OperatorNote {
    pub id: Uuid,
    pub operator_id: String,
    pub pros: String,
    pub cons: String,
    pub notes: String,
    pub trivia: String,
    pub summary: String,
    pub tags: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OperatorNoteAuditLog {
    pub id: Uuid,
    pub operator_id: String,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Option<Uuid>,
    pub changed_at: DateTime<Utc>,
}

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateOperatorNote {
    pub pros: Option<String>,
    pub cons: Option<String>,
    pub notes: Option<String>,
    pub trivia: Option<String>,
    pub summary: Option<String>,
    pub tags: Option<serde_json::Value>,
}

// ============================================================================
// OperatorNote queries
// ============================================================================

impl OperatorNote {
    pub async fn find_by_operator(
        pool: &PgPool,
        operator_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM operator_notes WHERE operator_id = $1")
            .bind(operator_id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM operator_notes ORDER BY operator_id")
            .fetch_all(pool)
            .await
    }

    /// Returns only notes that have at least one non-empty content field.
    pub async fn find_all_with_content(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM operator_notes
            WHERE pros != '' OR cons != '' OR notes != '' OR trivia != '' OR summary != ''
            ORDER BY updated_at DESC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    /// Partial update using COALESCE so only provided fields are changed.
    pub async fn update(
        pool: &PgPool,
        operator_id: &str,
        input: &UpdateOperatorNote,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE operator_notes SET
                pros = COALESCE($2, pros),
                cons = COALESCE($3, cons),
                notes = COALESCE($4, notes),
                trivia = COALESCE($5, trivia),
                summary = COALESCE($6, summary),
                tags = COALESCE($7, tags),
                updated_at = NOW()
            WHERE operator_id = $1
            RETURNING *
            "#,
        )
        .bind(operator_id)
        .bind(&input.pros)
        .bind(&input.cons)
        .bind(&input.notes)
        .bind(&input.trivia)
        .bind(&input.summary)
        .bind(&input.tags)
        .fetch_one(pool)
        .await
    }

    /// Bulk-insert operator notes rows for all given IDs.
    /// Uses UNNEST for efficiency. ON CONFLICT DO NOTHING makes it idempotent.
    /// Returns the number of newly inserted rows.
    pub async fn seed_operators(pool: &PgPool, operator_ids: &[&str]) -> Result<u64, sqlx::Error> {
        let ids: Vec<String> = operator_ids.iter().map(|s| s.to_string()).collect();
        let result = sqlx::query(
            r#"
            INSERT INTO operator_notes (operator_id)
            SELECT unnest($1::varchar[])
            ON CONFLICT (operator_id) DO NOTHING
            "#,
        )
        .bind(&ids)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

// ============================================================================
// OperatorNoteAuditLog queries
// ============================================================================

impl OperatorNoteAuditLog {
    pub async fn create(
        pool: &PgPool,
        operator_id: &str,
        field_name: &str,
        old_value: Option<&str>,
        new_value: Option<&str>,
        changed_by: Uuid,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO operator_notes_audit_log (operator_id, field_name, old_value, new_value, changed_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(operator_id)
        .bind(field_name)
        .bind(old_value)
        .bind(new_value)
        .bind(changed_by)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_operator(
        pool: &PgPool,
        operator_id: &str,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM operator_notes_audit_log
            WHERE operator_id = $1
            ORDER BY changed_at DESC
            LIMIT $2
            "#,
        )
        .bind(operator_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn find_recent(pool: &PgPool, limit: i64) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM operator_notes_audit_log
            ORDER BY changed_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}
