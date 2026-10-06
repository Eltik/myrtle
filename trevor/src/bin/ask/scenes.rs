//! Scene-scoped passages: the scenes of a named event in which a named character speaks, excerpts of a
//! character's own scenes, the line pool, and the composed evidence of identity questions.

use std::collections::HashMap;

use anyhow::{Context, Result};
use trevor::search::runtime::{Runtime, RuntimeArgs};
use trevor::tools::{Tools, contains_words, norm};

use crate::chrono::{Chrono, label_links, proper_name};
use crate::detect::asks_scene;
use crate::retrieval::{GENERATED_KINDS, RECORD_KINDS, Names, Pre, file_or_story_label, people_named};
use crate::terms::count_word;

/// Scenes of the event a question names where the character it names speaks or is named (the scoped scenes, default since
/// 2026-10-05 night; `--no-scoped-scenes`). Ian's "what is the name of the npc that mon3tr suggests assassinating in masses
/// travels" was declined although the line is in the corpus ("Mon3tr: (I can take her out. Nobody would notice.)",
/// act42side_08_beg#0000): the whole-corpus search never ranks it, because nothing in the scene says "assassinate". The
/// event comes from `Tools::event_in_question`, the characters from `Tools::event_characters` (appearance index and identity
/// links) kept when the corpus writes the name capitalized (`proper_name`); the candidates are the event's chunks whose
/// speakers or text hold one of their forms, ranked by the question with dense and BM25 fused by reciprocal rank (k 60),
/// and the best are kept within `SCOPED_TOKENS`, at most `SCOPED_K`. Returns (event name, the characters, rows).
pub(crate) fn scoped_scenes(rt: &mut Runtime, tools: &Tools, q: &str) -> Option<(String, Vec<String>, Vec<usize>)> {
    let g = tools.event_in_question(q)?;
    let gid = g["groupId"].as_str()?.to_owned();
    let ename = g["name"].as_str().unwrap_or(&gid).to_owned();
    let chars: Vec<(String, Vec<String>)> = tools.event_characters(q, &gid, &ename).into_iter()
        .filter(|(name, _)| proper_name(rt, name)).collect();
    if chars.is_empty() {
        return None;
    }
    let forms: Vec<String> = chars.iter().flat_map(|(_, f)| f.iter().map(|x| norm(x)))
        .filter(|x| x.chars().count() >= 3).collect();
    let cands: Vec<usize> = rt.store.chunks.iter().enumerate().filter(|(_, c)| c.group_id == gid).filter(|(_, c)| {
        c.speakers.iter().any(|s| forms.contains(&norm(s)))
            || { let t = norm(&c.text); forms.iter().any(|f| contains_words(&t, f)) }
    }).map(|(i, _)| i).collect();
    if cands.is_empty() {
        return None;
    }
    let ranked = rank_by_question(rt, q, &cands)?;
    if std::env::var("TREVOR_SCOPED_DEBUG").is_ok() {
        eprintln!("scoped {gid}: {} candidates, {} tokens", cands.len(), cands.iter().map(|&r| rt.store.chunks[r].token_count).sum::<u32>());
        for (i, &r) in ranked.iter().enumerate() {
            eprintln!("  {i} {}", rt.store.chunks[r].chunk_id);
        }
    }
    let mut rows = Vec::new();
    let mut used = 0u32;
    for r in ranked {
        let cost = rt.store.chunks[r].token_count;
        if rows.len() >= SCOPED_K || (!rows.is_empty() && used + cost > SCOPED_TOKENS) {
            break;
        }
        used += cost;
        rows.push(r);
    }
    Some((ename, chars.into_iter().map(|(n, _)| n).collect(), rows))
}

/// Every form of the character a `people_named` key names: the operator, speaker or identity-link name it folds from,
/// and the names its identity links give the same person.
pub(crate) fn person_forms(tools: &Tools, key: &str) -> Vec<String> {
    let mut forms: Vec<String> = tools.operator_names().into_iter().chain(tools.speaker_names())
        .chain(tools.identities.iter().flatten().map(String::as_str))
        .filter(|x| norm(x) == key).map(str::to_owned).collect();
    for ident in tools.identities.iter().filter(|i| i.iter().any(|x| norm(x) == key)) {
        forms.extend(ident.iter().cloned());
    }
    forms.sort();
    forms.dedup();
    forms
}

