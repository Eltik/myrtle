//! Pins the generated OpenAPI document so an API change shows up as a reviewable diff.
//! Sibling of `api_shape_test` (`/static` JSON) and `bindings_shape_test` (generated
//! TS): no compiler checks the consumer contract. `every_route_is_documented` keeps
//! every route inside the document.
//!
//! Refresh after an intentional change, and review the diff in the PR:
//!   UPDATE_OPENAPI=1 cargo test --test openapi_snapshot_test

use std::fs;
use std::path::PathBuf;

use backend::app::server::api_parts;
use utoipa::openapi::path::{Operation, PathItem};

/// `PathItem` holds one `Option<Operation>` per method, not a map.
fn operations(item: &PathItem) -> Vec<(&'static str, &Operation)> {
    [
        ("get", &item.get),
        ("put", &item.put),
        ("post", &item.post),
        ("delete", &item.delete),
        ("options", &item.options),
        ("head", &item.head),
        ("patch", &item.patch),
        ("trace", &item.trace),
    ]
    .into_iter()
    .filter_map(|(method, op)| op.as_ref().map(|op| (method, op)))
    .collect()
}

fn snapshot_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/openapi.json")
}

/// Pretty-printed so the snapshot diffs line by line.
fn render() -> String {
    let (_router, api) = api_parts();
    let mut json = api
        .to_pretty_json()
        .expect("the OpenAPI document serializes");
    json.push('\n');
    json
}

#[test]
fn openapi_document_matches_snapshot() {
    let path = snapshot_path();
    let current = render();

    if std::env::var("UPDATE_OPENAPI").is_ok() {
        fs::create_dir_all(path.parent().expect("snapshot dir")).expect("create snapshot dir");
        fs::write(&path, &current).expect("write snapshot");
        return;
    }

    let committed = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {}: {e}\n\
             If this is the first run, create it with:\n  \
             UPDATE_OPENAPI=1 cargo test --test openapi_snapshot_test",
            path.display()
        )
    });

    assert_eq!(
        committed, current,
        "\nThe OpenAPI document no longer matches tests/snapshots/openapi.json.\n\
         If the change is intended, refresh it with:\n  \
         UPDATE_OPENAPI=1 cargo test --test openapi_snapshot_test\n\
         and review the diff - a path or field disappearing is a breaking change.\n"
    );
}

/// An undescribed status code in the rendered docs is worse than an absent one:
/// it looks answered.
#[test]
fn every_documented_response_has_a_description() {
    let (_router, api) = api_parts();
    let mut missing = Vec::new();

    for (path, item) in &api.paths.paths {
        for (method, op) in operations(item) {
            for (status, response) in &op.responses.responses {
                let described = match response {
                    utoipa::openapi::RefOr::T(r) => !r.description.trim().is_empty(),
                    // A `$ref` points at a component response, which carries its
                    // own description; `openapi.rs` writes one for each.
                    utoipa::openapi::RefOr::Ref(_) => true,
                };
                assert!(
                    described,
                    "{method} {path} declares {status} with no description"
                );
                if !described {
                    missing.push(format!("{method} {path} {status}"));
                }
            }
        }
    }

    assert!(missing.is_empty(), "undescribed responses: {missing:?}");
}

/// A credentialed operation must document its 401. Not the converse:
/// `POST /api/login` answers 401 for a wrong code but needs no token.
#[test]
fn documented_operations_declare_their_auth_posture() {
    let (_router, api) = api_parts();

    for (path, item) in &api.paths.paths {
        for (method, op) in operations(item) {
            // `security(.., ())` (an empty requirement among the options) marks the
            // credential optional, so only a wholly-required one needs a 401.
            // Read via serde: `SecurityRequirement`'s map is private, and an empty
            // requirement is `{}` on the wire.
            let requires_credential = op.security.as_ref().is_some_and(|reqs| {
                !reqs.is_empty()
                    && reqs.iter().all(|req| {
                        serde_json::to_value(req)
                            .ok()
                            .and_then(|v| v.as_object().map(|o| !o.is_empty()))
                            .unwrap_or(false)
                    })
            });
            let documents_401 = op.responses.responses.contains_key("401");
            assert!(
                !requires_credential || documents_401,
                "{method} {path} requires a credential but documents no 401, \
                 so a caller cannot tell what a missing or expired token looks like"
            );
        }
    }
}

