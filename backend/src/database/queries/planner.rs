use crate::database::models::planner::{OperatorPlan, PlanGroup, PlanInput};
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// A `plan_presets` row as stored. The service validates `target` into a
/// `PresetTarget` before it leaves the API.
#[derive(Debug, sqlx::FromRow)]
pub struct PlanPresetRow {
    pub id: Uuid,
    pub name: String,
    pub target: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn list_plans(pool: &PgPool, user_id: Uuid) -> Result<Vec<OperatorPlan>, sqlx::Error> {
    sqlx::query_as::<_, OperatorPlan>(
        "SELECT * FROM operator_plans WHERE user_id = $1 ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Creates or replaces the caller's plan for `operator_id`. With `groups`,
/// the plan's group memberships become exactly those names, creating any
/// group that does not exist yet; without, they are left as they are. One
/// transaction, so a failed group write leaves the plan unchanged too.
pub async fn upsert_plan(
    pool: &PgPool,
    user_id: Uuid,
    operator_id: &str,
    input: &PlanInput,
    groups: Option<&[String]>,
) -> Result<OperatorPlan, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let plan = sqlx::query_as::<_, OperatorPlan>(
        r"
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
        ",
    )
    .bind(user_id)
    .bind(operator_id)
    .bind(input.target_elite)
    .bind(input.target_level)
    .bind(input.target_skill_level)
    .bind(&input.target_skills)
    .bind(&input.target_modules)
    .bind(input.display_on_profile)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(group_names) = groups {
        sqlx::query("DELETE FROM plan_group_members WHERE operator_plan_id = $1")
            .bind(plan.id)
            .execute(&mut *tx)
            .await?;

        // A fixed lock order keeps concurrent upserts sharing groups from deadlocking.
        let mut group_names: Vec<&String> = group_names.iter().collect();
        group_names.sort();
        group_names.dedup();
        for group_name in group_names {
            let group_id: Uuid = sqlx::query_scalar(
                r"
                INSERT INTO plan_groups (user_id, name)
                VALUES ($1, $2)
                ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
                RETURNING id
                ",
            )
            .bind(user_id)
            .bind(group_name)
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query(
                r"
                INSERT INTO plan_group_members (plan_group_id, operator_plan_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                ",
            )
            .bind(group_id)
            .bind(plan.id)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(plan)
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

/// Deletes the caller's plans for `operator_ids` in one statement and returns
/// how many rows went. Their group memberships cascade.
pub async fn delete_plans(
    pool: &PgPool,
    user_id: Uuid,
    operator_ids: &[String],
) -> Result<u64, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM operator_plans WHERE user_id = $1 AND operator_id = ANY($2)")
            .bind(user_id)
            .bind(operator_ids)
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
}

/// Pinned groups first, then by name.
pub async fn list_groups(pool: &PgPool, user_id: Uuid) -> Result<Vec<PlanGroup>, sqlx::Error> {
    sqlx::query_as::<_, PlanGroup>(
        "SELECT * FROM plan_groups WHERE user_id = $1 ORDER BY pinned DESC, name ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn create_group(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
) -> Result<PlanGroup, sqlx::Error> {
    sqlx::query_as::<_, PlanGroup>(
        r"
        INSERT INTO plan_groups (user_id, name)
        VALUES ($1, $2)
        ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
        RETURNING *
        ",
    )
    .bind(user_id)
    .bind(name)
    .fetch_one(pool)
    .await
}

/// Renames and/or pins the group named `old_name`. A `None` field keeps its
/// stored value. No such group is `RowNotFound`; renaming onto another
/// group's name is a unique violation.
pub async fn update_group(
    pool: &PgPool,
    user_id: Uuid,
    old_name: &str,
    name: Option<&str>,
    pinned: Option<bool>,
) -> Result<PlanGroup, sqlx::Error> {
    sqlx::query_as::<_, PlanGroup>(
        r"
        UPDATE plan_groups
        SET name = COALESCE($1, name), pinned = COALESCE($2, pinned), updated_at = NOW()
        WHERE user_id = $3 AND name = $4
        RETURNING *
        ",
    )
    .bind(name)
    .bind(pinned)
    .bind(user_id)
    .bind(old_name)
    .fetch_one(pool)
    .await
}

pub async fn delete_group(pool: &PgPool, user_id: Uuid, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM plan_groups WHERE user_id = $1 AND name = $2")
        .bind(user_id)
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

/// Every (plan id, group name) membership across the caller's plans, each
/// plan's groups by name, as `get_plan_group_names` orders them.
pub async fn get_all_plan_groups(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<(Uuid, String)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String)>(
        r"
        SELECT pgm.operator_plan_id, pg.name
        FROM plan_groups pg
        JOIN plan_group_members pgm ON pgm.plan_group_id = pg.id
        WHERE pg.user_id = $1
        ORDER BY pgm.operator_plan_id, pg.name ASC
        ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// The names of the groups one plan is filed under, by name.
pub async fn get_plan_group_names(
    pool: &PgPool,
    operator_plan_id: Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        r"
        SELECT pg.name
        FROM plan_groups pg
        JOIN plan_group_members pgm ON pgm.plan_group_id = pg.id
        WHERE pgm.operator_plan_id = $1
        ORDER BY pg.name ASC
        ",
    )
    .bind(operator_plan_id)
    .fetch_all(pool)
    .await
}

pub async fn list_presets(pool: &PgPool, user_id: Uuid) -> Result<Vec<PlanPresetRow>, sqlx::Error> {
    sqlx::query_as::<_, PlanPresetRow>(
        "SELECT id, name, target, created_at, updated_at FROM plan_presets \
         WHERE user_id = $1 ORDER BY name ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Creates the preset, or replaces the target of the one already by that name.
pub async fn upsert_preset(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    target: serde_json::Value,
) -> Result<PlanPresetRow, sqlx::Error> {
    sqlx::query_as::<_, PlanPresetRow>(
        r"
        INSERT INTO plan_presets (user_id, name, target)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, name) DO UPDATE SET target = EXCLUDED.target
        RETURNING id, name, target, created_at, updated_at
        ",
    )
    .bind(user_id)
    .bind(name)
    .bind(target)
    .fetch_one(pool)
    .await
}

pub async fn delete_preset(pool: &PgPool, user_id: Uuid, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM plan_presets WHERE user_id = $1 AND name = $2")
        .bind(user_id)
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}
