//! Publishing a corpus version: the activation checks and the publish.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::atomic::Ordering;

use serde_json::Value;

use crate::app::error::ApiError;
use crate::app::services::trevor::{
    AnswerSource, Era, NO_LIMIT_ERA, Profile, Rung, WorkerAnswer, parse_server, qkey,
    trim_citation_text, validate_question,
};
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

use super::{BankAnswer, BankOutcome, BankSource, PublishOutcome, PublishVersion};

/// The longest cited passage an imported answer keeps, as a worker's result.
const CITATION_CHARS: usize = 600;

/// The era buckets a bank answer is cached under: the first era whose
/// position is at or after its gatePos, and the spoiler-mode bucket
/// ([`NO_LIMIT_ERA`]). Every later era reads the first one through the bank
/// fallback ([`q::hit_bank`]), so it is stored once. An undatable gate
/// (`None`) or one past the last era gets the spoiler-mode bucket only: no
/// spoiler-safe reader is known to be past it, as `scripts/panels.py`
/// hides such an entry at every horizon.
pub fn bank_buckets(gate_pos: Option<i32>, eras: &[Era]) -> Vec<i32> {
    let first = gate_pos.and_then(|g| {
        eras.iter()
            .map(|e| e.position)
            .filter(|&p| p >= g && p != NO_LIMIT_ERA)
            .min()
    });
    first.into_iter().chain([NO_LIMIT_ERA]).collect()
}

/// Whether a bank answer can be imported at all: a non-empty answer, at
/// least one citation, every citation with a chunk id and a sha (within the
/// `chunkShas` limits), and a gatePos of at least 1 when there is one.
fn bank_answer_ok(a: &BankAnswer) -> bool {
    !a.answer.trim().is_empty()
        && !a.citations.is_empty()
        && a.citations.iter().all(|c| {
            !c.chunk_id.is_empty()
                && c.chunk_id.len() <= 200
                && c.sha
                    .as_deref()
                    .is_some_and(|s| !s.is_empty() && s.len() <= 128)
        })
        && a.gate_pos.is_none_or(|g| g >= 1)
}

