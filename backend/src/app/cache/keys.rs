use std::time::Duration;

pub enum CacheKey<'a> {
    User {
        uid: &'a str,
    },
    Stats,
    StaticData {
        resource: &'a str,
        server: &'a str,
        fields_hash: u64,
        page: u32,
    },
    Leaderboard {
        sort: &'a str,
        server: Option<&'a str>,
        movement_interval: Option<&'a str>,
        movement_only: bool,
        q: Option<&'a str>,
        limit: u32,
        offset: u32,
    },
    Search {
        query_hash: u64,
    },
    TierList {
        slug: &'a str,
    },
    GameSession {
        uid: &'a str,
    },
    PortalSession {
        uid: &'a str,
    },
    GachaGlobalStats,
    GachaEnhancedStats {
        top_n: u32,
        include_timing: bool,
    },
    GachaPerBannerStats,
    LeaderboardMovers {
        direction: &'a str,
        interval: &'a str,
        server: Option<&'a str>,
    },
    LeaderboardDistribution {
        top_n: u32,
    },
    LeaderboardStanding {
        uid: &'a str,
        server: &'a str,
        window: u32,
    },
    SkinPopularity,
    OperatorOwnership {
        server: &'a str,
    },
    CommunityEnemyAverage,
}

impl CacheKey<'_> {
    pub fn to_key_string(&self) -> String {
        match self {
            CacheKey::User { uid } => format!("user:{uid}"),
            CacheKey::Stats => "stats:global".to_owned(),
            CacheKey::StaticData {
                resource,
                server,
                fields_hash,
                page,
            } => {
                format!("static:{server}:{resource}:{fields_hash}:{page}")
            }
            CacheKey::Leaderboard {
                sort,
                server,
                movement_interval,
                movement_only,
                q,
                limit,
                offset,
            } => {
                let srv = server.unwrap_or("all");
                let mv =
                    movement_interval.map_or_else(|| "none".to_owned(), |s| s.replace(' ', "_"));
                let mo = if *movement_only { "only" } else { "all" };
                let qk = q.unwrap_or("");
                format!("leaderboard:{sort}:{srv}:{mv}:{mo}:{qk}:{limit}:{offset}")
            }
            CacheKey::Search { query_hash } => format!("search:{query_hash}"),
            CacheKey::TierList { slug } => format!("tierlist:{slug}"),
            CacheKey::GameSession { uid } => format!("game_session:{uid}"),
            CacheKey::PortalSession { uid } => format!("portal_session:{uid}"),
            CacheKey::GachaGlobalStats => "gacha:global_stats".to_owned(),
            CacheKey::GachaEnhancedStats {
                top_n,
                include_timing,
            } => format!("gacha:enhanced_stats:{top_n}:{include_timing}"),
            CacheKey::GachaPerBannerStats => "gacha:per_banner_stats".to_owned(),
            CacheKey::LeaderboardMovers {
                direction,
                interval,
                server,
            } => {
                let srv = server.unwrap_or("all");
                let i = interval.replace(' ', "_");
                format!("leaderboard:movers:{direction}:{i}:{srv}")
            }
            CacheKey::LeaderboardDistribution { top_n } => {
                format!("leaderboard:distribution:{top_n}")
            }
            CacheKey::LeaderboardStanding {
                uid,
                server,
                window,
            } => {
                format!("leaderboard:standing:{server}:{uid}:{window}")
            }
            CacheKey::SkinPopularity => "skins:popularity".to_owned(),
            CacheKey::OperatorOwnership { server } => format!("operators:ownership:{server}"),
            CacheKey::CommunityEnemyAverage => "enemies:community_average".to_owned(),
        }
    }

    pub const fn ttl(&self) -> Duration {
        match self {
            CacheKey::User { .. } => Duration::from_mins(10), // 10 min
            CacheKey::Stats => Duration::from_mins(5),        // 5 min
            CacheKey::StaticData { .. } => Duration::from_hours(24), // invalidated on game-data reload
            CacheKey::Leaderboard { .. } => Duration::from_mins(5),  // 5 min
            CacheKey::Search { .. } => Duration::from_mins(2),       // 2 min
            CacheKey::TierList { .. } => Duration::from_mins(10),    // 10 min
            CacheKey::GameSession { .. } => Duration::from_hours(1), // 1 hour
            CacheKey::PortalSession { .. } => Duration::from_hours(168), // 1 week
            CacheKey::GachaGlobalStats => Duration::from_mins(5),    // 5 min
            CacheKey::GachaEnhancedStats { .. } => Duration::from_mins(10), // 10 min
            CacheKey::GachaPerBannerStats => Duration::from_mins(10), // 10 min
            CacheKey::LeaderboardMovers { .. } => Duration::from_mins(15), // 15 min
            CacheKey::LeaderboardDistribution { .. } => Duration::from_mins(10), // 10 min
            CacheKey::LeaderboardStanding { .. } => Duration::from_mins(1), // 1 min
            CacheKey::SkinPopularity => Duration::from_hours(1),     // 1 hour
            CacheKey::OperatorOwnership { .. } => Duration::from_hours(1), // 1 hour
            CacheKey::CommunityEnemyAverage => Duration::from_mins(30), // 30 min
        }
    }
}
