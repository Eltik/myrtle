//! Unit tests of the router prompt variants.

use super::model::{source_variant, with_wide_topics, without_compare};
use super::*;

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
    assert_eq!(ROUTER_SYSTEM.lines().count() - s.lines().count(), 6, "the tool line, the rule, and 2 examples of 2 lines");
    assert_eq!(ROUTER_GRAMMAR.lines().count() - g.lines().count(), 2);
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

/// Pins the exact bytes of the router prompt and grammar `ask` sends: the default (`source`, `compare` and
/// `wide_topics` on), `--no-router-source`, `--lore v1`, and every part off. A prompt edit that is meant to change
/// nothing must leave these hashes as they are.
#[test]
fn router_prompt_bytes_are_pinned() {
    let default = RouterPrompt { source: true, compare: true, wide_topics: true };
    let cases = [
        ("default", default),
        ("no_router_source", RouterPrompt { source: false, ..default }),
        ("lore_v1", RouterPrompt { wide_topics: false, ..default }),
        ("all_off", RouterPrompt::default()),
    ];
    let got: Vec<(&str, String)> = cases
        .iter()
        .map(|(name, o)| {
            let (s, g) = router_prompt(o);
            (*name, crate::util::sha16(&format!("{s}\u{0}{g}")))
        })
        .collect();
    let want = [
        ("default", "80700fbf1135697a"),
        ("no_router_source", "d4f6d6a025dc9925"),
        ("lore_v1", "d743a4d4f1abc807"),
        ("all_off", "8277433626664b44"),
    ];
    for ((name, sha), (wname, wsha)) in got.iter().zip(want) {
        assert_eq!((*name, sha.as_str()), (wname, wsha), "router prompt bytes changed: {got:?}");
    }
}
