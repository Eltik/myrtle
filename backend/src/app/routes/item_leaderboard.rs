use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_uid;
use crate::app::services::item_leaderboard::{
    ItemLeaderboardPage, get_item_catalog, get_item_leaderboard, get_item_standing,
};
use crate::database::queries::item_leaderboard::is_valid_item_id;
use crate::{
    app::{error::ApiError, extractors::pagination::Pagination, state::AppState},
    database::models::item_leaderboard::{ItemHoldingSummary, ItemStanding},
};

fn validated_item_id(item: &str) -> Result<&str, ApiError> {
    let item = item.trim();
    if is_valid_item_id(item) {
        Ok(item)
    } else {
        Err(ApiError::BadRequest(
            "item must be a game item id: 1 to 50 characters of [A-Za-z0-9_]".into(),
        ))
    }
}

#[derive(Deserialize)]
pub struct ItemLeaderboardParams {
    /// Game item id. Currencies use their item id too: `4001` LMD, `4002`
    /// Originite Prime, `4003` Orundum, `4004`/`4005` certificates, `7001`
    /// recruitment permits, `7003`/`7004` headhunting permits, `SOCIAL_PT`
    /// credits, `6001` drill plans.
    pub item: String,
    pub server: Option<String>,
    /// Free-text filter applied to nickname / uid via ILIKE. Rows keep the
    /// rank they hold among all visible holders.
    pub q: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

/// One page of the players holding the most of one item.
///
/// Currencies and inventory items share one key space, the game's item id, so
/// `item=4002` ranks Originite Prime and `item=30011` ranks Orirock the same
/// way. Only public profiles are ranked; ties share a rank.
#[utoipa::path(
    get,
    path = "/leaderboard/items",
    tag = "leaderboard",
    params(
        ("item" = String, Query, description = "Game item id to rank by, e.g. `4002` for Originite Prime or `4003` for Orundum."),
        ("server" = Option<String>, Query, description = "Restrict the page to one game server."),
        ("q" = Option<String>, Query, description = "Free-text filter applied to nickname and uid. Matching rows keep their rank among all visible holders."),
        ("limit" = Option<u32>, Query, description = "Page size. Defaults to 20, capped at 100."),
        ("offset" = Option<u32>, Query, description = "Rows to skip. Defaults to 0.")
    ),
    responses(
        (status = 200, description = "A page of holders ranked by quantity, and how many visible players hold the item at all.", body = ItemLeaderboardPage),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn item_leaderboard(
    State(state): State<AppState>,
    Query(params): Query<ItemLeaderboardParams>,
) -> Result<Json<ItemLeaderboardPage>, ApiError> {
    let item = validated_item_id(&params.item)?;
    let q = params.q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let page = get_item_leaderboard(
        &state,
        item,
        params.server.as_deref(),
        q,
        params.pagination.limit(),
        params.pagination.offset(),
    )
    .await?;
    Ok(Json(page))
}

#[derive(Deserialize)]
pub struct ItemCatalogParams {
    pub server: Option<String>,
}

/// Every item at least one visible player holds, with the holder count and
/// the largest single holding. Ordered by holders, most first.
#[utoipa::path(
    get,
    path = "/leaderboard/items/catalog",
    tag = "leaderboard",
    params(
        ("server" = Option<String>, Query, description = "Count holders on one game server only.")
    ),
    responses(
        (status = 200, description = "Holder count and top holding per item.", body = Vec<ItemHoldingSummary>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn item_catalog(
    State(state): State<AppState>,
    Query(params): Query<ItemCatalogParams>,
) -> Result<Json<Vec<ItemHoldingSummary>>, ApiError> {
    let rows = get_item_catalog(&state, params.server.as_deref()).await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct ItemStandingParams {
    pub item: String,
    pub uid: String,
    pub server: String,
}

/// One player's rank among the holders of one item, globally and on their
/// own server.
///
/// Subject to the same privacy gate as every other by-uid endpoint: a private
/// profile is visible only to its owner. 404 when the player holds none of
/// the item.
#[utoipa::path(
    get,
    path = "/leaderboard/items/standing",
    tag = "leaderboard",
    params(
        ("item" = String, Query, description = "Game item id."),
        ("uid" = String, Query, description = "The player to locate."),
        ("server" = String, Query, description = "The player's game server.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The player's quantity and rank, globally and on their server.", body = ItemStanding),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn item_standing(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<ItemStandingParams>,
) -> Result<Json<ItemStanding>, ApiError> {
    let item = validated_item_id(&params.item)?;
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let standing = get_item_standing(&state, item, &uid, &params.server).await?;
    Ok(Json(standing))
}
