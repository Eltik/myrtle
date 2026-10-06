//! Retrieval metrics over one ranked list. Pure functions; relevance is
//! binary (a row is gold or it is not).
//!
//! MRR is deliberately absent: it assumes one right answer at one rank, and
//! lore questions are routinely answered by several passages.

use std::collections::BTreeSet;

fn top(ranked: &[usize], k: usize) -> &[usize] {
    &ranked[..k.min(ranked.len())]
}

/// 1.0 if any gold row is in the top `k`.
#[must_use]
pub fn hit_at_k(ranked: &[usize], gold: &BTreeSet<usize>, k: usize) -> f64 {
    if top(ranked, k).iter().any(|r| gold.contains(r)) {
        1.0
    } else {
        0.0
    }
}

/// Fraction of gold rows found in the top `k`. The primary gated metric at
/// k = 10.
#[must_use]
pub fn recall_at_k(ranked: &[usize], gold: &BTreeSet<usize>, k: usize) -> f64 {
    if gold.is_empty() {
        return 0.0;
    }
    let found = top(ranked, k).iter().filter(|r| gold.contains(r)).count();
    #[allow(clippy::cast_precision_loss)]
    let v = found as f64 / gold.len() as f64;
    v
}

/// Fraction of the top `k` that is gold, over `k` itself (not over however
/// many were returned), so a short list is not rewarded.
#[must_use]
pub fn precision_at_k(ranked: &[usize], gold: &BTreeSet<usize>, k: usize) -> f64 {
    if k == 0 {
        return 0.0;
    }
    let found = top(ranked, k).iter().filter(|r| gold.contains(r)).count();
    #[allow(clippy::cast_precision_loss)]
    let v = found as f64 / k as f64;
    v
}

/// Binary-relevance nDCG. Diagnostic only: an LLM reads all k passages at
/// once, and its accuracy against gold position is U-shaped while nDCG falls
/// monotonically, so nDCG is never gated.
#[must_use]
pub fn ndcg_at_k(ranked: &[usize], gold: &BTreeSet<usize>, k: usize) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let disc = |i: usize| 1.0 / ((i + 2) as f64).log2();
    let dcg: f64 = top(ranked, k)
        .iter()
        .enumerate()
        .filter(|(_, r)| gold.contains(r))
        .map(|(i, _)| disc(i))
        .sum();
    let ideal: f64 = (0..gold.len().min(k)).map(disc).sum();
    if ideal == 0.0 { 0.0 } else { dcg / ideal }
}

/// 1-based rank of the first gold row, if any is retrieved at all.
#[must_use]
pub fn first_gold_rank(ranked: &[usize], gold: &BTreeSet<usize>) -> Option<usize> {
    ranked.iter().position(|r| gold.contains(r)).map(|p| p + 1)
}

/// The metrics every run reports, in a fixed order.
pub const METRICS: [&str; 8] = [
    "recall@10",
    "recall@20",
    "hit@1",
    "hit@5",
    "hit@10",
    "hit@20",
    "precision@5",
    "ndcg@10",
];

/// All per-query metrics, in [`METRICS`] order.
#[must_use]
pub fn all(ranked: &[usize], gold: &BTreeSet<usize>) -> Vec<(&'static str, f64)> {
    vec![
        ("recall@10", recall_at_k(ranked, gold, 10)),
        ("recall@20", recall_at_k(ranked, gold, 20)),
        ("hit@1", hit_at_k(ranked, gold, 1)),
        ("hit@5", hit_at_k(ranked, gold, 5)),
        ("hit@10", hit_at_k(ranked, gold, 10)),
        ("hit@20", hit_at_k(ranked, gold, 20)),
        ("precision@5", precision_at_k(ranked, gold, 5)),
        ("ndcg@10", ndcg_at_k(ranked, gold, 10)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_computed_values() {
        let gold: BTreeSet<usize> = [3, 7].into();
        let ranked = [1, 3, 5, 7, 9];
        assert!((hit_at_k(&ranked, &gold, 1) - 0.0).abs() < 1e-12);
        assert!((hit_at_k(&ranked, &gold, 2) - 1.0).abs() < 1e-12);
        assert!((recall_at_k(&ranked, &gold, 2) - 0.5).abs() < 1e-12);
        assert!((recall_at_k(&ranked, &gold, 10) - 1.0).abs() < 1e-12);
        assert!((precision_at_k(&ranked, &gold, 5) - 0.4).abs() < 1e-12);
        assert!(
            (precision_at_k(&ranked, &gold, 10) - 0.2).abs() < 1e-12,
            "short list not rewarded"
        );
        // DCG = 1/log2(3) + 1/log2(5); IDCG = 1 + 1/log2(3)
        let want = (1.0 / 3f64.log2() + 1.0 / 5f64.log2()) / (1.0 + 1.0 / 3f64.log2());
        assert!((ndcg_at_k(&ranked, &gold, 10) - want).abs() < 1e-12);
        assert_eq!(first_gold_rank(&ranked, &gold), Some(2));
        assert_eq!(first_gold_rank(&[1, 2], &gold), None);
    }
}
