#![allow(clippy::doc_markdown)]

use std::collections::BTreeSet;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use regex::Regex;
use serde_json::Value;

use myrtle_parity::DEFAULT_SEED;
use myrtle_parity::corpus::{self, CorpusEntry, ExpandOptions, RouteMatcher};
use myrtle_parity::http::{HttpOptions, Side as TokenSide, Target, Tokens};
use myrtle_parity::ids::{self, IdSet};
use myrtle_parity::nginx::{self, LogOptions};
use myrtle_parity::openapi::check_superset;
use myrtle_parity::report::Summary;
use myrtle_parity::rules::{RuleOverrides, Rules, RulesFile};
use myrtle_parity::run::{self, CompareRun, Recording, Side};

#[derive(Parser)]
#[command(
    name = "parity",
    version,
    about = "Parity harness for the Myrtle backend rewrite"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Ids {
        #[arg(long)]
        excel: PathBuf,
        #[arg(long)]
        merge: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
    },
    Corpus(CorpusArgs),
    Record {
        #[arg(long)]
        base: String,
        #[arg(long)]
        corpus: PathBuf,
        #[arg(long)]
        out: PathBuf,
        /// Also store non-JSON bodies (images, audio)
        #[arg(long)]
        keep_binary: bool,
        /// Overwrite an existing recording in --out.
        #[arg(long)]
        force: bool,
        #[command(flatten)]
        http: HttpArgs,
    },
    /// Fetch the corpus from a backend and compare with a recording
    Replay {
        #[arg(long)]
        base: String,
        #[arg(long)]
        corpus: PathBuf,
        #[arg(long)]
        against: PathBuf,
        #[command(flatten)]
        http: HttpArgs,
        #[command(flatten)]
        compare: CompareArgs,
    },
    /// Fetch the corpus from two live backends and compare
    Diff {
        #[arg(long)]
        a: String,
        #[arg(long)]
        b: String,
        #[arg(long)]
        corpus: PathBuf,
        #[command(flatten)]
        http: HttpArgs,
        #[command(flatten)]
        compare: CompareArgs,
    },
    /// Compare two recordings offline
    Compare {
        #[arg(long)]
        a: PathBuf,
        #[arg(long)]
        b: PathBuf,
        #[arg(long)]
        corpus: PathBuf,
        #[command(flatten)]
        compare: CompareArgs,
    },
    /// Fail if the new OpenAPI document drops an operation or schema name of the old one
    OpenapiSuperset {
        #[arg(long)]
        old: PathBuf,
        #[arg(long)]
        new: PathBuf,
        /// Also write the report as JSON
        #[arg(long)]
        json: Option<PathBuf>,
    },
}

#[derive(Args)]
struct CorpusArgs {
    /// OpenAPI document
    #[arg(long)]
    openapi: Option<PathBuf>,
    /// Ids file from `parity ids`
    #[arg(long)]
    ids: Option<PathBuf>,
    /// At most N requests per route
    #[arg(long)]
    sample: Option<u64>,
    #[arg(long, default_value_t = DEFAULT_SEED)]
    seed: u64,
    /// Signed-in role to emit bearer-token GETs for
    #[arg(long = "role")]
    roles: Vec<String>,
    /// Keep only routes whose template matches one of these regexes
    #[arg(long)]
    include: Vec<String>,
    /// Drop routes whose template matches one of these regexes
    #[arg(long)]
    exclude: Vec<String>,
    /// nginx access log in combined format
    #[arg(long = "nginx-log")]
    nginx_logs: Vec<PathBuf>,
    /// At most N distinct log paths
    #[arg(long)]
    log_sample: Option<u64>,
    #[arg(long, default_value = "/api/")]
    log_prefix: String,
    /// Also write the per-route expansion plan
    #[arg(long)]
    plan: Option<PathBuf>,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Args)]
struct HttpArgs {
    /// Requests in flight per side
    #[arg(long, default_value_t = 4)]
    concurrency: usize,
    /// Per-request timeout in seconds
    #[arg(long, default_value_t = 30.0)]
    timeout_secs: f64,
    /// Requests per second per side
    #[arg(long, default_value_t = 0.0)]
    rps: f64,
    /// Ignore HTTP_PROXY / HTTPS_PROXY
    #[arg(long)]
    no_proxy: bool,
    /// No progress lines on stderr
    #[arg(long)]
    quiet: bool,
}

