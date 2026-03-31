use crate::app::cache::{keys::CacheKey, store};
use crate::app::error::ApiError;
use crate::app::state::AppState;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    pub users: UserStats,
    pub gacha: GachaPublicStats,
    pub game_data: GameDataStats,
    pub tier_lists: TierListStats,
    pub computed_at: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub total: i64,
    pub by_server: ServerBreakdown,
    pub signups_7d: i64,
    pub signups_30d: i64,
    pub public_profiles: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerBreakdown {
    pub en: i64,
    pub jp: i64,
    pub kr: i64,
    pub cn: i64,
    pub bili: i64,
    pub tw: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaPublicStats {
    pub total_pulls: i64,
    pub contributing_users: i64,
    pub six_star_count: i64,
    pub five_star_count: i64,
    pub four_star_count: i64,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TierListStats {
    pub total: i64,
    pub active: i64,
    pub total_versions: i64,
    pub total_placements: i64,
}

pub async fn get_stats(state: &AppState) -> Result<StatsResponse, ApiError> {
    let key = CacheKey::Stats;
    if let Some(cached) = store::get::<StatsResponse>(&mut state.redis.clone(), &key).await {
        return Ok(cached);
    }

    // Run all DB queries in parallel
    let (user_row, gacha_row, tier_list_row) = tokio::try_join!(
        fetch_user_stats(&state.db),
        fetch_gacha_stats(&state.db),
        fetch_tier_list_stats(&state.db),
    )?;

    // Game data stats are free — just read from memory
    let game_data = GameDataStats {
        operators: state.game_data.operators.len(),
        skills: state.game_data.skills.len(),
        modules: state.game_data.modules.equip_dict.len(),
        skins: state.game_data.skins.char_skins.len(),
        stages: state.game_data.stages.len(),
        zones: state.game_data.zones.len(),
        enemies: state.game_data.enemies.enemy_data.len(),
    };

    let response = StatsResponse {
        users: user_row,
        gacha: gacha_row,
        game_data,
        tier_lists: tier_list_row,
        computed_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };

    store::set(&mut state.redis.clone(), &key, &response).await;
    Ok(response)
}

async fn fetch_user_stats(db: &PgPool) -> Result<UserStats, sqlx::Error> {
    let row = sqlx::query_as::<_, UserStatsRow>(
        r#"
        SELECT
            COUNT(*) AS total,
            COUNT(*) FILTER (WHERE server_id = 0) AS en,
            COUNT(*) FILTER (WHERE server_id = 1) AS jp,
            COUNT(*) FILTER (WHERE server_id = 2) AS kr,
            COUNT(*) FILTER (WHERE server_id = 3) AS cn,
            COUNT(*) FILTER (WHERE server_id = 4) AS bili,
            COUNT(*) FILTER (WHERE server_id = 5) AS tw,
            COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '7 days') AS signups_7d,
            COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '30 days') AS signups_30d,
            (SELECT COUNT(*) FROM user_settings WHERE public_profile = true) AS public_profiles
        FROM users
        "#,
    )
    .fetch_one(db)
    .await?;

    Ok(UserStats {
        total: row.total,
        by_server: ServerBreakdown {
            en: row.en,
            jp: row.jp,
            kr: row.kr,
            cn: row.cn,
            bili: row.bili,
            tw: row.tw,
        },
        signups_7d: row.signups_7d,
        signups_30d: row.signups_30d,
        public_profiles: row.public_profiles,
    })
}

#[derive(sqlx::FromRow)]
struct UserStatsRow {
    total: i64,
    en: i64,
    jp: i64,
    kr: i64,
    cn: i64,
    bili: i64,
    tw: i64,
    signups_7d: i64,
    signups_30d: i64,
    public_profiles: i64,
}

