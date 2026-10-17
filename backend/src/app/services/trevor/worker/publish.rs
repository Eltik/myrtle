//! Publishing a corpus version: the activation checks and the publish.

use serde_json::Value;

use crate::app::error::ApiError;
use crate::app::services::trevor::{Era, parse_server};
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

use super::{PublishOutcome, PublishVersion};

/// What the activating call must carry: the manifest with a well-formed
/// `storyOrder` and `eras`, and the gate report.
fn check_activation(req: &PublishVersion) -> Result<(), ApiError> {
    let manifest = req
        .manifest
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("the activating call must resend `manifest`".into()))?;
    if req.gate_report.is_none() {
        return Err(ApiError::BadRequest(
            "the activating call must resend `gateReport`".into(),
        ));
    }
    let order_ok = manifest
        .get("storyOrder")
        .and_then(Value::as_array)
        .is_some_and(|ids| ids.iter().all(Value::is_string));
    if !order_ok {
        return Err(ApiError::BadRequest(
            "manifest.storyOrder must be an array of story ids".into(),
        ));
    }
    serde_json::from_value::<Vec<Era>>(manifest.get("eras").cloned().unwrap_or(Value::Null))
        .map_err(|e| ApiError::BadRequest(format!("manifest.eras: {e}")))?;
    Ok(())
}

/// `versions`.
pub async fn publish(state: &AppState, req: PublishVersion) -> Result<PublishOutcome, ApiError> {
    let server = parse_server(req.server.as_deref())?;
    if req.version.is_empty() || req.version.len() > 100 {
        return Err(ApiError::BadRequest("version is empty or too long".into()));
    }
    if req.activate {
        check_activation(&req)?;
    }
    let panels: Vec<q::PanelRow> = req
        .panels
        .into_iter()
        .map(|(story_id, panels)| q::PanelRow {
            gate_pos: panels
                .get("position")
                .and_then(Value::as_i64)
                .and_then(|p| i32::try_from(p).ok()),
            story_id,
            panels,
        })
        .collect();
    if req
        .chunk_shas
        .iter()
        .any(|(id, sha)| id.is_empty() || id.len() > 200 || sha.is_empty() || sha.len() > 128)
    {
        return Err(ApiError::BadRequest(
            "chunkShas: every chunk id and sha must be non-empty (ids at most 200 characters, shas 128)"
                .into(),
        ));
    }
    let shas: Vec<(String, String)> = req.chunk_shas.into_iter().collect();
    let carried = q::publish_version(
        &state.db,
        &req.version,
        server,
        req.built_at,
        req.gate_report.as_ref(),
        req.manifest.as_ref(),
        &panels,
        req.activate,
        &shas,
        state.trevor.config().carry_over,
    )
    .await?;
    if req.activate {
        tracing::info!(version = %req.version, server, carried, "trevor: version activated");
    }
    Ok(PublishOutcome {
        version: req.version,
        server: server.to_owned(),
        panels: panels.len(),
        active: req.activate,
        chunk_shas: shas.len(),
        carried,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn publish_req(v: Value) -> PublishVersion {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn activation_requires_the_manifest_order_eras_and_gate_report() {
        let ok = serde_json::json!({
            "version": "v1", "gateReport": {"ok": true},
            "manifest": {"builtFrom": "26-01", "storyOrder": ["a", "b"],
                         "eras": [{"id": "main_0", "name": "Evil Time", "position": 1}]}
        });
        assert!(check_activation(&publish_req(ok.clone())).is_ok());
        let mut no_report = ok.clone();
        no_report.as_object_mut().unwrap().remove("gateReport");
        assert!(check_activation(&publish_req(no_report)).is_err());
        let mut bad_eras = ok.clone();
        bad_eras["manifest"]["eras"] = serde_json::json!([{"id": "x"}]);
        assert!(check_activation(&publish_req(bad_eras)).is_err());
        let mut bad_order = ok;
        bad_order["manifest"]["storyOrder"] = serde_json::json!([1, 2]);
        assert!(check_activation(&publish_req(bad_order)).is_err());
    }
}
