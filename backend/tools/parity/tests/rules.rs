use myrtle_parity::rules::{RuleOverrides, Rules, RulesFile};

#[test]
fn file_and_cli_merge() {
    let file: RulesFile = serde_json::from_str(
        r#"{"rel_tol": 1e-6, "unordered": ["/a"], "routes": {"/api/health": {"ignore": ["/uptime"], "skip": false}, "/api/stream": {"skip": true}}}"#,
    )
    .unwrap();
    let cli = RuleOverrides {
        rel_tol: Some(1e-12),
        unordered: vec!["/b".to_owned()],
        max_diffs: 5,
        ..RuleOverrides::default()
    };
    let rules = Rules::build(Some(file), &cli).unwrap();
    assert!((rules.rel_tol() - 1e-12).abs() < 1e-24);
    let base = rules.for_route("/api/other");
    assert_eq!(base.unordered.len(), 2);
    assert!(base.ignore.is_empty());
    let health = rules.for_route("/api/health");
    assert_eq!(health.ignore.len(), 1);
    assert_eq!(health.unordered.len(), 2);
    assert!(rules.skipped("/api/stream"));
    assert!(!rules.skipped("/api/health"));
}

#[test]
fn rejects_bad_input() {
    assert!(serde_json::from_str::<RulesFile>(r#"{"unknown": 1}"#).is_err());
    let bad_glob = RuleOverrides {
        ignore: vec!["no-slash".to_owned()],
        ..RuleOverrides::default()
    };
    assert!(Rules::build(None, &bad_glob).is_err());
    let bad_tol = RuleOverrides {
        rel_tol: Some(-1.0),
        ..RuleOverrides::default()
    };
    assert!(Rules::build(None, &bad_tol).is_err());
}
