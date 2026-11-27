use std::collections::HashMap;
use std::io::{BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use futures_util::StreamExt;
use futures_util::stream;
use serde::{Deserialize, Serialize};

use crate::compare::{
    Body, Observed, compare_observed, is_json_type, normalize_content_type, observe, sha256_hex,
};
use crate::corpus::CorpusEntry;
use crate::http::{FetchError, Target};
use crate::report::{Outcome, Summary};
use crate::rules::Rules;

pub const RECORDING_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub tool: String,
    pub version: u32,
    pub base: String,
    pub corpus_sha256: String,
    pub entries: usize,
    pub recorded_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedResponse {
    pub id: String,
    pub route: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub len: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skipped: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordStats {
    pub total: usize,
    pub ok: usize,
    pub errors: usize,
    pub skipped: usize,
    pub body_bytes: u64,
}

pub enum Side<'a> {
    Live(&'a Target),
    Recorded(&'a Recording),
}

impl Side<'_> {
    pub fn label(&self) -> String {
        match self {
            Self::Live(t) => t.label.clone(),
            Self::Recorded(r) => format!("recording {} ({})", r.dir.display(), r.manifest.base),
        }
    }
}

enum SideResult {
    Ok(Observed),
    Skipped(String),
    Error(String),
}

pub fn corpus_hash(corpus: &[CorpusEntry]) -> String {
    let mut text = String::new();
    for e in corpus {
        text.push_str(&e.id);
        text.push('\n');
    }
    sha256_hex(text.as_bytes())
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn record_one(
    entry: &CorpusEntry,
    fetched: Result<crate::http::Fetched, FetchError>,
    dir: &Path,
    keep_binary: bool,
) -> Result<RecordedResponse> {
    let mut rec = RecordedResponse {
        id: entry.id.clone(),
        route: entry.route.clone(),
        path: entry.path.clone(),
        role: entry.role.clone(),
        status: None,
        content_type: None,
        sha256: None,
        len: None,
        body_file: None,
        elapsed_ms: None,
        error: None,
        skipped: None,
    };
    match fetched {
        Err(e @ FetchError::MissingToken { .. }) => rec.skipped = Some(e.to_string()),
        Err(FetchError::Transport(m)) => rec.error = Some(m),
        Ok(f) => {
            let ct = f.content_type.as_deref().map(normalize_content_type);
            let json = is_json_type(ct.as_deref());
            if json || keep_binary {
                let name = format!("bodies/{}.{}", entry.id, if json { "json" } else { "bin" });
                std::fs::write(dir.join(&name), &f.bytes)
                    .with_context(|| format!("writing {name}"))?;
                rec.body_file = Some(name);
            }
            rec.status = Some(f.status);
            rec.content_type = f.content_type;
            rec.sha256 = Some(sha256_hex(&f.bytes));
            rec.len = Some(f.bytes.len() as u64);
            rec.elapsed_ms = Some(f.elapsed_ms);
        }
    }
    Ok(rec)
}

pub async fn record(
    target: &Target,
    corpus: &[CorpusEntry],
    dir: &Path,
    concurrency: usize,
    keep_binary: bool,
    force: bool,
    progress: bool,
) -> Result<RecordStats> {
    let responses = dir.join("responses.jsonl");
    if responses.exists() && !force {
        bail!(
            "{} already holds a recording; pass --force to overwrite",
            dir.display()
        );
    }
    let bodies = dir.join("bodies");
    if force && bodies.exists() {
        std::fs::remove_dir_all(&bodies)
            .with_context(|| format!("clearing {}", bodies.display()))?;
    }
    std::fs::create_dir_all(&bodies).with_context(|| format!("creating {}", bodies.display()))?;

    let mut results: Vec<Option<RecordedResponse>> = vec![None; corpus.len()];
    let mut stats = RecordStats {
        total: corpus.len(),
        ..RecordStats::default()
    };
    let mut done = 0_usize;
    let mut s = stream::iter(corpus.iter().enumerate())
        .map(|(i, e)| async move { (i, e, target.fetch(e).await) })
        .buffer_unordered(concurrency.max(1));
    while let Some((i, entry, fetched)) = s.next().await {
        let rec = record_one(entry, fetched, dir, keep_binary)?;
        if rec.skipped.is_some() {
            stats.skipped += 1;
        } else if rec.error.is_some() {
            stats.errors += 1;
        } else {
            stats.ok += 1;
            stats.body_bytes += rec.len.unwrap_or(0);
        }
        results[i] = Some(rec);
        done += 1;
        if progress && (done.is_multiple_of(250) || done == corpus.len()) {
            eprintln!("  recorded {done}/{}", corpus.len());
        }
    }

    let file = std::fs::File::create(&responses)
        .with_context(|| format!("creating {}", responses.display()))?;
    let mut w = BufWriter::new(file);
    for rec in results.into_iter().flatten() {
        serde_json::to_writer(&mut w, &rec)?;
        w.write_all(b"\n")?;
    }
    w.flush()?;
    let manifest = Manifest {
        tool: "myrtle-parity".to_owned(),
        version: RECORDING_VERSION,
        base: target.label.clone(),
        corpus_sha256: corpus_hash(corpus),
        entries: corpus.len(),
        recorded_at_unix: now_unix(),
    };
    std::fs::write(
        dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(stats)
}

#[derive(Debug)]
pub struct Recording {
    pub dir: PathBuf,
    pub manifest: Manifest,
    by_id: HashMap<String, RecordedResponse>,
}

impl Recording {
    pub fn open(dir: &Path) -> Result<Self> {
        let manifest: Manifest =
            serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).with_context(
                || format!("{} is not a recording (no manifest.json)", dir.display()),
            )?)
            .context("parsing manifest.json")?;
        if manifest.version != RECORDING_VERSION {
            bail!(
                "recording version {} is not {RECORDING_VERSION}; re-record with this build",
                manifest.version
            );
        }
        let file =
            std::fs::File::open(dir.join("responses.jsonl")).context("opening responses.jsonl")?;
        let mut by_id = HashMap::new();
        for (n, line) in std::io::BufReader::new(file).lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let rec: RecordedResponse = serde_json::from_str(&line)
                .with_context(|| format!("responses.jsonl:{}", n + 1))?;
            by_id.insert(rec.id.clone(), rec);
        }
        Ok(Self {
            dir: dir.to_owned(),
            manifest,
            by_id,
        })
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    fn observed(&self, entry: &CorpusEntry) -> SideResult {
        let Some(rec) = self.by_id.get(&entry.id) else {
            return SideResult::Error(
                "not in recording (corpus changed since it was recorded?)".to_owned(),
            );
        };
        if let Some(reason) = &rec.skipped {
            return SideResult::Skipped(reason.clone());
        }
        if let Some(e) = &rec.error {
            return SideResult::Error(format!("recorded error: {e}"));
        }
        let Some(status) = rec.status else {
            return SideResult::Error("recording has no status".to_owned());
        };
        if let Some(name) = &rec.body_file {
            return match std::fs::read(self.dir.join(name)) {
                Ok(bytes) => SideResult::Ok(observe(status, rec.content_type.as_deref(), &bytes)),
                Err(e) => SideResult::Error(format!("reading {name}: {e}")),
            };
        }
        SideResult::Ok(Observed {
            status,
            content_type: rec.content_type.as_deref().map(normalize_content_type),
            body: Body::Opaque {
                sha256: rec.sha256.clone().unwrap_or_default(),
                len: rec.len.unwrap_or(0),
            },
        })
    }
}

async fn observe_side(side: &Side<'_>, entry: &CorpusEntry) -> SideResult {
    match side {
        Side::Recorded(r) => r.observed(entry),
        Side::Live(t) => match t.fetch(entry).await {
            Ok(f) => SideResult::Ok(observe(f.status, f.content_type.as_deref(), &f.bytes)),
            Err(e @ FetchError::MissingToken { .. }) => SideResult::Skipped(e.to_string()),
            Err(FetchError::Transport(m)) => SideResult::Error(m),
        },
    }
}

fn combine(left: SideResult, right: SideResult, rules: &Rules, route: &str) -> Outcome {
    match (left, right) {
        (SideResult::Skipped(reason), _) | (_, SideResult::Skipped(reason)) => {
            Outcome::Skipped(reason)
        }
        (SideResult::Error(err), _) => Outcome::Error(format!("A: {err}")),
        (_, SideResult::Error(err)) => Outcome::Error(format!("B: {err}")),
        (SideResult::Ok(seen_a), SideResult::Ok(seen_b)) => {
            if seen_a.status == 429 || seen_b.status == 429 {
                return Outcome::RateLimited {
                    a: seen_a.status,
                    b: seen_b.status,
                };
            }
            let cmp = compare_observed(&seen_a, &seen_b, rules.for_route(route));
            if cmp.is_equal() {
                Outcome::Equal
            } else {
                Outcome::Different(cmp)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompareRun {
    pub concurrency: usize,
    pub max_problems: usize,
    pub progress: bool,
}

pub async fn compare(
    a: &Side<'_>,
    b: &Side<'_>,
    corpus: &[CorpusEntry],
    rules: &Rules,
    run: &CompareRun,
) -> Summary {
    let mut outcomes: Vec<Option<Outcome>> = vec![None; corpus.len()];
    let mut done = 0_usize;
    let mut s = stream::iter(corpus.iter().enumerate())
        .map(|(i, e)| async move {
            if rules.skipped(&e.route) {
                return (i, Outcome::Skipped("skipped by rules".to_owned()));
            }
            let (x, y) = tokio::join!(observe_side(a, e), observe_side(b, e));
            (i, combine(x, y, rules, &e.route))
        })
        .buffer_unordered(run.concurrency.max(1));
    while let Some((i, outcome)) = s.next().await {
        outcomes[i] = Some(outcome);
        done += 1;
        if run.progress && (done.is_multiple_of(250) || done == corpus.len()) {
            eprintln!("  compared {done}/{}", corpus.len());
        }
    }
    let mut summary = Summary::new(&a.label(), &b.label(), rules.rel_tol());
    for (entry, outcome) in corpus.iter().zip(outcomes) {
        let outcome = outcome.unwrap_or_else(|| Outcome::Error("not run".to_owned()));
        summary.add(entry, outcome, run.max_problems);
    }
    summary
}