/// One excerpt per scene: (row, text, estimated tokens).
pub(crate) type Excerpts = Vec<(usize, String, u32)>;

/// The lines `idx` of chunk `r` with `CHAR_WINDOW` lines either side, merged, gaps marked "...", and its token estimate
/// (the chunk's token count scaled by the kept share of its text).
pub(crate) fn excerpt(rt: &Runtime, r: usize, idx: &[usize]) -> (String, u32) {
    let c = &rt.store.chunks[r];
    let lines: Vec<&str> = c.text.lines().collect();
    let mut keep = vec![false; lines.len()];
    for &i in idx {
        let hi = (i + CHAR_WINDOW).min(lines.len().saturating_sub(1));
        keep.iter_mut().take(hi + 1).skip(i.saturating_sub(CHAR_WINDOW)).for_each(|k| *k = true);
    }
    let mut parts: Vec<String> = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for (k, l) in lines.iter().enumerate() {
        if keep[k] {
            cur.push(l);
        } else if !cur.is_empty() {
            parts.push(cur.join("\n"));
            cur.clear();
        }
    }
    if !cur.is_empty() {
        parts.push(cur.join("\n"));
    }
    let text = parts.join("\n...\n");
    let tok = u32::try_from(u64::from(c.token_count) * text.len() as u64 / c.text.len().max(1) as u64).unwrap_or(u32::MAX);
    (text, tok)
}

