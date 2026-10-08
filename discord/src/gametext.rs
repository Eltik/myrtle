//! Arknights game text: rich-text markup and blackboard templates.
//!
//! Descriptions in the game data look like `ATK <@ba.vup>+{atk:0%}</> for {duration} seconds`.
//! [`interpolate`] fills the `{...}` templates from a blackboard (the key/value list that comes
//! with each skill level, talent, trait or module stage) and [`strip_rich_text`] drops the
//! `<@...>` / `<$...>` / `</>` markup, leaving plain text for Discord.

use std::collections::HashMap;
use std::hash::BuildHasher;

/// Drop the game's rich-text markup (`<$ba.stun>Stun</>`, `<@lv.item>`) but keep angle-bracket
/// names that are plain text (`<Frigid>`).
#[must_use]
pub fn strip_rich_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let tail = &rest[start..];
        let is_markup = tail.starts_with("</>") || tail.starts_with("<$") || tail.starts_with("<@");
        match tail.find('>') {
            Some(end) if is_markup => rest = &tail[end + 1..],
            _ => {
                out.push('<');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Archive, outfit and voice text as plain Discord text.
///
/// `<i>` becomes Discord italics, `<color ...>` and `</color>` drop, `{@nickname}` reads
/// "Doctor" (the frontend's default nickname), and the game's own `<@...>` markup goes as in
/// [`strip_rich_text`]. Those are the only tags the 441 EN records carry in these tables
/// (surveyed 2026-10-08).
#[must_use]
pub fn plain_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find(['<', '{']) {
        out.push_str(&rest[..start]);
        let tail = &rest[start..];
        let lower = tail.get(..11).unwrap_or(tail).to_ascii_lowercase();
        if tail.starts_with("<i>") || tail.starts_with("</i>") {
            out.push('*');
            rest = &tail[tail.find('>').map_or(1, |e| e + 1)..];
        } else if let Some(after) = tail.strip_prefix("</color>") {
            rest = after;
        } else if tail.starts_with("<color")
            && let Some(end) = tail.find('>')
        {
            rest = &tail[end + 1..];
        } else if lower == "{@nickname}" {
            out.push_str("Doctor");
            rest = &tail["{@nickname}".len()..];
        } else {
            out.push_str(&tail[..1]);
            rest = &tail[1..];
        }
    }
    out.push_str(rest);
    strip_rich_text(&out)
}

/// A blackboard keyed by lowercased key: the game matches template keys case-insensitively
/// (`{ATK}` and `{atk}` read the same entry).
#[must_use]
pub fn blackboard<'a>(entries: impl IntoIterator<Item = (&'a str, f64)>) -> HashMap<String, f64> {
    entries
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect()
}

/// Fill every `{key}` / `{key:format}` / `{-key...}` template in `template` from `board`.
///
/// - The key is everything before an optional `:` and matched case-insensitively. Keys may
///   contain `@`, `.`, `[` and `]` (`{attack@atk_scale}`, `{peacok_s_1[crit].prob}`); they are
///   blackboard keys verbatim, not paths.
/// - A leading `-` negates the value: debuffs are stored negative and written `-{-atk:0%}`.
/// - The format is the game's number pattern: `0%` / `0.0%` (times 100, that many decimals,
///   a percent sign), `0` / `0.0` (that many decimals), none (the value without float noise).
///
/// Returns the text and the keys that had no value. An unresolved template stays in the text
/// as written, so a miss is visible rather than silently blank.
#[must_use]
pub fn interpolate<S: BuildHasher>(
    template: &str,
    board: &HashMap<String, f64, S>,
) -> (String, Vec<String>) {
    let mut out = String::with_capacity(template.len());
    let mut missing = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let tail = &rest[open..];
        let Some(close) = tail.find('}') else {
            out.push_str(tail);
            return (out, missing);
        };
        let body = &tail[1..close];
        let (key, format) = match body.split_once(':') {
            Some((k, f)) => (k.trim(), Some(f.trim())),
            None => (body.trim(), None),
        };
        let (negate, key) = match key.strip_prefix('-') {
            Some(k) => (true, k),
            None => (false, key),
        };
        if let Some(&value) = board.get(&key.to_lowercase()) {
            let value = if negate { -value } else { value };
            out.push_str(&format_value(value, format));
        } else {
            missing.push(key.to_lowercase());
            out.push_str(&tail[..=close]);
        }
        rest = &tail[close + 1..];
    }
    out.push_str(rest);
    (out, missing)
}

/// [`interpolate`], then [`strip_rich_text`]: the plain text a description shows in game.
#[must_use]
pub fn render<S: BuildHasher>(
    template: &str,
    board: &HashMap<String, f64, S>,
) -> (String, Vec<String>) {
    let (text, missing) = interpolate(template, board);
    (strip_rich_text(&text), missing)
}

