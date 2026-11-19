use chrono::{DateTime, Utc};
use myrtle_types::UserId;
use serde_json::Value;
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct PlanRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub operator_id: String,
    pub target_elite: i16,
    pub target_level: i16,
    pub target_skill_level: i16,
    pub target_skills: Value,
    pub target_modules: Value,
    pub display_on_profile: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct PlanInput<'a> {
    pub target_elite: i16,
    pub target_level: i16,
    pub target_skill_level: i16,
    pub target_skills: &'a Value,
    pub target_modules: &'a Value,
    pub display_on_profile: bool,
}

#[derive(Debug, Clone)]
pub struct PlanGroupRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PlanMembershipRow {
    pub operator_plan_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PlanPresetRow {
    pub id: Uuid,
    pub name: String,
    pub target: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ShownPlanRow {
    pub id: Uuid,
    pub operator_id: String,
}

pub async fn list_plans(conn: &mut Conn, user: UserId) -> Result<Vec<PlanRow>, DbError> {
    Ok(sqlx::query_as!(
        PlanRow,
        "SELECT id, user_id, operator_id, target_elite, target_level, target_skill_level,
                target_skills, target_modules, display_on_profile, created_at, updated_at
         FROM operator_plans WHERE user_id = $1 ORDER BY updated_at DESC",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn upsert_plan(
    conn: &mut Conn,
    user: UserId,
    operator_id: &str,
    input: PlanInput<'_>,
    groups: Option<&[String]>,
) -> Result<PlanRow, DbError> {
    let mut tx = sqlx::Connection::begin(&mut *conn).await?;
    let plan = sqlx::query_as!(
        PlanRow,
        "INSERT INTO operator_plans (
             user_id, operator_id, target_elite, target_level, target_skill_level,
             target_skills, target_modules, display_on_profile
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (user_id, operator_id) DO UPDATE SET
             target_elite = EXCLUDED.target_elite,
             target_level = EXCLUDED.target_level,
             target_skill_level = EXCLUDED.target_skill_level,
             target_skills = EXCLUDED.target_skills,
             target_modules = EXCLUDED.target_modules,
             display_on_profile = EXCLUDED.display_on_profile
         RETURNING id, user_id, operator_id, target_elite, target_level, target_skill_level,
                   target_skills, target_modules, display_on_profile, created_at, updated_at",
        user.0,
        operator_id,
        input.target_elite,
        input.target_level,
        input.target_skill_level,
        input.target_skills,
        input.target_modules,
        input.display_on_profile,
    )
    .fetch_one(&mut *tx)
    .await?;

    if let Some(groups) = groups {
        sqlx::query!(
            "DELETE FROM plan_group_members WHERE operator_plan_id = $1",
            plan.id,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "WITH grouped AS (
                 INSERT INTO plan_groups (user_id, name)
                 SELECT DISTINCT $1::uuid, n FROM UNNEST($2::text[]) AS n ORDER BY n
                 ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
                 RETURNING id
             )
             INSERT INTO plan_group_members (plan_group_id, operator_plan_id)
             SELECT id, $3 FROM grouped
             ON CONFLICT DO NOTHING",
            user.0,
            groups, // upserted in name order, so saves sharing groups cannot deadlock
            plan.id,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(plan)
}

pub async fn delete_plan(conn: &mut Conn, user: UserId, operator_id: &str) -> Result<(), DbError> {
    sqlx::query!(
        "DELETE FROM operator_plans WHERE user_id = $1 AND operator_id = $2",
        user.0,
        operator_id,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn delete_plans(
    conn: &mut Conn,
    user: UserId,
    operator_ids: &[String],
) -> Result<u64, DbError> {
    let result = sqlx::query!(
        "DELETE FROM operator_plans WHERE user_id = $1 AND operator_id = ANY($2)",
        user.0,
        operator_ids,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected())
}

pub async fn shown_plans(
    conn: &mut Conn,
    owner: UserId,
    ids: &[Uuid],
    operator_ids: &[String],
) -> Result<Vec<ShownPlanRow>, DbError> {
    Ok(sqlx::query_as!(
        ShownPlanRow,
        "SELECT id, operator_id FROM operator_plans
         WHERE user_id = $2 AND display_on_profile
           AND (id = ANY($1) OR operator_id = ANY($3))",
        ids,
        owner.0,
        operator_ids,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn list_groups(conn: &mut Conn, user: UserId) -> Result<Vec<PlanGroupRow>, DbError> {
    Ok(sqlx::query_as!(
        PlanGroupRow,
        "SELECT id, user_id, name, pinned, created_at, updated_at
         FROM plan_groups WHERE user_id = $1 ORDER BY pinned DESC, name ASC",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn create_group(
    conn: &mut Conn,
    user: UserId,
    name: &str,
) -> Result<PlanGroupRow, DbError> {
    Ok(sqlx::query_as!(
        PlanGroupRow,
        "INSERT INTO plan_groups (user_id, name) VALUES ($1, $2)
         ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
         RETURNING id, user_id, name, pinned, created_at, updated_at",
        user.0,
        name,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn update_group(
    conn: &mut Conn,
    user: UserId,
    old_name: &str,
    name: Option<&str>,
    pinned: Option<bool>,
) -> Result<PlanGroupRow, DbError> {
    Ok(sqlx::query_as!(
        PlanGroupRow,
        "UPDATE plan_groups
         SET name = COALESCE($1, name), pinned = COALESCE($2, pinned), updated_at = NOW()
         WHERE user_id = $3 AND name = $4
         RETURNING id, user_id, name, pinned, created_at, updated_at",
        name,
        pinned,
        user.0,
        old_name,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn delete_group(conn: &mut Conn, user: UserId, name: &str) -> Result<(), DbError> {
    sqlx::query!(
        "DELETE FROM plan_groups WHERE user_id = $1 AND name = $2",
        user.0,
        name,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn list_memberships(
    conn: &mut Conn,
    user: UserId,
) -> Result<Vec<PlanMembershipRow>, DbError> {
    Ok(sqlx::query_as!(
        PlanMembershipRow,
        "SELECT pgm.operator_plan_id, pg.name
         FROM plan_groups pg
         JOIN plan_group_members pgm ON pgm.plan_group_id = pg.id
         WHERE pg.user_id = $1
         ORDER BY pgm.operator_plan_id, pg.name ASC",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn group_names(conn: &mut Conn, operator_plan_id: Uuid) -> Result<Vec<String>, DbError> {
    Ok(sqlx::query_scalar!(
        "SELECT pg.name
         FROM plan_groups pg
         JOIN plan_group_members pgm ON pgm.plan_group_id = pg.id
         WHERE pgm.operator_plan_id = $1
         ORDER BY pg.name ASC",
        operator_plan_id,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn list_presets(conn: &mut Conn, user: UserId) -> Result<Vec<PlanPresetRow>, DbError> {
    Ok(sqlx::query_as!(
        PlanPresetRow,
        "SELECT id, name, target, created_at, updated_at FROM plan_presets
         WHERE user_id = $1 ORDER BY name ASC",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn upsert_preset(
    conn: &mut Conn,
    user: UserId,
    name: &str,
    target: &Value,
) -> Result<PlanPresetRow, DbError> {
    Ok(sqlx::query_as!(
        PlanPresetRow,
        "INSERT INTO plan_presets (user_id, name, target) VALUES ($1, $2, $3)
         ON CONFLICT (user_id, name) DO UPDATE SET target = EXCLUDED.target
         RETURNING id, name, target, created_at, updated_at",
        user.0,
        name,
        target,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn delete_preset(conn: &mut Conn, user: UserId, name: &str) -> Result<(), DbError> {
    sqlx::query!(
        "DELETE FROM plan_presets WHERE user_id = $1 AND name = $2",
        user.0,
        name,
    )
    .execute(conn)
    .await?;
    Ok(())
}
