use std::fs;
use std::path::Path;

use serde_json::json;
use xtask::bindings::{GENERATED, exporting_crates, files, impossible_types};

#[test]
fn only_crates_that_depend_on_ts_rs_export() {
    let metadata = json!({"packages": [
        {"name": "types", "dependencies": [{"name": "serde"}, {"name": "ts-rs"}]},
        {"name": "db", "dependencies": [{"name": "sqlx"}]},
        {"name": "app", "dependencies": [{"name": "ts-rs"}]},
        {"name": "xtask", "dependencies": []},
    ]});
    assert_eq!(exporting_crates(&metadata), ["types", "app"]);
}

#[test]
fn a_bigint_or_any_binding_is_reported() {
    assert_eq!(
        impossible_types("export type A = { id: bigint; raw: any };"),
        ["bigint", "any"]
    );
    assert_eq!(
        impossible_types("export type A = { n: Array<bigint | null> };"),
        ["bigint"]
    );
}

#[test]
fn comments_and_longer_identifiers_are_not_reported() {
    let binding = "/**\n * any bigint, in a doc comment\n */\n\
                   export type Company = { many: number; anything: string; bigint_like: number }; // any";
    assert!(impossible_types(binding).is_empty());
    assert!(impossible_types("/* unterminated bigint").is_empty());
}

#[test]
fn the_committed_bindings_hold_no_impossible_types() {
    let generated =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../frontend/{GENERATED}"));
    let bindings = files(&generated).expect("the generated directory is readable");
    assert!(bindings.len() > 500, "only {} bindings", bindings.len());
    for path in bindings {
        let binding = fs::read_to_string(&path).expect("a readable binding");
        assert_eq!(
            impossible_types(&binding),
            Vec::<&str>::new(),
            "{}",
            path.display()
        );
    }
}
