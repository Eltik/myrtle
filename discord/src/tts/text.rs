//! Turning a Discord message into the words the voice says.
//!
//! Everything here is pure: mentions resolve through a caller-supplied closure, so the rules are
//! tested without a cache. The order matters and is fixed in [`clean`]: code blocks first (their
//! insides would otherwise be read as markdown), then `<...>` tokens (mentions, emoji, timestamps,
//! bracketed links), then masked links, then bare URLs, then markdown symbols, then repeats.

/// A `<...>` mention the caller resolves to a spoken name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mention {
    User(u64),
    Role(u64),
    Channel(u64),
}

/// The word spoken in place of every URL.
pub const LINK_WORD: &str = "link";

/// The words spoken in place of a fenced code block.
const CODE_BLOCK_WORDS: &str = "code block";

/// Longest run of one character that survives; `soooooo` -> `sooo`, `!!!!!!` -> `!!!`.
const MAX_REPEAT: usize = 3;

/// The spoken form of `content`, at most `max_chars` characters, or `None` when nothing worth
/// saying is left (empty, attachments only, punctuation or emoji only).
///
/// `resolve` names a mention; a mention it can't name is dropped rather than read as digits.
#[must_use]
pub fn clean(
    content: &str,
    resolve: &dyn Fn(Mention) -> Option<String>,
    max_chars: usize,
) -> Option<String> {
    let text = replace_code_blocks(content);
    let text = replace_angle_tokens(&text, resolve);
    let text = replace_masked_links(&text);
    let text = replace_bare_urls(&text);
    let text = strip_markdown(&text);
    let text = collapse_repeats(&text, MAX_REPEAT);
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if !text.chars().any(char::is_alphanumeric) {
        return None;
    }
    Some(cap_at_word(&text, max_chars))
}

/// Fenced code blocks become [`CODE_BLOCK_WORDS`]; reading code aloud helps nobody. An unclosed
/// fence is left for [`strip_markdown`] to drop the backticks from.
fn replace_code_blocks(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("```") {
        let after = &rest[start + 3..];
        let Some(len) = after.find("```") else {
            break;
        };
        out.push_str(&rest[..start]);
        out.push(' ');
        out.push_str(CODE_BLOCK_WORDS);
        out.push(' ');
        rest = &after[len + 3..];
    }
    out.push_str(rest);
    out
}

/// Replace every recognised `<...>` token: user, role and channel mentions by name, custom emoji
/// by name (a run of the same emoji once), timestamps and unknown names dropped, `<https://...>`
/// as [`LINK_WORD`], `</command:id>` as the command. Anything else, like `<3`, is left as typed.
fn replace_angle_tokens(s: &str, resolve: &dyn Fn(Mention) -> Option<String>) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    let mut last_emoji: Option<String> = None;
    while let Some(open) = rest.find('<') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('>') else {
            break;
        };
        let inner = &after[..close];
        let Some(token) = angle_token(inner, resolve) else {
            // Not ours: keep the `<` and carry on scanning just past it.
            out.push_str(&rest[..=open]);
            rest = after;
            continue;
        };
        out.push_str(&rest[..open]);
        let between = rest[..open].trim();
        match token {
            Token::Emoji(name) => {
                if !(between.is_empty() && last_emoji.as_deref() == Some(name.as_str())) {
                    push_spaced(&mut out, &name);
                }
                last_emoji = Some(name);
            }
            Token::Text(text) => {
                push_spaced(&mut out, &text);
                last_emoji = None;
            }
        }
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}

/// What one `<...>` token reads as.
enum Token {
    Emoji(String),
    Text(String),
}

