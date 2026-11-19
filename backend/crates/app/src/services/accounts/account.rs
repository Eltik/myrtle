use std::time::Duration;

use chrono::{DateTime, Utc};
use myrtle_db::repos::users::{self, ProfileRow};
use myrtle_db::repos::{gacha, i18n};
use myrtle_types::{Role, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::jwt::{Claims, parse_role, role_name};
use super::sessions::{portal_session_key, session_key};
use crate::privacy::layout::{ProfileHeaderField, ProfileLayout, ProfileTabId};
use crate::privacy::{self, mask_identifier};
use crate::services::community::showcase;
use crate::{AppContext, AppError, Subject};

const USER_TTL: Duration = Duration::from_mins(10);

#[must_use]
pub fn user_key(uid: &str) -> String {
    format!("user:{uid}")
}

/// A `v_user_profile` row as the owner sees it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
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
    pub profile_layout: Option<ProfileLayout>,
    /// Not in the view: the caller fills it from a fresh read
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_gacha: Option<bool>,
}

impl From<ProfileRow> for UserProfile {
    fn from(row: ProfileRow) -> Self {
        Self {
            id: row.id,
            uid: row.uid,
            nickname: row.nickname,
            nick_number: row.nick_number,
            level: row.level,
            avatar_id: row.avatar_id,
            secretary: row.secretary,
            secretary_skin_id: row.secretary_skin_id,
            resume_id: row.resume_id,
            role: row.role,
            server: row.server,
            total_score: row.total_score,
            grade: row.grade,
            public_profile: row.public_profile,
            store_gacha: row.store_gacha,
            share_stats: row.share_stats,
            exp: row.exp,
            orundum: row.orundum,
            lmd: row.lmd,
            sanity: row.sanity,
            max_sanity: row.max_sanity,
            gacha_tickets: row.gacha_tickets,
            ten_pull_tickets: row.ten_pull_tickets,
            monthly_sub_end: row.monthly_sub_end,
            register_ts: row.register_ts,
            last_online_ts: row.last_online_ts,
            resume: row.resume,
            friend_num_limit: row.friend_num_limit,
            cumulative_signin: row.cumulative_signin,
            operator_count: row.operator_count,
            item_count: row.item_count,
            skin_count: row.skin_count,
            non_default_skin_count: row.non_default_skin_count,
            updated_at: row.updated_at,
            originite: row.originite,
            profile_layout: row.profile_layout.as_ref().map(ProfileLayout::normalize),
            public_gacha: None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifySession {
    /// Always true: a failing token is a 401
    pub valid: bool,
    pub user_id: String,
    pub uid: String,
    pub server: String,
    /// The stored role, so a promotion counts without a new login
    pub role: String,
    pub can_access_admin_panel: bool,
}

impl UserProfile {
    /// What a visitor sees: Score tab numbers and the hidden header numbers nulled, private tabs gone
    pub fn hide_private(&mut self) {
        let Some(layout) = &mut self.profile_layout else {
            return;
        };
        if !layout.is_visible(ProfileTabId::Score) {
            self.total_score = None;
            self.grade = None;
        }
        for field in ProfileHeaderField::ALL {
            if layout.header_shows(field) {
                continue;
            }
            match field {
                ProfileHeaderField::Level => {
                    self.level = None;
                    self.exp = None;
                }
                ProfileHeaderField::Registered => self.register_ts = None,
                ProfileHeaderField::SignIn => self.cumulative_signin = None,
                ProfileHeaderField::Operators => self.operator_count = None,
                ProfileHeaderField::Skins => {
                    self.skin_count = None;
                    self.non_default_skin_count = None;
                }
                ProfileHeaderField::Items => self.item_count = None,
                ProfileHeaderField::Orundum => self.orundum = None,
                ProfileHeaderField::Originite => self.originite = None,
                ProfileHeaderField::Lmd => self.lmd = None,
                ProfileHeaderField::HeadhuntingPermits => self.gacha_tickets = None,
                ProfileHeaderField::TenPullPermits => self.ten_pull_tickets = None,
            }
        }
        *layout = layout.visible_only();
    }
}

/// The profile behind `uid` as the subject may see it; a visitor gets [`UserProfile::hide_private`]
pub async fn get_user(
    ctx: &AppContext,
    subject: &Subject,
    uid: &str,
) -> Result<UserProfile, AppError> {
    let (gate, own) = privacy::resolve_public_profile(ctx, subject, uid).await?;
    let owner = UserId(gate.id);
    let mut profile = cached_profile(ctx, &gate.uid).await?;
    profile.profile_layout = gate.profile_layout.as_ref().map(ProfileLayout::normalize); // the cached row may predate a tab going private
    profile.public_gacha =
        Some(gacha::public_history_enabled(&mut *ctx.db.conn().await?, owner).await?);
    if !own {
        profile.hide_private();
        if let Some(layout) = profile.profile_layout.as_mut() {
            showcase::prune_for_visitor(ctx, owner, layout).await?;
        }
    }
    Ok(profile)
}

/// The owner's row, cached for [`USER_TTL`]
async fn cached_profile(ctx: &AppContext, uid: &str) -> Result<UserProfile, AppError> {
    let key = user_key(uid);
    if let Some(cached) = ctx.cache.get_json(&key).await {
        return Ok(cached);
    }
    let profile: UserProfile = users::profile_by_uid(&mut *ctx.db.conn().await?, uid, None)
        .await?
        .ok_or(AppError::NotFound(None))?
        .into();
    ctx.cache.set_json(&key, &profile, USER_TTL).await;
    Ok(profile)
}

/// The session a verified token stands for, its role re-read; 401 once the account is gone
pub async fn verify(ctx: &AppContext, claims: &Claims) -> Result<VerifySession, AppError> {
    let mut conn = ctx.db.conn().await?;
    let id = claims.user_id().ok();
    let stored = match id {
        Some(id) => users::role(&mut conn, id).await?,
        None => None,
    };
    let role = match stored {
        Some(role) => role,
        None if ctx.policy.role_recheck && id.is_some() => return Err(AppError::Unauthorized),
        None => role_name(parse_role(&claims.role)).to_owned(),
    };
    let can_access_admin_panel = parse_role(&role) != Role::User
        || match id {
            Some(id) => i18n::has_any_permission(&mut conn, id).await?,
            None => false,
        };
    Ok(VerifySession {
        valid: true,
        user_id: claims.sub.clone(),
        uid: claims.uid.clone(),
        server: claims.server.clone(),
        role,
        can_access_admin_panel,
    })
}

/// Deletes the caller's account and its rows in one transaction; 401 when it is already gone
pub async fn delete_account(ctx: &AppContext, subject: &Subject) -> Result<(), AppError> {
    let user = subject.user_id().ok_or(AppError::Unauthorized)?;
    let mut tx = ctx.db.begin().await?;
    let deleted = users::delete_account(&mut tx, user)
        .await?
        .ok_or(AppError::Unauthorized)?;
    tx.commit().await?;

    let uid = deleted.uid.as_str(); // after the commit, so a racing read cannot cache the old rows back
    let cache = &ctx.cache;
    cache.invalidate(&user_key(uid)).await;
    cache.invalidate(&session_key(uid)).await;
    cache.invalidate(&portal_session_key(uid)).await;
    for slug in &deleted.tier_list_slugs {
        cache.invalidate(&format!("tierlist:{slug}")).await;
    }
    for prefix in ["leaderboard:", "search:"] {
        cache.invalidate_prefix(prefix).await;
    }
    for scope in ["improvements", "base:rotation", "base:optimize"] {
        cache.invalidate_prefix(&format!("{scope}:{uid}:")).await;
    }
    tracing::info!(
        uid = %mask_identifier(uid),
        tier_lists = deleted.tier_list_slugs.len(),
        "account deleted"
    );
    Ok(())
}
