//! The answering side of the service: loading, a question's args and horizon, the answer, the no-model passages
//! answer, citations, and the result's size limits.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use trevor::llm::{Llm, UsageSnapshot};
use trevor::search::horizon::{Horizon, ReleaseOrder};
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::Runtime;
use trevor::tools::Route;

use crate::answer::{Tables, answer_one, passage_label, route_checked};
use crate::chrono::Chrono;
use crate::cli::{Args, Profile, RetrievalKind, RouterKind, apply_profile, deep_for, p4_dir, retrieval_plan,
                 router_source, tool_options};
use crate::detect::source_kind;
use crate::kinds::is_generated;
use crate::retrieval::{Runtimes, load_names, load_runtime};
use crate::llama_server::{cloudflare, connect_local, connect_or_spawn};

use super::{
    AskRequest, AskResult, Citation, Job, JobError, LOAD_FLAGS, Need, PASSAGES_HEAD, PASSAGES_NONE, SERVER, Service,
    Timings, bad, corpus_version, failed, recorded_sha,
};

impl Service {
    /// Load everything once: the indexes of the main corpus (P3b by default; BM25 only with `--retrieval bm25`), the
    /// models `need` asks for (a llama-server is connected or started under the model lock and stopped when the service
    /// ends), the Cloudflare model list when configured, and the release order.
    pub(super) async fn load(a: Args, corpus_default: bool, tables: Tables, chrono: Chrono, need: Need)
        -> Result<Self> {
        let dense = a.retrieval == RetrievalKind::Hybrid;
        let main = load_runtime(&a.runtime, dense, None)?;
        let mut parts = vec![(a.runtime.corpus.display().to_string(), main.store.chunks_sha.clone())];
        let dir = p4_dir(&a);
        if let Some(sha) = recorded_sha(dir).filter(|_| std::path::Path::new(dir) != a.runtime.corpus) {
            parts.push((dir.to_owned(), sha));
        }
        let corpus_version = corpus_version(&parts);
        let rts = Runtimes { main, p4: None, art: None, p4_dir: p4_dir(&a), switch: corpus_default && router_source(&a),
                             args: a.runtime.clone(), horizon: None, dense };
        let (llm, server) = match need {
            Need::Model => {
                let (l, s) = connect_or_spawn(&a).await?;
                (Some(l), s)
            }
            Need::Local => match connect_local(&a).await {
                Ok((l, s)) => (Some(l), s),
                Err(e) => {
                    eprintln!("worker: no local model ({e:#}); the cpu rung is off until {} answers", a.server);
                    (None, None)
                }
            },
            Need::Nothing => (None, None),
        };
        let cf = cloudflare(&a)?;
        let order = Arc::new(ReleaseOrder::load(std::path::Path::new("."))?);
        eprintln!("horizon: {} stories on the gatePos scale, {} unplaced sources dated, {} eras",
                  order.gate().story_order.len(), order.dated_sources(), order.gate().eras.len());
        Ok(Self { argv: std::env::args().collect(), corpus_default, deep0: tables.tools.topics_deep,
                  opts0: tables.tools.opts, main_corpus: a.runtime.corpus.clone(), tables, rts, llm, cf,
                  names: load_names(), chrono, order, corpus_version, a, server, guard_stopped: false })
    }

    /// The job's arguments: the service's own, or (with job flags or another profile) the command line again plus the
    /// job's flags (refusing the ones that change what is loaded), with the job's profile applied.
    fn args_for(&self, flags: &[String], profile: Option<Profile>) -> Result<Args, JobError> {
        if flags.is_empty() && profile.is_none_or(|p| p == self.a.profile) {
            return Ok(self.a.clone());
        }
        for f in flags {
            let name = f.split('=').next().unwrap_or_default();
            if !f.starts_with("--") || LOAD_FLAGS.contains(&name) {
                return Err(bad(format!("flag {f:?} is not allowed in a job")));
            }
        }
        let m = <Args as clap::CommandFactory>::command()
            .try_get_matches_from(self.argv.iter().chain(flags))
            .map_err(bad)?;
        let mut a = <Args as clap::FromArgMatches>::from_arg_matches(&m).map_err(bad)?;
        a.relation_rule = !a.no_relation_rule;
        a.runtime = self.a.runtime.clone();
        a.retrieval = self.a.retrieval;
        let p = profile.unwrap_or(a.profile);
        apply_profile(&mut a, p);
        Ok(a)
    }