/// Parse the inside of one `<...>`, or `None` when it isn't a token Discord renders.
fn angle_token(inner: &str, resolve: &dyn Fn(Mention) -> Option<String>) -> Option<Token> {
    let id = |digits: &str| -> Option<u64> {
        (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
            .then(|| digits.parse().ok())
            .flatten()
    };
    let named = |m: Mention| Some(Token::Text(resolve(m).unwrap_or_default()));

    if inner.starts_with("http://") || inner.starts_with("https://") {
        return (!inner.contains(char::is_whitespace)).then(|| Token::Text(LINK_WORD.into()));
    }
    if let Some(rest) = inner.strip_prefix("@&") {
        return named(Mention::Role(id(rest)?));
    }
    if let Some(rest) = inner.strip_prefix('@') {
        let rest = rest.strip_prefix('!').unwrap_or(rest);
        return named(Mention::User(id(rest)?));
    }
    if let Some(rest) = inner.strip_prefix('#') {
        return named(Mention::Channel(id(rest)?));
    }
    if let Some(rest) = inner.strip_prefix("t:") {
        let stamp = rest.split_once(':').map_or(rest, |(t, _)| t);
        id(stamp)?;
        return Some(Token::Text(String::new()));
    }
    if let Some(rest) = inner.strip_prefix('/') {
        let (command, snowflake) = rest.rsplit_once(':')?;
        id(snowflake)?;
        return Some(Token::Text(command.to_string()));
    }
    let emoji = inner
        .strip_prefix("a:")
        .or_else(|| inner.strip_prefix(':'))?;
    let (name, snowflake) = emoji.rsplit_once(':')?;
    id(snowflake)?;
    (!name.is_empty() && !name.contains(char::is_whitespace))
        .then(|| Token::Emoji(name.to_string()))
}

/// Append `word` with a space on either side, so a replaced token never glues to its neighbours.
fn push_spaced(out: &mut String, word: &str) {
    out.push(' ');
    out.push_str(word);
    out.push(' ');
}

/// `[label](https://...)` reads as its label.
fn replace_masked_links(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(mid) = rest.find("](http") {
        let head = &rest[..mid];
        let tail = &rest[mid + 2..];
        match (head.rfind('['), tail.find(')')) {
            (Some(open), Some(close)) if !tail[..close].contains(char::is_whitespace) => {
                out.push_str(&head[..open]);
                push_spaced(&mut out, &head[open + 1..]);
                rest = &tail[close + 1..];
            }
            _ => {
                out.push_str(&rest[..mid + 2]);
                rest = tail;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Every word that carries a URL becomes [`LINK_WORD`]; line breaks are kept for
/// [`strip_markdown`], which reads them as pauses.
fn replace_bare_urls(s: &str) -> String {
    s.split('\n')
        .map(|line| {
            line.split(' ')
                .map(|word| {
                    let lower = word.to_ascii_lowercase();
                    if lower.contains("http://")
                        || lower.contains("https://")
                        || lower.starts_with("www.")
                    {
                        LINK_WORD
                    } else {
                        word
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Drop markdown that would otherwise be read out or garble the phonemiser: emphasis, strike,
/// spoiler and code marks anywhere, and quote, header and subtext markers at a line's start.
/// Underscores become spaces, so `snake_case` and emoji names read as words. A line break becomes
/// a comma, a pause, unless the line already ends in punctuation; other control characters go.
fn strip_markdown(s: &str) -> String {
    let lines: Vec<String> = s
        .lines()
        .map(|line| {
            let mut line = line.trim_start();
            loop {
                let next = line
                    .strip_prefix("-# ")
                    .or_else(|| line.strip_prefix('>'))
                    .or_else(|| line.strip_prefix('#'))
                    .map_or(line, str::trim_start);
                if next.len() == line.len() {
                    break;
                }
                line = next;
            }
            line.chars()
                .filter(|c| !matches!(c, '*' | '~' | '`' | '|' | '[' | ']') && !c.is_control())
                .map(|c| if c == '_' { ' ' } else { c })
                .collect::<String>()
                .trim()
                .to_string()
        })
        .filter(|line| !line.is_empty())
        .collect();

    let mut out = String::new();
    for line in lines {
        if !out.is_empty() {
            if !out.ends_with(['.', '!', '?', ',', ';', ':']) {
                out.push(',');
            }
            out.push(' ');
        }
        out.push_str(&line);
    }
    out
}

/// Cut every run of one character down to `max`.
fn collapse_repeats(s: &str, max: usize) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last = None;
    let mut run = 0;
    for c in s.chars() {
        if Some(c) == last {
            run += 1;
        } else {
            last = Some(c);
            run = 1;
        }
        if run <= max {
            out.push(c);
        }
    }
    out
}

/// `s` cut to at most `max` characters, at the last space when one falls in the second half of
/// the cut, otherwise mid-word. Counts characters, not bytes.
fn cap_at_word(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max).collect();
    let next_is_space = s.chars().nth(max).is_some_and(char::is_whitespace);
    if next_is_space {
        return cut.trim_end().to_string();
    }
    match cut.rfind(' ') {
        Some(space) if cut[..space].chars().count() >= max / 2 => cut[..space].trim_end().into(),
        _ => cut.trim_end().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(m: Mention) -> Option<String> {
        match m {
            Mention::User(1) => Some("Amiya".into()),
            Mention::Role(2) => Some("Doctors".into()),
            Mention::Channel(3) => Some("general".into()),
            _ => None,
        }
    }

    fn say(s: &str) -> Option<String> {
        clean(s, &names, 300)
    }

    #[test]
    fn mentions_read_as_names() {
        assert_eq!(
            say("hi <@1> and <@!1>").as_deref(),
            Some("hi Amiya and Amiya")
        );
        assert_eq!(
            say("ping <@&2> in <#3>").as_deref(),
            Some("ping Doctors in general")
        );
        // An id the resolver can't name is dropped, never read as digits.
        assert_eq!(say("hey <@999> come").as_deref(), Some("hey come"));
        assert_eq!(say("<@999>"), None);
    }

    #[test]
    fn emoji_read_as_names_once_per_run() {
        assert_eq!(
            say("nice <:pepe_laugh:123>").as_deref(),
            Some("nice pepe laugh")
        );
        assert_eq!(
            say("<a:wave:1><a:wave:1> <a:wave:1>").as_deref(),
            Some("wave")
        );
        assert_eq!(
            say("<:a:1> then <:a:1>").as_deref(),
            Some("a then a"),
            "the same emoji after other words is a new run"
        );
        // Unicode emoji alone are not speech.
        assert_eq!(say("😂😂😂"), None);
    }

    #[test]
    fn other_angle_tokens() {
        assert_eq!(say("at <t:1700000000:R> ok").as_deref(), Some("at ok"));
        assert_eq!(
            say("run </tts status:42>").as_deref(),
            Some("run tts status")
        );
        assert_eq!(say("i <3 you").as_deref(), Some("i <3 you"));
        assert_eq!(say("a < b > c").as_deref(), Some("a < b > c"));
    }

    #[test]
    fn urls_read_as_link() {
        assert_eq!(
            say("see https://myrtle.moe/x?y=1 now").as_deref(),
            Some("see link now")
        );
        assert_eq!(
            say("see <https://example.com>").as_deref(),
            Some("see link")
        );
        assert_eq!(say("www.example.com").as_deref(), Some("link"));
        assert_eq!(
            say("read [the docs](https://example.com/docs) please").as_deref(),
            Some("read the docs please")
        );
    }

    #[test]
    fn markdown_symbols_go() {
        assert_eq!(
            say("**bold** _it_ ~~gone~~ ||spoiler||").as_deref(),
            Some("bold it gone spoiler")
        );
        assert_eq!(
            say("> quoted\n# Header\n-# small").as_deref(),
            Some("quoted, Header, small")
        );
        assert_eq!(say("one.\ntwo").as_deref(), Some("one. two"));
        assert_eq!(
            say("look ```rust\nfn main() {}\n``` there").as_deref(),
            Some("look code block there")
        );
        assert_eq!(
            say("inline `code` here").as_deref(),
            Some("inline code here")
        );
        assert_eq!(say("snake_case_name").as_deref(), Some("snake case name"));
    }

    #[test]
    fn repeats_collapse() {
        assert_eq!(
            say("soooooooo good!!!!!!!").as_deref(),
            Some("sooo good!!!")
        );
        assert_eq!(say("aaa").as_deref(), Some("aaa"));
    }

    #[test]
    fn nothing_to_say() {
        assert_eq!(say(""), None);
        assert_eq!(say("   \n  "), None);
        assert_eq!(say("?!... ,,"), None);
        assert_eq!(say("**__~~||``||~~__**"), None);
    }

    #[test]
    fn controls_are_dropped() {
        assert_eq!(say("a\u{0}b\u{7}c").as_deref(), Some("abc"));
    }

    #[test]
    fn cap_cuts_at_a_word() {
        assert_eq!(
            clean("one two three", &names, 9).as_deref(),
            Some("one two")
        );
        assert_eq!(
            clean("one two three", &names, 7).as_deref(),
            Some("one two")
        );
        assert_eq!(
            clean("one two three", &names, 13).as_deref(),
            Some("one two three")
        );
        // No space in the second half: cut mid-word rather than drop most of the text.
        assert_eq!(
            clean("a bcdefghijkl", &names, 8).as_deref(),
            Some("a bcdefg")
        );
        // Characters, not bytes.
        assert_eq!(
            clean("Wiš'adel Wiš'adel", &names, 10).as_deref(),
            Some("Wiš'adel")
        );
        let long = "word ".repeat(100);
        let capped = clean(&long, &names, 300).unwrap();
        assert!(capped.chars().count() <= 300);
        assert!(capped.ends_with("word"));
    }
}
