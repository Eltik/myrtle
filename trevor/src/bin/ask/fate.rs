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
use trevor::tools::{Tools, contains_words, norm};

use crate::detect::has;

/// The most rows given as evidence: the first row that shows the death, then the latest report.
pub(crate) const DEATH_ROWS: usize = 2;

/// Lines kept either side of the quoted line.
const DEATH_WINDOW: usize = 2;

/// The character an alive-or-dead question asks about: a death or survival word, a question shape that asks whether
/// (not who, which or how many, and not how, why, when or where as its first word: "How did Mandragora die?" takes the
/// death as given), and exactly one named character of the deaths table as whole words, a possessive included
/// ("Mandragora's"; the longest name when one holds another).
pub(crate) fn alive_dead_subject(q: &str, tools: &Tools) -> Option<String> {
    let first_word = q.split_whitespace().next().unwrap_or_default().to_lowercase();
    if matches!(first_word.as_str(), "how" | "why" | "when" | "where") {
        return None;
    }
    let ql = format!(" {} ", q.to_lowercase().replace(['?', ',', '.', '!', '"'], " "));
    let fate = has(
        &ql,
        &[
            " alive ",
            " dead ",
            " die ",
            " died ",
            " dies ",
            " death ",
            " killed ",
            " survive",
            " deceased ",
            " passed away ",
            " still living ",
            " still around ",
        ],
    );
    let whether = has(
        &ql,
        &[
            " is ", " was ", " did ", " does ", " has ", " are ", " were ", " will ", " or ",
        ],
    );
    let listing = has(
        &ql,
        &[
            " who ",
            " which ",
            " what ",
            " how many ",
            " list ",
            " every ",
            " all the ",
            " anyone ",
            " characters ",
            " operators ",
        ],
    );
    if !fate || !whether || listing {
        return None;
    }
    let names = tools.death_names();
    // A name counts as written ("Was Crownslayer's teacher killed" does not name "Teacher"), or in any case when the
    // whole question is lowercase.
    let lower = !q.chars().any(char::is_uppercase);
    // A possessive ("Mandragora's") is the name and "s" once `norm` drops the apostrophe, as in `appear.rs`.
    let (qn, qp) = (
        format!(" {} ", norm(q)),
        format!(" {} ", norm(&q.replace('\u{2019}', "'").replace("'s", " "))),
    );
    let mut hits: Vec<&String> = names
        .iter()
        .filter(|n| {
            let k = format!(" {} ", norm(n));
            k.chars().count() >= 5
                && (qn.contains(&k) || qp.contains(&k))
                && (lower
                    || q.match_indices(n.as_str()).any(|(i, _)| {
                        q[..i].chars().last().is_none_or(|c| !c.is_alphanumeric())
                            && q[i + n.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric())
                    }))
        })
        .collect();
    hits.sort_by_key(|n| std::cmp::Reverse(n.len()));
    let first = (*hits.first()?).clone();
    // Another name counts unless it is only part of the first one ("Talulah" in "Talulah Artorius").
    let f = format!(" {} ", norm(&first));
    let (rn, rp) = (qn.replace(&f, " "), qp.replace(&f, " "));
    let others = hits
        .iter()
        .skip(1)
        .filter(|n| {
            let k = format!(" {} ", norm(n));
            rn.contains(&k) || rp.contains(&k)
        })
        .count();
    (others == 0).then_some(first)
}

