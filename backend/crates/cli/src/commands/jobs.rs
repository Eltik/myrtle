use std::fmt::Write as _;

use anyhow::{Context, anyhow};
use clap::Subcommand;
use myrtle_jobs::{Job, JobFilter, JobKind, JobQueue, JobSpec, JobState, PgQueue};

use crate::Env;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Job counts by state then most recent jobs
    List {
        /// queued, running, succeeded, or dead
        #[arg(long, value_parser = parse_state)]
        state: Option<JobState>,
        /// Job kind, eg. `regrade_all`
        #[arg(long, value_parser = parse_kind)]
        kind: Option<JobKind>,
        /// Jobs to print
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Queue one job now
    Enqueue {
        #[arg(value_parser = parse_kind)]
        kind: JobKind,
        /// JSON payload, default {}
        #[arg(long, value_parser = parse_payload)]
        payload: Option<serde_json::Value>,
    },
}

pub async fn run(env: &Env, command: Command) -> anyhow::Result<()> {
    let queue = PgQueue::new(env.db().await?);
    match command {
        Command::List { state, kind, limit } => {
            let counts = queue.counts(kind).await?;
            let counts: Vec<String> = counts
                .iter()
                .map(|(state, n)| format!("{} {n}", state.as_str()))
                .collect();
            println!("{}", counts.join("  "));
            let jobs = queue.recent(JobFilter { state, kind, limit }).await?;
            for job in &jobs {
                println!("{}", line(job));
            }
        }
        Command::Enqueue { kind, payload } => {
            let mut spec = JobSpec::new(kind);
            if let Some(payload) = payload {
                spec.payload = payload;
            }
            let id = queue.enqueue(spec).await?;
            println!("queued {} as job {}", kind.as_str(), id.0);
        }
    }
    Ok(())
}

fn line(job: &Job) -> String {
    let mut line = format!(
        "{:>8}  {:<26} {:<9} {}/{}  {}",
        job.id.0,
        job.kind.as_str(),
        job.state.as_str(),
        job.attempts,
        job.max_attempts,
        job.run_at.format("%Y-%m-%d %H:%M:%S")
    );
    if let Some(server) = job.server {
        let _ = write!(line, "  {}", server.as_str());
    }
    if let Some(error) = &job.last_error {
        let _ = write!(line, "  {}", error.replace('\n', " "));
    }
    line
}

pub fn parse_kind(s: &str) -> Result<JobKind, String> {
    JobKind::parse(s).ok_or_else(|| {
        let names: Vec<&str> = JobKind::ALL.iter().map(|k| k.as_str()).collect();
        format!("unknown job kind {s:?}; one of {}", names.join(", "))
    })
}

pub fn parse_state(s: &str) -> Result<JobState, String> {
    JobState::parse(s).ok_or_else(|| {
        let names: Vec<&str> = JobState::ALL.iter().map(|s| s.as_str()).collect();
        format!("unknown job state {s:?}; one of {}", names.join(", "))
    })
}

fn parse_payload(s: &str) -> anyhow::Result<serde_json::Value> {
    let value: serde_json::Value = serde_json::from_str(s).context("payload is not JSON")?;
    value
        .is_object()
        .then_some(value)
        .ok_or_else(|| anyhow!("payload must be a JSON object"))
}
