//! The story script parser: one `.txt` into a generic command stream.
//!
//! The grammar is one command per LOGICAL line. A raw line that ends in a
//! backslash is a CONTINUATION: the client's `_ReadNextBlock` (RVA 0x4a9afc0,
//! `cmp w8, #0x5c`) removes the backslash, reads the next line and appends, in
//! a loop, which is how the tutorial scripts write a `[Tutorial]` across four
//! lines. The joined line keeps the FIRST raw line's number.
//!
//! A logical line that starts with `[` is a command up to its matching `]`
//! (quotes are respected, so a `]` inside a quoted value does not close it);
//! whatever follows the bracket is the command's text. A line with no leading
//! `[` is a bare text line. The bracket holds one of three shapes:
//!
//! ```text
//! [Name(k=v, k2="v")] text     -> kind "name",  args {k, k2}, text
//! [key=value] text             -> kind "key",   args {key: value}, text
//! [Name] text                  -> kind "name",  no args, text
//! ```
//!
//! The second shape is how `[name="Guard"] line` names a speaker, and the
//! corpus also uses it for `[delay=1]`, `[isAvatarRight=false]` and the
//! `[delay9ti=2]` typo. Kinds and argument KEYS are lowercased; values are
//! trimmed, unquoted and JSON-DECODED. Nothing in here interprets a command:
//! scene semantics live in the frontend engine, which is what keeps this
//! parser total over the 4,860 EN files and their 122 distinct command names.
//!
//! The decode is the client's: `_ReplaceEqualSignWithColon` (0x4a9b900)
//! rewrites every `=` outside quotes to `:` and `_ParseCommand` (0x4a9a8e8)
//! hands the result to a JSON deserializer, so `\n` is a real newline, `\t` a
//! tab, `\uXXXX` its code point and `\"` a quote. TWO DELIBERATE DEVIATIONS
//! from the client are recorded as [`ParseDiagnostic`]s rather than silently
//! taken, because each recovers content the game itself drops:
//!
//! * An escape JSON rejects (`\ `, `\l`, `\A`) makes the client's whole
//!   argument block fail to deserialize, so the command runs with NO
//!   arguments. We keep the raw, undecoded value instead.
//! * The client has no typo alias table (there is no such table anywhere in
//!   the binary), so `[palysound]` and friends are silently ignored in-game.
//!   We fold them, which plays sound the game never plays.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One parsed line. `kind` is lowercased and typo-normalised; a bare text line
/// is `kind: "text"` with the whole line in `text`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryCommand {
    pub kind: String,
    pub args: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub text: Option<String>,
    /// 1-based line number in the script file.
    pub line: u32,
}

/// The kinds whose `text` is prose the reader shows, for word counts.
pub const PROSE_KINDS: &[&str] = &["name", "text", "multiline", "narration"];

/// The kinds whose prose sits in `args.text` rather than after the bracket.
pub const PROSE_ARG_KINDS: &[&str] = &["subtitle", "sticker"];

/// Misspelled command names found in the EN corpus (the doc's census, grepped
/// again 2026-09-21 over 4,860 files), mapped to the name they mean. Counts on
/// EN: palysound 4, delya 4, delau 3, delat 3, daley 3, dealy 2, dalay 2,
/// delayt 1, delay9ti 2, stopmucis 1, musicvolune 1, dialogs 1, dialo 1,
/// charslsot 1, chaa 1. `plysound` is not in EN; it is kept for other servers.
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

/// The canonical name for a (lowercased) command kind.
#[must_use]
pub fn normalize_kind(kind: &str) -> &str {
    TYPO_TABLE
        .iter()
        .find(|(typo, _)| *typo == kind)
        .map_or(kind, |(_, canon)| canon)
}

/// Every typo the parser folds, `(typo, canonical)`, for reports and tests.
#[must_use]
pub const fn typo_table() -> &'static [(&'static str, &'static str)] {
    TYPO_TABLE
}

/// Where our parse deliberately differs from the client, per logical line.
/// Counted by the corpus test and reported in `docs/story-reader.md`; nothing
/// in the wire shape carries them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deviation {
    /// An escape the client's JSON decoder rejects. The client drops the
    /// WHOLE argument block and runs the command bare; we keep the raw value.
    InvalidEscape,
    /// A command name the client has no alias for, so it silently ignores the
    /// command; we fold it to the name it means.
    TypoFolded,
}

