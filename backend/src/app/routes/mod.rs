use axum::Json;
use ts_rs::TS;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::state::AppState;
use crate::database::models::user::UserProfile;
use crate::database::queries::users::{find_by_id, find_by_uid};

/// The `{"status":"ok"}` success body for endpoints that return no payload.
///
/// The type IS the documented schema and the serialized body, so there is no
/// second declaration to keep in agreement. It reached the wire as an untyped
/// `serde_json::json!` before, which meant the `OpenAPI` document described it
/// from a hand-written copy.
#[derive(serde::Serialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StatusOk {
    #[schema(example = "ok")]
    pub status: String,
}

pub fn ok_status() -> Json<StatusOk> {
    Json(StatusOk {
        status: "ok".to_owned(),
    })
}

/// The privacy gate for a `uid` naming someone other than the caller.
///
/// This is the single place that decides whether one player may see another's
/// data: a named profile is readable when it is the caller's own or it is
/// marked public, and 403 otherwise. Any handler that accepts a `uid` must
/// reach it through [`resolve_user_id`] or [`resolve_uid`] rather than reading
/// the parameter directly.
///
/// The gate belongs in the handler, ahead of any cache read. Gating inside a
/// service instead leaves a cache hit able to answer before the check runs.
async fn resolve_public_profile(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid: &str,
) -> Result<UserProfile, ApiError> {
    let profile = find_by_uid(&state.db, uid)
        .await?
        .ok_or(ApiError::NotFound)?;

    let is_own = auth
        .0
        .as_ref()
        .and_then(|a| a.user_id.parse::<Uuid>().ok())
        .is_some_and(|id| id == profile.id);

    if !is_own && profile.public_profile != Some(true) {
        return Err(ApiError::Forbidden);
    }

    Ok(profile)
}

/// Resolve the target `user_id` from either a `uid` query param (public access)
/// or the authenticated user's token (private access). A `uid` lookup is only
/// allowed for the caller's own profile or a profile marked public.
pub(crate) async fn resolve_user_id(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
) -> Result<Uuid, ApiError> {
    match uid_param {
        Some(uid) => Ok(resolve_public_profile(state, auth, uid).await?.id),
        // The token already carries the id, so the self case costs no query.
        None => auth.0.as_ref().ok_or(ApiError::Unauthorized)?.user_uuid(),
    }
}

/// As [`resolve_user_id`], for the endpoints keyed on the game-account `uid`
/// string rather than the internal row id.
pub(crate) async fn resolve_uid(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
) -> Result<String, ApiError> {
    if let Some(uid) = uid_param {
        return Ok(resolve_public_profile(state, auth, uid).await?.uid);
    }

    // No uid given: the caller's own, which needs a lookup because the token
    // carries the row id rather than the game-account uid.
    let auth = auth.0.as_ref().ok_or(ApiError::Unauthorized)?;
    let user_uuid: Uuid = auth.user_uuid()?;
    Ok(find_by_id(&state.db, user_uuid)
        .await?
        .ok_or(ApiError::Unauthorized)?
        .uid)
}

pub mod account;
pub mod assets;
pub mod auth;
pub mod base;
pub mod chibis;
pub mod dps;
pub mod enemies;
pub mod gacha;
pub mod health;
pub mod i18n;
pub mod improvements;
pub mod inventory;
pub mod item_leaderboard;
pub mod leaderboard;
pub mod level;
pub mod operator_notes;
pub mod operators;
pub mod planner;
pub mod release;
pub mod roster;
pub mod search;
pub mod skins;
pub mod social;
pub mod stages;
pub mod static_data;
pub mod stats;
pub mod story;
pub mod story_progress;
pub mod tier_lists;
pub mod user;