    /// The job's horizon: its gatePos, its story or group id, or the lower of the two.
    fn horizon_for(&self, req: &AskRequest) -> Result<Option<Arc<Horizon>>, JobError> {
        let by_id = req.horizon.as_deref().map(|h| self.order.horizon(h)).transpose().map_err(bad)?;
        let by_pos = req.horizon_pos.map(|p| self.order.horizon_at(p)).transpose().map_err(bad)?;
        Ok(match (by_id, by_pos) {
            (Some(a), Some(b)) => Some(if b.pos < a.pos { b } else { a }),
            (a, b) => a.or(b),
        }.map(Arc::new))
    }

    fn profile_of(req: &AskRequest) -> Result<Option<Profile>, JobError> {
        req.profile
            .as_deref()
            .map(|p| Profile::parse(p)
                .ok_or_else(|| bad(format!("profile {p:?} is not one of {}", Profile::NAMES.join(", ")))))
            .transpose()
    }

    /// Answer one job with `llm` (the service's own model when None).
    pub(crate) async fn ask(&mut self, req: AskRequest, llm: Option<Llm>) -> Result<Job, JobError> {
        let t0 = Instant::now();
        let server = req.server.as_deref().unwrap_or(SERVER);
        if server != SERVER {
            return Err(bad(format!("server {server:?} has no corpus (only \"{SERVER}\")")));
        }
        let q = req.question.trim().to_owned();
        if q.is_empty() {
            return Err(bad("empty question"));
        }
        let a = self.args_for(&req.flags, Self::profile_of(&req)?)?;
        let horizon = self.horizon_for(&req)?;
        let llm = llm.or_else(|| self.llm.clone()).ok_or((503, "no model is connected".to_owned()))?;
        let before = llm.usage();
        self.tables.tools.opts = tool_options(&a);
        let profile = a.profile;
        let res = self.answer(&q, &req, a, horizon, &llm, t0).await;
        self.tables.tools.opts = self.opts0;
        let (result, generated) = res?;
        Ok(Job { result, generated, usage: llm.usage() - before, profile, source: "live" })
    }

    async fn answer(&mut self, q: &str, req: &AskRequest, mut a: Args, horizon: Option<Arc<Horizon>>, llm: &Llm,
                    t0: Instant) -> Result<(AskResult, BTreeMap<String, String>), JobError> {
        if horizon.is_some() {
            // Built from every story: the chronology of the subject.
            a.no_chrono_subject = true;
        }
        let horizon_id = horizon.as_ref().map(|h| h.id.clone());
        self.rts.set_horizon(horizon);
        self.tables.topic = None;
        self.tables.route = None;
        self.tables.tools.topics_deep = self.deep0;
        let mut out = AskResult { answer: String::new(), citations: Vec::new(), passages: Vec::new(), route: None,
                                  table: false, flags: req.flags.clone(), form: None, horizon: horizon_id,
                                  server: SERVER.to_owned(),
                                  corpus_version: self.corpus_version.clone(), invalid_citations: 0, prompt_tokens: 0,
                                  timings: Timings { route_ms: 0.0, answer_ms: 0.0, total_ms: 0.0 } };
        // As the one-shot `ask "question"` does in `main`: P4 for a source question when the router names no source,
        // then route, then P4 when the route's source asks for it.
        let mut use_p4 = self.corpus_default && !router_source(&a) && source_kind(q).is_some();
        if !a.no_route {
            let router_llm = (a.router == RouterKind::Model).then_some(llm);
            self.tables.tools.topics_deep = deep_for(&a, &self.tables.tools, q);
            let r = route_checked(q, &a, &mut self.tables, router_llm).await.map_err(failed)?;
            out.timings.route_ms = r.ms;
            let route = match r.text {
                Some(text) if !self.rts.horizon_set() => {
                    out.answer = text;
                    out.route = Some(r.route);
                    out.table = true;
                    out.timings.total_ms = t0.elapsed().as_secs_f64() * 1000.0;
                    return Ok((out, BTreeMap::new()));
                }
                // Under a horizon a table answer is not served; the passages answer instead.
                Some(_) => Route::retrieve(),
                None => {
                    self.tables.topic = if self.rts.horizon_set() { None } else { r.topic };
                    r.route
                }
            };
            if self.corpus_default && retrieval_plan(q, &a, Some(&route)).0 == Some(true) {
                use_p4 = true;
            }
            out.route = Some(route.clone());
            self.tables.route = Some(route);
        }
        let mut once = a.clone();
        once.routed = true;
        let swapped = if use_p4 { self.swap_p4().map_err(failed)? } else { false };
        let res = answer_one(&mut self.rts, llm, &self.names, &self.chrono, q, &once, &mut self.tables).await;
        if swapped {
            self.swap_back();
        }
        let ans = res.map_err(failed)?;
        out.citations = ans.cited.iter().map(|id| self.citation(id)).collect();
        out.answer = ans.answer;
        out.passages = ans.passages;
        out.form = ans.form;
        out.invalid_citations = ans.invalid_citations;
        out.prompt_tokens = ans.prompt_tokens;
        out.timings.answer_ms = ans.ms;
        out.timings.total_ms = t0.elapsed().as_secs_f64() * 1000.0;
        Ok((out, ans.generated))
    }

