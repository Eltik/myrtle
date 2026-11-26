use serde_json::{Value, json};
use xtask::layers::{check, metadata};

type Deps<'a> = &'a [(&'a str, &'a str)];

fn workspace(members: &[(&str, Deps<'_>)], layers: &[(u64, &[&str])], tools: &[&str]) -> Value {
    let packages: Vec<Value> = members
        .iter()
        .map(|(name, deps)| {
            let deps: Vec<Value> = deps
                .iter()
                .map(|(dep, kind)| {
                    let kind = if *kind == "normal" {
                        Value::Null
                    } else {
                        json!(kind)
                    };
                    json!({"name": dep, "kind": kind})
                })
                .collect();
            json!({"id": format!("path+file:///ws/{name}#0.1.0"), "name": name, "dependencies": deps})
        })
        .collect();
    let ids: Vec<&Value> = packages.iter().map(|p| &p["id"]).collect();
    let layers: Vec<Value> = layers
        .iter()
        .map(|(level, crates)| json!({"level": level, "crates": crates}))
        .collect();
    json!({
        "workspace_members": ids,
        "packages": packages,
        "metadata": {
            "layers": layers,
            "layer-rules": {
                "tools": tools,
                "dev-only": ["testkit"],
                "pure": ["types", "engine-*"],
                "pure-forbidden": ["sqlx", "storage"],
            },
        },
    })
}

fn violations(ws: &Value) -> Vec<String> {
    check(ws).expect("a readable layer table").violations
}

#[test]
fn downward_and_sideways_edges_pass() {
    let ws = workspace(
        &[
            ("types", &[]),
            ("engine-a", &[("types", "normal"), ("engine-b", "normal")]),
            ("engine-b", &[("types", "build"), ("serde", "normal")]),
        ],
        &[(0, &["types"]), (3, &["engine-a", "engine-b"])],
        &[],
    );
    let report = check(&ws).expect("a readable layer table");
    assert_eq!(report.violations, Vec::<String>::new());
    assert_eq!((report.crates, report.edges), (3, 3));
    assert_eq!(
        report.to_string(),
        "check-layers: OK, 3 crates, 3 internal edges, 0 violation(s)"
    );
}

#[test]
fn an_upward_edge_fails_even_as_a_dev_dependency() {
    let ws = workspace(
        &[("engine-a", &[("db", "dev")]), ("db", &[])],
        &[(3, &["engine-a"]), (4, &["db"])],
        &[],
    );
    assert_eq!(
        violations(&ws),
        [
            "UPWARD     engine-a (L3) -> db (L4) [dev]: a crate may depend only on its own layer or lower"
        ]
    );
}

#[test]
fn a_dev_only_tool_is_allowed_only_as_a_dev_dependency() {
    let layers: &[(u64, &[&str])] = &[(3, &["engine-a"])];
    let dev = workspace(
        &[("engine-a", &[("testkit", "dev")]), ("testkit", &[])],
        layers,
        &["testkit"],
    );
    assert!(violations(&dev).is_empty());

    let normal = workspace(
        &[("engine-a", &[("testkit", "normal")]), ("testkit", &[])],
        layers,
        &["testkit"],
    );
    assert_eq!(
        violations(&normal),
        ["TOOL       engine-a -> testkit [normal]: tools are never linked into a layered crate"]
    );

    let other_tool = workspace(
        &[("engine-a", &[("parity", "dev")]), ("parity", &[])],
        layers,
        &["parity"],
    );
    assert_eq!(violations(&other_tool).len(), 1);
}

#[test]
fn tools_may_link_anything() {
    let ws = workspace(
        &[
            (
                "parity",
                &[("app", "normal"), ("testkit", "normal"), ("sqlx", "normal")],
            ),
            ("app", &[]),
            ("testkit", &[]),
        ],
        &[(5, &["app"])],
        &["parity", "testkit"],
    );
    assert!(violations(&ws).is_empty());
}

#[test]
fn a_pure_crate_takes_a_forbidden_dependency_only_for_tests() {
    let ws = workspace(
        &[
            ("types", &[("sqlx", "dev")]),
            ("engine-a", &[("sqlx", "normal"), ("storage", "build")]),
            ("storage", &[("sqlx", "normal")]),
            ("types2", &[("sqlx", "normal")]),
        ],
        &[
            (0, &["types", "types2"]),
            (1, &["storage"]),
            (3, &["engine-a"]),
        ],
        &[],
    );
    assert_eq!(
        violations(&ws),
        [
            "IMPURE     engine-a -> sqlx [normal]: pure crates take values and return values",
            "IMPURE     engine-a -> storage [build]: pure crates take values and return values",
        ]
    );
}

#[test]
fn the_table_and_the_members_must_agree() {
    let ws = workspace(
        &[("types", &[]), ("db", &[]), ("stray", &[]), ("parity", &[])],
        &[
            (0, &["types", "gone"]),
            (4, &["db", "parity"]),
            (5, &["db"]),
        ],
        &["parity"],
    );
    let report = check(&ws).expect("a readable layer table");
    assert_eq!(
        report.violations,
        [
            "DUPLICATE  db is in L4 and L5",
            "DUPLICATE  parity is in a layer and in layer-rules.tools",
            "STALE      gone is in the layer table but is not a workspace member",
            "UNLAYERED  stray is in no layer and not a tool (add it to [workspace.metadata.layers])",
        ]
    );
    assert!(
        report
            .to_string()
            .ends_with("check-layers: FAILED, 4 crates, 0 internal edges, 4 violation(s)")
    );
}

#[test]
fn a_missing_or_malformed_table_is_an_error() {
    assert!(check(&json!({"workspace_members": [], "packages": [], "metadata": null})).is_err());
    let mut ws = workspace(&[("types", &[])], &[(0, &["types"])], &[]);
    ws["metadata"]["layers"][0]["level"] = json!("zero");
    assert!(check(&ws).is_err());
}

#[test]
fn this_workspace_obeys_its_own_table() {
    let ws = metadata(xtask::WORKSPACE_MANIFEST).expect("cargo metadata runs");
    let report = check(&ws).expect("a readable layer table");
    assert_eq!(report.violations, Vec::<String>::new());
    assert!(report.crates > 20 && report.edges > 50, "{report}");
}
