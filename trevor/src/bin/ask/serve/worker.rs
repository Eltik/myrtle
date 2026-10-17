//! `--worker URL`: the job loop, the heartbeat and its rung probe, the power guard's effect on jobs.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;
use trevor::llm::Llm;

use crate::cli::Profile;
use crate::guard::{self, GuardConfig, GuardView};
use crate::llama_server::connect_local;

use super::{AskRequest, Service};
use super::canary::{canary_passing, canary_path};
use super::delivery::{Api, Posted, deliver, forward_deltas, resend_spool, spool_dir, worker_id};
use super::service::fit_result;

/// A worker job (contract section 3).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkerJob {
    pub(crate) job_id: String,
    pub(crate) question: String,
    #[serde(default)]
    pub(crate) server: Option<String>,
    #[serde(default)]
    pub(crate) horizon_pos: Option<u32>,
    #[serde(default)]
    pub(crate) rung: Option<String>,
    #[serde(default)]
    pub(crate) profile: Option<String>,
    #[serde(default)]
    pub(crate) flags: Vec<String>,
}

/// The heartbeat's view of the rungs.
#[derive(Clone)]
pub(crate) struct Probe {
    pub(crate) http: reqwest::Client,
    pub(crate) llama: String,
    pub(crate) cf_models: Option<Vec<String>>,
    pub(crate) canary: PathBuf,
}

impl Probe {
    /// `cpu` is false while the power guard is tripped (contract v1.1 section 2.1).
    async fn rungs(&self, guard: Option<&GuardView>) -> serde_json::Value {
        let cf = self.cf_models.as_ref()
            .is_some_and(|m| canary_passing(&self.canary, m, trevor::util::unix_now()).is_some());
        let cpu = !guard.is_some_and(GuardView::tripped)
            && matches!(self.http.get(format!("{}/health", self.llama)).timeout(Duration::from_secs(5)).send().await,
                        Ok(r) if r.status().is_success());
        serde_json::json!({"cf": cf, "cpu": cpu})
    }
}

/// The heartbeat body; `guard` is None when the guard is off (`TREVOR_GUARD=0`), which leaves the v1 body unchanged.
pub(crate) fn heartbeat_body(worker_id: &str, rungs: serde_json::Value, corpus_version: &str, busy: Option<String>,
                             guard: Option<&GuardView>) -> serde_json::Value {
    let mut body = serde_json::json!({"workerId": worker_id, "rungs": rungs, "corpusVersion": corpus_version,
                                      "busyJobId": busy});
    if let Some(g) = guard {
        body["guard"] = serde_json::to_value(g.status()).unwrap_or_default();
    }
    body
}

/// A heartbeat every `every`, and one at once whenever the guard trips or recovers (the backend settles a claimed job's
/// rung by the last heartbeat, so a trip must not wait up to 30 s to be seen).
pub(crate) async fn heartbeat_loop(api: Api, probe: Probe, corpus_version: String, busy: Arc<Mutex<Option<String>>>,
                                   every: Duration, mut guard: Option<tokio::sync::watch::Receiver<GuardView>>) {
    loop {
        let view = guard.as_mut().map(|g| g.borrow_and_update().clone());
        let rungs = probe.rungs(view.as_ref()).await;
        let job = busy.lock().ok().and_then(|b| b.clone());
        let body = heartbeat_body(&api.worker_id, rungs, &corpus_version, job, view.as_ref());
        if trevor::env::is("TREVOR_HEARTBEAT_LOG", "1") {
            eprintln!("worker: heartbeat {body}");
        }
        match api.post("heartbeat", &body, Duration::from_secs(15)).await {
            Posted::Ok => {}
            other => eprintln!("worker: heartbeat: {other:?}"),
        }
        match guard.as_mut() {
            Some(g) => {
                tokio::select! {
                    () = tokio::time::sleep(every) => {}
                    r = g.changed() => if r.is_err() { guard = None; },
                }
            }
            None => tokio::time::sleep(every).await,
        }
    }
}

/// The profile of a job that names none: `full` on the cf rung; on the others `TREVOR_CPU_PROFILE` (default `mid`,
/// contract v1.1 section 4; `slim` is the v1 default, the kill switch).
pub(crate) fn default_profile(rung: &str, cpu_profile: Option<&str>) -> String {
    if rung == "cf" {
        return "full".into();
    }
    cpu_profile.map(str::trim).filter(|p| Profile::parse(p).is_some()).unwrap_or("mid").to_owned()
}

impl Service {
    /// The local llama-server for the `cpu` rung: the one connected at load, or (when it was down then) a connection
    /// made now.
    async fn local(&mut self) -> Option<Llm> {
        if self.llm.is_none() {
            if let Ok(l) = Llm::connect(&self.a.server).await {
                self.llm = Some(if self.a.no_cache_prompt { l.without_prompt_cache() } else { l });
            }
        }
        self.llm.clone()
    }

