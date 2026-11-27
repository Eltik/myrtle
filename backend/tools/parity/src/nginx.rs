use std::collections::BTreeSet;
use std::io::BufRead;
use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

use crate::corpus::{CorpusEntry, RouteMatcher, Source};
use crate::rng::{SplitMix64, fnv1a64, sample_indices};

/// `$remote_addr - $remote_user [$time_local] "$request" $status ...`
static COMBINED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\S+ \S+ \S+ \[[^\]]*\] "((?:[^"\\]|\\.)*)" (\d{3}) "#)
        .unwrap_or_else(|e| panic!("static regex: {e}"))
});

/// JWT: three base64url parts, the first starting `eyJ`
static JWT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"eyJ[A-Za-z0-9_-]{4,}\.[A-Za-z0-9_-]{4,}\.[A-Za-z0-9_-]*")
        .unwrap_or_else(|e| panic!("static regex: {e}"))
});

/// Case-insensitive substrings of the percent-decoded parameter name
pub const SECRET_PARAM_FRAGMENTS: &[&str] = &[
    "token",
    "key",
    "session",
    "secret",
    "pass",
    "auth",
    "jwt",
    "sig",
    "code",
    "credential",
    "cookie",
];

#[derive(Debug, Clone)]
pub struct LogOptions {
    pub prefix: String,
    pub sample: Option<u64>,
    pub seed: u64,
}

impl Default for LogOptions {
    fn default() -> Self {
        Self {
            prefix: "/api/".to_owned(),
            sample: None,
            seed: crate::DEFAULT_SEED,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LogStats {
    pub lines: usize,
    pub unparsed: usize,
    pub not_get: usize,
    pub not_2xx: usize,
    pub outside_prefix: usize,
    pub secret_in_path: usize,
    pub redacted_params: usize,
    pub duplicates: usize,
    pub sampled_out: usize,
    pub kept: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    pub method: String,
    pub target: String,
    pub status: u16,
}

pub fn parse_combined(line: &str) -> Option<LogLine> {
    let caps = COMBINED.captures(line)?;
    let request = caps.get(1)?.as_str();
    let status: u16 = caps.get(2)?.as_str().parse().ok()?;
    let mut parts = request.split_whitespace();
    let method = parts.next()?.to_owned();
    let target = parts.next()?.to_owned();
    Some(LogLine {
        method,
        target,
        status,
    })
}

fn percent_decode_lossy(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = bytes.get(i + 1..i + 3)
            && let Ok(hex) = std::str::from_utf8(hex)
            && let Ok(b) = u8::from_str_radix(hex, 16)
        {
            out.push(b);
            i += 3;
            continue;
        }
        out.push(if bytes[i] == b'+' { b' ' } else { bytes[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn is_secret_param(name: &str) -> bool {
    let name = percent_decode_lossy(name).to_ascii_lowercase();
    SECRET_PARAM_FRAGMENTS.iter().any(|f| name.contains(f))
}

pub fn redact_target(target: &str) -> Option<(String, usize)> {
    let mut t = target;
    if let Some(rest) = t
        .strip_prefix("http://")
        .or_else(|| t.strip_prefix("https://"))
    {
        t = rest.find('/').map_or("/", |i| &rest[i..]);
    }
    let t = t.split('#').next().unwrap_or(t);
    let (path, query) = t.split_once('?').map_or((t, None), |(p, q)| (p, Some(q)));
    if JWT.is_match(&percent_decode_lossy(path)) {
        return None;
    }
    let mut kept = Vec::new();
    let mut removed = 0;
    for pair in query.unwrap_or("").split('&').filter(|p| !p.is_empty()) {
        let name = pair.split('=').next().unwrap_or(pair);
        if is_secret_param(name) {
            removed += 1;
        } else {
            if JWT.is_match(&percent_decode_lossy(pair)) {
                return None;
            }
            kept.push(pair);
        }
    }
    let mut out = path.to_owned();
    if !kept.is_empty() {
        out.push('?');
        out.push_str(&kept.join("&"));
    }
    Some((out, removed))
}

pub fn import<R: BufRead>(
    reader: R,
    opts: &LogOptions,
    matcher: Option<&RouteMatcher>,
) -> std::io::Result<(Vec<CorpusEntry>, LogStats)> {
    let mut stats = LogStats::default();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for line in reader.lines() {
        let line = line?;
        stats.lines += 1;
        let Some(parsed) = parse_combined(&line) else {
            stats.unparsed += 1;
            continue;
        };
        if parsed.method != "GET" {
            stats.not_get += 1;
            continue;
        }
        if !(200..300).contains(&parsed.status) {
            stats.not_2xx += 1;
            continue;
        }
        let Some((target, removed)) = redact_target(&parsed.target) else {
            stats.secret_in_path += 1;
            continue;
        };
        if !target.starts_with(&opts.prefix) {
            stats.outside_prefix += 1;
            continue;
        }
        stats.redacted_params += removed;
        if !seen.insert(target) {
            stats.duplicates += 1;
        }
    }
    let all: Vec<String> = seen.into_iter().collect();
    let chosen: Vec<&String> = match opts.sample {
        Some(k) if (all.len() as u64) > k => {
            let mut rng = SplitMix64::new(opts.seed ^ fnv1a64(b"nginx"));
            let idx = sample_indices(all.len() as u64, k, &mut rng);
            stats.sampled_out = all.len() - idx.len();
            idx.into_iter().map(|i| &all[i as usize]).collect()
        }
        _ => all.iter().collect(),
    };
    let entries: Vec<CorpusEntry> = chosen
        .into_iter()
        .map(|path| {
            let route = matcher
                .and_then(|m| m.route_for(path))
                .map_or_else(|| "(unmatched)".to_owned(), str::to_owned);
            CorpusEntry::new(&route, path, None, Source::Nginx)
        })
        .collect();
    stats.kept = entries.len();
    Ok((entries, stats))
}
