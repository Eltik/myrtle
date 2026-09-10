//! Registry of every statistic a background job would otherwise refresh.
//!
//! Each job in this module tree exposes one `refresh_once(&AppState)` that does
//! a single pass and returns a one-line summary. The job's own loop calls it,
//! and so does anything that wants to force a refresh out of band: the
//! `refresh-stats` binary today, an admin endpoint tomorrow. That shared call is
//! the point. A forced refresh that re-implemented the work would drift from the
//! scheduled one the first time either changed, and the drift would be silent.
//!
//! Adding a job is two lines: a `refresh_once` beside its loop, and an entry in
//! [`TASKS`]. Nothing that consumes the registry needs editing, because every
//! consumer iterates it rather than naming tasks individually.

use crate::app::state::AppState;
use crate::core::{
    gacha_detail_job, leaderboard_snapshot_job, medal_ownership_job, operator_ownership_job,
    regrade_job, trending_job,
};
use std::future::Future;
use std::pin::Pin;

/// A boxed future so [`RefreshTask::run`] can be a plain function pointer and
/// [`TASKS`] can stay a `const` slice. Without the box each async fn would have
/// its own opaque type and the entries could not share one table.
pub type TaskFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send + 'a>>;

/// What running a task costs, which decides whether `--all` includes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cost {
    /// A bounded amount of work, near enough constant in the user count: one or
    /// a few aggregate queries. Safe to run at any time.
    Cheap,
    /// Work that grows with the number of users or makes network calls. Excluded
    /// from a bare `--all` so the safe thing stays the default thing.
    Heavy,
}

pub struct RefreshTask {
    /// Stable kebab-case identifier. This is the CLI argument, so renaming one
    /// breaks anybody's scripts; add an alias instead.
    pub name: &'static str,
    pub about: &'static str,
    pub cost: Cost,
    pub run: for<'a> fn(&'a AppState) -> TaskFuture<'a>,
}

/// Every forceable statistic, in the order `--all` runs them.
///
/// Ordering is deliberate rather than alphabetical: ownership aggregates feed
/// the operator pages, so they come first and a partial run still leaves the
/// most-read surfaces current.
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
        name: "regrade",
        about: "Recompute every user's grade (walks the whole users table)",
        cost: Cost::Heavy,
        run: |state| Box::pin(regrade_job::refresh_once(state)),
    },
];

/// Look one up by name. Returns `None` rather than a fuzzy match, so a typo
/// fails loudly instead of refreshing something the caller did not ask for.
pub fn find(name: &str) -> Option<&'static RefreshTask> {
    TASKS.iter().find(|t| t.name == name)
}

/// The tasks a bare `--all` runs: everything whose cost does not scale with the
/// user count or reach the network.
pub fn default_set() -> impl Iterator<Item = &'static RefreshTask> {
    TASKS.iter().filter(|t| t.cost == Cost::Cheap)
}
