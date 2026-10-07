use axum::Json;
use ts_rs::TS;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::state::AppState;
use crate::database::models::profile_layout::ProfileTabId;
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

/// Whether the signed-in caller is `profile`'s owner.
fn is_own(auth: &MaybeAuthUser, profile: &UserProfile) -> bool {
    auth.0
        .as_ref()
        .and_then(|a| a.user_id.parse::<Uuid>().ok())
        .is_some_and(|id| id == profile.id)
}

/// The privacy gate for a `uid` naming someone other than the caller: readable
/// when it's the caller's own or marked public, 403 otherwise. Every `uid`-taking
/// handler goes through [`resolve_user_id`] or a `*_for_tab` variant, never the raw param.
/// Returns the profile and whether the caller owns it.
///
/// Gate in the handler, ahead of any cache read; inside a service, a cache hit
/// can answer before the check runs.
async fn resolve_public_profile(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid: &str,
) -> Result<(UserProfile, bool), ApiError> {
    let profile = find_by_uid(&state.db, uid)
        .await?
        .ok_or(ApiError::NotFound)?;
    ensure_profile_readable(auth, profile)
}

/// The public-profile half of [`resolve_public_profile`], for a handler that
/// found the profile by other means.
pub(crate) fn ensure_profile_readable(
    auth: &MaybeAuthUser,
    profile: UserProfile,
) -> Result<(UserProfile, bool), ApiError> {
    let own = is_own(auth, &profile);
    if !own && profile.public_profile != Some(true) {
        return Err(ApiError::Forbidden);
    }
    Ok((profile, own))
}

/// The per-tab gate: 403 when a visitor asks for the data behind a tab the
/// owner made private. The owner always passes, and so does every viewer of a
/// profile that never set a layout. Runs after the public-profile gate, so a
/// private profile still answers 403 before any tab is considered.
pub fn ensure_tab_visible(
    profile: &UserProfile,
    is_own: bool,
    tab: ProfileTabId,
) -> Result<(), ApiError> {
    if is_own {
        return Ok(());
    }
    match &profile.profile_layout {
        Some(layout) if !layout.is_visible(tab) => Err(ApiError::Forbidden),
        _ => Ok(()),
    }
}

/// [`resolve_public_profile`], then the per-tab gate for each of `tabs`.
async fn resolve_profile_for_tabs(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid: &str,
    tabs: &[ProfileTabId],
) -> Result<UserProfile, ApiError> {
    let (profile, own) = resolve_public_profile(state, auth, uid).await?;
    for &tab in tabs {
        ensure_tab_visible(&profile, own, tab)?;
    }
    Ok(profile)
}

/// Target `user_id` from the `uid` param (own or public profiles only) or from the
/// caller's token.
pub(crate) async fn resolve_user_id(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
) -> Result<Uuid, ApiError> {
    resolve_user_id_for_tabs(state, auth, uid_param, &[]).await
}

/// As [`resolve_user_id`], for data that belongs to one profile tab: a visitor
/// also needs that tab to be visible ([`ensure_tab_visible`]).
pub(crate) async fn resolve_user_id_for_tab(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
    tab: ProfileTabId,
) -> Result<Uuid, ApiError> {
    resolve_user_id_for_tabs(state, auth, uid_param, &[tab]).await
}

/// As [`resolve_user_id_for_tab`], for data more than one tab shows: a visitor
/// needs every one of `tabs` visible.
pub(crate) async fn resolve_user_id_for_tabs(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
    tabs: &[ProfileTabId],
) -> Result<Uuid, ApiError> {
    match uid_param {
        Some(uid) => Ok(resolve_profile_for_tabs(state, auth, uid, tabs).await?.id),
        // The token already carries the id, so the self case costs no query.
        None => auth.0.as_ref().ok_or(ApiError::Unauthorized)?.user_uuid(),
    }
}

/// As [`resolve_user_id_for_tab`], for the endpoints keyed on the game-account
/// `uid` string rather than the internal row id. Every such endpoint reads one
/// tab's data, so there is no tab-less variant.
pub(crate) async fn resolve_uid_for_tab(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
    tab: ProfileTabId,
) -> Result<String, ApiError> {
    match uid_param {
        Some(uid) => Ok(resolve_profile_for_tabs(state, auth, uid, &[tab])
            .await?
            .uid),
        None => own_uid(state, auth).await,
    }
}

/// No uid given: the caller's own, which needs a lookup because the token
/// carries the row id rather than the game-account uid.
async fn own_uid(state: &AppState, auth: &MaybeAuthUser) -> Result<String, ApiError> {
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
pub mod grids;
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
pub mod trevor;
pub mod user;

/// The `/api` route tree.
///
/// Every handler goes through `.routes(routes!(..))` with a `#[utoipa::path]`, so
/// route and docs are one declaration. A `.route(..)` still serves but is missing
/// from the `OpenAPI` document and fails `tests/openapi_snapshot_test.rs`; see
/// [`crate::app::openapi`].
///
/// The builder chain holds one `OpenApiRouter` temporary per `.routes(..)` in a
/// debug frame (~700 KB); it runs once at startup on the main thread's 8 MB stack.
#[allow(clippy::large_stack_frames)]
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(health::health))
        .routes(routes!(user::get_user))
        .routes(routes!(user::get_user_score))
        .routes(routes!(user::get_user_checkin))
        .routes(routes!(user::get_user_showcase))
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
        .routes(routes!(assets::story_sprite_thumb))
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
        .routes(routes!(operators::recruitment))
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
        .routes(routes!(planner::delete_many))
        .routes(routes!(planner::list_presets))
        .routes(routes!(planner::upsert_preset))
        .routes(routes!(planner::delete_preset))
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
        .routes(routes!(story::sprites))
        .routes(routes!(story::sprite_detail))
        .routes(routes!(story::sprite_variant_thumb))
        .routes(routes!(story::gallery))
        .routes(routes!(story::gallery_picture))
        .routes(routes!(story::art_gallery))
        .routes(routes!(story::art_picture))
        .routes(routes!(story::detail))
        .routes(routes!(story::community_srv))
        .routes(routes!(trevor::ask))
        .routes(routes!(trevor::job))
        .routes(routes!(trevor::story_panels))
        .routes(routes!(trevor::feedback))
        .routes(routes!(trevor::worker_next))
        .routes(routes!(trevor::worker_result))
        .routes(routes!(trevor::worker_heartbeat))
        .routes(routes!(trevor::publish_version))
        .routes(routes!(story::index_srv))
        .routes(routes!(story::illustrations_srv))
        .routes(routes!(story::archive_srv))
        .routes(routes!(story::sprites_srv))
        .routes(routes!(story::sprite_detail_srv))
        .routes(routes!(story::sprite_variant_thumb_srv))
        .routes(routes!(story::gallery_srv))
        .routes(routes!(story::gallery_picture_srv))
        .routes(routes!(story::art_gallery_srv))
        .routes(routes!(story::art_picture_srv))
        .routes(routes!(story::detail_srv))
        .routes(routes!(operators::skins_detail))
        .routes(routes!(operators::skins_detail_srv))
        .routes(routes!(operators::index_srv))
        .routes(routes!(operators::ownership_srv))
        .routes(routes!(operators::recruitment_srv))
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
        .routes(routes!(assets::story_sprite_thumb_srv))
        .routes(routes!(assets::generic_srv))
        .merge(tier_lists::router())
        .merge(grids::router())
        .merge(i18n::router())
}
