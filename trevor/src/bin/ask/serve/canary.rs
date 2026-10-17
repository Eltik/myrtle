//! The Cloudflare model canary (`ask --canary`): which configured model last passed, and when.

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use trevor::llm::OpenAiConfig;

use crate::answer::Tables;
use crate::chrono::Chrono;
use crate::cli::Args;
use crate::llama_server::openai_llm;

use super::{AskRequest, Need, Service};
use super::service::trim_at_line;

/// The canary result (`ask --canary`).
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanaryFile {
    /// Unix seconds.
    pub(crate) checked_at: u64,
    pub(crate) models: Vec<CanaryModel>,
    /// The first model id that passed, None when none did.
    pub(crate) passing: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanaryModel {
    pub(crate) id: String,
    pub(crate) passed: bool,
    /// Answers naming the expected term with at least one valid citation and none invalid.
    pub(crate) hits: usize,
    pub(crate) errors: Vec<String>,
    pub(crate) json_calls: u64,
    pub(crate) json_failures: u64,
    pub(crate) tokens_in: u64,
    pub(crate) tokens_out: u64,
    pub(crate) secs: f64,
    pub(crate) answers: Vec<String>,
}

/// The 10 fixed canary questions and a term a right answer names (any of them).
const CANARY: [(&str, &[&str]); 10] = [
    ("Who is the leader of Reunion?", &["talulah"]),
    ("What is the name of the creature that protects Kal'tsit?", &["mon3tr"]),
    ("Which city's Guard Department does Ch'en work for?", &["lungmen"]),
    ("What disease do the Infected suffer from?", &["oripathy"]),
    ("What is the name of the Ursus city Reunion attacks at the start of the story?", &["chernobog"]),
    ("What material causes Oripathy?", &["originium"]),
    ("Who is the boss of Penguin Logistics?", &["emperor"]),
    ("What is the name of the company the Doctor works with?", &["rhodes island"]),
    ("Who was the leader of Babel?", &["theresa"]),
    ("What race is Amiya?", &["cautus", "chimera"]),
];
/// A model passes with no failed call and at least this many hits.
const CANARY_PASS: usize = 8;

pub(super) fn canary_path() -> PathBuf {
    PathBuf::from(trevor::env::non_empty("TREVOR_CANARY_FILE").unwrap_or_else(|| "artifacts/canary/cf.json".into()))
}


/// The model id the last canary passed, when it is in `models` and the canary is at most `TREVOR_CANARY_MAX_AGE_DAYS`
/// (8) days old.
pub(crate) fn canary_passing(path: &Path, models: &[String], now: u64) -> Option<String> {
    let f: CanaryFile = trevor::util::read_json_opt(path)?;
    let max_days: u64 = trevor::env::parse_or("TREVOR_CANARY_MAX_AGE_DAYS", 8);
    let id = f.passing?;
    (now.saturating_sub(f.checked_at) <= max_days * 86_400 && models.contains(&id)).then_some(id)
}

/// `ask --canary`: the 10 questions on each configured Cloudflare model id in order (full profile, at the horizon of the
/// last dated story, so every answer comes from the model and not a table), until one passes.
pub(crate) async fn run_canary(a: Args, corpus_default: bool, tables: Tables, chrono: Chrono) -> Result<()> {
    let base = OpenAiConfig::cloudflare_from_env()
        .context("--canary needs TREVOR_CF_MODELS and TREVOR_CF_ACCOUNT_ID (or TREVOR_CF_BASE_URL)")?;
    let mut svc = Service::load(a, corpus_default, tables, chrono, Need::Nothing).await?;
    let gate = svc.order.gate();
    let last = u32::try_from(gate.story_order.len() - gate.never.len()).unwrap_or(u32::MAX);
    let mut out = CanaryFile { checked_at: trevor::util::unix_now(), ..CanaryFile::default() };
    for id in &base.models {
        let llm = openai_llm(&svc.a, OpenAiConfig { models: vec![id.clone()], ..base.clone() })?;
        let t = Instant::now();
        let mut m = CanaryModel { id: id.clone(), ..CanaryModel::default() };
        for (q, expect) in CANARY {
            let req = AskRequest { question: q.into(), horizon: None, horizon_pos: Some(last), server: None, flags: Vec::new(),
                                   profile: Some("full".into()) };
            match svc.ask(req, Some(llm.clone())).await {
                Ok(job) => {
                    let ans = job.result.answer.to_lowercase();
                    let cited = !job.result.citations.is_empty() && job.result.invalid_citations == 0;
                    if cited && expect.iter().any(|e| ans.contains(e)) {
                        m.hits += 1;
                    }
                    m.answers.push(trim_at_line(&job.result.answer, 300));
                }
                Err((_, e)) => {
                    m.errors.push(format!("{q}: {e}"));
                    m.answers.push(String::new());
                }
            }
        }
        let u = llm.usage();
        (m.json_calls, m.json_failures, m.tokens_in, m.tokens_out) = (u.json_calls, u.json_failures, u.tokens_in, u.tokens_out);
        m.secs = t.elapsed().as_secs_f64();
        m.passed = m.errors.is_empty() && m.hits >= CANARY_PASS;
        eprintln!("canary {id}: {} of {} hits, {} errors, {} of {} JSON calls failed, {:.0} s: {}", m.hits, CANARY.len(), m.errors.len(),
                  m.json_failures, m.json_calls, m.secs, if m.passed { "PASS" } else { "fail" });
        let passed = m.passed;
        out.models.push(m);
        if passed {
            out.passing = Some(id.clone());
            break;
        }
    }
    let path = canary_path();
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&out)?)?;
    std::fs::rename(&tmp, &path)?;
    println!("{}: passing {:?}", path.display(), out.passing);
    Ok(())
}
