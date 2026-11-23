use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use myrtle_db::repos::{gacha, roster, tier_lists, users};
use serde::{Deserialize, Serialize};

use crate::{Action, AppContext, AppError, Resource, Subject};

const STATS_KEY: &str = "stats:global";
const STATS_TTL: Duration = Duration::from_mins(5);
const RECENT_USERS: i64 = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    pub users: UserStats,
    pub gacha: GachaPublicStats,
    pub game_data: GameDataStats,
    pub tier_lists: TierListSiteStats,
    pub rosters: RostersStats,
    pub computed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub total: i64,
    pub by_server: ServerBreakdown,
    pub signups_7d: i64,
    pub signups_30d: i64,
    pub public_profiles: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerBreakdown {
    pub en: i64,
    pub jp: i64,
    pub kr: i64,
    pub cn: i64,
    pub bili: i64,
    pub tw: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaPublicStats {
    pub total_pulls: i64,
    pub contributing_users: i64,
    pub six_star_count: i64,
    pub five_star_count: i64,
    pub four_star_count: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDataStats {
    pub operators: usize,
    pub skills: usize,
    pub modules: usize,
    pub skins: usize,
    pub stages: usize,
    pub zones: usize,
    pub enemies: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TierListSiteStats {
    pub total: i64,
    pub active: i64,
    pub total_versions: i64,
    pub total_placements: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RostersStats {
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminStatsResponse {
    #[serde(flatten)]
    pub public: StatsResponse,
    pub users_by_role: RoleBreakdown,
    pub recent_users: Vec<RecentUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleBreakdown {
    pub user: i64,
    pub tier_list_editor: i64,
    pub tier_list_admin: i64,
    pub translator: i64,
    pub super_admin: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentUser {
    pub uid: String,
    pub server_id: i16,
    pub nickname: Option<String>,
    pub level: Option<i16>,
    pub created_at: String,
}

pub async fn site(ctx: &AppContext) -> Result<StatsResponse, AppError> {
    let snapshot = ctx.snapshots.get(ctx.snapshots.default_server());
    let key = snapshot
        .as_ref()
        .map_or_else(|| STATS_KEY.to_owned(), |s| s.key(STATS_KEY));
    if let Some(cached) = ctx.cache.get_json(&key).await {
        return Ok(cached);
    }
    let mut conn = ctx.db.conn().await?;
    let u = users::site_counts(&mut conn).await?;
    let g = gacha::shared_totals(&mut conn).await?;
    let t = tier_lists::site_counts(&mut conn).await?;
    let rosters = roster::users_with_roster(&mut conn).await?;
    drop(conn);
    let game_data = snapshot
        .map(|s| {
            let m = &s.model;
            GameDataStats {
                operators: m.operators.len(),
                skills: m.skills.len(),
                modules: m.modules.equip_dict.len(),
                skins: m.skins.char_skins.len(),
                stages: m.stages.len(),
                zones: m.zones.len(),
                enemies: m.enemies.enemy_data.len(),
            }
        })
        .unwrap_or_default();
    let stats = StatsResponse {
        users: UserStats {
            total: u.total,
            by_server: ServerBreakdown {
                en: u.en,
                jp: u.jp,
                kr: u.kr,
                cn: u.cn,
                bili: u.bili,
                tw: u.tw,
            },
            signups_7d: u.signups_7d,
            signups_30d: u.signups_30d,
            public_profiles: u.public_profiles,
        },
        gacha: GachaPublicStats {
            total_pulls: g.total_pulls,
            contributing_users: g.total_users,
            six_star_count: g.six_star_count,
            five_star_count: g.five_star_count,
            four_star_count: g.four_star_count,
        },
        game_data,
        tier_lists: TierListSiteStats {
            total: t.total,
            active: t.active,
            total_versions: t.total_versions,
            total_placements: t.total_placements,
        },
        rosters: RostersStats { total: rosters },
        computed_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
    };
    ctx.cache.set_json(&key, &stats, STATS_TTL).await;
    Ok(stats)
}

pub async fn admin(ctx: &AppContext, subject: &Subject) -> Result<AdminStatsResponse, AppError> {
    ctx.policy
        .require(subject, Action::ViewSiteStats, &Resource::Site)?;
    let public = site(ctx).await?;
    let mut conn = ctx.db.conn().await?;
    let r = users::role_counts(&mut conn).await?;
    let recent = users::recent(&mut conn, RECENT_USERS).await?;
    Ok(AdminStatsResponse {
        public,
        users_by_role: RoleBreakdown {
            user: r.user,
            tier_list_editor: r.tier_list_editor,
            tier_list_admin: r.tier_list_admin,
            translator: r.translator,
            super_admin: r.super_admin,
        },
        recent_users: recent
            .into_iter()
            .map(|a| RecentUser {
                uid: a.uid,
                server_id: a.server_id,
                nickname: a.nickname,
                level: a.level,
                created_at: a.created_at.to_rfc3339(),
            })
            .collect(),
    })
}
