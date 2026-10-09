use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services;
use crate::app::services::operator_notes::GlobalAuditLogResponse;
use crate::app::services::operator_notes::get_all;
use crate::app::services::operator_notes::get_audit_log;
use crate::app::services::operator_notes::get_by_operator;
use crate::app::services::operator_notes::get_global_audit_log;
use crate::app::state::AppState;
use crate::database::models::operator_notes::{OperatorNote, OperatorNoteAuditEntry};

/// Every editorial operator note.
#[utoipa::path(
    get,
    path = "/operator-notes",
    operation_id = "operator_notes_list",
    tag = "notes",
    responses(
        (status = 200, description = "All notes.", body = Vec<OperatorNote>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<OperatorNote>>, ApiError> {
    let notes = get_all(&state).await?;
    Ok(Json(notes))
}

/// One operator's editorial note.
#[utoipa::path(
    get,
    path = "/operator-notes/{operator_id}",
    operation_id = "operator_note_get",
    tag = "notes",
    params(
        ("operator_id" = String, Path, description = "Operator id.")
    ),
    responses(
        (status = 200, description = "The note.", body = OperatorNote),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Path(operator_id): Path<String>,
) -> Result<Json<OperatorNote>, ApiError> {
    let note = get_by_operator(&state, &operator_id).await?;
    Ok(Json(note))
}

/// Edit history for one operator's note.
#[utoipa::path(
    get,
    path = "/operator-notes/{operator_id}/audit",
    operation_id = "operator_note_audit_log",
    tag = "notes",
    params(
        ("operator_id" = String, Path, description = "Operator id.")
    ),
    responses(
        (status = 200, description = "Audit entries, newest first.", body = Vec<OperatorNoteAuditEntry>),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn audit_log(
    State(state): State<AppState>,
    Path(operator_id): Path<String>,
) -> Result<Json<Vec<OperatorNoteAuditEntry>>, ApiError> {
    let log = get_audit_log(&state, &operator_id).await?;
    Ok(Json(log))
}

#[derive(Deserialize)]
pub struct GlobalAuditQuery {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub before: Option<chrono::DateTime<chrono::Utc>>,
    /// `me`, or an account id (UUID): only that account's edits.
    #[serde(default)]
    pub actor: Option<String>,
}

/// Whose rows the global audit returns: `None` for everyone's. A tier list
/// admin reads anyone's; a tier list editor reads only their own, so their
/// `actor` defaults to themselves and naming anyone else is 403.
fn audit_actor_scope(auth: &AuthUser, actor: Option<&str>) -> Result<Option<uuid::Uuid>, ApiError> {
    if !auth.role.is_any_admin_role() {
        return Err(ApiError::Forbidden);
    }
    let me = || auth.user_uuid();
    let named = match actor.map(str::trim).filter(|s| !s.is_empty()) {
        None => None,
        Some("me") => Some(me()?),
        Some(id) => Some(id.parse::<uuid::Uuid>().map_err(|_| {
            ApiError::BadRequest(format!("actor must be `me` or an account id, got `{id}`"))
        })?),
    };
    if auth.role.is_tier_list_admin() {
        return Ok(named);
    }
    let me = me()?;
    match named {
        Some(id) if id != me => Err(ApiError::Forbidden),
        _ => Ok(Some(me)),
    }
}

/// Edit history across every operator note.
///
/// Tier list admins read every account's edits. A tier list editor reads
/// only their own: `actor` defaults to them, and naming anyone else is 403.
#[utoipa::path(
    get,
    path = "/admin/operator-notes/audit",
    tag = "admin",
    params(
        ("limit" = Option<i64>, Query, description = "Page size. Defaults to 100, clamped to 1..=500."),
        ("before" = Option<String>, Query, description = "RFC 3339 timestamp: only edits strictly older, for paging."),
        ("actor" = Option<String>, Query, description = "`me` or an account id (UUID): only that account's edits. Editors always get their own.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Audit entries, newest first.", body = GlobalAuditLogResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn global_audit_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<GlobalAuditQuery>,
) -> Result<Json<GlobalAuditLogResponse>, ApiError> {
    let actor = audit_actor_scope(&auth, params.actor.as_deref())?;
    let limit = params.limit.unwrap_or(100);
    let response = get_global_audit_log(&state, limit, params.before, actor).await?;
    Ok(Json(response))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateNoteRequest {
    pub pros: Option<String>,
    pub cons: Option<String>,
    pub notes: Option<String>,
    pub trivia: Option<String>,
    pub summary: Option<String>,
    pub tags: Option<serde_json::Value>,
}

/// Write an operator's editorial note.
///
/// Every write is recorded in the note's audit log.
#[utoipa::path(
    put,
    path = "/operator-notes/{operator_id}",
    operation_id = "operator_note_update",
    tag = "notes",
    params(
        ("operator_id" = String, Path, description = "Operator id.")
    ),
    request_body = UpdateNoteRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The stored note.", body = OperatorNote),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(operator_id): Path<String>,
    Json(body): Json<UpdateNoteRequest>,
) -> Result<Json<OperatorNote>, ApiError> {
    if !auth.role.is_any_admin_role() {
        return Err(ApiError::Forbidden);
    }

    let user_id: uuid::Uuid = auth.user_uuid()?;

    let note = services::operator_notes::update(
        &state,
        &operator_id,
        user_id,
        services::operator_notes::UpdateFields {
            pros: body.pros,
            cons: body.cons,
            notes: body.notes,
            trivia: body.trivia,
            summary: body.summary,
            tags: body.tags,
        },
    )
    .await?;
    Ok(Json(note))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::auth::permissions::GlobalRole;

    const ME: &str = "00000000-0000-0000-0000-000000000001";
    const OTHER: &str = "00000000-0000-0000-0000-000000000002";

    fn auth(role: GlobalRole) -> AuthUser {
        AuthUser {
            user_id: ME.to_owned(),
            uid: "1".to_owned(),
            server: "en".to_owned(),
            role,
        }
    }

    fn id(s: &str) -> uuid::Uuid {
        s.parse().expect("uuid")
    }

    #[test]
    fn admins_read_everyone_or_whoever_they_name() {
        let admin = auth(GlobalRole::TierListAdmin);
        assert_eq!(audit_actor_scope(&admin, None).ok(), Some(None));
        assert_eq!(
            audit_actor_scope(&admin, Some("me")).ok(),
            Some(Some(id(ME)))
        );
        assert_eq!(
            audit_actor_scope(&admin, Some(OTHER)).ok(),
            Some(Some(id(OTHER)))
        );
        assert!(matches!(
            audit_actor_scope(&admin, Some("nobody")),
            Err(ApiError::BadRequest(_))
        ));
    }

    #[test]
    fn editors_read_only_their_own_rows() {
        let editor = auth(GlobalRole::TierListEditor);
        assert_eq!(audit_actor_scope(&editor, None).ok(), Some(Some(id(ME))));
        assert_eq!(
            audit_actor_scope(&editor, Some("me")).ok(),
            Some(Some(id(ME)))
        );
        assert_eq!(
            audit_actor_scope(&editor, Some(ME)).ok(),
            Some(Some(id(ME)))
        );
        assert!(matches!(
            audit_actor_scope(&editor, Some(OTHER)),
            Err(ApiError::Forbidden)
        ));
    }

    #[test]
    fn other_roles_are_refused() {
        for role in [GlobalRole::User, GlobalRole::Translator] {
            assert!(matches!(
                audit_actor_scope(&auth(role), None),
                Err(ApiError::Forbidden)
            ));
        }
    }
}
