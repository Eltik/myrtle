use crate::types::CoverageRowKey;

pub const SQUAD_SIZE: usize = 12;
pub const SUPPORT_SEAT: i32 = 12;
/// `user_squads.slot`
pub const SLOT_LIMIT: i32 = 4;
/// `user_squads.name varchar(40)`
pub const NAME_MAX_CHARS: usize = 40;
/// `user_squads.stage_id varchar(80)`
pub const STAGE_ID_MAX: usize = 80;

pub const START_DP_MAX: i32 = 999;
pub const DEPLOY_LIMIT_MAX: i32 = 20;
pub const BUILD_FIELD_MAX: i32 = 999;

pub const LEVEL_GAP: i32 = 15;

pub const FAST_REDEPLOY_SECS: f64 = 25.0;

pub const BLOCK_ROW_MIN: i32 = 3;

pub const THEME_LABEL_MIN_SEATS: usize = 2;
pub const THEME_CLOSE_MIN_SEATS: usize = 4;
pub const THEME_CLOSE_MIN_SHARED: usize = 3;
pub const THEME_CLOSE_MAX_MISSING: usize = 2;
pub const THEME_SHARED_MIN: usize = 2;

pub const CC_STATUS_KEYS: &[&str] = &[
    "stun", "sluggish", "sleep", "cold", "frozen", "root", "lure", "fear", "attract", "binding",
    "slowdown", "dying",
];

/// `gacha_table.GachaTags` `TagId` -> the Coverage row it answers when nothing structural does
pub const TAG_FALLBACK: &[(i32, CoverageRowKey)] = &[
    (18, CoverageRowKey::DpGen),         // DP-Recovery
    (25, CoverageRowKey::FastRedeploy),  // Fast-Redeploy
    (29, CoverageRowKey::TrueElemental), // Elemental
    (21, CoverageRowKey::Multi),         // AoE
    (23, CoverageRowKey::Cc),            // Slow
    (12, CoverageRowKey::Cc),            // Crowd-Control
    (26, CoverageRowKey::Shift),         // Shift
    (24, CoverageRowKey::Debuff),        // Debuff
    (27, CoverageRowKey::Summons),       // Summon
];
