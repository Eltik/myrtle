//! `GET /trevor/jobs/{id}/stream`: a job's answer as the worker writes it,
//! over SSE (contract section 4).
//!
//! Events: `delta` `{seq, text}`, `done` (the job view), `error`
//! `{message}`, and a `: keepalive` comment every 15 s. The stream closes
//! after `done` or `error`; a job that already finished sends one of them at
//! once. Deltas are fanned out in-process (a `broadcast` per job) and never
//! stored, so a reader that connects late misses the text written so far and
//! gets it whole in `done`.
//!
//! The 30 s handler timeout (`middleware::observe`) bounds only the handler
//! future, which returns as soon as the stream exists; the body is not cut.
//! `tests/trevor_handler_test.rs` holds a stream past it.
use std::convert::Infallible;
use std::time::Duration;

use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::Stream;
use serde_json::json;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::runtime::StreamEvent;
use super::{JobStatus, KEEPALIVE_SECS};
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

fn to_event(ev: &StreamEvent) -> Event {
    let ev = match ev {
        StreamEvent::Delta { seq, text } => Event::default()
            .event("delta")
            .json_data(json!({ "seq": seq, "text": text })),
        StreamEvent::Done(view) => Event::default().event("done").json_data(view),
        StreamEvent::Error(message) => Event::default()
            .event("error")
            .json_data(json!({ "message": message })),
    };
    ev.unwrap_or_else(|_| Event::default().event("error").data("{}"))
}

/// The SSE body: `first` (a finished job's terminal event) alone, or the
/// job's channel until its terminal event. A reader that lags past the
/// channel's capacity skips the deltas it missed.
pub fn events(
    first: Option<StreamEvent>,
    rx: broadcast::Receiver<StreamEvent>,
) -> impl Stream<Item = Result<Event, Infallible>> + Send + 'static {
    futures_util::stream::unfold((first, Some(rx)), |(first, rx)| async move {
        if let Some(ev) = first {
            // A finished job: its one event, then the end.
            return Some((Ok(to_event(&ev)), (None, None)));
        }
        let mut rx = rx?;
        loop {
            match rx.recv().await {
                Ok(ev) if ev.is_terminal() => {
                    return Some((Ok(to_event(&ev)), (None, None)));
                }
                Ok(ev) => return Some((Ok(to_event(&ev)), (None, Some(rx)))),
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::debug!(skipped, "trevor: a stream reader lagged");
                }
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    })
}

/// Wrap a stream body in an SSE response with the keep-alive comment.
pub fn sse<S>(body: S, keepalive: Duration) -> Response
where
    S: Stream<Item = Result<Event, Infallible>> + Send + 'static,
{
    Sse::new(body)
        .keep_alive(KeepAlive::new().interval(keepalive).text("keepalive"))
        .into_response()
}

/// The terminal event of a finished job, if it finished.
async fn terminal_event(
    state: &AppState,
    job: &q::JobRow,
) -> Result<Option<StreamEvent>, ApiError> {
    Ok(match JobStatus::parse(&job.status) {
        JobStatus::Done => Some(StreamEvent::Done(Box::new(
            super::ask::job_view(state, job).await?,
        ))),
        JobStatus::Failed => Some(StreamEvent::Error(
            job.error.clone().unwrap_or_else(|| "failed".into()),
        )),
        JobStatus::Queued | JobStatus::Running => None,
    })
}

/// Open a job's stream for its asker (or a twin who joined it). 404 for an
/// unknown job, 403 for anyone else's.
pub async fn open(state: &AppState, user_id: Uuid, job_id: Uuid) -> Result<Response, ApiError> {
    let job = q::job_by_id(&state.db, job_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if !super::ask::may_stream(state, &job, user_id) {
        return Err(ApiError::Forbidden);
    }
    // Subscribe, then look again: a job that finished between the first read
    // and the subscription sends its terminal event from the second read.
    let rx = state.trevor.subscribe(job_id);
    let job = q::job_by_id(&state.db, job_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let first = terminal_event(state, &job).await?;
    Ok(sse(events(first, rx), Duration::from_secs(KEEPALIVE_SECS)))
}
