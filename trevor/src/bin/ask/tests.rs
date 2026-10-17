//! Unit tests of the question tests, the term helpers and the chronology lines.

use trevor::tools::Tools;

use crate::batch::{batch_items, qid_question};
use crate::chrono::{Chrono, chronology_for, time_evidence_block};
use crate::detect::{asks_identity, asks_origin, asks_relation, asks_scene, asks_term, asks_when, term_of};
use crate::retrieval::{Names, one_operator, one_operator_wide, story_label, typed_owner};
use crate::terms::{compact, count_word, first_sentences};

#[test]
fn term_lookup_helpers_find_the_term_and_its_spelling_parts() {
    assert_eq!(term_of("What is the khagan quest?").as_deref(), Some("khagan quest"));
    assert_eq!(term_of("who are the sarkaz?").as_deref(), Some("sarkaz"));
    assert_eq!(term_of("What is Amiya's power?"), None, "a possessive is not a term question");
    assert_eq!(compact("Khagan quest"), compact("Khaganquest"));
    assert_eq!(compact("Kal'tsit"), "kaltsit");
    assert_eq!(count_word("Iris looked for Mabel. Iris'", "Iris"), 2);
    assert_eq!(count_word("Irisa and Iris", "Iris"), 1);
    assert_eq!(first_sentences("What it is\nA land of ice. It is cold. It is far.", 2), "A land of ice. It is cold.");
}

#[test]
fn rule_triggers_fire_on_their_questions_only() {
    assert!(asks_origin("Why did swire join the LGD?") && asks_origin("When did the abyssal hunter project start?"));
    assert!(!asks_origin("Why is Kal'tsit so old?") && !asks_origin("What did Swire join?"));
    assert!(asks_relation("What is the relationship between Kristen and Friston?") && asks_relation("Which elite op is Lava a fan of / idolises?"));
    assert!(!asks_relation("Who is Ed?"));
    assert!(asks_when("When was Rhodes Island founded?") && asks_when("in what year did X die") && !asks_when("Who founded Rhodes Island?"));
    assert_eq!(typed_owner("Operator art: Pallas, E2 (Elite 2) art").as_deref(), Some("Pallas"));
    assert_eq!(typed_owner("Voice lines: Aroma").as_deref(), Some("Aroma"));
    assert_eq!(typed_owner("Module story: New Friends (Mutsumi Wakaba's module, PUM-Y)").as_deref(), Some("Mutsumi Wakaba"));
    assert!(asks_term("What is the khagan quest?") && asks_term("Who are the sarkaz?") && asks_term("What is sami guarding?"));
    assert!(!asks_term("What is Kal'tsit's real name?") && !asks_term("What is the name of the operator who visited Iris's castle of dreams?"));
    assert!(asks_identity("Which Rhodes Island Operator had visited Iris' castle of dreams?") && !asks_identity("Why did swire join the LGD?"));
    let mon3tr = ["mon3tr".to_owned()];
    assert!(asks_scene("During which event Mon3tr suggested assassinating a child", &mon3tr));
    assert!(asks_scene("What did Mon3tr say to the Doctor?", &mon3tr) && asks_scene("In which story Mon3tr ambushed a Sarkaz?", &mon3tr));
    assert!(!asks_scene("Why does Mon3tr's hat say KEE?", &mon3tr) && !asks_scene("What does the captain say about Mon3tr?", &mon3tr));
    assert!(!asks_scene("Is Mon3tr related to the Aggeloi?", &mon3tr));
}

