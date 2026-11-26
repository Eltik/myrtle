//! The request corpus: JSON lines, byte-identical for the same inputs and seed

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::io::{BufRead, BufWriter, Write};
use std::path::Path;

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::compare::sha256_hex;
use crate::ids::IdSet;
use crate::rng::{SplitMix64, fnv1a64, sample_indices};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Openapi,
    Nginx,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpusEntry {
    /// First 16 hex chars of SHA-256 over method, path and role
    pub id: String,
    pub method: String,
    /// `OpenAPI` path template
    pub route: String,
    /// Percent-encoded, query included
    pub path: String,
    /// Token read from `PARITY_TOKEN_<ROLE>` at run time, never stored
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub source: Source,
}

impl CorpusEntry {
    pub fn new(route: &str, path: &str, role: Option<&str>, source: Source) -> Self {
        let method = "GET";
        Self {
            id: entry_id(method, path, role),
            method: method.to_owned(),
            route: route.to_owned(),
            path: path.to_owned(),
            role: role.map(str::to_owned),
            source,
        }
    }
}

pub fn entry_id(method: &str, path: &str, role: Option<&str>) -> String {
    let key = format!("{method} {path} {}", role.unwrap_or(""));
    sha256_hex(key.as_bytes())[..16].to_owned()
}

pub fn read_corpus(path: &Path) -> Result<Vec<CorpusEntry>> {
    let file =
        std::fs::File::open(path).with_context(|| format!("opening corpus {}", path.display()))?;
    let mut out = Vec::new();
    for (n, line) in std::io::BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: CorpusEntry = serde_json::from_str(&line)
            .with_context(|| format!("{}:{}: not a corpus entry", path.display(), n + 1))?;
        out.push(entry);
    }
    Ok(out)
}

pub fn write_corpus(path: &Path, entries: &[CorpusEntry]) -> Result<()> {
    let file = std::fs::File::create(path)
        .with_context(|| format!("creating corpus {}", path.display()))?;
    let mut w = BufWriter::new(file);
    for e in entries {
        serde_json::to_writer(&mut w, e)?;
        w.write_all(b"\n")?;
    }
    w.flush()?;
    Ok(())
}

const fn is_unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~')
}

