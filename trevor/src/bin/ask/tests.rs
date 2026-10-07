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