impl HttpArgs {
    fn options(&self) -> Result<HttpOptions> {
        if !(self.timeout_secs > 0.0 && self.timeout_secs.is_finite()) {
            bail!("--timeout-secs must be > 0");
        }
        Ok(HttpOptions {
            concurrency: self.concurrency.max(1),
            timeout: Duration::from_secs_f64(self.timeout_secs),
            rps: self.rps,
            no_proxy: self.no_proxy,
        })
    }
}

#[derive(Args)]
struct CompareArgs {
    /// Relative float tolerance (default 1e-9)
    #[arg(long)]
    rel_tol: Option<f64>,
    /// Absolute float tolerance floor (default 0)
    #[arg(long)]
    abs_tol: Option<f64>,
    /// JSON-pointer glob of an array compared as a multiset (repeatable)
    #[arg(long)]
    unordered: Vec<String>,
    /// JSON-pointer glob of values not compared (repeatable)
    #[arg(long)]
    ignore: Vec<String>,
    /// Rules file (tolerances, globs, per-route rules)
    #[arg(long)]
    rules: Option<PathBuf>,
    /// Problems listed in the report
    #[arg(long, default_value_t = 20)]
    max_problems: usize,
    /// Diffs listed per problem
    #[arg(long, default_value_t = 10)]
    max_diffs: usize,
    /// Write the machine-readable summary here
    #[arg(long)]
    summary: Option<PathBuf>,
}

impl CompareArgs {
    fn rules(&self) -> Result<Rules> {
        let file = self.rules.as_deref().map(RulesFile::load).transpose()?;
        Rules::build(
            file,
            &RuleOverrides {
                rel_tol: self.rel_tol,
                abs_tol: self.abs_tol,
                unordered: self.unordered.clone(),
                ignore: self.ignore.clone(),
                max_diffs: self.max_diffs,
            },
        )
    }
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}

fn regexes(list: &[String]) -> Result<Vec<Regex>> {
    list.iter()
        .map(|r| Regex::new(r).with_context(|| format!("bad regex {r:?}")))
        .collect()
}

fn corpus_roles(entries: &[CorpusEntry]) -> BTreeSet<&str> {
    entries.iter().filter_map(|e| e.role.as_deref()).collect()
}

fn finish(summary: &Summary, compare: &CompareArgs) -> Result<bool> {
    print!("{}", summary.render_text());
    if let Some(path) = &compare.summary {
        std::fs::write(path, serde_json::to_vec_pretty(summary)?)
            .with_context(|| format!("writing {}", path.display()))?;
        eprintln!("summary written to {}", path.display());
    }
    Ok(summary.passed())
}

fn warn_missing_tokens(roles: &BTreeSet<&str>, tokens: &Tokens, side: Option<TokenSide>) {
    for role in roles {
        if tokens.get(role).is_none() {
            let names = myrtle_parity::http::token_env_names(role, side).join(" or ");
            eprintln!(
                "warning: no token for role `{role}` ({names} unset); its requests are skipped"
            );
        }
    }
}

fn cmd_ids(excel: &Path, merge: &[PathBuf], out: &Path) -> Result<bool> {
    let (mut set, report) = ids::extract_from_excel(excel)?;
    for m in merge {
        set.merge(IdSet::load(m)?);
    }
    std::fs::write(out, serde_json::to_vec_pretty(&set)?)
        .with_context(|| format!("writing {}", out.display()))?;
    eprintln!(
        "read {} tables: {}",
        report.tables_read.len(),
        report.tables_read.join(", ")
    );
    if !report.tables_missing.is_empty() {
        eprintln!(
            "missing (kinds from them stay empty): {}",
            report.tables_missing.join(", ")
        );
    }
    for (kind, values) in &set.kinds {
        eprintln!("  {kind:<18} {:>6}", values.len());
    }
    let empty: Vec<&str> = set
        .kinds
        .iter()
        .filter(|(_, v)| v.is_empty())
        .map(|(k, _)| k.as_str())
        .collect();
    if !empty.is_empty() {
        eprintln!(
            "empty kinds (fill by hand or --merge; routes needing them are skipped): {}",
            empty.join(", ")
        );
    }
    eprintln!("wrote {}", out.display());
    Ok(true)
}

