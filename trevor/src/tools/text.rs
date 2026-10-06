//! Name and text matching shared by every tool: folding, whole-word tests, episode numbers and the fuzzy
//! match of a misspelled name.


/// Drop "[3]", "[2, 3, 5]" citations and the space before them.
pub(super) fn strip_citations(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    let mut rest = t;
    while let Some(i) = rest.find('[') {
        let close = rest[i..].find(']').map(|j| i + j);
        match close {
            Some(j) if j > i + 1 && rest[i + 1..j].chars().all(|c| c.is_ascii_digit() || c == ',' || c == ' ') => {
                out.push_str(rest[..i].trim_end_matches(' '));
                rest = &rest[j + 1..];
            }
            _ => {
                out.push_str(&rest[..=i]);
                rest = &rest[i + 1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Lowercase, accents folded, apostrophes dropped ("Ch'en" is "chen"), other punctuation as spaces, spaces single.
#[must_use]
pub fn norm(x: &str) -> String {
    let mut out = String::with_capacity(x.len());
    for c in x.chars().flat_map(char::to_lowercase) {
        let c = match c {
            'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            'š' => 's',
            c => c,
        };
        if matches!(c, '\'' | '\u{2019}' | '\u{2018}' | '`' | '\u{b4}') {
            continue;
        }
        out.push(if c.is_alphanumeric() { c } else { ' ' });
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Byte offsets of the occurrences of `f` in `text` as a whole word: case-sensitive, with no alphanumeric character
/// right before or after it.
pub fn whole_word_matches<'a>(text: &'a str, f: &'a str) -> impl Iterator<Item = usize> + 'a {
    text.match_indices(f).filter(move |(i, _)| {
        let before = text[..*i].chars().next_back();
        let after = text[i + f.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    }).map(|(i, _)| i)
}

/// `hay` contains `needle` as whole words (both already normalized).
#[must_use]
pub fn contains_words(hay: &str, needle: &str) -> bool {
    !needle.is_empty() && format!(" {hay} ").contains(&format!(" {needle} "))
}

/// The longest value `text` holds as whole words (a plural "s" allowed), for an argument with extra words.
pub(super) fn contained_value<'a>(text: &str, vals: &[&'a str]) -> Option<&'a str> {
    let n = norm(text);
    vals.iter().filter(|v| { let vn = norm(v); vn.len() >= 3 && (contains_words(&n, &vn) || contains_words(&n, &format!("{vn}s"))) })
        .max_by_key(|v| v.len()).copied()
}

/// "episode 7", "chapter 14", "ep 7", "ep07", "ch14", "main 7" or a bare "7" in a normalized name.
pub(super) fn episode_number(n: &str) -> Option<u64> {
    let t: Vec<&str> = n.split(' ').collect();
    if t.len() == 1 {
        if let Ok(x) = t[0].parse() {
            return Some(x);
        }
    }
    for (i, w) in t.iter().enumerate() {
        if matches!(*w, "episode" | "chapter" | "ep" | "ch" | "main") {
            if let Some(x) = t.get(i + 1).and_then(|x| x.parse().ok()) {
                return Some(x);
            }
        }
        for p in ["episode", "chapter", "ep", "ch"] {
            if let Some(x) = w.strip_prefix(p).and_then(|x| x.parse().ok()) {
                return Some(x);
            }
        }
    }
    None
}

/// The one candidate within the edit budget with the smallest distance; none on a tie or past the budget.
/// Budget: 0 under 5 characters (Ines is not Inez), 1 under 9, 2 from 9.
pub(super) fn fuzzy_unique<'a, T: Copy>(n: &str, cands: impl Iterator<Item = (T, &'a str)>) -> Option<T> {
    let len = n.chars().count();
    let budget = if len < 5 { 0 } else if len < 9 { 1 } else { 2 };
    if budget == 0 {
        return None;
    }
    let mut best: Option<(usize, T)> = None;
    let mut tie = false;
    for (c, cn) in cands {
        if cn.chars().count().abs_diff(len) > budget {
            continue;
        }
        let d = levenshtein(n, cn);
        if d > budget {
            continue;
        }
        match best {
            Some((bd, _)) if d > bd => {}
            Some((bd, _)) if d == bd => tie = true,
            _ => { best = Some((d, c)); tie = false; }
        }
    }
    if tie { None } else { best.map(|(_, c)| c) }
}

pub(super) fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            cur[j + 1] = (prev[j] + usize::from(ca != *cb)).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}
