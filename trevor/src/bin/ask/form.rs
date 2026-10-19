//! The question form call (fact, yes/no or opinion, with an evidence query) and the answer-rule flags.

use anyhow::Result;
use serde::Deserialize;
use trevor::llm::{Llm, Request};

use crate::detect::{is_canon_question, is_source_question, is_synthesis_question, is_time_question};

/// The question-form call: prompt and grammar.
pub(crate) const FORM_SYSTEM: &str = include_str!("../../../prompts/form.system.txt");
/// The second look at an opinion form (`--form-v1` turns it off): the same prompt with opinion kept for judgments
/// across people, and a question about one character's trait, feelings or likes sent to fact or yes_no ("Is Ulpianus
/// talkative or untalkative?" as an opinion gathered other operators' files as candidates and quoted Nightblade's file as
/// his). It runs only when the first call says opinion, and an opinion it confirms keeps the first call's evidence query,
/// so fact and yes/no questions and confirmed opinion questions retrieve exactly as before.
pub(crate) const FORM_SYSTEM_V2: &str = include_str!("../../../prompts/form.system.v2.txt");
pub(crate) const FORM_GRAMMAR: &str = include_str!("../../../prompts/form.gbnf");

/// Which answer rules a retrieval question needs: time (TIME_RULE and the timeline notes), canon (CANON_RULE), a
/// comparison or overview (SYNTH_RULE) or one kind of source text (SOURCE_RULE), by the English keyword tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Flags {
    pub(crate) asks_time: bool,
    pub(crate) asks_canon: bool,
    pub(crate) asks_synthesis: bool,
    pub(crate) asks_source_text: bool,
}

impl Flags {
    pub(crate) fn keywords(q: &str) -> Flags {
        Flags {
            asks_time: is_time_question(q),
            asks_canon: is_canon_question(q),
            asks_synthesis: is_synthesis_question(q),
            asks_source_text: is_source_question(q),
        }
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
    /// The opinion gate held this opinion question back (`opinion_needs_candidates`): no candidates, OPINION_RULE.
    #[serde(skip)]
    pub(crate) gated: bool,
    /// For an opinion question naming two or more groups (organizations, nations, places, races): each group's topic
    /// name and its forms, set by `answer_one` (`Tools::named_group_topics`; `--no-group-evidence` leaves it empty).
    #[serde(skip)]
    pub(crate) groups: Vec<(String, Vec<String>)>,
}

/// One grammar-bound call to the answer model (about 0.5 s): fact, or yes_no or opinion with an evidence query. Output
/// that does not parse is a fact question, which leaves the answer as before.
pub(crate) async fn question_form(llm: &Llm, q: &str, v1: bool) -> Result<Form> {
    let ask = |system: &'static str| async move {
        let c = llm
            .complete(&Request::greedy(
                system,
                &format!("Q: {q}"),
                Some(FORM_GRAMMAR),
                80,
                &[],
            ))
            .await?;
        anyhow::Ok(serde_json::from_str::<Form>(c.content.trim()).unwrap_or_default())
    };
    let first = ask(FORM_SYSTEM).await?;
    if v1 || first.form != "opinion" {
        return Ok(first);
    }
    let second = ask(FORM_SYSTEM_V2).await?;
    Ok(if second.form == "opinion" || second.form.is_empty() {
        first
    } else {
        second
    })
}