/// The `/api` route tree.
///
/// Every handler here is reached through `.routes(routes!(..))` and carries a
/// `#[utoipa::path]` annotation: the path and method come from that
/// annotation, so the route and its documentation are one declaration rather
/// than two that have to be kept in agreement.
///
/// `.route(..)` still works and would still serve, but a route added that way
/// is invisible to the `OpenAPI` document, so `tests/openapi_snapshot_test.rs`
/// fails on it. See [`crate::app::openapi`] for what to write instead.
///
/// The builder chain keeps one `OpenApiRouter` temporary per `.routes(..)` call
/// live in a debug frame (~700 KB); it runs once at startup, on the main
/// thread's 8 MB stack, so the frame is not a risk.
#[allow(clippy::large_stack_frames)]
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(health::health))
        .routes(routes!(user::get_user))
        .routes(routes!(user::get_user_score))
        .routes(routes!(user::get_user_checkin))
        .routes(routes!(leaderboard::leaderboard))
        .routes(routes!(leaderboard::top_movers))
        .routes(routes!(leaderboard::distribution))
        .routes(routes!(leaderboard::standing))
        .routes(routes!(leaderboard::score_history))
        .routes(routes!(item_leaderboard::item_leaderboard))
        .routes(routes!(item_leaderboard::item_catalog))
        .routes(routes!(item_leaderboard::item_standing))
        .routes(routes!(search::search))
        .routes(routes!(static_data::get_static))
        .routes(routes!(level::get_level_map))
        .routes(routes!(assets::avatar))
        .routes(routes!(assets::portrait))
        .routes(routes!(assets::skill_icon))
        .routes(routes!(assets::module_icon))
        .routes(routes!(assets::module_big))
        .routes(routes!(assets::enemy_icon))
        .routes(routes!(assets::item_icon))
        .routes(routes!(assets::medal_icon))
        .routes(routes!(assets::charart))
        .routes(routes!(assets::skin_portrait))
        .routes(routes!(assets::banner_image))
        .routes(routes!(assets::event_image))
        .routes(routes!(assets::brand_kv))
        .routes(routes!(assets::brand_logo))
        .routes(routes!(assets::generic))
        .routes(routes!(auth::send_code))
        .routes(routes!(auth::login))
        .routes(routes!(auth::login_bilibili))
        .routes(routes!(auth::send_bilibili_sms))
        .routes(routes!(auth::login_bilibili_sms))
        .routes(routes!(auth::send_code_cn))
        .routes(routes!(auth::login_cn))
        .routes(routes!(auth::verify))
        .routes(routes!(auth::update_settings))
        .routes(routes!(auth::disconnect))
        .routes(routes!(release::events))
        .routes(routes!(release::banners))
        .routes(routes!(release::skins))
        .routes(routes!(release::lag))
        .routes(routes!(release::get_plan, release::put_plan))
        .routes(routes!(release::list_overrides, release::put_override))
        .routes(routes!(release::delete_override))
        .routes(routes!(gacha::history))
        .routes(routes!(gacha::history_by_char))
        .routes(routes!(gacha::stored_records))
        .routes(routes!(gacha::stats))
        .routes(routes!(gacha::get_settings, gacha::update_settings))
        .routes(routes!(auth::refresh))
        .routes(routes!(roster::get_roster))
        .routes(routes!(roster::get_operator))
        .routes(routes!(stages::get_stage_clears))
        .routes(routes!(enemies::get_encountered_enemies))
        .routes(routes!(enemies::get_community_average))
        .routes(routes!(improvements::get_user_improvements))
        .routes(routes!(account::get_max_level_cost))
        .routes(routes!(
            story_progress::get_story_progress,
            story_progress::put_story_progress
        ))
        .routes(routes!(story_progress::import_story_progress))
        .routes(routes!(base::get_catalog))
        .routes(routes!(base::get_layout))
        .routes(routes!(base::evaluate_layout))
        .routes(routes!(base::optimize_layout))
        .routes(routes!(base::rotation_plan))
        .routes(routes!(base::put_facts))
        .routes(routes!(roster::get_supports))
        .routes(routes!(inventory::get_inventory))
        .routes(routes!(skins::get_owned_skins))
        .routes(routes!(skins::get_skin_popularity))
        .routes(routes!(gacha::fetch))
        .routes(routes!(gacha::global_stats))
        .routes(routes!(gacha::enhanced_stats))
        .routes(routes!(gacha::per_banner_stats))
        .routes(routes!(stats::stats))
        .routes(routes!(stats::admin_stats))
        .routes(routes!(user::set_user_role))
        .routes(routes!(operators::index))
        .routes(routes!(operators::ownership))
        .routes(routes!(stages::stage_detail))
        .routes(routes!(enemies::enemy_detail))
        .routes(routes!(enemies::enemy_stages))
        .routes(routes!(chibis::chibi_detail))
        .routes(routes!(skins::skins_index))
        .routes(routes!(dps::operators))
        .routes(routes!(dps::calculate))
        .routes(routes!(dps::healers))
        .routes(routes!(dps::calculate_hps))
        .routes(routes!(operator_notes::list))
        .routes(routes!(operator_notes::get, operator_notes::update))
        .routes(routes!(operator_notes::audit_log))
        .routes(routes!(operator_notes::global_audit_log))
        .routes(routes!(social::get_friends))
        .routes(routes!(social::search_players))
        .routes(routes!(planner::list))
        .routes(routes!(planner::list_public))
        .routes(routes!(planner::upsert, planner::delete))
        .routes(routes!(planner::create_group))
        .routes(routes!(planner::update_group, planner::delete_group))
        .routes(routes!(operators::upcoming))
        .routes(routes!(operators::upcoming_srv))
        .routes(routes!(operators::detail))
        .routes(routes!(operators::build_stats))
        .routes(routes!(operators::voices_detail))
        .routes(routes!(operators::voices_detail_srv))
        .routes(routes!(story::community))
        .routes(routes!(story::index))
        .routes(routes!(story::illustrations))
        .routes(routes!(story::archive))
        .routes(routes!(story::detail))
        .routes(routes!(story::community_srv))
        .routes(routes!(story::index_srv))
        .routes(routes!(story::illustrations_srv))
        .routes(routes!(story::archive_srv))
        .routes(routes!(story::detail_srv))
        .routes(routes!(operators::skins_detail))
        .routes(routes!(operators::skins_detail_srv))
        .routes(routes!(operators::index_srv))
        .routes(routes!(operators::ownership_srv))
        .routes(routes!(operators::detail_srv))
        .routes(routes!(operators::build_stats_srv))
        .routes(routes!(stages::stage_detail_srv))
        .routes(routes!(enemies::enemy_detail_srv))
        .routes(routes!(enemies::enemy_stages_srv))
        .routes(routes!(chibis::chibi_detail_srv))
        .routes(routes!(skins::skins_index_srv))
        .routes(routes!(static_data::get_static_srv))
        .routes(routes!(level::get_level_map_srv))
        .routes(routes!(assets::avatar_srv))
        .routes(routes!(assets::portrait_srv))
        .routes(routes!(assets::skill_icon_srv))
        .routes(routes!(assets::module_icon_srv))
        .routes(routes!(assets::module_big_srv))
        .routes(routes!(assets::enemy_icon_srv))
        .routes(routes!(assets::item_icon_srv))
        .routes(routes!(assets::medal_icon_srv))
        .routes(routes!(assets::charart_srv))
        .routes(routes!(assets::skin_portrait_srv))
        .routes(routes!(assets::banner_image_srv))
        .routes(routes!(assets::event_image_srv))
        .routes(routes!(assets::brand_kv_srv))
        .routes(routes!(assets::brand_logo_srv))
        .routes(routes!(assets::generic_srv))
        .merge(tier_lists::router())
        .merge(i18n::router())
}
