use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::user::{User, UserCheckin, UserProfile};

/// Create minimal user
pub async fn create_user(pool: &PgPool, uid: &str, server_id: i16) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>("INSERT INTO users (uid, server_id) VALUES ($1, $2) RETURNING *")
        .bind(uid)
        .bind(server_id)
        .fetch_one(pool)
        .await
}

/// Find user profile by Arknights UID
pub async fn find_by_uid(pool: &PgPool, uid: &str) -> Result<Option<UserProfile>, sqlx::Error> {
    sqlx::query_as::<_, UserProfile>("SELECT * FROM v_user_profile WHERE uid = $1")
        .bind(uid)
        .fetch_optional(pool)
        .await
}

/// Fetch the daily sign-in row for one user by Arknights UID.
/// Returns `None` if the user has never been synced.
pub async fn get_checkin_by_uid(
    pool: &PgPool,
    uid: &str,
) -> Result<Option<UserCheckin>, sqlx::Error> {
    sqlx::query_as::<_, UserCheckin>(
        r"
        SELECT ck.history, ck.cumulative_signin, ck.checkin_group_id,
               ck.reward_index, ck.can_check_in,
               st.register_ts, st.last_online_ts, u.updated_at
        FROM user_checkin ck
        JOIN users u ON u.id = ck.user_id
        LEFT JOIN user_status st ON st.user_id = ck.user_id
        WHERE u.uid = $1
        ",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await
}

/// Find user profile by internal UUID
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserProfile>, sqlx::Error> {
    sqlx::query_as::<_, UserProfile>("SELECT * FROM v_user_profile WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Search public user profiles by nickname, ranked by total score.
/// When `query` is `None`, returns all public profiles ranked by score.
pub async fn search_by_nickname(
    pool: &PgPool,
    query: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<UserProfile>, sqlx::Error> {
    let pattern = query.map(|q| format!("%{q}%"));
    sqlx::query_as::<_, UserProfile>(
        "SELECT * FROM v_user_profile \
         WHERE public_profile = true \
           AND ($1::text IS NULL OR nickname ILIKE $1) \
         ORDER BY total_score DESC NULLS LAST \
         LIMIT $2 OFFSET $3",
    )
    .bind(pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Count public user profiles matching a nickname search.
/// When `query` is `None`, counts all public profiles.
pub async fn count_by_nickname(pool: &PgPool, query: Option<&str>) -> Result<i64, sqlx::Error> {
    let pattern = query.map(|q| format!("%{q}%"));
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM v_user_profile \
         WHERE public_profile = true \
           AND ($1::text IS NULL OR nickname ILIKE $1)",
    )
    .bind(pattern)
    .fetch_one(pool)
    .await
}

/// Find raw user by UID + server
pub async fn find_raw_by_uid(
    pool: &PgPool,
    uid: &str,
    server_id: i16,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE uid = $1 AND server_id = $2")
        .bind(uid)
        .bind(server_id)
        .fetch_optional(pool)
        .await
}

/// Update user settings
pub async fn update_settings(
    pool: &PgPool,
    user_id: Uuid,
    public_profile: bool,
    store_gacha: bool,
    share_stats: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE user_settings SET public_profile = $2, store_gacha = $3, share_stats = $4 WHERE user_id = $1"
    )
    .bind(user_id)
    .bind(public_profile)
    .bind(store_gacha)
    .bind(share_stats)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update user role
pub async fn update_role(pool: &PgPool, user_id: Uuid, role: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET role = $2 WHERE id = $1")
        .bind(user_id)
        .bind(role)
        .execute(pool)
        .await?;
    Ok(())
}
