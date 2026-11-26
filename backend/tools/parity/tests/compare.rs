#![allow(clippy::unwrap_used)]

use myrtle_parity::compare::{
    Body, CompareOptions, Comparison, Diff, DiffKind, PREVIEW_CHARS, canonical_json, compare_json,
    compare_observed, floats_equal, observe, preview,
};
use myrtle_parity::glob::PointerGlob;
use serde_json::{Value, json};

fn opts() -> CompareOptions {
    CompareOptions::default()
}

fn with_unordered(globs: &[&str]) -> CompareOptions {
    CompareOptions {
        unordered: globs
            .iter()
            .map(|g| PointerGlob::parse(g).unwrap())
            .collect(),
        ..CompareOptions::default()
    }
}

fn pointers(c: &Comparison) -> Vec<(String, DiffKind)> {
    c.diffs
        .iter()
        .map(|d| (d.pointer.clone(), d.kind))
        .collect()
}

#[test]
fn float_inside_relative_tolerance_is_equal() {
    let c = compare_json(
        &json!({"x": 1000.0}),
        &json!({"x": 1_000.000_000_1}),
        &opts(),
    );
    assert!(c.is_equal(), "{c:?}");
}

#[test]
fn float_just_outside_relative_tolerance_differs() {
    let c = compare_json(&json!({"x": 1.0}), &json!({"x": 1.000_000_002}), &opts());
    assert_eq!(pointers(&c), vec![("/x".to_owned(), DiffKind::Number)]);
}

#[test]
fn tolerance_boundary_is_inclusive_and_symmetric() {
    let rel = 1e-9;
    let x = 1.0e6;
    let y = x * (1.0 + rel * 0.5);
    assert!(floats_equal(x, y, rel, 0.0));
    assert!(floats_equal(y, x, rel, 0.0));
    let z = x * (1.0 + rel * 3.0);
    assert!(!floats_equal(x, z, rel, 0.0));
    assert!(!floats_equal(z, x, rel, 0.0));
}

#[test]
fn zero_needs_absolute_floor() {
    assert!(!floats_equal(0.0, 1e-300, 1e-9, 0.0));
    assert!(floats_equal(0.0, 1e-300, 1e-9, 1e-12));
    assert!(floats_equal(0.0, -0.0, 1e-9, 0.0));
}

#[test]
fn configurable_tolerance() {
    let loose = CompareOptions {
        rel_tol: 0.0015,
        ..CompareOptions::default()
    };
    assert!(compare_json(&json!(100.0), &json!(100.1), &loose).is_equal());
    assert!(!compare_json(&json!(100.0), &json!(100.2), &loose).is_equal());
}

#[test]
fn integers_compare_exactly_and_int_equals_same_float() {
    assert!(
        !compare_json(
            &json!(10_000_000_000_i64),
            &json!(10_000_000_001_i64),
            &opts()
        )
        .is_equal()
    );
    assert!(compare_json(&json!(1), &json!(1.0), &opts()).is_equal());
    assert!(compare_json(&json!(u64::MAX), &json!(u64::MAX), &opts()).is_equal());
    assert!(!compare_json(&json!(-1), &json!(u64::MAX), &opts()).is_equal());
}

#[test]
fn nan_and_infinities() {
    assert!(floats_equal(f64::NAN, f64::NAN, 1e-9, 0.0));
    assert!(!floats_equal(f64::NAN, 1.0, 1e-9, 0.0));
    assert!(!floats_equal(1.0, f64::NAN, 1e-9, 0.0));
    assert!(floats_equal(f64::INFINITY, f64::INFINITY, 1e-9, 0.0));
    assert!(!floats_equal(f64::INFINITY, f64::NEG_INFINITY, 1e-9, 0.0));
    assert!(!floats_equal(f64::INFINITY, f64::MAX, 1e-9, 1e300));
}

#[test]
fn null_is_its_own_type() {
    assert!(compare_json(&json!({"v": null}), &json!({"v": null}), &opts()).is_equal());
    let c = compare_json(&json!({"v": 0.0}), &json!({"v": null}), &opts());
    assert_eq!(pointers(&c), vec![("/v".to_owned(), DiffKind::Type)]);
    let c = compare_json(&json!({"v": null}), &json!({}), &opts());
    assert_eq!(pointers(&c), vec![("/v".to_owned(), DiffKind::MissingInB)]);
}

