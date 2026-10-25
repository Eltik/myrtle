use axum::Json;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Response;

use crate::app::cache::keys::CacheKey;
use crate::app::cpu;
use crate::app::error::ApiError;
use crate::app::routes::static_data::json_response;
use crate::app::services;
use crate::app::services::dps::{
    BatchRequest, BatchResponse, CalculateRequest, HpsBatchResponse, RankRequest, RankResponse,
    SupporterInvestment, SupportersResponse,
};
use crate::app::services::dps_retention::RetentionResponse;
use crate::app::state::AppState;
use crate::dps::engine::{DpsResult, HpsResult};

/// Every operator the damage simulator supports, with the configuration
/// options each one accepts.
#[utoipa::path(
    get,
    path = "/dps/operators",
    tag = "dps",
    params(
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The supported operator list. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched the current `ETag`; no body is sent."),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn operators(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let cached = services::dps::list_operators_json(&state).await?;
    Ok(json_response(cached, &headers))
}

/// Hash a request body into a cache key component.
///
/// `DefaultHasher::new()` uses fixed keys, so the value is stable across
/// processes and safe to share through Redis; `RandomState` would not be.
/// Not guaranteed across Rust releases, where a bump only cold-starts the cache.
fn body_hash(body: &impl serde::Serialize) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    serde_json::to_string(body)
        .unwrap_or_default()
        .hash(&mut hasher);
    hasher.finish()
}

/// Simulate damage per second for one operator configuration.
///
/// Pure CPU on an unauthenticated route, so the body is bounds-checked first
/// and the work runs on the blocking pool under admission control.
///
/// Memoised: the result is a pure function of the body and the loaded game
/// data (no clock, no RNG, no user scoping), so two people posting the same
/// configuration get the same three floats. The UI compares configurations by
/// firing a burst of these at once, which was filling the admission queue; a
/// burst of repeats costs one simulation.
///
/// Validation stays AHEAD of the cache read so an out-of-range body is rejected
/// rather than keyed, and the cache read sits ahead of `cpu::run` so a hit never
/// consumes a permit.
#[utoipa::path(
    post,
    path = "/dps/calculate",
    tag = "dps",
    request_body = CalculateRequest,
    responses(
        (status = 200, description = "Skill-window DPS, total damage over the window, and the cycle average.", body = DpsResult),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, description = "The CPU admission queue is full. Retry shortly.")
    )
)]
pub async fn calculate(
    State(state): State<AppState>,
    Json(body): Json<services::dps::CalculateRequest>,
) -> Result<Json<DpsResult>, ApiError> {
    body.validate()?;
    let (_, data) = services::dps::data_identity(&state).await?;
    // `dps3` since phase 2 (2026-10-24): the Muelsyse clone module fallback and
    // the v2 ammo cycle change results, so entries cached before them are not
    // read; the game-data identity keeps a reload from serving old numbers.
    let key = CacheKey::DpsCalculate {
        kind: "dps3",
        data: &data,
        body_hash: body_hash(&body),
    };
    if let Some(hit) = state.cache.get::<DpsResult>(&key).await {
        return Ok(Json(hit));
    }
    // `cpu::run` needs an owned closure. AppState is an Arc, so this is a refcount
    // bump, and `state` stays free for the cache write.
    let compute_state = state.clone();
    let result = cpu::run("dps_calculate", move || {
        services::dps::calculate(&compute_state, body)
    })
    .await??;
    state.cache.set(&key, &result).await;
    Ok(Json(result))
}

/// Every operator the healing simulator supports.
#[utoipa::path(
    get,
    path = "/hps/operators",
    tag = "dps",
    params(
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The supported healer list. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched the current `ETag`; no body is sent."),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn healers(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let cached = services::dps::list_healers_json(&state).await?;
    Ok(json_response(cached, &headers))
}

