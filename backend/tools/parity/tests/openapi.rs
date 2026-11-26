use myrtle_parity::openapi::check_superset;
use serde_json::{Value, json};

fn old() -> Value {
    json!({
        "paths": {
            "/api/operators/{id}": {"get": {"responses": {
                "200": {"content": {"application/json": {"schema": {"$ref": "#/components/schemas/Operator"}}}},
                "404": {"$ref": "#/components/responses/NotFound"}
            }}},
            "/api/operators/index": {"get": {"responses": {
                "200": {"content": {"application/json": {"schema": {"type": "array", "items": {"$ref": "#/components/schemas/OperatorIndexEntry"}}}}}
            }}},
            "/api/tier-lists": {"post": {"responses": {"201": {"description": "created"}}}}
        },
        "components": {
            "schemas": {"Operator": {}, "OperatorIndexEntry": {}},
            "responses": {"NotFound": {}}
        }
    })
}

#[test]
fn identical_documents_pass() {
    let r = check_superset(&old(), &old());
    assert!(r.ok(), "{}", r.render_text());
    assert_eq!((r.old_operations, r.old_schemas), (3, 2));
}

#[test]
fn additions_pass() {
    let mut new = old();
    new["paths"]["/api/new"] = json!({"get": {"responses": {}}});
    new["components"]["schemas"]["Extra"] = json!({});
    new["paths"]["/api/operators/{id}"]["get"]["responses"]["500"] = json!({"description": "x"});
    assert!(check_superset(&old(), &new).ok());
}

#[test]
fn missing_operation_and_schema_fail() {
    let mut new = old();
    new["paths"]
        .as_object_mut()
        .unwrap()
        .remove("/api/tier-lists");
    new["components"]["schemas"]
        .as_object_mut()
        .unwrap()
        .remove("OperatorIndexEntry");
    let r = check_superset(&old(), &new);
    assert!(!r.ok());
    assert_eq!(r.missing_operations, ["POST /api/tier-lists"]);
    assert_eq!(r.missing_schemas, ["OperatorIndexEntry"]);
}

#[test]
fn method_matters() {
    let mut new = old();
    let op = new["paths"]["/api/tier-lists"]["post"].take();
    new["paths"]["/api/tier-lists"] = json!({"put": op});
    let r = check_superset(&old(), &new);
    assert_eq!(r.missing_operations, ["POST /api/tier-lists"]);
}

#[test]
fn renamed_response_ref_fails_even_when_both_schemas_exist() {
    let mut new = old();
    new["components"]["schemas"]["OperatorV2"] = json!({});
    new["paths"]["/api/operators/{id}"]["get"]["responses"]["200"]["content"]["application/json"]
        ["schema"]["$ref"] = json!("#/components/schemas/OperatorV2");
    let r = check_superset(&old(), &new);
    assert!(!r.ok());
    assert_eq!(
        r.changed_response_refs,
        ["GET /api/operators/{id} 200 application/json: [Operator] -> [OperatorV2]"]
    );
}

#[test]
fn array_item_ref_and_shared_response_are_tracked() {
    let mut new = old();
    new["paths"]["/api/operators/index"]["get"]["responses"]["200"]["content"]["application/json"]
        ["schema"]["items"]["$ref"] = json!("#/components/schemas/Operator");
    new["paths"]["/api/operators/{id}"]["get"]["responses"]["404"] =
        json!({"$ref": "#/components/responses/Gone"});
    let r = check_superset(&old(), &new);
    assert_eq!(r.changed_response_refs.len(), 2, "{}", r.render_text());
}

#[test]
fn dropped_status_or_content_type_fails() {
    let mut new = old();
    new["paths"]["/api/operators/{id}"]["get"]["responses"]
        .as_object_mut()
        .unwrap()
        .remove("404");
    let r = check_superset(&old(), &new);
    assert_eq!(
        r.missing_response_variants,
        ["GET /api/operators/{id} 404 $ref"]
    );
}

#[test]
fn path_param_rename_only_warns() {
    let mut new = old();
    let item = new["paths"]
        .as_object_mut()
        .unwrap()
        .remove("/api/operators/{id}")
        .unwrap();
    new["paths"]["/api/operators/{char_id}"] = item;
    let r = check_superset(&old(), &new);
    assert!(r.ok(), "{}", r.render_text());
    assert_eq!(
        r.renamed_path_params,
        ["GET /api/operators/{id} -> GET /api/operators/{char_id}"]
    );
}

#[test]
fn normalize_handles_wildcards() {
    let old = json!({"paths": {"/api/{server}/assets/{*path}": {"get": {}}}});
    let renamed = json!({"paths": {"/api/{s}/assets/{*p}": {"get": {}}}});
    let r = check_superset(&old, &renamed);
    assert!(r.ok(), "{}", r.render_text());
    assert_eq!(
        r.renamed_path_params,
        ["GET /api/{server}/assets/{*path} -> GET /api/{s}/assets/{*p}"]
    );
    let single = json!({"paths": {"/api/{server}/assets/{path}": {"get": {}}}});
    assert_eq!(
        check_superset(&old, &single).missing_operations,
        ["GET /api/{server}/assets/{*path}"]
    );
}
