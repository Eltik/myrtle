use chrono::{DateTime, Utc};
use myrtle_types::{Server, UserId};
use serde_json::Value;
use uuid::Uuid;

use crate::sql::{contains_pattern, escape_like};
use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: Uuid,
    pub uid: String,
    pub server_id: i16,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub role: String,
    pub avatar_id: Option<String>,
    pub resume_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProfileRow {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub resume_id: Option<String>,
    pub role: String,
    pub server: String,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
    pub public_profile: Option<bool>,
    pub store_gacha: Option<bool>,
    pub share_stats: Option<bool>,
    pub exp: Option<i32>,
    pub orundum: Option<i32>,
    pub lmd: Option<i32>,
    pub sanity: Option<i16>,
    pub max_sanity: Option<i16>,
    pub gacha_tickets: Option<i32>,
    pub ten_pull_tickets: Option<i32>,
    pub monthly_sub_end: Option<i64>,
    pub register_ts: Option<i64>,
    pub last_online_ts: Option<i64>,
    pub resume: Option<String>,
    pub friend_num_limit: Option<i16>,
    pub cumulative_signin: Option<i32>,
    pub operator_count: Option<i64>,
    pub item_count: Option<i64>,
    pub skin_count: Option<i64>,
    pub non_default_skin_count: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub originite: Option<i32>,
    pub profile_layout: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct CheckinRow {
    pub history: Vec<i16>,
    pub claimed_this_month: i32,
    pub cumulative_signin: i32,
    pub checkin_group_id: Option<String>,
    pub reward_index: i16,
    pub can_check_in: bool,
    pub register_ts: Option<i64>,
    pub last_online_ts: Option<i64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AccountRow {
    pub id: Uuid,
    pub uid: String,
    pub server_id: i16,
    pub server: String,
    pub nickname: Option<String>,
    pub level: Option<i16>,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct KeyRow {
    pub id: Uuid,
    pub uid: String,
}

#[derive(Debug, Clone)]
pub struct SiteCountsRow {
    pub total: i64,
    pub en: i64,
    pub jp: i64,
    pub kr: i64,
    pub cn: i64,
    pub bili: i64,
    pub tw: i64,
    pub signups_7d: i64,
    pub signups_30d: i64,
    pub public_profiles: i64,
}

#[derive(Debug, Clone)]
pub struct RoleCountsRow {
    pub user: i64,
    pub tier_list_editor: i64,
    pub tier_list_admin: i64,
    pub translator: i64,
    pub super_admin: i64,
}

#[derive(Debug, Clone)]
pub struct AdminUserRow {
    pub id: Uuid,
    pub uid: String,
    pub server: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub role: String,
    pub avatar_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub public_profile: bool,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
    pub operator_count: i64,
    pub item_count: i64,
    pub skin_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AdminFilter<'a> {
    pub q: Option<&'a str>,
    pub roles: Option<&'a [String]>,
    pub server: Option<Server>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SettingsUpdate<'a> {
    pub public_profile: Option<bool>,
    pub store_gacha: Option<bool>,
    pub share_stats: Option<bool>,
    pub public_gacha: Option<bool>,
    pub layout: LayoutUpdate<'a>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum LayoutUpdate<'a> {
    #[default]
    Keep,
    Clear,
    Merge {
        patch: &'a Value,
        removed: &'a [String],
    },
}

#[derive(Debug, Clone)]
pub struct DeletedAccount {
    pub uid: String,
    pub tier_list_slugs: Vec<String>,
}

pub async fn create(conn: &mut Conn, uid: &str, server: Server) -> Result<UserRow, DbError> {
    Ok(sqlx::query_as!(
        UserRow,
        "INSERT INTO users (uid, server_id) VALUES ($1, $2)
         RETURNING id, uid, server_id, nickname, nick_number, level, role, avatar_id,
                   resume_id, secretary, secretary_skin_id, created_at, updated_at",
        uid,
        server.index(),
    )
    .fetch_one(conn)
    .await?)
}

pub async fn find_or_create(
    conn: &mut Conn,
    uid: &str,
    server: Server,
) -> Result<UserRow, DbError> {
    let inserted = sqlx::query_as!(
        UserRow,
        "INSERT INTO users (uid, server_id) VALUES ($1, $2)
         ON CONFLICT (uid, server_id) DO NOTHING
         RETURNING id, uid, server_id, nickname, nick_number, level, role, avatar_id,
                   resume_id, secretary, secretary_skin_id, created_at, updated_at",
        uid,
        server.index(),
    )
    .fetch_optional(&mut *conn)
    .await?;
    match inserted {
        Some(row) => Ok(row),
        None => find(conn, uid, server).await?.ok_or(DbError::NotFound), // a new snapshot sees a concurrent insert
    }
}

pub async fn find(conn: &mut Conn, uid: &str, server: Server) -> Result<Option<UserRow>, DbError> {
    Ok(sqlx::query_as!(
        UserRow,
        "SELECT id, uid, server_id, nickname, nick_number, level, role, avatar_id,
                resume_id, secretary, secretary_skin_id, created_at, updated_at
         FROM users WHERE uid = $1 AND server_id = $2",
        uid,
        server.index(),
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn exists(conn: &mut Conn, user: UserId) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM users WHERE id = $1) AS "exists!""#,
        user.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn lock(conn: &mut Conn, user: UserId) -> Result<bool, DbError> {
    let row = sqlx::query_scalar!("SELECT id FROM users WHERE id = $1 FOR UPDATE", user.0)
        .fetch_optional(conn)
        .await?;
    Ok(row.is_some())
}

pub async fn profile(conn: &mut Conn, user: UserId) -> Result<Option<ProfileRow>, DbError> {
    Ok(sqlx::query_as!(
        ProfileRow,
        r#"SELECT id AS "id!", uid AS "uid!", nickname, nick_number, level, avatar_id, secretary,
                  secretary_skin_id, resume_id, role AS "role!", server AS "server!", total_score,
                  grade, public_profile, store_gacha, share_stats, exp, orundum, lmd, sanity,
                  max_sanity, gacha_tickets, ten_pull_tickets, monthly_sub_end, register_ts,
                  last_online_ts, resume, friend_num_limit, cumulative_signin, operator_count,
                  item_count, skin_count, non_default_skin_count, updated_at AS "updated_at!",
                  originite, profile_layout
           FROM v_user_profile WHERE id = $1"#,
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn profile_by_uid(
    conn: &mut Conn,
    uid: &str,
    server: Option<Server>,
) -> Result<Option<ProfileRow>, DbError> {
    Ok(sqlx::query_as!(
        ProfileRow,
        r#"SELECT id AS "id!", uid AS "uid!", nickname, nick_number, level, avatar_id, secretary,
                  secretary_skin_id, resume_id, role AS "role!", server AS "server!", total_score,
                  grade, public_profile, store_gacha, share_stats, exp, orundum, lmd, sanity,
                  max_sanity, gacha_tickets, ten_pull_tickets, monthly_sub_end, register_ts,
                  last_online_ts, resume, friend_num_limit, cumulative_signin, operator_count,
                  item_count, skin_count, non_default_skin_count, updated_at AS "updated_at!",
                  originite, profile_layout
           FROM v_user_profile
           WHERE id = (SELECT id FROM users
                       WHERE uid = $1 AND ($2::smallint IS NULL OR server_id = $2)
                       ORDER BY created_at, id LIMIT 1)"#,
        uid,
        server.map(Server::index),
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn checkin_by_uid(
    conn: &mut Conn,
    uid: &str,
    server: Option<Server>,
) -> Result<Option<CheckinRow>, DbError> {
    Ok(sqlx::query_as!(
        CheckinRow,
        r#"SELECT ck.history, cardinality(ck.history) AS "claimed_this_month!",
                  ck.cumulative_signin, ck.checkin_group_id, ck.reward_index, ck.can_check_in,
                  st.register_ts AS "register_ts?", st.last_online_ts AS "last_online_ts?",
                  u.updated_at
           FROM user_checkin ck
           JOIN users u ON u.id = ck.user_id
           LEFT JOIN user_status st ON st.user_id = ck.user_id
           WHERE u.id = (SELECT id FROM users
                         WHERE uid = $1 AND ($2::smallint IS NULL OR server_id = $2)
                         ORDER BY created_at, id LIMIT 1)"#,
        uid,
        server.map(Server::index),
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn role(conn: &mut Conn, user: UserId) -> Result<Option<String>, DbError> {
    Ok(
        sqlx::query_scalar!("SELECT role FROM users WHERE id = $1", user.0)
            .fetch_optional(conn)
            .await?,
    )
}

pub async fn set_role(conn: &mut Conn, user: UserId, role: &str) -> Result<(), DbError> {
    sqlx::query!("UPDATE users SET role = $2 WHERE id = $1", user.0, role)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn touch(conn: &mut Conn, user: UserId) -> Result<(), DbError> {
    sqlx::query!("UPDATE users SET updated_at = NOW() WHERE id = $1", user.0)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn update_settings(
    conn: &mut Conn,
    user: UserId,
    update: &SettingsUpdate<'_>,
) -> Result<(), DbError> {
    let (touch_layout, patch, removed): (bool, Option<&Value>, &[String]) = match update.layout {
        LayoutUpdate::Keep => (false, None, &[]),
        LayoutUpdate::Clear => (true, None, &[]),
        LayoutUpdate::Merge { patch, removed } => (true, Some(patch), removed),
    };
    let updated = sqlx::query!(
        "UPDATE user_settings SET public_profile = COALESCE($2, public_profile),
             store_gacha = COALESCE($3, store_gacha), share_stats = COALESCE($4, share_stats),
             public_gacha = COALESCE($8, public_gacha),
             profile_layout = CASE WHEN NOT $5 THEN profile_layout
                 WHEN $6::jsonb IS NULL THEN NULL
                 ELSE NULLIF((COALESCE(profile_layout, '{}'::jsonb) || $6::jsonb) - $7::text[],
                             '{}'::jsonb) END
         WHERE user_id = $1",
        user.0,
        update.public_profile,
        update.store_gacha,
        update.share_stats,
        touch_layout,
        patch,
        removed,
        update.public_gacha,
    )
    .execute(conn)
    .await?
    .rows_affected();
    if updated == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

pub async fn base_facts(conn: &mut Conn, user: UserId) -> Result<Option<Value>, DbError> {
    let row = sqlx::query_scalar!(
        "SELECT base_facts FROM user_settings WHERE user_id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?;
    Ok(row.flatten())
}

pub async fn set_base_facts(conn: &mut Conn, user: UserId, facts: &Value) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO user_settings (user_id, base_facts) VALUES ($1, $2)
         ON CONFLICT (user_id) DO UPDATE SET base_facts = EXCLUDED.base_facts",
        user.0,
        facts,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn account(conn: &mut Conn, user: UserId) -> Result<Option<AccountRow>, DbError> {
    Ok(sqlx::query_as!(
        AccountRow,
        "SELECT u.id, u.uid, u.server_id, s.code AS server, u.nickname, u.level, u.role,
                u.created_at
         FROM users u JOIN servers s ON s.id = u.server_id WHERE u.id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn accounts_by_uid(conn: &mut Conn, uid: &str) -> Result<Vec<AccountRow>, DbError> {
    Ok(sqlx::query_as!(
        AccountRow,
        "SELECT u.id, u.uid, u.server_id, s.code AS server, u.nickname, u.level, u.role,
                u.created_at
         FROM users u JOIN servers s ON s.id = u.server_id
         WHERE u.uid = $1 ORDER BY u.created_at, u.id",
        uid,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn recent(conn: &mut Conn, limit: i64) -> Result<Vec<AccountRow>, DbError> {
    Ok(sqlx::query_as!(
        AccountRow,
        "SELECT u.id, u.uid, u.server_id, s.code AS server, u.nickname, u.level, u.role,
                u.created_at
         FROM users u JOIN servers s ON s.id = u.server_id
         ORDER BY u.created_at DESC LIMIT $1",
        limit,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn search_accounts(conn: &mut Conn, needle: &str) -> Result<Vec<AccountRow>, DbError> {
    Ok(sqlx::query_as!(
        AccountRow,
        r"SELECT u.id, u.uid, u.server_id, s.code AS server, u.nickname, u.level, u.role,
                u.created_at
         FROM users u JOIN servers s ON s.id = u.server_id
         WHERE u.uid ILIKE $1 ESCAPE '\' OR u.nickname ILIKE $1 ESCAPE '\'
         ORDER BY u.created_at DESC LIMIT 20",
        contains_pattern(needle),
    )
    .fetch_all(conn)
    .await?)
}

pub async fn recent_with_base(conn: &mut Conn, limit: i64) -> Result<Vec<AccountRow>, DbError> {
    Ok(sqlx::query_as!(
        AccountRow,
        "SELECT u.id, u.uid, u.server_id, s.code AS server, u.nickname, u.level, u.role,
                u.created_at
         FROM users u JOIN servers s ON s.id = u.server_id
         JOIN user_building b ON b.user_id = u.id
         ORDER BY u.updated_at DESC LIMIT $1",
        limit,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn count(conn: &mut Conn, server: Option<Server>) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM users WHERE ($1::smallint IS NULL OR server_id = $1)"#,
        server.map(Server::index),
    )
    .fetch_one(conn)
    .await?)
}

pub async fn count_by_uid(
    conn: &mut Conn,
    uid: &str,
    server: Option<Server>,
) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM users
           WHERE uid = $1 AND ($2::smallint IS NULL OR server_id = $2)"#,
        uid,
        server.map(Server::index),
    )
    .fetch_one(conn)
    .await?)
}

pub async fn keys_page(
    conn: &mut Conn,
    server: Option<Server>,
    after: Option<UserId>,
    limit: i64,
) -> Result<Vec<KeyRow>, DbError> {
    let rows = match (server, after) {
        (None, None) => {
            sqlx::query_as!(
                KeyRow,
                "SELECT id, uid FROM users ORDER BY id LIMIT $1",
                limit
            )
            .fetch_all(conn)
            .await?
        }
        (None, Some(after)) => {
            sqlx::query_as!(
                KeyRow,
                "SELECT id, uid FROM users WHERE id > $1 ORDER BY id LIMIT $2",
                after.0,
                limit,
            )
            .fetch_all(conn)
            .await?
        }
        (Some(server), None) => {
            sqlx::query_as!(
                KeyRow,
                "SELECT id, uid FROM users WHERE server_id = $1 ORDER BY id LIMIT $2",
                server.index(),
                limit,
            )
            .fetch_all(conn)
            .await?
        }
        (Some(server), Some(after)) => {
            sqlx::query_as!(
                KeyRow,
                "SELECT id, uid FROM users WHERE server_id = $1 AND id > $2
                 ORDER BY id LIMIT $3",
                server.index(),
                after.0,
                limit,
            )
            .fetch_all(conn)
            .await?
        }
    };
    Ok(rows)
}

pub async fn keys_by_uid(
    conn: &mut Conn,
    uid: &str,
    server: Option<Server>,
) -> Result<Vec<KeyRow>, DbError> {
    Ok(sqlx::query_as!(
        KeyRow,
        "SELECT id, uid FROM users
         WHERE uid = $1 AND ($2::smallint IS NULL OR server_id = $2) ORDER BY id",
        uid,
        server.map(Server::index),
    )
    .fetch_all(conn)
    .await?)
}

pub async fn site_counts(conn: &mut Conn) -> Result<SiteCountsRow, DbError> {
    Ok(sqlx::query_as!(
        SiteCountsRow,
        r#"SELECT
             count(*) AS "total!",
             count(*) FILTER (WHERE server_id = 0) AS "en!",
             count(*) FILTER (WHERE server_id = 1) AS "jp!",
             count(*) FILTER (WHERE server_id = 2) AS "kr!",
             count(*) FILTER (WHERE server_id = 3) AS "cn!",
             count(*) FILTER (WHERE server_id = 4) AS "bili!",
             count(*) FILTER (WHERE server_id = 5) AS "tw!",
             count(*) FILTER (WHERE created_at > NOW() - INTERVAL '7 days') AS "signups_7d!",
             count(*) FILTER (WHERE created_at > NOW() - INTERVAL '30 days') AS "signups_30d!",
             (SELECT count(*) FROM user_settings WHERE public_profile = true) AS "public_profiles!"
           FROM users"#,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn role_counts(conn: &mut Conn) -> Result<RoleCountsRow, DbError> {
    Ok(sqlx::query_as!(
        RoleCountsRow,
        r#"SELECT
             count(*) FILTER (WHERE role = 'user') AS "user!",
             count(*) FILTER (WHERE role = 'tier_list_editor') AS "tier_list_editor!",
             count(*) FILTER (WHERE role = 'tier_list_admin') AS "tier_list_admin!",
             count(*) FILTER (WHERE role = 'translator') AS "translator!",
             count(*) FILTER (WHERE role = 'super_admin') AS "super_admin!"
           FROM users"#,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn admin_page(
    conn: &mut Conn,
    filter: &AdminFilter<'_>,
    limit: i64,
    offset: i64,
) -> Result<Vec<AdminUserRow>, DbError> {
    let (name, uid_prefix) = admin_patterns(filter.q);
    Ok(sqlx::query_as!(
        AdminUserRow,
        r#"SELECT p.id AS "id!", p.uid AS "uid!", s.code AS "server!", p.nickname, p.nick_number,
                  p.level, p.role AS "role!", p.avatar_id, p.secretary, p.secretary_skin_id,
                  COALESCE(us.public_profile, false) AS "public_profile!",
                  sc.total_score AS "total_score?", sc.grade,
                  (SELECT count(*) FROM user_operators uo WHERE uo.user_id = p.id) AS "operator_count!",
                  (SELECT count(*) FROM user_items ui WHERE ui.user_id = p.id) AS "item_count!",
                  (SELECT count(*) FROM user_skins sk WHERE sk.user_id = p.id) AS "skin_count!",
                  p.created_at AS "created_at!", p.updated_at AS "updated_at!"
           FROM (
               SELECT u.id, u.uid, u.server_id, u.nickname, u.nick_number, u.level, u.role,
                      u.avatar_id, u.secretary, u.secretary_skin_id, u.created_at, u.updated_at,
                      COALESCE(u.uid = $1::text, false) AS exact_hit,
                      CASE u.role WHEN 'super_admin' THEN 0 WHEN 'tier_list_admin' THEN 1
                          WHEN 'tier_list_editor' THEN 2 WHEN 'translator' THEN 3 ELSE 4 END
                          AS role_rank
               FROM users u
               WHERE ($1::text IS NULL OR u.nickname ILIKE $2 ESCAPE '\' OR u.uid LIKE $3 ESCAPE '\')
                 AND ($4::text[] IS NULL OR u.role = ANY($4))
                 AND ($5::smallint IS NULL OR u.server_id = $5)
               ORDER BY exact_hit DESC, role_rank, lower(u.nickname) NULLS LAST, u.id
               LIMIT $6 OFFSET $7
           ) p
           JOIN servers s ON s.id = p.server_id
           LEFT JOIN user_settings us ON us.user_id = p.id
           LEFT JOIN user_scores sc ON sc.user_id = p.id
           ORDER BY p.exact_hit DESC, p.role_rank, lower(p.nickname) NULLS LAST, p.id"#,
        filter.q,
        name,
        uid_prefix,
        filter.roles,
        filter.server.map(Server::index),
        limit,
        offset,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn admin_count(conn: &mut Conn, filter: &AdminFilter<'_>) -> Result<i64, DbError> {
    let (name, uid_prefix) = admin_patterns(filter.q);
    Ok(sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM users u
           WHERE ($1::text IS NULL OR u.nickname ILIKE $2 ESCAPE '\' OR u.uid LIKE $3 ESCAPE '\')
             AND ($4::text[] IS NULL OR u.role = ANY($4))
             AND ($5::smallint IS NULL OR u.server_id = $5)"#,
        filter.q,
        name,
        uid_prefix,
        filter.roles,
        filter.server.map(Server::index),
    )
    .fetch_one(conn)
    .await?)
}

fn admin_patterns(q: Option<&str>) -> (Option<String>, Option<String>) {
    q.map_or((None, None), |q| {
        (
            Some(contains_pattern(q)),
            Some(format!("{}%", escape_like(q))),
        )
    })
}

pub async fn delete_account(
    conn: &mut Conn,
    user: UserId,
) -> Result<Option<DeletedAccount>, DbError> {
    let Some(uid) = sqlx::query_scalar!("SELECT uid FROM users WHERE id = $1 FOR UPDATE", user.0)
        .fetch_optional(&mut *conn)
        .await?
    else {
        return Ok(None);
    };

    let lists = sqlx::query!(
        r#"WITH gone AS (
               DELETE FROM tier_lists WHERE created_by = $1 AND list_type = 'community'
               RETURNING id, slug
           )
           SELECT g.id, g.slug,
                  ARRAY(SELECT t.id FROM tiers t WHERE t.tier_list_id = g.id) AS "tier_ids!"
           FROM gone g"#,
        user.0,
    )
    .fetch_all(&mut *conn)
    .await?;
    let list_ids: Vec<Uuid> = lists.iter().map(|l| l.id).collect();
    let tier_ids: Vec<Uuid> = lists
        .iter()
        .flat_map(|l| l.tier_ids.iter().copied())
        .collect();
    sqlx::query!(
        "UPDATE tier_list_permissions SET granted_by = NULL WHERE granted_by = $1", // no ON DELETE action, it would block the delete
        user.0,
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        "DELETE FROM operator_notes_audit_log WHERE changed_by = $1",
        user.0,
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        "UPDATE audit_log SET changed_by = NULL WHERE changed_by = $1",
        user.0,
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        "UPDATE release_overrides SET updated_by = NULL WHERE updated_by = $1",
        user.0,
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query!("DELETE FROM users WHERE id = $1", user.0)
        .execute(&mut *conn)
        .await?;

    sqlx::query!(
        "DELETE FROM audit_log
         WHERE (table_name IN ('users', 'user_scores') AND record_id = $1::uuid::text)
            OR (table_name = 'tier_lists' AND record_id = ANY($2::uuid[]::text[]))
            OR (table_name = 'tier_placements'
                AND COALESCE(new_data, old_data)->>'tier_id' = ANY($3::uuid[]::text[]))",
        user.0,
        &list_ids,
        &tier_ids,
    )
    .execute(&mut *conn)
    .await?;

    Ok(Some(DeletedAccount {
        uid,
        tier_list_slugs: lists.into_iter().map(|l| l.slug).collect(),
    }))
}
