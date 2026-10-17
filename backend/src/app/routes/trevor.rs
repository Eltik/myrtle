//! Trevor, the story question answerer: the reader routes (ask, poll, stream,
//! panels, feedback, status) and the worker routes behind the scoped worker
//! key. See `app/services/trevor` and `trevor/design/trevor-contract-v1.md`.
use std::net::IpAddr;

use axum::{
    Json,
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, FromRequestParts, Path, Query, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tower_http::decompression::RequestDecompressionLayer;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::app::cache::strong_etag;
use crate::app::extractors::auth::{AuthUser, MaybeAuthUser};
use crate::app::services::trevor::{
    self as svc, AskRequest, AskResponse, FeedbackRequest, JobStatus, JobView, MyQuestion, Popular,
    StoryPanels, TrevorStatus,
    worker::{
        self, Heartbeat, PublishOutcome, PublishVersion, ResultError, WarmList, WarmOutcome,
        WarmRequest, WorkerDelta, WorkerHello, WorkerJob, WorkerKey, WorkerResult,
    },
};
use crate::app::{error::ApiError, state::AppState};

/// A publish request's body as it arrives (gzip or not): at most 4 MiB.
pub const PUBLISH_COMPRESSED_MAX: usize = 4 * 1024 * 1024;
/// A publish request's body once decompressed: at most 8 MiB.
pub const PUBLISH_DECOMPRESSED_MAX: usize = 8 * 1024 * 1024;

/// `Cache-Control` of `GET /trevor/status`.
pub const STATUS_CACHE_CONTROL: &str = "public, max-age=15";
/// `Cache-Control` of a panel response whose `?v=` names the active version.
pub const PANELS_IMMUTABLE: &str = "public, max-age=86400, immutable";
/// `Cache-Control` of any other panel response (the old header for all).
pub const PANELS_SHORT: &str = "public, max-age=300";

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ServerQuery {
    /// `en` (the default) or `cn`.
    pub server: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PanelsQuery {
    /// `en` (the default) or `cn`.
    pub server: Option<String>,
    /// The corpus version the caller expects. When it is the active one the
    /// response is cacheable for a day and immutable.
    pub v: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct MyJobsQuery {
    /// At most this many (default 20, at most 50).
    pub limit: Option<i64>,
    /// Only questions asked before this instant (RFC 3339): pass the last
    /// item's `createdAt` for the next page.
    pub before: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PopularQuery {
    /// `en` (the default) or `cn`.
    pub server: Option<String>,
    /// At most this many (default 8, at most 50).
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct WorkerPopularQuery {
    /// The newly published version: questions it already holds a cached
    /// answer for (carried over or answered) are left out.
    pub version: String,
    /// `en` (the default) or `cn`.
    pub server: Option<String>,
    /// At most this many (1 to 500, default 100).
    pub limit: Option<i64>,
}

/// The client address Turnstile's `remoteip` gets: the rate limiter's view
/// (`X-Forwarded-For` behind `TRUSTED_PROXY_HOPS`, else the socket peer).
#[derive(Debug, Clone, Copy)]
pub struct ClientIp(pub Option<IpAddr>);

impl FromRequestParts<AppState> for ClientIp {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(crate::app::middleware::peer_ip(
            &parts.headers,
            &parts.extensions,
        )))
    }
}

/// Ask Trevor a question about the story.
///
/// Answers 200 with a cached answer (`status: done`, `rung: cache`), or 202
/// with the job that will answer: its rung, ETA and SSE `streamUrl`. Thirty
/// questions an hour per account. The spoiler horizon is the furthest story
/// the account has read, lowered (never raised) by `horizonStory`, then
/// bucketed to its era (`spoilerSafeUpTo`). With `TURNSTILE_SECRET` set,
/// `turnstileToken` is verified and an unreachable Cloudflare is a 503.
#[utoipa::path(
    post,
    path = "/trevor/ask",
    tag = "trevor",
    request_body = AskRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "A cached answer.", body = AskResponse),
        (status = 202, description = "Queued (or joined to an identical open question): the job, its rung, an ETA in seconds and the stream URL.", body = AskResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn ask(
    State(state): State<AppState>,
    auth: AuthUser,
    ClientIp(ip): ClientIp,
    Json(body): Json<AskRequest>,
) -> Result<Response, ApiError> {
    let outcome = svc::ask::ask(&state, auth.user_uuid()?, ip, body).await?;
    let status = if outcome.status == JobStatus::Done && outcome.job_id.is_none() {
        StatusCode::OK
    } else {
        StatusCode::ACCEPTED
    };
    Ok((status, Json(outcome)).into_response())
}

/// A queued question's state: the polling fallback to the stream. Only its
/// asker (or a twin who joined the same question) may read it.
#[utoipa::path(
    get,
    path = "/trevor/jobs/{id}",
    tag = "trevor",
    params(("id" = Uuid, Path, description = "The job id from `POST /trevor/ask`.")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "The job, with the answer once done.", body = JobView),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn job(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<JobView>, ApiError> {
    Ok(Json(svc::ask::job(&state, auth.user_uuid()?, id).await?))
}

/// A job's answer as it is written, over server-sent events.
///
/// Events: `delta` `{seq, text}` (append in `seq` order), `done` (the job
/// view, with the answer), `error` `{message}`, and a `: keepalive` comment
/// every 15 s. Closes after `done` or `error`; a finished job sends one at
/// once. Only the asker (or a twin who joined the same question) may open it.
#[utoipa::path(
    get,
    path = "/trevor/jobs/{id}/stream",
    tag = "trevor",
    params(("id" = Uuid, Path, description = "The job id from `POST /trevor/ask`.")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "An SSE stream of `delta`, `done` and `error` events.", content_type = "text/event-stream", body = String),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn job_stream(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    svc::stream::open(&state, auth.user_uuid()?, id).await
}

/// The `Cache-Control` of a panel response.
pub fn panels_cache_control(
    requested: Option<&str>,
    active: &str,
    immutable: bool,
) -> &'static str {
    if immutable && requested == Some(active) {
        PANELS_IMMUTABLE
    } else {
        PANELS_SHORT
    }
}

/// The precomputed, no-model panels of one story (who speaks, where else they
/// appear, what to read before, topics), from the active corpus version.
#[utoipa::path(
    get,
    path = "/trevor/panels/{storyId}",
    tag = "trevor",
    params(
        ("storyId" = String, Path, description = "A story id from the story index."),
        PanelsQuery,
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The story's panels under the active version. `Cache-Control: public, max-age=86400, immutable` when `v` is the active version, else `max-age=300`.", body = StoryPanels),
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
    Query(query): Query<PanelsQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let server = svc::parse_server(query.server.as_deref())?;
    let panels = svc::ask::panels(&state, server, &story_id).await?;
    let cache_control = panels_cache_control(
        query.v.as_deref(),
        &panels.version,
        state.trevor.config().panels_immutable,
    );
    let body = serde_json::to_vec(&panels).map_err(|e| ApiError::Internal(e.into()))?;
    let etag = strong_etag(&body);
    let mut response = if headers
        .get(header::IF_NONE_MATCH)
        .is_some_and(|inm| inm.as_bytes() == etag.as_bytes())
    {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        ([(header::CONTENT_TYPE, "application/json")], body).into_response()
    };
    let h = response.headers_mut();
    if let Ok(v) = HeaderValue::from_str(&etag) {
        h.insert(header::ETAG, v);
    }
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(cache_control),
    );
    Ok(response)
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

/// Whether Trevor can answer right now: a worker online, which model rungs
/// it offers, the active corpus version, the model queues and today's
/// Cloudflare budget as a bucket (`ok`, `low` under 20% left, `spent`).
#[utoipa::path(
    get,
    path = "/trevor/status",
    tag = "trevor",
    params(ServerQuery),
    responses(
        (status = 200, description = "The status; `Cache-Control: public, max-age=15`.", body = TrevorStatus),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn status(
    State(state): State<AppState>,
    Query(query): Query<ServerQuery>,
) -> Result<Response, ApiError> {
    let server = svc::parse_server(query.server.as_deref())?;
    let body = svc::ask::status(&state, server).await?;
    Ok(([(header::CACHE_CONTROL, STATUS_CACHE_CONTROL)], Json(body)).into_response())
}

/// The reader's recent questions for "My questions": each with its job
/// (the deferred one of an overflowed ask), status, rung, source, times, and
/// the answer once done. Cache hits are listed too (`rung: cache`, no job).
#[utoipa::path(
    get,
    path = "/trevor/me/jobs",
    tag = "trevor",
    params(MyJobsQuery),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Newest first.", body = Vec<MyQuestion>),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn my_jobs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<MyJobsQuery>,
) -> Result<Json<Vec<MyQuestion>>, ApiError> {
    Ok(Json(
        svc::ask::my_questions(&state, auth.user_uuid()?, query.before, query.limit).await?,
    ))
}

/// The reader's era and the most-asked cached questions inside it, with their
/// answers, asked by at least two accounts. Signed out: no era and an empty
/// list (era 0, nothing released).
#[utoipa::path(
    get,
    path = "/trevor/popular",
    tag = "trevor",
    params(PopularQuery),
    responses(
        (status = 200, description = "The era and its questions, most askers first.", body = Popular),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn popular(
    State(state): State<AppState>,
    MaybeAuthUser(auth): MaybeAuthUser,
    Query(query): Query<PopularQuery>,
) -> Result<Response, ApiError> {
    let server = svc::parse_server(query.server.as_deref())?;
    let user = auth.map(|a| a.user_uuid()).transpose()?;
    let body = svc::ask::popular(&state, user, server, query.limit).await?;
    Ok(([(header::CACHE_CONTROL, "private, max-age=60")], Json(body)).into_response())
}

/// Worker: the warm-up list after a publish: the (normQ, eraPos) pairs most
/// asked in the last 30 days that `version` holds no cached answer for
/// (carried over or made since). 404 for an unknown version.
#[utoipa::path(
    get,
    path = "/trevor/worker/popular",
    tag = "trevor",
    params(WorkerPopularQuery),
    security(("trevor_worker_key" = [])),
    responses(
        (status = 200, description = "Most asked first.", body = WarmList),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn worker_popular(
    State(state): State<AppState>,
    _key: WorkerKey,
    Query(query): Query<WorkerPopularQuery>,
) -> Result<Json<WarmList>, ApiError> {
    Ok(Json(
        worker::popular(&state, query.server.as_deref(), &query.version, query.limit).await?,
    ))
}

/// Worker: queue internal re-answers (no user) as deferred jobs, each under
/// its era as the horizon: `queued`, `cached` (already answered in the
/// version, nothing queued) and `joined` (an open job asks it). 409 when the
/// version is no longer active; 503 unless `TREVOR_MODE` is `on`; a body
/// over 64 KB is a 413.
#[utoipa::path(
    post,
    path = "/trevor/worker/warm",
    tag = "trevor",
    request_body = WarmRequest,
    security(("trevor_worker_key" = [])),
    responses(
        (status = 200, description = "Queued, cached and joined counts.", body = WarmOutcome),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 413, description = "The body is over 64 KB."),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn worker_warm(
    State(state): State<AppState>,
    _key: WorkerKey,
    Json(body): Json<WarmRequest>,
) -> Result<Json<WarmOutcome>, ApiError> {
    Ok(Json(worker::warm(&state, body).await?))
}

/// Worker: claim the next queued job, with the rung, profile and horizon
/// gatePos it runs under. Holds an empty poll open for up to 20 s (woken on
/// enqueue), then answers 204.
#[utoipa::path(
    post,
    path = "/trevor/worker/next",
    tag = "trevor",
    request_body = WorkerHello,
    security(("trevor_worker_key" = [])),
    responses(
        (status = 200, description = "A claimed job.", body = WorkerJob),
        (status = 204, description = "No job arrived within the wait."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn worker_next(
    State(state): State<AppState>,
    _key: WorkerKey,
    Json(body): Json<WorkerHello>,
) -> Result<Response, ApiError> {
    Ok(match worker::next(&state, body).await? {
        Some(job) => Json(job).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    })
}

/// Worker: stream a running job's answer text to its readers. Not stored.
#[utoipa::path(
    post,
    path = "/trevor/worker/jobs/{jobId}/delta",
    tag = "trevor",
    params(("jobId" = Uuid, Path, description = "The running job.")),
    request_body = WorkerDelta,
    security(("trevor_worker_key" = [])),
    responses(
        (status = 204, description = "Fanned out to the job's stream readers (if any)."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn worker_delta(
    State(state): State<AppState>,
    _key: WorkerKey,
    Path(job_id): Path<Uuid>,
    Json(body): Json<WorkerDelta>,
) -> Result<StatusCode, ApiError> {
    worker::delta(&state, job_id, body).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Worker: post a job's answer (only the cited passages, trimmed) or failure.
/// A body over 64 KB is refused with a 413.
#[utoipa::path(
    post,
    path = "/trevor/worker/result",
    tag = "trevor",
    request_body = WorkerResult,
    security(("trevor_worker_key" = [])),
    responses(
        (status = 204, description = "Stored; the job is closed."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 413, description = "The result is over `TREVOR_RESULT_MAX_BYTES` (64 KB)."),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn worker_result(
    State(state): State<AppState>,
    _key: WorkerKey,
    body: Bytes,
) -> Result<Response, ApiError> {
    match worker::result(&state, &body).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(ResultError::Api(e)) => Err(e),
        Err(ResultError::TooLarge(cap)) => Ok(payload_too_large(&format!(
            "the result is over {cap} bytes; send only the cited passages"
        ))),
    }
}

fn payload_too_large(message: &str) -> Response {
    (
        StatusCode::PAYLOAD_TOO_LARGE,
        Json(serde_json::json!({
            "error": { "code": "PAYLOAD_TOO_LARGE", "message": message }
        })),
    )
        .into_response()
}

/// Worker: report liveness and live rungs (every 30 s, mid-job too); drives
/// `/trevor/status` and the rung choice.
#[utoipa::path(
    post,
    path = "/trevor/worker/heartbeat",
    tag = "trevor",
    request_body = Heartbeat,
    security(("trevor_worker_key" = [])),
    responses(
        (status = 204, description = "Recorded."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn worker_heartbeat(
    State(state): State<AppState>,
    _key: WorkerKey,
    Json(body): Json<Heartbeat>,
) -> Result<StatusCode, ApiError> {
    worker::heartbeat(&state, &body).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Worker: publish a corpus version's manifest and panel JSON, and (by
/// default) make it the server's active version. The body may be gzip
/// (`Content-Encoding: gzip`): at most 4 MiB as sent and 8 MiB decompressed.
/// Send panels in `activate: false` chunks, then one `activate: true` call
/// that resends `manifest` (with `builtFrom`, `storyOrder`, `eras`) and
/// `gateReport`.
#[utoipa::path(
    post,
    path = "/trevor/versions",
    tag = "trevor",
    request_body = PublishVersion,
    security(("trevor_worker_key" = [])),
    responses(
        (status = 200, description = "Published.", body = PublishOutcome),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 413, description = "Over 4 MiB as sent or 8 MiB decompressed."),
        (status = 415, description = "A `Content-Encoding` other than gzip."),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn publish_version(
    State(state): State<AppState>,
    _key: WorkerKey,
    Json(body): Json<PublishVersion>,
) -> Result<Json<PublishOutcome>, ApiError> {
    Ok(Json(worker::publish(&state, body).await?))
}

/// Refuse a publish body over [`PUBLISH_COMPRESSED_MAX`] as it arrives,
/// before anything decompresses it. Buffers at most that much.
async fn limit_compressed(req: Request, next: Next) -> Response {
    let (parts, body) = req.into_parts();
    let declared = parts
        .headers
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok());
    let too_large = || {
        payload_too_large(&format!(
            "the request body is over {PUBLISH_COMPRESSED_MAX} bytes as sent"
        ))
    };
    if declared.is_some_and(|n| n > PUBLISH_COMPRESSED_MAX) {
        return too_large();
    }
    match axum::body::to_bytes(body, PUBLISH_COMPRESSED_MAX).await {
        Ok(bytes) => {
            next.run(Request::from_parts(parts, Body::from(bytes)))
                .await
        }
        Err(_) => too_large(),
    }
}

/// The publish route with its own body handling (contract section 3):
/// outermost the 4 MiB limit on the body as sent, then gzip decompression,
/// then the 8 MiB limit the JSON extractor applies to the decompressed body.
///
/// Also carries the contract v1.1 routes (`me/jobs`, `popular`,
/// `worker/popular`, `worker/warm`) without those layers, so they need no
/// line in `routes/mod.rs` (which belongs to the route table's owner; they
/// may move there).
pub fn publish_router() -> OpenApiRouter<AppState> {
    let publish = OpenApiRouter::new()
        .routes(routes!(publish_version))
        .layer(DefaultBodyLimit::max(PUBLISH_DECOMPRESSED_MAX))
        .layer(RequestDecompressionLayer::new())
        .layer(axum::middleware::from_fn(limit_compressed));
    OpenApiRouter::new()
        .routes(routes!(my_jobs))
        .routes(routes!(popular))
        .routes(routes!(worker_popular))
        .merge(
            OpenApiRouter::new()
                .routes(routes!(worker_warm))
                .layer(DefaultBodyLimit::max(worker::WARM_BODY_MAX)),
        )
        .merge(publish)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels_are_immutable_only_for_the_active_version() {
        assert_eq!(
            panels_cache_control(Some("v2"), "v2", true),
            PANELS_IMMUTABLE
        );
        assert_eq!(panels_cache_control(Some("v1"), "v2", true), PANELS_SHORT);
        assert_eq!(panels_cache_control(None, "v2", true), PANELS_SHORT);
        // TREVOR_PANELS_IMMUTABLE=0: the old header for everything.
        assert_eq!(panels_cache_control(Some("v2"), "v2", false), PANELS_SHORT);
    }
}