#[test]
fn chronology_lines_name_the_subject_and_one_word_names_widen_only_when_unique() {
    let c = Chrono { history: vec![(1086.0, "1086: After excavating the Rhodes Island landship, Babel sought an engineer.".into()),
                                   (1094.0, "1094 (scene dated): Babelite detonates explosives on Rhodes Island.".into()),
                                   (1097.0, "1097: Ch'en leaves Lungmen.".into())], ..Chrono::default() };
    let out = chronology_for("When was Rhodes Island founded?", &c);
    assert!(out.contains("RHODES ISLAND") && out.contains("1086") && out.contains("1094") && !out.contains("Lungmen"));
    assert!(chronology_for("when was it founded?", &c).is_empty());
    let te = time_evidence_block(&out);
    assert!(te.starts_with("\nTIME EVIDENCE") && te.contains("T1. 1086") && te.contains("T2. 1094") && !te.contains("T3."));
    let t = Tools::load(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
    if t.attributes.is_some() {
        assert_eq!(one_operator_wide("who helped Mutsumi care for the plants?", &t).as_deref(), Some("Mutsumi Wakaba"));
        assert_eq!(one_operator("who helped Mutsumi care for the plants?", &t), None);
    }
}

#[test]
fn story_labels_and_batch_items_keep_their_shapes() {
    let mut names = std::collections::HashMap::new();
    names.insert("main_00-01".to_owned(), Names { story: "Prologue".into(), group: "Episode 00".into() });
    assert_eq!(story_label(&names, "main_00-01"), "story: Episode 00, Prologue");
    assert_eq!(story_label(&names, "act1_x"), "story: act1_x");
    let p = std::env::temp_dir().join(format!("trevor-ask-batch-test-{}.jsonl", std::process::id()));
    std::fs::write(&p, "{\"qid\": \"a\", \"question\": \"Q?\"}\n\n{\"question\": \"R?\"}\nnot json\n").unwrap();
    let items = batch_items(&p).unwrap();
    assert_eq!(items.len(), 3, "blank lines are skipped, malformed ones kept as errors");
    assert_eq!(qid_question(items[0].as_ref().unwrap()), ("a", "Q?"));
    assert_eq!(qid_question(items[1].as_ref().unwrap()), ("", "R?"));
    assert!(items[2].is_err());
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn serve_request_and_result_schema() {
    use crate::serve::{AskRequest, AskResult, Citation, Timings, corpus_version};
    // The request: question required, the rest optional, unknown fields refused.
    let r: AskRequest = serde_json::from_str(r#"{"question":"Who is Amiya?"}"#).unwrap();
    assert_eq!((r.question.as_str(), r.horizon, r.server, r.flags.len()), ("Who is Amiya?", None, None, 0));
    let r: AskRequest = serde_json::from_str(r#"{"question":"Q","horizon":"main_3","server":"en","flags":["--no-answer-check"]}"#).unwrap();
    assert_eq!((r.horizon.as_deref(), r.server.as_deref(), r.flags[0].as_str()), (Some("main_3"), Some("en"), "--no-answer-check"));
    assert!(serde_json::from_str::<AskRequest>(r#"{"question":"Q","k":3}"#).is_err());
    assert!(serde_json::from_str::<AskRequest>(r#"{"horizon":"main_3"}"#).is_err());
    // The result: camelCase keys, a citation's story and line span, the generated passage's without them.
    let out = AskResult { answer: "A [1].".into(), passages: vec!["main_0_x#0001".into(), "topic:Ursus".into()],
        citations: vec![Citation { story_id: Some("main_0_x".into()), chunk_id: "main_0_x#0001".into(), line_start: Some(3), line_end: Some(9) },
                        Citation { story_id: None, chunk_id: "topic:Ursus".into(), line_start: None, line_end: None }],
        route: Some(trevor::tools::Route::retrieve()), table: false, flags: Vec::new(), form: None, horizon: Some("main_3".into()),
        server: "en".into(), corpus_version: corpus_version(&[("artifacts/p3b".into(), "abc".into())]), invalid_citations: 0,
        prompt_tokens: 10, timings: Timings { route_ms: 1.0, answer_ms: 2.0, total_ms: 3.0 } };
    let v = serde_json::to_value(&out).unwrap();
    for key in ["answer", "citations", "passages", "route", "table", "flags", "horizon", "server", "corpusVersion", "invalidCitations",
                "promptTokens", "timings"] {
        assert!(v.get(key).is_some(), "missing {key}");
    }
    assert_eq!(v["citations"][0], serde_json::json!({"storyId": "main_0_x", "chunkId": "main_0_x#0001", "lineStart": 3, "lineEnd": 9}));
    assert_eq!(v["citations"][1], serde_json::json!({"chunkId": "topic:Ursus"}));
    assert_eq!(v["timings"], serde_json::json!({"routeMs": 1.0, "answerMs": 2.0, "totalMs": 3.0}));
    assert!(v.get("form").is_none());
    // The corpus version: server-prefixed, 16 hex, stable, and moved by any corpus sha.
    let cv = v["corpusVersion"].as_str().unwrap();
    assert!(cv.starts_with("en-") && cv.len() == 19, "{cv}");
    assert_eq!(cv, corpus_version(&[("artifacts/p3b".into(), "abc".into())]));
    assert_ne!(cv, corpus_version(&[("artifacts/p3b".into(), "abd".into())]));
}

#[test]
fn serve_reads_a_request_with_its_body() {
    use crate::serve::read_request;
    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
    let raw = b"POST /v1/ask HTTP/1.1\r\nHost: x\r\ncontent-length: 16\r\n\r\n{\"question\":\"Q\"}";
    let (m, p, b) = rt.block_on(read_request(&mut &raw[..])).unwrap();
    assert_eq!((m.as_str(), p.as_str(), b.as_slice()), ("POST", "/v1/ask", &b"{\"question\":\"Q\"}"[..]));
    let (m, p, b) = rt.block_on(read_request(&mut &b"GET /health HTTP/1.1\r\n\r\n"[..])).unwrap();
    assert_eq!((m.as_str(), p.as_str(), b.len()), ("GET", "/health", 0));
    assert!(rt.block_on(read_request(&mut &b"POST /v1/ask HTTP/1.1\r\nContent-Length: 99\r\n\r\n{}"[..])).is_err(), "short body");
    assert!(rt.block_on(read_request(&mut &b"POST /v1/ask HTTP/1.1\r\nContent-Length: 999999\r\n\r\n"[..])).is_err(), "too large");
}

/// A scripted HTTP server: answers each request with the next status, records (path, body), and stops after the script.
async fn scripted(statuses: Vec<u16>) -> (String, tokio::task::JoinHandle<Vec<(String, serde_json::Value)>>) {
    use tokio::io::AsyncWriteExt;
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", l.local_addr().unwrap());
    let h = tokio::spawn(async move {
        let mut seen = Vec::new();
        for status in statuses {
            let (mut s, _) = l.accept().await.unwrap();
            let (_, path, body) = crate::serve::read_request(&mut s).await.unwrap();
            seen.push((path, serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null)));
            let head = format!("HTTP/1.1 {status} X\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            s.write_all(head.as_bytes()).await.unwrap();
            s.shutdown().await.unwrap();
        }
        seen
    });
    (base, h)
}

#[tokio::test]
async fn worker_result_retries_then_spools_and_resends_on_start() {
    use crate::serve::{Api, Posted, deliver, resend_spool};
    let spool = std::env::temp_dir().join(format!("trevor-spool-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&spool);
    let api = |base: String| Api { http: reqwest::Client::new(), base, key: "k".into(), worker_id: "w".into(),
                                   backoff: vec![std::time::Duration::from_millis(1); 3] };
    let body = serde_json::json!({"jobId": "j/1", "workerId": "w", "ok": true, "result": {"answer": "A"}});
    // 1 attempt + 3 retries, all 500: spooled under a safe name.
    let (base, h) = scripted(vec![500, 500, 503, 500]).await;
    assert!(matches!(deliver(&api(base), &spool, &body).await, Posted::Failed(_)));
    let seen = h.await.unwrap();
    assert_eq!(seen.len(), 4);
    assert!(seen.iter().all(|(p, b)| p == "/api/trevor/worker/result" && *b == body));
    let file = spool.join("j_1.json");
    assert!(file.exists());
    // A refused result (413) is not retried and not spooled; a retry that succeeds stops there.
    let (base, h) = scripted(vec![413]).await;
    assert!(matches!(deliver(&api(base), &spool, &serde_json::json!({"jobId": "x"})).await, Posted::Refused(413, _)));
    assert_eq!(h.await.unwrap().len(), 1);
    let (base, h) = scripted(vec![502, 200]).await;
    assert_eq!(deliver(&api(base), &spool, &serde_json::json!({"jobId": "y"})).await, Posted::Ok);
    assert_eq!(h.await.unwrap().len(), 2);
    assert!(!spool.join("x.json").exists() && !spool.join("y.json").exists());
    // On start: a failing backend keeps the file, an accepting one gets it and the file goes.
    let (base, h) = scripted(vec![500]).await;
    assert_eq!(resend_spool(&api(base), &spool).await, (0, 1));
    h.await.unwrap();
    let (base, h) = scripted(vec![200]).await;
    assert_eq!(resend_spool(&api(base), &spool).await, (1, 0));
    assert_eq!(h.await.unwrap()[0].1, body);
    assert!(!file.exists());
    let _ = std::fs::remove_dir_all(&spool);
}

#[test]
fn worker_deltas_send_only_text_that_continues_the_stream() {
    use crate::serve::DeltaState;
    use trevor::goldgen::llm::StreamEvent::{Piece, Start};
    let mut d = DeltaState::default();
    let mut out = String::new();
    for e in [Start, Piece("Talulah".into()), Piece(" leads".into()),
              // A second call (the answer extension) repeats the text, then goes on: only the rest is sent.
              Start, Piece("Talulah lea".into()), Piece("ds Reunion".into()), Piece(" [1].".into()),
              // A call that diverges (a check's second ask) stops the stream for the job.
              Start, Piece("Someone else".into()), Start, Piece("Talulah leads Reunion [1]. More".into())] {
        out.push_str(&d.feed(e));
    }
    assert_eq!(out, "Talulah leads Reunion [1].");
}

#[test]
fn worker_result_parts_keep_the_contract() {
    use crate::serve::{AskRequest, WorkerJob, canary_passing, fit_result, heartbeat_body, trim_at_line, trim_text};
    // Citation text: at most 600 characters, cut at a line break when there is one.
    let text = format!("{}\n{}\n{}", "a".repeat(300), "b".repeat(250), "c".repeat(100));
    assert_eq!(trim_at_line(&text, 600), format!("{}\n{}", "a".repeat(300), "b".repeat(250)));
    assert_eq!(trim_at_line(&"x".repeat(700), 600).chars().count(), 600);
    assert_eq!(trim_at_line("short\ntext", 600), "short\ntext");
    assert_eq!(trim_at_line(&"é".repeat(601), 600), "é".repeat(600));
    // A title line, then one long paragraph (an operator file): cut at a word break, not after the title.
    let para = format!("Operator archive: Insider\n{}", "sugar donut ".repeat(100));
    let cut = trim_text(&para, 600, true);
    assert!(cut.starts_with("Operator archive: Insider\nsugar donut"), "{cut:?}");
    assert!(cut.chars().count() <= 600 && cut.chars().count() >= 590, "{}", cut.chars().count());
    assert!(para.starts_with(&cut) && para[cut.len()..].starts_with(' '), "a whole word: {cut:?}");
    // TREVOR_CITE_WORD_CUT=0: the old rule keeps only the title.
    assert_eq!(trim_text(&para, 600, false), "Operator archive: Insider");
    // A line break that keeps half or more still wins over a later word break.
    let lines = format!("{}\n{}", "a ".repeat(160).trim_end(), "b ".repeat(300));
    assert_eq!(trim_text(&lines, 600, true), "a ".repeat(160).trim_end());
    // No break at all: the hard cut, as before.
    assert_eq!(trim_text(&"x".repeat(700), 600, true), "x".repeat(600));
    // Both rules agree whenever the last line break inside 600 keeps at least 300 characters.
    for t in [text.as_str(), "short\ntext"] {
        assert_eq!(trim_text(t, 600, true), trim_text(t, 600, false));
    }
    // A result over 64 KB loses citation text first.
    let cites: Vec<serde_json::Value> = (0..200).map(|n| serde_json::json!({"n": n, "text": "z".repeat(600)})).collect();
    let big = serde_json::json!({"jobId": "j", "result": {"answer": "A", "citations": cites}});
    let fitted = fit_result(big);
    assert!(serde_json::to_vec(&fitted).unwrap().len() <= 64 * 1024);
    assert_eq!(fitted["result"]["citations"][0]["text"].as_str().unwrap().len(), 200);
    // The job and the request.
    let j: WorkerJob = serde_json::from_str(r#"{"jobId":"j1","question":"Q","server":"en","horizonPos":812,"rung":"cpu","profile":"slim","flags":[]}"#).unwrap();
    assert_eq!((j.job_id.as_str(), j.horizon_pos, j.rung.as_deref(), j.profile.as_deref()), ("j1", Some(812), Some("cpu"), Some("slim")));
    let j: WorkerJob = serde_json::from_str(r#"{"jobId":"j2","question":"Q","horizonPos":null}"#).unwrap();
    assert_eq!(j.horizon_pos, None);
    let r: AskRequest = serde_json::from_str(r#"{"question":"Q","horizonPos":5,"profile":"slim"}"#).unwrap();
    assert_eq!((r.horizon_pos, r.profile.as_deref()), (Some(5), Some("slim")));
    // The heartbeat.
    let hb = heartbeat_body("w", serde_json::json!({"cf": false, "cpu": true}), "en-1", None, None);
    assert_eq!(hb, serde_json::json!({"workerId": "w", "rungs": {"cf": false, "cpu": true}, "corpusVersion": "en-1", "busyJobId": null}));
    // The canary: a passing id counts while it is in the list and at most 8 days old.
    let p = std::env::temp_dir().join(format!("trevor-canary-test-{}.json", std::process::id()));
    std::fs::write(&p, r#"{"checkedAt": 1000000, "models": [], "passing": "@cf/b"}"#).unwrap();
    let models = ["@cf/a".to_owned(), "@cf/b".to_owned()];
    assert_eq!(canary_passing(&p, &models, 1_000_000 + 86_400).as_deref(), Some("@cf/b"));
    assert_eq!(canary_passing(&p, &models, 1_000_000 + 9 * 86_400), None);
    assert_eq!(canary_passing(&p, &models[..1], 1_000_000), None);
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn profiles_slim_sets_the_cpu_pipeline_and_full_changes_nothing() {
    use crate::cli::{Args, Profile, RouterKind, apply_profile};
    let parse = |v: &[&str]| <Args as clap::Parser>::parse_from(std::iter::once("ask").chain(v.iter().copied()));
    let mut full = parse(&["q"]);
    apply_profile(&mut full, Profile::Full);
    let base = parse(&["q"]);
    assert_eq!((full.router == base.router, full.k, full.max_context_tokens, full.n_predict, full.no_typed_fallback, full.no_answer_extend),
               (true, base.k, base.max_context_tokens, base.n_predict, base.no_typed_fallback, base.no_answer_extend));
    let mut slim = parse(&["q", "--profile", "slim"]);
    apply_profile(&mut slim, Profile::Slim);
    assert!(slim.router == RouterKind::Keywords && slim.no_typed_fallback && slim.no_answer_extend && slim.no_answer_check
            && slim.no_operator_list_check && !slim.retry);
    assert_eq!((slim.k, slim.max_context_tokens, slim.n_predict), (5, 2000, 250));
    assert!(!crate::cli::form_on(&slim), "the keyword router has no form call");
}

/// A fake backend that records (path, x-trevor-worker-key, body) of `n` requests and answers 200 (or 404 for a path
/// containing "gone").
async fn fake_backend(n: usize) -> (String, tokio::task::JoinHandle<Vec<(String, String, serde_json::Value)>>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", l.local_addr().unwrap());
    let h = tokio::spawn(async move {
        let mut seen = Vec::new();
        for _ in 0..n {
            let (mut s, _) = l.accept().await.unwrap();
            let mut buf = Vec::new();
            let mut tmp = [0u8; 4096];
            let (head, body) = loop {
                let k = s.read(&mut tmp).await.unwrap();
                buf.extend_from_slice(&tmp[..k]);
                let t = String::from_utf8_lossy(&buf).to_string();
                if let Some(i) = t.find("\r\n\r\n") {
                    let len: usize = t[..i].lines().find_map(|x| x.to_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse().unwrap())).unwrap_or(0);
                    if buf.len() >= i + 4 + len {
                        break (t[..i].to_owned(), t[i + 4..i + 4 + len].to_owned());
                    }
                }
            };
            let path = head.split_whitespace().nth(1).unwrap_or_default().to_owned();
            let key = head.lines().find_map(|x| x.to_lowercase().strip_prefix("x-trevor-worker-key:").map(|v| v.trim().to_owned())).unwrap_or_default();
            let status = if path.contains("gone") { 404 } else { 200 };
            seen.push((path, key, serde_json::from_str(&body).unwrap_or(serde_json::Value::Null)));
            s.write_all(format!("HTTP/1.1 {status} X\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes()).await.unwrap();
            s.shutdown().await.unwrap();
        }
        seen
    });
    (base, h)
}

#[tokio::test]
async fn worker_heartbeat_and_deltas_reach_the_fake_backend() {
    use crate::serve::{Api, Probe, forward_deltas, heartbeat_loop};
    use trevor::goldgen::llm::StreamEvent::{Piece, Start};
    let (base, h) = fake_backend(3).await;
    let api = Api { http: reqwest::Client::new(), base, key: "secret".into(), worker_id: "w1".into(), backoff: Vec::new() };
    // A heartbeat while a job is busy: cf off (no canary), cpu off (nothing on the probe port).
    let busy = std::sync::Arc::new(std::sync::Mutex::new(Some("job-1".to_owned())));
    let probe = Probe { http: reqwest::Client::new(), llama: "http://127.0.0.1:9".into(), cf_models: Some(vec!["@cf/x".into()]),
                        canary: std::path::PathBuf::from("/nonexistent/canary.json") };
    let hb = tokio::spawn(heartbeat_loop(api.clone(), probe, "en-abc".into(), busy, std::time::Duration::from_secs(3600), None));
    // Two deltas: the events waiting at a send are coalesced, and a repeated prefix is not resent.
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    for e in [Start, Piece("Tal".into()), Piece("ulah".into())] {
        tx.send(e).unwrap();
    }
    let fwd = tokio::spawn(forward_deltas(api.clone(), "job-1".into(), rx));
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    for e in [Start, Piece("Talulah".into()), Piece(" [1].".into())] {
        tx.send(e).unwrap();
    }
    drop(tx);
    assert_eq!(fwd.await.unwrap(), 2);
    let mut seen = h.await.unwrap();
    hb.abort();
    seen.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(seen.iter().all(|(_, k, _)| k == "secret"), "every call carries x-trevor-worker-key");
    assert_eq!(seen[0].0, "/api/trevor/worker/heartbeat");
    assert_eq!(seen[0].2, serde_json::json!({"workerId": "w1", "rungs": {"cf": false, "cpu": false}, "corpusVersion": "en-abc", "busyJobId": "job-1"}));
    let deltas: Vec<&serde_json::Value> = seen.iter().filter(|s| s.0 == "/api/trevor/worker/jobs/job-1/delta").map(|s| &s.2).collect();
    assert_eq!(deltas.len(), 2);
    let text: String = { let mut d = deltas.clone(); d.sort_by_key(|v| v["seq"].as_u64()); d.iter().map(|v| v["text"].as_str().unwrap().to_owned()).collect() };
    assert_eq!(text, "Talulah [1].");
    assert!(deltas.iter().all(|v| v["workerId"] == "w1"));
}
