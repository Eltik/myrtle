use std::sync::LazyLock;
use std::time::Duration;

use crate::dps::ENGINE_VERSION;

/// TTL of a rank entry whose context is off the page's grid. One hour, the
/// calculate keys' TTL. A miss recomputes 1,343 rows (25.377 ms in commit
/// 089b4ebc's debug run, 116.213 ms in a contended debug run) and an entry is
/// 512,650 bytes of JSON on EN data, so a shorter life costs little and bounds
/// what a caller can leave behind to one hour of misses.
///
/// `DPS_RANK_OFF_GRID_TTL_SECS` overrides it; `86400` restores the old 24 h for
/// every rank entry EXACTLY. Unset means the default; a value that is not a
/// whole number of seconds above 0 is ignored with a warning.
static RANK_OFF_GRID_TTL: LazyLock<Duration> =
    LazyLock::new(|| off_grid_ttl(std::env::var("DPS_RANK_OFF_GRID_TTL_SECS").ok().as_deref()));

const RANK_OFF_GRID_TTL_DEFAULT: Duration = Duration::from_hours(1);

fn off_grid_ttl(raw: Option<&str>) -> Duration {
    let Some(raw) = raw else {
        return RANK_OFF_GRID_TTL_DEFAULT;
    };
    match raw.trim().parse::<u64>() {
        Ok(secs) if secs > 0 => Duration::from_secs(secs),
        _ => {
            tracing::warn!(
                value = raw,
                "DPS_RANK_OFF_GRID_TTL_SECS is not a whole number of seconds above 0; using 3600"
            );
            RANK_OFF_GRID_TTL_DEFAULT
        }
    }
}

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
    ItemLeaderboard {
        item: &'a str,
        server: Option<&'a str>,
        q: Option<&'a str>,
        limit: u32,
        offset: u32,
    },
    ItemLeaderboardCatalog {
        server: Option<&'a str>,
    },
    MusicStats,
    MusicTopTracks {
        window: &'a str,
        limit: u32,
    },
    MusicListeners {
        window: &'a str,
        sort: &'a str,
        track: Option<&'a str>,
        limit: u32,
        offset: u32,
    },
    ItemLeaderboardStanding {
        item: &'a str,
        uid: &'a str,
        server: &'a str,
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
    /// One user's improvements body, keyed on `users.updated_at`, which
    /// `trg_users_timestamp` bumps on every sync upsert. A sync writes a new key,
    /// so a stale body can't be served even if an invalidation hook is missed.
    /// Same trick as `I18nCatalog`.
    ///
    /// KNOWN GAP, bounded by the TTL: `set_base_facts` writes `user_settings`
    /// without touching `users`, so saving base facts does not move the version
    /// and the body can lag by up to one TTL. Closing it properly needs the uid at
    /// `base_planner::save_facts`, which only has the viewer's UUID.
    UserImprovements {
        uid: &'a str,
        version: i64,
    },
    /// One memoised simulation (or batch, or supporter table). `kind`
    /// separates dps from hps, which share a request type but not a result
    /// type. `data` is the game-data identity
    /// (`dps::data_cache::game_data_identity`), so a reload cannot serve a
    /// result computed on the previous data even before the watcher's prefix
    /// invalidation lands, and a restart onto other data cannot read old ones.
    DpsCalculate {
        kind: &'a str,
        data: &'a str,
        body_hash: u64,
    },
    DpsList {
        kind: &'a str,
        data: &'a str,
    },
    /// The full all-operator ranking for one shared context, before filters.
    /// Keyed like `DpsCalculate`, engine version included. `on_page_grid`
    /// (`RankRequest::on_page_grid`) picks the TTL: a context the rank page
    /// reaches with its own controls keeps 24 h, any other expires like a
    /// calculation, so a stream of distinct DEF/RES floats cannot pile up day-long
    /// entries.
    DpsRank {
        data: &'a str,
        context_hash: u64,
        on_page_grid: bool,
    },
    /// A rendered UI message catalog. The content hash is in the key, so a body is
    /// never stale and the long TTL is safe; `invalidate_by_prefix("i18n:")` on
    /// write only clears orphaned bodies.
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
            CacheKey::ItemLeaderboard {
                item,
                server,
                q,
                limit,
                offset,
            } => {
                let srv = server.unwrap_or("all");
                let qk = q.unwrap_or("");
                format!("leaderboard:items:{item}:{srv}:{qk}:{limit}:{offset}")
            }
            CacheKey::ItemLeaderboardCatalog { server } => {
                let srv = server.unwrap_or("all");
                format!("leaderboard:items:catalog:{srv}")
            }
            CacheKey::ItemLeaderboardStanding { item, uid, server } => {
                format!("leaderboard:items:standing:{item}:{server}:{uid}")
            }
            CacheKey::MusicStats => "music:stats".to_owned(),
            CacheKey::MusicTopTracks { window, limit } => {
                format!("music:stats:top_tracks:{window}:{limit}")
            }
            CacheKey::MusicListeners {
                window,
                sort,
                track,
                limit,
                offset,
            } => {
                let track = track.unwrap_or("all");
                format!("music:stats:listeners:{track}:{window}:{sort}:{limit}:{offset}")
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
            CacheKey::DpsCalculate {
                kind,
                data,
                body_hash,
            } => format!("dps:calc:{kind}:{ENGINE_VERSION}:{data}:{body_hash}"),
            CacheKey::DpsList { kind, data } => format!("dps:list:{kind}:{ENGINE_VERSION}:{data}"),
            // `on_page_grid` sets the TTL only: the context hash already tells
            // the two kinds of context apart.
            CacheKey::DpsRank {
                data, context_hash, ..
            } => format!("dps:rank:{ENGINE_VERSION}:{data}:{context_hash}"),
            CacheKey::I18nCatalog {
                locale,
                namespace,
                hash,
            } => format!("i18n:catalog:{locale}:{namespace}:{hash}"),
            CacheKey::I18nManifest => "i18n:manifest".to_owned(),
        }
    }

    pub fn ttl(&self) -> Duration {
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
            CacheKey::ItemLeaderboard { .. } => Duration::from_mins(5),
            CacheKey::ItemLeaderboardCatalog { .. } => Duration::from_mins(10),
            CacheKey::ItemLeaderboardStanding { .. } => Duration::from_mins(1),
            CacheKey::MusicStats => Duration::from_mins(1),
            CacheKey::MusicTopTracks { .. } => Duration::from_mins(5),
            CacheKey::MusicListeners { .. } => Duration::from_mins(5),
            CacheKey::SkinPopularity => Duration::from_hours(1),
            CacheKey::OperatorOwnership { .. } => Duration::from_hours(1),
            CacheKey::OperatorBuildStats { .. } => Duration::from_hours(1),
            CacheKey::CommunityEnemyAverage => Duration::from_mins(30),
            CacheKey::BaseRotation { .. } => Duration::from_mins(5),
            CacheKey::BaseOptimize { .. } => Duration::from_mins(5),
            // Deliberately SHORT, not for the roster (the key's version covers that).
            // `build_stage_improvements` (open events) and `build_medal_improvements`
            // (earnable vs missed medals) read the wall clock and change set
            // membership at a rotation boundary. This TTL is the only bound on
            // calling a closed event open.
            CacheKey::UserImprovements { .. } => Duration::from_mins(5),
            // Same hour as the list. Pure in body + game data, so only a reload
            // invalidates it, and `asset_watcher` clears the `dps:` prefix on one.
            CacheKey::DpsCalculate { .. } => Duration::from_hours(1),
            CacheKey::DpsList { .. } => Duration::from_hours(1),
            CacheKey::DpsRank { on_page_grid, .. } => {
                if *on_page_grid {
                    Duration::from_hours(24)
                } else {
                    *RANK_OFF_GRID_TTL
                }
            }
            CacheKey::I18nCatalog { .. } => Duration::from_hours(24), // content-addressed; cannot go stale
            CacheKey::I18nManifest => Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CacheKey, Duration, ENGINE_VERSION, off_grid_ttl};

    fn rank(on_page_grid: bool) -> CacheKey<'static> {
        CacheKey::DpsRank {
            data: "0123456789abcdef",
            context_hash: 42,
            on_page_grid,
        }
    }

    #[test]
    fn every_dps_key_carries_the_engine_version() {
        let calc = CacheKey::DpsCalculate {
            kind: "dps3",
            data: "d",
            body_hash: 7,
        };
        let list = CacheKey::DpsList {
            kind: "dps",
            data: "d",
        };
        assert_eq!(
            rank(true).to_key_string(),
            format!("dps:rank:{ENGINE_VERSION}:0123456789abcdef:42")
        );
        assert_eq!(
            calc.to_key_string(),
            format!("dps:calc:dps3:{ENGINE_VERSION}:d:7")
        );
        assert_eq!(
            list.to_key_string(),
            format!("dps:list:dps:{ENGINE_VERSION}:d")
        );
        // The grid flag picks the TTL, not the key.
        assert_eq!(rank(true).to_key_string(), rank(false).to_key_string());
    }

    #[test]
    fn only_a_page_grid_context_keeps_a_day() {
        assert_eq!(rank(true).ttl(), Duration::from_hours(24));
        assert_eq!(rank(false).ttl(), Duration::from_hours(1));
    }

    #[test]
    fn the_off_grid_ttl_switch_reads_missing_and_bad_values_as_the_default() {
        assert_eq!(off_grid_ttl(None), Duration::from_hours(1));
        assert_eq!(off_grid_ttl(Some("86400")), Duration::from_hours(24));
        assert_eq!(off_grid_ttl(Some(" 600 ")), Duration::from_secs(600));
        for bad in ["", "0", "-5", "1h", "3.5"] {
            assert_eq!(off_grid_ttl(Some(bad)), Duration::from_hours(1), "{bad:?}");
        }
    }
}
