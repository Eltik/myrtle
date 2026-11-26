use std::collections::HashMap;
use std::fmt::{self, Write as _};

use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use sha2::{Digest, Sha256};

use crate::glob::{PointerGlob, any_matches, format_pointer};

pub const DEFAULT_REL_TOL: f64 = 1e-9;
pub const PREVIEW_CHARS: usize = 160;

#[derive(Debug, Clone)]
pub struct CompareOptions {
    pub rel_tol: f64,
    pub abs_tol: f64,
    pub unordered: Vec<PointerGlob>,
    pub ignore: Vec<PointerGlob>,
    pub max_diffs: usize,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            rel_tol: DEFAULT_REL_TOL,
            abs_tol: 0.0,
            unordered: Vec::new(),
            ignore: Vec::new(),
            max_diffs: 20,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffKind {
    Status,
    ContentType,
    BodyHash,
    Type,
    Value,
    Number,
    MissingInB,
    MissingInA,
    Length,
    UnmatchedInA,
    UnmatchedInB,
}

impl DiffKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Status => "status differs",
            Self::ContentType => "content-type differs",
            Self::BodyHash => "body hash differs",
            Self::Type => "type differs",
            Self::Value => "value differs",
            Self::Number => "number outside tolerance",
            Self::MissingInB => "only in A",
            Self::MissingInA => "only in B",
            Self::Length => "array length differs",
            Self::UnmatchedInA => "unordered element only in A",
            Self::UnmatchedInB => "unordered element only in B",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diff {
    pub pointer: String,
    pub kind: DiffKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub b: Option<String>,
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let at = if self.pointer.is_empty()
            && !matches!(
                self.kind,
                DiffKind::Status | DiffKind::ContentType | DiffKind::BodyHash
            ) {
            "(root)"
        } else {
            &self.pointer
        };
        if at.is_empty() {
            write!(f, "{}", self.kind.label())?;
        } else {
            write!(f, "{at}: {}", self.kind.label())?;
        }
        if let Some(a) = &self.a {
            write!(f, "  A={a}")?;
        }
        if let Some(b) = &self.b {
            write!(f, "  B={b}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Comparison {
    pub diffs: Vec<Diff>,
    pub total: usize,
}

impl Comparison {
    pub const fn is_equal(&self) -> bool {
        self.total == 0
    }

    fn push(&mut self, diff: Diff, max: usize) {
        self.total += 1;
        if self.diffs.len() < max {
            self.diffs.push(diff);
        }
    }
}

pub fn floats_equal(x: f64, y: f64, rel: f64, abs: f64) -> bool {
    if x.to_bits() == y.to_bits() {
        return true;
    }
    if x.is_nan() || y.is_nan() {
        return x.is_nan() && y.is_nan();
    }
    if !x.is_finite() || !y.is_finite() {
        return false;
    }
    let diff = (x - y).abs();
    diff <= abs || diff <= rel * x.abs().max(y.abs())
}

pub fn numbers_equal(a: &Number, b: &Number, rel: f64, abs: f64) -> bool {
    if a.is_f64() || b.is_f64() {
        return match (a.as_f64(), b.as_f64()) {
            (Some(x), Some(y)) => floats_equal(x, y, rel, abs),
            _ => false,
        };
    }
    if let (Some(x), Some(y)) = (a.as_i64(), b.as_i64()) {
        return x == y;
    }
    if let (Some(x), Some(y)) = (a.as_u64(), b.as_u64()) {
        return x == y;
    }
    false
}

const fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub fn preview(v: &Value) -> String {
    let s = v.to_string();
    if s.chars().count() <= PREVIEW_CHARS {
        return s;
    }
    let cut: String = s.chars().take(PREVIEW_CHARS).collect();
    format!("{cut}...")
}

pub fn canonical_json(v: &Value) -> String {
    let mut out = String::new();
    write_canonical(v, &mut out);
    out
}

fn write_canonical(v: &Value, out: &mut String) {
    match v {
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (i, key) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String(key.clone()).to_string());
                out.push(':');
                write_canonical(&map[key], out);
            }
            out.push('}');
        }
        scalar => {
            let _ = write!(out, "{scalar}");
        }
    }
}

