use chrono::{DateTime, Utc};
use myrtle_db::repos::i18n::GrantRow;
use myrtle_db::repos::{i18n, users};
use myrtle_types::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{require_locale, require_panel};
use crate::policy::{Action, Permission, Resource, Subject};
use crate::{AppContext, AppError};

#[derive(Debug, Clone, Serialize)]
pub struct TranslationGrant {
    pub locale: String,
    pub user_id: Uuid,
    pub user_uid: Option<String>,
    pub user_nickname: Option<String>,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub granted_by_nickname: Option<String>,
    pub granted_at: DateTime<Utc>,
}

impl From<GrantRow> for TranslationGrant {
    fn from(r: GrantRow) -> Self {
        Self {
            locale: r.locale,
            user_id: r.user_id,
            user_uid: r.user_uid,
            user_nickname: r.user_nickname,
            permission: r.permission,
            granted_by: r.granted_by,
            granted_by_nickname: r.granted_by_nickname,
            granted_at: r.granted_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TranslationGrantRequest {
    pub locale: String,
    pub user_id: UserId,
    pub permission: String,
}

pub async fn list_grants(
    ctx: &AppContext,
    subject: &Subject,
    locale: Option<&str>,
) -> Result<Vec<TranslationGrant>, AppError> {
    let subject = require_panel(ctx, subject).await?;
    let only = if ctx
        .policy
        .can(&subject, Action::ListUsers, &Resource::Site)
        .is_allowed()
    {
        None
    } else {
        Some(subject.user_id().ok_or(AppError::Unauthorized)?)
    };
    let rows = i18n::list_permissions(&mut *ctx.db.conn().await?, locale, only).await?;
    Ok(rows.into_iter().map(TranslationGrant::from).collect())
}

pub async fn my_grants(
    ctx: &AppContext,
    subject: &Subject,
) -> Result<Vec<TranslationGrant>, AppError> {
    let user = subject.user_id().ok_or(AppError::Unauthorized)?;
    let rows = i18n::list_permissions(&mut *ctx.db.conn().await?, None, Some(user)).await?;
    Ok(rows.into_iter().map(TranslationGrant::from).collect())
}

pub async fn grant(
    ctx: &AppContext,
    subject: &Subject,
    req: &TranslationGrantRequest,
) -> Result<(), AppError> {
    let granted_by = require_locale(ctx, subject, Action::ManageTranslators, &req.locale).await?;
    let permission = parse_permission(&req.permission)?;
    let mut conn = ctx.db.conn().await?;
    if i18n::get_locale(&mut conn, &req.locale).await?.is_none()
        || users::role(&mut conn, req.user_id).await?.is_none()
    {
        return Err(AppError::NotFound(None));
    }
    i18n::grant_permission(
        &mut conn,
        &req.locale,
        req.user_id,
        permission.as_str(),
        granted_by,
    )
    .await?;
    Ok(())
}

pub async fn revoke(
    ctx: &AppContext,
    subject: &Subject,
    req: &TranslationGrantRequest,
) -> Result<(), AppError> {
    require_locale(ctx, subject, Action::ManageTranslators, &req.locale).await?;
    let permission = parse_permission(&req.permission)?;
    i18n::revoke_permission(
        &mut *ctx.db.conn().await?,
        &req.locale,
        req.user_id,
        permission.as_str(),
    )
    .await?;
    Ok(())
}

fn parse_permission(s: &str) -> Result<Permission, AppError> {
    Permission::parse(s).ok_or_else(|| AppError::bad_request(format!("Unknown permission: {s}")))
}