    /// One worker job: the result body to post. With the power guard on, a `cpu` job never runs the local model while
    /// it is tripped, and a trip during the answer stops it: the job is answered with passages instead.
    async fn run_job(&mut self, api: &Api, job: WorkerJob, guard: Option<&tokio::sync::watch::Receiver<GuardView>>)
                     -> serde_json::Value {
        let rung = job.rung.clone().unwrap_or_else(|| "cpu".into());
        let fail =
            |e: String| serde_json::json!({"jobId": job.job_id, "workerId": api.worker_id, "ok": false, "error": e});
        let profile = match job.profile.as_deref() {
            Some(p) => p.to_owned(),
            None => default_profile(&rung, trevor::env::var("TREVOR_CPU_PROFILE").as_deref()),
        };
        let guarded = (rung == "cpu").then_some(guard).flatten();
        if guarded.is_some_and(|g| g.borrow().tripped()) {
            eprintln!("worker: the power guard is tripped; job {} gets passages instead of the cpu rung", job.job_id);
            return self.passages_body(api, &job, &AskRequest {
                question: job.question.clone(), horizon: None, horizon_pos: job.horizon_pos, server: job.server.clone(),
                flags: job.flags.clone(), profile: Some(profile)
            });
        }
        let req = AskRequest { question: job.question.clone(), horizon: None, horizon_pos: job.horizon_pos,
                               server: job.server.clone(), flags: job.flags.clone(), profile: Some(profile) };
        let llm = match rung.as_str() {
            "cf" => self.cf.clone().inspect(|l| {
                if let Some(id) = canary_passing(&canary_path(), &l.model_ids(), trevor::util::unix_now()) {
                    l.prefer_model(&id);
                }
            }),
            "cpu" => self.local().await,
            "passages" => None,
            other => return fail(format!("unknown rung {other:?}")),
        };
        let answered = match llm {
            Some(llm) => {
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                let fwd = (!self.a.no_stream).then(|| {
                    llm.set_stream(Some(tx));
                    tokio::spawn(forward_deltas(api.clone(), job.job_id.clone(), rx))
                });
                let r = match guarded {
                    None => self.ask(req.clone(), Some(llm.clone())).await,
                    Some(g) => {
                        let mut g = g.clone();
                        let r = tokio::select! {
                            r = self.ask(req.clone(), Some(llm.clone())) => Some(r),
                            _ = g.wait_for(GuardView::tripped) => None,
                        };
                        match r {
                            Some(r) => r,
                            None => {
                                // The answer future was dropped mid-await: undo what `ask` restores after it.
                                self.reset_after_cancel();
                                Err((503, "the power guard tripped during the answer".to_owned()))
                            }
                        }
                    }
                };
                llm.set_stream(None);
                if let Some(f) = fwd {
                    let _ = tokio::time::timeout(Duration::from_secs(30), f).await;
                }
                match r {
                    Ok(j) => Ok((j, rung.clone())),
                    Err((400, e)) => return fail(e),
                    Err((_, e)) => {
                        eprintln!("worker: job {} failed on the {rung} rung ({e}); answering with passages",
                                  job.job_id);
                        self.passages(&req).map(|j| (j, "passages".to_owned()))
                    }
                }
            }
            None => {
                if rung != "passages" {
                    eprintln!("worker: no model for the {rung} rung; job {} gets passages", job.job_id);
                }
                self.passages(&req).map(|j| (j, "passages".to_owned()))
            }
        };
        match answered {
            Ok((j, used)) => fit_result(serde_json::json!({"jobId": job.job_id, "workerId": api.worker_id, "ok": true,
                                                           "result": self.worker_result(&j, &used)})),
            Err((_, e)) => fail(e),
        }
    }

    fn passages_body(&mut self, api: &Api, job: &WorkerJob, req: &AskRequest) -> serde_json::Value {
        match self.passages(req) {
            Ok(j) => fit_result(serde_json::json!({"jobId": job.job_id, "workerId": api.worker_id, "ok": true,
                                                   "result": self.worker_result(&j, "passages")})),
            Err((_, e)) => serde_json::json!({"jobId": job.job_id, "workerId": api.worker_id, "ok": false, "error": e}),
        }
    }

    /// After an answer future was dropped: the tool options and the main corpus as `ask` would have restored them.
    fn reset_after_cancel(&mut self) {
        self.tables.tools.opts = self.opts0;
        if self.rts.args.corpus != self.main_corpus {
            self.swap_back();
        }
    }

