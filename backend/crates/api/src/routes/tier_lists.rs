use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderName, header};
use axum::routing::{get, patch, post, put};
use myrtle_app::AppContext;
use myrtle_app::services::tier_lists::entities::EntitySummary;
use myrtle_app::services::tier_lists::{
    self as tier_lists, AddPlacementRequest, CreateFlairRequest, CreateListRequest, FavoriteState,
    GrantRequest, GrantedTierList, MovePlacementRequest, PublishRequest, Tier, TierList,
    TierListDetail, TierListFlair, TierListGrant, TierListPermission, TierListStats,
    TierListVersion, TierPlacement, TierRequest, UpdateListRequest, ViewRecorded, VisibilityState,
};
use myrtle_types::{EntityKind, UserId};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::extract::{self, Caller, ClientIp, MaybeCaller, ServerSel};
use crate::respond::GAMEDATA_CACHE;
use crate::respond::{StatusOk, ok};
use crate::router::ApiRouter;

const SESSION_COOKIE: &str = "mtl_sid";

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/tier-lists", get(list).post(create))
        .route("/tier-lists/details", get(list_details))
        .route("/tier-lists/mine", get(mine))
        .route("/tier-lists/favorites", get(favorites))
        .route("/tier-lists/granted", get(granted))
        .route("/tier-lists/catalogue/{kind}", get(catalogue))
        .route("/tier-lists/{slug}", get(one).put(update).delete(remove))
        .route("/tier-lists/{slug}/tiers", post(create_tier))
        .route(
            "/tier-lists/{slug}/tiers/{tier_id}",
            put(update_tier).delete(delete_tier),
        )
        .route("/tier-lists/{slug}/placements", post(add_placement))
        .route(
            "/tier-lists/{slug}/placements/{kind}/{id}",
            patch(set_placement_description).delete(remove_placement),
        )
        .route(
            "/tier-lists/{slug}/placements/{kind}/{id}/move",
            post(move_placement),
        )
        .route("/tier-lists/{slug}/versions", get(versions))
        .route("/tier-lists/{slug}/publish", post(publish))
        .route(
            "/tier-lists/{slug}/permissions",
            get(permissions).post(grant),
        )
        .route(
            "/tier-lists/{slug}/permissions/{user_id}/{permission}",
            axum::routing::delete(revoke),
        )
        .route("/admin/tier-lists/permissions", get(all_grants))
        .route("/tier-lists/{slug}/view", post(record_view))
        .route("/tier-lists/{slug}/stats", get(stats))
        .route(
            "/tier-lists/{slug}/favorite",
            get(favorite).post(toggle_favorite),
        )
        .route("/tier-lists/{slug}/flair", put(set_flair))
        .route("/tier-lists/{slug}/visibility", put(set_visibility))
        .route("/tier-list-flairs", get(flairs).post(create_flair))
}

async fn list(State(ctx): State<AppContext>) -> ApiResult<Json<Vec<TierList>>> {
    Ok(Json(tier_lists::list(&ctx).await?))
}

#[derive(Deserialize)]
struct ListDetailsQuery {
    limit: Option<i64>,
    author: Option<Uuid>,
}

async fn list_details(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Query(query): Query<ListDetailsQuery>,
) -> ApiResult<Json<Vec<TierListDetail>>> {
    let author = query.author.map(UserId);
    let details = tier_lists::list_details(&ctx, query.limit, author, Some(server)).await?;
    Ok(Json(details))
}

async fn create(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<CreateListRequest>,
) -> ApiResult<Json<TierList>> {
    Ok(Json(tier_lists::create(&ctx, &caller, &body).await?))
}

async fn mine(State(ctx): State<AppContext>, caller: Caller) -> ApiResult<Json<Vec<TierList>>> {
    Ok(Json(tier_lists::mine(&ctx, &caller).await?))
}

