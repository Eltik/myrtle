//! Parity of the Rust reranker head against sentence-transformers.
//!
//! Needs the model and a reference file, so it is skipped unless
//! `TREVOR_RERANK_DIR` (the model directory), `TREVOR_RERANK_PAIRS` (JSON
//! `[{"q","d"}]`) and `TREVOR_RERANK_REF` (JSON scores from
//! `CrossEncoder.predict`) are set.
#![cfg(feature = "embed-core")]

use trevor::search::rerank::{RERANK_MAX_TOKENS, Reranker};

#[test]
fn scores_match_sentence_transformers() {
    let (Ok(dir), Ok(pairs), Ok(reference)) = (
        std::env::var("TREVOR_RERANK_DIR"),
        std::env::var("TREVOR_RERANK_PAIRS"),
        std::env::var("TREVOR_RERANK_REF"),
    ) else {
        eprintln!("skipped: TREVOR_RERANK_DIR / _PAIRS / _REF not set");
        return;
    };
    let pairs: Vec<serde_json::Value> =
        serde_json::from_slice(&std::fs::read(pairs).expect("pairs")).expect("pairs json");
    let want: Vec<f32> =
        serde_json::from_slice(&std::fs::read(reference).expect("ref")).expect("ref json");
    let rr =
        Reranker::load(dir.as_ref(), "onnx/model.onnx", RERANK_MAX_TOKENS, None).expect("load");
    let mut worst = 0f32;
    for (p, w) in pairs.iter().zip(&want) {
        let got = rr
            .score(p["q"].as_str().expect("q"), &[p["d"].as_str().expect("d")])
            .expect("score")[0];
        worst = worst.max((got - w).abs());
    }
    eprintln!(
        "{} pairs, max |rust - sentence-transformers| = {worst:.2e}",
        want.len()
    );
    assert!(worst < 1e-3, "max abs diff {worst}");
}
