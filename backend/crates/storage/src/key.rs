//! Object keys and public URLs.
//!
//! Key = `[{S3_PREFIX}/]{server}/{rel}`, `rel` raw (not percent-encoded) with
//! forward slashes. The public URL percent-encodes each segment with the
//! `encodeURIComponent` set, so the frontend and backend build the same URL.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

/// What `encodeURIComponent` encodes: everything except ASCII alphanumerics
/// and `-_.!~*'()`.
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

/// `rel` as the key's tail: leading slashes and `.` segments dropped, joined
/// with `/`. `None` for anything the backend's `validate_asset_path` rejects
/// (`..`, NUL) and for an empty path. A backslash is kept as a literal
/// character, as on the Unix filesystem the tree lives on.
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

/// The key prefix every object of `server` lives under, with its trailing
/// slash: `assets/en/`, or `en/` without a prefix.
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

/// True when the normalized `rel` has a file under one of `roots`: its first
/// segment is a root and something follows it.
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

/// `{base}/{key}` with each key segment encoded as `encodeURIComponent` would.
#[must_use]
pub fn public_url(base: &str, key: &str) -> String {
    let mut url = base.trim_end_matches('/').to_owned();
    for seg in key.split('/') {
        url.push('/');
        url.extend(utf8_percent_encode(seg, URI_COMPONENT));
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roots() -> Vec<String> {
        ["textures", "spine", "audio", "video", "portraits"]
            .iter()
            .map(|r| (*r).to_owned())
            .collect()
    }

    #[test]
    fn key_layout_matches_the_contract() {
        let rel = normalize_rel("spine/DynIllust/char_1012_skadi2/[scene].json").unwrap();
        assert_eq!(
            object_key("assets", "en", &rel),
            "assets/en/spine/DynIllust/char_1012_skadi2/[scene].json"
        );
        assert_eq!(object_key("", "cn", "textures/a.png"), "cn/textures/a.png");
    }

    #[test]
    fn brackets_spaces_hash_and_percent_are_encoded() {
        let base = "https://assets.myrtle.moe";
        assert_eq!(
            public_url(
                base,
                "assets/en/spine/DynIllust/char_1012_skadi2/[scene].json"
            ),
            "https://assets.myrtle.moe/assets/en/spine/DynIllust/char_1012_skadi2/%5Bscene%5D.json"
        );
        assert_eq!(
            public_url(base, "en/textures/a b#c%d?.png"),
            "https://assets.myrtle.moe/en/textures/a%20b%23c%25d%3F.png"
        );
        // encodeURIComponent leaves these alone.
        assert_eq!(
            public_url(base, "en/audio/x-y_z.!~*'().ogg"),
            "https://assets.myrtle.moe/en/audio/x-y_z.!~*'().ogg"
        );
        assert_eq!(
            public_url("https://cdn.example/", "en/a+b,c.png"),
            "https://cdn.example/en/a%2Bb%2Cc.png"
        );
        assert_eq!(
            public_url(base, "en/textures/\u{30a2}.png"),
            "https://assets.myrtle.moe/en/textures/%E3%82%A2.png"
        );
    }

    #[test]
    fn rel_normalization() {
        assert_eq!(
            normalize_rel("/textures/a.png").as_deref(),
            Some("textures/a.png")
        );
        assert_eq!(
            normalize_rel("./textures//./a.png").as_deref(),
            Some("textures/a.png")
        );
        assert_eq!(normalize_rel("textures/../gamedata/x.json"), None);
        assert_eq!(normalize_rel("../textures/a.png"), None);
        assert_eq!(normalize_rel("a\0.png"), None);
        assert_eq!(normalize_rel("/"), None);
        assert_eq!(normalize_rel(""), None);
    }

    #[test]
    fn roots_check() {
        let roots = roots();
        assert!(is_under_roots("spine/x/[scene].json", &roots));
        assert!(is_under_roots("/textures/a.png", &roots));
        assert!(is_under_roots("./audio/a.ogg", &roots));
        assert!(!is_under_roots("gamedata/excel/a.json", &roots));
        assert!(!is_under_roots("derived/a.json", &roots));
        assert!(!is_under_roots("Textures/a.png", &roots));
        assert!(!is_under_roots("texturesx/a.png", &roots));
        assert!(!is_under_roots("textures", &roots));
        assert!(!is_under_roots("textures/../gamedata/a.json", &roots));
        assert!(!is_under_roots("", &roots));
    }

    #[test]
    fn server_prefix_has_one_trailing_slash() {
        assert_eq!(server_prefix("assets", "en"), "assets/en/");
        assert_eq!(server_prefix("", "en"), "en/");
    }
}
