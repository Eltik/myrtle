//! Scene-scoped passages: the scenes of a named event in which a named character speaks, and excerpts of a
//! character's own scenes.

use std::collections::HashMap;

use anyhow::{Context, Result};
use trevor::search::runtime::{Runtime, RuntimeArgs};
use trevor::tools::{Tools, contains_words, norm};

use crate::chrono::proper_name;
use crate::detect::asks_scene_with;
use crate::kinds::{RECORD_KINDS, is_generated};
use crate::people::people_named;

/// Scenes of the event a question names where the character it names speaks or is named (the scoped scenes, the
/// default; `--no-scoped-scenes`). A question can describe a line in words the scene never uses ("the npc that mon3tr
/// suggests assassinating in masses travels" for "Mon3tr: (I can take her out. Nobody would notice.)"), and the
/// whole-corpus search then never ranks it; the event and the character narrow the search to a few scenes. The
/// event comes from `Tools::event_in_question`, the characters from `Tools::event_characters` (appearance index and identity
/// links) kept when the corpus writes the name capitalized (`proper_name`); the candidates are the event's chunks whose
/// speakers or text hold one of their forms, ranked by the question with dense and BM25 fused by reciprocal rank (k 60),
/// and the best are kept within `SCOPED_TOKENS`, at most `SCOPED_K`. Returns (event name, the characters, rows).
pub(crate) fn scoped_scenes(rt: &mut Runtime, tools: &Tools, q: &str, max_tokens: u32) -> Option<(String, Vec<String>, Vec<usize>)> {
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
    if trevor::env::debug_on("TREVOR_SCOPED_DEBUG") {
        eprintln!(
            "scoped {gid}: {} candidates, {} tokens",
            cands.len(),
            cands.iter().map(|&r| rt.store.chunks[r].token_count).sum::<u32>()
        );
        for (i, &r) in ranked.iter().enumerate() {
            eprintln!("  {i} {}", rt.store.chunks[r].chunk_id);
        }
    }
    let mut rows = Vec::new();
    let mut used = 0u32;
    for r in ranked {
        let cost = rt.store.chunks[r].token_count;
        if rows.len() >= SCOPED_K || (!rows.is_empty() && used + cost > max_tokens) {
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

/// Trevor's summaries of the stories the scoped scenes come from (opt-in, `--scoped-summaries`), in the
/// scenes' rank order, each story once, within `SCOPED_SUMMARY_TOKENS` (a token is taken as 4 characters): (story id,
/// summary). A scene shows a moment without its setting; "In chapter 15, when Priestess separates the Doctor, Amiya, and
/// Kal'tsit from the elite operators, where does she take them?" got the scene where the elite operators find themselves
/// alone (`main_15_level_main_15-15_beg#0003`, ranked 8th of 130) but not the one on the deck (57th), and the summary
/// of their story says "encountered ... the Priestess on the deck of Rhodes Island".
pub(crate) fn scoped_summaries(rt: &Runtime, tools: &Tools, rows: &[usize]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut used = 0usize;
    for &r in rows {
        let sid = &rt.store.chunks[r].story_id;
        if out.iter().any(|(s, _)| s == sid) {
            continue;
        }
        let id = format!("story:{sid}");
        let Some(text) = tools.bank.iter().find(|b| b["id"].as_str() == Some(id.as_str()))
            .and_then(|b| b["answer"].as_str()) else { continue };
        let cost = text.len().div_ceil(4);
        if used + cost > SCOPED_SUMMARY_TOKENS as usize {
            break;
        }
        used += cost;
        out.push((sid.clone(), text.to_owned()));
    }
    out
}

/// One excerpt per scene: (row, text, estimated tokens).
pub(crate) type Excerpts = Vec<(usize, String, u32)>;

/// The lines `idx` of chunk `r` with `CHAR_WINDOW` lines either side, merged, gaps marked "...", and its token estimate
/// (the chunk's token count scaled by the kept share of its text).
pub(crate) fn excerpt(rt: &Runtime, r: usize, idx: &[usize]) -> (String, u32) {
    excerpt_window(rt, r, idx, CHAR_WINDOW)
}

/// `excerpt` with `window` lines kept either side of each line of `idx`.
pub(crate) fn excerpt_window(rt: &Runtime, r: usize, idx: &[usize], window: usize) -> (String, u32) {
    let c = &rt.store.chunks[r];
    let lines: Vec<&str> = c.text.lines().collect();
    let mut keep = vec![false; lines.len()];
    for &i in idx {
        let hi = (i + window).min(lines.len().saturating_sub(1));
        keep.iter_mut().take(hi + 1).skip(i.saturating_sub(window)).for_each(|k| *k = true);
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
    let tok = u32::try_from(u64::from(c.token_count) * text.len() as u64 / c.text.len().max(1) as u64)
        .unwrap_or(u32::MAX);
    (text, tok)
}

/// The one named person the question makes the subject of an act: of the people it names, those a said, hint or
/// reaction verb follows (`asks_scene_with`), exactly one; with their folded name forms of 3 characters or more.
/// With `one_person` (the default; `--scene-subject` lifts it, unmeasured) the question must name exactly one person:
/// "How does Mon3tr react when she wakes up after Kal'tsit dies" names two, and got no character scenes.
pub(crate) fn scene_subject(tools: &Tools, people: &[String], q: &str, extra: &[&str], one_person: bool)
                            -> Option<(String, Vec<String>)> {
    let cands: Vec<&String> = if one_person {
        match people { [k] => vec![k], _ => return None }
    } else {
        people.iter().collect()
    };
    let mut hits = cands.into_iter().filter_map(|k| {
        let forms: Vec<String> = person_forms(tools, k).iter().map(|x| norm(x))
            .filter(|x| x.chars().count() >= 3).collect();
        (!forms.is_empty() && !tools.names_group_topic(k) && asks_scene_with(q, &forms, extra))
            .then(|| (k.clone(), forms))
    });
    let first = hits.next()?;
    hits.next().is_none().then_some(first)
}

/// Character-scoped scenes (the default; `--no-character-scenes`): a question that names no event, names one character
/// (`people_named`) and makes them the subject of a said or done act (`asks_scene`) gets excerpts of the scenes where
/// that character speaks, placed first, on top of the retrieved passages' whole budget (cutting that budget drops
/// passages other questions need). The question may describe the act in words the line never uses ("During which event
/// Mon3tr suggested assassinating a child" for "Mon3tr: (I can take her out. Nobody would notice.)"), which neither the
/// whole-corpus search nor the bi-encoder over the character's lines ranks well, so the reranker (a cross-encoder) reads
/// the question with each of the character's lines and the line before it. Each picked line keeps `CHAR_WINDOW` lines
/// either side, at most `CHAR_LINES_PER_SCENE` lines a scene and `CHAR_K` scenes, within `CHAR_TOKENS`.
/// Returns (the character as the corpus writes them, excerpts best first).
pub(crate) fn character_scenes(rt: &mut Runtime, ra: &RuntimeArgs, tools: &Tools, q: &str, max_tokens: u32,
                               extra: &[&str], one_person: bool) -> Option<(String, Excerpts)> {
    if tools.event_in_question(q).is_some() || tools.appearance_question(q).is_some() {
        return None;
    }
    let people = people_named(rt, tools, q);
    let (key, forms) = scene_subject(tools, &people, q, extra, one_person)?;
    let key = &key;
    let shown = tools.operator_names().into_iter().chain(tools.speaker_names()).find(|x| norm(x) == *key)
        .map_or_else(|| key.clone(), str::to_owned);
    let skip = |g: &str| is_generated(g) || RECORD_KINDS.contains(&g) || g == "voice";
    let cands: Vec<usize> = rt.store.chunks.iter().enumerate()
        .filter(|(_, c)| !skip(&c.group_id) && c.speakers.iter().any(|s| forms.contains(&norm(s))))
        .map(|(i, _)| i).collect();
    if cands.is_empty() {
        return None;
    }
    let t0 = std::time::Instant::now();
    let units = ranked_lines(rt, ra, q, &cands, &forms)?;
    if let Some(target) = trevor::env::non_empty("TREVOR_SCOPED_DEBUG").filter(|v| v != "0").as_deref() {
        let mut by_line: Vec<usize> = Vec::new();
        for (r, _) in &units { if !by_line.contains(r) { by_line.push(*r); } }
        let rrf = rank_by_question(rt, q, &cands).unwrap_or_default();
        let pos = |v: &[usize]| v.iter().position(|&r| rt.store.chunks[r].chunk_id == target)
            .map_or(-1, |p| p as i64 + 1);
        eprintln!("charscope {shown}: {} scenes, {} lines, rerank {:.2} s; {target}: chunk rrf {}, reranked lines {}",
                  cands.len(), units.len(), t0.elapsed().as_secs_f64(), pos(&rrf), pos(&by_line));
    }
    Some((shown, pick_excerpts(rt, units, max_tokens)))
}

/// Excerpts of ranked lines: the best lines first, each with its window, merged per scene, at most
/// `CHAR_LINES_PER_SCENE` lines a scene and `CHAR_K` scenes, within `max_tokens`.
fn pick_excerpts(rt: &Runtime, units: Vec<(usize, usize)>, max_tokens: u32) -> Excerpts {
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
        if used + grow > max_tokens && !picked.is_empty() {
            break;
        }
        used += grow;
        match at { Some(p) => picked[p].1 = idx, None => picked.push((r, idx)) }
    }
    picked.into_iter().map(|(r, idx)| { let (t, n) = excerpt(rt, r, &idx); (r, t, n) }).collect()
}

/// Scoped excerpts (`--no-scoped-line-rank`): a question that names an event and asks what a character of it said or
/// did there, in a wh-question ("what is the name of the npc that mon3tr suggests assassinating in masses travels"),
/// gets excerpts of that character's best lines in the event by the reranker, as the character scenes do, instead of
/// whole scoped scenes ranked by the question. A whether-question ("Is the Black Knight mentioned in the Near Light
/// event the operator Blacknight") keeps the scoped scenes. Returns (event name, the character as the corpus writes
/// them, excerpts best first).
pub(crate) fn scoped_excerpts(rt: &mut Runtime, ra: &RuntimeArgs, tools: &Tools, q: &str, max_tokens: u32,
                              extra: &[&str]) -> Option<(String, String, Excerpts)> {
    if crate::theory::asks_whether(q) {
        return None;
    }
    let g = tools.event_in_question(q)?;
    let gid = g["groupId"].as_str()?.to_owned();
    let ename = g["name"].as_str().unwrap_or(&gid).to_owned();
    let chars: Vec<(String, Vec<String>)> = tools.event_characters(q, &gid, &ename).into_iter()
        .filter(|(name, _)| proper_name(rt, name)).collect();
    let [(name, f)] = chars.as_slice() else { return None };
    let forms: Vec<String> = f.iter().map(|x| norm(x)).filter(|x| x.chars().count() >= 3).collect();
    if forms.is_empty() || !asks_scene_with(q, &forms, extra) {
        return None;
    }
    let cands: Vec<usize> = rt.store.chunks.iter().enumerate()
        .filter(|(_, c)| c.group_id == gid && c.speakers.iter().any(|s| forms.contains(&norm(s))))
        .map(|(i, _)| i).collect();
    if cands.is_empty() {
        return None;
    }
    let units = ranked_lines(rt, ra, q, &cands, &forms)?;
    let out = pick_excerpts(rt, units, max_tokens);
    (!out.is_empty()).then(|| (ename, name.clone(), out))
}

/// The lines of `cands` that a form of `forms` speaks, each with the line before it as context, ranked by the reranker
/// (a cross-encoder reads the question and the line together, so "take her out" can match "assassinating"), best
/// first, ties by (row, line): (row, line index).
fn ranked_lines(rt: &mut Runtime, ra: &RuntimeArgs, q: &str, cands: &[usize], forms: &[String])
    -> Option<Vec<(usize, usize)>> {
    let speaks = |line: &str| line.split_once(':').is_some_and(|(s, _)| s.len() < 60 && forms.contains(&norm(s)));
    let mut units: Vec<(usize, usize)> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for &r in cands {
        let lines: Vec<&str> = rt.store.chunks[r].text.lines().collect();
        for (i, line) in lines.iter().enumerate().filter(|(_, l)| speaks(l)) {
            units.push((r, i));
            texts.push(if i > 0 { format!("{}\n{line}", lines[i - 1]) } else { (*line).to_owned() });
        }
    }
    ensure_reranker(rt, ra).ok()?;
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let sc = rt.reranker.as_ref()?.score(q, &refs).ok()?;
    let mut ord: Vec<usize> = (0..units.len()).collect();
    ord.sort_by(|&x, &y| sc[y].total_cmp(&sc[x]).then(units[x].cmp(&units[y])));
    Some(ord.into_iter().map(|o| units[o]).collect())
}

/// The line study (`--line-study`, measurement only): every line of the corpus outside the generated kinds
/// with the line before it is a unit; a stride sample of them is embedded (one text per call) and, per question, the
/// evidence line's rank among all units is estimated as the sample units scoring above it times the stride, by line dense
/// and by the reranker, next to the evidence chunk's exact dense, BM25 and fused (RRF k 60) ranks.
pub(crate) fn line_study(rt: &mut Runtime, ra: &RuntimeArgs, batch: &std::path::Path) -> Result<()> {
    let stride: usize = trevor::env::parse_or("TREVOR_LINE_STRIDE", 30);
    let rr_stride: usize = trevor::env::parse_or("TREVOR_LINE_RR_STRIDE", 300);
    let pool: usize = trevor::env::parse_or("TREVOR_LINE_POOL", 100);
    let mut units: Vec<(usize, usize)> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for (r, c) in rt.store.chunks.iter().enumerate().filter(|(_, c)| !is_generated(&c.group_id)) {
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
        let mut top: Vec<(usize, f64)> = fused
            .into_iter()
            .filter(|(r, _)| !is_generated(&rt.store.chunks[*r].group_id))
            .collect();
        top.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
        let pool_rows: Vec<usize> = top.into_iter().take(pool).map(|(r, _)| r).collect();
        let pu: Vec<usize> = (0..n).filter(|&u| pool_rows.contains(&units[u].0)).collect();
        let t1 = std::time::Instant::now();
        let pt: Vec<&str> = pu.iter().map(|&u| texts[u].as_str()).collect();
        let ps = rt.reranker.as_ref().context("reranker")?.score(&q, &pt)?;
        let pool_rank = pu
            .iter()
            .position(|&u| u == ev)
            .map_or_else(|| "out".to_owned(), |p| (ps.iter().filter(|&&x| x > ps[p]).count() + 1).to_string());
        println!(
            "{qid}\t{dense_est}\t{rr_est}\t{}\t{}\t{fused_rank}\t{pool_rank}/{}\t{:.2}",
            pos(&d),
            pos(&b),
            pu.len(),
            t1.elapsed().as_secs_f64()
        );
    }
    Ok(())
}

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
        let mut by: Vec<(usize, f32)> = cands
            .iter()
            .map(|&r| (r, d.row(r).iter().zip(&qv).map(|(x, y)| x * y).sum()))
            .collect();
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

/// The scoped scenes take up to this many tokens, two thirds of the default 9,000-token passage budget (the scope is what
/// the question names), at most `SCOPED_K` scenes; the retrieved passages then get `SCOPED_REST`. A smaller budget
/// (2,400 tokens, 4 scenes) can leave out the scene the question asks about.
pub(crate) const SCOPED_K: usize = 16;
pub(crate) const SCOPED_TOKENS: u32 = 6000;
pub(crate) const SCOPED_REST: u32 = 3000;
/// The story summaries of the scoped scenes take at most this many tokens (about 8 summaries of 200 tokens).
pub(crate) const SCOPED_SUMMARY_TOKENS: u32 = 1600;

/// Load the reranker into `rt` on first use: the scene paths rerank even when retrieval does not.
fn ensure_reranker(rt: &mut Runtime, ra: &RuntimeArgs) -> Result<()> {
    if rt.reranker.is_none() {
        rt.reranker = Some(trevor::search::rerank::Reranker::load(
            &ra.rerank_dir,
            &ra.rerank_onnx,
            ra.rerank_max_tokens,
            ra.threads
        )?);
    }
    Ok(())
}
