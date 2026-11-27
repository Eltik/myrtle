use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::Serialize;
use serde_json::Value;

const METHODS: &[&str] = &[
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

/// `/api/{server}/operators/{id}` -> `/api/{}/operators/{}`
fn normalize_template(path: &str) -> (String, Vec<String>) {
    let mut names = Vec::new();
    let segs: Vec<String> = path
        .split('/')
        .map(|s| {
            if let Some(inner) = s.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                names.push(inner.trim_start_matches('*').to_owned());
                if inner.starts_with('*') {
                    "{*}".to_owned()
                } else {
                    "{}".to_owned()
                }
            } else {
                s.to_owned()
            }
        })
        .collect();
    (segs.join("/"), names)
}

struct Operation<'a> {
    display: String,
    param_names: Vec<String>,
    op: &'a Value,
}

fn operations(doc: &Value) -> BTreeMap<String, Operation<'_>> {
    let mut out = BTreeMap::new();
    let Some(paths) = doc.get("paths").and_then(Value::as_object) else {
        return out;
    };
    for (path, item) in paths {
        let (norm, names) = normalize_template(path);
        for m in METHODS {
            if let Some(op) = item.get(*m) {
                let up = m.to_ascii_uppercase();
                out.insert(
                    format!("{up} {norm}"),
                    Operation {
                        display: format!("{up} {path}"),
                        param_names: names.clone(),
                        op,
                    },
                );
            }
        }
    }
    out
}

fn component_names(doc: &Value, kind: &str) -> BTreeSet<String> {
    doc.get("components")
        .and_then(|c| c.get(kind))
        .and_then(Value::as_object)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

fn collect_refs(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(map) => {
            if let Some(r) = map.get("$ref").and_then(Value::as_str) {
                out.insert(r.rsplit('/').next().unwrap_or(r).to_owned());
            }
            for (k, child) in map {
                if k != "$ref" {
                    collect_refs(child, out);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|i| collect_refs(i, out)),
        _ => {}
    }
}

fn response_refs(op: &Value) -> BTreeMap<(String, String), BTreeSet<String>> {
    let mut out = BTreeMap::new();
    let Some(responses) = op.get("responses").and_then(Value::as_object) else {
        return out;
    };
    for (status, resp) in responses {
        if let Some(r) = resp.get("$ref").and_then(Value::as_str) {
            let mut set = BTreeSet::new();
            set.insert(r.rsplit('/').next().unwrap_or(r).to_owned());
            out.insert((status.clone(), "$ref".to_owned()), set);
            continue;
        }
        match resp.get("content").and_then(Value::as_object) {
            Some(content) if !content.is_empty() => {
                for (ct, media) in content {
                    let mut set = BTreeSet::new();
                    if let Some(schema) = media.get("schema") {
                        collect_refs(schema, &mut set);
                    }
                    out.insert((status.clone(), ct.clone()), set);
                }
            }
            _ => {
                out.insert((status.clone(), "(no body)".to_owned()), BTreeSet::new());
            }
        }
    }
    out
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SupersetReport {
    pub old_operations: usize,
    pub new_operations: usize,
    pub old_schemas: usize,
    pub new_schemas: usize,
    pub missing_operations: Vec<String>,
    pub missing_schemas: Vec<String>,
    pub missing_responses: Vec<String>,
    pub missing_response_variants: Vec<String>,
    pub changed_response_refs: Vec<String>,
    pub renamed_path_params: Vec<String>,
}

impl SupersetReport {
    pub const fn ok(&self) -> bool {
        self.missing_operations.is_empty()
            && self.missing_schemas.is_empty()
            && self.missing_responses.is_empty()
            && self.missing_response_variants.is_empty()
            && self.changed_response_refs.is_empty()
    }

    pub fn render_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(
            s,
            "operations: old {}  new {}    schemas: old {}  new {}",
            self.old_operations, self.new_operations, self.old_schemas, self.new_schemas
        );
        let sections: [(&str, &Vec<String>); 6] = [
            ("operations missing from new", &self.missing_operations),
            ("schemas missing from new", &self.missing_schemas),
            (
                "components.responses missing from new",
                &self.missing_responses,
            ),
            (
                "response statuses or content types missing from new",
                &self.missing_response_variants,
            ),
            ("response schema refs changed", &self.changed_response_refs),
            (
                "path parameters renamed (warning only)",
                &self.renamed_path_params,
            ),
        ];
        for (title, items) in sections {
            if items.is_empty() {
                continue;
            }
            let _ = writeln!(s, "\n{title} ({}):", items.len());
            for i in items {
                let _ = writeln!(s, "  {i}");
            }
        }
        let _ = writeln!(s, "\nresult: {}", if self.ok() { "PASS" } else { "FAIL" });
        s
    }
}

pub fn check_superset(old: &Value, new: &Value) -> SupersetReport {
    let old_ops = operations(old);
    let new_ops = operations(new);
    let old_schemas = component_names(old, "schemas");
    let new_schemas = component_names(new, "schemas");
    let mut r = SupersetReport {
        old_operations: old_ops.len(),
        new_operations: new_ops.len(),
        old_schemas: old_schemas.len(),
        new_schemas: new_schemas.len(),
        ..SupersetReport::default()
    };
    r.missing_schemas = old_schemas.difference(&new_schemas).cloned().collect();
    r.missing_responses = component_names(old, "responses")
        .difference(&component_names(new, "responses"))
        .cloned()
        .collect();
    for (key, o) in &old_ops {
        let Some(n) = new_ops.get(key) else {
            r.missing_operations.push(o.display.clone());
            continue;
        };
        if o.param_names != n.param_names {
            r.renamed_path_params
                .push(format!("{} -> {}", o.display, n.display));
        }
        let new_refs = response_refs(n.op);
        for ((status, ct), refs) in response_refs(o.op) {
            match new_refs.get(&(status.clone(), ct.clone())) {
                None => r
                    .missing_response_variants
                    .push(format!("{} {status} {ct}", o.display)),
                Some(nr) if *nr != refs => r.changed_response_refs.push(format!(
                    "{} {status} {ct}: [{}] -> [{}]",
                    o.display,
                    refs.iter().cloned().collect::<Vec<_>>().join(", "),
                    nr.iter().cloned().collect::<Vec<_>>().join(", ")
                )),
                Some(_) => {}
            }
        }
    }
    r
}
