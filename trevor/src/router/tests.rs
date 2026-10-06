//! Unit tests of the router prompt variants, the dictionary arguments and the vote.

use serde_json::json;

use super::model::{
    ART_CLAUSE, ART_EXAMPLE, CROSS_REF_EXAMPLE, CROSS_REF_GRAMMAR, CROSS_REF_LINE, VOICE_CLAUSE, VOICE_CLAUSE_EVERYDAY,
    VOICE_EXAMPLE, source_variant, with_wide_topics, without_compare, without_overview,
};
use super::*;
use crate::tools::Tools;

#[test]
fn wide_topic_line_changes_only_that_line() {
    for (sys, _) in [(ROUTER_SYSTEM.to_owned(), ""), (source_variant().0.clone(), "")] {
        let w = with_wide_topics(&sys);
        let (a, b): (Vec<&str>, Vec<&str>) = (sys.lines().collect(), w.lines().collect());
        assert_eq!(a.len(), b.len());
        assert_eq!(a.iter().zip(&b).filter(|(x, y)| x != y).count(), 1);
        assert!(w.contains("Reunion"));
    }
}

#[test]
fn dropping_reading_compare_leaves_no_trace_of_it() {
    let (s, g) = without_compare(ROUTER_SYSTEM, ROUTER_GRAMMAR);
    assert!(!s.contains("reading_compare") && !g.contains("reading_compare") && !g.contains("other ::="));
    assert_eq!(ROUTER_SYSTEM.lines().count() - s.lines().count(), 7, "the tool line, the two rules, and 2 examples of 2 lines");
    assert_eq!(ROUTER_GRAMMAR.lines().count() - g.lines().count(), 2);
}

#[test]
fn dropping_overview_leaves_no_trace_of_it() {
    let (s, g) = without_overview(ROUTER_SYSTEM, ROUTER_GRAMMAR);
    assert!(!s.contains("overview") && !g.contains("overview"));
    assert_eq!(ROUTER_SYSTEM.lines().count() - s.lines().count(), 6, "the tool line, the reading_compare rule, and 2 examples of 2 lines");
    assert_eq!(ROUTER_GRAMMAR.lines().count() - g.lines().count(), 1);
    let (s2, g2) = without_overview(&source_variant().0, &source_variant().1);
    assert!(!s2.contains("overview") && !g2.contains("overview"));
}

#[test]
fn voice_rule_and_art_are_inert_when_off() {
    let base = RouterPrompt { source: true, compare: true, ..RouterPrompt::default() };
    let (s0, g0) = router_prompt(&base);
    // off: the prompt and grammar route_model_with sent before these options existed
    let (mut s_old, mut g_old) = (source_variant().0.clone(), source_variant().1.clone());
    (s_old, g_old) = without_overview(&s_old, &g_old);
    assert_eq!((s0.clone(), g0.clone()), (s_old, g_old));
    let (s1, g1) = router_prompt(&RouterPrompt { voice_rule: true, ..base });
    assert_eq!(g1, g0);
    assert!(s1.contains(VOICE_CLAUSE_EVERYDAY) && s1.contains("Hoshiguma") && !s0.contains("Hoshiguma"));
    assert_eq!(s1.replacen(VOICE_CLAUSE_EVERYDAY, VOICE_CLAUSE, 1).replacen(VOICE_EXAMPLE, "", 1), s0);
    assert!(s1.contains("}}\nQ: what does Hoshiguma") && s1.contains("\"voice\"}}\n\nAnswer with the JSON object only."));
    let (s2, g2) = router_prompt(&RouterPrompt { art: true, ..base });
    assert!(g2.contains("| \"any\" | \"art\")") && g2.replacen(" | \"art\"", "", 1) == g0);
    assert_eq!(s2.replacen(ART_CLAUSE, "", 1).replacen(ART_EXAMPLE, "", 1), s0);
    let (s3, g3) = router_prompt(&RouterPrompt { cross_ref: true, ..base });
    assert_eq!(s3.replacen(CROSS_REF_LINE, "", 1).replacen(CROSS_REF_EXAMPLE, "", 1), s0);
    assert_eq!(g3.replacen(CROSS_REF_GRAMMAR, "", 1), g0);
    assert!(g3.contains("cross_ref") && s3.contains("bosses_playable"));
    // without a source the options do nothing
    let plain = RouterPrompt { compare: true, ..RouterPrompt::default() };
    assert_eq!(router_prompt(&RouterPrompt { voice_rule: true, art: true, ..plain }), router_prompt(&plain));
}

