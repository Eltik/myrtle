//! Keeping credentials out of the logs.
//!
//! The upstream clients log a response body when one fails to parse, and those
//! bodies carry session tokens and access keys; the login services log the
//! identifier an attempt used. Both log at WARN, which is on in production.
//!
//! A parse failure still needs a body to diagnose, so these do not drop the
//! field - they bound its length and blank the values that are credentials.

use std::sync::LazyLock;

use regex::Regex;

/// Longest body kept in a log line: enough to see the shape of an envelope and
/// the error code in it, short enough that a large payload cannot flood the
/// log.
const MAX_BODY_CHARS: usize = 300;

/// JSON string fields whose VALUE is a credential. Matched case-insensitively
/// against the key, so `accessKey`, `access_key` and `ACCESSKEY` all hit.
static SECRET_FIELD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)"(token|secret|access_?key|session|password|passwd|passwd_?token|ticket|cred|credential|sign|signature|auth|cookie|yostar_?token|u8_?token)"\s*:\s*"[^"]*""#,
    )
    .expect("static redaction regex is valid")
});

/// A response body safe to log: credential values blanked, length bounded.
///
/// Truncation is char-boundary safe, so a body cut mid-UTF-8 cannot panic the
/// logging path - which would turn a diagnostic into an outage.
pub fn redacted_body(text: &str) -> String {
    let cleaned = SECRET_FIELD.replace_all(text, r#""${1}":"<redacted>""#);

    if cleaned.chars().count() <= MAX_BODY_CHARS {
        return cleaned.into_owned();
    }

    let head: String = cleaned.chars().take(MAX_BODY_CHARS).collect();
    format!("{head}… (+{} bytes)", cleaned.len() - head.len())
}

/// An identifier reduced to something that can still be matched against a
/// support report without being the identifier itself.
///
/// Anything four characters or shorter is blanked outright rather than
/// half-revealed.
pub fn mask_identifier(value: &str) -> String {
    let count = value.chars().count();
    match count {
        0 => "<empty>".to_owned(),
        1..=4 => "*".repeat(count),
        _ => {
            let first = value.chars().next().unwrap_or('*');
            let last = value.chars().last().unwrap_or('*');
            format!("{first}{}{last}", "*".repeat(count - 2))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{mask_identifier, redacted_body};

    #[test]
    fn blanks_credential_values_and_keeps_the_rest() {
        let body = r#"{"code":0,"msg":"ok","data":{"token":"abc123","uid":"9001"}}"#;
        let out = redacted_body(body);
        assert!(!out.contains("abc123"), "token value survived: {out}");
        assert!(out.contains(r#""token":"<redacted>""#), "{out}");
        // Non-credential fields are diagnostic and stay.
        assert!(out.contains(r#""uid":"9001""#), "{out}");
        assert!(out.contains(r#""msg":"ok""#), "{out}");
    }

    #[test]
    fn matches_key_spelling_variants() {
        for key in ["accessKey", "access_key", "ACCESSKEY", "Secret"] {
            let body = format!(r#"{{"{key}":"leaked"}}"#);
            assert!(
                !redacted_body(&body).contains("leaked"),
                "{key} value survived"
            );
        }
    }

    #[test]
    fn bounds_length_without_splitting_a_char() {
        // Multi-byte throughout, so a naive byte slice would land mid-char.
        let body = "日".repeat(5_000);
        let out = redacted_body(&body);
        assert!(out.chars().count() < 400, "not truncated: {}", out.len());
        assert!(out.contains('…'));
    }

    #[test]
    fn short_body_is_returned_whole() {
        assert_eq!(redacted_body(r#"{"code":1}"#), r#"{"code":1}"#);
    }

    #[test]
    fn masks_identifiers() {
        assert_eq!(mask_identifier("13800138000"), "1*********0");
        assert_eq!(mask_identifier("abcd"), "****");
        assert_eq!(mask_identifier(""), "<empty>");
        // Multi-byte input must not panic or split.
        assert_eq!(mask_identifier("日本語です"), "日***す");
    }
}