/// A plain `.route(..)` has no `#[utoipa::path]`, so the document would omit it.
/// The budget was 155 and is now zero; don't raise it to land an endpoint.
/// `src/app/openapi.rs` lists the four steps to annotate a handler.
#[test]
fn every_route_is_documented() {
    const UNDOCUMENTED_ROUTE_BUDGET: usize = 0;

    let routes_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/app/routes");
    let mut undocumented = 0usize;
    let mut documented = 0usize;

    let mut queue = vec![routes_dir];
    while let Some(next) = queue.pop() {
        for entry in fs::read_dir(&next).expect("routes dir is readable") {
            let path = entry.expect("readable dir entry").path();
            if path.is_dir() {
                queue.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let src = fs::read_to_string(&path).expect("route module is readable");
            for line in src.lines() {
                // Doc comments in this tree spell both forms while explaining
                // them, which would otherwise inflate both counts.
                if line.trim_start().starts_with("//") {
                    continue;
                }
                // `.routes(routes!(..))` is the documented form and does not
                // contain the `.route(` substring, so the counts do not overlap.
                undocumented += line.matches(".route(").count();
                documented += line.matches(".routes(routes!(").count();
            }
        }
    }

    // Spelled `==` rather than `<=`: with the budget at zero, `<= 0` on a usize
    // is always-or-never true and clippy rejects it.
    assert!(
        undocumented == UNDOCUMENTED_ROUTE_BUDGET,
        "{undocumented} route(s) are registered with plain `.route(..)` and so are absent \
         from the OpenAPI document; the budget is {UNDOCUMENTED_ROUTE_BUDGET}.\n\
         Annotate the handler with `#[utoipa::path(..)]` and register it with \
         `.routes(routes!(..))` - see `src/app/openapi.rs` for the four steps.\n\
         ({documented} routes are currently documented.)"
    );
}

/// utoipa derives `operationId` from the bare handler name, and six modules export
/// a `list`, three a `delete`: 32 operations collided before this test. Client
/// generators name methods after it, so duplicates collide or drop operations, and
/// the spec requires uniqueness. Fix with an explicit `operation_id = "..."` on the
/// colliding handler only; this test, not a naming convention, keeps the guarantee.
#[test]
fn operation_ids_are_unique() {
    let (_router, api) = api_parts();
    let mut seen: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for (path, item) in &api.paths.paths {
        for (method, op) in operations(item) {
            let id = op
                .operation_id
                .clone()
                .unwrap_or_else(|| panic!("{method} {path} has no operationId"));
            seen.entry(id).or_default().push(format!("{method} {path}"));
        }
    }

    let mut collisions: Vec<_> = seen.iter().filter(|(_, v)| v.len() > 1).collect();
    collisions.sort_by_key(|(id, _)| (*id).clone());

    assert!(
        collisions.is_empty(),
        "duplicate operationIds: {collisions:#?}\n\
         Give each colliding handler an explicit `operation_id = \"...\"` in its \
         `#[utoipa::path(..)]`."
    );
}

/// A dangling `$ref` invalidates the whole document for validators and generators.
/// `#[schema(no_recursion)]` stops utoipa collecting the type into `components`
/// while the field still emits the `$ref`; that is how `PlanRecipe` went missing.
/// Fix: name the type in `components(schemas(..))` in `src/app/openapi.rs`.
#[test]
fn every_ref_resolves() {
    let (_router, api) = api_parts();
    let doc: serde_json::Value =
        serde_json::to_value(&api).expect("the document serializes to JSON");

    fn collect(node: &serde_json::Value, out: &mut Vec<String>) {
        match node {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    if k == "$ref" {
                        if let Some(r) = v.as_str() {
                            out.push(r.to_owned());
                        }
                    } else {
                        collect(v, out);
                    }
                }
            }
            serde_json::Value::Array(items) => {
                for v in items {
                    collect(v, out);
                }
            }
            _ => {}
        }
    }

    let mut refs = Vec::new();
    collect(&doc, &mut refs);
    refs.sort();
    refs.dedup();
    assert!(
        !refs.is_empty(),
        "a document with no $ref at all is suspicious"
    );

    let mut dangling = Vec::new();
    for r in &refs {
        let Some(rest) = r.strip_prefix("#/") else {
            dangling.push(format!("{r} (not a local reference)"));
            continue;
        };
        let mut cur = &doc;
        let mut ok = true;
        for segment in rest.split('/') {
            match cur.get(segment) {
                Some(next) => cur = next,
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if !ok {
            dangling.push(r.clone());
        }
    }

    assert!(
        dangling.is_empty(),
        "{} of {} $refs do not resolve: {dangling:#?}\n\
         Register the missing type in `components(schemas(..))` in `src/app/openapi.rs`.",
        dangling.len(),
        refs.len()
    );
}

/// Scalar labels the sidebar with `summary` and falls back to the raw path. Three
/// operations lacked one and showed as `/api/login/cn/send-code` among 180 prose
/// labels. The spec is valid without it, so only this test catches it. The summary
/// is the first line of the handler's doc comment.
#[test]
fn every_operation_has_a_summary() {
    let (_router, api) = api_parts();
    let mut missing = Vec::new();

    for (path, item) in &api.paths.paths {
        for (method, op) in operations(item) {
            let has_summary = op.summary.as_ref().is_some_and(|s| !s.trim().is_empty());
            if !has_summary {
                missing.push(format!("{method} {path}"));
            }
        }
    }
    missing.sort();

    assert!(
        missing.is_empty(),
        "operations with no summary: {missing:#?}\n\
         Add a doc comment to the handler; its first line becomes the summary."
    );
}