/// Simulate healing per second for one operator configuration.
///
/// Same admission control and memoisation as `/dps/calculate`.
#[utoipa::path(
    post,
    path = "/hps/calculate",
    tag = "dps",
    request_body = CalculateRequest,
    responses(
        (status = 200, description = "Skill-window HPS, charge-phase HPS, and the cycle average.", body = HpsResult),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, description = "The CPU admission queue is full. Retry shortly.")
    )
)]
pub async fn calculate_hps(
    State(state): State<AppState>,
    Json(body): Json<services::dps::CalculateRequest>,
) -> Result<Json<HpsResult>, ApiError> {
    body.validate()?;
    let (_, data) = services::dps::data_identity(&state).await?;
    // `hps3`: see `dps3` above.
    let key = CacheKey::DpsCalculate {
        kind: "hps3",
        data: &data,
        body_hash: body_hash(&body),
    };
    if let Some(hit) = state.cache.get::<HpsResult>(&key).await {
        return Ok(Json(hit));
    }
    let compute_state = state.clone();
    let result = cpu::run("hps_calculate", move || {
        services::dps::calculate_hps(&compute_state, body)
    })
    .await??;
    state.cache.set(&key, &result).await;
    Ok(Json(result))
}

/// Many DPS calculations in one request, optionally swept across a DEF or RES
/// range.
///
/// Each entry of `requests` is a `/dps/calculate` body, validated the same way.
/// Without `sweep`, `results[i].result` is what `/dps/calculate` returns for
/// `requests[i]`, bit for bit. With `sweep`, `results[i].points` holds one
/// `{x, ...result}` per swept value, `x` replacing the primary `defense` or
/// `res`. A request that cannot be computed gets `error` and does not fail the
/// others. The whole batch is ONE job on the CPU pool, and the response is
/// memoised like a single calculation.
#[utoipa::path(
    post,
    path = "/dps/batch",
    tag = "dps",
    request_body = BatchRequest,
    responses(
        (status = 200, description = "One entry per request, in request order.", body = BatchResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, description = "The CPU admission queue is full. Retry shortly.")
    )
)]
pub async fn batch(
    State(state): State<AppState>,
    Json(body): Json<BatchRequest>,
) -> Result<Json<BatchResponse>, ApiError> {
    body.validate()?;
    let (gd, data) = services::dps::data_identity(&state).await?;
    let key = CacheKey::DpsCalculate {
        kind: "dpsbatch1",
        data: &data,
        body_hash: body_hash(&body),
    };
    if let Some(hit) = state.cache.get::<BatchResponse>(&key).await {
        return Ok(Json(hit));
    }
    let result = cpu::run("dps_batch", move || services::dps::batch_in(&gd, body)).await?;
    state.cache.set(&key, &result).await;
    Ok(Json(result))
}

/// Many HPS calculations in one request. `sweep` is rejected: healing does not
/// read DEF or RES.
#[utoipa::path(
    post,
    path = "/hps/batch",
    tag = "dps",
    request_body = BatchRequest,
    responses(
        (status = 200, description = "One entry per request, in request order.", body = HpsBatchResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, description = "The CPU admission queue is full. Retry shortly.")
    )
)]
pub async fn batch_hps(
    State(state): State<AppState>,
    Json(body): Json<BatchRequest>,
) -> Result<Json<HpsBatchResponse>, ApiError> {
    body.validate()?;
    if body.sweep.is_some() {
        return Err(ApiError::BadRequest(
            "sweep is DPS only: healing does not read DEF or RES".into(),
        ));
    }
    let (gd, data) = services::dps::data_identity(&state).await?;
    let key = CacheKey::DpsCalculate {
        kind: "hpsbatch1",
        data: &data,
        body_hash: body_hash(&body),
    };
    if let Some(hit) = state.cache.get::<HpsBatchResponse>(&key).await {
        return Ok(Json(hit));
    }
    let result = cpu::run("hps_batch", move || services::dps::hps_batch_in(&gd, body)).await?;
    state.cache.set(&key, &result).await;
    Ok(Json(result))
}

/// Every operator, skill and module the simulator supports, ranked under one
/// shared context (enemy, buffs, zones, targets, cycle model, investment
/// preset).
///
/// The full ranking is computed once per context and game-data load, on the
/// CPU pool, and cached; `filters`, `sortBy` and `limit` are applied to the
/// cached rows and never cost a recomputation.
#[utoipa::path(
    post,
    path = "/dps/rank",
    tag = "dps",
    request_body = RankRequest,
    responses(
        (status = 200, description = "Rows sorted descending by `sortBy`.", body = RankResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, description = "The CPU admission queue is full. Retry shortly.")
    )
)]
pub async fn rank(
    State(state): State<AppState>,
    Json(body): Json<RankRequest>,
) -> Result<Json<RankResponse>, ApiError> {
    body.validate()?;
    let (gd, data) = services::dps::data_identity(&state).await?;
    let gd_for_retention = std::sync::Arc::clone(&gd);
    let key = CacheKey::DpsRank {
        data: &data,
        context_hash: body.context_hash(),
        on_page_grid: body.on_page_grid(),
    };
    let computed = if let Some(hit) = state.cache.get::<services::dps::RankComputed>(&key).await {
        hit
    } else {
        let context = body.clone();
        let computed = cpu::run("dps_rank", move || {
            services::dps::rank_rows_in(&gd, &context)
        })
        .await?;
        state.cache.set(&key, &computed).await;
        computed
    };
    let mut response = services::dps::finish_rank(&computed, &body, &data);
    // Reads the finished retention table, or starts it in the background:
    // never waits for it.
    services::dps_retention::attach(&gd_for_retention, &mut response);
    Ok(Json(response))
}