    /// The `passages` rung (contract section 2): no model, the top `--passages-n` source passages within the horizon.
    pub(crate) fn passages(&mut self, req: &AskRequest) -> Result<Job, JobError> {
        let t0 = Instant::now();
        let q = req.question.trim().to_owned();
        if q.is_empty() {
            return Err(bad("empty question"));
        }
        let profile = Self::profile_of(req)?.unwrap_or(self.a.profile);
        let horizon = self.horizon_for(req)?;
        let horizon_id = horizon.as_ref().map(|h| h.id.clone());
        self.rts.set_horizon(horizon);
        let n = self.a.passages_n.max(1);
        let rt = self.rts.pick(None).map_err(failed)?;
        let hits = rt
            .retrieve(&q, &RetrievalConfig { mode: Mode::Hybrid, k: n * 4, ..RetrievalConfig::default() })
            .map_err(failed)?;
        let rows: Vec<usize> = hits.iter().map(|h| h.row).filter(|&r| !is_generated(&rt.store.chunks[r].group_id))
            .take(n).collect();
        let labelled: Vec<(String, String)> = rows.iter().map(|&r| {
            let c = &rt.store.chunks[r];
            (c.chunk_id.clone(), passage_label(&self.names, c))
        }).collect();
        let answer = if labelled.is_empty() { PASSAGES_NONE.to_owned() } else {
            let lines: Vec<String> = labelled.iter().enumerate()
                .map(|(i, (_, l))| format!("[{}] {l}", i + 1))
                .collect();
            format!("{PASSAGES_HEAD}\n\n{}", lines.join("\n"))
        };
        let ids: Vec<String> = labelled.into_iter().map(|(id, _)| id).collect();
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        let result = AskResult { answer, citations: ids.iter().map(|id| self.citation(id)).collect(), passages: ids,
                                 route: None, table: false, flags: req.flags.clone(), form: None, horizon: horizon_id,
                                 server: SERVER.to_owned(),
                                 corpus_version: self.corpus_version.clone(), invalid_citations: 0, prompt_tokens: 0,
                                 timings: Timings { route_ms: 0.0, answer_ms: ms, total_ms: ms } };
        Ok(Job { result, generated: BTreeMap::new(), usage: UsageSnapshot::default(), profile, source: "passages" })
    }

    /// Make P4 the main corpus for this job, as the one-shot `ask` loads it as its only corpus: loaded on first use
    /// (sharing the main corpus's query embedder), swapped with the main corpus, swapped back after. False when the main
    /// corpus already is P4.
    fn swap_p4(&mut self) -> Result<bool> {
        let dir = PathBuf::from(self.rts.p4_dir);
        if self.rts.args.corpus == dir {
            return Ok(false);
        }
        if self.rts.p4.is_none() {
            let mut ra = self.rts.args.clone();
            ra.corpus.clone_from(&dir);
            self.rts.p4 = Some(load_runtime(&ra, self.rts.dense, self.rts.main.shared_embedder())?);
        }
        let p4 = self.rts.p4.as_mut().expect("loaded above");
        std::mem::swap(&mut self.rts.main, p4);
        self.rts.args.corpus = dir;
        // A P4 loaded just now has no horizon yet; `answer_one` also reads `rts.main` directly, not only through `pick`.
        self.rts.set_horizon(self.rts.horizon.clone());
        Ok(true)
    }

    pub(super) fn swap_back(&mut self) {
        if let Some(p4) = self.rts.p4.as_mut() {
            std::mem::swap(&mut self.rts.main, p4);
        }
        self.rts.args.corpus.clone_from(&self.main_corpus);
    }

    fn runtimes(&self) -> impl Iterator<Item = &Runtime> {
        std::iter::once(&self.rts.main).chain(self.rts.p4.as_ref()).chain(self.rts.art.as_ref())
    }

    /// A cited passage's story and line span, from whichever loaded corpus holds it.
    fn citation(&self, id: &str) -> Citation {
        for rt in self.runtimes() {
            if let Some(c) = rt.store.row(id).map(|r| &rt.store.chunks[r]) {
                return Citation { story_id: Some(c.story_id.clone()), chunk_id: id.to_owned(),
                                  line_start: Some(c.line_start), line_end: Some(c.line_end) };
            }
        }
        Citation { story_id: None, chunk_id: id.to_owned(), line_start: None, line_end: None }
    }

