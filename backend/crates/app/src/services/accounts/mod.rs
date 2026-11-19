//! Sign-in, game sessions, account settings and deletion.

pub mod account;
pub mod credentials;
pub mod game_session;
pub mod jwt;
pub mod login;
pub mod service_account;
pub mod sessions;

use myrtle_db::repos::{i18n, tier_lists, users};
use myrtle_db::{Conn, DbError};
use myrtle_types::{Role, UserId};

use crate::policy::{Action, Grant, Permission, Subject};
use crate::{AppContext, AppError};
use jwt::Claims;

/// The token's subject; a claim above `user` is capped by the stored role under `role_recheck`, 401 when the row is gone
pub async fn subject(ctx: &AppContext, claims: &Claims) -> Result<Subject, AppError> {
    let id = claims.user_id()?;
    let claimed = jwt::parse_role(&claims.role);
    let role = if ctx.policy.role_recheck && claimed != Role::User {
        let stored = users::role(&mut *ctx.db.conn().await?, id)
            .await?
            .ok_or(AppError::Unauthorized)?;
        claimed.min(jwt::parse_role(&stored))
    } else {
        claimed
    };
    Ok(Subject::User {
        id,
        uid: claims.uid.as_str().into(),
        server: claims.server(),
        role,
        grants: [].into(),
    })
}

/// [`subject`] for a route open to everyone: any token or role check that fails reads as anonymous
pub async fn subject_optional(ctx: &AppContext, token: Option<&str>) -> Subject {
    let Some(claims) = token.and_then(|t| jwt::verify_token(&ctx.secrets.jwt, t).ok()) else {
        return Subject::Anonymous;
    };
    subject(ctx, &claims).await.unwrap_or(Subject::Anonymous)
}

/// `subject` with its per-list and per-locale grants loaded, for a gate that reads them
pub async fn with_grants(ctx: &AppContext, subject: Subject) -> Result<Subject, AppError> {
    let Subject::User {
        id,
        uid,
        server,
        role,
        ..
    } = subject
    else {
        return Ok(subject);
    };
    let grants = grants(&mut *ctx.db.conn().await?, id).await?;
    Ok(Subject::User {
        id,
        uid,
        server,
        role,
        grants: grants.into(),
    })
}

pub async fn grants(conn: &mut Conn, user: UserId) -> Result<Vec<Grant>, DbError> {
    let lists = tier_lists::get_granted_lists(conn, user).await?;
    let locales = i18n::list_permissions_for_user(conn, user).await?;
    let lists = lists.into_iter().filter_map(|row| {
        Some(Grant {
            action: Permission::parse(&row.permission)?.tier_list_action(),
            resource_id: Some(row.tier_list_id.to_string()),
        })
    });
    let locales = locales.into_iter().map(|row| Grant {
        action: Permission::parse(&row.permission)
            .map_or(Action::AccessAdminPanel, Permission::translation_action),
        resource_id: Some(row.locale),
    });
    Ok(lists.chain(locales).collect())
}