struct Walker<'o> {
    opts: &'o CompareOptions,
    path: Vec<String>,
    out: Comparison,
    probe: bool,
}

impl<'o> Walker<'o> {
    const fn new(opts: &'o CompareOptions, path: Vec<String>, probe: bool) -> Self {
        Self {
            opts,
            path,
            out: Comparison {
                diffs: Vec::new(),
                total: 0,
            },
            probe,
        }
    }

    const fn stop(&self) -> bool {
        self.probe && self.out.total > 0
    }

    fn report(&mut self, kind: DiffKind, a: Option<String>, b: Option<String>) {
        let max = if self.probe { 0 } else { self.opts.max_diffs };
        self.out.push(
            Diff {
                pointer: format_pointer(&self.path),
                kind,
                a,
                b,
            },
            max,
        );
    }

    fn walk(&mut self, a: &Value, b: &Value) {
        if self.stop() || any_matches(&self.opts.ignore, &self.path) {
            return;
        }
        match (a, b) {
            (Value::Null, Value::Null) => {}
            (Value::Bool(x), Value::Bool(y)) => {
                if x != y {
                    self.report(DiffKind::Value, Some(preview(a)), Some(preview(b)));
                }
            }
            (Value::String(x), Value::String(y)) => {
                if x != y {
                    self.report(DiffKind::Value, Some(preview(a)), Some(preview(b)));
                }
            }
            (Value::Number(x), Value::Number(y)) => {
                if !numbers_equal(x, y, self.opts.rel_tol, self.opts.abs_tol) {
                    self.report(DiffKind::Number, Some(preview(a)), Some(preview(b)));
                }
            }
            (Value::Array(xs), Value::Array(ys)) => {
                if any_matches(&self.opts.unordered, &self.path) {
                    self.walk_unordered(xs, ys);
                } else {
                    self.walk_ordered(xs, ys);
                }
            }
            (Value::Object(xm), Value::Object(ym)) => {
                let mut keys: Vec<&String> = xm.keys().collect();
                keys.sort();
                for key in keys {
                    if self.stop() {
                        return;
                    }
                    self.path.push(key.clone());
                    match ym.get(key) {
                        Some(yv) => self.walk(&xm[key], yv),
                        None => {
                            if !any_matches(&self.opts.ignore, &self.path) {
                                self.report(DiffKind::MissingInB, Some(preview(&xm[key])), None);
                            }
                        }
                    }
                    self.path.pop();
                }
                let mut extra: Vec<&String> = ym.keys().filter(|k| !xm.contains_key(*k)).collect();
                extra.sort();
                for key in extra {
                    if self.stop() {
                        return;
                    }
                    self.path.push(key.clone());
                    if !any_matches(&self.opts.ignore, &self.path) {
                        self.report(DiffKind::MissingInA, None, Some(preview(&ym[key])));
                    }
                    self.path.pop();
                }
            }
            _ => self.report(
                DiffKind::Type,
                Some(format!("{} {}", type_name(a), preview(a))),
                Some(format!("{} {}", type_name(b), preview(b))),
            ),
        }
    }

    fn walk_ordered(&mut self, xs: &[Value], ys: &[Value]) {
        if xs.len() != ys.len() {
            self.report(
                DiffKind::Length,
                Some(xs.len().to_string()),
                Some(ys.len().to_string()),
            );
        }
        for (i, (x, y)) in xs.iter().zip(ys).enumerate() {
            if self.stop() {
                return;
            }
            self.path.push(i.to_string());
            self.walk(x, y);
            self.path.pop();
        }
    }