    /// A passage's text: the chunk's, a generated passage's, or a topic summary's.
    fn passage_text(&self, id: &str, generated: &BTreeMap<String, String>) -> String {
        for rt in self.runtimes() {
            if let Some(r) = rt.store.row(id) {
                return rt.store.chunks[r].text.clone();
            }
        }
        if let Some(t) = generated.get(id) {
            return t.clone();
        }
        id.strip_prefix("topic:").and_then(|t| self.tables.tools.topic(t).ok()).map(|t| t.text).unwrap_or_default()
    }

    /// The cited passage's text hash for the cache carry-over (contract v1.1 section 3b): the chunk's own `contentSha`
    /// (sha256 of its text, first 16 hex) when a loaded corpus holds it, else the same hash of the passage text (a
    /// generated passage: topic, dossier, game-data note).
    fn passage_sha(&self, id: &str, text: &str) -> String {
        self.runtimes()
            .find_map(|rt| rt.store.row(id).map(|r| rt.store.chunks[r].content_sha.clone()))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| trevor::util::sha16(text))
    }

    pub(super) fn health(&self) -> serde_json::Value {
        serde_json::json!({"ok": true, "server": SERVER, "corpusVersion": self.corpus_version,
                           "model": self.llm.as_ref().map(|l| l.model.clone())})
    }

    /// The contract's `result` of an answered job.
    pub(super) fn worker_result(&self, job: &Job, rung: &str) -> serde_json::Value {
        let r = &job.result;
        let citations: Vec<serde_json::Value> = r.citations.iter().map(|c| {
            let n = r.passages.iter().position(|p| *p == c.chunk_id).map_or(0, |i| i + 1);
            let text = self.passage_text(&c.chunk_id, &job.generated);
            serde_json::json!({"n": n, "storyId": c.story_id, "chunkId": c.chunk_id, "lineStart": c.line_start,
                               "lineEnd": c.line_end, "sha": self.passage_sha(&c.chunk_id, &text),
                               "text": trim_at_line(&text, CITE_CHARS)})
        }).collect();
        serde_json::json!({"answer": r.answer, "rung": rung, "profile": job.profile.name(), "source": job.source,
                           "citations": citations, "route": r.route, "form": r.form, "corpusVersion": r.corpus_version,
                           "tokensIn": job.usage.tokens_in, "tokensOut": job.usage.tokens_out, "timings": r.timings})
    }
}

/// The longest citation text a result carries (contract section 3).
const CITE_CHARS: usize = 600;
/// The backend's result limit.
const RESULT_MAX: usize = 64 * 1024;

/// `text` cut to at most `max` characters at a line break, or at a word break when the last line break inside `max`
/// would keep under half of it (`TREVOR_CITE_WORD_CUT=0`: the line break wherever it is, else `max` characters).
pub(crate) fn trim_at_line(text: &str, max: usize) -> String {
    trim_text(text, max, trevor::env::on_unless_0("TREVOR_CITE_WORD_CUT"))
}

/// [`trim_at_line`] with the word-break rule chosen. Without it, an operator file (a title line, then one paragraph of
/// 1,000 characters) kept only its 25-character title: on P4x, 785 of the 19,604 chunks over 600 characters kept
/// under 100 of them and 2,021 under 300 (2026-10-16).
pub(crate) fn trim_text(text: &str, max: usize, word_cut: bool) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let end = text.char_indices().nth(max).map_or(text.len(), |(i, _)| i);
    let head = &text[..end];
    let floor = if word_cut { max / 2 } else { 0 };
    let keeps = |i: usize| i > 0 && (!word_cut || head[..i].trim_end().chars().count() >= floor);
    if let Some(i) = head.rfind('\n').filter(|&i| keeps(i)) {
        return head[..i].trim_end().to_owned();
    }
    if word_cut && let Some(i) = head.rfind(char::is_whitespace).filter(|&i| keeps(i)) {
        return head[..i].trim_end().to_owned();
    }
    head.to_owned()
}

/// A result body under the backend's 64 KB limit: citation texts are cut to 200 characters, then dropped, when needed.
pub(crate) fn fit_result(mut body: serde_json::Value) -> serde_json::Value {
    for cut in [Some(200), None] {
        if serde_json::to_vec(&body).map_or(0, |b| b.len()) <= RESULT_MAX {
            break;
        }
        if let Some(cs) = body["result"]["citations"].as_array_mut() {
            for c in cs {
                let t = c["text"].as_str().unwrap_or_default().to_owned();
                c["text"] = serde_json::json!(cut.map_or(String::new(), |n| trim_at_line(&t, n)));
            }
        }
    }
    body
}