/// Character-scoped scenes (default since 2026-10-06; `--no-character-scenes`): a question that names no event, names one
/// character (`people_named`) and makes them the subject of a said or done act (`asks_scene`) gets excerpts of the
/// scenes where that character speaks, placed first, on top of the retrieved passages' whole budget. Ian's "During which
/// event Mon3tr suggested assassinating a child" (ian12) was declined: nothing in the scene says "assassinate" ("Mon3tr:
/// (I can take her out. Nobody would notice.)", act42side_08_beg#0000), so the whole-corpus search ranks that chunk 6,619th
/// dense, 93rd BM25 and 195th fused of 16,050 in P3b. Among Mon3tr's 172 speaking scenes (421 lines) the chunk ranks 71st
/// by the scoped scenes' dense + BM25 fusion, 158th by chunk dense and 134th by line dense (the bi-encoder is not
/// paraphrase-tolerant here), but 1st when the reranker (a cross-encoder) reads the question with each of her lines and the
/// line before it, in 0.97 s. Each picked line keeps `CHAR_WINDOW` lines either side, at most `CHAR_LINES_PER_SCENE` lines
/// a scene and `CHAR_K` scenes, within `CHAR_TOKENS`. A first version took the scoped scenes' 6,000 tokens and cut the
/// retrieved passages to 3,000: it lost gold g0004 (the retrieved "I won't burn Rhodes Island, or my homework" fell out).
/// Returns (the character as the corpus writes them, excerpts best first).
pub(crate) fn character_scenes(rt: &mut Runtime, ra: &RuntimeArgs, tools: &Tools, q: &str) -> Option<(String, Excerpts)> {
    if tools.event_in_question(q).is_some() || tools.appearance_question(q).is_some() {
        return None;
    }
    let people = people_named(rt, tools, q);
    let [key] = people.as_slice() else { return None };
    let forms: Vec<String> = person_forms(tools, key).iter().map(|x| norm(x)).filter(|x| x.chars().count() >= 3).collect();
    if forms.is_empty() || tools.names_group_topic(key) || !asks_scene(q, &forms) {
        return None;
    }
    let shown = tools.operator_names().into_iter().chain(tools.speaker_names()).find(|x| norm(x) == *key)
        .map_or_else(|| key.clone(), str::to_owned);
    let skip = |g: &str| GENERATED_KINDS.contains(&g) || RECORD_KINDS.contains(&g) || g == "voice";
    let cands: Vec<usize> = rt.store.chunks.iter().enumerate()
        .filter(|(_, c)| !skip(&c.group_id) && c.speakers.iter().any(|s| forms.contains(&norm(s))))
        .map(|(i, _)| i).collect();
    if cands.is_empty() {
        return None;
    }
    // The units: each line the character speaks, with the line before it as context, scored by the reranker (a
    // cross-encoder reads the question and the line together, so "take her out" can match "assassinating").
    let speaks = |line: &str| line.split_once(':').is_some_and(|(s, _)| s.len() < 60 && forms.contains(&norm(s)));
    let mut units: Vec<(usize, usize)> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for &r in &cands {
        let lines: Vec<&str> = rt.store.chunks[r].text.lines().collect();
        for (i, line) in lines.iter().enumerate().filter(|(_, l)| speaks(l)) {
            units.push((r, i));
            texts.push(if i > 0 { format!("{}\n{line}", lines[i - 1]) } else { (*line).to_owned() });
        }
    }
    ensure_reranker(rt, ra).ok()?;
    let t0 = std::time::Instant::now();
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let sc = rt.reranker.as_ref()?.score(q, &refs).ok()?;
    let mut ord: Vec<usize> = (0..units.len()).collect();
    ord.sort_by(|&x, &y| sc[y].total_cmp(&sc[x]).then(units[x].cmp(&units[y])));
    let units: Vec<(usize, usize)> = ord.into_iter().map(|o| units[o]).collect();
    if let Some(target) = std::env::var("TREVOR_SCOPED_DEBUG").ok().as_deref() {
        let mut by_line: Vec<usize> = Vec::new();
        for (r, _) in &units { if !by_line.contains(r) { by_line.push(*r); } }
        let rrf = rank_by_question(rt, q, &cands).unwrap_or_default();
        let pos = |v: &[usize]| v.iter().position(|&r| rt.store.chunks[r].chunk_id == target).map_or(-1, |p| p as i64 + 1);
        eprintln!("charscope {shown}: {} scenes, {} lines, rerank {:.2} s; {target}: chunk rrf {}, reranked lines {}",
                  cands.len(), units.len(), t0.elapsed().as_secs_f64(), pos(&rrf), pos(&by_line));
    }
    // Excerpts: the best lines first, each with its window, merged per scene, within `CHAR_TOKENS`.
    let mut picked: Vec<(usize, Vec<usize>)> = Vec::new();
    let mut used = 0u32;
    for (r, i) in units {
        let at = picked.iter().position(|(x, _)| *x == r);
        let mut idx = at.map_or_else(Vec::new, |p| picked[p].1.clone());
        if idx.len() >= CHAR_LINES_PER_SCENE || (at.is_none() && picked.len() >= CHAR_K) {
            continue;
        }
        let before = if idx.is_empty() { 0 } else { excerpt(rt, r, &idx).1 };
        idx.push(i);
        let grow = excerpt(rt, r, &idx).1.saturating_sub(before);
        if used + grow > CHAR_TOKENS && !picked.is_empty() {
            break;
        }
        used += grow;
        match at { Some(p) => picked[p].1 = idx, None => picked.push((r, idx)) }
    }
    let out = picked.into_iter().map(|(r, idx)| { let (t, n) = excerpt(rt, r, &idx); (r, t, n) }).collect();
    Some((shown, out))
}

