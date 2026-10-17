//! The worker's side of the backend API: posting with retries, the result spool, and the streamed answer deltas.

use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;
use trevor::llm::StreamEvent;



/// The worker's side of the backend API.
#[derive(Clone)]
pub(crate) struct Api {
    pub(crate) http: reqwest::Client,
    pub(crate) base: String,
    pub(crate) key: String,
    pub(crate) worker_id: String,
    /// The waits before each retry of a result POST (1, 4, 16 s).
    pub(crate) backoff: Vec<Duration>,
}

/// One POST's outcome.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Posted {
    Ok,
    /// A 4xx other than 408 and 429: sending it again would not help.
    Refused(u16, String),
    Failed(String),
}

impl Api {
    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}/api/trevor/worker/{path}", self.base)
    }

    pub(crate) async fn post(&self, path: &str, body: &serde_json::Value, timeout: Duration) -> Posted {
        let r = self.http.post(self.url(path))
            .header("x-trevor-worker-key", &self.key)
            .timeout(timeout)
            .json(body)
            .send()
            .await;
        match r {
            Ok(r) if r.status().is_success() => Posted::Ok,
            Ok(r) if r.status().is_client_error() && !matches!(r.status().as_u16(), 408 | 429) => {
                let s = r.status().as_u16();
                Posted::Refused(s, r.text().await.unwrap_or_default())
            }
            Ok(r) => Posted::Failed(format!("HTTP {}", r.status())),
            Err(e) => Posted::Failed(e.to_string()),
        }
    }

    /// POST with the result retries; the last outcome.
    pub(crate) async fn post_retrying(&self, path: &str, body: &serde_json::Value) -> Posted {
        let mut out = self.post(path, body, Duration::from_secs(30)).await;
        for wait in &self.backoff {
            if !matches!(out, Posted::Failed(_)) {
                break;
            }
            tokio::time::sleep(*wait).await;
            out = self.post(path, body, Duration::from_secs(30)).await;
        }
        out
    }
}

/// The worker id: `TREVOR_WORKER_ID`, else "trevor-<host name>" (stable across restarts, so a spooled result names the
/// worker that claimed its job).
pub(super) fn worker_id() -> String {
    if let Some(id) = trevor::env::non_empty("TREVOR_WORKER_ID") {
        return id;
    }
    let host = std::process::Command::new("hostname").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| "local".into());
    format!("trevor-{host}")
}

pub(super) fn spool_dir() -> PathBuf {
    PathBuf::from(trevor::env::non_empty("TREVOR_SPOOL_DIR").unwrap_or_else(|| "artifacts/spool".into()))
}

/// Post a result; when every retry fails, write it to the spool directory.
pub(crate) async fn deliver(api: &Api, spool: &Path, body: &serde_json::Value) -> Posted {
    let out = api.post_retrying("result", body).await;
    match &out {
        Posted::Ok => {}
        Posted::Refused(s, t) =>
            eprintln!("worker: the backend refused the result of {} (HTTP {s}): {t}", body["jobId"]),
        Posted::Failed(e) => {
            let id: String = body["jobId"].as_str().unwrap_or("job").chars()
                .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
                .collect();
            let path = spool.join(format!("{id}.json"));
            let written = std::fs::create_dir_all(spool)
                .and_then(|()| std::fs::write(&path, serde_json::to_vec(body).unwrap_or_default()));
            match written {
                Ok(()) => eprintln!("worker: posting the result of {id} failed ({e}); spooled to {}", path.display()),
                Err(w) => eprintln!("worker: posting the result of {id} failed ({e}) and spooling it failed too ({w})"),
            }
        }
    }
    out
}

/// Send every spooled result once; a sent or refused one is deleted. Returns (sent, kept).
pub(crate) async fn resend_spool(api: &Api, spool: &Path) -> (usize, usize) {
    let Ok(dir) = std::fs::read_dir(spool) else { return (0, 0) };
    let mut files: Vec<PathBuf> = dir
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    let (mut sent, mut kept) = (0, 0);
    for f in files {
        let Some(body) = std::fs::read(&f).ok()
            .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok()) else { continue };
        match api.post("result", &body, Duration::from_secs(30)).await {
            Posted::Ok | Posted::Refused(..) => {
                let _ = std::fs::remove_file(&f);
                sent += 1;
            }
            Posted::Failed(_) => kept += 1,
        }
    }
    (sent, kept)
}

/// What the stream of one job has sent: later calls of the job are sent only where they continue it.
#[derive(Default)]
pub(crate) struct DeltaState {
    sent: String,
    cur: String,
    stopped: bool,
}

impl DeltaState {
    /// The new text an event adds to what was sent (empty when none, or once a call has diverged).
    pub(crate) fn feed(&mut self, ev: StreamEvent) -> String {
        match ev {
            StreamEvent::Start => self.cur.clear(),
            StreamEvent::Piece(p) => self.cur.push_str(&p),
        }
        if self.stopped {
            return String::new();
        }
        if self.cur.len() > self.sent.len() {
            if self.cur.starts_with(&self.sent) {
                let add = self.cur[self.sent.len()..].to_owned();
                self.sent.push_str(&add);
                return add;
            }
            self.stopped = true;
        } else if !self.sent.starts_with(&self.cur) {
            self.stopped = true;
        }
        String::new()
    }
}

/// Post a job's deltas as they arrive (the events waiting at each send go in one delta). Ends when the sender is
/// dropped; returns the deltas sent.
pub(crate) async fn forward_deltas(api: Api, job_id: String, mut rx: UnboundedReceiver<StreamEvent>) -> u64 {
    let mut state = DeltaState::default();
    let mut seq = 0u64;
    let mut gone = false;
    while let Some(ev) = rx.recv().await {
        let mut text = state.feed(ev);
        while let Ok(ev) = rx.try_recv() {
            text.push_str(&state.feed(ev));
        }
        if text.is_empty() || gone {
            continue;
        }
        let body = serde_json::json!({"workerId": api.worker_id, "seq": seq, "text": text});
        match api.post(&format!("jobs/{job_id}/delta"), &body, Duration::from_secs(10)).await {
            Posted::Ok => seq += 1,
            Posted::Refused(404, _) => gone = true,
            other => {
                eprintln!("worker: delta {seq} of {job_id}: {other:?}");
                seq += 1;
            }
        }
    }
    seq
}
