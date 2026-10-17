//! The warm-up after a publish (contract v1.1 section 5): the popular
//! questions a new version lacks, and queueing them as internal jobs.

use crate::app::error::ApiError;
use crate::app::services::trevor::{
    Mode, ONLINE_SECS, Rung, RungFlags, parse_server, qkey, validate_question,
};
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

use super::{WarmList, WarmOutcome, WarmQuestion, WarmRequest};

/// The warm-up list (contract v1.1 section 3c): at most `limit` (default
/// `TREVOR_WARM_TOP`'s 100, 1 to 500) (normQ, eraPos) pairs asked in the
/// last 30 days that `version` has no cached answer for. 404 for an unknown
/// version.
pub async fn popular(
    state: &AppState,
    server: Option<&str>,
    version: &str,
    limit: Option<i64>,
) -> Result<WarmList, ApiError> {
    let server = parse_server(server)?;
    if version.is_empty() || version.len() > 100 {
        return Err(ApiError::BadRequest("version is empty or too long".into()));
    }
    let limit = limit.unwrap_or(WARM_TOP_DEFAULT);
    if !(1..=WARM_MAX).contains(&limit) {
        return Err(ApiError::BadRequest(format!(
            "limit must be 1 to {WARM_MAX}"
        )));
    }
    if !q::version_known(&state.db, version, server).await? {
        return Err(ApiError::NotFoundMessage(format!(
            "no Trevor version {version}"
        )));
    }
    let items = q::warm_candidates(&state.db, version, server, WARM_DAYS, limit)
        .await?
        .into_iter()
        .map(|(question, norm_q, era_pos, hits)| WarmQuestion {
            question,
            norm_q,
            era_pos,
            hits,
        })
        .collect();
    Ok(WarmList {
        version: version.to_owned(),
        items,
    })
}

/// `TREVOR_WARM_TOP`'s default: the update script's warm-up size.
pub const WARM_TOP_DEFAULT: i64 = 100;

/// The warm-up counts asks in this many days.
pub const WARM_DAYS: i32 = 30;

/// One warm list or call holds at most this many items.
pub const WARM_MAX: i64 = 500;

/// A warm call's body is at most this many bytes (413 over it).
pub const WARM_BODY_MAX: usize = 64 * 1024;

/// `worker/warm`: queue each item as an internal deferred job with no user
/// and its era as the horizon. Nothing is queued for an item already cached
/// in `version` under that era (`cached`); an item with an open job (normal
/// or deferred) joins it (`joined`). The rung stored is the best model rung
/// a heartbeat offers; the claim settles it like any job's. 409 when
/// `version` is not the active version, 503 unless `TREVOR_MODE` is `on`.
pub async fn warm(state: &AppState, req: WarmRequest) -> Result<WarmOutcome, ApiError> {
    let server = parse_server(req.server.as_deref())?;
    let cfg = state.trevor.config();
    if cfg.mode != Mode::On {
        return Err(ApiError::ServiceUnavailableMessage(
            "Trevor is not answering with a model (TREVOR_MODE is not on)".into(),
        ));
    }
    if req.items.len() > WARM_MAX as usize {
        return Err(ApiError::BadRequest(format!(
            "at most {WARM_MAX} items per call"
        )));
    }
    if q::active_version(&state.db, server).await?.as_deref() != Some(req.version.as_str()) {
        return Err(ApiError::Conflict(format!(
            "{} is not the active version",
            req.version
        )));
    }
    let adm = q::admission_inputs(&state.db, ONLINE_SECS).await?;
    let rungs: RungFlags = adm
        .rungs
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let rung = if rungs.cf {
        Rung::Cf
    } else if rungs.cpu {
        Rung::Cpu
    } else {
        Rung::Passages
    };
    let mut out = WarmOutcome::default();
    for item in &req.items {
        let normalized = validate_question(&item.question, cfg.normq)?;
        let key = qkey(&normalized);
        let era = item.era_pos.max(0);
        if q::has_cached(&state.db, &req.version, server, &key, era).await? {
            out.cached += 1;
            continue;
        }
        if q::open_job_for_key(&state.db, server, &key, era, false)
            .await?
            .is_some()
        {
            out.joined += 1;
            continue;
        }
        let inserted = q::insert_job(
            &state.db,
            &q::NewJob {
                user_id: None,
                question: item.question.trim(),
                qkey: &key,
                horizon_pos: Some(era),
                server,
                corpus_version: Some(&req.version),
                rung: rung.as_str(),
                profile: rung.profile_with(cfg.cpu_profile).as_str(),
                deferred: true,
                kind: "warm",
            },
        )
        .await?;
        if inserted.is_some() {
            out.queued += 1;
        } else {
            out.joined += 1;
        }
    }
    if out.queued > 0 {
        state.trevor.job_enqueued();
    }
    Ok(out)
}