/// The roster retention score of every ranked (operator, skill, module), with
/// the enemy set it runs over.
///
/// `retention = sum_p w_p * dps(DEF_p, RES_p) / (sum_p w_p * dps(0, 0))` over
/// the distinct (DEF, RES) pairs of every (enemy, stage) appearance, `w_p` the
/// appearances at that pair, `dps` the row's `skillDps` at the REFERENCE
/// context (p1Max, no buffs, single target, legacy cycle). Computed once per
/// game-data load in the background: while it runs, `status` is `pending` and
/// `rows` is empty.
#[utoipa::path(
    get,
    path = "/dps/retention",
    tag = "dps",
    responses(
        (status = 200, description = "Per-row scores and the enemy roster, or `status: pending` with no rows while the table is computed.", body = RetentionResponse),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn retention(State(state): State<AppState>) -> Result<Json<RetentionResponse>, ApiError> {
    let (gd, data) = services::dps::data_identity(&state).await?;
    Ok(Json(services::dps_retention::response(&gd, &data)))
}

/// Supporter presets: a curated table of supporter skills and talents, each
/// mapped onto the buff zones `/dps/calculate` takes, with every value read
/// from game data at the requested investment.
#[utoipa::path(
    get,
    path = "/dps/supporters",
    tag = "dps",
    params(
        ("promotion" = Option<i32>, Query, description = "0-2, default 2, clamped to each operator's highest."),
        ("skillLevel" = Option<i32>, Query, description = "1-10 (8-10 = M1-M3), default 10, clamped to the promotion's highest."),
        ("potential" = Option<i32>, Query, description = "1-6, default 1."),
        ("moduleLevel" = Option<i32>, Query, description = "0 (no module, the default) to 3: the level of the module each entry names, for talents a module upgrades.")
    ),
    responses(
        (status = 200, description = "Resolved entries, and the table entries the loaded data cannot resolve.", body = SupportersResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn supporters(
    State(state): State<AppState>,
    Query(q): Query<SupportersQuery>,
) -> Result<Json<SupportersResponse>, ApiError> {
    let d = SupporterInvestment::default();
    let inv = SupporterInvestment {
        promotion: q.promotion.unwrap_or(d.promotion),
        skill_level: q.skill_level.unwrap_or(d.skill_level),
        potential: q.potential.unwrap_or(d.potential),
        module_level: q.module_level.unwrap_or(d.module_level),
    };
    for (field, v, min, max) in [
        ("promotion", inv.promotion, 0, 2),
        ("skillLevel", inv.skill_level, 1, 10),
        ("potential", inv.potential, 1, 6),
        ("moduleLevel", inv.module_level, 0, 3),
    ] {
        if v < min || v > max {
            return Err(ApiError::BadRequest(format!(
                "{field} must be between {min} and {max} (got {v})"
            )));
        }
    }
    let (gd, data) = services::dps::data_identity(&state).await?;
    let key = CacheKey::DpsCalculate {
        kind: "supporters1",
        data: &data,
        body_hash: body_hash(&inv),
    };
    if let Some(hit) = state.cache.get::<SupportersResponse>(&key).await {
        return Ok(Json(hit));
    }
    let result = cpu::offload("dps_supporters", move || {
        services::dps::supporters_in(&gd, inv)
    })
    .await?;
    state.cache.set(&key, &result).await;
    Ok(Json(result))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportersQuery {
    pub promotion: Option<i32>,
    pub skill_level: Option<i32>,
    pub potential: Option<i32>,
    pub module_level: Option<i32>,
}
