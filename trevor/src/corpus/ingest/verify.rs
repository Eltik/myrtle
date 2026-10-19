//! Verification: a story's chunks must reproduce its prose word for word, by the backend's own word rule.

use std::collections::BTreeSet;

use crate::corpus::chunk::{Chunk, PROSE_ARG_KINDS, PROSE_KINDS, StoryCommand};

/// Prose words of a script by the backend's own rule
/// (`parser::word_count`, parser.rs:427).
pub(super) fn prose_words(commands: &[StoryCommand]) -> Vec<&str> {
    commands
        .iter()
        .filter_map(|c| {
            if PROSE_KINDS.contains(&c.kind.as_str()) {
                c.text.as_deref()
            } else if PROSE_ARG_KINDS.contains(&c.kind.as_str()) {
                c.args.get("text").map(String::as_str)
            } else {
                None
            }
        })
        .filter(|t| !t.trim().is_empty())
        .flat_map(str::split_whitespace)
        .collect()
}

/// True when the chunks reproduce the script's prose exactly, word for word,
/// with no loss and no duplication. Each rendered line's `Speaker: ` prefix
/// is removed using the story's own speaker strings, longest first, so a
/// multi-word speaker is stripped whole.
#[must_use]
pub fn reconstructs(commands: &[StoryCommand], chunks: &[Chunk]) -> bool {
    let speakers: BTreeSet<&str> = commands
        .iter()
        .filter(|c| c.kind == "name")
        .filter_map(|c| c.args.get("name").map(String::as_str))
        .filter(|s| !s.trim().is_empty())
        .collect();
    let mut by_len: Vec<&str> = speakers.into_iter().collect();
    by_len.sort_by_key(|s| std::cmp::Reverse(s.len()));

    let want = prose_words(commands);
    let mut got: Vec<&str> = Vec::with_capacity(want.len());
    for c in chunks {
        for line in c.text.split('\n') {
            let body = by_len
                .iter()
                .find_map(|s| line.strip_prefix(s).and_then(|r| r.strip_prefix(": ")))
                .unwrap_or(line);
            got.extend(body.split_whitespace());
        }
    }
    got == want
}
