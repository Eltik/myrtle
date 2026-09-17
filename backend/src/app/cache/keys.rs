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
    /// Per-operator default-skill and default-module distributions. Keyed under
    /// its own prefix rather than `operators:ownership:` so the two can be
    /// invalidated independently, even though today one job refreshes both.
    OperatorBuildStats {
        server: &'a str,
        operator_id: &'a str,
    },
    CommunityEnemyAverage,
    /// A computed base rotation for one uid + exact request (layout, locks,
    /// promotion flag, facts). Deterministic given the inputs, so a short TTL
    /// only bounds staleness against a re-sync.
    BaseRotation {
        uid: &'a str,
        request_hash: u64,
    },
    /// The planner's optimize search, keyed like the rotation.
    BaseOptimize {
        uid: &'a str,
        request_hash: u64,
    },
    /// One memoised simulation. `kind` separates dps from hps, which share a
    /// request type but not a result type.
    /// One user's improvements body, keyed on the sync generation that produced
    /// it. `version` is `users.updated_at`, which `trg_users_timestamp` bumps on
    /// every sync upsert, so a fresh sync writes a NEW key rather than needing the
    /// old one cleared: a stale body cannot be served even if an invalidation hook
    /// is forgotten. Same self-addressing trick as `I18nCatalog`.
    ///
    /// KNOWN GAP, bounded by the TTL: `set_base_facts` writes `user_settings`
    /// without touching `users`, so saving base facts does not move the version
    /// and the body can lag by up to one TTL. Closing it properly needs the uid at
    /// `base_planner::save_facts`, which only has the viewer's UUID.
    UserImprovements {
        uid: &'a str,
        version: i64,
    },
    DpsCalculate {
        kind: &'a str,
        body_hash: u64,
    },
    DpsList {
        kind: &'a str,
    },
    /// A rendered UI message catalog. The content hash is part of the key, so
    /// a body under a given key can never be stale - a translator's edit moves
    /// the hash and therefore the key. The long TTL is safe for the same
    /// reason; `invalidate_by_prefix("i18n:")` on write only keeps the store
    /// from accumulating orphaned bodies.
    I18nCatalog {
        locale: &'a str,
        namespace: &'a str,
        hash: &'a str,
    },
    /// locale+namespace -> current catalog hash. This is the only i18n read
    /// that must go stale quickly, because it is what publishes a new hash.
    I18nManifest,
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
            CacheKey::OperatorBuildStats {
                server,
                operator_id,
            } => format!("operators:buildstats:{server}:{operator_id}"),
            CacheKey::CommunityEnemyAverage => "enemies:community_average".to_owned(),
            CacheKey::BaseRotation { uid, request_hash } => {
                format!("base:rotation:{uid}:{request_hash}")
            }
            CacheKey::BaseOptimize { uid, request_hash } => {
                format!("base:optimize:{uid}:{request_hash}")
            }
            CacheKey::UserImprovements { uid, version } => {
                format!("improvements:{uid}:{version}")
            }
            CacheKey::DpsCalculate { kind, body_hash } => {
                format!("dps:calc:{kind}:{body_hash}")
            }
            CacheKey::DpsList { kind } => format!("dps:list:{kind}"),
            CacheKey::I18nCatalog {
                locale,
                namespace,
                hash,
            } => format!("i18n:catalog:{locale}:{namespace}:{hash}"),
            CacheKey::I18nManifest => "i18n:manifest".to_owned(),
        }
    }

    pub const fn ttl(&self) -> Duration {
        match self {
            CacheKey::User { .. } => Duration::from_mins(10),
            CacheKey::Stats => Duration::from_mins(5),
            CacheKey::StaticData { .. } => Duration::from_hours(24), // invalidated on game-data reload
            CacheKey::Leaderboard { .. } => Duration::from_mins(5),
            CacheKey::Search { .. } => Duration::from_mins(2),
            CacheKey::TierList { .. } => Duration::from_mins(10),
            CacheKey::GameSession { .. } => Duration::from_hours(1),
            CacheKey::PortalSession { .. } => Duration::from_hours(168), // 1 week
            CacheKey::GachaGlobalStats => Duration::from_mins(5),
            CacheKey::GachaEnhancedStats { .. } => Duration::from_mins(10),
            CacheKey::GachaPerBannerStats => Duration::from_mins(10),
            CacheKey::LeaderboardMovers { .. } => Duration::from_mins(15),
            CacheKey::LeaderboardDistribution { .. } => Duration::from_mins(10),
            CacheKey::LeaderboardStanding { .. } => Duration::from_mins(1),
            CacheKey::SkinPopularity => Duration::from_hours(1),
            CacheKey::OperatorOwnership { .. } => Duration::from_hours(1),
            CacheKey::OperatorBuildStats { .. } => Duration::from_hours(1),
            CacheKey::CommunityEnemyAverage => Duration::from_mins(30),
            CacheKey::BaseRotation { .. } => Duration::from_mins(5),
            CacheKey::BaseOptimize { .. } => Duration::from_mins(5),
            // Same hour as the list it belongs to. A simulation is a pure
            // function of the body and the game data, so the only thing that can
            // invalidate it is a reload, and `asset_watcher` clears the whole
            // `dps:` prefix on one.
            // Deliberately SHORT, and not because of the roster: the version in
            // the key already handles that. Two builders read the wall clock,
            // `build_stage_improvements` to decide which events are open and
            // `build_medal_improvements` to bucket medals as still-earnable or
            // missed forever, and both change SET MEMBERSHIP at a rotation
            // boundary rather than just a label. This TTL is the only bound on
            // how long a user is told an event is open after it closed.
            CacheKey::UserImprovements { .. } => Duration::from_mins(5),
            CacheKey::DpsCalculate { .. } => Duration::from_hours(1),
            CacheKey::DpsList { .. } => Duration::from_hours(1),
            CacheKey::I18nCatalog { .. } => Duration::from_hours(24), // content-addressed; cannot go stale
            CacheKey::I18nManifest => Duration::from_secs(30),
        }
    }
}
