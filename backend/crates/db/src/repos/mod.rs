pub mod content_audit;
pub mod credentials;
pub mod gacha;
pub mod grids;
pub mod i18n;
pub mod items;
pub mod leaderboard;
pub mod lookback;
pub mod music;
pub mod music_stats;
pub mod oauth;
pub mod operator_notes;
pub mod planner;
pub mod progress;
pub mod release;
pub mod roster;
pub mod scores;
pub mod squads;
pub mod stats;
pub mod story;
pub mod tier_lists;
pub mod user_search;
pub mod users;

fn kind_names(kinds: &[myrtle_types::EntityKind]) -> Vec<String> {
    kinds.iter().map(|k| k.as_str().to_owned()).collect()
}