/// The deaths-table rows given for `who`, at most `DEATH_ROWS`: the earliest row that shows the death ("dies"), then
/// the latest row, in release order.
pub(crate) fn death_rows<'a>(tools: &'a Tools, who: &str) -> Vec<&'a Value> {
    let Some(events) = tools.deaths.as_ref() else {
        return Vec::new();
    };
    // A row without a quote has no line to show: its first line would stand in for it.
    let mut mine: Vec<&Value> = events
        .iter()
        .filter(|e| e["character"].as_str() == Some(who) && e["quote"].as_str().is_some_and(|q| !q.trim().is_empty()))
        .collect();
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
/// holds the quoted line. The label names the line ("the line where X dies", "a line reporting that X is dead");
/// with `v1` (`--death-label-v1`) it reads "the line where Trevor's deaths table finds ...", which the 12B copied into
/// an answer as "a report found by Closure" (2026-10-20).
pub(crate) fn death_evidence(rt: &Runtime, tools: &Tools, who: &str, v1: bool) -> Vec<(String, String, String, u32)> {
    death_rows(tools, who)
        .into_iter()
        .filter_map(|e| {
            let id = e["chunkId"].as_str()?;
            let r = rt.store.row(id)?;
            let quote = e["quote"].as_str()?.trim();
            let key: String = quote.chars().take(60).collect();
            let li = rt.store.chunks[r].text.lines().position(|l| l.contains(key.as_str()))?;
            let (text, tok) = crate::scenes::excerpt_window(rt, r, &[li], DEATH_WINDOW);
            let dies = e["kind"].as_str() == Some("dies");
            let label = if v1 {
                let what = if dies {
                    format!("{who}'s death")
                } else {
                    format!("a report that {who} is dead")
                };
                format!("the line where Trevor's deaths table finds {what}")
            } else if dies {
                format!("the line where {who} dies")
            } else {
                format!("a line reporting that {who} is dead")
            };
            Some((id.to_owned(), label, text, tok))
        })
        .collect()
}

/// The words a disappearance is told in, added to the search query (never the question the model reads) of a
/// vanished-fate question (`--no-vanish-rule`): Episode 15 says Kal'tsit "has disappeared", not that she died.
pub(crate) const VANISH_TERMS: &str = "disappeared vanished lost";

/// The death words that, after a name, make it the one whose death a question naming several people asks ("after
/// Kal'tsit dies"); "killed" after a name can be the killer's act, so it counts only before one ("killed Kal'tsit").
const DEATH_WORDS: [&str; 7] = ["die", "dies", "died", "dying", "death", "dead", "deceased"];

/// A death-worded question whose character has no death scene in the deaths table (`--no-vanish-rule`).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Vanished {
    /// The character as the tables write them.
    pub(crate) who: String,
    /// The deaths-table name of the character's later reports ("dead" rows), when there are any.
    pub(crate) reports: Option<String>,
}

/// The vanished fate: a death-worded question (`is_death_question`, whole words, so "die," counts) about one
/// character (not only as another noun's possessive, "Crownslayer's teacher", nor beside a relation's death, "her
/// mom's stuff ... when she passed away", `names_relation`), `people` being the folded keys of
/// `people_named`, for whom the deaths table has no row that shows the death ("dies") under any of the character's
/// names. With one person named, that person; with several, the one a death word follows or that "death of" or
/// "killed" precedes ("after Kal'tsit dies"), unless "and", "or" or "nor" joins the name to another ("Did Kal'tsit or
/// Amiya die?"). A character with only reports of the death ("dead" rows: Kal'tsit's "I know she's dead, obviously."
/// in Episode 17) keeps those reports as evidence. Group topics are not characters.
pub(crate) fn vanished_subject(q: &str, people: &[String], tools: &Tools) -> Option<Vanished> {
    if !crate::detect::is_death_question(q) {
        return None;
    }
    // The key itself is a form too, for a name `person_forms` finds in no table.
    let forms_of = |k: &str| -> Vec<String> {
        let mut f: Vec<String> = crate::scenes::person_forms(tools, k)
            .iter()
            .map(|x| norm(x))
            .filter(|x| x.chars().count() >= 3)
            .collect();
        if !f.iter().any(|x| x == k) {
            f.push(k.to_owned());
        }
        f
    };
    let (qn, qp) = (norm(q), norm(&q.replace('\u{2019}', "'").replace("'s", " ")));
    let key = match people {
        // A name written only as the possessive of another noun is not the one whose death is asked ("Was
        // Crownslayer's teacher killed"), unless a death word follows it ("Tin Man's death").
        [k] => {
            let forms = forms_of(k);
            if !forms.iter().any(|f| contains_words(&qn, f)) && !death_attached(&qp, &forms) {
                return None;
            }
            // A relation's death ("was Delphine hesitant to get her mom's stuff back when she passed away?") is not
            // the named character's, unless a death word attaches to the name ("Did Delphine die?").
            if names_relation(q) && !death_attached(&qn, &forms) && !death_attached(&qp, &forms) {
                return None;
            }
            k.clone()
        }
        [] => return None,
        _ => {
            let mut hit = people.iter().filter(|k| {
                let forms = forms_of(k);
                death_attached(&qn, &forms) || death_attached(&qp, &forms)
            });
            let first = hit.next()?;
            if hit.next().is_some() {
                return None;
            }
            first.clone()
        }
    };
    if tools.names_group_topic(&key) {
        return None;
    }
    let forms = forms_of(&key);
    let rows: Vec<&Value> = tools
        .deaths
        .iter()
        .flatten()
        .filter(|e| {
            e["generic"].as_bool() != Some(true) && e["character"].as_str().is_some_and(|c| forms.contains(&norm(c)))
        })
        .collect();
    if rows.iter().any(|e| e["kind"].as_str() == Some("dies")) {
        return None;
    }
    let reports = rows
        .iter()
        .filter(|e| e["quote"].as_str().is_some_and(|x| !x.trim().is_empty()))
        .max_by_key(|e| e["releaseTime"].as_i64().unwrap_or(0))
        .and_then(|e| e["character"].as_str())
        .map(str::to_owned);
    let who = tools
        .operator_names()
        .into_iter()
        .chain(tools.speaker_names())
        .chain(tools.identities.iter().flatten().map(String::as_str))
        .find(|x| norm(x) == key)
        .map_or_else(|| reports.clone().unwrap_or_else(|| key.clone()), str::to_owned);
    Some(Vanished { who, reports })
}

