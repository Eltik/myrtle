//! The question form call (fact, yes/no or opinion, with an evidence query), the answer-rule flags and the
//! `--story-query` rewrite.

use anyhow::Result;
use serde::Deserialize;
use trevor::goldgen::llm::{Llm, Request};

use crate::detect::{is_canon_question, is_source_question, is_synthesis_question, is_time_question};

/// The question-form call: prompt and grammar.
pub(crate) const FORM_SYSTEM: &str = include_str!("../../../prompts/form.system.txt");
/// The second look at an opinion form (2026-10-01; `--form-v1` turns it off): the same prompt with opinion kept for
/// judgments across people, and a question about one character's trait, feelings or likes sent to fact or yes_no. The
/// first prompt classed "Is Ulpianus talkative or untalkative?", "is nian actually a nice person?" and "how does
/// greythroat feel about other people now?" as opinion, which gathered other operators' files as candidates and the
/// hedge "the story does not rank them"; the Ulpianus answer quoted Nightblade's file as his, and 6 of the 14 trivia
/// misses with the source chunk in the passages were one-character questions in the opinion form. It runs only when
/// the first call says opinion, and an opinion it confirms keeps the first call's evidence query, so fact and yes/no
/// questions and confirmed opinion questions retrieve exactly as before.
pub(crate) const FORM_SYSTEM_V2: &str = include_str!("../../../prompts/form.system.v2.txt");
pub(crate) const FORM_GRAMMAR: &str = include_str!("../../../prompts/form.gbnf");
/// The answer-rule flags (2026-10-01; opt-in with `--model-flags`): one more grammar-bound call beside the form call
/// says whether a retrieval question asks about time (TIME_RULE and the timeline notes), canon (CANON_RULE), a comparison
/// or overview (SYNTH_RULE) or one kind of source text (SOURCE_RULE), in place of the English keyword tests. A separate
/// call, so the form and its evidence query, and with them the passages, stay exactly as before.
pub(crate) const FLAGS_SYSTEM: &str = include_str!("../../../prompts/flags.system.txt");
pub(crate) const FLAGS_GRAMMAR: &str = include_str!("../../../prompts/flags.gbnf");

/// Which answer rules a retrieval question needs (`question_form` with `--model-flags`).
#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Flags {
    pub(crate) asks_time: bool,
    pub(crate) asks_canon: bool,
    pub(crate) asks_synthesis: bool,
    pub(crate) asks_source_text: bool,
}

impl Flags {
    /// The English keyword tests, used unless `--model-flags` is given and the form call runs.
    pub(crate) fn keywords(q: &str) -> Flags {
        Flags { asks_time: is_time_question(q), asks_canon: is_canon_question(q), asks_synthesis: is_synthesis_question(q),
                asks_source_text: is_source_question(q) }
    }

    pub(crate) fn show(self) -> String {
        [(self.asks_time, "time"), (self.asks_canon, "canon"), (self.asks_synthesis, "synthesis"),
         (self.asks_source_text, "source")].iter().filter(|(on, _)| *on).map(|(_, n)| *n).collect::<Vec<_>>().join(",")
    }
}

/// What kind of answer a retrieval question needs (`question_form`).
#[derive(Deserialize, Clone, Debug, Default)]
pub(crate) struct Form {
    pub(crate) form: String,
    /// A search query: for an opinion question in the words a character's file would use for the quality judged, for a
    /// yes/no question in the words a passage that settles it would use.
    pub(crate) evidence: Option<String>,
    /// For an opinion question: the characters it names, as (file story id, the name the question used, the file's
    /// name), set by `answer_one` from the corpus it retrieves from (not part of the model's output).
    #[serde(skip)]
    pub(crate) named: Vec<(String, String, String)>,
    /// `--story-query`: the question in the words the story would use (not part of the form call's output).
    #[serde(skip)]
    pub(crate) story: Option<String>,
    /// The answer-rule flags, from their own call (not part of the form call's output); None = the keyword tests.
    #[serde(skip)]
    pub(crate) flags: Option<Flags>,
    /// The opinion gate held this opinion question back (`opinion_needs_candidates`): no candidates, OPINION_RULE.
    #[serde(skip)]
    pub(crate) gated: bool,
}

/// One grammar-bound call to the answer model (about 0.5 s): fact, or yes_no or opinion with an evidence query. Output
/// that does not parse is a fact question, which leaves the answer as before.
pub(crate) async fn question_form(llm: &Llm, q: &str, v1: bool, flags: bool) -> Result<Form> {
    let ask = |system: &'static str| async move {
        let c = llm.complete(&Request::greedy(system, &format!("Q: {q}"), Some(FORM_GRAMMAR), 80, &[])).await?;
        anyhow::Ok(serde_json::from_str::<Form>(c.content.trim()).unwrap_or_default())
    };
    let first = ask(FORM_SYSTEM).await?;
    let mut form = if v1 || first.form != "opinion" { first } else {
        let second = ask(FORM_SYSTEM_V2).await?;
        if second.form == "opinion" || second.form.is_empty() { first } else { second }
    };
    if flags {
        // Output that does not parse leaves the keyword tests in charge (None).
        let c = llm.complete(&Request::greedy(FLAGS_SYSTEM, &format!("Q: {q}"), Some(FLAGS_GRAMMAR), 60, &[])).await?;
        form.flags = serde_json::from_str::<Flags>(c.content.trim()).ok();
    }
    Ok(form)
}

/// `--story-query`: the question as the story's own lines would put it.
pub(crate) const STORY_WORDS: &str = "Rewrite this question about the Arknights story as a search query in the words the story's own \
lines would use: the names it gives, and for what it asks about, the plain, blunt, casual or slang words characters would \
actually say (not formal or clinical terms). Output the query only, one line.";

pub(crate) async fn story_words(llm: &Llm, q: &str) -> Result<Option<String>> {
    let c = llm.complete(&Request::greedy(STORY_WORDS, q, None, 60, &["\n"])).await?;
    let t = c.content.trim().trim_matches('"').trim().to_owned();
    Ok((!t.is_empty()).then_some(t))
}
