//! A worker's answer as stored and served (contract section 3): citations,
//! their trimmed text, and where the answer came from.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;
use utoipa::ToSchema;

use super::{Profile, Rung};

/// Trim a cited passage to at most `max` characters, at the last line break
/// inside the limit when there is one.
pub fn trim_citation_text(text: &str, max: usize) -> String {
    let Some((cut, _)) = text.char_indices().nth(max) else {
        return text.to_owned();
    };
    let head = &text[..cut];
    match head.rfind('\n') {
        Some(i) if i > 0 => head[..i].trim_end().to_owned(),
        _ => head.to_owned(),
    }
}

/// One cited passage.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    pub n: u32,
    pub story_id: String,
    pub chunk_id: String,
    pub line_start: u32,
    pub line_end: u32,
    pub text: String,
    /// The cited chunk's text sha (contract v1.1 section 3b); answers whose
    /// every citation carries one can be carried to a new version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
}

/// Whether a model wrote the answer or it is the top passages only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorAnswerSource")]
#[schema(as = TrevorAnswerSource)]
pub enum AnswerSource {
    Live,
    Passages,
}

impl AnswerSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Passages => "passages",
        }
    }

    pub fn parse(s: &str) -> Self {
        if s == "live" {
            Self::Live
        } else {
            Self::Passages
        }
    }
}

/// A worker's answer as stored and served. Unknown fields (the old full
/// `passages` list) are dropped on the way in.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkerAnswer {
    pub answer: String,
    pub rung: Rung,
    pub profile: Profile,
    pub source: AnswerSource,
    /// Only the cited passages, each trimmed to 600 characters.
    pub citations: Vec<Citation>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub route: Value,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub form: Value,
    pub corpus_version: String,
    #[serde(default)]
    pub tokens_in: u64,
    #[serde(default)]
    pub tokens_out: u64,
    /// `{routeMs, answerMs, totalMs}`.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub timings: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citations_trim_at_a_line_break_inside_the_limit() {
        assert_eq!(trim_citation_text("short", 600), "short");
        let text = format!("{}\n{}", "a".repeat(500), "b".repeat(200));
        assert_eq!(trim_citation_text(&text, 600), "a".repeat(500));
        // No break inside the limit: a hard cut at the character limit.
        let long = "\u{00e9}".repeat(700);
        assert_eq!(trim_citation_text(&long, 600).chars().count(), 600);
        let exact = "x".repeat(600);
        assert_eq!(trim_citation_text(&exact, 600), exact);
    }
}