/// The relations whose death a question can ask about beside a named character: written as a possessive phrase ("her
/// mom", "Delphine's mother", "their parents"), they make the death another person's (`--no-vanish-rule`).
const RELATIONS: [&str; 26] = [
    "mother",
    "mothers",
    "mom",
    "moms",
    "mum",
    "mums",
    "father",
    "fathers",
    "dad",
    "dads",
    "parent",
    "parents",
    "sister",
    "sisters",
    "brother",
    "brothers",
    "son",
    "sons",
    "daughter",
    "daughters",
    "family",
    "friend",
    "friends",
    "mentor",
    "teacher",
    "master",
];

/// Whether `q` holds a possessive phrase naming one of `RELATIONS`: a possessive pronoun or a name with "'s" (or a
/// plural's "s'") right before the relation, which may itself be a possessive ("her mom's stuff").
fn names_relation(q: &str) -> bool {
    let ql = q.to_lowercase().replace(['\u{2019}', '\u{2018}'], "'");
    let words: Vec<&str> = ql
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .collect();
    words.windows(2).any(|w| {
        // "it's", "she's" and the like are "is", not a possessive.
        let owner = matches!(w[0], "her" | "his" | "their" | "its" | "my" | "your" | "our")
            || (w[0].ends_with("'s")
                && !matches!(
                    w[0],
                    "it's" | "he's" | "she's" | "that's" | "who's" | "what's" | "there's" | "here's" | "let's"
                ))
            || w[0].ends_with("s'");
        owner && RELATIONS.contains(&w[1].trim_end_matches("'s").trim_end_matches('\''))
    })
}