/// Keeps RFC 3986 `pchar`, so a skin id's `#` is encoded
pub fn encode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        if is_unreserved(b)
            || matches!(
                b,
                b'!' | b'$'
                    | b'&'
                    | b'\''
                    | b'('
                    | b')'
                    | b'*'
                    | b'+'
                    | b','
                    | b';'
                    | b'='
                    | b':'
                    | b'@'
            )
        {
            out.push(b as char);
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

/// Unreserved bytes only
pub fn encode_query(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        if is_unreserved(b) {
            out.push(b as char);
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TSeg {
    Lit(String),
    Param(String),
    Rest(String),
}

fn parse_template(t: &str) -> Vec<TSeg> {
    t.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| {
            if let Some(inner) = s.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                inner.strip_prefix('*').map_or_else(
                    || TSeg::Param(inner.to_owned()),
                    |r| TSeg::Rest(r.to_owned()),
                )
            } else {
                TSeg::Lit(s.to_owned())
            }
        })
        .collect()
}

fn render(template: &[TSeg], values: &HashMap<&str, &str>) -> String {
    let mut out = String::new();
    for seg in template {
        out.push('/');
        match seg {
            TSeg::Lit(l) => out.push_str(l),
            TSeg::Param(p) => out.push_str(&encode_segment(
                values.get(p.as_str()).copied().unwrap_or(""),
            )),
            TSeg::Rest(p) => {
                let v = values.get(p.as_str()).copied().unwrap_or("");
                let encoded: Vec<String> = v
                    .trim_start_matches('/')
                    .split('/')
                    .map(encode_segment)
                    .collect();
                out.push_str(&encoded.join("/"));
            }
        }
    }
    out
}

/// Most literal template first: `/api/operators/index` before `/api/operators/{id}`
#[derive(Debug, Clone)]
pub struct RouteMatcher {
    routes: Vec<(String, Vec<TSeg>)>,
}

impl RouteMatcher {
    pub fn new<I: IntoIterator<Item = String>>(templates: I) -> Self {
        let mut routes: Vec<(String, Vec<TSeg>)> = templates
            .into_iter()
            .map(|t| {
                let segs = parse_template(&t);
                (t, segs)
            })
            .collect();
        routes.sort_by(|(ta, a), (tb, b)| {
            let lits = |s: &[TSeg]| s.iter().filter(|x| matches!(x, TSeg::Lit(_))).count();
            let rest = |s: &[TSeg]| s.iter().any(|x| matches!(x, TSeg::Rest(_)));
            lits(b)
                .cmp(&lits(a))
                .then(rest(a).cmp(&rest(b)))
                .then(ta.cmp(tb))
        });
        Self { routes }
    }

    /// GET templates only
    pub fn from_openapi(doc: &Value) -> Self {
        let templates = doc
            .get("paths")
            .and_then(Value::as_object)
            .map(|paths| {
                paths
                    .iter()
                    .filter(|(_, item)| item.get("get").is_some())
                    .map(|(p, _)| p.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Self::new(templates)
    }

    pub fn route_for(&self, path: &str) -> Option<&str> {
        let path = path.split('?').next().unwrap_or(path);
        let segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        self.routes
            .iter()
            .find(|(_, t)| template_matches(t, &segs))
            .map(|(name, _)| name.as_str())
    }
}

fn template_matches(t: &[TSeg], segs: &[&str]) -> bool {
    match t.split_first() {
        None => segs.is_empty(),
        Some((TSeg::Rest(_), _)) => !segs.is_empty(),
        Some((TSeg::Lit(l), rest)) => {
            segs.first().is_some_and(|s| s == l) && template_matches(rest, &segs[1..])
        }
        Some((TSeg::Param(_), rest)) => !segs.is_empty() && template_matches(rest, &segs[1..]),
    }
}

#[derive(Debug, Clone)]
pub struct ExpandOptions {
    /// Per route and role; `None` takes every combination
    pub sample: Option<u64>,
    pub seed: u64,
    /// Each GET that takes a bearer token is emitted once per role
    pub roles: Vec<String>,
    /// Empty keeps every route
    pub include: Vec<Regex>,
    pub exclude: Vec<Regex>,
}

impl Default for ExpandOptions {
    fn default() -> Self {
        Self {
            sample: None,
            seed: crate::DEFAULT_SEED,
            roles: Vec::new(),
            include: Vec::new(),
            exclude: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoutePlan {
    pub route: String,
    /// Before sampling, per role variant
    pub combinations: u128,
    pub emitted: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Expansion {
    pub entries: Vec<CorpusEntry>,
    pub plans: Vec<RoutePlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Access {
    Public,
    /// Anonymous works, and answers differently when signed in
    Optional,
    Required,
}

fn access_of(doc: &Value, op: &Value) -> Access {
    let sec = op.get("security").or_else(|| doc.get("security"));
    let Some(list) = sec.and_then(Value::as_array) else {
        return Access::Public;
    };
    if list.is_empty() {
        return Access::Public;
    }
    let anonymous_ok = list
        .iter()
        .any(|req| req.as_object().is_some_and(serde_json::Map::is_empty));
    let bearer = list.iter().any(|req| req.get("bearer_auth").is_some());
    match (anonymous_ok, bearer) {
        (true, true) => Access::Optional,
        (true, false) => Access::Public,
        (false, _) => Access::Required,
    }
}

fn resolve_ref<'a>(doc: &'a Value, v: &'a Value) -> &'a Value {
    let Some(r) = v.get("$ref").and_then(Value::as_str) else {
        return v;
    };
    let Some(pointer) = r.strip_prefix('#') else {
        return v;
    };
    doc.pointer(pointer).unwrap_or(v)
}

fn schema_enum(doc: &Value, param: &Value) -> Option<Vec<String>> {
    let schema = resolve_ref(doc, param.get("schema")?);
    let values = schema.get("enum")?.as_array()?;
    Some(
        values
            .iter()
            .map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_owned))
            .collect(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Loc {
    Path,
    Query,
}

struct Dim {
    name: String,
    loc: Loc,
    values: Vec<String>,
}

/// Operation level overrides path-item level by (name, in)
fn parameters<'a>(doc: &'a Value, item: &'a Value, op: &'a Value) -> Vec<&'a Value> {
    let mut out: Vec<&Value> = Vec::new();
    for source in [item.get("parameters"), op.get("parameters")] {
        for p in source.and_then(Value::as_array).into_iter().flatten() {
            let p = resolve_ref(doc, p);
            let key = (p.get("name"), p.get("in"));
            out.retain(|q| (q.get("name"), q.get("in")) != key);
            out.push(p);
        }
    }
    out
}

fn route_seed(seed: u64, route: &str) -> u64 {
    seed ^ fnv1a64(route.as_bytes())
}

pub fn expand_openapi(doc: &Value, ids: &IdSet, opts: &ExpandOptions) -> Expansion {
    let mut out = Expansion::default();
    let Some(paths) = doc.get("paths").and_then(Value::as_object) else {
        return out;
    };
    let mut routes: Vec<&String> = paths.keys().collect();
    routes.sort();
    for route in routes {
        let item = &paths[route];
        let Some(op) = item.get("get") else {
            continue;
        };
        let plan = expand_route(doc, ids, opts, route, item, op, &mut out.entries);
        out.plans.push(plan);
    }
    out.entries
        .sort_by(|a, b| (&a.route, &a.path, &a.role).cmp(&(&b.route, &b.path, &b.role)));
    out.entries.dedup_by(|a, b| a.id == b.id);
    out
}

fn skip(route: &str, reason: String) -> RoutePlan {
    RoutePlan {
        route: route.to_owned(),
        combinations: 0,
        emitted: 0,
        skipped: Some(reason),
    }
}

fn expand_route(
    doc: &Value,
    ids: &IdSet,
    opts: &ExpandOptions,
    route: &str,
    item: &Value,
    op: &Value,
    sink: &mut Vec<CorpusEntry>,
) -> RoutePlan {
    if !opts.include.is_empty() && !opts.include.iter().any(|r| r.is_match(route)) {
        return skip(route, "not matched by --include".to_owned());
    }
    if let Some(r) = opts.exclude.iter().find(|r| r.is_match(route)) {
        return skip(route, format!("excluded by --exclude {}", r.as_str()));
    }
    let roles: Vec<Option<&str>> = match access_of(doc, op) {
        Access::Public => vec![None],
        Access::Optional => std::iter::once(None)
            .chain(opts.roles.iter().map(|r| Some(r.as_str())))
            .collect(),
        Access::Required if opts.roles.is_empty() => {
            return skip(
                route,
                "requires a signed-in session; pass --role".to_owned(),
            );
        }
        Access::Required => opts.roles.iter().map(|r| Some(r.as_str())).collect(),
    };

    let template = parse_template(route);
    let params = parameters(doc, item, op);
    let mut dims: Vec<Dim> = Vec::new();
    for seg in &template {
        let (TSeg::Param(name) | TSeg::Rest(name)) = seg else {
            continue;
        };
        let spec = params.iter().find(|p| {
            p.get("in").and_then(Value::as_str) == Some("path")
                && p.get("name").and_then(Value::as_str) == Some(name)
        });
        match values_for(doc, ids, route, name, spec.copied()) {
            Ok(values) => dims.push(Dim {
                name: name.clone(),
                loc: Loc::Path,
                values,
            }),
            Err(reason) => return skip(route, reason),
        }
    }
    for p in &params {
        let required = p.get("required").and_then(Value::as_bool).unwrap_or(false);
        if p.get("in").and_then(Value::as_str) != Some("query") || !required {
            continue;
        }
        let Some(name) = p.get("name").and_then(Value::as_str) else {
            continue;
        };
        match values_for(doc, ids, route, name, Some(p)) {
            Ok(values) => dims.push(Dim {
                name: name.to_owned(),
                loc: Loc::Query,
                values,
            }),
            Err(reason) => return skip(route, format!("required query: {reason}")),
        }
    }

    let combinations: u128 = dims
        .iter()
        .map(|d| d.values.len() as u128)
        .fold(1_u128, u128::saturating_mul);
    let population = u64::try_from(combinations).unwrap_or(u64::MAX);
    let indices: Vec<u64> = match opts.sample {
        Some(k) if population > k => {
            let mut rng = SplitMix64::new(route_seed(opts.seed, route));
            sample_indices(population, k, &mut rng)
        }
        _ => (0..population).collect(),
    };

    let mut emitted = 0;
    for index in indices {
        let mut rem = index;
        let mut chosen: Vec<&str> = vec![""; dims.len()];
        for (slot, dim) in dims.iter().enumerate().rev() {
            let n = dim.values.len() as u64;
            chosen[slot] = &dim.values[(rem % n) as usize];
            rem /= n; // last dimension fastest, so the server stays fixed
        }
        let path_values: HashMap<&str, &str> = dims
            .iter()
            .zip(&chosen)
            .filter(|(d, _)| d.loc == Loc::Path)
            .map(|(d, v)| (d.name.as_str(), *v))
            .collect();
        let mut path = render(&template, &path_values);
        let query: Vec<String> = dims
            .iter()
            .zip(&chosen)
            .filter(|(d, _)| d.loc == Loc::Query)
            .map(|(d, v)| format!("{}={}", encode_query(&d.name), encode_query(v)))
            .collect();
        if !query.is_empty() {
            path.push('?');
            path.push_str(&query.join("&"));
        }
        for role in &roles {
            sink.push(CorpusEntry::new(route, &path, *role, Source::Openapi));
            emitted += 1;
        }
    }
    RoutePlan {
        route: route.to_owned(),
        combinations,
        emitted,
        skipped: None,
    }
}

fn values_for(
    doc: &Value,
    ids: &IdSet,
    route: &str,
    name: &str,
    spec: Option<&Value>,
) -> Result<Vec<String>, String> {
    let has_binding = ids
        .bindings
        .iter()
        .any(|b| b.route == route && b.param == name);
    if !has_binding // a binding beats the schema's enum
        && let Some(values) = spec.and_then(|p| schema_enum(doc, p))
        && !values.is_empty()
    {
        return Ok(values);
    }
    let Some(kind) = ids.kind_for(route, name) else {
        return Err(format!(
            "no id kind known for parameter `{name}`; add a binding to the ids file"
        ));
    };
    let values = ids.values(&kind);
    if values.is_empty() {
        return Err(format!(
            "ids file has no `{kind}` values (parameter `{name}`)"
        ));
    }
    Ok(values.to_vec())
}

pub fn skipped_by_reason(plans: &[RoutePlan]) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in plans {
        if let Some(reason) = &p.skipped {
            out.entry(reason.clone()).or_default().push(p.route.clone());
        }
    }
    out
}
