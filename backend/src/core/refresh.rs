//! Registry of every statistic a background job refreshes.
//!
//! Each job exposes one `refresh_once(&AppState)` that does a pass and returns a
//! one-line summary. Its own loop calls it, and so does any forced refresh (the
//! `refresh-stats` binary). A forced refresh that re-implemented the work would
//! silently drift from the scheduled one.
//!
//! Adding a job: a `refresh_once` beside its loop and an entry in [`TASKS`].
//! Consumers iterate the registry, so nothing else changes.

use crate::app::state::AppState;
use crate::core::{
    event_shop_job, gacha_detail_job, leaderboard_snapshot_job, medal_ownership_job,
    operator_ownership_job, regrade_job, trending_job,
};
use std::future::Future;
use std::pin::Pin;

/// Boxed so [`RefreshTask::run`] is a plain fn pointer and [`TASKS`] a `const`
/// slice; each async fn otherwise has its own opaque type.
pub type TaskFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send + 'a>>;

/// Decides whether `--all` includes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cost {
    /// Roughly constant in user count (a few aggregate queries). Safe any time.
    Cheap,
    /// Grows with user count or hits the network. Left out of a bare `--all` so the
    /// safe thing is the default.
    Heavy,
}

pub struct RefreshTask {
    /// Kebab-case CLI argument; renaming breaks scripts, so add an alias instead.
    pub name: &'static str,
    pub about: &'static str,
    pub cost: Cost,
    pub run: for<'a> fn(&'a AppState) -> TaskFuture<'a>,
}

/// Every forceable statistic, in `--all` order. Ownership aggregates feed the
/// operator pages, so they go first and a partial run keeps the most-read
/// surfaces current.
pub const TASKS: &[RefreshTask] = &[
    RefreshTask {
        name: "operator-ownership",
        about: "Operator ownership, E2 conversion, and the default skill/module distributions",
        cost: Cost::Cheap,
        run: |state| Box::pin(operator_ownership_job::refresh_once(state)),
    },
    RefreshTask {
        name: "medal-ownership",
        about: "Per-medal ownership counts",
        cost: Cost::Cheap,
        run: |state| Box::pin(medal_ownership_job::refresh_once(state)),
    },
    RefreshTask {
        name: "trending",
        about: "Tier-list trending windows and scores",
        cost: Cost::Cheap,
        run: |state| Box::pin(trending_job::refresh_once(state)),
    },
    RefreshTask {
        name: "leaderboard-snapshot",
        about: "Take a leaderboard snapshot (appends a row; not idempotent)",
        cost: Cost::Cheap,
        run: |state| Box::pin(leaderboard_snapshot_job::refresh_once(state)),
    },
    RefreshTask {
        name: "gacha-details",
        about: "Gacha pool details for every configured server (makes network calls)",
        cost: Cost::Heavy,
        run: |state| Box::pin(gacha_detail_job::refresh_once(state)),
    },
    RefreshTask {
        name: "event-shops",
        about: "Event token shops for every configured server (makes network calls)",
        cost: Cost::Heavy,
        run: |state| Box::pin(event_shop_job::refresh_once(state)),
    },
    RefreshTask {
        name: "regrade",
        about: "Recompute every user's grade (walks the whole users table)",
        cost: Cost::Heavy,
        run: |state| Box::pin(regrade_job::refresh_once(state)),
    },
];

/// Exact match only, so a typo fails loudly instead of refreshing something else.
pub fn find(name: &str) -> Option<&'static RefreshTask> {
    TASKS.iter().find(|t| t.name == name)
}

/// What a bare `--all` runs: tasks that don't scale with users or hit the network.
pub fn default_set() -> impl Iterator<Item = &'static RefreshTask> {
    TASKS.iter().filter(|t| t.cost == Cost::Cheap)
}
