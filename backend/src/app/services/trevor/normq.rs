//! The question as the cache key sees it: normQ (contract v1.1 section 1a),
//! the v1 normalization `TREVOR_NORMQ=0` keeps, the key hash and the
//! length check.

use sha2::{Digest, Sha256};

use super::MAX_QUESTION_CHARS;
use crate::app::error::ApiError;

/// The question as the cache saw it before contract v1.1 (kept verbatim for
/// `TREVOR_NORMQ=0`): case-folded, whitespace collapsed, typographic quotes
/// and dashes made plain, trailing question marks and full stops dropped.
pub fn normalize_question_v1(question: &str) -> String {
    let mapped: String = question
        .chars()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{2032}' => '\'',
            '\u{201C}' | '\u{201D}' => '"',
            '\u{2013}' | '\u{2014}' => '-',
            '\u{3000}' => ' ',
            '\u{FF1F}' => '?',
            '\u{3002}' => '.',
            c => c,
        })
        .collect();
    let collapsed = mapped
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    collapsed.trim_end_matches(['?', '.', '!', ' ']).to_owned()
}

/// The compatibility fold of one character: the NFKC mappings a typed
/// question meets (full-width ASCII, the ideographic and other Unicode
/// spaces, ligatures, super/subscript digits, the ellipsis). Since
/// 2026-10-17 [`normalize_question`] runs full NFKC (`unicode-normalization`)
/// before it, so this keeps only what NFKC leaves alone.
fn nfkc_fold(c: char, out: &mut String) {
    match c {
        // Full-width ASCII block -> ASCII.
        '\u{FF01}'..='\u{FF5E}' => {
            out.push(char::from_u32(c as u32 - 0xFF01 + 0x21).unwrap_or(c));
        }
        '\u{00A0}'
        | '\u{1680}'
        | '\u{2000}'..='\u{200A}'
        | '\u{202F}'
        | '\u{205F}'
        | '\u{3000}' => out.push(' '),
        '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}' | '\u{00AD}' => {}
        '\u{FB00}' => out.push_str("ff"),
        '\u{FB01}' => out.push_str("fi"),
        '\u{FB02}' => out.push_str("fl"),
        '\u{FB03}' => out.push_str("ffi"),
        '\u{FB04}' => out.push_str("ffl"),
        '\u{2026}' => out.push_str("..."),
        '\u{00B2}' => out.push('2'),
        '\u{00B3}' => out.push('3'),
        '\u{00B9}' => out.push('1'),
        '\u{2070}' => out.push('0'),
        '\u{2074}'..='\u{2079}' => {
            out.push(char::from_u32(c as u32 - 0x2074 + u32::from(b'4')).unwrap_or(c));
        }
        '\u{2080}'..='\u{2089}' => {
            out.push(char::from_u32(c as u32 - 0x2080 + u32::from(b'0')).unwrap_or(c));
        }
        '\u{2160}'..='\u{2169}' => {
            const ROMAN: [&str; 10] = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"];
            out.push_str(ROMAN[(c as u32 - 0x2160) as usize]);
        }
        '\u{3002}' | '\u{FF61}' => out.push('.'),
        c => out.push(c),
    }
}

/// An apostrophe inside a word joins it ("Kal'tsit" = "Kaltsit"); every
/// other punctuation mark is a word break.
const fn is_apostrophe(c: char) -> bool {
    matches!(
        c,
        '\'' | '\u{2018}' | '\u{2019}' | '\u{02BC}' | '\u{2032}' | '`' | '\u{00B4}'
    )
}

