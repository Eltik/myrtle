//! Deaths evidence for alive-or-dead questions (`--no-death-evidence`): a question asking whether one character is
//! alive or dead ("Is Mandragora alive or dead?", "Did Mandragora die?", "Is W still alive?") whose character has rows in
//! Trevor's deaths table (`artifacts/entities/deaths.jsonl`, `scripts/deaths.py`) gets the story lines those rows quote,
//! each with two lines either side, as the first passages, labelled as what the table found. The rows are story
//! chunks, so under a spoiler horizon only those released at or before it are kept (`answer_from`), and the chunk is
//! not repeated among the retrieved passages.
//!
//! Why: retrieval for "Is Mandragora alive or dead?" (E2B, mid) brought Stormwatch's "Verse in the Ashes", where
//! Mandragora says "Hah, she's dead" of an executed traitor, and the answer took that line for Mandragora's death. The
//! deaths table holds the line that settles it (The Brigadier, What the Firelight Casts: "Mandragora is dead.").

use serde_json::Value;
use trevor::search::runtime::Runtime;
use trevor::tools::{Tools, norm};

use crate::detect::has;

/// The most rows given as evidence: the first row that shows the death, then the latest report.
pub(crate) const DEATH_ROWS: usize = 2;

/// Lines kept either side of the quoted line.
const DEATH_WINDOW: usize = 2;

/// The character an alive-or-dead question asks about: a death or survival word, a question shape that asks whether
/// (not who, which or how many), and exactly one named character of the deaths table as whole words (the longest name
/// when one holds another).
pub(crate) fn alive_dead_subject(q: &str, tools: &Tools) -> Option<String> {
    let ql = format!(" {} ", q.to_lowercase().replace(['?', ',', '.', '!', '"'], " "));
    let fate = has(&ql, &[" alive ", " dead ", " die ", " died ", " dies ", " death ", " killed ", " survive",
                          " deceased ", " passed away ", " still living ", " still around "]);
    let whether = has(&ql, &[" is ", " was ", " did ", " does ", " has ", " are ", " were ", " will ", " or "]);
    let listing = has(&ql, &[" who ", " which ", " what ", " how many ", " list ", " every ", " all the ",
                             " anyone ", " characters ", " operators "]);
    if !fate || !whether || listing {
        return None;
    }
    let names = tools.death_names();
    // A name counts as written ("Was Crownslayer's teacher killed" does not name "Teacher"), or in any case when the
    // whole question is lowercase.
    let lower = !q.chars().any(char::is_uppercase);
    let mut hits: Vec<&String> = names.iter().filter(|n| {
        let k = norm(n);
        k.chars().count() >= 3 && format!(" {} ", norm(q)).contains(&format!(" {k} "))
            && (lower || q.match_indices(n.as_str()).any(|(i, _)| {
                q[..i].chars().last().is_none_or(|c| !c.is_alphanumeric())
                    && q[i + n.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric())
            }))
    }).collect();
    hits.sort_by_key(|n| std::cmp::Reverse(n.len()));
    let first = (*hits.first()?).clone();
    // Another name counts unless it is only part of the first one ("Talulah" in "Talulah Artorius").
    let rest = format!(" {} ", norm(q)).replace(&format!(" {} ", norm(&first)), " ");
    let others = hits.iter().skip(1).filter(|n| rest.contains(&format!(" {} ", norm(n)))).count();
    (others == 0).then_some(first)
}

/// The deaths-table rows given for `who`, at most `DEATH_ROWS`: the earliest row that shows the death ("dies"), then
/// the latest row, in release order.
pub(crate) fn death_rows<'a>(tools: &'a Tools, who: &str) -> Vec<&'a Value> {
    let Some(events) = tools.deaths.as_ref() else { return Vec::new() };
    let mut mine: Vec<&Value> = events.iter().filter(|e| e["character"].as_str() == Some(who)).collect();
    mine.sort_by_key(|e| e["releaseTime"].as_i64().unwrap_or(0));
    let mut out: Vec<&Value> = Vec::new();
    if let Some(d) = mine.iter().find(|e| e["kind"].as_str() == Some("dies")) {
        out.push(d);
    }
    if let Some(l) = mine.last() {
        if !out.iter().any(|o| o["chunkId"] == l["chunkId"]) {
            out.push(l);
        }
    }
    out.sort_by_key(|e| e["releaseTime"].as_i64().unwrap_or(0));
    out.truncate(DEATH_ROWS);
    out
}

/// One evidence passage per row: (chunk id, label tail, excerpt text, tokens), for the rows whose chunk is in `rt` and
/// holds the quoted line.
pub(crate) fn death_evidence(rt: &Runtime, tools: &Tools, who: &str) -> Vec<(String, String, String, u32)> {
    death_rows(tools, who).into_iter().filter_map(|e| {
        let id = e["chunkId"].as_str()?;
        let r = rt.store.row(id)?;
        let quote = e["quote"].as_str()?.trim();
        let key: String = quote.chars().take(60).collect();
        let li = rt.store.chunks[r].text.lines().position(|l| l.contains(key.as_str()))?;
        let (text, tok) = crate::scenes::excerpt_window(rt, r, &[li], DEATH_WINDOW);
        let what = if e["kind"].as_str() == Some("dies") { format!("{who}'s death") }
            else { format!("a report that {who} is dead") };
        Some((id.to_owned(), format!("the line where Trevor's deaths table finds {what}"), text, tok))
    }).collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn tools() -> Tools {
        Tools { deaths: Some(vec![
            json!({"character": "Mandragora", "kind": "dead", "generic": false, "releaseTime": 30, "chunkId": "c#3",
                   "quote": "Mandragora is dead."}),
            json!({"character": "Mandragora", "kind": "dies", "generic": false, "releaseTime": 20, "chunkId": "c#2",
                   "quote": "x"}),
            json!({"character": "Mandragora", "kind": "dies", "generic": false, "releaseTime": 25, "chunkId": "c#4",
                   "quote": "y"}),
            json!({"character": "Mandra", "kind": "dies", "generic": false, "releaseTime": 1, "chunkId": "c#1",
                   "quote": "z"}),
            json!({"character": "Teacher", "kind": "dead", "generic": false, "releaseTime": 1, "chunkId": "c#6",
                   "quote": "z"}),
            json!({"character": "W", "kind": "dead", "generic": false, "releaseTime": 1, "chunkId": "c#5",
                   "quote": "z"}),
        ]), ..Tools::default() }
    }

    #[test]
    fn alive_or_dead_questions_name_one_character() {
        let t = tools();
        for q in ["Is Mandragora alive or dead?", "Did Mandragora die?", "Is Mandragora still alive?",
                  "mandragora dead or alive", "Does Mandragora survive Episode 10?"] {
            assert_eq!(alive_dead_subject(q, &t).as_deref(), Some("Mandragora"), "{q}");
        }
        for q in ["Who killed Mandragora?", "Which characters died in Episode 10?", "Tell me about Mandragora",
                  "What does Arturia play during the death of Mandragora?", "Was the teacher of W killed?",
                  "Is Mandragora related to Mandra or W? Is W dead?"] {
            assert_eq!(alive_dead_subject(q, &t), None, "{q}");
        }
    }

    #[test]
    fn rows_are_the_first_death_and_the_latest_report() {
        let t = tools();
        let ids: Vec<&str> = death_rows(&t, "Mandragora").iter().filter_map(|e| e["chunkId"].as_str()).collect();
        assert_eq!(ids, vec!["c#2", "c#3"]);
        assert!(death_rows(&t, "Nobody").is_empty());
    }
}
