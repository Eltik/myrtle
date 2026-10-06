//! Gold-set generation without a human in the loop.
//!
//! A generator model (Gemma 4 12B) writes one question per sampled passage
//! under a grammar. Deterministic filters then drop ungrounded, deictic,
//! wording-copying, BM25-trivial and duplicate questions. A judge model from
//! a different family (Qwen3.5 9B) takes the place of the human reviewer
//! with binary verdicts only: is the question self-contained, does the source
//! passage answer it, does each half of a multi-hop pair fail to answer it
//! alone, and is the generator's closed-book answer correct. The judge also
//! completes anchors: any other retrieved passage it says answers the
//! question becomes gold too, which is the "missed anchors" work a human
//! reviewer would otherwise do.
//!
//! Binary verdicts only, because small generative judges reach 89.55% on
//! binary rubric checks but collapse to Spearman 0.21 on graded scores. Two
//! families, so the judge is not grading its own generations.

pub mod filters;
pub mod llm;
pub mod sample;

use serde::{Deserialize, Serialize};

use crate::eval::goldset::Stratum;

pub const SYSTEM_SINGLE: &str = include_str!("../../prompts/goldset.system.txt");
pub const GRAMMAR_SINGLE: &str = include_str!("../../prompts/goldset.gbnf");
pub const SYSTEM_MULTI: &str = include_str!("../../prompts/multihop.system.txt");
pub const GRAMMAR_MULTI: &str = include_str!("../../prompts/multihop.gbnf");
pub const GRAMMAR_VERDICT: &str = include_str!("../../prompts/verdict.gbnf");
pub const JUDGE_ANSWERABLE: &str = include_str!("../../prompts/judge.answerable.txt");
pub const JUDGE_SELF_CONTAINED: &str = include_str!("../../prompts/judge.self_contained.txt");
pub const CLOSED_BOOK: &str = include_str!("../../prompts/closed_book.txt");
pub const JUDGE_CLOSED_BOOK: &str = include_str!("../../prompts/judge.closed_book.txt");
pub const UNANSWERABLE: &str = include_str!("../../prompts/unanswerable.txt");

#[must_use]
pub fn sha16(s: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(s.as_bytes()))[..16].to_owned()
}

/// What the generator returned for one sampled item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Generated {
    pub id: String,
    pub stratum: Stratum,
    pub chunk_ids: Vec<String>,
    pub entity: Option<String>,
    pub question: Option<String>,
    pub answerable: bool,
    /// One quote per passage.
    pub evidence: Vec<String>,
    pub stop_type: String,
    pub parse_error: Option<String>,
    pub model: String,
    pub prompt_sha: String,
    pub grammar_sha: String,
    pub seed: u64,
    pub ms: f64,
    /// Sampling temperature, recorded only when `--temperature` overrode the
    /// v1 default of 0.7, so v1 rows serialize byte for byte as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// The generation temperature gold set v1 used.
pub const V1_TEMPERATURE: f32 = 0.7;

/// The evidence rule both generation grammars share, verbatim.
const ESTRING_V1: &str = r#"estring ::= "\"" qchar{10,300} "\"""#;

/// Quote characters without the `\b \f \n \r \t` escapes.
const ECHAR_ONE_LINE: &str = r#"echar   ::= [^"\\\x00-\x1F] | "\\" (["\\/] | "u" hex hex hex hex)"#;

