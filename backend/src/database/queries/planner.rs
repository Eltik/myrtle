use crate::database::models::planner::OperatorPlan;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_plans(pool: &PgPool, user_id: Uuid) -> Result<Vec<OperatorPlan>, sqlx::Error> {
    sqlx::query_as::<_, OperatorPlan>(
        "SELECT * FROM operator_plans WHERE user_id = $1 ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn get_plan(
    pool: &PgPool,
    user_id: Uuid,
    operator_id: &str,
) -> Result<Option<OperatorPlan>, sqlx::Error> {
    sqlx::query_as::<_, OperatorPlan>(
        "SELECT * FROM operator_plans WHERE user_id = $1 AND operator_id = $2",
    )
    .bind(user_id)
    .bind(operator_id)
    .fetch_optional(pool)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_plan(
    pool: &PgPool,
    user_id: Uuid,
    operator_id: &str,
    target_elite: i16,
    target_level: i16,
    target_skill_level: i16,
    target_skills: serde_json::Value,
    target_modules: serde_json::Value,
    display_on_profile: bool,
) -> Result<OperatorPlan, sqlx::Error> {
    sqlx::query_as::<_, OperatorPlan>(
        r#"
        INSERT INTO operator_plans (
            user_id,
            operator_id,
            target_elite,
            target_level,
            target_skill_level,
            target_skills,
            target_modules,
            display_on_profile
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (user_id, operator_id) DO UPDATE SET
            target_elite = EXCLUDED.target_elite,
            target_level = EXCLUDED.target_level,
            target_skill_level = EXCLUDED.target_skill_level,
            target_skills = EXCLUDED.target_skills,
            target_modules = EXCLUDED.target_modules,
            display_on_profile = EXCLUDED.display_on_profile,
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(operator_id)
    .bind(target_elite)
    .bind(target_level)
    .bind(target_skill_level)
    .bind(target_skills)
    .bind(target_modules)
    .bind(display_on_profile)
    .fetch_one(pool)
    .await
}

pub async fn delete_plan(
    pool: &PgPool,
    user_id: Uuid,
    operator_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM operator_plans WHERE user_id = $1 AND operator_id = $2")
        .bind(user_id)
        .bind(operator_id)
        .execute(pool)
        .await?;
    Ok(())
}
