use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::cache::hmac_hex;
use dashmap::DashMap;
use myrtle_db::repos::{tier_lists as repo, users};
use myrtle_types::{ContentAuditAction, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    GrantedTierList, TierListFlair, TierListGrant, TierListPermission, TierListStats, actor,
    audit_event, find, find_and_authorize, invalidate_detail, list_id, validate_hex_color,
};
use crate::audit::ContentAuditDetails;
use crate::policy::{Action, Permission, Resource, Subject};
use crate::{AppContext, AppError};

const SESSION_ID_LEN: usize = 64;
const VIEW_RATE_WINDOW: Duration = Duration::from_mins(1);
const VIEW_RATE_LIMIT: u32 = 10;
const VIEW_RATE_SWEEP_EVERY: u64 = 1_000;

static VIEW_RATE_BUCKETS: LazyLock<DashMap<String, (Instant, u32)>> = LazyLock::new(DashMap::new);
static VIEW_RATE_CALLS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize)]
pub struct ViewRecorded {
    pub unique: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FavoriteState {
    pub favorited: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VisibilityState {
    pub is_listed: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GrantRequest {
    pub user_id: Uuid,
    pub permission: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFlairRequest {
    pub code: String,
    pub label: String,
    pub color: Option<String>,
    pub display_order: Option<i16>,
}

/// Hex HMAC-SHA256 of the visitor's session id, the dedupe key a view row stores
#[must_use]
pub fn session_hash(secret: &str, session_id: &str) -> String {
    hmac_hex(secret.as_bytes(), session_id.as_bytes())
}

fn valid_session_id(value: &str) -> bool {
    value.len() == SESSION_ID_LEN && value.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Ten views a minute per address; idle addresses are swept every thousand calls
fn check_view_rate(ip: &str) -> bool {
    let now = Instant::now();
    if VIEW_RATE_CALLS
        .fetch_add(1, Ordering::Relaxed)
        .is_multiple_of(VIEW_RATE_SWEEP_EVERY)
    {
        VIEW_RATE_BUCKETS.retain(|_, (start, _)| now.duration_since(*start) < VIEW_RATE_WINDOW);
    }
    let mut entry = VIEW_RATE_BUCKETS.entry(ip.to_owned()).or_insert((now, 0));
    let (window_start, count) = *entry;
    if now.duration_since(window_start) >= VIEW_RATE_WINDOW {
        *entry = (now, 1);
        true
    } else if count >= VIEW_RATE_LIMIT {
        false
    } else {
        entry.1 = count + 1;
        true
    }
}

/// Counts a view; with neither an account nor a valid session id nothing is written
pub async fn record_view(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    session_id: Option<&str>,
    client_ip: Option<&str>,
) -> Result<ViewRecorded, AppError> {
    let mut conn = ctx.db.conn().await?;
    let list = find(&mut conn, slug).await?;
    if client_ip.is_some_and(|ip| !check_view_rate(ip)) {
        return Err(AppError::RateLimited);
    }
    let user = subject.user_id();
    let session = user
        .is_none()
        .then(|| session_id.filter(|s| valid_session_id(s)))
        .flatten()
        .map(|sid| session_hash(&ctx.secrets.jwt, sid));
    let Some(viewer) = repo::Viewer::from_parts(user, session.as_deref()) else {
        return Ok(ViewRecorded { unique: false });
    };
    let unique = repo::record_view(&mut conn, list_id(&list), viewer).await?;
    Ok(ViewRecorded { unique })
}

pub async fn stats(ctx: &AppContext, slug: &str) -> Result<TierListStats, AppError> {
    let mut conn = ctx.db.conn().await?;
    let list = find(&mut conn, slug).await?;
    repo::get_stats(&mut conn, list_id(&list))
        .await?
        .map(Into::into)
        .ok_or(AppError::NotFound(None))
}

pub async fn toggle_favorite(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
) -> Result<FavoriteState, AppError> {
    let user = actor(subject)?;
    let mut conn = ctx.db.conn().await?;
    let id = list_id(&find(&mut conn, slug).await?);
    let favorited = if repo::is_favorited(&mut conn, id, user).await? {
        repo::remove_favorite(&mut conn, id, user).await?;
        false
    } else {
        repo::add_favorite(&mut conn, id, user).await?;
        true
    };
    drop(conn);
    invalidate_detail(ctx, slug).await;
    Ok(FavoriteState { favorited })
}

pub async fn favorite(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
) -> Result<FavoriteState, AppError> {
    let user = actor(subject)?;
    let mut conn = ctx.db.conn().await?;
    let id = list_id(&find(&mut conn, slug).await?);
    let favorited = repo::is_favorited(&mut conn, id, user).await?;
    Ok(FavoriteState { favorited })
}

/// `None` clears the flair; an audit row only when it changed
pub async fn set_flair(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    flair_id: Option<i16>,
) -> Result<(), AppError> {
    let mut conn = ctx.db.conn().await?;
    let (list, user) =
        find_and_authorize(ctx, &mut conn, subject, slug, Action::EditTierList).await?;
    repo::set_flair(&mut conn, list_id(&list), flair_id).await?;
    let flair = match flair_id {
        Some(id) if list.flair_id != flair_id => repo::get_flair_by_id(&mut conn, id).await?,
        _ => None,
    };
    drop(conn);
    invalidate_detail(ctx, slug).await;
    if list.flair_id != flair_id {
        let details = ContentAuditDetails {
            flair: flair.map(|f| f.label),
            ..Default::default()
        };
        ctx.audit
            .record(audit_event(&list, user, ContentAuditAction::Flair, details))
            .await;
    }
    Ok(())
}

/// Active flairs
pub async fn flairs(ctx: &AppContext) -> Result<Vec<TierListFlair>, AppError> {
    let rows = repo::list_flairs(&mut *ctx.db.conn().await?, true).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn create_flair(
    ctx: &AppContext,
    subject: &Subject,
    req: &CreateFlairRequest,
) -> Result<TierListFlair, AppError> {
    ctx.policy
        .require(subject, Action::ManageTierListGrants, &Resource::Site)?;
    validate_hex_color(req.color.as_deref())?;
    let flair = repo::create_flair(
        &mut *ctx.db.conn().await?,
        &req.code,
        &req.label,
        req.color.as_deref(),
        req.display_order.unwrap_or(0),
    )
    .await?;
    Ok(flair.into())
}

pub async fn set_visibility(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    is_listed: bool,
) -> Result<VisibilityState, AppError> {
    let mut conn = ctx.db.conn().await?;
    let (list, user) =
        find_and_authorize(ctx, &mut conn, subject, slug, Action::EditTierList).await?;
    repo::set_visibility(&mut conn, list_id(&list), is_listed).await?;
    drop(conn);
    invalidate_detail(ctx, slug).await;
    if list.is_listed != is_listed {
        let details = ContentAuditDetails {
            listed: Some(is_listed),
            ..Default::default()
        };
        ctx.audit
            .record(audit_event(
                &list,
                user,
                ContentAuditAction::Visibility,
                details,
            ))
            .await;
    }
    Ok(VisibilityState { is_listed })
}

pub async fn permissions(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
) -> Result<Vec<TierListPermission>, AppError> {
    ctx.policy
        .require(subject, Action::ManageTierListGrants, &Resource::Site)?;
    let mut conn = ctx.db.conn().await?;
    let list = find(&mut conn, slug).await?;
    let rows = repo::get_permissions(&mut conn, list_id(&list)).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn grant(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    req: &GrantRequest,
) -> Result<(), AppError> {
    ctx.policy
        .require(subject, Action::ManageTierListGrants, &Resource::Site)?;
    let granter = actor(subject)?;
    let permission = Permission::parse(&req.permission)
        .ok_or_else(|| AppError::BadRequest(format!("unknown permission `{}`", req.permission)))?
        .as_str();
    let grantee = UserId(req.user_id);
    let mut conn = ctx.db.conn().await?;
    let list = find(&mut conn, slug).await?;
    users::role(&mut conn, grantee)
        .await?
        .ok_or_else(|| AppError::NotFound(Some("user not found".into())))?;
    repo::grant_permission(&mut conn, list_id(&list), grantee, permission, granter).await?;
    drop(conn);
    audit_grant(
        ctx,
        &list,
        granter,
        grantee,
        ContentAuditAction::Grant,
        permission,
    )
    .await;
    Ok(())
}

pub async fn revoke(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    user: UserId,
    permission: &str,
) -> Result<(), AppError> {
    ctx.policy
        .require(subject, Action::ManageTierListGrants, &Resource::Site)?;
    let mut conn = ctx.db.conn().await?;
    let list = find(&mut conn, slug).await?;
    repo::revoke_permission(&mut conn, list_id(&list), user, permission).await?;
    drop(conn);
    if let Subject::User { id: actor, .. } = subject {
        audit_grant(
            ctx,
            &list,
            *actor,
            user,
            ContentAuditAction::Revoke,
            permission,
        )
        .await;
    }
    Ok(())
}

async fn audit_grant(
    ctx: &AppContext,
    list: &repo::TierListRow,
    actor: UserId,
    grantee: UserId,
    action: ContentAuditAction,
    permission: &str,
) {
    let details = ContentAuditDetails {
        permission: Some(permission.to_owned()),
        ..Default::default()
    };
    let mut event = audit_event(list, actor, action, details);
    event.subject = Some(grantee);
    ctx.audit.record(event).await;
}

/// Every grant on every active list, or one user's
pub async fn all_grants(
    ctx: &AppContext,
    subject: &Subject,
    user: Option<UserId>,
) -> Result<Vec<TierListGrant>, AppError> {
    ctx.policy
        .require(subject, Action::ManageTierListGrants, &Resource::Site)?;
    let rows = repo::get_all_grants(&mut *ctx.db.conn().await?, user).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// The lists the caller holds a grant on
pub async fn granted(
    ctx: &AppContext,
    subject: &Subject,
) -> Result<Vec<GrantedTierList>, AppError> {
    let user = actor(subject)?;
    let rows = repo::get_granted_lists(&mut *ctx.db.conn().await?, user).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}
