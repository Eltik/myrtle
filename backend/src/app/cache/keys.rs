use std::time::Duration;

pub enum CacheKey<'a> {
    User {
        uid: &'a str,
    },
    Stats,
    StaticData {
        resource: &'a str,
        fields_hash: u64,
        page: u32,
    },
    Leaderboard {
        sort: &'a str,
        server: Option<&'a str>,
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
}

impl CacheKey<'_> {
    pub fn to_key_string(&self) -> String {
        match self {
            CacheKey::User { uid } => format!("user:{uid}"),
            CacheKey::Stats => "stats:global".to_owned(),
            CacheKey::StaticData {
                resource,
                fields_hash,
                page,
            } => {
                format!("static:{resource}:{fields_hash}:{page}")
            }
            CacheKey::Leaderboard {
                sort,
                server,
                limit,
                offset,
            } => {
                let srv = server.unwrap_or("all");
                format!("leaderboard:{sort}:{srv}:{limit}:{offset}")
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
        }
    }

    pub fn ttl(&self) -> Duration {
        match self {
            CacheKey::User { .. } => Duration::from_secs(600), // 10 min
            CacheKey::Stats => Duration::from_secs(300),       // 5 min
            CacheKey::StaticData { .. } => Duration::from_secs(1800), // 30 min
            CacheKey::Leaderboard { .. } => Duration::from_secs(300), // 5 min
            CacheKey::Search { .. } => Duration::from_secs(120), // 2 min
            CacheKey::TierList { .. } => Duration::from_secs(600), // 10 min
            CacheKey::GameSession { .. } => Duration::from_secs(3600), // 1 hour
            CacheKey::PortalSession { .. } => Duration::from_secs(604800), // 1 week
            CacheKey::GachaGlobalStats => Duration::from_secs(300), // 5 min
            CacheKey::GachaEnhancedStats { .. } => Duration::from_secs(600), // 10 min
            CacheKey::LeaderboardMovers { .. } => Duration::from_secs(900), // 15 min
            CacheKey::LeaderboardDistribution { .. } => Duration::from_secs(600), // 10 min
            CacheKey::LeaderboardStanding { .. } => Duration::from_secs(60), // 1 min
        }
    }
}