/// One recorded deviation, with the line it is on and the piece that caused
/// it (the argument key, or the typo as written).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub line: u32,
    pub deviation: Deviation,
    pub detail: String,
}

/// What a parse measured about itself, for the corpus report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParseStats {
    /// Raw lines swallowed by a trailing backslash (a four-line `[Tutorial]`
    /// counts 3).
    pub continued_lines: u32,
    /// Argument values in which at least one escape decoded.
    pub decoded_values: u32,
    /// Logical lines whose argument block the client would have dropped.
    pub dropped_arg_blocks: u32,
    pub diagnostics: Vec<ParseDiagnostic>,
}

/// Parse a whole script. Never fails: a malformed line becomes a text line.
/// An empty bracket (`[]`, one line in the EN corpus) is dropped.
#[must_use]
pub fn parse(script: &str) -> Vec<StoryCommand> {
    parse_with_stats(script).0
}

/// [`parse`] plus what it measured and where it deviated from the client.
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

/// Join every raw line that ends in a backslash to the one after it, the way
/// `_ReadNextBlock` does: drop the backslash, append the next line, repeat.
/// The check is on the RAW line, so a backslash followed by a space does not
/// continue; the joined line carries the first raw line's number and is
/// trimmed once, at the end, exactly as a single line was before.
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
    // A file whose last line ends in a backslash: the client hits EOF and
    // keeps what it has.
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
    // `[[character(fadetime=0.3)]]` (one EN file): the doubled bracket is noise.
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

/// Byte index of the `]` that closes the leading `[`, skipping quoted spans.
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