/// The cache rows of a publish's bank answers, and how many answers were
/// refused ([`bank_answer_ok`], or no wording that validates as a question).
/// One row per wording (the question and its aliases, deduplicated by cache
/// key) and era bucket ([`bank_buckets`]). Two answers under one key: a
/// re-answer beats a pre-generated one, then the later `answeredAt`, then the
/// first sent.
pub fn bank_rows(
    answers: &[BankAnswer],
    eras: &[Era],
    normq: bool,
    version: &str,
) -> (Vec<q::BankRow>, usize) {
    let mut invalid = 0;
    let mut by_key: HashMap<(String, i32), (usize, q::BankRow, BankSource)> = HashMap::new();
    for (i, a) in answers.iter().enumerate() {
        if !bank_answer_ok(a) {
            invalid += 1;
            continue;
        }
        let mut keys: Vec<(String, &str)> = Vec::new();
        for wording in std::iter::once(&a.question).chain(a.aliases.iter()) {
            let Ok(normalized) = validate_question(wording, normq) else {
                continue;
            };
            let key = qkey(&normalized);
            if !keys.iter().any(|(k, _)| *k == key) {
                keys.push((key, wording.trim()));
            }
        }
        if keys.is_empty() {
            invalid += 1;
            continue;
        }
        let citations = a
            .citations
            .iter()
            .map(|c| {
                let mut c = c.clone();
                c.text = trim_citation_text(&c.text, CITATION_CHARS);
                c
            })
            .collect::<Vec<_>>();
        let cited_shas: serde_json::Map<String, Value> = citations
            .iter()
            .filter_map(|c| Some((c.chunk_id.clone(), Value::String(c.sha.clone()?))))
            .collect();
        let answer = serde_json::to_value(WorkerAnswer {
            answer: a.answer.clone(),
            rung: Rung::Cpu,
            profile: a.profile.unwrap_or(Profile::Full),
            source: AnswerSource::Live,
            citations,
            route: Value::Object(serde_json::Map::new()),
            form: Value::Object(serde_json::Map::new()),
            corpus_version: version.to_owned(),
            tokens_in: 0,
            tokens_out: 0,
            timings: Value::Object(serde_json::Map::new()),
        })
        .unwrap_or(Value::Null);
        let cited_shas = Value::Object(cited_shas);
        for (key, wording) in &keys {
            for era_pos in bank_buckets(a.gate_pos, eras) {
                let row = q::BankRow {
                    qkey: key.clone(),
                    era_pos,
                    gate_pos: a.gate_pos,
                    question: (*wording).to_owned(),
                    answer: answer.clone(),
                    cited_shas: cited_shas.clone(),
                    origin: a.source.as_str(),
                    answered_at: a.answered_at,
                };
                match by_key.entry((key.clone(), era_pos)) {
                    Entry::Vacant(v) => {
                        v.insert((i, row, a.source));
                    }
                    Entry::Occupied(mut o) => {
                        let (_, held, held_source) = o.get();
                        let wins = match (a.source, *held_source) {
                            (BankSource::Reanswer, BankSource::Bank) => true,
                            (BankSource::Bank, BankSource::Reanswer) => false,
                            _ => row.answered_at > held.answered_at,
                        };
                        if wins {
                            o.insert((i, row, a.source));
                        }
                    }
                }
            }
        }
    }
    let mut rows: Vec<(usize, q::BankRow)> =
        by_key.into_values().map(|(i, row, _)| (i, row)).collect();
    rows.sort_by(|a, b| (a.0, &a.1.qkey, a.1.era_pos).cmp(&(b.0, &b.1.qkey, b.1.era_pos)));
    (rows.into_iter().map(|(_, r)| r).collect(), invalid)
}

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
    let cfg = state.trevor.config();
    let mut bank = BankOutcome {
        sent: req.bank_answers.len(),
        ..BankOutcome::default()
    };
    // Read on the activating call only, whose manifest carries the eras.
    let bank_rows = if req.activate && cfg.bank_import && !req.bank_answers.is_empty() {
        let eras: Vec<Era> = req
            .manifest
            .as_ref()
            .and_then(|m| m.get("eras"))
            .and_then(|e| serde_json::from_value(e.clone()).ok())
            .unwrap_or_default();
        let (rows, invalid) = bank_rows(&req.bank_answers, &eras, cfg.normq, &req.version);
        bank.invalid = invalid;
        bank.rows = rows.len();
        rows
    } else {
        Vec::new()
    };
    let published = q::publish_version_with_bank(
        &state.db,
        &req.version,
        server,
        req.built_at,
        req.gate_report.as_ref(),
        req.manifest.as_ref(),
        &panels,
        req.activate,
        &shas,
        cfg.carry_over,
        &bank_rows,
    )
    .await?;
    let carried = published.carried;
    match published.bank {
        Ok(n) => {
            bank.imported = n.imported;
            bank.stale = n.stale;
            bank.kept = n.kept;
        }
        Err(e) => {
            bank.failed = true;
            let failures = state
                .trevor
                .bank_import_failures
                .fetch_add(1, Ordering::Relaxed)
                + 1;
            tracing::warn!(version = %req.version, server, error = %e, failures,
                "trevor: the bank import failed and was rolled back; the publish went on");
        }
    }
    if req.activate {
        tracing::info!(version = %req.version, server, carried, bank_sent = bank.sent,
            bank_imported = bank.imported, bank_stale = bank.stale, bank_kept = bank.kept,
            "trevor: version activated");
    }
    Ok(PublishOutcome {
        version: req.version,
        server: server.to_owned(),
        panels: panels.len(),
        active: req.activate,
        chunk_shas: shas.len(),
        carried,
        bank,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn publish_req(v: Value) -> PublishVersion {
        serde_json::from_value(v).unwrap()
    }

    fn era(position: i32) -> Era {
        Era {
            id: format!("e{position}"),
            name: format!("Era {position}"),
            position,
        }
    }

    fn bank(question: &str, gate_pos: Option<i32>, source: &str, at: Option<&str>) -> BankAnswer {
        serde_json::from_value(serde_json::json!({
            "question": question, "answer": format!("{source} answer"), "gatePos": gate_pos,
            "source": source, "answeredAt": at,
            "citations": [{"n": 1, "storyId": "s1", "chunkId": "s1#0", "lineStart": 1,
                           "lineEnd": 2, "text": "t", "sha": "aaa"}]
        }))
        .unwrap()
    }

    #[test]
    fn bank_buckets_are_the_first_era_at_or_after_the_gate_and_spoiler_mode() {
        let eras = [era(14), era(1), era(26)];
        assert_eq!(bank_buckets(Some(1), &eras), vec![1, NO_LIMIT_ERA]);
        assert_eq!(bank_buckets(Some(2), &eras), vec![14, NO_LIMIT_ERA]);
        assert_eq!(bank_buckets(Some(26), &eras), vec![26, NO_LIMIT_ERA]);
        // Past the last era, or undatable: spoiler mode only.
        assert_eq!(bank_buckets(Some(27), &eras), vec![NO_LIMIT_ERA]);
        assert_eq!(bank_buckets(None, &eras), vec![NO_LIMIT_ERA]);
        assert_eq!(bank_buckets(Some(1), &[]), vec![NO_LIMIT_ERA]);
    }

    #[test]
    fn bank_rows_refuse_bad_answers_and_keep_one_answer_per_key() {
        let eras = [era(1), era(4)];
        let mut no_sha = bank("no sha", Some(1), "bank", None);
        no_sha.citations[0].sha = None;
        let mut empty = bank("   ", Some(1), "bank", None);
        empty.aliases = vec!["?".into()];
        let answers = [
            bank("Who is Amiya?", Some(2), "bank", None),
            bank("who is amiya", Some(3), "reanswer", None),
            bank("Older?", None, "reanswer", Some("2026-10-18T00:00:00Z")),
            bank("older", None, "reanswer", Some("2026-10-19T00:00:00Z")),
            no_sha,
            empty,
            bank("gate zero", Some(0), "bank", None),
        ];
        let (rows, invalid) = bank_rows(&answers, &eras, true, "v1");
        assert_eq!(invalid, 3);
        let got: Vec<(&str, i32, &str)> = rows
            .iter()
            .map(|r| (r.question.as_str(), r.era_pos, r.origin))
            .collect();
        // One key per normalized question and era: the re-answer beats the
        // pre-generated answer, then the later answeredAt.
        assert_eq!(
            got,
            vec![
                ("who is amiya", 4, "reanswer"),
                ("who is amiya", NO_LIMIT_ERA, "reanswer"),
                ("older", NO_LIMIT_ERA, "reanswer"),
            ]
        );
        let a: WorkerAnswer = serde_json::from_value(rows[0].answer.clone()).unwrap();
        assert_eq!(
            (a.rung, a.profile, a.source),
            (Rung::Cpu, Profile::Full, AnswerSource::Live)
        );
        assert_eq!(rows[0].cited_shas, serde_json::json!({"s1#0": "aaa"}));
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
