//! Pins the generated OpenAPI document, so a change to the public API surface
//! shows up as a reviewable diff at the moment it is made.
//!
//! This is the third instance of a pattern already in this directory:
//! `api_shape_test` pins the JSON shape of the `/static` payloads and
//! `bindings_shape_test` pins the generated TypeScript. All three exist for the
//! same reason: the contract with consumers is not checked by any compiler, so
//! it has to be checked by a snapshot.
//!
//! What lands in the diff:
//!
//! * A new documented endpoint, its parameters, and its responses.
//! * A renamed or removed field on any request or response schema. A line
//!   disappearing from the snapshot is a breaking change for every consumer of
//!   that field, the same rule as `api_shape_test`.
//! * A path or method changing, because both are read from the same
//!   `#[utoipa::path]` annotation that registers the route.
//!
//! Nothing is outside it. `every_route_is_documented` below asserts that no
//! route is registered with a plain `.route(..)`, so every endpoint the server
//! serves is an endpoint this snapshot pins.
//!
//! Refresh after an intentional change:
//!   UPDATE_OPENAPI=1 cargo test --test openapi_snapshot_test
//! and review the diff as part of the PR.

use std::fs;
use std::path::PathBuf;

use backend::app::server::api_parts;
use utoipa::openapi::path::{Operation, PathItem};

/// `PathItem` stores one `Option<Operation>` per HTTP method rather than a map,
/// so flatten it into the pairs these tests want to iterate.
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

/// Render the document the server serves, pretty-printed so the diff is
/// line-oriented rather than one enormous line.
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

/// Every documented operation must carry a description for each response it
/// declares, because an undescribed status code in the rendered docs is worse
/// than an absent one: it looks answered.
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

/// An operation that requires a credential must document what happens when the
/// credential is missing or bad.
///
/// Note the direction. The converse is NOT an invariant: `POST /api/login`
/// answers 401 for a wrong code while requiring no credential of its own, so
/// "documents a 401" does not imply "needs a token".
#[test]
fn documented_operations_declare_their_auth_posture() {
    let (_router, api) = api_parts();

    for (path, item) in &api.paths.paths {
        for (method, op) in operations(item) {
            // `security(.., ())` - an empty requirement among the options - is how
            // an endpoint says the credential is optional, so it is not a promise
            // that unauthenticated calls fail. Only a wholly-required credential
            // obliges a documented 401.
            //
            // Read through the serialized form: `SecurityRequirement`'s inner map
            // is private, and an empty requirement is exactly `{}` on the wire.
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

/// Every route is documented, and this is what keeps it that way.
///
/// The budget started at 155 and is now zero: there is no route in the API
/// reached through a plain `.route(..)`, so there is no endpoint the OpenAPI
/// document omits. That is a stronger guarantee than "the docs are current",
/// because it does not rely on anyone remembering: a handler registered with
/// `.route(..)` instead of `.routes(routes!(..))` has no `#[utoipa::path]` to
/// take its path and method from, and fails here.
///
/// Do not raise this to land an endpoint quickly. Annotating a handler is four
/// lines and `src/app/openapi.rs` lists the steps; a raised budget is a silent
/// undocumented endpoint, which is the thing this file exists to prevent.
#[test]
fn every_route_is_documented() {
    /// Zero, and it should stay zero.
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

/// `operationId` must be unique across the whole document.
///
/// utoipa derives it from the bare handler function name, and this API has six
/// modules that each export a `list`, three that export a `delete`, and so on.
/// Thirty-two of the operations collided before this test existed. That is not
/// cosmetic: client generators name their methods after `operationId`, so
/// duplicates collide or silently drop operations, and the OpenAPI spec itself
/// requires uniqueness.
///
/// The fix for a collision is an explicit `operation_id = "..."` on the handler,
/// qualified enough to stand alone. Only the colliding ones carry it, because
/// this test rather than a naming convention is what keeps the guarantee.
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

/// Every `$ref` in the document must resolve to something the document defines.
///
/// A dangling `$ref` makes the whole document invalid to every validator and
/// code generator, and it is easy to introduce without noticing: cutting a
/// recursive schema with `#[schema(no_recursion)]` stops utoipa collecting the
/// referenced type into `components` while the field still emits a `$ref` to
/// it. That is exactly how `PlanRecipe` went missing. The cure is to name the
/// type in `components(schemas(..))` in `src/app/openapi.rs`.
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

/// Every operation needs a summary, because the rendered docs fall back to the
/// raw path without one.
///
/// Scalar uses `summary` as the sidebar label. Three operations were missing
/// theirs and showed as `/api/login/cn/send-code` in a list of 180 prose
/// labels, which reads as a hole rather than as an endpoint. Nothing in the
/// spec is invalid without it, so only looking at the page catches this, which
/// is why it is pinned here instead.
///
/// The summary is the first line of the handler's doc comment. Write one.
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
