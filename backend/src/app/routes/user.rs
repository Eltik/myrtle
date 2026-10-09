use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::extractors::auth::{AuthUser, MaybeAuthUser};
use crate::app::routes::{
    StatusOk, ensure_tab_visible, resolve_public_profile, resolve_uid_for_tab,
};
use crate::app::{error::ApiError, services, state::AppState};
use crate::core::hypergryph::constants::Server;
use crate::database::models::profile_layout::ProfileTabId;
use crate::database::models::score::UserScore;
use crate::database::models::user::{UserCheckin, UserProfile};
use crate::database::queries::score::get_score_by_uid;
use crate::database::queries::users::{find_by_id, get_checkin_by_uid, update_role};

#[derive(Deserialize)]
pub struct GetUserParams {
    pub uid: String,
}

/// A player's profile. `UserProfile` carries currency, sanity, subscription
/// expiry, last-online time and the account's role, so the `uid` goes through
/// the shared privacy gate: the caller's own profile or a public one, 403
/// otherwise.
///
/// Never gated by tab: the row feeds the page header, which every visible tab
/// sits under. A visitor gets the layout with the private tabs left out, and
/// no `total_score` or `grade` while the Score tab is private, as
/// `/get-user-score` refuses that tab's data to them.
#[utoipa::path(
    get,
    path = "/get-user",
    tag = "player",
    params(
        ("uid" = String, Query, description = "Player to read.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The profile.", body = UserProfile),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_user(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<UserProfile>, ApiError> {
    let (gate, own) = resolve_public_profile(&state, &auth, &params.uid).await?;
    let mut profile = services::user::get_user(&state, &gate.uid).await?;
    // The layout decides what a visitor may see, so it comes from the gate's
    // fresh read, not the row cached for up to 10 minutes: a tab made private
    // since the cache filled is private now.
    profile.profile_layout = gate.profile_layout;
    // Projected per caller after the cache read: the cached row is the owner's.
    if !own {
        profile.hide_private_tabs();
        // A block whose grid, tier list or plan is gone is no block to a
        // visitor, so an emptied showcase is no tab either.
        if let Some(layout) = profile.profile_layout.as_mut() {
            services::showcase::prune_for_visitor(&state, gate.id, layout).await?;
        }
    }
    Ok(Json(profile))
}

#[derive(Deserialize)]
pub struct GetUserShowcaseParams {
    pub uid: String,
    pub server: Option<Server>,
}

/// A player's Showcase tab: their blocks, each favourite resolved against
/// `server`'s game data (then every other loaded server, as grid cells are).
///
/// Gated like every tab: a private profile or a private Showcase tab is 403
/// to a visitor. The owner gets every block, those whose grid, tier list or
/// plan is gone marked `removed`; a visitor gets only the blocks that still
/// resolve, and no plan block while the Plans tab is private. A player who
/// never set a layout has an empty showcase.
#[utoipa::path(
    get,
    path = "/get-user-showcase",
    tag = "player",
    params(
        ("uid" = String, Query, description = "Player to read."),
        ("server" = Option<String>, Query, description = "Game server whose data names the favourites: `en`, `jp`, `kr`, `cn`, `tw` or `bili`. Defaults to the deployment's default server.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The showcase.", body = services::showcase::ShowcaseView),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_user_showcase(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserShowcaseParams>,
) -> Result<Json<services::showcase::ShowcaseView>, ApiError> {
    let (profile, own) = resolve_public_profile(&state, &auth, &params.uid).await?;
    ensure_tab_visible(&profile, own, ProfileTabId::Showcase)?;
    let server = params.server.unwrap_or(state.default_server);
    let view = services::showcase::view(
        &state,
        profile.id,
        profile.profile_layout.as_ref(),
        own,
        server,
    )
    .await?;
    Ok(Json(view))
}

/// A player's graded score across every dimension.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/get-user-score",
    tag = "player",
    params(
        ("uid" = String, Query, description = "Player to read.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The stored score row, or null when the player has never been graded.", body = Option<crate::database::models::score::UserScore>),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_user_score(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<Option<UserScore>>, ApiError> {
    // Typed row, not a `serde_json::Value` round-trip: same bytes on the wire,
    // but the type is the schema and the generated TS.
    let uid = resolve_uid_for_tab(&state, &auth, Some(&params.uid), ProfileTabId::Score).await?;
    let score = get_score_by_uid(&state.db, &uid).await?;
    Ok(Json(score))
}

/// A player's monthly sign-in state.
///
/// Null when the player has no stored check-in row.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/get-user-checkin",
    tag = "player",
    params(
        ("uid" = String, Query, description = "Player to read.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The sign-in state, or null.", body = Option<UserCheckin>),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_user_checkin(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<Option<UserCheckin>>, ApiError> {
    // `null` until the user has synced once.
    let uid = resolve_uid_for_tab(&state, &auth, Some(&params.uid), ProfileTabId::Stats).await?;
    let checkin = get_checkin_by_uid(&state.db, &uid).await?;
    Ok(Json(checkin))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SetRoleRequest {
    pub role: String,
}

/// `PUT /admin/users/{user_id}/role`
///
/// The admin panel could display a role but never assign one; this is that
/// route (the Translator grant needs it).
///
/// Super-admin only, and it refuses to change a super-admin's own row: an
/// accidental self-demotion here would need the CLI to undo.
///
/// The role reaches `AuthUser` from the JWT, so the target keeps their old
/// privileges until their token refreshes. Per-locale translation grants are
/// read from the database per request and take effect immediately.
#[utoipa::path(
    put,
    path = "/admin/users/{user_id}/role",
    tag = "admin",
    params(
        ("user_id" = String, Path, description = "Target account id (UUID).")
    ),
    request_body = SetRoleRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The role was set.", body = StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn set_user_role(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    Json(body): Json<SetRoleRequest>,
) -> Result<Json<StatusOk>, ApiError> {
    if !auth.role.is_super_admin() {
        return Err(ApiError::Forbidden);
    }

    let role: crate::core::auth::permissions::GlobalRole =
        body.role.parse().map_err(ApiError::BadRequest)?;

    let target = find_by_id(&state.db, user_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if auth.user_uuid()? == user_id {
        return Err(ApiError::BadRequest(
            "refusing to change your own role; use the manage_permissions CLI".to_owned(),
        ));
    }

    if target.role == role.to_string() {
        return Ok(crate::app::routes::ok_status());
    }

    update_role(&state.db, user_id, &role.to_string()).await?;
    state
        .cache
        .invalidate_by_prefix(&format!("user:{}", target.uid))
        .await;

    tracing::info!(
        actor = %auth.user_id,
        target = %user_id,
        from = %target.role,
        to = %role,
        "role changed"
    );

    Ok(crate::app::routes::ok_status())
}

#[derive(Deserialize)]
pub struct AdminUsersParams {
    /// Nickname substring or UID prefix.
    pub q: Option<String>,
    /// `all` (default), `staff`, `translators` or `players`.
    pub role: Option<String>,
    pub server: Option<Server>,
    #[serde(flatten)]
    pub pagination: crate::app::extractors::pagination::Pagination,
}

/// The page size for [`list_users`] when the caller names none, and its cap.
const ADMIN_USERS_DEFAULT_LIMIT: u32 = 50;
const ADMIN_USERS_MAX_LIMIT: u32 = 200;

/// One page of the admin people list, and how many accounts the filters admit.
#[derive(serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AdminUsersPage {
    pub users: Vec<crate::database::models::user::AdminUserEntry>,
    #[ts(type = "number")]
    pub total: i64,
}

/// Every account, private profiles included, for the admin panel's People tab.
///
/// Tier list admin or super-admin. Not cached: a role change must show on the
/// next read.
#[utoipa::path(
    get,
    path = "/admin/users",
    tag = "admin",
    params(
        ("q" = Option<String>, Query, description = "Nickname substring (case-insensitive) or UID prefix. An exact UID match sorts first."),
        ("role" = Option<String>, Query, description = "`all` (default), `staff` (super-admins, tier list admins and editors), `translators` or `players` (no panel role)."),
        ("server" = Option<String>, Query, description = "Server code: `en`, `jp`, `kr`, `cn`, `bili` or `tw`. Absent means every server."),
        ("limit" = Option<u32>, Query, description = "Page size. Defaults to 50, capped at 200."),
        ("offset" = Option<u32>, Query, description = "Rows to skip. Defaults to 0.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "One page of accounts, most privileged roles first, then by nickname.", body = AdminUsersPage),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<AdminUsersParams>,
) -> Result<Json<AdminUsersPage>, ApiError> {
    use crate::database::queries::admin_users::{AdminUserRole, AdminUserSearch};

    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let role = match params
        .role
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        None => AdminUserRole::All,
        Some(token) => AdminUserRole::parse(token).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "role must be `all`, `staff`, `translators` or `players`, got `{token}`"
            ))
        })?,
    };
    let q = params.q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let search = AdminUserSearch {
        q,
        role,
        server: params.server,
    };
    let limit = params
        .pagination
        .limit
        .unwrap_or(ADMIN_USERS_DEFAULT_LIMIT)
        .clamp(1, ADMIN_USERS_MAX_LIMIT);
    let offset = params.pagination.offset();
    let (users, total) = tokio::try_join!(
        search.fetch_page(&state.db, i64::from(limit), i64::from(offset)),
        search.count(&state.db),
    )?;
    Ok(Json(AdminUsersPage { users, total }))
}