/// The line study of Ian's item 1 (`--line-study`, measurement only): every line of the corpus outside the generated kinds
/// with the line before it is a unit; a stride sample of them is embedded (one text per call) and, per question, the
/// evidence line's rank among all units is estimated as the sample units scoring above it times the stride, by line dense
/// and by the reranker, next to the evidence chunk's exact dense, BM25 and fused (RRF k 60) ranks.
pub(crate) fn line_study(rt: &mut Runtime, ra: &RuntimeArgs, batch: &std::path::Path) -> Result<()> {
    let env = |k: &str, d: usize| std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d);
    let (stride, rr_stride, pool) = (env("TREVOR_LINE_STRIDE", 30), env("TREVOR_LINE_RR_STRIDE", 300), env("TREVOR_LINE_POOL", 100));
    let mut units: Vec<(usize, usize)> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for (r, c) in rt.store.chunks.iter().enumerate().filter(|(_, c)| !GENERATED_KINDS.contains(&c.group_id.as_str())) {
        let lines: Vec<&str> = c.text.lines().collect();
        for (i, line) in lines.iter().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
            units.push((r, i));
            texts.push(if i > 0 { format!("{}\n{line}", lines[i - 1]) } else { (*line).to_owned() });
        }
    }
    let n = units.len();
    let sample: Vec<usize> = (0..n).step_by(stride).collect();
    let t0 = std::time::Instant::now();
    let mut sv: Vec<Vec<f32>> = Vec::with_capacity(sample.len());
    for &u in &sample {
        sv.push(rt.embed_query(&texts[u])?);
    }
    eprintln!("line study: {n} units, {} sampled, embedded in {:.1} s ({:.1} units/s)", sample.len(),
              t0.elapsed().as_secs_f64(), sample.len() as f64 / t0.elapsed().as_secs_f64());
    ensure_reranker(rt, ra)?;
    let rr_sample: Vec<usize> = (0..n).step_by(rr_stride).collect();
    let fold = |x: &str| norm(&x.replace(['\u{2019}', '\u{2018}'], "'").replace(['\u{201c}', '\u{201d}'], "\""));
    println!("qid\tline_dense_est\tline_rerank_est\tchunk_dense\tchunk_bm25\tchunk_fused\tunits={n}");
    for l in std::fs::read_to_string(batch)?.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(l)?;
        let s = |k: &str| v[k].as_str().unwrap_or_default().to_owned();
        let (qid, q, chunk, quote) = (s("qid"), s("question"), s("chunk"), fold(&s("quote")));
        let key: String = quote.chars().take(40).collect();
        let Some(ev) = units.iter().position(|&(r, i)| rt.store.chunks[r].chunk_id == chunk
            && fold(rt.store.chunks[r].text.lines().nth(i).unwrap_or_default()).contains(&key)) else {
            println!("{qid}\tnot-found");
            continue;
        };
        let qv = rt.embed_query(&q)?;
        let dot = |x: &[f32]| x.iter().zip(&qv).map(|(a, b)| a * b).sum::<f32>();
        let es = dot(&rt.embed_query(&texts[ev])?);
        let dense_est = sv.iter().filter(|x| dot(x) > es).count() * stride + 1;
        let mut rr_texts: Vec<&str> = vec![texts[ev].as_str()];
        rr_texts.extend(rr_sample.iter().map(|&u| texts[u].as_str()));
        let sc = rt.reranker.as_ref().context("reranker")?.score(&q, &rr_texts)?;
        let rr_est = sc[1..].iter().filter(|&&x| x > sc[0]).count() * rr_stride + 1;
        let row = units[ev].0;
        let d = rt.dense.as_ref().context("dense")?.search(&qv, rt.store.len())?;
        let b = rt.bm25.as_ref().context("bm25")?.search(&q, rt.store.len(), &rt.store)?;
        let pos = |h: &[(usize, f32)]| h.iter().position(|&(r, _)| r == row).map_or(0, |p| p + 1);
        let mut fused: HashMap<usize, f64> = HashMap::new();
        for h in [&d, &b] {
            for (k, (r, _)) in h.iter().enumerate() {
                *fused.entry(*r).or_default() += 1.0 / (61.0 + k as f64);
            }
        }
        let fr = fused.get(&row).copied().unwrap_or(0.0);
        let fused_rank = fused.values().filter(|&&x| x > fr).count() + 1;
        // The pool variant: the lines of the fused top `pool` chunks, reranked (no line index).
        let mut top: Vec<(usize, f64)> = fused.into_iter().filter(|(r, _)| !GENERATED_KINDS.contains(&rt.store.chunks[*r].group_id.as_str())).collect();
        top.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
        let pool_rows: Vec<usize> = top.into_iter().take(pool).map(|(r, _)| r).collect();
        let pu: Vec<usize> = (0..n).filter(|&u| pool_rows.contains(&units[u].0)).collect();
        let t1 = std::time::Instant::now();
        let pt: Vec<&str> = pu.iter().map(|&u| texts[u].as_str()).collect();
        let ps = rt.reranker.as_ref().context("reranker")?.score(&q, &pt)?;
        let pool_rank = pu.iter().position(|&u| u == ev).map_or_else(|| "out".to_owned(), |p| (ps.iter().filter(|&&x| x > ps[p]).count() + 1).to_string());
        println!("{qid}\t{dense_est}\t{rr_est}\t{}\t{}\t{fused_rank}\t{pool_rank}/{}\t{:.2}", pos(&d), pos(&b), pu.len(), t1.elapsed().as_secs_f64());
    }
    Ok(())
}

