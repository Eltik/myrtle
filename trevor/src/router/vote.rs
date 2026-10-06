//! The kNN vote over labelled example questions, and the routing decision from it.

use crate::tools::{self, Route};
use super::args::{Dict, extract_args};

/// The nearest-neighbour vote for one question.
#[derive(Debug, Clone)]
pub struct Vote {
    /// The winning tool (possibly `retrieve`) before the thresholds.
    pub tool: String,
    /// Cosine of the nearest example.
    pub top: f32,
    /// The winner's share of the similarity-weighted vote.
    pub share: f32,
}

/// The `n` nearest examples to `v` by cosine (unit vectors), best first, leaving out those `skip` names.
#[must_use]
pub fn neighbours(examples: &[(Vec<f32>, String)], v: &[f32], n: usize, skip: impl Fn(usize) -> bool) -> Vec<(f32, usize)> {
    let mut sims: Vec<(f32, usize)> = examples.iter().enumerate().filter(|(i, _)| !skip(*i))
        .map(|(i, (e, _))| (e.iter().zip(v).map(|(a, b)| a * b).sum::<f32>(), i)).collect();
    let n = n.max(1).min(sims.len());
    if n > 0 && n < sims.len() {
        sims.select_nth_unstable_by(n - 1, |a, b| b.0.total_cmp(&a.0));
        sims.truncate(n);
    }
    sims.sort_by(|a, b| b.0.total_cmp(&a.0));
    sims
}

/// The similarity-weighted vote of the first `k` of `neigh`; ties go to the tool of the nearer example.
#[must_use]
pub fn vote_from(examples: &[(Vec<f32>, String)], neigh: &[(f32, usize)], k: usize) -> Vote {
    let sims = &neigh[..k.max(1).min(neigh.len())];
    let mut w: Vec<(String, f32, f32)> = Vec::new();
    for &(s, i) in sims {
        let t = &examples[i].1;
        match w.iter_mut().find(|x| &x.0 == t) {
            Some(x) => x.1 += s.max(0.0),
            None => w.push((t.clone(), s.max(0.0), s)),
        }
    }
    let total: f32 = w.iter().map(|x| x.1).sum();
    let best = w.iter().max_by(|a, b| a.1.total_cmp(&b.1).then(a.2.total_cmp(&b.2)));
    Vote {
        tool: best.map_or_else(|| tools::RETRIEVE.to_owned(), |b| b.0.clone()),
        top: sims.first().map_or(0.0, |s| s.0),
        share: best.map_or(0.0, |b| if total > 0.0 { b.1 / total } else { 0.0 }),
    }
}

/// k nearest neighbours over labelled examples; ties go to the tool of the nearer example.
#[must_use]
pub fn vote(examples: &[(Vec<f32>, String)], v: &[f32], k: usize, skip: Option<usize>) -> Vote {
    vote_from(examples, &neighbours(examples, v, k, |i| Some(i) == skip), k)
}

/// Thresholds of the kNN router: a table tool needs its nearest example at `min_sim` and `min_share` of the vote.
#[derive(Debug, Clone, Copy)]
pub struct KnnParams {
    pub k: usize,
    pub min_sim: f32,
    pub min_share: f32,
}

/// The routing decision from a vote: retrieval below either threshold, else the tool with extracted arguments
/// (retrieval when a required argument is not in the question).
#[must_use]
pub fn decide(v: &Vote, p: KnnParams, q: &str, d: &Dict) -> Route {
    if v.tool == tools::RETRIEVE || v.top < p.min_sim || v.share < p.min_share {
        return Route::retrieve();
    }
    extract_args(&v.tool, q, d).unwrap_or_else(Route::retrieve)
}
