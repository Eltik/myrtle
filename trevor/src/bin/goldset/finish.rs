//! `finish`: the stratified gold set with its metadata and spot-check sheet.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use anyhow::{Result, bail};
use trevor::eval::goldset::{Anchor, ClosedBook, GoldItem, Review, Stratum};
use trevor::goldgen::filters::fnv1a;
use trevor::goldgen::llm::Llm;
use trevor::goldgen::{ClosedBookAnswer, Filtered, Generated, Judged, self as gg, sha16};
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::{Runtime, RuntimeArgs};
use trevor::search::store::ChunkStore;

use crate::calibrate::Calibration;
use crate::files::{read_jsonl, write_jsonl};
use crate::prompt::passage_prompt;

/// Final set size per stratum. Ambiguous-entity (10) and spoiler-boundary
/// (5) items need hand construction and are deferred; unanswerable items
/// come from prompts/unanswerable.txt.
pub(crate) const QUOTAS: [(Stratum, usize); 6] = [
    (Stratum::SingleFact, 35),
    (Stratum::Entity, 25),
    (Stratum::Causal, 20),
    (Stratum::Temporal, 15),
    (Stratum::Aggregation, 10),
    (Stratum::MultiHop, 30),
];
pub(crate) const UNANSWERABLE_QUOTA: usize = 15;
/// Above this the eval measures pretraining, not retrieval (eval spec 2.4).
pub(crate) const CLOSED_BOOK_GATE: f64 = 0.30;
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Meta {
    pub(crate) judge_calibration: Option<Calibration>,
    pub(crate) goldset_version: &'static str,
    pub(crate) built_at_unix: u64,
    pub(crate) n: usize,
    pub(crate) strata: BTreeMap<String, usize>,
    pub(crate) shortfall: BTreeMap<String, usize>,
    pub(crate) deferred: Vec<&'static str>,
    pub(crate) funnel: BTreeMap<String, usize>,
    pub(crate) closed_book_rate_candidates: f64,
    pub(crate) closed_book_rate_selected: f64,
    pub(crate) closed_book_remediated: bool,
    pub(crate) bm25_rank1_share_selected_single: f64,
    pub(crate) anchors_added: usize,
    pub(crate) items_with_added_anchors: usize,
    pub(crate) unanswerable_dropped: Vec<String>,
    pub(crate) generator: String,
    pub(crate) judge: String,
    pub(crate) prompt_shas: BTreeMap<&'static str, String>,
    /// Rows per `grammarSha@temperature`, present only when some row was
    /// generated with a knob that differs from v1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) generation_variants: Option<BTreeMap<String, usize>>,
    pub(crate) review: &'static str,
}

