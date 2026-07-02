use axum::{
    Json, Router,
    routing::{get, post, put},
};
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::state::AppState;
use crate::database::queries::users::find_by_uid;

/// The standard `{"status":"ok"}` success body for endpoints that return no payload.
pub fn ok_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

/// Resolve the target `user_id` from either a `uid` query param (public access)
/// or the authenticated user's token (private access). A `uid` lookup is only
/// allowed for the caller's own profile or a profile marked public.
pub(crate) async fn resolve_user_id(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
) -> Result<Uuid, ApiError> {
    if let Some(uid) = uid_param {
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

        Ok(profile.id)
    } else {
        let auth = auth.0.as_ref().ok_or(ApiError::Unauthorized)?;
        auth.user_uuid()
    }
}

pub mod assets;
pub mod auth;
pub mod chibis;
pub mod dps;
pub mod enemies;
pub mod gacha;
pub mod health;
pub mod improvements;
pub mod inventory;
pub mod leaderboard;
pub mod level;
pub mod operator_notes;
pub mod operators;
pub mod planner;
pub mod roster;
pub mod search;
pub mod skins;
pub mod social;
pub mod stages;
pub mod static_data;
pub mod stats;
pub mod tier_lists;
pub mod user;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/get-user", get(user::get_user))
        .route("/get-user-score", get(user::get_user_score))
        .route("/get-user-checkin", get(user::get_user_checkin))
        .route("/leaderboard", get(leaderboard::leaderboard))
        .route("/leaderboard/movers", get(leaderboard::top_movers))
        .route("/leaderboard/distribution", get(leaderboard::distribution))
        .route("/leaderboard/standing", get(leaderboard::standing))
        .route("/search", get(search::search))
        .route("/static/{resource}", get(static_data::get_static))
        .route("/level/{stage_id}", get(level::get_level_map))
        .route("/avatar/{id}", get(assets::avatar))
        .route("/portrait/{id}", get(assets::portrait))
        .route("/skill-icon/{id}", get(assets::skill_icon))
        .route("/module-icon/{id}", get(assets::module_icon))
        .route("/module-big/{id}", get(assets::module_big))
        .route("/enemy-icon/{id}", get(assets::enemy_icon))
        .route("/item-icon/{id}", get(assets::item_icon))
        .route("/medal-icon/{id}", get(assets::medal_icon))
        .route("/charart/{id}", get(assets::charart))
        .route("/skin-portrait/{id}", get(assets::skin_portrait))
        .route("/assets/{*path}", get(assets::generic))
        .route("/login/send-code", post(auth::send_code))
        .route("/login", post(auth::login))
        .route("/auth/verify", get(auth::verify))
        .route("/auth/update-settings", post(auth::update_settings))
        .route("/gacha/history", get(gacha::history))
        .route("/gacha/history/{char_id}", get(gacha::history_by_char))
        .route("/gacha/stored-records", get(gacha::stored_records))
        .route("/gacha/stats", get(gacha::stats))
        .route(
            "/gacha/settings",
            get(gacha::get_settings).post(gacha::update_settings),
        )
        .route("/refresh", post(auth::refresh))
        .route("/roster", get(roster::get_roster))
        .route("/roster/{operator_id}", get(roster::get_operator))
        .route("/stage-clears", get(stages::get_stage_clears))
        .route(
            "/encountered-enemies",
            get(enemies::get_encountered_enemies),
        )
        .route(
            "/encountered-enemies/community-average",
            get(enemies::get_community_average),
        )
        .route(
            "/user/improvements",
            get(improvements::get_user_improvements),
        )
        .route("/get-user-supports", get(roster::get_supports))
        .route("/inventory", get(inventory::get_inventory))
        .route("/user-skins", get(skins::get_owned_skins))
        .route("/skins/popularity", get(skins::get_skin_popularity))
        .route("/gacha/fetch", post(gacha::fetch))
        .route("/gacha/global-stats", get(gacha::global_stats))
        .route("/gacha/stats/enhanced", get(gacha::enhanced_stats))
        .route("/gacha/stats/per-banner", get(gacha::per_banner_stats))
        .route("/stats", get(stats::stats))
        .route("/admin/stats", get(stats::admin_stats))
        .route("/operators/index", get(operators::index))
        .route("/operators/ownership", get(operators::ownership))
        .route("/stages/{stage_id}/detail", get(stages::stage_detail))
        .route("/enemies/{id}", get(enemies::enemy_detail))
        .route("/enemies/{id}/stages", get(enemies::enemy_stages))
        .route("/chibis/{operator_id}", get(chibis::chibi_detail))
        .route("/skins/index", get(skins::skins_index))
        .route("/dps/operators", get(dps::operators))
        .route("/dps/calculate", post(dps::calculate))
        .route("/hps/operators", get(dps::healers))
        .route("/hps/calculate", post(dps::calculate_hps))
        .route("/operator-notes", get(operator_notes::list))
        .route("/operator-notes/{operator_id}", get(operator_notes::get))
        .route("/operator-notes/{operator_id}", put(operator_notes::update))
        .route(
            "/operator-notes/{operator_id}/audit",
            get(operator_notes::audit_log),
        )
        .route(
            "/admin/operator-notes/audit",
            get(operator_notes::global_audit_log),
        )
        .route("/friends", get(social::get_friends))
        .route("/players/search", get(social::search_players))
        .route("/plans", get(planner::list))
        .route("/plans/public", get(planner::list_public))
        .route(
            "/plan/{operator_id}",
            post(planner::upsert).delete(planner::delete),
        )
        .route("/plan/group", post(planner::create_group))
        .route(
            "/plan/group/{group_name}",
            put(planner::update_group).delete(planner::delete_group),
        )
        .route("/upcoming", get(operators::upcoming))
        .route("/{server}/upcoming", get(operators::upcoming_srv))
        .route("/operators/{id}", get(operators::detail))
        .route("/voices/{id}", get(operators::voices_detail))
        .route("/{server}/voices/{id}", get(operators::voices_detail_srv))
        .route("/skins/{id}", get(operators::skins_detail))
        .route("/{server}/skins/{id}", get(operators::skins_detail_srv))
        .route("/{server}/operators/index", get(operators::index_srv))
        .route(
            "/{server}/operators/ownership",
            get(operators::ownership_srv),
        )
        .route("/{server}/operators/{id}", get(operators::detail_srv))
        .route(
            "/{server}/stages/{stage_id}/detail",
            get(stages::stage_detail_srv),
        )
        .route("/{server}/enemies/{id}", get(enemies::enemy_detail_srv))
        .route(
            "/{server}/enemies/{id}/stages",
            get(enemies::enemy_stages_srv),
        )
        .route(
            "/{server}/chibis/{operator_id}",
            get(chibis::chibi_detail_srv),
        )
        .route("/{server}/skins/index", get(skins::skins_index_srv))
        .route(
            "/{server}/static/{resource}",
            get(static_data::get_static_srv),
        )
        .route("/{server}/level/{stage_id}", get(level::get_level_map_srv))
        .route("/{server}/avatar/{id}", get(assets::avatar_srv))
        .route("/{server}/portrait/{id}", get(assets::portrait_srv))
        .route("/{server}/skill-icon/{id}", get(assets::skill_icon_srv))
        .route("/{server}/module-icon/{id}", get(assets::module_icon_srv))
        .route("/{server}/module-big/{id}", get(assets::module_big_srv))
        .route("/{server}/enemy-icon/{id}", get(assets::enemy_icon_srv))
        .route("/{server}/item-icon/{id}", get(assets::item_icon_srv))
        .route("/{server}/medal-icon/{id}", get(assets::medal_icon_srv))
        .route("/{server}/charart/{id}", get(assets::charart_srv))
        .route(
            "/{server}/skin-portrait/{id}",
            get(assets::skin_portrait_srv),
        )
        .route("/{server}/assets/{*path}", get(assets::generic_srv))
        .merge(tier_lists::router())
}
