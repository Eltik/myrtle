use chrono::{DateTime, Utc};
use myrtle_types::{JobId, Server, WorkerId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::PgRow;
use sqlx::types::Uuid;
use sqlx::{FromRow, Row};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    RarityReconcile,
    DpsUpstreamWatch,
    TrendingRecompute,
    LeaderboardSnapshot,
    OperatorOwnershipStats,
    MedalOwnershipStats,
    RegradeAll,
    RegradeUser,
    SandboxBackfill,
    GachaDetailRefresh,
    EventShopRefresh,
    MaintenancePoll,
    SirenSync,
    StoryCommunityAggregate,
    ReleaseLedgerRecord,
    ReleaseArtMirror,
    MusicStatDedupeSweep,
    ServiceAccountAlert,
}

impl JobKind {
    pub const ALL: [Self; 18] = [
        Self::RarityReconcile,
        Self::DpsUpstreamWatch,
        Self::TrendingRecompute,
        Self::LeaderboardSnapshot,
        Self::OperatorOwnershipStats,
        Self::MedalOwnershipStats,
        Self::RegradeAll,
        Self::RegradeUser,
        Self::SandboxBackfill,
        Self::GachaDetailRefresh,
        Self::EventShopRefresh,
        Self::MaintenancePoll,
        Self::SirenSync,
        Self::StoryCommunityAggregate,
        Self::ReleaseLedgerRecord,
        Self::ReleaseArtMirror,
        Self::MusicStatDedupeSweep,
        Self::ServiceAccountAlert,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RarityReconcile => "rarity_reconcile",
            Self::DpsUpstreamWatch => "dps_upstream_watch",
            Self::TrendingRecompute => "trending_recompute",
            Self::LeaderboardSnapshot => "leaderboard_snapshot",
            Self::OperatorOwnershipStats => "operator_ownership_stats",
            Self::MedalOwnershipStats => "medal_ownership_stats",
            Self::RegradeAll => "regrade_all",
            Self::RegradeUser => "regrade_user",
            Self::SandboxBackfill => "sandbox_backfill",
            Self::GachaDetailRefresh => "gacha_detail_refresh",
            Self::EventShopRefresh => "event_shop_refresh",
            Self::MaintenancePoll => "maintenance_poll",
            Self::SirenSync => "siren_sync",
            Self::StoryCommunityAggregate => "story_community_aggregate",
            Self::ReleaseLedgerRecord => "release_ledger_record",
            Self::ReleaseArtMirror => "release_art_mirror",
            Self::MusicStatDedupeSweep => "music_stat_dedupe_sweep",
            Self::ServiceAccountAlert => "service_account_alert",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == s)
    }

    #[must_use]
    pub const fn lease_secs(self) -> u32 {
        match self {
            Self::RegradeAll
            | Self::SirenSync
            | Self::StoryCommunityAggregate
            | Self::ReleaseArtMirror => 600,
            _ => 120,
        }
    }

    #[must_use]
    pub const fn max_attempts(self) -> u16 {
        match self {
            Self::TrendingRecompute | Self::MaintenancePoll => 1,
            _ => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Succeeded,
    Dead,
}

impl JobState {
    pub const ALL: [Self; 4] = [Self::Queued, Self::Running, Self::Succeeded, Self::Dead];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Dead => "dead",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|state| state.as_str() == s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    pub kind: JobKind,
    pub server: Option<Server>,
    pub payload: Value,
    /// At most one queued job per (kind, key)
    pub dedupe_key: Option<String>,
    pub priority: i16,
    pub run_at: Option<DateTime<Utc>>,
}

impl JobSpec {
    #[must_use]
    pub fn new(kind: JobKind) -> Self {
        Self {
            kind,
            server: None,
            payload: Value::Object(serde_json::Map::new()),
            dedupe_key: None,
            priority: 0,
            run_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub kind: JobKind,
    pub server: Option<Server>,
    pub payload: Value,
    pub state: JobState,
    pub priority: i16,
    pub attempts: u16,
    pub max_attempts: u16,
    pub lease_secs: u32,
    pub run_at: DateTime<Utc>,
    pub leased_by: Option<WorkerId>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl FromRow<'_, PgRow> for Job {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let kind: &str = row.try_get("kind")?;
        let state: &str = row.try_get("state")?;
        Ok(Self {
            id: JobId(row.try_get("id")?),
            kind: JobKind::parse(kind).ok_or_else(|| unknown("kind", kind))?,
            server: row
                .try_get::<Option<i16>, _>("server_id")?
                .and_then(Server::from_index),
            payload: row.try_get("payload")?,
            state: JobState::parse(state).ok_or_else(|| unknown("state", state))?,
            priority: row.try_get("priority")?,
            attempts: row.try_get::<i16, _>("attempts")? as u16,
            max_attempts: row.try_get::<i16, _>("max_attempts")? as u16,
            lease_secs: row.try_get::<i32, _>("lease_secs")? as u32,
            run_at: row.try_get("run_at")?,
            leased_by: row.try_get::<Option<Uuid>, _>("leased_by")?.map(WorkerId),
            lease_expires_at: row.try_get("lease_expires_at")?,
            last_error: row.try_get("last_error")?,
            created_at: row.try_get("created_at")?,
            finished_at: row.try_get("finished_at")?,
        })
    }
}

fn unknown(column: &str, value: &str) -> sqlx::Error {
    sqlx::Error::Decode(format!("unknown job {column} {value:?}").into())
}
