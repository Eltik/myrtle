//! Name matching for game-data lookups.
//!
//! The backend has no name-search endpoint, so `/collection` matches against the cached lists
//! locally. Everything goes through [`fold`] first: case, whitespace, punctuation and
//! diacritics all drop out, so "wis adel" finds "Wiš'adel" and "ce6" finds "CE-6".

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// How well a query matched one candidate string. Declaration order is rank order: a lower
/// variant always beats a higher one, whatever the tie-breaks below it say.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    /// The folded query is the whole folded candidate.
    Exact,
    /// The candidate starts with the query.
    Prefix,
    /// A later word of the candidate starts with the query ("slug" in "Originium Slug").
    WordPrefix,
    /// The query appears anywhere else.
    Substring,
}

/// Fold `s` to its match key: lowercase, diacritics stripped, only letters and digits kept.
#[must_use]
pub fn fold(s: &str) -> String {
    folded_chars(s).filter(|c| c.is_alphanumeric()).collect()
}

/// The folded words of `s`, split wherever a character is neither a letter nor a digit.
fn fold_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    for c in folded_chars(s) {
        if c.is_alphanumeric() {
            current.push(c);
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// Canonical decomposition with the combining marks dropped, lowercased. Letters with no
/// decomposition (like "ø") pass through unchanged, which is still consistent on both sides.
fn folded_chars(s: &str) -> impl Iterator<Item = char> + '_ {
    s.nfkd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
}

/// Rank one `candidate` against an already-folded `query`, or `None` when it doesn't match.
#[must_use]
pub fn rank(query: &str, candidate: &str) -> Option<Rank> {
    if query.is_empty() {
        return None;
    }
    let key = fold(candidate);
    if key == query {
        return Some(Rank::Exact);
    }
    if key.starts_with(query) {
        return Some(Rank::Prefix);
    }
    // A word prefix may run across word breaks, so "the holung" (folded "theholung") finds
    // "Ch'en the Holungday" even though no single word starts with all of it.
    let words = fold_words(candidate);
    for start in 1..words.len() {
        if words[start..].concat().starts_with(query) {
            return Some(Rank::WordPrefix);
        }
    }
    key.contains(query).then_some(Rank::Substring)
}

/// The best rank any of `names` reaches against `query` (already folded).
#[must_use]
pub fn best_rank<'a>(query: &str, names: impl IntoIterator<Item = &'a str>) -> Option<Rank> {
    names.into_iter().filter_map(|n| rank(query, n)).min()
}

/// Rank `items` against the raw `query` and return the best `limit` of them.
///
/// `names` lists the strings an item answers to (name, appellation, stage code...).
/// `priority` breaks ties inside a rank, lower first; after it, the shorter primary name wins,
/// then the original order, so equally good hits come back in a stable order.
pub fn search<'a, T>(
    items: &'a [T],
    query: &str,
    limit: usize,
    names: impl Fn(&'a T) -> Vec<&'a str>,
    priority: impl Fn(&'a T) -> u8,
) -> Vec<&'a T> {
    let query = fold(query);
    let mut hits: Vec<(Rank, u8, usize, usize, &T)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            let item_names = names(item);
            let primary_len = item_names.first().map_or(0, |n| n.chars().count());
            best_rank(&query, item_names).map(|r| (r, priority(item), primary_len, i, item))
        })
        .collect();
    hits.sort_unstable_by_key(|&(r, p, len, i, _)| (r, p, len, i));
    hits.into_iter()
        .take(limit)
        .map(|(_, _, _, _, item)| item)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_drops_case_punctuation_and_diacritics() {
        assert_eq!(fold("Wiš'adel"), "wisadel");
        assert_eq!(fold("Wis'adel"), "wisadel");
        assert_eq!(fold("  CE-6 "), "ce6");
        assert_eq!(fold("Ch'en the Holungday"), "chentheholungday");
        assert_eq!(fold("Pozëmka"), "pozemka");
        assert_eq!(fold("Истина"), "истина");
    }

    #[test]
    fn rank_orders_exact_prefix_word_substring() {
        assert_eq!(rank(&fold("wis'adel"), "Wiš'adel"), Some(Rank::Exact));
        assert_eq!(rank(&fold("ceo"), "Ceobe"), Some(Rank::Prefix));
        assert_eq!(
            rank(&fold("slug"), "Originium Slug"),
            Some(Rank::WordPrefix)
        );
        assert_eq!(
            rank(&fold("the holung"), "Ch'en the Holungday"),
            Some(Rank::WordPrefix)
        );
        assert_eq!(rank(&fold("obe"), "Ceobe"), Some(Rank::Substring));
        assert_eq!(rank(&fold("xyz"), "Ceobe"), None);
        assert_eq!(rank("", "Ceobe"), None);
    }

    #[test]
    fn search_ranks_and_breaks_ties() {
        let items = ["Ceobe", "Ceylon", "Pudding", "Ce", "Amiya"];
        let hits = search(&items, "ce", 10, |s| vec![*s], |_| 0);
        assert_eq!(hits, vec![&"Ce", &"Ceobe", &"Ceylon"]);

        // Stage codes: "1-7" must find code "1-7" ahead of "11-7" and "S1-7".
        let codes = ["11-7", "S1-7", "1-7", "1-70"];
        let hits = search(&codes, "1-7", 10, |s| vec![*s], |_| 0);
        assert_eq!(hits.first(), Some(&&"1-7"));

        // Priority beats length inside a rank.
        let forms = [("Amiya", 1_u8), ("Amiya", 0_u8)];
        let hits = search(&forms, "amiya", 10, |f| vec![f.0], |f| f.1);
        assert_eq!(hits, vec![&forms[1], &forms[0]]);
    }
}
