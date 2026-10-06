//! `calibrate`: the judge's verdicts on passages known not to answer a question, and on questions known to
//! be bad or good.

use std::path::Path;

use anyhow::{Context, Result};
use trevor::goldgen::filters::fnv1a;
use trevor::goldgen::llm::Llm;
use trevor::goldgen::sample;
use trevor::goldgen::{Filtered, Generated, self as gg};
use trevor::search::store::ChunkStore;

use crate::files::{generated_by_id, read_jsonl};
use crate::prompt::passage_prompt;

/// Questions whose self-containedness is known, for calibrating the judge.
pub(crate) const BAD_QUESTIONS: [&str; 10] = [
    "What does he want from her in this scene?",
    "Why does the speaker refuse the offer?",
    "What happens right after that?",
    "Who is being talked about in the passage?",
    "What did they decide to do about it?",
    "Why is she so upset here?",
    "What does the character reveal at the end of the conversation?",
    "What is the reason for his anger in the text above?",
    "Where are they going next?",
    "What does it do when the others arrive?",
];
pub(crate) const GOOD_QUESTIONS: [&str; 10] = [
    "Why does Ch'en leave the Lungmen Guard Department?",
    "Who does the Duke of Caster think might be Victoria's chosen one?",
    "What final wish does Faust leave Mephisto with?",
    "Where does the Doctor decide to take young Amiya for her Oripathy treatment?",
    "What was forced into Specter's spinal cord?",
    "Who serves as Siesta's Catastrophe Messenger?",
    "Why does Kal'tsit warn Nightingale to stop while she is suppressing the Confessarius?",
    "What names is Sciurus considering for her baby?",
    "Which Columbian law about the Infected is used to pressure the mayor of Siesta?",
    "What did Hoshiguma decide about the former Reunion member who now sells soda in Lungmen?",
];

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Calibration {
    pub(crate) judge: String,
    pub(crate) random_passage_trials: usize,
    pub(crate) random_passage_false_positives: usize,
    pub(crate) random_passage_fp_wilson95: (f64, f64),
    pub(crate) source_passage_trials: usize,
    pub(crate) source_passage_true: usize,
    pub(crate) bad_questions_passed: usize,
    pub(crate) good_questions_failed: usize,
}

pub(crate) async fn cmd_calibrate(work: &Path, server: &str, corpus: &Path, n: usize) -> Result<()> {
    let llm = Llm::connect(server).await?;
    let store = ChunkStore::load(&corpus.join("chunks.jsonl"))?;
    let gens = generated_by_id(work)?;
    let mut passing: Vec<Filtered> = read_jsonl::<Filtered>(&work.join("filtered.jsonl"))?
        .into_iter()
        .filter(|f| f.pass && !f.id.contains("multi_hop"))
        .collect();
    passing.sort_by_key(|f| fnv1a(&format!("cal{}", f.id)));
    let items: Vec<&Generated> = passing
        .iter()
        .take(n)
        .filter_map(|f| gens.get(&f.id))
        .collect();
    let (mut fp, mut src_true) = (0, 0);
    for g in &items {
        let q = g.question.clone().unwrap_or_default();
        let src_row = store.row(&g.chunk_ids[0]).context("source chunk missing")?;
        let src_group = store.chunks[src_row].group_id.clone();
        // A random passage from another group cannot answer a question
        // written from this one, so every "true" here is a false positive.
        let mut rng = sample::Rng(fnv1a(&g.id));
        let other = loop {
            let r = rng.below(store.len());
            if store.chunks[r].group_id != src_group
                && store.chunks[r].token_count >= sample::MIN_TOKENS
            {
                break r;
            }
        };
        if llm
            .verdict(
                gg::JUDGE_ANSWERABLE,
                &passage_prompt(&store.chunks[other].text, &q),
                gg::GRAMMAR_VERDICT,
            )
            .await?
        {
            fp += 1;
        }
        if llm
            .verdict(
                gg::JUDGE_ANSWERABLE,
                &passage_prompt(&store.chunks[src_row].text, &q),
                gg::GRAMMAR_VERDICT,
            )
            .await?
        {
            src_true += 1;
        }
    }
    let mut bad_passed = 0;
    for q in BAD_QUESTIONS {
        if llm
            .verdict(
                gg::JUDGE_SELF_CONTAINED,
                &format!("QUESTION: {q}"),
                gg::GRAMMAR_VERDICT,
            )
            .await?
        {
            bad_passed += 1;
        }
    }
    let mut good_failed = 0;
    for q in GOOD_QUESTIONS {
        if !llm
            .verdict(
                gg::JUDGE_SELF_CONTAINED,
                &format!("QUESTION: {q}"),
                gg::GRAMMAR_VERDICT,
            )
            .await?
        {
            good_failed += 1;
        }
    }
    let cal = Calibration {
        judge: llm.model.clone(),
        random_passage_trials: items.len(),
        random_passage_false_positives: fp,
        random_passage_fp_wilson95: trevor::eval::stats::wilson(
            fp,
            items.len(),
            trevor::eval::stats::Z95,
        ),
        source_passage_trials: items.len(),
        source_passage_true: src_true,
        bad_questions_passed: bad_passed,
        good_questions_failed: good_failed,
    };
    std::fs::write(
        work.join("calibration.json"),
        serde_json::to_vec_pretty(&cal)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&cal)?);
    Ok(())
}
