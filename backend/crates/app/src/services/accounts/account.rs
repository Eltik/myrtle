use std::time::Duration;

use chrono::{DateTime, Utc};
use myrtle_db::repos::scores::{self, ScoreRow};
use myrtle_db::repos::users::{self, AdminFilter, AdminUserRow, CheckinRow, ProfileRow};
use myrtle_db::repos::{gacha, i18n};
use myrtle_types::{PageRequest, Role, Server, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::jwt::{Claims, parse_role, role_name};
use super::sessions::{portal_session_key, session_key};
use crate::privacy::layout::{ProfileHeaderField, ProfileLayout, ProfileTabId};
use crate::privacy::{self, mask_identifier};
use crate::services::community::showcase;
use crate::{Action, AppContext, AppError, Resource, Subject};

const USER_TTL: Duration = Duration::from_mins(10);
const ADMIN_USERS_DEFAULT_LIMIT: u32 = 50;
const ADMIN_USERS_MAX_LIMIT: u32 = 200;

#[must_use]
pub fn user_key(uid: &str) -> String {
    format!("user:{uid}")
}

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
    pub valid: bool,
    pub user_id: String,
    pub uid: String,
    pub server: String,
    pub role: String,
    pub can_access_admin_panel: bool,
}

impl UserProfile {
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

#[derive(Debug, Clone, Serialize)]
pub struct UserScore {
    pub user_id: Uuid,
    pub total_score: f64,
    pub operator_score: f64,
    pub stage_score: f64,
    pub roguelike_score: f64,
    pub sandbox_score: f64,
    pub medal_score: f64,
    pub base_score: f64,
    pub base_utilization: Option<f64>,
    pub base_infrastructure: Option<f64>,
    pub skin_score: f64,
    pub grade: Option<String>,
    pub calculated_at: DateTime<Utc>,
}

impl From<ScoreRow> for UserScore {
    fn from(row: ScoreRow) -> Self {
        Self {
            user_id: row.user_id,
            total_score: row.total_score,
            operator_score: row.operator_score,
            stage_score: row.stage_score,
            roguelike_score: row.roguelike_score,
            sandbox_score: row.sandbox_score,
            medal_score: row.medal_score,
            base_score: row.base_score,
            base_utilization: row.base_utilization,
            base_infrastructure: row.base_infrastructure,
            skin_score: row.skin_score,
            grade: row.grade,
            calculated_at: row.calculated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UserCheckin {
    pub monthly_card_flags: Vec<i16>,
    pub claimed_this_month: i32,
    pub cumulative_signin: Option<i32>,
    pub checkin_group_id: Option<String>,
    pub reward_index: i16,
    pub can_check_in: bool,
    pub register_ts: Option<i64>,
    pub last_online_ts: Option<i64>,
    pub updated_at: DateTime<Utc>,
}

impl From<CheckinRow> for UserCheckin {
    fn from(row: CheckinRow) -> Self {
        Self {
            monthly_card_flags: row.history,
            claimed_this_month: row.claimed_this_month,
            cumulative_signin: Some(row.cumulative_signin),
            checkin_group_id: row.checkin_group_id,
            reward_index: row.reward_index,
            can_check_in: row.can_check_in,
            register_ts: row.register_ts,
            last_online_ts: row.last_online_ts,
            updated_at: row.updated_at,
        }
    }
}

impl UserCheckin {
    pub fn hide_private(&mut self, layout: &ProfileLayout) {
        if !layout.header_shows(ProfileHeaderField::SignIn) {
            self.cumulative_signin = None;
        }
        if !layout.header_shows(ProfileHeaderField::Registered) {
            self.register_ts = None;
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetRoleRequest {
    pub role: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct AdminUsersQuery {
    pub q: Option<String>,
    pub role: Option<String>,
    pub server: Option<Server>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminUserEntry {
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

impl From<AdminUserRow> for AdminUserEntry {
    fn from(row: AdminUserRow) -> Self {
        Self {
            id: row.id,
            uid: row.uid,
            server: row.server,
            nickname: row.nickname,
            nick_number: row.nick_number,
            level: row.level,
            role: row.role,
            avatar_id: row.avatar_id,
            secretary: row.secretary,
            secretary_skin_id: row.secretary_skin_id,
            public_profile: row.public_profile,
            total_score: row.total_score,
            grade: row.grade,
            operator_count: row.operator_count,
            item_count: row.item_count,
            skin_count: row.skin_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminUsersPage {
    pub users: Vec<AdminUserEntry>,
    pub total: i64,
}

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

pub async fn get_user_score(
    ctx: &AppContext,
    subject: &Subject,
    uid: &str,
) -> Result<Option<UserScore>, AppError> {
    let uid = privacy::resolve_uid(ctx, subject, Some(uid), ProfileTabId::Score).await?;
    let row = scores::get_by_uid(&mut *ctx.db.conn().await?, &uid, None).await?;
    Ok(row.map(UserScore::from))
}

pub async fn get_user_checkin(
    ctx: &AppContext,
    subject: &Subject,
    uid: &str,
) -> Result<Option<UserCheckin>, AppError> {
    let (profile, own, layout) =
        privacy::resolve_profile_for_tabs(ctx, subject, uid, &[ProfileTabId::Stats]).await?;
    let row = users::checkin_by_uid(&mut *ctx.db.conn().await?, &profile.uid, None).await?;
    let mut checkin = row.map(UserCheckin::from);
    if let (false, Some(layout), Some(checkin)) = (own, &layout, &mut checkin) {
        checkin.hide_private(layout);
    }
    Ok(checkin)
}

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

pub async fn delete_account(ctx: &AppContext, subject: &Subject) -> Result<(), AppError> {
    let user = subject.user_id().ok_or(AppError::Unauthorized)?;
    let mut tx = ctx.db.begin().await?;
    let deleted = users::delete_account(&mut tx, user)
        .await?
        .ok_or(AppError::Unauthorized)?;
    tx.commit().await?;

    let uid = deleted.uid.as_str();
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

pub async fn set_user_role(
    ctx: &AppContext,
    subject: &Subject,
    target: UserId,
    role: &str,
) -> Result<(), AppError> {
    ctx.policy
        .require(subject, Action::SetUserRole, &Resource::User { id: target })?;
    let to = role_name(parse_role(role));
    if to != role {
        return Err(AppError::BadRequest(format!("Unknown role: {role}")));
    }
    let mut conn = ctx.db.conn().await?;
    let account = users::account(&mut conn, target)
        .await?
        .ok_or(AppError::NotFound(None))?;
    if subject.user_id() == Some(target) {
        return Err(AppError::BadRequest(
            "refusing to change your own role; use the manage_permissions CLI".to_owned(),
        ));
    }
    if account.role == to {
        return Ok(());
    }
    users::set_role(&mut conn, target, to).await?;
    drop(conn);
    ctx.cache.invalidate(&user_key(&account.uid)).await;
    tracing::info!(
        actor = ?subject.user_id(),
        target = %target.0,
        from = %account.role,
        to,
        "role changed"
    );
    Ok(())
}

pub async fn list_users(
    ctx: &AppContext,
    subject: &Subject,
    query: &AdminUsersQuery,
    page: PageRequest,
) -> Result<AdminUsersPage, AppError> {
    ctx.policy
        .require(subject, Action::ListUsers, &Resource::Site)?;
    let roles: Option<&[Role]> = match query.role.as_deref().map(str::trim) {
        None | Some("" | "all") => None,
        Some("staff") => Some(&[Role::SuperAdmin, Role::TierListAdmin, Role::TierListEditor]),
        Some("translators") => Some(&[Role::Translator]),
        Some("players") => Some(&[Role::User]),
        Some(token) => {
            return Err(AppError::BadRequest(format!(
                "role must be `all`, `staff`, `translators` or `players`, got `{token}`"
            )));
        }
    };
    let roles: Option<Vec<String>> =
        roles.map(|r| r.iter().map(|&role| role_name(role).to_owned()).collect());
    let filter = AdminFilter {
        q: query.q.as_deref().map(str::trim).filter(|q| !q.is_empty()),
        roles: roles.as_deref(),
        server: query.server,
    };
    let limit = page
        .limit
        .unwrap_or(ADMIN_USERS_DEFAULT_LIMIT)
        .clamp(1, ADMIN_USERS_MAX_LIMIT);
    let mut conn = ctx.db.conn().await?;
    let rows = users::admin_page(
        &mut conn,
        &filter,
        i64::from(limit),
        i64::from(page.offset()),
    )
    .await?;
    let total = users::admin_count(&mut conn, &filter).await?;
    Ok(AdminUsersPage {
        users: rows.into_iter().map(AdminUserEntry::from).collect(),
        total,
    })
}
