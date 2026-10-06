//! `--design-deduce`: what real-world thing an operator's design is modelled on, deduced by the answer
//! model from the game's art and text.

use anyhow::{Context, Result};
use serde::Deserialize;
use trevor::goldgen::llm::{Llm, Request};
use trevor::tools::Tools;


pub(crate) const DEDUCE_REVERSE_SYS: &str = "You deduce what real-world thing an Arknights operator's character design is modelled on, \
using only the game evidence below (the race, operator-file sentences, a model-written description of the art, and vision-model \
readings of the whole art and of its four 2x2 crops) together with your general knowledge of real animals, creatures, plants and \
objects. Name a subject only when specific visible features in the evidence match that subject's distinguishing features (body \
shape, parts, markings); a race name, a setting or a color scheme alone is not enough, and a creature in the background counts \
only when the character's own features or several readings agree with it. Give the most specific species the features justify. \
Give the subject by its common English name only (null when nothing specific matches), then the matching features found in \
the evidence. Output JSON only.";
// The subject comes first: with it after the feature list, Gemma 4 12B wrote broken names at temperature 0 ("bearear",
// "foxrichton fox", "black catsoorted cat"; 9 of the first 15 design answers, 2026-10-05), and clean ones ("bear") first.
pub(crate) const DEDUCE_REVERSE_GRAMMAR: &str = r#"root ::= "{\"subject\": " ( name | "null" ) ", \"kind\": " kind ", \"matching_features\": [" ( str ( ", " str ){0,4} )? "], \"confidence\": " conf "}"
name ::= "\"" [a-zA-Z][a-zA-Z '-]{1,45} "\""
kind ::= "\"animal\"" | "\"mythical creature\"" | "\"plant\"" | "\"object\"" | "\"none\""
conf ::= "\"high\"" | "\"medium\"" | "\"low\""
str ::= "\"" [^"\n\\]{1,90} "\""
"#;
pub(crate) const DEDUCE_LIST_SYS: &str = "A question asks which Arknights operator is based on a real-world thing. List the real-world animals, \
creatures, plants or objects that satisfy the question's constraint, from general knowledge only (for \"an animal with X in its \
name\", animals whose common English name contains X). For each give its common English name, the broader group it belongs to \
(\"bear\", \"jellyfish\", \"bird of prey\") and 3 to 5 distinguishing features \
that would be visible in a character illustration (body shape, parts, colors, markings). At most 6, the best known first. Output \
JSON only.";
pub(crate) const DEDUCE_LIST_GRAMMAR: &str = r#"root ::= "{\"candidates\": [" cand ( ", " cand ){0,5} "]}"
cand ::= "{\"name\": " str ", \"group\": " str ", \"features\": [" str ( ", " str ){1,4} "]}"
str ::= "\"" [^"\n\\]{1,80} "\""
"#;
pub(crate) const DEDUCE_MATCH_SYS: &str = "You match real-world candidates against Arknights operators' game evidence: the race, operator-file \
sentences, a model-written description of the art, and vision-model readings of the whole art and of its four 2x2 crops. An \
operator matches a candidate only when its evidence shows at least two of the candidate's distinguishing features, or names the \
candidate or a near relative and shows one of its features; a color scheme or a setting alone is not a match. List the matching \
operators, best first, at most 3, each with the candidate and the features from its evidence that match; an empty list when no \
operator matches. Output JSON only.";

#[derive(Deserialize)]
pub(crate) struct DeduceReverse { subject: Option<String>, kind: String, matching_features: Vec<String>, confidence: String }
#[derive(Deserialize)]
pub(crate) struct DeduceCand { name: String, group: String, features: Vec<String> }
#[derive(Deserialize)]
pub(crate) struct DeduceList { candidates: Vec<DeduceCand> }
#[derive(Deserialize)]
pub(crate) struct DeduceMatch { operator: String, candidate: String, matching_features: Vec<String>, confidence: String }
#[derive(Deserialize)]
pub(crate) struct DeduceMatches { matches: Vec<DeduceMatch> }

pub(crate) async fn deduce_call<T: serde::de::DeserializeOwned>(llm: &Llm, system: &str, user: &str, grammar: &str, n: u32) -> Result<T> {
    let c = llm.complete(&Request::greedy(system, user, Some(grammar), n, &[])).await?;
    serde_json::from_str(c.content.trim()).with_context(|| format!("design deduction output did not parse: {:?}", c.content))
}

pub(crate) const DEDUCE_LABEL: &str = "Trevor's deduction from the game's art and text (the game never says what a design is based on; \
this matches visible features in the art and the operator files against real-world candidates, so treat it as an inference)";

/// `--design-deduce` (2026-10-05): the two-step design deduction, or `None` when there is nothing to deduce from (no
/// evidence rows; a reverse question naming no operator).
pub(crate) async fn design_deduce(llm: &Llm, tools: &Tools, q: &str, forward: bool, named: &[&serde_json::Value]) -> Result<Option<String>> {
    let st = |v: &serde_json::Value, k: &str| v[k].as_str().unwrap_or_default().to_owned();
    if !forward {
        let Some(r) = named.first() else { return Ok(None) };
        let name = st(r, "operator");
        let user = format!("Question: {q}\n\nOperator: {name}\n{}", st(r, "text"));
        let v: DeduceReverse = deduce_call(llm, DEDUCE_REVERSE_SYS, &user, DEDUCE_REVERSE_GRAMMAR, 300).await?;
        let race = st(r, "race");
        let subject = v.subject.filter(|x| !x.trim().is_empty() && !matches!(x.trim().to_lowercase().as_str(), "none" | "null" | "unknown")
            && v.kind != "none" && v.confidence != "low");
        return Ok(Some(match subject {
            Some(sub) => format!("{DEDUCE_LABEL}:\n- {name}: likely modelled on the {sub} ({}, {} confidence). Matching features in the \
game's art and text: {}.\nThe game's own data gives the race as {}.", v.kind, v.confidence, v.matching_features.join("; "),
                if race.is_empty() { "not stated".to_owned() } else { race }),
            None => format!("{DEDUCE_LABEL}:\n- {name}: the game does not show it. Nothing in {name}'s art readings or operator file \
points to a specific real-world animal, creature, plant or object beyond the race ({}).",
                if race.is_empty() { "not stated".to_owned() } else { race }),
        }));
    }
    let list: DeduceList = deduce_call(llm, DEDUCE_LIST_SYS, &format!("Question: {q}"), DEDUCE_LIST_GRAMMAR, 500).await?;
    if list.candidates.is_empty() {
        return Ok(None);
    }
    let mut terms: Vec<(String, f64)> = Vec::new();
    for c in &list.candidates {
        terms.push((c.name.clone(), 2.0));
        terms.push((c.group.clone(), 2.0));
        terms.extend(c.features.iter().map(|f| (f.clone(), 1.0)));
    }
    let short = tools.design_shortlist(&terms, 8);
    if std::env::var("TREVOR_DEDUCE_DEBUG").is_ok() {
        eprintln!("deduce terms {terms:?}\nshortlist {:?}", short.iter().map(|r| r["operator"].as_str().unwrap_or_default()).collect::<Vec<_>>());
    }
    let cands = list.candidates.iter().map(|c| format!("- {} ({}): {}", c.name, c.group, c.features.join("; "))).collect::<Vec<_>>().join("\n");
    let considered = list.candidates.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ");
    if short.is_empty() {
        return Ok(Some(format!("{DEDUCE_LABEL}:\nNothing in the game shows a match. Real-world candidates for the question (from general \
knowledge): {considered}; no operator's art readings or file share their features.")));
    }
    let names: Vec<String> = short.iter().map(|r| st(r, "operator")).collect();
    let ops = short.iter().enumerate().map(|(i, r)| format!("[{}] {}\n{}", i + 1, st(r, "operator"), st(r, "text"))).collect::<Vec<_>>().join("\n\n");
    let alts = names.iter().map(|n| format!("\"\\\"{}\\\"\"", n.replace(['"', '\\'], ""))).collect::<Vec<_>>().join(" | ");
    let grammar = format!(r#"root ::= "{{\"matches\": [" ( m ( ", " m ){{0,2}} )? "]}}"
m ::= "{{\"operator\": " op ", \"candidate\": " str ", \"matching_features\": [" str ( ", " str ){{0,4}} "], \"confidence\": " conf "}}"
op ::= {alts}
conf ::= "\"high\"" | "\"medium\"" | "\"low\""
str ::= "\"" [^"\n\\]{{1,90}} "\""
"#);
    let user = format!("Question: {q}\n\nCandidates:\n{cands}\n\nOperators:\n{ops}");
    let m: DeduceMatches = deduce_call(llm, DEDUCE_MATCH_SYS, &user, &grammar, 500).await?;
    let near = names.iter().take(4).cloned().collect::<Vec<_>>().join(", ");
    // Grounding check: a match keeps only the features its operator's evidence shares a rare word with, and needs two of
    // them, or one when the evidence names the candidate or its group (the matching step invented "branching dorsal
    // appendages" for Lucilla on the sea-slug dev question).
    let mut m = m;
    for x in &mut m.matches {
        let Some(row) = short.iter().find(|r| r["operator"].as_str() == Some(x.operator.as_str())) else { x.matching_features.clear(); continue };
        x.matching_features.retain(|f| tools.design_grounded(row, f));
        let cand = list.candidates.iter().find(|c| c.name == x.candidate);
        let named = cand.is_some_and(|c| tools.design_grounded(row, &c.name) || tools.design_grounded(row, &c.group)) || tools.design_grounded(row, &x.candidate);
        if x.matching_features.len() < if named { 1 } else { 2 } {
            x.matching_features.clear();
        }
    }
    m.matches.retain(|x| !x.matching_features.is_empty());
    if m.matches.is_empty() {
        return Ok(Some(format!("{DEDUCE_LABEL}:\nNothing in the game shows a match. Real-world candidates for the question (from general \
knowledge): {considered}. The operators whose art readings and files come closest ({near}) do not show their distinguishing \
features, so the game does not show which operator, if any, is based on one.")));
    }
    let lines = m.matches.iter().map(|x| format!("- {}: likely the {} ({} confidence). Matching features in the game's art and text: {}.",
        x.operator, x.candidate, x.confidence, x.matching_features.join("; "))).collect::<Vec<_>>().join("\n");
    Ok(Some(format!("{DEDUCE_LABEL}:\n{lines}\nReal-world candidates considered (from general knowledge): {considered}.")))
}
