use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use myrtle_db::repos::users::{self, KeyRow};
use myrtle_types::{Server, UserId};
use tokio::task::{JoinError, JoinSet};

use super::{compute_grade, compute_sandbox, regrade, rescore_sandbox};
use crate::{AppContext, AppError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RegradeMode {
    #[default]
    Full,
    Sandbox,
}

#[derive(Debug, Clone)]
pub struct RegradePass {
    pub mode: RegradeMode,
    pub server: Option<Server>,
    pub uid: Option<String>,
    pub concurrency: usize,
    pub page_size: i64,
    pub fail_fast: bool,
    pub dry_run: bool,
    pub progress_every: u64,
}

impl RegradePass {
    #[must_use]
    pub fn from_config(ctx: &AppContext) -> Self {
        let job = &ctx.config.jobs.regrade;
        Self {
            mode: RegradeMode::Full,
            server: None,
            uid: None,
            concurrency: job.concurrency.max(1),
            page_size: job.page_size.max(1),
            fail_fast: false,
            dry_run: false,
            progress_every: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegradeTally {
    pub targets: i64,
    pub regraded: u64,
    pub failed: u64,
    pub stopped_early: bool,
}

type Outcome = (KeyRow, Result<String, AppError>);

pub async fn regrade_pass(
    ctx: &AppContext,
    pass: &RegradePass,
    stop: &AtomicBool,
) -> Result<RegradeTally, AppError> {
    let targets = {
        let mut conn = ctx.db.conn().await?;
        match &pass.uid {
            Some(uid) => users::count_by_uid(&mut conn, uid, pass.server).await?,
            None => users::count(&mut conn, pass.server).await?,
        }
    };
    tracing::info!(targets, dry_run = pass.dry_run, "regrade pass starting");
    let mut run = Run {
        pass,
        started: Instant::now(),
        tally: RegradeTally {
            targets,
            ..RegradeTally::default()
        },
        abort: false,
    };
    let mut tasks: JoinSet<Outcome> = JoinSet::new();
    let mut after: Option<UserId> = None;
    let mut fetched_uid = false;

    let result = 'pages: loop {
        let page = match next_page(ctx, pass, after, &mut fetched_uid).await {
            Ok(page) if page.is_empty() => break Ok(()),
            Ok(page) => page,
            Err(e) => break Err(e),
        };
        after = page.last().map(|k| UserId(k.id));
        for key in page {
            while tasks.len() >= pass.concurrency.max(1) {
                if let Some(done) = tasks.join_next().await {
                    run.record(done);
                }
            }
            while let Some(done) = tasks.try_join_next() {
                run.record(done);
            }
            if run.abort || stop.load(Ordering::Relaxed) {
                run.tally.stopped_early = true;
                break 'pages Ok(());
            }
            let ctx = ctx.clone();
            let (mode, dry_run) = (pass.mode, pass.dry_run);
            tasks.spawn(async move {
                let outcome = grade_one(&ctx, UserId(key.id), mode, dry_run).await;
                (key, outcome)
            });
        }
    };
    while let Some(done) = tasks.join_next().await {
        run.record(done);
    }
    result?;
    let elapsed = run.started.elapsed().as_secs_f64();
    tracing::info!(
        regraded = run.tally.regraded,
        failed = run.tally.failed,
        stopped_early = run.tally.stopped_early,
        elapsed_s = format!("{elapsed:.2}"),
        "regrade pass done"
    );
    Ok(run.tally)
}

async fn next_page(
    ctx: &AppContext,
    pass: &RegradePass,
    after: Option<UserId>,
    fetched_uid: &mut bool,
) -> Result<Vec<KeyRow>, AppError> {
    let mut conn = ctx.db.conn().await?;
    Ok(match &pass.uid {
        Some(_) if *fetched_uid => Vec::new(),
        Some(uid) => {
            *fetched_uid = true;
            users::keys_by_uid(&mut conn, uid, pass.server).await?
        }
        None => users::keys_page(&mut conn, pass.server, after, pass.page_size.max(1)).await?,
    })
}

async fn grade_one(
    ctx: &AppContext,
    user: UserId,
    mode: RegradeMode,
    dry_run: bool,
) -> Result<String, AppError> {
    if mode == RegradeMode::Sandbox {
        let rescored = if dry_run {
            compute_sandbox(ctx, user).await?.map(|r| r.grade)
        } else {
            rescore_sandbox(ctx, user).await?
        };
        if let Some(grade) = rescored {
            return Ok(grade);
        }
    }
    let grade = if dry_run {
        compute_grade(ctx, user).await?
    } else {
        regrade(ctx, user).await?
    };
    Ok(grade.overall)
}

struct Run<'a> {
    pass: &'a RegradePass,
    started: Instant,
    tally: RegradeTally,
    abort: bool,
}

impl Run<'_> {
    fn record(&mut self, done: Result<Outcome, JoinError>) {
        match done {
            Ok((key, Ok(grade))) => {
                self.tally.regraded += 1;
                tracing::debug!(uid = %key.uid, user_id = %key.id, %grade, "regraded");
            }
            Ok((key, Err(e))) => {
                self.tally.failed += 1;
                self.abort |= self.pass.fail_fast;
                tracing::error!(uid = %key.uid, user_id = %key.id, error = %e, "regrade failed");
            }
            Err(e) => {
                self.tally.failed += 1;
                self.abort |= self.pass.fail_fast;
                tracing::error!(error = %e, "regrade task panicked");
            }
        }
        let processed = self.tally.regraded + self.tally.failed;
        let every = self.pass.progress_every;
        if every > 0 && processed.is_multiple_of(every) {
            let rate = f64::from(u32::try_from(processed).unwrap_or(u32::MAX))
                / self.started.elapsed().as_secs_f64().max(0.001);
            tracing::info!(
                processed,
                of = self.tally.targets,
                failed = self.tally.failed,
                per_s = format!("{rate:.2}"),
                "regrade progress"
            );
        }
    }
}