#[test]
fn the_source_variant_changes_only_retrieval() {
    let (system, grammar) = source_variant();
    assert!(system.contains("source names the kind of game text") && !system.contains("- retrieve {}: everything else."));
    assert!(!system.contains(r#""retrieve", "args": {}}"#) && system.contains("Answer with the JSON object only."));
    assert!(grammar.contains(r#""retrieve\", \"args\": {" source "}""#), "{grammar}");
    assert!(grammar.ends_with("source ::= \"\\\"source\\\": \\\"\" (\"story\" | \"operator_file\" | \"module\" | \"voice\" | \"skin\" | \"is\" | \"enemy\" | \"item\" | \"any\") \"\\\"\"\n"), "{grammar}");
    assert!(system.len() - ROUTER_SYSTEM.len() > 500);
}

fn dict() -> Dict {
    let mut t = Tools::default();
    for (id, name, ch) in [("act33side", "Babel", None), ("main_7", "The Birth of Tragedy", Some(7)), ("act25side", "Lone Trail", None)] {
        let mut g = json!({"groupId": id, "name": name});
        if let Some(c) = ch {
            g["chapter"] = json!(c);
        }
        t.groups.insert(id.to_owned(), g);
    }
    t.attributes = Some(vec![
        json!({"name": "Texas", "race": "Lupo", "nation": "Lungmen", "class": "Vanguard", "branch": "Pioneer"}),
        json!({"name": "Texas the Omertosa", "race": "Lupo", "nation": "Siracusa", "class": "Specialist", "branch": "Executor"}),
        json!({"name": "Gummy", "race": "Ursus", "nation": "Ursus", "class": "Defender", "branch": "Fortress"}),
        json!({"name": "Ceobe", "race": "Perro", "nation": "Rhodes Island", "class": "Caster", "branch": "Medic"}),
    ]);
    t.deaths = Some(vec![json!({"character": "W", "generic": false}), json!({"character": "Patriot", "generic": false})]);
    Dict::new(&t)
}

#[test]
fn arguments_come_from_the_longest_whole_word_match() {
    let d = dict();
    let a = |tool: &str, q: &str| extract_args(tool, q, &d).map(|r| r.args.into_iter().collect::<Vec<_>>());
    assert_eq!(a("operator_attribute", "How tall is Texas the Omertosa?").unwrap(),
               vec![("field".into(), "height".into()), ("operator".into(), "Texas the Omertosa".into())]);
    assert_eq!(a("reading_time", "How long is Lone Trail?").unwrap(), vec![("event".into(), "Lone Trail".into())]);
    assert_eq!(a("deaths_in_event", "Who died in episode 7?").unwrap(),
               vec![("event".into(), "Episode 7".into()), ("who".into(), "all".into())]);
    assert_eq!(a("death_of", "Does W die?").unwrap(), vec![("character".into(), "W".into())]);
    assert!(a("death_of", "Does Wisadel die?").is_none(), "no name in the table: retrieval");
    assert!(a("reading_event", "When should I read the good one?").is_none());
    // Ursus as place, not also race; a branch named like its class (Medic) is left to the class.
    assert_eq!(a("operator_filter", "Which 6-star operators are Ursus?").unwrap(),
               vec![("place".into(), "Ursus".into()), ("rarity".into(), "6".into())]);
    assert_eq!(a("operator_filter", "Which medics are infected?").unwrap(),
               vec![("class".into(), "Medic".into()), ("infected".into(), "true".into())]);
}

#[test]
fn the_vote_is_weighted_and_reports_its_share() {
    let ex = vec![(vec![1.0, 0.0], "death_of".to_owned()), (vec![0.9, 0.436], "death_of".to_owned()),
                  (vec![0.0, 1.0], "retrieve".to_owned())];
    let v = vote(&ex, &[1.0, 0.0], 3, None);
    assert_eq!(v.tool, "death_of");
    assert!((v.top - 1.0).abs() < 1e-6 && v.share > 0.99);
    let v = vote(&ex, &[1.0, 0.0], 3, Some(0));
    assert!((v.top - 0.9).abs() < 1e-6);
}
