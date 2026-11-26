#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use myrtle_parity::compare::{DiffKind, sha256_hex};
use myrtle_parity::corpus::{CorpusEntry, Source};
use myrtle_parity::report::Summary;
use myrtle_parity::rules::{RuleOverrides, Rules};
use myrtle_parity::run::{
    self, CompareRun, Manifest, RECORDING_VERSION, RecordedResponse, Recording, Side,
};

enum Answer {
    Body(u16, &'static str, &'static [u8], Option<&'static str>),
    Error(&'static str),
    Skipped(&'static str),
}

fn rules() -> Rules {
    Rules::build(
        None,
        &RuleOverrides {
            max_diffs: 10,
            ..RuleOverrides::default()
        },
    )
    .unwrap()
}

fn entry(path: &str) -> CorpusEntry {
    CorpusEntry::new(path, path, None, Source::Openapi)
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("parity-run-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bodies")).unwrap();
    dir
}

fn write_recording(dir: &Path, answers: &[(&CorpusEntry, Answer)]) -> Recording {
    let mut lines = Vec::new();
    for (e, answer) in answers {
        let mut rec = RecordedResponse {
            id: e.id.clone(),
            route: e.route.clone(),
            path: e.path.clone(),
            role: None,
            status: None,
            content_type: None,
            sha256: None,
            len: None,
            body_file: None,
            elapsed_ms: None,
            error: None,
            skipped: None,
        };
        match answer {
            Answer::Body(status, ct, bytes, stored) => {
                if let Some(ext) = stored {
                    let name = format!("bodies/{}.{ext}", e.id);
                    std::fs::write(dir.join(&name), bytes).unwrap();
                    rec.body_file = Some(name);
                }
                rec.status = Some(*status);
                rec.content_type = Some((*ct).to_owned());
                rec.sha256 = Some(sha256_hex(bytes));
                rec.len = Some(bytes.len() as u64);
            }
            Answer::Error(m) => rec.error = Some((*m).to_owned()),
            Answer::Skipped(m) => rec.skipped = Some((*m).to_owned()),
        }
        lines.push(serde_json::to_string(&rec).unwrap());
    }
    std::fs::write(dir.join("responses.jsonl"), lines.join("\n")).unwrap();
    let manifest = Manifest {
        tool: "myrtle-parity".to_owned(),
        version: RECORDING_VERSION,
        base: "http://x".to_owned(),
        corpus_sha256: String::new(),
        entries: answers.len(),
        recorded_at_unix: 0,
    };
    std::fs::write(
        dir.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    Recording::open(dir).unwrap()
}

async fn offline(a: &Recording, b: &Recording, corpus: &[CorpusEntry]) -> Summary {
    let run = CompareRun {
        concurrency: 1,
        max_problems: 50,
        progress: false,
    };
    run::compare(
        &Side::Recorded(a),
        &Side::Recorded(b),
        corpus,
        &rules(),
        &run,
    )
    .await
}

fn message<'a>(s: &'a Summary, path: &str) -> &'a str {
    s.problems
        .iter()
        .find(|p| p.path == path)
        .and_then(|p| p.message.as_deref())
        .unwrap()
}

#[tokio::test]
async fn combine_rules() {
    let json = "application/json";
    let (eq, diff, limited, err, skip) = (
        entry("/eq"),
        entry("/diff"),
        entry("/429"),
        entry("/err"),
        entry("/skip"),
    );
    let dir_a = temp_dir("combine-a");
    let dir_b = temp_dir("combine-b");
    let a = write_recording(
        &dir_a,
        &[
            (&eq, Answer::Body(200, json, b"[1]", Some("json"))),
            (&diff, Answer::Body(200, json, b"[1]", Some("json"))),
            (&limited, Answer::Body(429, json, b"{}", Some("json"))),
            (&err, Answer::Error("x")),
            (&skip, Answer::Skipped("t")),
        ],
    );
    let b = write_recording(
        &dir_b,
        &[
            (&eq, Answer::Body(200, json, b"[1.0]", Some("json"))),
            (&diff, Answer::Body(200, json, b"[2]", Some("json"))),
            (&limited, Answer::Body(200, json, b"[1]", Some("json"))),
            (&err, Answer::Body(200, json, b"[1]", Some("json"))),
            (&skip, Answer::Error("x")),
        ],
    );
    let corpus = [eq, diff, limited, err, skip];
    let s = offline(&a, &b, &corpus).await;
    assert_eq!(
        (s.equal, s.different, s.rate_limited, s.errors, s.skipped),
        (1, 1, 1, 1, 1),
        "{}",
        s.render_text()
    );
    assert_eq!(message(&s, "/429"), "HTTP 429 (A=429, B=200)");
    assert_eq!(message(&s, "/err"), "A: recorded error: x");
    assert_eq!(s.skip_reasons.keys().collect::<Vec<_>>(), ["t"]);
    std::fs::remove_dir_all(&dir_a).unwrap();
    std::fs::remove_dir_all(&dir_b).unwrap();
}

#[tokio::test]
async fn recording_reads_back_bodies_hashes_and_errors() {
    let (e_json, e_png, e_err, e_missing) = (
        entry("/r/json"),
        entry("/r/png"),
        entry("/r/err"),
        entry("/r/missing"),
    );
    let dir_a = temp_dir("readback-a");
    let dir_b = temp_dir("readback-b");
    let a = write_recording(
        &dir_a,
        &[
            (
                &e_json,
                Answer::Body(200, "application/json", br#"{"a":1}"#, Some("json")),
            ),
            (&e_png, Answer::Body(200, "image/png", b"PNG", None)),
            (&e_err, Answer::Error("timeout")),
        ],
    );
    let b = write_recording(
        &dir_b,
        &[
            (
                &e_json,
                Answer::Body(200, "application/json", br#"{"a":2}"#, Some("json")),
            ),
            (&e_png, Answer::Body(200, "image/png", b"PNG", Some("bin"))),
            (
                &e_err,
                Answer::Body(200, "application/json", b"{}", Some("json")),
            ),
            (
                &e_missing,
                Answer::Body(200, "application/json", b"{}", Some("json")),
            ),
        ],
    );
    assert_eq!(a.len(), 3);
    assert!(!a.is_empty());
    let corpus = [e_json, e_png, e_err, e_missing];
    let s = offline(&a, &b, &corpus).await;
    assert_eq!(
        (s.equal, s.different, s.errors),
        (1, 1, 2),
        "{}",
        s.render_text()
    );
    let json = s.problems.iter().find(|p| p.path == "/r/json").unwrap();
    assert_eq!(json.diffs[0].pointer, "/a");
    assert_eq!(json.diffs[0].kind, DiffKind::Number);
    assert_eq!(
        (json.diffs[0].a.as_deref(), json.diffs[0].b.as_deref()),
        (Some("1"), Some("2"))
    );
    assert!(message(&s, "/r/err").contains("timeout"));
    assert!(message(&s, "/r/missing").contains("not in recording"));
    std::fs::remove_dir_all(&dir_a).unwrap();
    std::fs::remove_dir_all(&dir_b).unwrap();
}
