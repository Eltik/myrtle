//! A worker's result: size-checked, parsed, stored as the job's answer.

use serde_json::Value;

use crate::app::error::ApiError;
use crate::app::services::trevor::runtime::StreamEvent;
use crate::app::services::trevor::{
    AnswerSource, CITATION_TEXT_CHARS, Rung, WorkerAnswer, neurons, trim_citation_text,
};
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

use super::WorkerResult;
use super::queue::check_worker_id;

/// `worker/result`, from the raw body so its size is checked first: a body
/// over `TREVOR_RESULT_MAX_BYTES` is a 413 (the route maps
/// [`ResultError::TooLarge`]). The answer is stored under the corpus
/// version the worker reports; an answer from a version that is not the
/// active one still reaches the job's poller but is never served from cache.
pub async fn result(state: &AppState, body: &[u8]) -> Result<(), ResultError> {
    let cap = state.trevor.config().result_max_bytes;
    if cap > 0 && body.len() > cap {
        return Err(ResultError::TooLarge(cap));
    }
    let res: WorkerResult = serde_json::from_slice(body)
        .map_err(|e| ApiError::BadRequest(format!("result body: {e}")))?;
    store_result(state, res).await.map_err(ResultError::Api)
}

/// Why a result was refused.
#[derive(Debug)]
pub enum ResultError {
    TooLarge(usize),
    Api(ApiError),
}

impl From<ApiError> for ResultError {
    fn from(e: ApiError) -> Self {
        Self::Api(e)
    }
}

async fn store_result(state: &AppState, res: WorkerResult) -> Result<(), ApiError> {
    check_worker_id(&res.worker_id)?;
    if !res.ok {
        let error = res.error.unwrap_or_else(|| "the worker failed".to_owned());
        if !q::fail_job(&state.db, res.job_id, &res.worker_id, &error).await? {
            return Err(ApiError::Conflict(
                "job is not running under this worker".into(),
            ));
        }
        state.trevor.finish(res.job_id, StreamEvent::Error(error));
        return Ok(());
    }
    let mut answer = res
        .result
        .ok_or_else(|| ApiError::BadRequest("ok result without `result`".into()))?;
    if answer.rung == Rung::Cache {
        return Err(ApiError::BadRequest(
            "result.rung must be cf, cpu or passages".into(),
        ));
    }
    if answer.corpus_version.is_empty() || answer.corpus_version.len() > 100 {
        return Err(ApiError::BadRequest(
            "result.corpusVersion is missing or too long".into(),
        ));
    }
    for c in &mut answer.citations {
        c.text = trim_citation_text(&c.text, CITATION_TEXT_CHARS);
    }
    let cited = cited_shas(&answer);
    let stored = serde_json::to_value(&answer).map_err(|e| ApiError::Internal(e.into()))?;
    let cfg = state.trevor.config();
    let cost = neurons(answer.tokens_in, answer.tokens_out);
    let Some((_, job)) = q::complete_job(
        &state.db,
        res.job_id,
        &res.worker_id,
        &q::CompletedAnswer {
            version: &answer.corpus_version,
            rung: answer.rung.as_str(),
            source: answer.source.as_str(),
            answer: &stored,
            neurons: cost,
            reported_cf: answer.rung == Rung::Cf,
            cacheable: !(cfg.cache_live_only && answer.source == AnswerSource::Passages),
            version_check: cfg.version_check,
            cited_shas: cited.as_ref(),
        },
    )
    .await?
    else {
        // A late result (the job was requeued or reclaimed): its answer is
        // refused, but Cloudflare already spent the Neurons.
        let job_cf = q::job_by_id(&state.db, res.job_id)
            .await?
            .is_some_and(|j| j.rung == "cf");
        if answer.rung == Rung::Cf || job_cf {
            q::book_neurons(&state.db, cost).await?;
        }
        return Err(ApiError::Conflict(
            "job is not running under this worker".into(),
        ));
    };
    match crate::app::services::trevor::ask::job_view(state, &job).await {
        Ok(view) => state
            .trevor
            .finish(job.id, StreamEvent::Done(Box::new(view))),
        Err(e) => {
            tracing::warn!(error = %e, job = %job.id, "trevor: done view failed");
            state.trevor.finish(
                job.id,
                StreamEvent::Error("the answer could not be read".into()),
            );
        }
    }
    Ok(())
}

/// `{chunkId: sha}` of an answer's citations when every one carries a
/// non-empty sha, else `None` (the answer is then never carried over).
pub fn cited_shas(answer: &WorkerAnswer) -> Option<Value> {
    if answer.citations.is_empty() {
        return None;
    }
    let mut map = serde_json::Map::new();
    for c in &answer.citations {
        let sha = c
            .sha
            .as_deref()
            .filter(|s| !s.is_empty() && s.len() <= 128)?;
        map.insert(c.chunk_id.clone(), Value::String(sha.to_owned()));
    }
    Some(Value::Object(map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn cited_shas_need_a_sha_on_every_citation() {
        let answer = |cites: Value| -> WorkerAnswer {
            serde_json::from_value(serde_json::json!({
                "answer": "A.", "rung": "cpu", "profile": "mid", "source": "live",
                "citations": cites, "corpusVersion": "v1"
            }))
            .unwrap()
        };
        let c = |chunk: &str, sha: Option<&str>| {
            let mut v = serde_json::json!({"n": 1, "storyId": "s", "chunkId": chunk,
                                           "lineStart": 1, "lineEnd": 2, "text": "t"});
            if let Some(sha) = sha {
                v["sha"] = serde_json::json!(sha);
            }
            v
        };
        let both = answer(serde_json::json!([
            c("a#0", Some("x1")),
            c("b#2", Some("y2"))
        ]));
        assert_eq!(
            cited_shas(&both),
            Some(serde_json::json!({"a#0": "x1", "b#2": "y2"}))
        );
        assert_eq!(
            serde_json::to_value(&both).unwrap()["citations"][0]["sha"],
            "x1"
        );
        assert_eq!(
            cited_shas(&answer(serde_json::json!([
                c("a#0", Some("x1")),
                c("b#2", None)
            ]))),
            None
        );
        assert_eq!(
            cited_shas(&answer(serde_json::json!([c("a#0", Some(""))]))),
            None
        );
        assert_eq!(cited_shas(&answer(serde_json::json!([]))), None);
    }

    #[test]
    fn a_result_drops_the_passages_list_and_keeps_tokens() {
        let r: WorkerResult = serde_json::from_value(serde_json::json!({
            "jobId": Uuid::nil(), "workerId": "w", "ok": true,
            "result": {
                "answer": "A.", "rung": "cf", "profile": "full", "source": "live",
                "citations": [{"n": 1, "storyId": "s", "chunkId": "c", "lineStart": 1,
                               "lineEnd": 2, "text": "t"}],
                "route": "who", "form": "short", "corpusVersion": "v1",
                "tokensIn": 2000, "tokensOut": 250,
                "timings": {"routeMs": 1, "answerMs": 2, "totalMs": 3},
                "passages": [{"text": "every passage"}]
            }
        }))
        .unwrap();
        let answer = r.result.unwrap();
        assert_eq!(cited_shas(&answer), None, "a citation without a sha");
        let stored = serde_json::to_value(answer).unwrap();
        assert!(stored.get("passages").is_none());
        assert!(stored["citations"][0].get("sha").is_none());
        assert_eq!(stored["tokensIn"], 2000);
        assert_eq!(stored["citations"][0]["storyId"], "s");
    }
}
