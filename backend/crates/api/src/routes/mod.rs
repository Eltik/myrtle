pub mod account;
pub mod announcements;
pub mod archives;
pub mod assets;
pub mod auth;
pub mod base;
pub mod chibis;
pub mod content_audit;
pub mod dps;
pub mod enemies;
pub mod gacha;
pub mod grids;
pub mod i18n;
pub mod improvements;
pub mod inventory;
pub mod item_leaderboard;
pub mod leaderboard;
pub mod level;
pub mod lookback;
pub mod maintenance;
pub mod music;
pub mod oauth;
pub mod operator_notes;
pub mod operators;
pub mod planner;
pub mod release;
pub mod roster;
pub mod search;
pub mod service_accounts;
pub mod skins;
pub mod social;
pub mod squads;
pub mod stages;
pub mod static_data;
pub mod stats;
pub mod story;
pub mod story_progress;
pub mod tier_lists;
pub mod user;

use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .merge(account::routes())
        .merge(announcements::routes())
        .merge(archives::routes())
        .merge(assets::routes())
        .merge(auth::routes())
        .merge(base::routes())
        .merge(chibis::routes())
        .merge(content_audit::routes())
        .merge(dps::routes())
        .merge(enemies::routes())
        .merge(gacha::routes())
        .merge(grids::routes())
        .merge(i18n::routes())
        .merge(improvements::routes())
        .merge(inventory::routes())
        .merge(item_leaderboard::routes())
        .merge(leaderboard::routes())
        .merge(lookback::routes())
        .merge(level::routes())
        .merge(maintenance::routes())
        .merge(music::routes())
        .merge(oauth::routes())
        .merge(operator_notes::routes())
        .merge(operators::routes())
        .merge(planner::routes())
        .merge(release::routes())
        .merge(roster::routes())
        .merge(search::routes())
        .merge(service_accounts::routes())
        .merge(skins::routes())
        .merge(social::routes())
        .merge(squads::routes())
        .merge(stages::routes())
        .merge(static_data::routes())
        .merge(stats::routes())
        .merge(story::routes())
        .merge(story_progress::routes())
        .merge(tier_lists::routes())
        .merge(user::routes())
}
