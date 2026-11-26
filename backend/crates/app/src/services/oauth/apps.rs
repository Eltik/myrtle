use myrtle_db::repos::{oauth as repo, users};
use myrtle_types::{FieldError, UserId};
use uuid::Uuid;

use super::dto::{OAuthApp, OAuthAppInput, OAuthAppWithSecret};
use super::protocol::{
    self, MAX_APPS_PER_USER, MAX_DESCRIPTION_LEN, MAX_NAME_LEN, MAX_REDIRECT_URIS, SecretKind,
};
use crate::policy::{Action, Resource, Subject};
use crate::{AppContext, AppError};

struct CleanApp {
    name: String,
    description: String,
    redirect_uris: Vec<String>,
}

fn field_error(field: &str, message: impl Into<String>) -> FieldError {
    FieldError {
        field: field.to_owned(),
        message: message.into(),
    }
}

fn owner(subject: &Subject) -> Result<UserId, AppError> {
    subject.user_id().ok_or(AppError::Unauthorized)
}

fn clean(input: OAuthAppInput) -> Result<CleanApp, AppError> {
    let mut errors = Vec::new();

    let name = input.name.trim().to_owned();
    let name_len = name.chars().count();
    if name_len == 0 {
        errors.push(field_error("name", "is required"));
    } else if name_len > MAX_NAME_LEN {
        errors.push(field_error(
            "name",
            format!("is longer than {MAX_NAME_LEN} characters"),
        ));
    } else if name.chars().any(char::is_control) {
        errors.push(field_error("name", "must not contain control characters"));
    }

    let description = input.description.unwrap_or_default().trim().to_owned();
    if description.chars().count() > MAX_DESCRIPTION_LEN {
        errors.push(field_error(
            "description",
            format!("is longer than {MAX_DESCRIPTION_LEN} characters"),
        ));
    } else if description.chars().any(char::is_control) {
        errors.push(field_error(
            "description",
            "must not contain line breaks or control characters",
        ));
    }

    let mut redirect_uris: Vec<String> = Vec::new();
    for uri in input.redirect_uris {
        if redirect_uris.len() > MAX_REDIRECT_URIS {
            break;
        }
        if let Err(reason) = protocol::validate_redirect_uri(&uri) {
            errors.push(field_error("redirect_uris", format!("{uri:?} {reason}")));
        } else if !redirect_uris.contains(&uri) {
            redirect_uris.push(uri);
        }
    }
    if redirect_uris.is_empty() && errors.iter().all(|e| e.field != "redirect_uris") {
        errors.push(field_error(
            "redirect_uris",
            "at least one redirect URI is required",
        ));
    } else if redirect_uris.len() > MAX_REDIRECT_URIS {
        errors.push(field_error(
            "redirect_uris",
            format!("at most {MAX_REDIRECT_URIS} redirect URIs are allowed"),
        ));
    }

    if errors.is_empty() {
        Ok(CleanApp {
            name,
            description,
            redirect_uris,
        })
    } else {
        Err(AppError::Validation(errors))
    }
}

pub async fn list(ctx: &AppContext, subject: &Subject) -> Result<Vec<OAuthApp>, AppError> {
    let owner = owner(subject)?;
    let rows = repo::list_apps(&mut *ctx.db.conn().await?, owner).await?;
    Ok(rows.into_iter().map(OAuthApp::from).collect())
}

pub async fn create(
    ctx: &AppContext,
    subject: &Subject,
    input: OAuthAppInput,
) -> Result<OAuthAppWithSecret, AppError> {
    let owner = owner(subject)?;
    let clean = clean(input)?;
    let mut tx = ctx.db.begin().await?;
    let locked = users::lock(&mut tx, owner).await?;
    if !locked {
        return Err(AppError::Unauthorized);
    }
    if repo::count_apps(&mut tx, owner).await? >= MAX_APPS_PER_USER {
        return Err(AppError::Conflict(format!(
            "an account can register at most {MAX_APPS_PER_USER} applications"
        )));
    }

    let client_secret = protocol::mint(SecretKind::ClientSecret);
    let app = repo::insert_app(
        &mut tx,
        owner,
        &clean.name,
        &clean.description,
        &clean.redirect_uris,
        &protocol::digest(&client_secret),
    )
    .await?;
    tx.commit().await?;

    tracing::info!(app = %app.id, "oauth application registered");
    Ok(OAuthAppWithSecret {
        app: app.into(),
        client_secret,
    })
}

pub async fn update(
    ctx: &AppContext,
    subject: &Subject,
    app_id: Uuid,
    input: OAuthAppInput,
) -> Result<OAuthApp, AppError> {
    let owner = owner(subject)?;
    let clean = clean(input)?;
    let row = repo::update_app(
        &mut *ctx.db.conn().await?,
        app_id,
        owner,
        &clean.name,
        &clean.description,
        &clean.redirect_uris,
    )
    .await?
    .ok_or(AppError::NotFound(None))?;
    Ok(row.into())
}

pub async fn rotate_secret(
    ctx: &AppContext,
    subject: &Subject,
    app_id: Uuid,
) -> Result<OAuthAppWithSecret, AppError> {
    let owner = owner(subject)?;
    let client_secret = protocol::mint(SecretKind::ClientSecret);
    let row = repo::replace_secret(
        &mut *ctx.db.conn().await?,
        app_id,
        owner,
        &protocol::digest(&client_secret),
    )
    .await?
    .ok_or(AppError::NotFound(None))?;
    tracing::info!(app = %row.id, "oauth client secret rotated");
    Ok(OAuthAppWithSecret {
        app: row.into(),
        client_secret,
    })
}

pub async fn delete(ctx: &AppContext, subject: &Subject, app_id: Uuid) -> Result<(), AppError> {
    let owner = owner(subject)?;
    remove(ctx, app_id, Some(owner)).await
}

pub async fn take_down(ctx: &AppContext, subject: &Subject, app_id: Uuid) -> Result<(), AppError> {
    ctx.policy
        .require(subject, Action::TakeDownOAuthApp, &Resource::Site)?;
    remove(ctx, app_id, None).await
}

async fn remove(ctx: &AppContext, app_id: Uuid, owner: Option<UserId>) -> Result<(), AppError> {
    if !repo::delete_app(&mut *ctx.db.conn().await?, app_id, owner).await? {
        return Err(AppError::NotFound(None));
    }
    tracing::info!(app = %app_id, by_admin = owner.is_none(), "oauth application deleted");
    Ok(())
}
