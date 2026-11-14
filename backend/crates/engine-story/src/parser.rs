use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// One parsed line
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryCommand {
    pub kind: String,
    pub args: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// 1-based line number in the script file
    pub line: u32,
}

/// The kinds whose `text` is prose the reader shows for word counts
pub const PROSE_KINDS: &[&str] = &["name", "text", "multiline", "narration"];

/// The kinds whose prose sits in `args.text` rather than after the bracket
pub const PROSE_ARG_KINDS: &[&str] = &["subtitle", "sticker"];

/// Misspelled command names found in the EN gamedata
const TYPO_TABLE: &[(&str, &str)] = &[
    ("palysound", "playsound"),
    ("plysound", "playsound"),
    ("delya", "delay"),
    ("delau", "delay"),
    ("delat", "delay"),
    ("daley", "delay"),
    ("dealy", "delay"),
    ("dalay", "delay"),
    ("delayt", "delay"),
    ("delay9ti", "delay"),
    ("stopmucis", "stopmusic"),
    ("musicvolune", "musicvolume"),
    ("dialo", "dialog"),
    ("dialogs", "dialog"),
    ("charslsot", "charslot"),
    ("chaa", "character"),
];

#[must_use]
pub fn normalize_kind(kind: &str) -> &str {
    TYPO_TABLE
        .iter()
        .find(|(typo, _)| *typo == kind)
        .map_or(kind, |(_, canon)| canon)
}

#[must_use]
pub const fn typo_table() -> &'static [(&'static str, &'static str)] {
    TYPO_TABLE
}

/// Where parse deliberately differs from the client
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deviation {
    InvalidEscape,
    TypoFolded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub line: u32,
    pub deviation: Deviation,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParseStats {
    /// Raw lines swallowed by a trailing backslash
    pub continued_lines: u32,
    /// Argument values in which at least one escape decoded
    pub decoded_values: u32,
    pub dropped_arg_blocks: u32,
    pub diagnostics: Vec<ParseDiagnostic>,
}

/// Parse a whole script
#[must_use]
pub fn parse(script: &str) -> Vec<StoryCommand> {
    parse_with_stats(script).0
}

/// [`parse`] plus what it measured and where it deviated from the client
#[must_use]
pub fn parse_with_stats(script: &str) -> (Vec<StoryCommand>, ParseStats) {
    let script = script.strip_prefix('\u{feff}').unwrap_or(script);
    let mut stats = ParseStats::default();
    let mut out = Vec::new();
    for (line_no, line) in logical_lines(script, &mut stats.continued_lines) {
        if let Some(c) = parse_line(&line, line_no, &mut stats) {
            out.push(c);
        }
    }
    (out, stats)
}

fn logical_lines(script: &str, continued: &mut u32) -> Vec<(u32, String)> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut start = 1u32;
    let mut continuing = false;
    for (i, raw) in script.lines().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let line_no = (i + 1) as u32;
        if continuing {
            *continued += 1;
        } else {
            start = line_no;
        }
        if let Some(head) = raw.strip_suffix('\\') {
            buf.push_str(head);
            continuing = true;
            continue;
        }
        buf.push_str(raw);
        continuing = false;
        let line = buf.trim();
        if !line.is_empty() {
            out.push((start, line.to_owned()));
        }
        buf.clear();
    }
    let line = buf.trim();
    if !line.is_empty() {
        out.push((start, line.to_owned()));
    }
    out
}

fn parse_line(line: &str, line_no: u32, stats: &mut ParseStats) -> Option<StoryCommand> {
    let text_command = |text: &str| StoryCommand {
        kind: "text".to_owned(),
        args: BTreeMap::new(),
        text: Some(text.to_owned()),
        line: line_no,
    };

    if !line.starts_with('[') {
        return Some(text_command(line));
    }
    let Some(close) = matching_bracket(line) else {
        return Some(text_command(line));
    };
    let head = line[1..close].trim().trim_start_matches('[').trim();
    let trailing = line[close + 1..].trim().trim_start_matches(']').trim();
    let text = (!trailing.is_empty()).then(|| trailing.to_owned());

    let before = stats.diagnostics.len();
    let (kind, args) = parse_head(head, line_no, stats);
    if stats.diagnostics[before..]
        .iter()
        .any(|d| d.deviation == Deviation::InvalidEscape)
    {
        stats.dropped_arg_blocks += 1;
    }
    if kind.is_empty() {
        return None;
    }
    Some(StoryCommand {
        kind,
        args,
        text,
        line: line_no,
    })
}

fn matching_bracket(line: &str) -> Option<usize> {
    let mut in_quote = false;
    let mut escaped = false;
    for (i, c) in line.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' if in_quote => escaped = true,
            '"' => in_quote = !in_quote,
            ']' if !in_quote => return Some(i),
            _ => {}
        }
    }
    None
}