/// Split the bracket body into `(kind, args)`.
fn parse_head(
    head: &str,
    line_no: u32,
    stats: &mut ParseStats,
) -> (String, BTreeMap<String, String>) {
    let paren = head.find('(');
    let eq = head.find('=');
    // `Name(args)`: the paren comes first (an `=` inside the args is later).
    if let Some(p) = paren
        && eq.is_none_or(|e| p < e)
    {
        let kind = canon_kind(&head[..p], line_no, stats);
        let inner = head[p + 1..].trim_end();
        let inner = inner.strip_suffix(')').unwrap_or(inner);
        return (kind, parse_args(inner, line_no, stats));
    }
    // `key=value`: the key is the kind and its own single argument.
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

/// `k=v, k2 = "a, b"` -> map. Commas inside quotes are part of the value;
/// keys are lowercased; values are trimmed, unquoted and JSON-decoded.
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

/// Strip one pair of surrounding double quotes and decode the JSON escapes
/// inside, recording a diagnostic and keeping the raw text when one of them
/// is an escape JSON rejects.
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

/// The JSON string escapes, and only those: `None` when one is invalid, which
/// is what costs the client the whole argument block.
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

/// `\uXXXX`, with a low surrogate joined to a high one the way a JSON reader
/// does. A lone surrogate is not a character and kills the block.
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

/// Whitespace-separated words over the prose the reader shows.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn one(line: &str) -> StoryCommand {
        let v = parse(line);
        assert_eq!(v.len(), 1, "{line}");
        v.into_iter().next().unwrap()
    }

    #[test]
    fn name_form_is_kind_name_with_text() {
        let c = one(r#"[name="Guard"]   Is it just me, or is the sky getting darker?"#);
        assert_eq!(c.kind, "name");
        assert_eq!(c.args.get("name").map(String::as_str), Some("Guard"));
        assert_eq!(
            c.text.as_deref(),
            Some("Is it just me, or is the sky getting darker?")
        );
        assert_eq!(c.line, 1);
    }

    #[test]
    fn bare_command_has_no_args_or_text() {
        let c = one("[Dialog]");
        assert_eq!(c.kind, "dialog");
        assert!(c.args.is_empty());
        assert_eq!(c.text, None);
    }

    #[test]
    fn header_keeps_trailing_text_and_lowercases_keys() {
        let c = one(
            r#"[HEADER(key="title_test", is_skippable=true, fit_mode="BLACK_MASK")] 第七关（前）"#,
        );
        assert_eq!(c.kind, "header");
        assert_eq!(c.args["key"], "title_test");
        assert_eq!(c.args["is_skippable"], "true");
        assert_eq!(c.args["fit_mode"], "BLACK_MASK");
        assert_eq!(c.text.as_deref(), Some("第七关（前）"));
    }

    #[test]
    fn arg_keys_lowercase_values_keep_case() {
        let c = one(r#"[Background(image="bg_Cher_1", x=0, xScale=1.1, Delay=0.5)]"#);
        assert_eq!(c.args["image"], "bg_Cher_1");
        assert_eq!(c.args["xscale"], "1.1");
        assert_eq!(c.args["delay"], "0.5");
        assert!(!c.args.contains_key("xScale"));
    }

    #[test]
    fn quoted_commas_survive() {
        let c = one(
            r#"[Decision(options="You sure are dedicated.;...How stubborn, really.", values="1;2")]"#,
        );
        assert_eq!(
            c.args["options"],
            "You sure are dedicated.;...How stubborn, really."
        );
        assert_eq!(c.args["values"], "1;2");
    }

    #[test]
    fn spaces_around_equals_and_escaped_quotes() {
        let c = one(
            r#"[Sticker(id="st1", multi = true, text="He said \"no\", twice", x=300,block = true)]"#,
        );
        assert_eq!(c.kind, "sticker");
        assert_eq!(c.args["multi"], "true");
        assert_eq!(c.args["text"], r#"He said "no", twice"#);
        assert_eq!(c.args["x"], "300");
        assert_eq!(c.args["block"], "true");
    }

    #[test]
    fn bare_text_line_is_kind_text() {
        let c = one("   The rain had not stopped for three days.  ");
        assert_eq!(c.kind, "text");
        assert!(c.args.is_empty());
        assert_eq!(
            c.text.as_deref(),
            Some("The rain had not stopped for three days.")
        );
    }

    #[test]
    fn typos_are_normalised() {
        assert_eq!(one(r#"[palysound(key="$x")]"#).kind, "playsound");
        assert_eq!(one("[Delya(time=1)]").kind, "delay");
        assert_eq!(one("[stopmucis]").kind, "stopmusic");
        assert_eq!(one("[musicvolune(volume=0)]").kind, "musicvolume");
        assert_eq!(one("[dialo]").kind, "dialog");
        assert_eq!(one("[charslsot]").kind, "charslot");
        assert_eq!(one("[chaa]").kind, "character");
        let c = one("[delay9ti=2]");
        assert_eq!(c.kind, "delay");
        assert_eq!(c.args["delay"], "2");
    }

    #[test]
    fn key_value_form_generalises() {
        let c = one("[isAvatarRight=false]Databank search detected.");
        assert_eq!(c.kind, "isavatarright");
        assert_eq!(c.args["isavatarright"], "false");
        assert_eq!(c.text.as_deref(), Some("Databank search detected."));
        let c = one("[Delay=(time=1.5)]");
        assert_eq!(c.kind, "delay");
        assert_eq!(c.args["time"], "1.5");
    }

    #[test]
    fn unknown_commands_pass_through_lowercased() {
        let c = one(r#"[SandboxV2(mode="x")]"#);
        assert_eq!(c.kind, "sandboxv2");
        assert_eq!(c.args["mode"], "x");
    }

    #[test]
    fn empty_lines_are_skipped_and_line_numbers_are_file_lines() {
        let v = parse("[Dialog]\n\n\n[name=\"A\"] hi\n");
        assert_eq!(v.len(), 2);
        assert_eq!(v[1].line, 4);
    }

    #[test]
    fn doubled_brackets_and_empty_brackets() {
        let c = one("[[character(fadetime=0.3)]]");
        assert_eq!(c.kind, "character");
        assert_eq!(c.args["fadetime"], "0.3");
        assert_eq!(c.text, None);
        assert!(parse("[]\n").is_empty());
    }

    #[test]
    fn unclosed_bracket_is_text() {
        let c = one("[Broken(line");
        assert_eq!(c.kind, "text");
        assert_eq!(c.text.as_deref(), Some("[Broken(line"));
    }

    #[test]
    fn a_trailing_backslash_joins_the_next_line() {
        let script = concat!(
            "[Tutorial(focusX=178, focusY=176,  \\\n",
            "          animStyle=\"Highlight\", black=\"$f_tut_black\", \\\n",
            "          protectTime=0.5)] \\\n",
            "[name=\"A\"] hi\n"
        );
        let (v, stats) = parse_with_stats(script);
        assert_eq!(v.len(), 1, "{v:?}");
        assert_eq!(v[0].kind, "tutorial");
        assert_eq!(
            v[0].line, 1,
            "the joined line keeps the FIRST line's number"
        );
        assert_eq!(v[0].args["focusx"], "178");
        assert_eq!(v[0].args["animstyle"], "Highlight");
        assert_eq!(v[0].args["protecttime"], "0.5");
        // The third line's trailing backslash swallows the `[name]` line too,
        // which is what the client does.
        assert_eq!(v[0].text.as_deref(), Some("[name=\"A\"] hi"));
        assert_eq!(stats.continued_lines, 3);
    }

    #[test]
    fn a_backslash_followed_by_a_space_does_not_continue() {
        let (v, stats) = parse_with_stats("[Dialog] \\ \n[Blocker]\n");
        assert_eq!(v.len(), 2);
        assert_eq!(stats.continued_lines, 0);
    }

    #[test]
    fn argument_values_are_json_decoded() {
        let (v, stats) = parse_with_stats(
            r#"[Subtitle(text="\n_ESTABLISHING\tSHOT—done", alignment="center")]"#,
        );
        assert_eq!(v[0].args["text"], "\n_ESTABLISHING\tSHOT\u{2014}done");
        assert_eq!(v[0].args["alignment"], "center");
        assert_eq!(stats.decoded_values, 1);
        assert_eq!(stats.dropped_arg_blocks, 0);
    }

    #[test]
    fn an_invalid_escape_keeps_the_raw_value_and_is_recorded() {
        // The client's JSON decoder rejects `\ ` and the WHOLE argument block
        // fails with it, so in-game the sticker runs with no arguments at all.
        // We keep the characters as written and say so.
        let (v, stats) = parse_with_stats(r#"[Sticker(id="s1", text="a\ b\nc")]"#);
        assert_eq!(v[0].args["text"], r"a\ b\nc");
        assert_eq!(v[0].args["id"], "s1");
        assert_eq!(stats.dropped_arg_blocks, 1);
        assert_eq!(stats.decoded_values, 0);
        assert_eq!(stats.diagnostics.len(), 1);
        assert_eq!(stats.diagnostics[0].deviation, Deviation::InvalidEscape);
        assert_eq!(stats.diagnostics[0].detail, "text");
        assert_eq!(stats.diagnostics[0].line, 1);
    }

    #[test]
    fn a_folded_typo_is_recorded_as_a_deviation() {
        let (_, stats) = parse_with_stats("[palysound(key=\"$x\")]\n[playsound(key=\"$y\")]\n");
        assert_eq!(stats.diagnostics.len(), 1);
        assert_eq!(stats.diagnostics[0].deviation, Deviation::TypoFolded);
        assert_eq!(stats.diagnostics[0].detail, "palysound");
    }

    #[test]
    fn escaped_quotes_still_decode_and_surrogate_pairs_join() {
        let c = one(r#"[Sticker(text="He said \"no\", twice 🙂")]"#);
        assert_eq!(c.args["text"], "He said \"no\", twice \u{1f642}");
        // A lone high surrogate is not a character: the raw value, recorded.
        let (v, stats) = parse_with_stats(r#"[Sticker(text="\ud83d only")]"#);
        assert_eq!(v[0].args["text"], r"\ud83d only");
        assert_eq!(stats.dropped_arg_blocks, 1);
    }

    #[test]
    fn word_count_covers_prose_kinds_only() {
        let v = parse(
            "[name=\"A\"] one two three\nfour five\n[multiline(name=\"B\")] six\n[Subtitle(text=\"seven eight\")]\n[Sticker(text=\"nine\")]\n[Background(image=\"ten eleven\")]\n[animtext(id=\"x\")] twelve",
        );
        assert_eq!(word_count(&v), 9);
    }
}
