use myrtle_parity::compare::{Comparison, Diff, DiffKind};
use myrtle_parity::corpus::{CorpusEntry, Source};
use myrtle_parity::report::{Outcome, Summary};

fn entry(route: &str, path: &str) -> CorpusEntry {
    CorpusEntry::new(route, path, None, Source::Openapi)
}

#[test]
fn counts_and_caps_problems() {
    let mut s = Summary::new("http://a", "http://b", 1e-9);
    let diff = Comparison {
        diffs: vec![Diff {
            pointer: "/x".to_owned(),
            kind: DiffKind::Number,
            a: Some("1".to_owned()),
            b: Some("2".to_owned()),
        }],
        total: 3,
    };
    s.add(&entry("/r", "/r/1"), Outcome::Equal, 1);
    s.add(&entry("/r", "/r/2"), Outcome::Different(diff), 1);
    s.add(&entry("/q", "/q"), Outcome::Error("timeout".to_owned()), 1);
    s.add(
        &entry("/q", "/q2"),
        Outcome::Skipped("no token".to_owned()),
        1,
    );
    s.add(
        &entry("/q", "/q3"),
        Outcome::RateLimited { a: 200, b: 429 },
        1,
    );
    assert_eq!(
        (
            s.total,
            s.equal,
            s.different,
            s.errors,
            s.skipped,
            s.rate_limited
        ),
        (5, 1, 1, 1, 1, 1)
    );
    assert_eq!(s.problems.len(), 1);
    assert_eq!(s.problems_not_listed, 2);
    assert!(!s.passed());
    let text = s.render_text();
    assert!(
        text.contains("/x: number outside tolerance  A=1  B=2"),
        "{text}"
    );
    assert!(text.contains("... 2 more"), "{text}");
    assert!(text.contains("result: FAIL"));
    let json = serde_json::to_value(&s).unwrap();
    assert_eq!(json["routes"]["/q"]["errors"], 1);
    assert_eq!(json["problems"][0]["diffs"][0]["pointer"], "/x");
    assert_eq!(json["skip_reasons"]["no token"], 1);
}

#[test]
fn skips_alone_pass() {
    let mut s = Summary::new("a", "b", 1e-9);
    s.add(&entry("/r", "/r"), Outcome::Skipped("x".to_owned()), 10);
    s.add(&entry("/r", "/r2"), Outcome::Equal, 10);
    assert!(s.passed());
    assert!(s.render_text().contains("result: PASS"));
}