/// Rewrite the evidence-quote rule of a generation grammar. With neither
/// option set this returns `base` unchanged, byte for byte (the v1 grammar
/// and its sha). `max` lowers the 300-character cap, and `one_line` forbids
/// line breaks inside a quote: in gold set v1, 50 of 353 quotes ran to the
/// cap because the model kept quoting across speaker lines instead of
/// closing the string.
///
/// # Errors
/// If `base` has no v1 evidence rule to rewrite, or `max` is below 10.
pub fn evidence_grammar(base: &str, max: Option<usize>, one_line: bool) -> anyhow::Result<String> {
    if max.is_none() && !one_line {
        return Ok(base.to_owned());
    }
    anyhow::ensure!(
        base.contains(ESTRING_V1),
        "grammar has no v1 evidence rule to rewrite"
    );
    let cap = max.unwrap_or(300);
    anyhow::ensure!(cap >= 10, "--evidence-max must be at least 10");
    let class = if one_line { "echar" } else { "qchar" };
    let rule = format!(r#"estring ::= "\"" {class}{{10,{cap}}} "\"""#);
    let mut g = base.replace(ESTRING_V1, &rule);
    if one_line {
        g.push_str(ECHAR_ONE_LINE);
        g.push('\n');
    }
    Ok(g)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filtered {
    pub id: String,
    pub pass: bool,
    pub reasons: Vec<String>,
    /// Fraction of the question's word trigrams found in the passage(s).
    pub word_trigram_containment: f64,
    /// The spec's character-trigram Jaccard, recorded for comparison.
    pub char_trigram_jaccard: f64,
    /// Best BM25 rank of any source chunk (1-based), within the top 100.
    pub bm25_rank: Option<usize>,
    pub dup_of: Option<String>,
    /// The evidence with each rescued quote cut to its verbatim prefix.
    /// Present only when `--evidence-min-prefix` rescued a quote, so v1 rows
    /// serialize byte for byte as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grounded_evidence: Option<Vec<String>>,
}

impl Filtered {
    /// The evidence later stages should use: the grounded prefixes when the
    /// filter cut any quote, else the generator's quotes unchanged.
    #[must_use]
    pub fn evidence(&self, g: &Generated) -> Vec<String> {
        self.grounded_evidence
            .clone()
            .unwrap_or_else(|| g.evidence.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedBookAnswer {
    pub id: String,
    pub answer: String,
    pub model: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Judged {
    pub id: String,
    pub self_contained: bool,
    /// The source passage(s) together answer the question.
    pub answerable: bool,
    /// Multi-hop only: each passage alone answers it (both must be false).
    pub a_alone: Option<bool>,
    pub b_alone: Option<bool>,
    pub closed_book_correct: Option<bool>,
    pub model: String,
}

/// Word trigrams of a lowercased, alphanumeric-only string.
#[must_use]
pub fn word_trigrams(s: &str) -> std::collections::HashSet<[String; 3]> {
    let w: Vec<String> = s
        .to_lowercase()
        .split(|c: char| !(c.is_alphanumeric() || c == '\''))
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect();
    w.windows(3)
        .map(|x| [x[0].clone(), x[1].clone(), x[2].clone()])
        .collect()
}

/// Share of the question's word trigrams that occur in the passage. The
/// spec's character-trigram Jaccard divides by the union, which a 2,300
/// character passage dominates, so it stays near 0.05 whatever the question
/// copies; containment measures the copying the filter is for.
#[must_use]
pub fn containment(question: &str, passage: &str) -> f64 {
    let q = word_trigrams(question);
    if q.is_empty() {
        return 0.0;
    }
    let p = word_trigrams(passage);
    #[allow(clippy::cast_precision_loss)]
    let c = q.iter().filter(|g| p.contains(*g)).count() as f64 / q.len() as f64;
    c
}

#[cfg(test)]
mod evidence_grammar_tests {
    use super::*;

    #[test]
    fn off_is_byte_identical_to_v1() {
        for base in [GRAMMAR_SINGLE, GRAMMAR_MULTI] {
            let g = evidence_grammar(base, None, false).unwrap();
            assert_eq!(g, base);
            assert_eq!(sha16(&g), sha16(base));
        }
        // The shas gold set v1 recorded in eval/goldset_v1.meta.json.
        assert_eq!(sha16(GRAMMAR_SINGLE), "54a50ef9b9f8a61b");
        assert_eq!(sha16(GRAMMAR_MULTI), "5dc0bddf0e5670ab");
    }

    #[test]
    fn rewrites_only_the_evidence_rule() {
        for base in [GRAMMAR_SINGLE, GRAMMAR_MULTI] {
            let g = evidence_grammar(base, Some(160), true).unwrap();
            assert!(!g.contains("qchar{10,300}"));
            assert!(g.contains(r#"estring ::= "\"" echar{10,160} "\"""#));
            assert!(g.contains("echar   ::="));
            // The question rule still uses the full character class.
            assert!(g.contains("qchar{20,"));
            let capped = evidence_grammar(base, Some(160), false).unwrap();
            assert!(capped.contains("qchar{10,160}") && !capped.contains("echar"));
        }
        assert!(evidence_grammar("root ::= \"x\"", Some(100), false).is_err());
        assert!(evidence_grammar(GRAMMAR_SINGLE, Some(5), false).is_err());
    }
}
