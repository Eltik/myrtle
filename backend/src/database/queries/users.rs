use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;

use crate::database::models::profile_layout::ProfileLayoutPatch;
use crate::database::models::user::{User, UserCheckin, UserProfile};

pub async fn create_user(pool: &PgPool, uid: &str, server_id: i16) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>("INSERT INTO users (uid, server_id) VALUES ($1, $2) RETURNING *")
        .bind(uid)
        .bind(server_id)
        .fetch_one(pool)
        .await
}

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
        SELECT ck.history, cardinality(ck.history) AS claimed_this_month,
               ck.cumulative_signin, ck.checkin_group_id,
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

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserProfile>, sqlx::Error> {
    sqlx::query_as::<_, UserProfile>("SELECT * FROM v_user_profile WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

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

/// A `None` flag keeps the stored one. `profile_layout`: `None` keeps the
/// stored layout, `Some(None)` clears it to NULL, `Some(Some(patch))` merges
/// the patch over it key by key: each key the save carried replaces the
/// stored one, each it left out survives, a cleared background is removed,
/// and a layout left with no key is NULL again. The tab editor sends only
/// `tabs`, the showcase editor only `showcase` (an empty one as
/// `{"blocks": []}`) and the background picker only `background`, so none
/// wipes another's work; `Some(None)` comes only from the tab editor's
/// explicit Reset. [`ProfileLayoutPatch::apply`] is the same rule in Rust,
/// for the unit tests.
pub async fn update_settings(
    pool: &PgPool,
    user_id: Uuid,
    public_profile: Option<bool>,
    store_gacha: Option<bool>,
    share_stats: Option<bool>,
    profile_layout: Option<Option<ProfileLayoutPatch>>,
) -> Result<(), sqlx::Error> {
    let touch_layout = profile_layout.is_some();
    let patch = profile_layout.flatten();
    let removed: Vec<&str> = patch
        .as_ref()
        .map(ProfileLayoutPatch::removed_keys)
        .unwrap_or_default();
    sqlx::query(
        "UPDATE user_settings SET public_profile = COALESCE($2, public_profile), \
         store_gacha = COALESCE($3, store_gacha), share_stats = COALESCE($4, share_stats), \
         profile_layout = CASE WHEN NOT $5 THEN profile_layout \
             WHEN $6::jsonb IS NULL THEN NULL \
             ELSE NULLIF((COALESCE(profile_layout, '{}'::jsonb) || $6::jsonb) - $7::text[], \
                         '{}'::jsonb) END \
         WHERE user_id = $1",
    )
    .bind(user_id)
    .bind(public_profile)
    .bind(store_gacha)
    .bind(share_stats)
    .bind(touch_layout)
    .bind(patch.as_ref().map(|p| Json(p.merge_json())))
    .bind(removed)
    .execute(pool)
    .await?;
    Ok(())
}

/// The user's saved base "account facts" (player state the sync cannot read).
pub async fn get_base_facts(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<serde_json::Value>>(
        "SELECT base_facts FROM user_settings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(Option::flatten)
}

/// Save the user's base "account facts".
pub async fn set_base_facts(
    pool: &PgPool,
    user_id: Uuid,
    facts: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_settings (user_id, base_facts) VALUES ($1, $2)
         ON CONFLICT (user_id) DO UPDATE SET base_facts = EXCLUDED.base_facts",
    )
    .bind(user_id)
    .bind(facts)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_role(pool: &PgPool, user_id: Uuid, role: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET role = $2 WHERE id = $1")
        .bind(user_id)
        .bind(role)
        .execute(pool)
        .await?;
    Ok(())
}