/// Format `value` with the game's pattern (`0%`, `0.0%`, `0`, `0.0`; `None` for plain).
fn format_value(value: f64, format: Option<&str>) -> String {
    let Some(format) = format.filter(|f| !f.is_empty()) else {
        return plain_number(value);
    };
    let (pattern, percent) = match format.strip_suffix('%') {
        Some(p) => (p, true),
        None => (format, false),
    };
    let decimals = pattern.split_once('.').map_or(0, |(_, d)| d.len());
    let scaled = if percent { value * 100.0 } else { value };
    let shown = fixed(scaled, decimals);
    if percent { format!("{shown}%") } else { shown }
}

/// `value` rounded half away from zero to `decimals` places, as the game's formatter does
/// (Rust's `{:.N}` would round 12.5 to even).
fn fixed(value: f64, decimals: usize) -> String {
    let factor = 10_f64.powi(i32::try_from(decimals).unwrap_or(0));
    let rounded = (value * factor).round() / factor;
    // `-0` from a tiny negative reads oddly; it is zero.
    let rounded = if rounded == 0.0 { 0.0 } else { rounded };
    format!("{rounded:.decimals$}")
}

/// A value with no format: whole numbers without a decimal point, and the float noise of the
/// stored `f32` (1.2000000476837158) trimmed away.
fn plain_number(value: f64) -> String {
    let text = fixed(value, 4);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() || text == "-" {
        "0".to_string()
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(entries: &[(&str, f64)]) -> HashMap<String, f64> {
        blackboard(entries.iter().copied())
    }

    #[test]
    fn strips_markup_but_keeps_names() {
        assert_eq!(
            strip_rich_text("Attacks inflict <$ba.dt.neural>Nervous Impairment</>."),
            "Attacks inflict Nervous Impairment."
        );
        assert_eq!(
            strip_rich_text("Attacks in <Frigid> areas deal Arts damage"),
            "Attacks in <Frigid> areas deal Arts damage"
        );
        assert_eq!(strip_rich_text("a < b"), "a < b");
    }

    #[test]
    fn interpolates_real_skill_text() {
        // Exusiai S3 at M3 (char_103_angel), stored values as the API sends them.
        let (text, missing) = render(
            "Shoots <@ba.vup>{attack@times}</> times in a row",
            &board(&[("attack@times", 5.0)]),
        );
        assert_eq!(text, "Shoots 5 times in a row");
        assert!(missing.is_empty());

        // Thorns the Lodestar: debuffs are negative and written with a `-` prefix.
        let (text, _) = render(
            "ATK <@ba.vup>-{-atk:0%}</>, DEF <@ba.vup>-{-def:0%}</>",
            &board(&[
                ("atk", -0.150_000_005_960_464_48),
                ("def", -0.349_999_994_039_535_5),
            ]),
        );
        assert_eq!(text, "ATK -15%, DEF -35%");

        // Float noise from f32 storage, integer vs decimal formats, case-insensitive keys.
        let b = board(&[
            ("ATK_SCALE", 1.100_000_023_841_858),
            ("max_hp", 0.079_999_998_211_860_66),
            ("cost", 2.0),
            ("interval", 0.5),
            ("peacok_s_1[crit].prob", 0.25),
        ]);
        assert_eq!(render("{atk_scale:0%}", &b).0, "110%");
        assert_eq!(render("{max_hp:0.0%}", &b).0, "8.0%");
        assert_eq!(render("{Atk_Scale}", &b).0, "1.1");
        assert_eq!(render("{cost:0}", &b).0, "2");
        assert_eq!(render("{interval:0.0}", &b).0, "0.5");
        assert_eq!(render("{cost}", &b).0, "2");
        assert_eq!(render("{peacok_s_1[crit].prob:0%}", &b).0, "25%");
    }

    #[test]
    fn plain_text_handles_archive_markup() {
        assert_eq!(
            plain_text("<color name=#ffffff>Brand Series/Outfit.</color> Worn <i>often</i>."),
            "Brand Series/Outfit. Worn *often*."
        );
        assert_eq!(plain_text("Morning, {@nickname}."), "Morning, Doctor.");
        assert_eq!(plain_text("{@Nickname}! {x}"), "Doctor! {x}");
        assert_eq!(plain_text("<Frigid> <@ba.kw>kw</>"), "<Frigid> kw");
    }

    #[test]
    fn rounds_half_away_from_zero() {
        assert_eq!(format_value(0.125, Some("0%")), "13%");
        assert_eq!(format_value(-0.125, Some("0%")), "-13%");
        assert_eq!(format_value(2.5, Some("0")), "3");
        assert_eq!(format_value(-0.000_01, Some("0%")), "0%");
    }

    #[test]
    fn leaves_unresolved_templates_visible() {
        let (text, missing) = render("For {duration} seconds, {Unknown:0%}", &board(&[]));
        assert_eq!(text, "For {duration} seconds, {Unknown:0%}");
        assert_eq!(missing, vec!["duration".to_string(), "unknown".to_string()]);
        // An unclosed brace is text, not a template.
        assert_eq!(render("a {b", &board(&[])).0, "a {b");
    }
}
