//! Trevor, the story question answerer: the reader routes (ask, poll, panels,
//! feedback) and the worker routes behind the service key. See
//! `app/services/trevor` and `trevor/design/trevor-integration.md`.
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::app::extractors::auth::AuthUser;
use crate::app::routes::static_data::json_response;
use crate::app::services::trevor::{
    self as svc, AskRequest, AskResponse, FeedbackRequest, JobView, StoryPanels,
    worker::{self, PublishOutcome, PublishVersion, WorkerHello, WorkerJob, WorkerResult},
};
use crate::app::{error::ApiError, state::AppState};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ServerQuery {
    /// `en` (the default) or `cn`.
    pub server: Option<String>,
}

/// Ask Trevor a question about the story.
///
/// Answers 200 with a cached answer, or 202 with a job to poll at
/// `GET /trevor/jobs/{id}`. Thirty questions an hour per account. The spoiler
/// horizon defaults to the caller's furthest-read story; `noHorizon` lifts it.
#[utoipa::path(
    post,
    path = "/trevor/ask",
    tag = "trevor",
    request_body = AskRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "A cached answer (`status: answered`).", body = AskResponse),
        (status = 202, description = "Queued for a worker (`status: queued`): the job id, its place in the queue and an ETA in seconds.", body = AskResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn ask(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<AskRequest>,
) -> Result<Response, ApiError> {
    let outcome = svc::ask::ask(&state, auth.user_uuid()?, body).await?;
    let status = match outcome {
        AskResponse::Answered(_) => StatusCode::OK,
        AskResponse::Queued(_) => StatusCode::ACCEPTED,
    };
    Ok((status, Json(outcome)).into_response())
}

/// A queued question's state; poll every 2 s until `done` or `failed`.
#[utoipa::path(
    get,
    path = "/trevor/jobs/{id}",
    tag = "trevor",
    params(("id" = Uuid, Path, description = "The job id from `POST /trevor/ask`.")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The job, with the answer once done.", body = JobView),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn job(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<JobView>, ApiError> {
    Ok(Json(svc::ask::job(&state, id).await?))
}

/// The precomputed, no-model panels of one story (who speaks, where else they
/// appear, what to read before, topics), from the active corpus version.
#[utoipa::path(
    get,
    path = "/trevor/story/{storyId}/panels",
    tag = "trevor",
    params(
        ("storyId" = String, Path, description = "A story id from the story index."),
        ServerQuery,
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The story's panels under the active version.", body = StoryPanels),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn story_panels(
    State(state): State<AppState>,
    Path(story_id): Path<String>,
    Query(query): Query<ServerQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let server = svc::parse_server(query.server.as_deref())?;
    let cached = svc::ask::panels(&state, server, &story_id).await?;
    Ok(json_response(cached, &headers))
}

/// Vote on an answer, optionally with a "this is wrong" note. A second call
/// replaces the caller's earlier vote.
#[utoipa::path(
    post,
    path = "/trevor/answers/{id}/feedback",
    tag = "trevor",
    params(("id" = Uuid, Path, description = "The answer id.")),
    request_body = FeedbackRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Recorded."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn feedback(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<FeedbackRequest>,
) -> Result<StatusCode, ApiError> {
    svc::ask::feedback(&state, auth.user_uuid()?, id, body).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Worker: claim the next queued job. Holds an empty poll open for up to 20 s,
/// then answers 204.
#[utoipa::path(
    post,
    path = "/trevor/worker/next",
    tag = "trevor",
    request_body = WorkerHello,
    security(("service_key" = [])),
    responses(
        (status = 200, description = "A claimed job.", body = WorkerJob),
        (status = 204, description = "No job arrived within the wait."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn worker_next(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<WorkerHello>,
) -> Result<Response, ApiError> {
    worker::require_service(&auth)?;
    Ok(match worker::next(&state, body).await? {
        Some(job) => Json(job).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    })
}

/// Worker: post a job's answer or failure.
#[utoipa::path(
    post,
    path = "/trevor/worker/result",
    tag = "trevor",
    request_body = WorkerResult,
    security(("service_key" = [])),
    responses(
        (status = 204, description = "Stored; the job is closed."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn worker_result(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<WorkerResult>,
) -> Result<StatusCode, ApiError> {
    worker::require_service(&auth)?;
    worker::result(&state, body).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Worker: report liveness (every 30 s); drives the reader's "online" state.
#[utoipa::path(
    post,
    path = "/trevor/worker/heartbeat",
    tag = "trevor",
    request_body = WorkerHello,
    security(("service_key" = [])),
    responses(
        (status = 204, description = "Recorded."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn worker_heartbeat(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<WorkerHello>,
) -> Result<StatusCode, ApiError> {
    worker::require_service(&auth)?;
    worker::heartbeat(&state, &body).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Worker: publish a corpus version's manifest and panel JSON, and (by
/// default) make it the server's active version.
#[utoipa::path(
    post,
    path = "/trevor/versions",
    tag = "trevor",
    request_body = PublishVersion,
    security(("service_key" = [])),
    responses(
        (status = 200, description = "Published.", body = PublishOutcome),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn publish_version(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<PublishVersion>,
) -> Result<Json<PublishOutcome>, ApiError> {
    worker::require_service(&auth)?;
    Ok(Json(worker::publish(&state, body).await?))
}