/// The line pool (`--line-pool`, opt-in since 2026-10-06): the hybrid (dense + BM25, RRF k 60) top `LINE_POOL_CHUNKS`
/// chunks for `search` outside the generated kinds, each of their lines with the line before it, scored by the reranker
/// with the question. On 84 single-line gold questions the gold line ranks in the reranked top 4 of about 1,430 pool
/// lines for 7 of the 8 that are answered wrong without the gold chunk among their passages (`--line-study`, design/
/// trevor-questions.md section 12), where a whole-corpus line dense index (221,517 lines, a 71-minute build, 170 MB as
/// int8) ranks the gold line in its top 200 for 68 of 87 against 83 of 87 for the chunk fusion, and Chiave's line (ian6)
/// about 19,400th. Answered 2026-10-06: gold correct 4 to 10 of the 14 changed of 30 (+7, -1: g0034), Qwen pairwise 6
/// wins, 0 losses; Ian's 33: fires on 8, pairwise 0 wins, 2 losses (ian17, ian30), so it is opt-in. Fires when the best line is outside `rows` (or always with `all`); then the best lines outside `rows`, at most
/// `CHAR_LINES_PER_SCENE` a chunk, as excerpts with `CHAR_WINDOW` lines either side within `LINE_POOL_TOKENS`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn line_pool(rt: &mut Runtime, ra: &RuntimeArgs, names: &HashMap<String, Names>, q: &str, search: &str, rows: &[usize], all: bool,
             min: Option<f32>) -> Result<Vec<Pre>> {
    if rt.dense.is_none() || rt.bm25.is_none() {
        return Ok(Vec::new());
    }
    ensure_reranker(rt, ra)?;
    let qv = rt.embed_query(search)?;
    let d = rt.dense.as_ref().context("dense")?.search(&qv, rt.store.len())?;
    let b = rt.bm25.as_ref().context("bm25")?.search(search, rt.store.len(), &rt.store)?;
    let mut fused: HashMap<usize, f64> = HashMap::new();
    for h in [&d, &b] {
        for (k, (r, _)) in h.iter().enumerate() {
            *fused.entry(*r).or_default() += 1.0 / (61.0 + k as f64);
        }
    }
    let mut top: Vec<(usize, f64)> = fused.into_iter().filter(|(r, _)| !GENERATED_KINDS.contains(&rt.store.chunks[*r].group_id.as_str())).collect();
    top.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
    let mut units: Vec<(usize, usize)> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for (r, _) in top.into_iter().take(LINE_POOL_CHUNKS) {
        let lines: Vec<&str> = rt.store.chunks[r].text.lines().collect();
        for (i, line) in lines.iter().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
            units.push((r, i));
            texts.push(if i > 0 { format!("{}\n{line}", lines[i - 1]) } else { (*line).to_owned() });
        }
    }
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let sc = rt.reranker.as_ref().context("reranker")?.score(q, &refs)?;
    let mut ord: Vec<usize> = (0..units.len()).collect();
    ord.sort_by(|&x, &y| sc[y].total_cmp(&sc[x]).then(units[x].cmp(&units[y])));
    let gate = ord.first().is_some_and(|&o| !rows.contains(&units[o].0));
    let best_out = ord.iter().find(|&&o| !rows.contains(&units[o].0)).map_or(f32::NEG_INFINITY, |&o| sc[o]);
    if std::env::var("TREVOR_LINE_POOL_DEBUG").is_ok() {
        eprintln!("linepool\tgate={gate}\tbest_out={best_out:.4}\t{q}");
    }
    if (!gate && !all) || min.is_some_and(|m| best_out < m) {
        return Ok(Vec::new());
    }
    let mut picked: Vec<(usize, Vec<usize>)> = Vec::new();
    let mut used = 0u32;
    for (r, i) in ord.into_iter().map(|o| units[o]).filter(|(r, _)| !rows.contains(r)) {
        let at = picked.iter().position(|(x, _)| *x == r);
        let mut idx = at.map_or_else(Vec::new, |p| picked[p].1.clone());
        if idx.len() >= CHAR_LINES_PER_SCENE {
            continue;
        }
        let before = if idx.is_empty() { 0 } else { excerpt(rt, r, &idx).1 };
        idx.push(i);
        let grow = excerpt(rt, r, &idx).1.saturating_sub(before);
        if used + grow > LINE_POOL_TOKENS && !picked.is_empty() {
            break;
        }
        used += grow;
        match at { Some(p) => picked[p].1 = idx, None => picked.push((r, idx)) }
        if picked.iter().map(|(_, v)| v.len()).sum::<usize>() >= LINE_POOL_LINES {
            break;
        }
    }
    Ok(picked.into_iter().map(|(r, idx)| {
        let c = &rt.store.chunks[r];
        let label = file_or_story_label(names, c);
        Pre::new(c.chunk_id.clone(), format!("{label}; an excerpt around a line that matches the question"), excerpt(rt, r, &idx).0).after()
    }).collect())
}

