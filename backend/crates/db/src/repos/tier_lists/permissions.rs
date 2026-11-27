use chrono::{DateTime, Utc};
use myrtle_types::{TierListId, UserId};
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct PermissionRow {
    pub tier_list_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GrantRow {
    pub tier_list_id: Uuid,
    pub slug: String,
    pub title: String,
    pub list_type: String,
    pub user_id: Uuid,
    pub user_uid: String,
    pub user_nickname: Option<String>,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub granted_by_nickname: Option<String>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GrantedListRow {
    pub tier_list_id: Uuid,
    pub slug: String,
    pub title: String,
    pub list_type: String,
    pub permission: String,
}

#[derive(Debug, Clone)]
pub struct GranteeRow {
    pub user_id: Uuid,
    pub permission: String,
    pub uid: String,
    pub nickname: Option<String>,
}

pub async fn get_permissions(
    conn: &mut Conn,
    tier_list: TierListId,
) -> Result<Vec<PermissionRow>, DbError> {
    Ok(sqlx::query_as!(
        PermissionRow,
        "SELECT tier_list_id, user_id, permission, granted_by, granted_at
         FROM tier_list_permissions WHERE tier_list_id = $1
         ORDER BY CASE permission WHEN 'admin' THEN 0 WHEN 'publish' THEN 1
                                  WHEN 'edit' THEN 2 ELSE 3 END,
                  user_id, permission",
        tier_list.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn get_all_grants(
    conn: &mut Conn,
    user: Option<UserId>,
) -> Result<Vec<GrantRow>, DbError> {
    Ok(sqlx::query_as!(
        GrantRow,
        r#"SELECT p.tier_list_id,
                  t.slug,
                  t.name AS title,
                  t.list_type,
                  p.user_id,
                  u.uid AS user_uid,
                  u.nickname AS user_nickname,
                  p.permission,
                  p.granted_by,
                  g.nickname AS "granted_by_nickname?",
                  p.granted_at
           FROM tier_list_permissions p
           JOIN tier_lists t ON t.id = p.tier_list_id AND t.is_active
           JOIN users u ON u.id = p.user_id
           LEFT JOIN users g ON g.id = p.granted_by
           WHERE ($1::uuid IS NULL OR p.user_id = $1)
           ORDER BY p.granted_at DESC, t.slug, p.user_id, p.permission"#,
        user.map(|u| u.0),
    )
    .fetch_all(conn)
    .await?)
}

pub async fn get_granted_lists(
    conn: &mut Conn,
    user: UserId,
) -> Result<Vec<GrantedListRow>, DbError> {
    Ok(sqlx::query_as!(
        GrantedListRow,
        r#"SELECT g.tier_list_id AS "tier_list_id!", g.slug AS "slug!", g.title AS "title!",
                  g.list_type AS "list_type!", g.permission AS "permission!"
           FROM (
               SELECT DISTINCT ON (t.id)
                      t.id AS tier_list_id,
                      t.slug,
                      t.name AS title,
                      t.list_type,
                      p.permission
               FROM tier_list_permissions p
               JOIN tier_lists t ON t.id = p.tier_list_id AND t.is_active
               WHERE p.user_id = $1
               ORDER BY t.id,
                        CASE p.permission WHEN 'admin' THEN 0 WHEN 'publish' THEN 1
                                          WHEN 'edit' THEN 2 ELSE 3 END
           ) g
           ORDER BY lower(g.title), g.slug"#,
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn get_user_permission(
    conn: &mut Conn,
    tier_list: TierListId,
    user: UserId,
) -> Result<Option<PermissionRow>, DbError> {
    Ok(sqlx::query_as!(
        PermissionRow,
        "SELECT tier_list_id, user_id, permission, granted_by, granted_at
         FROM tier_list_permissions
         WHERE tier_list_id = $1 AND user_id = $2
         ORDER BY CASE permission WHEN 'admin' THEN 0 WHEN 'publish' THEN 1
                                  WHEN 'edit' THEN 2 ELSE 3 END
         LIMIT 1",
        tier_list.0,
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn grant_permission(
    conn: &mut Conn,
    tier_list: TierListId,
    user: UserId,
    permission: &str,
    granted_by: UserId,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO tier_list_permissions (tier_list_id, user_id, permission, granted_by)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (tier_list_id, user_id, permission) DO NOTHING",
        tier_list.0,
        user.0,
        permission,
        granted_by.0,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn upsert_grant(
    conn: &mut Conn,
    tier_list: TierListId,
    user: UserId,
    permission: &str,
    granted_by: Option<UserId>,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO tier_list_permissions (tier_list_id, user_id, permission, granted_by)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (tier_list_id, user_id, permission) DO UPDATE SET granted_at = NOW()",
        tier_list.0,
        user.0,
        permission,
        granted_by.map(|g| g.0),
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn revoke_permission(
    conn: &mut Conn,
    tier_list: TierListId,
    user: UserId,
    permission: &str,
) -> Result<(), DbError> {
    sqlx::query!(
        "DELETE FROM tier_list_permissions
         WHERE tier_list_id = $1 AND user_id = $2 AND permission = $3",
        tier_list.0,
        user.0,
        permission,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn list_grantees(
    conn: &mut Conn,
    tier_list: TierListId,
) -> Result<Vec<GranteeRow>, DbError> {
    Ok(sqlx::query_as!(
        GranteeRow,
        "SELECT p.user_id, p.permission, u.uid, u.nickname
         FROM tier_list_permissions p
         JOIN users u ON p.user_id = u.id
         WHERE p.tier_list_id = $1
         ORDER BY CASE p.permission WHEN 'admin' THEN 0 WHEN 'publish' THEN 1
                                    WHEN 'edit' THEN 2 ELSE 3 END,
                  u.uid, p.permission",
        tier_list.0,
    )
    .fetch_all(conn)
    .await?)
}
