use uuid::Uuid;

use crate::{
    app::{error::ApiError, state::AppState},
    core::release::{PutOverride, ReleaseOverride, ledger},
    database::queries::release as q,
};

use super::CACHE_PREFIX;

const KINDS: &[&str] = &[
    ledger::KIND_ACTIVITY,
    ledger::KIND_POOL,
    ledger::KIND_SKIN,
    ledger::KIND_RETRO,
];

fn to_wire(r: q::OverrideRow) -> ReleaseOverride {
    ReleaseOverride {
        kind: r.kind,
        cn_id: r.cn_id,
        en_id: r.en_id,
        en_name: r.en_name,
        en_start: r.en_start,
        en_end: r.en_end,
        featured_chars: r.featured_chars,
        source: r.source,
        note: r.note,
        updated_at: r.updated_at.timestamp(),
    }
}

pub async fn list_overrides(state: &AppState) -> Result<Vec<ReleaseOverride>, ApiError> {
    Ok(q::list_overrides(&state.db)
        .await?
        .into_iter()
        .map(to_wire)
        .collect())
}

pub async fn put_override(
    state: &AppState,
    body: PutOverride,
    updated_by: Option<Uuid>,
) -> Result<ReleaseOverride, ApiError> {
    if !KINDS.contains(&body.kind.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "kind must be one of {}",
            KINDS.join(", ")
        )));
    }
    if body.cn_id.trim().is_empty() {
        return Err(ApiError::BadRequest("cn_id is required".into()));
    }
    let row = q::put_override(
        &state.db,
        &q::OverrideInput {
            kind: body.kind.clone(),
            cn_id: body.cn_id.trim().to_string(),
            en_id: body.en_id.clone(),
            en_name: body.en_name.clone().filter(|n| !n.trim().is_empty()),
            en_start: body.en_start,
            en_end: body.en_end,
            featured_chars: body.featured_chars.clone().filter(|f| !f.is_empty()),
            source: body.source.clone().unwrap_or_else(|| "manual".into()),
            note: body.note.clone().unwrap_or_default(),
            updated_by,
        },
    )
    .await?;
    state.cache.invalidate_by_prefix(CACHE_PREFIX).await;
    Ok(to_wire(row))
}

pub async fn delete_override(state: &AppState, kind: &str, cn_id: &str) -> Result<(), ApiError> {
    if !q::delete_override(&state.db, kind, cn_id).await? {
        return Err(ApiError::NotFound);
    }
    state.cache.invalidate_by_prefix(CACHE_PREFIX).await;
    Ok(())
}