async fn favorites(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<Vec<TierList>>> {
    Ok(Json(tier_lists::favorites(&ctx, &caller).await?))
}

async fn catalogue(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(kind): Path<EntityKind>,
) -> ApiResult<([(HeaderName, &'static str); 1], Json<Vec<EntitySummary>>)> {
    let entries = tier_lists::catalogue(&ctx, kind, Some(server)).await?;
    Ok(([(header::CACHE_CONTROL, GAMEDATA_CACHE)], Json(entries)))
}

async fn one(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(slug): Path<String>,
) -> ApiResult<Json<TierListDetail>> {
    Ok(Json(tier_lists::get(&ctx, &slug, Some(server)).await?))
}

async fn update(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
    extract::Json(body): extract::Json<UpdateListRequest>,
) -> ApiResult<Json<TierList>> {
    Ok(Json(tier_lists::update(&ctx, &caller, &slug, &body).await?))
}

async fn remove(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
) -> ApiResult<Json<StatusOk>> {
    tier_lists::delete(&ctx, &caller, &slug).await?;
    Ok(ok())
}

async fn create_tier(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
    extract::Json(body): extract::Json<TierRequest>,
) -> ApiResult<Json<Tier>> {
    Ok(Json(
        tier_lists::create_tier(&ctx, &caller, &slug, &body).await?,
    ))
}

async fn update_tier(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path((slug, tier_id)): Path<(String, Uuid)>,
    extract::Json(body): extract::Json<TierRequest>,
) -> ApiResult<Json<Tier>> {
    let tier = tier_lists::update_tier(&ctx, &caller, &slug, tier_id, &body).await?;
    Ok(Json(tier))
}

async fn delete_tier(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path((slug, tier_id)): Path<(String, Uuid)>,
) -> ApiResult<Json<StatusOk>> {
    tier_lists::delete_tier(&ctx, &caller, &slug, tier_id).await?;
    Ok(ok())
}

async fn add_placement(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
    extract::Json(body): extract::Json<AddPlacementRequest>,
) -> ApiResult<Json<TierPlacement>> {
    let placement = tier_lists::add_placement(&ctx, &caller, &slug, &body).await?;
    Ok(Json(placement))
}

#[derive(Deserialize)]
struct UpdateDescriptionRequest {
    description: Option<String>,
}

async fn set_placement_description(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path((slug, kind, id)): Path<(String, EntityKind, String)>,
    extract::Json(body): extract::Json<UpdateDescriptionRequest>,
) -> ApiResult<Json<TierPlacement>> {
    let description = body.description.as_deref();
    let placement =
        tier_lists::set_placement_description(&ctx, &caller, &slug, kind, &id, description).await?;
    Ok(Json(placement))
}

async fn remove_placement(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path((slug, kind, id)): Path<(String, EntityKind, String)>,
) -> ApiResult<Json<StatusOk>> {
    tier_lists::remove_placement(&ctx, &caller, &slug, kind, &id).await?;
    Ok(ok())
}

async fn move_placement(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path((slug, kind, id)): Path<(String, EntityKind, String)>,
    extract::Json(body): extract::Json<MovePlacementRequest>,
) -> ApiResult<Json<TierPlacement>> {
    let placement = tier_lists::move_placement(&ctx, &caller, &slug, kind, &id, &body).await?;
    Ok(Json(placement))
}

async fn versions(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(slug): Path<String>,
) -> ApiResult<Json<Vec<TierListVersion>>> {
    Ok(Json(tier_lists::versions(&ctx, &slug, Some(server)).await?))
}

async fn publish(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
    extract::Json(body): extract::Json<PublishRequest>,
) -> ApiResult<Json<TierListVersion>> {
    Ok(Json(
        tier_lists::publish(&ctx, &caller, &slug, &body).await?,
    ))
}

async fn permissions(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
) -> ApiResult<Json<Vec<TierListPermission>>> {
    Ok(Json(tier_lists::permissions(&ctx, &caller, &slug).await?))
}

async fn grant(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
    extract::Json(body): extract::Json<GrantRequest>,
) -> ApiResult<Json<StatusOk>> {
    tier_lists::grant(&ctx, &caller, &slug, &body).await?;
    Ok(ok())
}

async fn revoke(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path((slug, user_id, permission)): Path<(String, Uuid, String)>,
) -> ApiResult<Json<StatusOk>> {
    tier_lists::revoke(&ctx, &caller, &slug, UserId(user_id), &permission).await?;
    Ok(ok())
}

#[derive(Deserialize)]
struct AllGrantsQuery {
    user_id: Option<Uuid>,
}

async fn all_grants(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(query): Query<AllGrantsQuery>,
) -> ApiResult<Json<Vec<TierListGrant>>> {
    let user = query.user_id.map(UserId);
    Ok(Json(tier_lists::all_grants(&ctx, &caller, user).await?))
}

async fn granted(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<Vec<GrantedTierList>>> {
    Ok(Json(tier_lists::granted(&ctx, &caller).await?))
}

async fn record_view(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> ApiResult<Json<ViewRecorded>> {
    let well_formed = |v: &&str| v.len() == 64 && v.bytes().all(|b| b.is_ascii_hexdigit());
    let session = headers
        .get("x-session-id")
        .and_then(|v| v.to_str().ok())
        .filter(well_formed)
        .or_else(|| {
            headers
                .get(header::COOKIE)?
                .to_str()
                .ok()?
                .split(';')
                .filter_map(|part| part.trim().strip_prefix(SESSION_COOKIE)?.strip_prefix('='))
                .find(well_formed)
        });
    let ip = ip.map(|ip| ip.to_string());
    let recorded = tier_lists::record_view(&ctx, &caller, &slug, session, ip.as_deref()).await?;
    Ok(Json(recorded))
}

async fn stats(
    State(ctx): State<AppContext>,
    Path(slug): Path<String>,
) -> ApiResult<Json<TierListStats>> {
    Ok(Json(tier_lists::stats(&ctx, &slug).await?))
}

async fn favorite(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
) -> ApiResult<Json<FavoriteState>> {
    Ok(Json(tier_lists::favorite(&ctx, &caller, &slug).await?))
}

async fn toggle_favorite(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
) -> ApiResult<Json<FavoriteState>> {
    Ok(Json(
        tier_lists::toggle_favorite(&ctx, &caller, &slug).await?,
    ))
}

#[derive(Deserialize)]
struct SetFlairRequest {
    flair_id: Option<i16>,
}

async fn set_flair(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
    extract::Json(body): extract::Json<SetFlairRequest>,
) -> ApiResult<Json<StatusOk>> {
    tier_lists::set_flair(&ctx, &caller, &slug, body.flair_id).await?;
    Ok(ok())
}

#[derive(Deserialize)]
struct SetVisibilityRequest {
    is_listed: bool,
}

async fn set_visibility(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
    extract::Json(body): extract::Json<SetVisibilityRequest>,
) -> ApiResult<Json<VisibilityState>> {
    let state = tier_lists::set_visibility(&ctx, &caller, &slug, body.is_listed).await?;
    Ok(Json(state))
}

async fn flairs(State(ctx): State<AppContext>) -> ApiResult<Json<Vec<TierListFlair>>> {
    Ok(Json(tier_lists::flairs(&ctx).await?))
}

async fn create_flair(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<CreateFlairRequest>,
) -> ApiResult<Json<TierListFlair>> {
    Ok(Json(tier_lists::create_flair(&ctx, &caller, &body).await?))
}
