//! Every number and list the Squad Builder picks itself rather than reads from
//! the game data, in one place, each with why. Game facts never live here:
//! skill ranks, masteries, module ranks, level caps, potentials, enemy grade
//! bands and recruitment tag names are read from the tables (see `rules` and
//! `index`).

use crate::core::gamedata::types::operator::OperatorProfession;

use super::types::CoverageRowKey;

/// Seats in a squad. The game client's constant: no excel table carries it
/// (checked `gamedata_const`: `CharRotationPresetMaxCnt` 4 is the base's,
/// `MaxSquadAssistDisplayNum` 9 the support list's).
pub const SQUAD_SIZE: usize = 12;
/// The index the support seat reports as: the first index past the squad.
pub const SUPPORT_SEAT: i32 = 12;
/// Saved squads per account. A product choice; `user_squads.slot` has the
/// same bound as a CHECK (v035).
pub const SLOT_LIMIT: i32 = 4;
/// Longest squad name stored, in characters; `user_squads.name varchar(40)`.
pub const NAME_MAX_CHARS: usize = 40;
/// Longest stage id stored; `user_squads.stage_id varchar(80)`.
pub const STAGE_ID_MAX: usize = 80;

/// Highest starting DP accepted: a picked stage's `initialCost` is sent as is,
/// and the level files go up to 999 (20 levels on EN, 2026-10-11, the
/// sandbox and special-mode maps). The page's own stepper stops at 99.
pub const START_DP_MAX: i32 = 999;
/// Highest deploy limit accepted. A stage's `characterLimit` is used when known.
pub const DEPLOY_LIMIT_MAX: i32 = 20;
/// A sanity cap for any build number in an analyze request. The analysis clamps
/// every field to what the operator has (`build::clamp_build`), so this only
/// turns away garbage before any work is done.
pub const BUILD_FIELD_MAX: i32 = 999;

/// TRADE: a seat this many levels or more under the usual promotion's median
/// is flagged. 15 levels is about one promotion's worth of stat growth on a
/// 6-star past the halfway mark; chosen by us, not by the game.
pub const LEVEL_GAP: i32 = 15;

/// TRADE: a redeploy at or under this many seconds counts as fast. The
/// Fast-Redeploy Specialists sit at 18 s base; 25 s also takes in the
/// Merchants (25 s) and potential-cut ones. The game has no flag for it; its
/// Fast-Redeploy recruitment tag is the fallback (`TAG_FALLBACK`).
pub const FAST_REDEPLOY_SECS: f64 = 25.0;

/// Coverage "Block 3" row: a unit that blocks at least this many enemies,
/// from its stats or a skill or talent `block_cnt` bonus. A product threshold:
/// no table names a "high block" count, so 3 is the line we draw, not a game
/// fact.
pub const BLOCK_ROW_MIN: i32 = 3;

/// Themes: a value every seat shares is a label once the squad has this many.
pub const THEME_LABEL_MIN_SEATS: usize = 2;
/// Themes: "close" needs a squad of at least this many...
pub const THEME_CLOSE_MIN_SEATS: usize = 4;
/// ...at least this many sharing the value...
pub const THEME_CLOSE_MIN_SHARED: usize = 3;
/// ...and at most this many missing it.
pub const THEME_CLOSE_MAX_MISSING: usize = 2;
/// Themes: anything at least this many seats share is listed.
pub const THEME_SHARED_MIN: usize = 2;

/// The Themes "missing classes" order: the in-game roster sort. Which
/// professions exist is read from the operator table (`index::SquadIndex`);
/// only the order is ours.
pub const PROFESSION_ORDER: &[OperatorProfession] = &[
    OperatorProfession::Vanguard,
    OperatorProfession::Guard,
    OperatorProfession::Defender,
    OperatorProfession::Sniper,
    OperatorProfession::Caster,
    OperatorProfession::Medic,
    OperatorProfession::Supporter,
    OperatorProfession::Specialist,
];

/// Crowd-control statuses, matched as whole segments of a blackboard key
/// (`stun`, `stun_duration`, `attack@sluggish`, `sleach_t_1[enemy].stun`).
/// Which statuses count as control is a product choice; the keys are the
/// game's own and the same on every server.
pub const CC_STATUS_KEYS: &[&str] = &[
    "stun", "sluggish", "sleep", "silence", "levitate", "cold", "frozen", "root",
];

/// The recruitment tags that answer a Coverage row when nothing structural
/// does, by `gacha_table.GachaTags` `TagId` (the same on every server; each
/// server's `TagName` is read at load). Which tag answers which row is ours.
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

/// The enemy grade (`enemy_handbook_table.LevelInfoList` `ClassLevel`) from
/// which a stat counts as a stage threat; the threshold is that grade's lower
/// bound, read at load. The grade picked is ours: A for RES and DEF, S for
/// move speed.
pub const THREAT_GRADE_RES: &str = "A";
pub const THREAT_GRADE_DEF: &str = "A";
pub const THREAT_GRADE_MOVE_SPEED: &str = "S";