async fn fetch_gacha_stats(db: &PgPool) -> Result<GachaPublicStats, sqlx::Error> {
    let row = sqlx::query_as::<_, GachaStatsRow>(
        r#"
        SELECT
            COUNT(*) AS total_pulls,
            COUNT(DISTINCT gr.user_id) AS contributing_users,
            COUNT(*) FILTER (WHERE rarity = 6) AS six_star_count,
            COUNT(*) FILTER (WHERE rarity = 5) AS five_star_count,
            COUNT(*) FILTER (WHERE rarity = 4) AS four_star_count
        FROM gacha_records gr
        JOIN user_settings us ON us.user_id = gr.user_id
        WHERE us.share_stats = true
        "#,
    )
    .fetch_one(db)
    .await?;

    Ok(GachaPublicStats {
        total_pulls: row.total_pulls,
        contributing_users: row.contributing_users,
        six_star_count: row.six_star_count,
        five_star_count: row.five_star_count,
        four_star_count: row.four_star_count,
    })
}

#[derive(sqlx::FromRow)]
struct GachaStatsRow {
    total_pulls: i64,
    contributing_users: i64,
    six_star_count: i64,
    five_star_count: i64,
    four_star_count: i64,
}

async fn fetch_tier_list_stats(db: &PgPool) -> Result<TierListStats, sqlx::Error> {
    let row = sqlx::query_as::<_, TierListStatsRow>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM tier_lists) AS total,
            (SELECT COUNT(*) FROM tier_lists WHERE is_active = true) AS active,
            (SELECT COUNT(*) FROM tier_list_versions) AS total_versions,
            (SELECT COUNT(*) FROM tier_placements) AS total_placements
        "#,
    )
    .fetch_one(db)
    .await?;

    Ok(TierListStats {
        total: row.total,
        active: row.active,
        total_versions: row.total_versions,
        total_placements: row.total_placements,
    })
}

#[derive(sqlx::FromRow)]
struct TierListStatsRow {
    total: i64,
    active: i64,
    total_versions: i64,
    total_placements: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminStatsResponse {
    #[serde(flatten)]
    pub public: StatsResponse,
    pub users_by_role: RoleBreakdown,
    pub recent_users: Vec<RecentUser>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleBreakdown {
    pub user: i64,
    pub tier_list_editor: i64,
    pub tier_list_admin: i64,
    pub super_admin: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentUser {
    pub uid: String,
    pub server_id: i16,
    pub nickname: Option<String>,
    pub level: Option<i16>,
    pub created_at: String,
}

pub async fn get_admin_stats(state: &AppState) -> Result<AdminStatsResponse, ApiError> {
    let public = get_stats(state).await?;

    let (role_row, recent_users) = tokio::try_join!(
        fetch_role_breakdown(&state.db),
        fetch_recent_users(&state.db),
    )?;

    Ok(AdminStatsResponse {
        public,
        users_by_role: role_row,
        recent_users,
    })
}

async fn fetch_role_breakdown(db: &PgPool) -> Result<RoleBreakdown, sqlx::Error> {
    let row = sqlx::query_as::<_, RoleBreakdownRow>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE role = 'user') AS user_count,
            COUNT(*) FILTER (WHERE role = 'tier_list_editor') AS editor_count,
            COUNT(*) FILTER (WHERE role = 'tier_list_admin') AS admin_count,
            COUNT(*) FILTER (WHERE role = 'super_admin') AS super_count
        FROM users
        "#,
    )
    .fetch_one(db)
    .await?;

    Ok(RoleBreakdown {
        user: row.user_count,
        tier_list_editor: row.editor_count,
        tier_list_admin: row.admin_count,
        super_admin: row.super_count,
    })
}

#[derive(sqlx::FromRow)]
struct RoleBreakdownRow {
    user_count: i64,
    editor_count: i64,
    admin_count: i64,
    super_count: i64,
}

async fn fetch_recent_users(db: &PgPool) -> Result<Vec<RecentUser>, sqlx::Error> {
    let rows = sqlx::query_as::<_, RecentUserRow>(
        "SELECT uid, server_id, nickname, level, created_at FROM users ORDER BY created_at DESC LIMIT 20"
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| RecentUser {
            uid: r.uid,
            server_id: r.server_id,
            nickname: r.nickname,
            level: r.level,
            created_at: r.created_at.to_rfc3339(),
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct RecentUserRow {
    uid: String,
    server_id: i16,
    nickname: Option<String>,
    level: Option<i16>,
    created_at: chrono::DateTime<chrono::Utc>,
}