/// Split the bracket body into `(kind, args)`
fn parse_head(
    head: &str,
    line_no: u32,
    stats: &mut ParseStats,
) -> (String, BTreeMap<String, String>) {
    let paren = head.find('(');
    let eq = head.find('=');
    if let Some(p) = paren
        && eq.is_none_or(|e| p < e)
    {
        let kind = canon_kind(&head[..p], line_no, stats);
        let inner = head[p + 1..].trim_end();
        let inner = inner.strip_suffix(')').unwrap_or(inner);
        return (kind, parse_args(inner, line_no, stats));
    }
    if let Some(e) = eq {
        let kind = canon_kind(&head[..e], line_no, stats);
        let value = head[e + 1..].trim();
        let args = if let Some(inner) = value.strip_prefix('(').and_then(|v| v.strip_suffix(')')) {
            // `[Delay=(time=1.5)]`: a paren form behind an `=`.
            parse_args(inner, line_no, stats)
        } else {
            BTreeMap::from([(kind.clone(), decode_value(value, &kind, line_no, stats))])
        };
        return (kind, args);
    }
    (canon_kind(head, line_no, stats), BTreeMap::new())
}

fn canon_kind(raw: &str, line_no: u32, stats: &mut ParseStats) -> String {
    let lower = raw.trim().to_ascii_lowercase();
    let canon = normalize_kind(&lower).to_owned();
    if canon != lower {
        stats.diagnostics.push(ParseDiagnostic {
            line: line_no,
            deviation: Deviation::TypoFolded,
            detail: lower,
        });
    }
    canon
}

/// `k=v, k2 = "a, b"` -> map
fn parse_args(inner: &str, line_no: u32, stats: &mut ParseStats) -> BTreeMap<String, String> {
    let mut args = BTreeMap::new();
    for piece in split_args(inner) {
        let piece = piece.trim();
        if piece.is_empty() {
            continue;
        }
        let (key, value) = piece
            .split_once('=')
            .map_or((piece, ""), |(k, v)| (k.trim(), v.trim()));
        if key.is_empty() {
            continue;
        }
        let key = key.to_ascii_lowercase();
        let value = decode_value(value, &key, line_no, stats);
        args.insert(key, value);
    }
    args
}

fn split_args(inner: &str) -> Vec<&str> {
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut in_quote = false;
    let mut escaped = false;
    for (i, c) in inner.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' if in_quote => escaped = true,
            '"' => in_quote = !in_quote,
            ',' if !in_quote => {
                pieces.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    pieces.push(&inner[start..]);
    pieces
}

fn decode_value(value: &str, key: &str, line_no: u32, stats: &mut ParseStats) -> String {
    let v = value.trim();
    let inner = v
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(v);
    if !inner.contains('\\') {
        return inner.to_owned();
    }
    if let Some(decoded) = decode_escapes(inner) {
        stats.decoded_values += 1;
        return decoded;
    }
    stats.diagnostics.push(ParseDiagnostic {
        line: line_no,
        deviation: Deviation::InvalidEscape,
        detail: key.to_owned(),
    });
    inner.to_owned()
}

fn decode_escapes(s: &str) -> Option<String> {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next()? {
            '"' => out.push('"'),
            '\\' => out.push('\\'),
            '/' => out.push('/'),
            'b' => out.push('\u{8}'),
            'f' => out.push('\u{c}'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            'u' => out.push(decode_unicode_escape(&mut chars)?),
            _ => return None,
        }
    }
    Some(out)
}

fn decode_unicode_escape(chars: &mut std::str::Chars<'_>) -> Option<char> {
    fn hex4(chars: &mut std::str::Chars<'_>) -> Option<u32> {
        let mut n = 0u32;
        for _ in 0..4 {
            n = n * 16 + chars.next()?.to_digit(16)?;
        }
        Some(n)
    }
    let hi = hex4(chars)?;
    if !(0xD800..0xDC00).contains(&hi) {
        return char::from_u32(hi);
    }
    if chars.next()? != '\\' || chars.next()? != 'u' {
        return None;
    }
    let lo = hex4(chars)?;
    if !(0xDC00..0xE000).contains(&lo) {
        return None;
    }
    char::from_u32(0x1_0000 + ((hi - 0xD800) << 10) + (lo - 0xDC00))
}

#[must_use]
pub fn word_count(commands: &[StoryCommand]) -> u32 {
    let mut n = 0u32;
    for c in commands {
        let prose = if PROSE_KINDS.contains(&c.kind.as_str()) {
            c.text.as_deref()
        } else if PROSE_ARG_KINDS.contains(&c.kind.as_str()) {
            c.args.get("text").map(String::as_str)
        } else {
            None
        };
        if let Some(p) = prose {
            #[allow(clippy::cast_possible_truncation)]
            let words = p.split_whitespace().count() as u32;
            n = n.saturating_add(words);
        }
    }
    n
}