fn cmd_corpus(args: &CorpusArgs) -> Result<bool> {
    if args.openapi.is_none() && args.nginx_logs.is_empty() {
        bail!("give --openapi, --nginx-log, or both");
    }
    let doc = args.openapi.as_deref().map(read_json).transpose()?;
    let ids = args
        .ids
        .as_deref()
        .map(IdSet::load)
        .transpose()?
        .unwrap_or_default();
    let mut entries: Vec<CorpusEntry> = Vec::new();
    if let Some(doc) = &doc {
        let opts = ExpandOptions {
            sample: args.sample,
            seed: args.seed,
            roles: args.roles.clone(),
            include: regexes(&args.include)?,
            exclude: regexes(&args.exclude)?,
        };
        let exp = corpus::expand_openapi(doc, &ids, &opts);
        let expanded = exp.plans.iter().filter(|p| p.skipped.is_none()).count();
        eprintln!(
            "openapi: {} GET routes, {expanded} expanded, {} requests",
            exp.plans.len(),
            exp.entries.len()
        );
        for (reason, routes) in corpus::skipped_by_reason(&exp.plans) {
            eprintln!("  skipped {} routes: {reason}", routes.len());
            for r in routes.iter().take(6) {
                eprintln!("      {r}");
            }
            if routes.len() > 6 {
                eprintln!("      ... {} more", routes.len() - 6);
            }
        }
        if let Some(plan) = &args.plan {
            std::fs::write(plan, serde_json::to_vec_pretty(&exp.plans)?)
                .with_context(|| format!("writing {}", plan.display()))?;
        }
        entries.extend(exp.entries);
    }
    let matcher = doc.as_ref().map(RouteMatcher::from_openapi);
    let log_opts = LogOptions {
        prefix: args.log_prefix.clone(),
        sample: args.log_sample,
        seed: args.seed,
    };
    for log in &args.nginx_logs {
        let (found, stats) = if log.as_os_str() == "-" {
            nginx::import(std::io::stdin().lock(), &log_opts, matcher.as_ref())?
        } else {
            let f =
                std::fs::File::open(log).with_context(|| format!("opening {}", log.display()))?;
            nginx::import(BufReader::new(f), &log_opts, matcher.as_ref())?
        };
        eprintln!(
            "nginx {}: {}",
            log.display(),
            serde_json::to_string(&stats)?
        );
        let known: BTreeSet<String> = entries.iter().map(|e| e.id.clone()).collect();
        entries.extend(found.into_iter().filter(|e| !known.contains(&e.id)));
    }
    corpus::write_corpus(&args.out, &entries)?;
    eprintln!("wrote {} requests to {}", entries.len(), args.out.display());
    Ok(true)
}

async fn cmd_record(
    base: &str,
    corpus_path: &Path,
    out: &Path,
    keep_binary: bool,
    force: bool,
    http: &HttpArgs,
) -> Result<bool> {
    let entries = corpus::read_corpus(corpus_path)?;
    let roles = corpus_roles(&entries);
    let tokens = Tokens::from_env(roles.iter().copied(), None);
    warn_missing_tokens(&roles, &tokens, None);
    let target = Target::new(base, None, tokens, &http.options()?)?;
    let stats = run::record(
        &target,
        &entries,
        out,
        http.concurrency,
        keep_binary,
        force,
        !http.quiet,
    )
    .await?;
    eprintln!(
        "recorded {} requests to {}: {} ok, {} errors, {} skipped, {} body bytes",
        stats.total,
        out.display(),
        stats.ok,
        stats.errors,
        stats.skipped,
        stats.body_bytes
    );
    Ok(stats.ok > 0 || stats.total == 0)
}

