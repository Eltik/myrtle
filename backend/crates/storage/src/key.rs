//! Object keys and public URLs
//!
//! Key = `[{CLOUDFLARE_PREFIX}/]{server}/{rel}`, `rel` raw with
//! forward slashes.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

/// What `encodeURIComponent` encodes: everything except ASCII alphanumerics
/// and `-_.!~*'()`
pub const URI_COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

#[must_use]
pub fn normalize_rel(rel: &str) -> Option<String> {
    if rel.contains('\0') {
        return None;
    }
    let mut parts = Vec::new();
    for seg in rel.split('/') {
        match seg {
            "" | "." => {}
            ".." => return None,
            s => parts.push(s),
        }
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("/"))
}

/// The key prefix every object of `server` lives under
#[must_use]
pub fn server_prefix(prefix: &str, server: &str) -> String {
    if prefix.is_empty() {
        format!("{server}/")
    } else {
        format!("{prefix}/{server}/")
    }
}

/// `[prefix/]server/rel`. `rel` must already be normalized.
#[must_use]
pub fn object_key(prefix: &str, server: &str, rel: &str) -> String {
    format!("{}{rel}", server_prefix(prefix, server))
}

/// True when normalized `rel` has file under one of `roots`
#[must_use]
pub fn is_under_roots(rel: &str, roots: &[String]) -> bool {
    let Some(rel) = normalize_rel(rel) else {
        return false;
    };
    match rel.split_once('/') {
        Some((first, rest)) => !rest.is_empty() && roots.iter().any(|r| r == first),
        None => false,
    }
}

/// `{base}/{key}` with each key segment encoded as `encodeURIComponent` would
#[must_use]
pub fn public_url(base: &str, key: &str) -> String {
    let mut url = base.trim_end_matches('/').to_owned();
    for seg in key.split('/') {
        url.push('/');
        url.extend(utf8_percent_encode(seg, URI_COMPONENT));
    }
    url
}