/// normQ (contract v1.1 section 1a), the question as the cache, the twin
/// join and the open-job index see it: the NFKC compatibility fold
/// ([`nfkc_fold`]), lowercase, apostrophes dropped, every other punctuation
/// or symbol mark folded to a space, whitespace collapsed, which also drops
/// a trailing "?". Word order and digits are kept, so "Did W kill Theresa"
/// and "Did Theresa kill W" stay two questions. `TREVOR_NORMQ=0` restores
/// [`normalize_question_v1`].
pub fn normalize_question(question: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    // Full NFKC first (composes "e" + U+0301 into "é", folds compatibility
    // forms); `nfkc_fold` then applies the mappings NFKC leaves alone.
    let question: String = question.nfkc().collect();
    let mut folded = String::with_capacity(question.len());
    for c in question.chars() {
        nfkc_fold(c, &mut folded);
    }
    let mut out = String::with_capacity(folded.len());
    for c in folded.chars().flat_map(char::to_lowercase) {
        if is_apostrophe(c) {
            continue;
        }
        // Combining marks stay with their letter; anything else that is not
        // a letter or digit breaks the word.
        if c.is_alphanumeric() || is_combining(c) {
            out.push(c);
        } else {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The combining diacritical marks (U+0300 to U+036F), which belong to the
/// letter before them rather than breaking the word.
const fn is_combining(c: char) -> bool {
    matches!(c, '\u{0300}'..='\u{036F}')
}

/// The normalized question under the configured rule (`TREVOR_NORMQ`).
pub fn normalize_with(question: &str, normq: bool) -> String {
    if normq {
        normalize_question(question)
    } else {
        normalize_question_v1(question)
    }
}

/// The cache key of a normalized question: 32 hex chars of its SHA-256.
pub fn qkey(normalized: &str) -> String {
    hex::encode(&Sha256::digest(normalized.as_bytes())[..16])
}

/// Admit a question, or say why not; `normq` picks the normalization
/// (`TREVOR_NORMQ`).
pub fn validate_question(question: &str, normq: bool) -> Result<String, ApiError> {
    let normalized = normalize_with(question, normq);
    if normalized.is_empty() {
        return Err(ApiError::BadRequest("question is empty".into()));
    }
    if question.chars().count() > MAX_QUESTION_CHARS {
        return Err(ApiError::BadRequest(format!(
            "question is longer than {MAX_QUESTION_CHARS} characters"
        )));
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_v1_folds_case_space_quotes_and_trailing_marks() {
        assert_eq!(
            normalize_question_v1("  Who is   Kal\u{2019}tsit?? "),
            "who is kal'tsit"
        );
        assert_eq!(normalize_question_v1("Who is Kal'tsit"), "who is kal'tsit");
        assert_eq!(normalize_question_v1("Why?\u{FF1F}"), "why");
        assert_eq!(
            normalize_question_v1("\u{201C}Ch\u{2014}en\u{201D}!"),
            "\"ch-en\""
        );
        assert_eq!(normalize_question_v1("a, b"), "a, b");
        // TREVOR_NORMQ=0 is exactly the v1 rule.
        assert_eq!(
            normalize_with("A, b?", false),
            normalize_question_v1("A, b?")
        );
    }

    #[test]
    fn normq_folds_nfkc_case_punctuation_space_and_the_trailing_mark() {
        let n = normalize_question;
        assert_eq!(n("  Who is   Kal\u{2019}tsit?? "), "who is kaltsit");
        assert_eq!(n("Who is Kal'tsit"), "who is kaltsit");
        assert_eq!(n("who is kaltsit"), "who is kaltsit");
        // Full-width letters, ideographic space and question mark (NFKC).
        assert_eq!(
            n("\u{FF37}ho\u{3000}is \u{FF21}miya\u{FF1F}"),
            "who is amiya"
        );
        assert_eq!(n("Why?\u{FF1F}"), "why");
        assert_eq!(n("\u{201C}Ch\u{2014}en\u{201D}!"), "ch en");
        assert_eq!(n("a, b"), "a b");
        assert_eq!(n("a,b"), "a b");
        assert_eq!(n("Who is Amiya\u{2026}"), "who is amiya");
        assert_eq!(n("the \u{FB01}nal battle"), "the final battle");
        assert_eq!(n("Episode \u{2167}"), "episode viii");
        assert_eq!(n("zero\u{200B}width"), "zerowidth");
        assert_eq!(n("Wi\u{0161}'adel"), "wi\u{0161}adel");
        // Full NFKC (2026-10-17): a decomposed accent composes onto its letter.
        assert_eq!(n("Wis\u{030C}'adel"), "wi\u{0161}adel");
    }

    #[test]
    fn normq_keeps_what_changes_meaning() {
        let n = normalize_question;
        assert_ne!(n("Did W kill Theresa?"), n("Did Theresa kill W?"));
        assert_ne!(n("chapter 10"), n("chapter 1"));
        assert_ne!(n("chapter 1-10"), n("chapter 110"));
        assert_eq!(n("chapter 1-10"), "chapter 1 10");
    }

    #[test]
    fn qkey_is_stable_and_shared_by_equivalent_questions() {
        let a = qkey(&normalize_question("Who is Amiya?"));
        let b = qkey(&normalize_question("who is amiya"));
        assert_eq!(a, b);
        assert_eq!(a.len(), 32);
        assert_ne!(a, qkey(&normalize_question("Who is Kal'tsit?")));
        // Full NFKC (2026-10-17): a decomposed accent keys like the precomposed letter.
        assert_eq!(
            normalize_question("Who is Ve\u{0301}ronique?"),
            normalize_question("Who is V\u{E9}ronique?")
        );
        assert_eq!(normalize_question("\u{FB01}nal battle"), "final battle");
    }

    #[test]
    fn validate_refuses_empty_and_long() {
        for normq in [true, false] {
            assert!(validate_question(" ?? ", normq).is_err());
            assert!(validate_question(&"a".repeat(MAX_QUESTION_CHARS + 1), normq).is_err());
            assert!(validate_question(&"a".repeat(MAX_QUESTION_CHARS), normq).is_ok());
        }
        // Punctuation only is empty under normQ.
        assert!(validate_question("!?,.", true).is_err());
    }
}