#[test]
fn object_key_order_is_ignored() {
    let a: Value = serde_json::from_str(r#"{"a":1,"b":{"x":1,"y":2},"c":[1,2]}"#).unwrap();
    let b: Value = serde_json::from_str(r#"{"c":[1,2],"b":{"y":2,"x":1},"a":1}"#).unwrap();
    assert!(compare_json(&a, &b, &opts()).is_equal());
}

#[test]
fn missing_and_extra_keys_are_reported_sorted() {
    let c = compare_json(
        &json!({"a": 1, "z": 2}),
        &json!({"a": 1, "m": 3, "b": 4}),
        &opts(),
    );
    assert_eq!(
        pointers(&c),
        vec![
            ("/z".to_owned(), DiffKind::MissingInB),
            ("/b".to_owned(), DiffKind::MissingInA),
            ("/m".to_owned(), DiffKind::MissingInA),
        ]
    );
}

#[test]
fn nested_array_diffs_carry_full_pointer() {
    let a = json!({"ops": [{"id": "a", "stats": [[1, 2], [3, 4]]}]});
    let b = json!({"ops": [{"id": "a", "stats": [[1, 2], [3, 5]]}]});
    let c = compare_json(&a, &b, &opts());
    assert_eq!(
        pointers(&c),
        vec![("/ops/0/stats/1/1".to_owned(), DiffKind::Number)]
    );
    assert_eq!(c.diffs[0].a.as_deref(), Some("4"));
    assert_eq!(c.diffs[0].b.as_deref(), Some("5"));
}

#[test]
fn array_order_matters_by_default() {
    let c = compare_json(&json!([1, 2, 3]), &json!([3, 2, 1]), &opts());
    assert_eq!(c.total, 2);
    assert_eq!(c.diffs[0].pointer, "/0");
}

#[test]
fn array_length_reported_and_prefix_compared() {
    let c = compare_json(&json!([1, 2, 3]), &json!([1, 9]), &opts());
    assert_eq!(
        pointers(&c),
        vec![
            (String::new(), DiffKind::Length),
            ("/1".to_owned(), DiffKind::Number)
        ]
    );
}

#[test]
fn unordered_path_ignores_order() {
    let o = with_unordered(&["/tags"]);
    assert!(
        compare_json(
            &json!({"tags": ["a", "b", "c"]}),
            &json!({"tags": ["c", "a", "b"]}),
            &o
        )
        .is_equal()
    );
    let c = compare_json(
        &json!({"tags": ["a", "a", "b"]}),
        &json!({"tags": ["a", "b", "b"]}),
        &o,
    );
    assert_eq!(
        pointers(&c),
        vec![
            ("/tags/1".to_owned(), DiffKind::UnmatchedInA),
            ("/tags/2".to_owned(), DiffKind::UnmatchedInB)
        ]
    );
}

#[test]
fn unordered_glob_applies_at_depth_with_tolerance() {
    let o = with_unordered(&["/ops/*/drops"]);
    let a = json!({"ops": [{"drops": [{"id": 1, "p": 0.1}, {"id": 2, "p": 0.2}]}]});
    let b = json!({"ops": [{"drops": [{"p": 0.200_000_000_000_000_04, "id": 2}, {"id": 1, "p": 0.1}]}]});
    assert!(compare_json(&a, &b, &o).is_equal());
    assert!(!compare_json(&a, &b, &opts()).is_equal());
}

#[test]
fn nested_unordered_inside_unordered() {
    let o = with_unordered(&["", "/*/tags"]);
    let a = json!([{"id": 1, "tags": ["x", "y"]}, {"id": 2, "tags": []}]);
    let b = json!([{"id": 2, "tags": []}, {"id": 1, "tags": ["y", "x"]}]);
    assert!(compare_json(&a, &b, &o).is_equal());
}

#[test]
fn unordered_root_glob_with_double_star() {
    let o = with_unordered(&["/**"]);
    let a = json!({"a": [[1, 2], [3]], "b": [true, false]});
    let b = json!({"a": [[3], [2, 1]], "b": [false, true]});
    assert!(compare_json(&a, &b, &o).is_equal());
}

#[test]
fn ignored_pointers_are_skipped_including_missing_keys() {
    let o = CompareOptions {
        ignore: vec![
            PointerGlob::parse("/uptime").unwrap(),
            PointerGlob::parse("/**/generated_at").unwrap(),
        ],
        ..CompareOptions::default()
    };
    let a = json!({"uptime": 1, "x": {"generated_at": 5}, "v": 1});
    let b = json!({"uptime": 2, "x": {}, "v": 1});
    assert!(compare_json(&a, &b, &o).is_equal());
}

#[test]
fn max_diffs_caps_list_not_total() {
    let o = CompareOptions {
        max_diffs: 2,
        ..CompareOptions::default()
    };
    let c = compare_json(&json!([1, 2, 3, 4, 5]), &json!([0, 0, 0, 0, 0]), &o);
    assert_eq!(c.total, 5);
    assert_eq!(c.diffs.len(), 2);
}

#[test]
fn type_mismatch_and_bool_and_string() {
    let c = compare_json(
        &json!({"a": "1", "b": true, "c": "x"}),
        &json!({"a": 1, "b": false, "c": "y"}),
        &opts(),
    );
    assert_eq!(
        pointers(&c),
        vec![
            ("/a".to_owned(), DiffKind::Type),
            ("/b".to_owned(), DiffKind::Value),
            ("/c".to_owned(), DiffKind::Value),
        ]
    );
}

#[test]
fn canonical_json_sorts_keys() {
    let a: Value = serde_json::from_str(r#"{"b":1,"a":{"d":[1,{"z":0,"y":1}],"c":null}}"#).unwrap();
    assert_eq!(
        canonical_json(&a),
        r#"{"a":{"c":null,"d":[1,{"y":1,"z":0}]},"b":1}"#
    );
}

#[test]
fn preview_truncates() {
    let long = Value::String("x".repeat(500));
    assert!(preview(&long).ends_with("..."));
    assert!(preview(&long).chars().count() <= PREVIEW_CHARS + 3);
}

#[test]
fn status_mismatch_stops_before_body() {
    let a = observe(200, Some("application/json"), br#"{"a":1}"#);
    let b = observe(404, Some("application/json"), br#"{"error":"not found"}"#);
    let c = compare_observed(&a, &b, &opts());
    assert_eq!(pointers(&c), vec![(String::new(), DiffKind::Status)]);
}

#[test]
fn content_type_params_are_dropped() {
    let a = observe(200, Some("application/json; charset=utf-8"), b"[1]");
    let b = observe(200, Some("Application/JSON"), b"[1]");
    assert!(compare_observed(&a, &b, &opts()).is_equal());
    let c = observe(200, Some("text/plain"), b"[1]");
    let cmp = compare_observed(&a, &c, &opts());
    assert_eq!(
        pointers(&cmp),
        vec![
            (String::new(), DiffKind::ContentType),
            (String::new(), DiffKind::BodyHash)
        ]
    );
}

#[test]
fn binaries_compare_by_hash() {
    let a = observe(200, Some("image/png"), b"\x89PNG one");
    let same = observe(200, Some("image/png"), b"\x89PNG one");
    let other = observe(200, Some("image/png"), b"\x89PNG two");
    assert!(compare_observed(&a, &same, &opts()).is_equal());
    let c = compare_observed(&a, &other, &opts());
    assert_eq!(pointers(&c), vec![(String::new(), DiffKind::BodyHash)]);
    assert!(c.diffs[0].a.as_deref().unwrap().starts_with("sha256:"));
}

#[test]
fn invalid_json_falls_back_to_hash() {
    let a = observe(200, Some("application/json"), b"{not json");
    assert!(matches!(a.body, Body::Opaque { .. }));
    let b = observe(200, Some("application/json"), b"{\"ok\":1}");
    let c = compare_observed(&a, &b, &opts());
    assert_eq!(pointers(&c), vec![(String::new(), DiffKind::BodyHash)]);
}

#[test]
fn diff_display_is_readable() {
    let d = Diff {
        pointer: "/a/0".to_owned(),
        kind: DiffKind::Number,
        a: Some("1.0".to_owned()),
        b: Some("2.0".to_owned()),
    };
    assert_eq!(
        d.to_string(),
        "/a/0: number outside tolerance  A=1.0  B=2.0"
    );
    let s = Diff {
        pointer: String::new(),
        kind: DiffKind::Status,
        a: Some("200".to_owned()),
        b: Some("404".to_owned()),
    };
    assert_eq!(s.to_string(), "status differs  A=200  B=404");
    let r = Diff {
        pointer: String::new(),
        kind: DiffKind::Type,
        a: None,
        b: None,
    };
    assert_eq!(r.to_string(), "(root): type differs");
}
