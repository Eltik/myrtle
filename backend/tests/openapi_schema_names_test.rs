//! `OpenAPI` component schemas are keyed by bare type name, so two
//! `ToSchema` types with one name collapse into a single schema and every
//! operation that names either shows whichever was registered last. That hid
//! the translation grant body (it has a `locale`) behind the tier-list one.
//!
//! The document cannot show a collision after the fact, so this scans the
//! source: every `ToSchema` struct or enum, under its `#[schema(as = ...)]`
//! name when it has one, must have a unique name. `KNOWN` lists the collisions
//! that predate this test; shrink it when one is renamed, never grow it.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Pre-existing collisions: name -> the files that declare it.
const KNOWN: &[(&str, &[&str])] = &[
    (
        "AuditLogEntry",
        &["app/services/operator_notes.rs", "database/models/audit.rs"],
    ),
    (
        "Grid",
        &["app/services/grid.rs", "core/gamedata/types/range.rs"],
    ),
    (
        "UnlockCondition",
        &[
            "core/gamedata/types/operator.rs",
            "core/gamedata/types/stage.rs",
        ],
    ),
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("readable src dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The type name after `pub struct` / `pub enum` (any visibility), if the line
/// declares one.
fn declared_name(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("pub")?;
    let rest = rest
        .strip_prefix(|c: char| c == '(')
        .map_or(rest, |r| r.split_once(')').map_or(r, |(_, after)| after));
    let rest = rest.trim_start();
    let rest = rest
        .strip_prefix("struct ")
        .or_else(|| rest.strip_prefix("enum "))?;
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    Some(&rest[..end])
}

fn schema_alias(attr: &str) -> Option<&str> {
    let rest = attr.split_once("schema(")?.1;
    let rest = rest.trim_start().strip_prefix("as")?.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':'))
        .unwrap_or(rest.len());
    Some(rest[..end].rsplit("::").next().unwrap_or(&rest[..end]))
}

fn schema_names() -> BTreeMap<String, Vec<String>> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut names: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in files {
        let rel = file
            .strip_prefix(&src)
            .expect("under src")
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&file).expect("readable source");
        let mut attrs: Vec<&str> = Vec::new();
        for line in text.lines().map(str::trim) {
            if line.starts_with("#[") {
                attrs.push(line);
                continue;
            }
            if line.starts_with("//") || line.is_empty() {
                continue;
            }
            if let Some(name) = declared_name(line)
                && attrs.iter().any(|a| a.contains("ToSchema"))
            {
                let name = attrs.iter().find_map(|a| schema_alias(a)).unwrap_or(name);
                names.entry(name.to_owned()).or_default().push(rel.clone());
            }
            attrs.clear();
        }
    }
    names
}

#[test]
fn every_schema_name_is_unique() {
    let names = schema_names();
    assert!(
        names.len() > 100,
        "the scan found only {} schemas; the parser no longer matches the source",
        names.len()
    );
    let mut collisions: Vec<(String, Vec<String>)> = names
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(name, mut files)| {
            files.sort();
            (name, files)
        })
        .collect();
    collisions.sort();
    let known: Vec<(String, Vec<String>)> = KNOWN
        .iter()
        .map(|(name, files)| {
            (
                (*name).to_owned(),
                files.iter().map(|f| (*f).to_owned()).collect(),
            )
        })
        .collect();
    assert_eq!(
        collisions, known,
        "ToSchema types sharing a name: rename one or give it #[schema(as = ...)]"
    );
}

#[test]
fn the_translation_grant_body_documents_its_locale() {
    let (_router, api) = backend::app::server::api_parts();
    let json = serde_json::to_value(&api).expect("the document serializes");
    let schemas = &json["components"]["schemas"];
    let required = |name: &str| -> Vec<String> {
        serde_json::from_value(schemas[name]["required"].clone()).unwrap_or_default()
    };
    assert_eq!(
        required("TranslationGrantRequest"),
        ["locale", "user_id", "permission"]
    );
    assert_eq!(required("GrantRequest"), ["user_id", "permission"]);
    for path in [
        "/api/admin/i18n/permissions",
        "/api/admin/i18n/permissions/revoke",
    ] {
        assert_eq!(
            json["paths"][path]["post"]["requestBody"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/TranslationGrantRequest",
            "{path}"
        );
    }
}

#[test]
fn tier_list_stats_documents_the_engagement_counters() {
    let (_router, api) = backend::app::server::api_parts();
    let json = serde_json::to_value(&api).expect("the document serializes");
    let schemas = &json["components"]["schemas"];
    let required = |name: &str| -> Vec<String> {
        serde_json::from_value(schemas[name]["required"].clone()).unwrap_or_default()
    };
    // `GET /tier-lists/{slug}/stats` and `TierListDetail.stats` serve the
    // per-list row; the site-wide counters in `StatsResponse` are their own schema.
    assert!(required("TierListStats").contains(&"view_count".to_owned()));
    assert_eq!(
        required("TierListSiteStats"),
        ["total", "active", "totalVersions", "totalPlacements"]
    );
    assert_eq!(
        json["paths"]["/api/tier-lists/{slug}/stats"]["get"]["responses"]["200"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/TierListStats"
    );
    assert_eq!(
        schemas["StatsResponse"]["properties"]["tierLists"]["$ref"],
        "#/components/schemas/TierListSiteStats"
    );
}