/// The line pool: chunks whose lines are reranked, lines added, and their token budget on top of the passages'.
pub(crate) const LINE_POOL_CHUNKS: usize = 100;
pub(crate) const LINE_POOL_LINES: usize = 4;
pub(crate) const LINE_POOL_TOKENS: u32 = 1500;

/// Character-scoped excerpts: lines kept either side of a picked line, picked lines per scene, scenes, and tokens (on top
/// of the retrieved passages' budget; at 3,000 the 8 questions it fires on get 2 to 14 scenes).
pub(crate) const CHAR_WINDOW: usize = 2;
pub(crate) const CHAR_LINES_PER_SCENE: usize = 2;
pub(crate) const CHAR_K: usize = 24;
pub(crate) const CHAR_TOKENS: u32 = 3000;

/// `cands` ranked by the question: dense (dot product with the query vector) and BM25 (its whole-corpus ranking, kept to
/// `cands`) fused by reciprocal rank (k 60), ties by row. Used by the scoped scenes and the evidence composition.
pub(crate) fn rank_by_question(rt: &mut Runtime, q: &str, cands: &[usize]) -> Option<Vec<usize>> {
    let mut score: HashMap<usize, f64> = cands.iter().map(|&r| (r, 0.0)).collect();
    if rt.dense.is_some() {
        let qv = rt.embed_query(q).ok()?;
        let d = rt.dense.as_ref()?;
        let mut by: Vec<(usize, f32)> = cands.iter().map(|&r| (r, d.row(r).iter().zip(&qv).map(|(x, y)| x * y).sum())).collect();
        by.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
        for (rank, (r, _)) in by.iter().enumerate() {
            *score.entry(*r).or_default() += 1.0 / (60.0 + rank as f64 + 1.0);
        }
    }
    if let Some(b) = rt.bm25.as_ref() {
        let hits = b.search(q, rt.store.len(), &rt.store).ok()?;
        let inside: Vec<usize> = hits.iter().map(|(r, _)| *r).filter(|r| score.contains_key(r)).collect();
        for (rank, r) in inside.into_iter().enumerate() {
            *score.entry(r).or_default() += 1.0 / (60.0 + rank as f64 + 1.0);
        }
    }
    let mut ranked: Vec<(usize, f64)> = score.into_iter().collect();
    ranked.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
    Some(ranked.into_iter().map(|(r, _)| r).collect())
}