    /// Act on the guard between jobs: on a memory trip stop the llama-server this worker started; on recovery start it
    /// again (without waiting for the model lock: when another job holds it, the next turn tries again).
    async fn follow_guard(&mut self, view: &GuardView) {
        match &view.trip {
            Some(t) if t.memory && !self.guard_stopped => {
                if self.server.take().is_some() {
                    eprintln!("worker: memory trip: stopped the llama-server this worker started");
                    self.llm = None;
                } else {
                    eprintln!("worker: memory trip, but the llama-server at {} was not started by this worker \
                               (--share-server or already running); it is left running and no cpu job uses it \
                               until the guard recovers", self.a.server);
                }
                self.guard_stopped = true;
            }
            None if self.guard_stopped => {
                let mut a = self.a.clone();
                a.no_wait = true;
                match connect_local(&a).await {
                    Ok((l, s)) => {
                        eprintln!("worker: guard recovered; llama-server {}",
                                  if s.is_some() { "started again" } else { "connected" });
                        self.llm = Some(l);
                        self.server = s;
                        self.guard_stopped = false;
                    }
                    Err(e) => eprintln!("worker: guard recovered, but no llama-server yet ({e:#}); \
                                         trying again after the next poll"),
                }
            }
            _ => {}
        }
    }
}

/// `--worker`: long-poll the backend for jobs, answer each, post the result; the heartbeat runs in its own task.
pub(super) async fn pull(svc: &mut Service, base: &str) -> Result<()> {
    let key = trevor::env::non_empty("TREVOR_WORKER_KEY").context("--worker needs TREVOR_WORKER_KEY")?;
    let http = reqwest::Client::builder().timeout(Duration::from_secs(90)).build()?;
    let api = Api { http: http.clone(), base: base.trim_end_matches('/').to_owned(), key, worker_id: worker_id(),
                    backoff: [1, 4, 16].map(Duration::from_secs).to_vec() };
    let spool = spool_dir();
    let (sent, kept) = resend_spool(&api, &spool).await;
    if sent + kept > 0 {
        eprintln!("worker: spool {}: {sent} results sent, {kept} kept", spool.display());
    }
    let busy = Arc::new(Mutex::new(None::<String>));
    let every = Duration::from_secs(trevor::env::parse_or("TREVOR_HEARTBEAT_SECS", 30));
    let probe = Probe { http, llama: svc.llm.as_ref().and_then(Llm::llama_base).unwrap_or(&svc.a.server)
                                         .trim_end_matches('/').to_owned(),
                        cf_models: svc.cf.as_ref().map(Llm::model_ids), canary: canary_path() };
    let guard = GuardConfig::from_env().map(|cfg| {
        eprintln!("worker: power guard on ({cfg:?})");
        let (tx, rx) = tokio::sync::watch::channel(GuardView::default());
        tokio::spawn(guard::run(cfg, tx));
        rx
    });
    if guard.is_none() {
        eprintln!("worker: power guard off (TREVOR_GUARD=0)");
    }
    let hb = tokio::spawn(heartbeat_loop(api.clone(), probe, svc.corpus_version.clone(), Arc::clone(&busy), every,
                                         guard.clone()));
    eprintln!("worker {}: polling {} ({}; cf {}, local model {})", api.worker_id, api.base, svc.corpus_version,
              svc.cf.as_ref().map_or("off".to_owned(), |l| l.model_ids().join(",")),
              svc.llm.as_ref().map_or("none", |l| l.model.as_str()));
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    loop {
        if let Some(g) = &guard {
            let view = g.borrow().clone();
            svc.follow_guard(&view).await;
        }
        let next = serde_json::json!({"workerId": api.worker_id});
        let resp = tokio::select! {
            r = api.http.post(api.url("next")).header("x-trevor-worker-key", &api.key).json(&next).send() => r,
            _ = tokio::signal::ctrl_c() => break,
            _ = term.recv() => break,
        };
        let job = match resp {
            Ok(r) if r.status() == reqwest::StatusCode::NO_CONTENT => continue,
            Ok(r) if r.status().is_success() => match r.bytes().await {
                Ok(b) if b.is_empty() => continue,
                Ok(b) => serde_json::from_slice::<WorkerJob>(&b),
                Err(e) => { eprintln!("worker: reading a job failed: {e}"); continue; }
            },
            Ok(r) => {
                eprintln!("worker: next answered {}", r.status());
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
            Err(e) => {
                eprintln!("worker: next failed: {e}");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };
        let job = match job {
            Ok(j) => j,
            Err(e) => { eprintln!("worker: unreadable job: {e}"); continue; }
        };
        if let Ok(mut b) = busy.lock() {
            *b = Some(job.job_id.clone());
        }
        let body = svc.run_job(&api, job, guard.as_ref()).await;
        deliver(&api, &spool, &body).await;
        if let Ok(mut b) = busy.lock() {
            *b = None;
        }
    }
    hb.abort();
    eprintln!("worker: stopping");
    Ok(())
}