    fn walk_unordered(&mut self, xs: &[Value], ys: &[Value]) {
        let mut buckets: HashMap<String, Vec<usize>> = HashMap::new();
        for (j, y) in ys.iter().enumerate().rev() {
            buckets.entry(canonical_json(y)).or_default().push(j);
        }
        let mut y_used = vec![false; ys.len()];
        let mut x_left = Vec::new();
        for (i, x) in xs.iter().enumerate() {
            match buckets.get_mut(&canonical_json(x)).and_then(Vec::pop) {
                Some(j) => y_used[j] = true,
                None => x_left.push(i),
            }
        }
        let mut x_unmatched = Vec::new();
        for i in x_left {
            if self.stop() {
                return;
            }
            let mut path = self.path.clone();
            path.push(i.to_string());
            let found = (0..ys.len()).find(|&j| {
                if y_used[j] {
                    return false;
                }
                let mut probe = Walker::new(self.opts, path.clone(), true);
                probe.walk(&xs[i], &ys[j]);
                probe.out.total == 0
            });
            match found {
                Some(j) => y_used[j] = true,
                None => x_unmatched.push(i),
            }
        }
        for i in x_unmatched {
            if self.stop() {
                return;
            }
            self.path.push(i.to_string());
            self.report(DiffKind::UnmatchedInA, Some(preview(&xs[i])), None);
            self.path.pop();
        }
        for (j, used) in y_used.iter().enumerate() {
            if self.stop() {
                return;
            }
            if !used {
                self.path.push(j.to_string());
                self.report(DiffKind::UnmatchedInB, None, Some(preview(&ys[j])));
                self.path.pop();
            }
        }
    }
}

pub fn compare_json(a: &Value, b: &Value, opts: &CompareOptions) -> Comparison {
    let mut walker = Walker::new(opts, Vec::new(), false);
    walker.walk(a, b);
    walker.out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    Json(Value),
    Opaque {
        sha256: String,
        len: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observed {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Body,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn normalize_content_type(raw: &str) -> String {
    raw.split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

pub fn is_json_type(content_type: Option<&str>) -> bool {
    content_type.is_some_and(|ct| ct == "application/json" || ct.ends_with("+json"))
}

pub fn observe(status: u16, content_type: Option<&str>, bytes: &[u8]) -> Observed {
    let content_type = content_type.map(normalize_content_type);
    let body = if is_json_type(content_type.as_deref()) {
        serde_json::from_slice::<Value>(bytes).map_or_else(
            |_| Body::Opaque {
                sha256: sha256_hex(bytes),
                len: bytes.len() as u64,
            },
            Body::Json,
        )
    } else {
        Body::Opaque {
            sha256: sha256_hex(bytes),
            len: bytes.len() as u64,
        }
    };
    Observed {
        status,
        content_type,
        body,
    }
}

fn body_label(body: &Body) -> String {
    match body {
        Body::Json(_) => "json".to_owned(),
        Body::Opaque { sha256, len } => format!("sha256:{} ({len} bytes)", &sha256[..16]),
    }
}

pub fn compare_observed(a: &Observed, b: &Observed, opts: &CompareOptions) -> Comparison {
    let mut out = Comparison::default();
    if a.status != b.status {
        out.push(
            Diff {
                pointer: String::new(),
                kind: DiffKind::Status,
                a: Some(a.status.to_string()),
                b: Some(b.status.to_string()),
            },
            opts.max_diffs,
        );
        return out;
    }
    if a.content_type != b.content_type {
        out.push(
            Diff {
                pointer: String::new(),
                kind: DiffKind::ContentType,
                a: Some(
                    a.content_type
                        .clone()
                        .unwrap_or_else(|| "(none)".to_owned()),
                ),
                b: Some(
                    b.content_type
                        .clone()
                        .unwrap_or_else(|| "(none)".to_owned()),
                ),
            },
            opts.max_diffs,
        );
    }
    match (&a.body, &b.body) {
        (Body::Json(x), Body::Json(y)) => {
            let inner = compare_json(x, y, opts);
            out.total += inner.total;
            let room = opts.max_diffs.saturating_sub(out.diffs.len());
            out.diffs.extend(inner.diffs.into_iter().take(room));
        }
        (Body::Opaque { sha256: x, .. }, Body::Opaque { sha256: y, .. }) if x == y => {}
        (x, y) => out.push(
            Diff {
                pointer: String::new(),
                kind: DiffKind::BodyHash,
                a: Some(body_label(x)),
                b: Some(body_label(y)),
            },
            opts.max_diffs,
        ),
    }
    out
}
