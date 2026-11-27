use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use anyhow::{anyhow, bail};
use myrtle_app::AppContext;
use myrtle_app::context::GameData;
use myrtle_app::services::gacha::{pools, shops};
use myrtle_app::services::grading::{RegradePass, regrade_pass};
use myrtle_app::services::leaderboard::jobs;
use tokio::sync::Semaphore;

use crate::Env;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Show tasks and exit
    #[arg(long, short)]
    pub list: bool,
    /// Run every cheap task
    #[arg(long, short)]
    pub all: bool,
    /// With --all, also run heavy tasks
    #[arg(long, requires = "all")]
    pub include_heavy: bool,
    /// Tasks to run at once
    #[arg(long, short = 'j', default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..))]
    pub concurrency: u16,
    /// Names of tasks to run
    pub names: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Task {
    pub name: &'static str,
    pub about: &'static str,
    pub heavy: bool,
    pub data: GameData,
}

pub const TASKS: &[Task] = &[
    Task {
        name: "operator-ownership",
        about: "Operator ownership, E2 conversion, default skill and module shares",
        heavy: false,
        data: GameData::None,
    },
    Task {
        name: "medal-ownership",
        about: "Per-medal ownership counts",
        heavy: false,
        data: GameData::None,
    },
    Task {
        name: "trending",
        about: "Tier-list trending windows and scores",
        heavy: false,
        data: GameData::None,
    },
    Task {
        name: "leaderboard-snapshot",
        about: "Take a leaderboard snapshot (appends a row)",
        heavy: false,
        data: GameData::None,
    },
    Task {
        name: "gacha-details",
        about: "Gacha pool details for every server with an account (network)",
        heavy: true,
        data: GameData::All,
    },
    Task {
        name: "event-shops",
        about: "Event token shops for every server with an account (network)",
        heavy: true,
        data: GameData::All,
    },
    Task {
        name: "regrade",
        about: "Recompute every user's grade",
        heavy: true,
        data: GameData::Default,
    },
];

pub fn select(
    all: bool,
    include_heavy: bool,
    names: &[String],
) -> anyhow::Result<Vec<&'static Task>> {
    if all {
        if !names.is_empty() {
            bail!("pass either --all or task names, not both");
        }
        return Ok(TASKS.iter().filter(|t| include_heavy || !t.heavy).collect());
    }
    if names.is_empty() {
        bail!("nothing to do: name a task, or pass --all (try --list)");
    }
    names
        .iter()
        .map(|name| {
            TASKS
                .iter()
                .find(|t| t.name == name)
                .ok_or_else(|| anyhow!("unknown task {name} (try --list)"))
        })
        .collect()
}

#[must_use]
pub fn game_data(tasks: &[&Task]) -> GameData {
    if tasks.iter().any(|t| t.data == GameData::All) {
        GameData::All
    } else if tasks.iter().any(|t| t.data == GameData::Default) {
        GameData::Default
    } else {
        GameData::None
    }
}

pub async fn run(env: &Env, args: Args) -> anyhow::Result<()> {
    if args.list {
        print_tasks();
        return Ok(());
    }
    let selected = select(args.all, args.include_heavy, &args.names)?;
    let ctx = env.ctx(game_data(&selected)).await?;
    println!(
        "running {} task(s) with concurrency {}",
        selected.len(),
        args.concurrency
    );
    let permits = Arc::new(Semaphore::new(usize::from(args.concurrency)));
    let started = Instant::now();
    let mut handles = Vec::with_capacity(selected.len());
    for task in selected {
        let permit = Arc::clone(&permits).acquire_owned().await?;
        let ctx = ctx.clone();
        handles.push(tokio::spawn(async move {
            let _permit = permit;
            let began = Instant::now();
            let outcome = run_task(&ctx, task.name).await;
            (task.name, began.elapsed(), outcome)
        }));
    }
    let mut failed = 0;
    for handle in handles {
        let (name, elapsed, outcome) = handle.await?;
        let secs = elapsed.as_secs_f64();
        match outcome {
            Ok(summary) => println!("ok    {name} ({secs:.2}s): {summary}"),
            Err(e) => {
                failed += 1;
                println!("FAIL  {name} ({secs:.2}s): {e:#}");
            }
        }
    }
    println!("finished in {:.2}s", started.elapsed().as_secs_f64());
    if failed > 0 {
        bail!("{failed} task(s) failed");
    }
    Ok(())
}

async fn run_task(ctx: &AppContext, name: &str) -> anyhow::Result<String> {
    Ok(match name {
        "operator-ownership" => jobs::refresh_operator_ownership(ctx).await?,
        "medal-ownership" => jobs::refresh_medal_ownership(ctx, true).await?,
        "trending" => jobs::recompute_trending(ctx).await?,
        "leaderboard-snapshot" => jobs::snapshot_leaderboard(ctx, true).await?,
        "gacha-details" => {
            let n = pools::refresh_pool_details(ctx, None).await?;
            format!("{n} pool details fetched")
        }
        "event-shops" => {
            let n = shops::refresh_event_shops(ctx, None).await?;
            format!("{n} event shops fetched")
        }
        "regrade" => {
            let pass = RegradePass::from_config(ctx);
            let tally = regrade_pass(ctx, &pass, &AtomicBool::new(false)).await?;
            let summary = format!(
                "regraded {} of {} users (concurrency {})",
                tally.regraded, tally.targets, pass.concurrency
            );
            if tally.failed > 0 {
                bail!("{summary}, {} failed", tally.failed);
            }
            summary
        }
        other => bail!("no runner for task {other}"),
    })
}

fn print_tasks() {
    let width = TASKS.iter().map(|t| t.name.len()).max().unwrap_or(0);
    for task in TASKS {
        let tag = if task.heavy { "HEAVY" } else { "     " };
        println!("{:<width$}  {tag}  {}", task.name, task.about);
    }
    println!("\n--all skips HEAVY tasks unless --include-heavy is given");
}
