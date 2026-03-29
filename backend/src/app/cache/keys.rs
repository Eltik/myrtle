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
        page: u32,
    },
    Search {
        query_hash: u64,
    },
    TierList {
        slug: &'a str,
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
            CacheKey::Leaderboard { sort, server, page } => {
                let srv = server.unwrap_or("all");
                format!("leaderboard:{sort}:{srv}:{page}")
            }
            CacheKey::Search { query_hash } => format!("search:{query_hash}"),
            CacheKey::TierList { slug } => format!("tierlist:{slug}"),
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
        }
    }
}