async fn cmd_replay(
    base: &str,
    corpus_path: &Path,
    against: &Path,
    http: &HttpArgs,
    compare: &CompareArgs,
) -> Result<bool> {
    let entries = corpus::read_corpus(corpus_path)?;
    let recording = Recording::open(against)?;
    if recording.manifest.corpus_sha256 != run::corpus_hash(&entries) {
        eprintln!("warning: corpus differs from the one recorded; entries are matched by id");
    }
    let roles = corpus_roles(&entries);
    let tokens = Tokens::from_env(roles.iter().copied(), None);
    warn_missing_tokens(&roles, &tokens, None);
    let target = Target::new(base, None, tokens, &http.options()?)?;
    let rules = compare.rules()?;
    let run = CompareRun {
        concurrency: http.concurrency,
        max_problems: compare.max_problems,
        progress: !http.quiet,
    };
    let summary = run::compare(
        &Side::Recorded(&recording),
        &Side::Live(&target),
        &entries,
        &rules,
        &run,
    )
    .await;
    finish(&summary, compare)
}

async fn cmd_diff(
    a: &str,
    b: &str,
    corpus_path: &Path,
    http: &HttpArgs,
    compare: &CompareArgs,
) -> Result<bool> {
    let entries = corpus::read_corpus(corpus_path)?;
    let roles = corpus_roles(&entries);
    let opts = http.options()?;
    let ta = Tokens::from_env(roles.iter().copied(), Some(TokenSide::A));
    let tb = Tokens::from_env(roles.iter().copied(), Some(TokenSide::B));
    warn_missing_tokens(&roles, &ta, Some(TokenSide::A));
    warn_missing_tokens(&roles, &tb, Some(TokenSide::B));
    let target_a = Target::new(a, Some(TokenSide::A), ta, &opts)?;
    let target_b = Target::new(b, Some(TokenSide::B), tb, &opts)?;
    let rules = compare.rules()?;
    let run = CompareRun {
        concurrency: opts.concurrency,
        max_problems: compare.max_problems,
        progress: !http.quiet,
    };
    let summary = run::compare(
        &Side::Live(&target_a),
        &Side::Live(&target_b),
        &entries,
        &rules,
        &run,
    )
    .await;
    finish(&summary, compare)
}

async fn cmd_compare(
    a: &Path,
    b: &Path,
    corpus_path: &Path,
    compare: &CompareArgs,
) -> Result<bool> {
    let entries = corpus::read_corpus(corpus_path)?;
    let ra = Recording::open(a)?;
    let rb = Recording::open(b)?;
    let rules = compare.rules()?;
    let run = CompareRun {
        concurrency: 1,
        max_problems: compare.max_problems,
        progress: false,
    };
    let summary = run::compare(
        &Side::Recorded(&ra),
        &Side::Recorded(&rb),
        &entries,
        &rules,
        &run,
    )
    .await;
    finish(&summary, compare)
}

fn cmd_superset(old: &Path, new: &Path, json: Option<&Path>) -> Result<bool> {
    let report = check_superset(&read_json(old)?, &read_json(new)?);
    print!("{}", report.render_text());
    if let Some(path) = json {
        std::fs::write(path, serde_json::to_vec_pretty(&report)?)
            .with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(report.ok())
}

async fn dispatch(cli: Cli) -> Result<bool> {
    match cli.cmd {
        Cmd::Ids { excel, merge, out } => cmd_ids(&excel, &merge, &out),
        Cmd::Corpus(args) => cmd_corpus(&args),
        Cmd::Record {
            base,
            corpus,
            out,
            keep_binary,
            force,
            http,
        } => cmd_record(&base, &corpus, &out, keep_binary, force, &http).await,
        Cmd::Replay {
            base,
            corpus,
            against,
            http,
            compare,
        } => cmd_replay(&base, &corpus, &against, &http, &compare).await,
        Cmd::Diff {
            a,
            b,
            corpus,
            http,
            compare,
        } => cmd_diff(&a, &b, &corpus, &http, &compare).await,
        Cmd::Compare {
            a,
            b,
            corpus,
            compare,
        } => cmd_compare(&a, &b, &corpus, &compare).await,
        Cmd::OpenapiSuperset { old, new, json } => cmd_superset(&old, &new, json.as_deref()),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("parity: error: starting runtime: {e}");
            return ExitCode::from(2);
        }
    };
    match runtime.block_on(dispatch(cli)) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("parity: error: {e:#}");
            ExitCode::from(2)
        }
    }
}
