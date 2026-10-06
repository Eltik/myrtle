//! `closed-book` (can the generator answer without the passage?) and `judge` (the judge model's
//! answerable and self-contained verdicts).

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use anyhow::{Result, bail};
use futures::StreamExt as _;
use trevor::goldgen::llm::{Llm, Request};
use trevor::goldgen::{ClosedBookAnswer, Filtered, Generated, Judged, self as gg};
use trevor::search::store::ChunkStore;

use crate::files::{append, generated_by_id, read_jsonl};
use crate::prompt::passage_prompt;

pub(crate) async fn cmd_closed_book(work: &Path, server: &str) -> Result<()> {
    let llm = Llm::connect(server).await?;
    let gens = generated_by_id(work)?;
    let passing: Vec<Filtered> = read_jsonl::<Filtered>(&work.join("filtered.jsonl"))?
        .into_iter()
        .filter(|f| f.pass)
        .collect();
    let out = work.join("closed_book.jsonl");
    let done: BTreeSet<String> = read_jsonl::<ClosedBookAnswer>(&out)?
        .into_iter()
        .map(|c| c.id)
        .collect();
    let todo: Vec<&Generated> = passing
        .iter()
        .filter(|f| !done.contains(&f.id))
        .filter_map(|f| gens.get(&f.id))
        .collect();
    eprintln!("closed-book: {} to do with {}", todo.len(), llm.model);
    let mut stream = futures::stream::iter(todo.into_iter().map(|g| {
        let llm = llm.clone();
        async move {
            let q = g.question.clone().unwrap_or_default();
            let c = llm
                .complete(&Request {
                    system: gg::CLOSED_BOOK,
                    user: &format!("QUESTION: {q}"),
                    grammar: None,
                    seed: 0,
                    temperature: 0.0,
                    n_predict: 96,
                    stop: &[],
                })
                .await?;
            anyhow::Ok(ClosedBookAnswer {
                id: g.id.clone(),
                // The first non-empty line: some templates open with a newline.
                answer: c
                    .content
                    .lines()
                    .map(str::trim)
                    .find(|l| !l.is_empty())
                    .unwrap_or("")
                    .to_owned(),
                model: llm.model.clone(),
            })
        }
    }))
    .buffer_unordered(llm.slots);
    let mut n = 0;
    while let Some(r) = stream.next().await {
        append(&out, &[r?])?;
        n += 1;
    }
    eprintln!("closed-book: {n} answered");
    Ok(())
}

pub(crate) async fn cmd_judge(work: &Path, server: &str, corpus: &Path) -> Result<()> {
    let llm = Llm::connect(server).await?;
    let store = ChunkStore::load(&corpus.join("chunks.jsonl"))?;
    let gens = generated_by_id(work)?;
    let cb: HashMap<String, String> =
        read_jsonl::<ClosedBookAnswer>(&work.join("closed_book.jsonl"))?
            .into_iter()
            .map(|c| (c.id, c.answer))
            .collect();
    let passing: Vec<Filtered> = read_jsonl::<Filtered>(&work.join("filtered.jsonl"))?
        .into_iter()
        .filter(|f| f.pass)
        .collect();
    let out = work.join("judged.jsonl");
    let done: BTreeSet<String> = read_jsonl::<Judged>(&out)?
        .into_iter()
        .map(|j| j.id)
        .collect();
    let todo: Vec<(&Generated, Vec<String>)> = passing
        .iter()
        .filter(|f| !done.contains(&f.id))
        .filter_map(|f| gens.get(&f.id).map(|g| (g, f.evidence(g))))
        .collect();
    if gens.values().any(|g| g.model == llm.model) {
        bail!(
            "the judge server runs {}, the model that generated the questions; start the judge model instead",
            llm.model
        );
    }
    eprintln!("judge: {} to do with {}", todo.len(), llm.model);
    let started = std::time::Instant::now();
    let store = &store;
    let cb = &cb;
    let mut stream = futures::stream::iter(todo.into_iter().map(|(g, evidence)| {
        let llm = llm.clone();
        async move {
            let q = g.question.clone().unwrap_or_default();
            let text = |i: usize| store.row(&g.chunk_ids[i]).map(|r| store.chunks[r].text.clone()).unwrap_or_default();
            let mut j = Judged { id: g.id.clone(), model: llm.model.clone(), ..Judged::default() };
            j.self_contained = llm.verdict(gg::JUDGE_SELF_CONTAINED, &format!("QUESTION: {q}"), gg::GRAMMAR_VERDICT).await?;
            if g.chunk_ids.len() == 2 {
                let (a, b) = (text(0), text(1));
                j.answerable = llm.verdict(gg::JUDGE_ANSWERABLE, &passage_prompt(&format!("{a}\n\n{b}"), &q), gg::GRAMMAR_VERDICT).await?;
                j.a_alone = Some(llm.verdict(gg::JUDGE_ANSWERABLE, &passage_prompt(&a, &q), gg::GRAMMAR_VERDICT).await?);
                j.b_alone = Some(llm.verdict(gg::JUDGE_ANSWERABLE, &passage_prompt(&b, &q), gg::GRAMMAR_VERDICT).await?);
            } else {
                j.answerable = llm.verdict(gg::JUDGE_ANSWERABLE, &passage_prompt(&text(0), &q), gg::GRAMMAR_VERDICT).await?;
            }
            if let Some(ans) = cb.get(&g.id) {
                j.closed_book_correct = Some(if ans.to_lowercase().contains("don't know") || ans.is_empty() {
                    false
                } else {
                    llm.verdict(
                        gg::JUDGE_CLOSED_BOOK,
                        &format!("QUESTION: {q}\n\nREFERENCE EVIDENCE:\n{}\n\nCANDIDATE ANSWER: {ans}", evidence.join("\n")),
                        gg::GRAMMAR_VERDICT,
                    )
                    .await?
                });
            }
            anyhow::Ok(j)
        }
    }))
    .buffer_unordered(llm.slots);
    let mut n = 0usize;
    while let Some(r) = stream.next().await {
        append(&out, &[r?])?;
        n += 1;
        if n % 25 == 0 {
            eprintln!(
                "  {n} judged, {:.1}/min",
                n as f64 / started.elapsed().as_secs_f64() * 60.0
            );
        }
    }
    eprintln!(
        "judge: {n} judged in {:.0}s",
        started.elapsed().as_secs_f64()
    );
    Ok(())
}