pub(crate) async fn cmd_finish(
    work: &Path,
    server: &str,
    runtime: &RuntimeArgs,
    out: &Path,
    pool_depth: usize,
) -> Result<()> {
    let judge = Llm::connect(server).await?;
    let mut rt = Runtime::load(runtime, true, true, false)?;
    let gens: Vec<Generated> = read_jsonl(&work.join("generated.jsonl"))?;
    let filtered: HashMap<String, Filtered> = read_jsonl::<Filtered>(&work.join("filtered.jsonl"))?
        .into_iter()
        .map(|f| (f.id.clone(), f))
        .collect();
    let judged: HashMap<String, Judged> = read_jsonl::<Judged>(&work.join("judged.jsonl"))?
        .into_iter()
        .map(|j| (j.id.clone(), j))
        .collect();
    let cb: HashMap<String, String> =
        read_jsonl::<ClosedBookAnswer>(&work.join("closed_book.jsonl"))?
            .into_iter()
            .map(|c| (c.id, c.answer))
            .collect();
    let generator = gens.first().map(|g| g.model.clone()).unwrap_or_default();
    if judge.model == generator {
        bail!("finish needs the judge model, not the generator ({generator})");
    }

    let mut funnel: BTreeMap<String, usize> = BTreeMap::new();
    funnel.insert("1 generated".into(), gens.len());
    funnel.insert(
        "2 passed deterministic filters".into(),
        filtered.values().filter(|f| f.pass).count(),
    );
    let ok = |g: &Generated| -> bool {
        let Some(j) = judged.get(&g.id) else {
            return false;
        };
        let multi_ok = j.a_alone != Some(true) && j.b_alone != Some(true);
        filtered.get(&g.id).is_some_and(|f| f.pass) && j.self_contained && j.answerable && multi_ok
    };
    let mut candidates: Vec<&Generated> = gens.iter().filter(|g| ok(g)).collect();
    funnel.insert(
        "3 judged self-contained".into(),
        judged.values().filter(|j| j.self_contained).count(),
    );
    funnel.insert("4 passed every judge check".into(), candidates.len());
    candidates.sort_by_key(|g| fnv1a(&g.id));

    let cb_correct = |g: &Generated| {
        judged
            .get(&g.id)
            .and_then(|j| j.closed_book_correct)
            .unwrap_or(false)
    };
    #[allow(clippy::cast_precision_loss)]
    let rate = |xs: &[&Generated]| {
        if xs.is_empty() {
            0.0
        } else {
            xs.iter().filter(|g| cb_correct(g)).count() as f64 / xs.len() as f64
        }
    };
    let cand_rate = rate(&candidates);

    let pick = |prefer_unknown: bool| -> Vec<&Generated> {
        let mut sel = Vec::new();
        for (s, quota) in QUOTAS {
            let mut pool: Vec<&Generated> = candidates
                .iter()
                .copied()
                .filter(|g| g.stratum == s)
                .collect();
            if prefer_unknown {
                pool.sort_by_key(|g| (cb_correct(g), fnv1a(&g.id)));
            }
            sel.extend(pool.into_iter().take(quota));
        }
        sel
    };
    let mut selected = pick(false);
    let mut remediated = false;
    if rate(&selected) > CLOSED_BOOK_GATE {
        // Spec remediation 1: prefer questions the model could not answer
        // from memory, which drops the wiki-famous ones first.
        selected = pick(true);
        remediated = true;
    }
    let sel_rate = rate(&selected);
    funnel.insert("5 selected".into(), selected.len());

    // Anchor completion: any pooled passage the judge says answers the
    // question on its own becomes gold as well.
    let cfgs = [Mode::Bm25, Mode::Dense, Mode::Hybrid].map(|m| RetrievalConfig {
        mode: m,
        k: pool_depth,
        ..RetrievalConfig::default()
    });
    let mut items = Vec::new();
    let (mut added_total, mut items_added) = (0, 0);
    let started = std::time::Instant::now();
    for (n, g) in selected.iter().enumerate() {
        let q = g.question.clone().unwrap_or_default();
        let gold: BTreeSet<usize> = g
            .chunk_ids
            .iter()
            .filter_map(|id| rt.store.row(id))
            .collect();
        let mut pooled: BTreeSet<usize> = BTreeSet::new();
        for c in &cfgs {
            pooled.extend(
                rt.retrieve(&q, c)?
                    .into_iter()
                    .map(|h| h.row)
                    .filter(|r| !gold.contains(r)),
            );
        }
        let checks = futures::future::join_all(pooled.iter().map(|&row| {
            let judge = judge.clone();
            let prompt = passage_prompt(&rt.store.chunks[row].text, &q);
            async move {
                judge
                    .verdict(gg::JUDGE_ANSWERABLE, &prompt, gg::GRAMMAR_VERDICT)
                    .await
                    .map(|v| (row, v))
            }
        }))
        .await;
        let mut anchors: Vec<Anchor> = g
            .chunk_ids
            .iter()
            .filter_map(|id| rt.store.row(id))
            .map(|r| {
                let c = &rt.store.chunks[r];
                Anchor {
                    story_id: c.story_id.clone(),
                    line_start: c.line_start,
                    line_end: c.line_end,
                }
            })
            .collect();
        let mut added = Vec::new();
        for r in checks {
            let (row, yes) = r?;
            if yes {
                let c = &rt.store.chunks[row];
                anchors.push(Anchor {
                    story_id: c.story_id.clone(),
                    line_start: c.line_start,
                    line_end: c.line_end,
                });
                added.push(c.chunk_id.clone());
            }
        }
        if !added.is_empty() {
            items_added += 1;
            added_total += added.len();
        }
        let f = &filtered[&g.id];
        let j = &judged[&g.id];
        let item = GoldItem {
            qid: format!("g{:04}", n + 1),
            question: q,
            anchors,
            stratum: g.stratum,
            source_chunk_id: g.chunk_ids.first().cloned(),
            generated_by: Some(format!("{}@{}", g.model, g.prompt_sha)),
            reviewed_by: Some(format!("auto:{}", judge.model)),
            review: Review::Accepted,
            closed_book: j.closed_book_correct.map(|correct| ClosedBook {
                model: g.model.clone(),
                correct,
                checked_at: "2026-09-25".into(),
            }),
            notes: None,
        };
        let mut v = serde_json::to_value(&item)?;
        v["evidence"] = serde_json::json!(f.evidence(g));
        v["gen_id"] = serde_json::json!(g.id);
        v["anchors_added_by_judge"] = serde_json::json!(added);
        v["closed_book_answer"] = serde_json::json!(cb.get(&g.id));
        v["filters"] = serde_json::json!({"word_trigram_containment": f.word_trigram_containment, "bm25_rank": f.bm25_rank});
        items.push(v);
        if (n + 1) % 25 == 0 {
            eprintln!(
                "  anchors: {} of {} items, {:.0}s",
                n + 1,
                selected.len(),
                started.elapsed().as_secs_f64()
            );
        }
    }

    // Unanswerable: keep a hand-written question only if no pooled passage
    // answers it.
    let mut dropped = Vec::new();
    let mut kept_unanswerable = 0;
    for q in gg::UNANSWERABLE
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        if kept_unanswerable >= UNANSWERABLE_QUOTA {
            break;
        }
        let mut pooled: BTreeSet<usize> = BTreeSet::new();
        for c in &cfgs {
            pooled.extend(rt.retrieve(q, c)?.into_iter().map(|h| h.row));
        }
        let checks = futures::future::join_all(pooled.iter().map(|&row| {
            let judge = judge.clone();
            let prompt = passage_prompt(&rt.store.chunks[row].text, q);
            async move {
                judge
                    .verdict(gg::JUDGE_ANSWERABLE, &prompt, gg::GRAMMAR_VERDICT)
                    .await
            }
        }))
        .await;
        let mut answered = false;
        for c in checks {
            answered |= c?;
        }
        if answered {
            dropped.push(q.to_owned());
            continue;
        }
        kept_unanswerable += 1;
        let item = GoldItem {
            qid: format!("u{kept_unanswerable:03}"),
            question: q.to_owned(),
            anchors: vec![],
            stratum: Stratum::Unanswerable,
            source_chunk_id: None,
            generated_by: Some("hand-written (Claude)".into()),
            reviewed_by: Some(format!("auto:{}", judge.model)),
            review: Review::Accepted,
            closed_book: None,
            notes: Some("no pooled passage judged to answer it".into()),
        };
        items.push(serde_json::to_value(&item)?);
    }

    let mut strata: BTreeMap<String, usize> = BTreeMap::new();
    for v in &items {
        *strata
            .entry(v["stratum"].as_str().unwrap_or("").to_owned())
            .or_default() += 1;
    }
    let mut shortfall = BTreeMap::new();
    for (s, q) in QUOTAS
        .iter()
        .copied()
        .chain([(Stratum::Unanswerable, UNANSWERABLE_QUOTA)])
    {
        let have = strata.get(s.name()).copied().unwrap_or(0);
        if have < q {
            shortfall.insert(s.name().to_owned(), q - have);
        }
    }
    let single_sel: Vec<&&Generated> = selected
        .iter()
        .filter(|g| g.stratum != Stratum::MultiHop)
        .collect();
    #[allow(clippy::cast_precision_loss)]
    let rank1 = if single_sel.is_empty() {
        0.0
    } else {
        single_sel
            .iter()
            .filter(|g| filtered[&g.id].bm25_rank == Some(1))
            .count() as f64
            / single_sel.len() as f64
    };
    let meta = Meta {
        judge_calibration: std::fs::read(work.join("calibration.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok()),
        goldset_version: "v1",
        built_at_unix: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        n: items.len(),
        strata,
        shortfall,
        deferred: vec![
            "ambiguous (10): needs shared-name and ??? masking cases built by hand",
            "spoiler (5): needs the progress-boundary design",
        ],
        funnel,
        closed_book_rate_candidates: cand_rate,
        closed_book_rate_selected: sel_rate,
        closed_book_remediated: remediated,
        bm25_rank1_share_selected_single: rank1,
        anchors_added: added_total,
        items_with_added_anchors: items_added,
        unanswerable_dropped: dropped,
        generator,
        judge: judge.model.clone(),
        prompt_shas: BTreeMap::from([
            ("goldset.system", sha16(gg::SYSTEM_SINGLE)),
            ("goldset.gbnf", sha16(gg::GRAMMAR_SINGLE)),
            ("multihop.system", sha16(gg::SYSTEM_MULTI)),
            ("multihop.gbnf", sha16(gg::GRAMMAR_MULTI)),
            ("judge.answerable", sha16(gg::JUDGE_ANSWERABLE)),
            ("judge.self_contained", sha16(gg::JUDGE_SELF_CONTAINED)),
            ("closed_book", sha16(gg::CLOSED_BOOK)),
            ("judge.closed_book", sha16(gg::JUDGE_CLOSED_BOOK)),
        ]),
        generation_variants: generation_variants(&gens),
        review: "automatic: deterministic filters plus binary judge verdicts; no human review",
    };
    write_jsonl(&out.with_extension("jsonl"), &items)?;
    std::fs::write(
        out.with_extension("meta.json"),
        serde_json::to_vec_pretty(&meta)?,
    )?;
    write_spotcheck(&out.with_extension("spotcheck.md"), &items, &rt.store)?;
    println!("{}", serde_json::to_string_pretty(&meta)?);
    Ok(())
}

/// `None` when every row used the v1 grammars and temperature.
pub(crate) fn generation_variants(gens: &[Generated]) -> Option<BTreeMap<String, usize>> {
    let v1 = [sha16(gg::GRAMMAR_SINGLE), sha16(gg::GRAMMAR_MULTI)];
    if gens
        .iter()
        .all(|g| g.temperature.is_none() && v1.contains(&g.grammar_sha))
    {
        return None;
    }
    let mut m = BTreeMap::new();
    for g in gens {
        let t = g.temperature.unwrap_or(gg::V1_TEMPERATURE);
        *m.entry(format!("{}@{t}", g.grammar_sha)).or_insert(0) += 1;
    }
    Some(m)
}

/// Twenty items, seeded, for an optional human spot-check that measures how
/// often the automatic review is wrong.
pub(crate) fn write_spotcheck(path: &Path, items: &[serde_json::Value], store: &ChunkStore) -> Result<()> {
    let mut idx: Vec<usize> = (0..items.len())
        .filter(|&i| items[i]["stratum"] != "unanswerable")
        .collect();
    idx.sort_by_key(|&i| fnv1a(&format!("spot{}", items[i]["qid"])));
    let mut md = String::from(
        "# Gold set v1: optional spot-check\n\nTwenty items drawn at random. For each, mark whether the question is fair and the gold passage answers it. The share you reject estimates the automatic review's error rate; with 20 items, 0 rejections bounds it at 16% (Wilson 95%).\n",
    );
    for &i in idx.iter().take(20) {
        let v = &items[i];
        md.push_str(&format!(
            "\n## {} ({})\n\n**Q:** {}\n\n**Evidence:** {}\n\n",
            v["qid"].as_str().unwrap_or(""),
            v["stratum"].as_str().unwrap_or(""),
            v["question"].as_str().unwrap_or(""),
            v["evidence"]
                .as_array()
                .map(|a| a
                    .iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(" / "))
                .unwrap_or_default()
        ));
        if let Some(src) = v["source_chunk_id"].as_str().and_then(|id| store.row(id)) {
            let t: String = store.chunks[src].text.chars().take(900).collect();
            md.push_str(&format!(
                "<details><summary>{}</summary>\n\n{}\n\n</details>\n\n",
                store.chunks[src].chunk_id,
                t.replace('\n', "  \n")
            ));
        }
        md.push_str("- [ ] fair question, passage answers it\n- [ ] reject: \n");
    }
    std::fs::write(path, md)?;
    Ok(())
}