/// Whether one of `forms` stands in `qn` (folded words) where a death word follows it (a possessive "s" between) or
/// "death of", "kill" or "killed" precedes it, with no "and", "or" or "nor" right before it.
fn death_attached(qn: &str, forms: &[String]) -> bool {
    let words: Vec<&str> = qn.split_whitespace().collect();
    let death = |w: Option<&&str>| w.is_some_and(|w| DEATH_WORDS.contains(w));
    forms.iter().any(|f| {
        let fw: Vec<&str> = f.split_whitespace().collect();
        (0..words.len())
            .filter(|&i| !fw.is_empty() && words[i..].starts_with(&fw))
            .any(|i| {
                let end = i + fw.len();
                let before = i.checked_sub(1).and_then(|j| words.get(j));
                if matches!(before, Some(&("and" | "or" | "nor"))) {
                    return false;
                }
                let follows = death(words.get(end)) || (words.get(end) == Some(&"s") && death(words.get(end + 1)));
                let precedes = matches!(before, Some(&("kill" | "killed")))
                    || (before == Some(&"of") && i.checked_sub(2).and_then(|j| words.get(j)) == Some(&"death"));
                follows || precedes
            })
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn tools() -> Tools {
        Tools {
            deaths: Some(vec![
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
            ]),
            ..Tools::default()
        }
    }

    #[test]
    fn alive_or_dead_questions_name_one_character() {
        let t = tools();
        for q in [
            "Is Mandragora alive or dead?",
            "Did Mandragora die?",
            "Is Mandragora still alive?",
            "mandragora dead or alive",
            "Does Mandragora survive Episode 10?",
        ] {
            assert_eq!(alive_dead_subject(q, &t).as_deref(), Some("Mandragora"), "{q}");
        }
        for q in [
            "Who killed Mandragora?",
            "Which characters died in Episode 10?",
            "Tell me about Mandragora",
            "What does Arturia play during the death of Mandragora?",
            "Was the teacher of W killed?",
            "Is Mandragora related to Mandra or W? Is W dead?",
        ] {
            assert_eq!(alive_dead_subject(q, &t), None, "{q}");
        }
    }

    #[test]
    fn possessives_count_and_how_questions_do_not() {
        let t = tools();
        for q in [
            "Is Mandragora's death confirmed?",
            "Was Mandragora\u{2019}s death real?",
        ] {
            assert_eq!(alive_dead_subject(q, &t).as_deref(), Some("Mandragora"), "{q}");
        }
        for q in [
            "How did Mandragora die?",
            "Why did Mandragora die?",
            "When did Mandragora die?",
            "Where did Mandragora die?",
        ] {
            assert_eq!(alive_dead_subject(q, &t), None, "{q}");
        }
    }

    #[test]
    fn a_row_without_a_quote_is_not_given() {
        let mut t = tools();
        if let Some(d) = t.deaths.as_mut() {
            d.push(
                json!({"character": "Mandragora", "kind": "dead", "generic": false, "releaseTime": 40,
                          "chunkId": "c#7", "quote": "  "}),
            );
            d.push(json!({"character": "W", "kind": "dies", "generic": false, "releaseTime": 2, "chunkId": "c#8"}));
        }
        let ids: Vec<&str> = death_rows(&t, "Mandragora")
            .iter()
            .filter_map(|e| e["chunkId"].as_str())
            .collect();
        assert_eq!(ids, vec!["c#2", "c#3"]);
        let w: Vec<&str> = death_rows(&t, "W")
            .iter()
            .filter_map(|e| e["chunkId"].as_str())
            .collect();
        assert_eq!(w, vec!["c#5"]);
    }

    fn vanish_tools() -> Tools {
        Tools {
            deaths: Some(vec![
                json!({"character": "Kal'tsit", "kind": "dead", "generic": false, "releaseTime": 50, "chunkId": "k#1",
                   "quote": "I know she's dead, obviously."}),
                json!({"character": "Mandragora", "kind": "dies", "generic": false, "releaseTime": 20, "chunkId": "c#2",
                   "quote": "x"}),
            ]),
            identities: vec![
                vec!["AMa-10".into(), "Kal'tsit".into()],
                vec!["Amiya".into()],
                vec!["Delphine".into()],
            ],
            ..Tools::default()
        }
    }

    #[test]
    fn a_death_question_without_a_death_scene_is_a_vanished_fate() {
        let t = vanish_tools();
        let keys = |xs: &[&str]| xs.iter().map(|x| norm(x)).collect::<Vec<_>>();
        let none = Vanished {
            who: "Amiya".into(),
            reports: None,
        };
        assert_eq!(
            vanished_subject("How did Amiya die?", &keys(&["Amiya"]), &t),
            Some(none)
        );
        let kal = || {
            Some(Vanished {
                who: "Kal'tsit".into(),
                reports: Some("Kal'tsit".into()),
            })
        };
        assert_eq!(
            vanished_subject(
                "How does Kal'tsit die, and how does she get revived?",
                &keys(&["Kal'tsit"]),
                &t
            ),
            kal()
        );
        // Of two people, the one the death word follows; an identity name finds the same rows.
        assert_eq!(
            vanished_subject(
                "How does Mon3tr react when she wakes up after Kal'tsit dies",
                &keys(&["Mon3tr", "Kal'tsit"]),
                &t
            ),
            kal()
        );
        assert_eq!(
            vanished_subject(
                "What happened after the death of Kal'tsit, according to Mon3tr?",
                &keys(&["Kal'tsit", "Mon3tr"]),
                &t
            ),
            kal()
        );
        assert_eq!(
            vanished_subject("How did AMa-10 die?", &keys(&["AMa-10"]), &t).map(|v| v.reports),
            Some(Some("Kal'tsit".into()))
        );
    }

    #[test]
    fn a_death_scene_two_people_or_no_death_word_leave_the_question_alone() {
        let t = vanish_tools();
        let keys = |xs: &[&str]| xs.iter().map(|x| norm(x)).collect::<Vec<_>>();
        // A character with a death row ("dies").
        assert_eq!(
            vanished_subject("How did Mandragora die?", &keys(&["Mandragora"]), &t),
            None
        );
        // Two characters, neither the one the death word is about alone.
        for q in [
            "Did Kal'tsit or Amiya die?",
            "Are Amiya and Kal'tsit dead?",
            "Which of Amiya and Kal'tsit died?",
        ] {
            assert_eq!(vanished_subject(q, &keys(&["Kal'tsit", "Amiya"]), &t), None, "{q}");
        }
        // The name only as another's possessive.
        assert_eq!(
            vanished_subject("Was Amiya's teacher killed before chapter 2?", &keys(&["Amiya"]), &t),
            None
        );
        assert!(vanished_subject("Was Amiya's death real?", &keys(&["Amiya"]), &t).is_some());
        // No death word, or nobody named.
        assert_eq!(
            vanished_subject("Where did Kal'tsit go?", &keys(&["Kal'tsit"]), &t),
            None
        );
        // Two deaths asked of at once ("kill Mon3tr", "Kal'tsit died").
        assert_eq!(
            vanished_subject(
                "Did Amiya kill Mon3tr after Kal'tsit died?",
                &keys(&["Amiya", "Mon3tr", "Kal'tsit"]),
                &t
            ),
            None
        );
        assert_eq!(vanished_subject("Which characters died in Episode 15?", &[], &t), None);
    }

    #[test]
    fn a_relation_s_death_or_a_name_holding_death_is_not_a_vanished_fate() {
        let t = vanish_tools();
        let keys = |xs: &[&str]| xs.iter().map(|x| norm(x)).collect::<Vec<_>>();
        let delphine = || {
            Some(Vanished {
                who: "Delphine".into(),
                reports: None,
            })
        };
        // Trivia t093: her mother is the one who passed away.
        assert_eq!(
            vanished_subject(
                "was delphine hesitant to get her mom's stuff back when she passed away?",
                &keys(&["Delphine"]),
                &t
            ),
            None
        );
        assert_eq!(
            vanished_subject("Did Delphine's mother die?", &keys(&["Delphine"]), &t),
            None
        );
        assert_eq!(
            vanished_subject("Was Delphine sad when her parents died?", &keys(&["Delphine"]), &t),
            None
        );
        // A death word on the name itself still counts, a relation beside it or not; "she's" is not a possessive.
        assert_eq!(
            vanished_subject("Did Delphine die?", &keys(&["Delphine"]), &t),
            delphine()
        );
        assert_eq!(
            vanished_subject("Did Delphine die after her mother's death?", &keys(&["Delphine"]), &t),
            delphine()
        );
        assert_eq!(
            vanished_subject(
                "Is Delphine dead now that she's master of the house?",
                &keys(&["Delphine"]),
                &t
            ),
            delphine()
        );
        // Discord 85: "Deathless" is part of a name, not a death word.
        assert_eq!(
            vanished_subject(
                "Didn't they already kill the Deathless Black Snake or make it move on to someone \
                                     else?",
                &keys(&["Deathless Black Snake"]),
                &t
            ),
            None
        );
    }

    #[test]
    fn rows_are_the_first_death_and_the_latest_report() {
        let t = tools();
        let ids: Vec<&str> = death_rows(&t, "Mandragora")
            .iter()
            .filter_map(|e| e["chunkId"].as_str())
            .collect();
        assert_eq!(ids, vec!["c#2", "c#3"]);
        assert!(death_rows(&t, "Nobody").is_empty());
    }
}
