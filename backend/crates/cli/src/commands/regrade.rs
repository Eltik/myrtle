use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use anyhow::bail;
use myrtle_app::context::GameData;
use myrtle_app::services::grading::{self, RegradeMode, RegradePass, regrade_pass};
use myrtle_db::repos::users;
use myrtle_types::UserId;

use crate::Env;

#[derive(Debug, clap::Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    /// Users graded at once; default from the jobs.regrade config
    #[arg(long, short = 'j', value_parser = clap::value_parser!(u16).range(1..))]
    pub concurrency: Option<u16>,
    /// Users fetched per page; default from the jobs.regrade config
    #[arg(long, value_parser = clap::value_parser!(i64).range(1..))]
    pub page_size: Option<i64>,
    /// Only accounts with this game uid
    #[arg(long)]
    pub uid: Option<String>,
    /// Only users on this server
    #[arg(long)]
    pub server: Option<String>,
    #[arg(long, conflicts_with = "enqueue")]
    pub dry_run: bool,
    /// Stop dispatching at first failure
    #[arg(long, conflicts_with = "enqueue")]
    pub fail_fast: bool,
    /// Recompute RA section only
    #[arg(long, conflicts_with = "enqueue")]
    pub only_sandbox: bool,
    /// Log running totals every N users; 0 never
    #[arg(long, default_value_t = 50, conflicts_with = "enqueue")]
    pub progress_every: u64,
    /// Queue regrade jobs instead of grading here
    #[arg(long, conflicts_with_all = ["concurrency", "page_size"])]
    pub enqueue: bool,
}

pub async fn run(env: &Env, args: Args) -> anyhow::Result<()> {
    let server = args
        .server
        .as_deref()
        .map(|s| env.server(Some(s)))
        .transpose()?;
    if args.enqueue {
        return enqueue(env, &args, server).await;
    }
    let ctx = env.ctx(GameData::Default).await?;
    let defaults = RegradePass::from_config(&ctx);
    let pass = RegradePass {
        mode: if args.only_sandbox {
            RegradeMode::Sandbox
        } else {
            RegradeMode::Full
        },
        server,
        uid: args.uid,
        concurrency: args.concurrency.map_or(defaults.concurrency, usize::from),
        page_size: args.page_size.unwrap_or(defaults.page_size),
        fail_fast: args.fail_fast,
        dry_run: args.dry_run,
        progress_every: args.progress_every,
    };

    let stop = Arc::new(AtomicBool::new(false));
    let on_signal = Arc::clone(&stop);
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::warn!("interrupted: draining in-flight users (Ctrl-C again to quit now)");
            on_signal.store(true, Ordering::SeqCst);
            if tokio::signal::ctrl_c().await.is_ok() {
                std::process::exit(130);
            }
        }
    });

    let started = Instant::now();
    let tally = regrade_pass(&ctx, &pass, &stop).await?;
    let secs = started.elapsed().as_secs_f64();
    println!(
        "{} {} of {} users, {} failed{} in {secs:.1}s{}",
        if pass.dry_run { "graded" } else { "regraded" },
        tally.regraded,
        tally.targets,
        tally.failed,
        if tally.stopped_early {
            ", stopped early"
        } else {
            ""
        },
        if pass.dry_run {
            " (dry run, nothing written)"
        } else {
            ""
        },
    );
    if tally.failed > 0 {
        bail!("{} user(s) failed", tally.failed);
    }
    if tally.stopped_early {
        bail!("stopped before every user was graded");
    }
    Ok(())
}

async fn enqueue(
    env: &Env,
    args: &Args,
    server: Option<myrtle_types::Server>,
) -> anyhow::Result<()> {
    let ctx = env.ctx(GameData::None).await?;
    let queued = match &args.uid {
        Some(uid) => {
            let keys = users::keys_by_uid(&mut *ctx.db.conn().await?, uid, server).await?;
            let mut queued = 0;
            for key in &keys {
                grading::enqueue_regrade(&ctx, UserId(key.id)).await?;
                queued += 1;
            }
            queued
        }
        None if server.is_some() => bail!("--enqueue takes --server only with --uid"),
        None => grading::enqueue_regrade_all(&ctx).await?,
    };
    println!("queued {queued} regrade job(s)");
    tracing::warn!("no job worker runs in the rewrite yet; queued jobs wait for one");
    Ok(())
}