/// Evidence composition for a who/which question with a description (`--compose-evidence`, opt-in since 2026-10-05 night):
/// when the answer to a question whose answer category the check reads (`Tools::answer_constraint`) still declines,
/// the stated identity links whose source is among its passages (`label_links`, at most 2) become candidates, each as a
/// note ("Bluishsilver and Mabel are the same person ...", with the quote) followed by the scenes naming either name
/// best for the question (`rank_by_question` over P4x; generated kinds left out), within 3,000 tokens per link. ian24
/// ("Which Rhodes Island Operator (not playable unit) had visited Iris' castle of dreams?") declined with Iris's file,
/// which says Mabel was the deceased Operator Bluishsilver, as passage [1]: the interlude where Iris looks for Mabel Grimm
/// never reached the prompt, and the model does not join two passages on its own. Returns (id, label, text) passages.
pub(crate) fn compose_evidence(rt: &mut Runtime, chrono: &Chrono, names: &HashMap<String, Names>, q: &str, passages: &[String]) -> Vec<(String, String, String)> {
    let rows: Vec<usize> = passages.iter().filter_map(|id| rt.store.row(id)).collect();
    let links = label_links(rt, &rows, chrono);
    let mut out = Vec::new();
    for (at, x, y, rel, quote) in links.iter().take(2) {
        out.push((format!("identity:{x}={y}"), format!("Trevor's identity note (from the game text, {at})"),
                  format!("{x} and {y} are the same person: {y} is the {rel} of {x} (\"{quote}\").")));
        let cands: Vec<usize> = rt.store.chunks.iter().enumerate()
            .filter(|(_, c)| !GENERATED_KINDS.contains(&c.group_id.as_str())
                && (count_word(&c.text, x) > 0 || count_word(&c.text, y) > 0))
            .map(|(i, _)| i).collect();
        let mut used = 0u32;
        for r in rank_by_question(rt, q, &cands).unwrap_or_default() {
            let c = &rt.store.chunks[r];
            // The scoped scenes' budget, shared by the links (2 at most): with 3 scenes ian24's ask lacked Iris's voice
            // line "I'm looking for a girl named Mabel ... twenty years ago" and declined.
            if used + c.token_count > SCOPED_TOKENS / 2 && used > 0 {
                break;
            }
            used += c.token_count;
            let (who, other) = if count_word(&c.text, x) > 0 { (x, y) } else { (y, x) };
            let label = file_or_story_label(names, c);
            out.push((c.chunk_id.clone(), format!("{label}; a scene that names {who}, the same person as {other}"), c.text.clone()));
        }
    }
    out
}

/// The scoped scenes take up to this many tokens, two thirds of the default 9,000-token passage budget (the scope is what
/// the question names), at most `SCOPED_K` scenes; the retrieved passages then get `SCOPED_REST`. With 2,400 tokens
/// (4 scenes) Ian's Mon3tr scene ranked 9th of the 20 candidates (8,878 tokens) and was left out.
pub(crate) const SCOPED_K: usize = 16;
pub(crate) const SCOPED_TOKENS: u32 = 6000;
pub(crate) const SCOPED_REST: u32 = 3000;

/// Load the reranker into `rt` on first use: the scene and line-pool paths rerank even when retrieval does not.
fn ensure_reranker(rt: &mut Runtime, ra: &RuntimeArgs) -> Result<()> {
    if rt.reranker.is_none() {
        rt.reranker = Some(trevor::search::rerank::Reranker::load(&ra.rerank_dir, &ra.rerank_onnx, ra.rerank_max_tokens, ra.threads)?);
    }
    Ok(())
}
