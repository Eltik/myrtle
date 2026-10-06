//! The lexical analyzer, and the single funnel through which every index is
//! opened.
//!
//! Postgres full-text search is disqualified for this corpus: its parser
//! splits on apostrophes and then stems the remainder, so `to_tsvector('peter
//! o''toole')` yields `'o':2 'peter':1 'tool':3`. Applied to `Kal'tsit` that
//! destroys the single most important token class in an Arknights corpus, and
//! the parser cannot be fixed without a C extension.
//!
//! tantivy's default `SimpleTokenizer` has the same problem, but tantivy lets
//! the tokenizer be replaced, which is the whole point. `RegexTokenizer`
//! matches TOKENS, not separators (its `advance()` calls `regex.find`), so the
//! pattern below captures a run of letters or digits optionally continued by
//! `'`, `U+2019`, `-` or `.` when and only when more letters or digits follow.
//! That single constraint drops trailing punctuation without lookahead, which
//! the `regex` crate does not support.
//!
//! Two failure modes here are silent and both are handled by
//! [`LoreNormalizer`]:
//!
//! * A curly apostrophe is a different byte sequence from an ASCII one, so
//!   `kal’tsit` and `kal'tsit` become different terms and a query in one
//!   spelling scores zero against a corpus in the other. Game script dumps mix
//!   both. Fold U+2019 onto `'`.
//! * `Amiya's` survives as one token and therefore does not match a query for
//!   `amiya`. In a dialogue corpus that is a real recall hole. Strip the
//!   English possessive after lowercasing.
//!
//! No stemmer is registered. Stemming destroys invented proper nouns and buys
//! almost nothing on a corpus whose distinctive vocabulary is names.

use tantivy::TantivyError;
use tantivy::schema::{IndexRecordOption, STORED, STRING, Schema, TextFieldIndexing, TextOptions};
use tantivy::tokenizer::{
    LowerCaser, RegexTokenizer, RemoveLongFilter, TextAnalyzer, Token, TokenFilter, TokenStream,
    Tokenizer,
};

pub const LORE_TOKENIZER: &str = "lore";

/// Letters/digits, optionally joined by `'`, `U+2019`, `-` or `.` when
/// followed by more letters or digits. Keeps `Kal'tsit`, `Ch'en`,
/// `Ifrit-Nian` and `W` whole; drops the period from `Kal'tsit.` and from a
/// sentence-final `Dr.`.
pub const LORE_PATTERN: &str = r"[\p{L}\p{N}]+(?:['\u{2019}\-.][\p{L}\p{N}]+)*";

/// Folds `U+2019` to `'` and strips the English possessive.
#[derive(Clone, Copy, Debug)]
pub struct LoreNormalizer;

impl TokenFilter for LoreNormalizer {
    type Tokenizer<T: Tokenizer> = LoreNormalizerFilter<T>;
    fn transform<T: Tokenizer>(self, tokenizer: T) -> LoreNormalizerFilter<T> {
        LoreNormalizerFilter { inner: tokenizer }
    }
}

#[derive(Clone, Debug)]
pub struct LoreNormalizerFilter<T> {
    inner: T,
}

impl<T: Tokenizer> Tokenizer for LoreNormalizerFilter<T> {
    type TokenStream<'a> = LoreNormalizerStream<T::TokenStream<'a>>;
    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        LoreNormalizerStream {
            tail: self.inner.token_stream(text),
        }
    }
}

pub struct LoreNormalizerStream<T> {
    tail: T,
}

impl<T: TokenStream> TokenStream for LoreNormalizerStream<T> {
    fn advance(&mut self) -> bool {
        if !self.tail.advance() {
            return false;
        }
        let token = self.tail.token_mut();
        if token.text.contains('\u{2019}') {
            let folded = token.text.replace('\u{2019}', "'");
            token.text.clear();
            token.text.push_str(&folded);
        }
        // Runs after LowerCaser, so only the lowercase form needs checking.
        // The length guard keeps a bare "'s" from becoming empty.
        if token.text.len() > 3 && token.text.ends_with("'s") {
            let keep = token.text.len() - 2;
            token.text.truncate(keep);
        }
        true
    }
    fn token(&self) -> &Token {
        self.tail.token()
    }
    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}

/// # Errors
/// Returns `TantivyError::InvalidArgument` when the pattern fails to compile.
pub fn lore_analyzer() -> Result<TextAnalyzer, TantivyError> {
    Ok(TextAnalyzer::builder(RegexTokenizer::new(LORE_PATTERN)?)
        .filter(RemoveLongFilter::limit(64))
        .filter(LowerCaser)
        .filter(LoreNormalizer)
        .build())
}

pub struct LoreSchema {
    pub schema: Schema,
    pub chunk_id: tantivy::schema::Field,
    pub body: tantivy::schema::Field,
}

/// `TextFieldIndexing::default()` yields `IndexRecordOption::Basic`, which is
/// NOT what `TEXT` gives you: no positions and no term frequencies. Phrase
/// queries then fail loudly, but BM25 degrades silently, which is worse.
/// `set_index_option` here is load-bearing.
#[must_use]
pub fn lore_schema() -> LoreSchema {
    let mut sb = Schema::builder();
    let chunk_id = sb.add_text_field("chunk_id", STRING | STORED);
    let indexing = TextFieldIndexing::default()
        .set_tokenizer(LORE_TOKENIZER)
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    // Deliberately not stored: the text lives beside the vectors. Measured on
    // 14k docs, storing the body costs 8.46 MB against 2.78 MB unstored.
    let body = sb.add_text_field(
        "body",
        TextOptions::default().set_indexing_options(indexing),
    );
    LoreSchema {
        schema: sb.build(),
        chunk_id,
        body,
    }
}

/// Open or create the index, registering the analyzer.
///
/// EVERY caller must come through here. `Index::open_in_dir` assigns a fresh
/// `TokenizerManager::default()` unconditionally and nothing about analyzers
/// is persisted in `meta.json`, so a reopened index has no `lore` tokenizer.
/// Writing then fails loudly and `QueryParser` fails loudly, but a hand-built
/// `TermQuery` returns wrong results with no error at all. Since the query
/// path here builds terms by hand (the query parser cannot parse a bare
/// apostrophe: `parse_query("kal'tsit")` is a `SyntaxError`), that silent path
/// is the one that matters.
///
/// # Errors
/// Propagates tantivy's directory, schema and tokenizer errors.
pub fn open_index(path: &std::path::Path) -> Result<tantivy::Index, TantivyError> {
    let LoreSchema { schema, .. } = lore_schema();
    let index = if path.join("meta.json").exists() {
        tantivy::Index::open_in_dir(path)?
    } else {
        std::fs::create_dir_all(path).map_err(|e| TantivyError::IoError(std::sync::Arc::new(e)))?;
        tantivy::Index::create_in_dir(path, schema)?
    };
    index
        .tokenizers()
        .register(LORE_TOKENIZER, lore_analyzer()?);
    Ok(index)
}

/// Run text through the registered analyzer. Query terms must be built from
/// this, never from raw input, or they will not match the indexed form.
///
/// # Errors
/// Returns `InvalidArgument` when the analyzer is not registered on `index`,
/// which can only happen if something bypassed [`open_index`].
pub fn analyze(index: &tantivy::Index, text: &str) -> Result<Vec<String>, TantivyError> {
    let mut analyzer = index.tokenizers().get(LORE_TOKENIZER).ok_or_else(|| {
        TantivyError::InvalidArgument(format!("{LORE_TOKENIZER} tokenizer is not registered"))
    })?;
    let mut stream = analyzer.token_stream(text);
    let mut out = Vec::new();
    while stream.advance() {
        out.push(stream.token().text.clone());
    }
    Ok(out)
}

/// Reciprocal rank fusion over ranked lists, best first.
///
/// `k` damps the head of each list. It is a tuning knob, not a settled
/// constant: the widely repeated "k=60" comes from Cormack 2009, and the claim
/// that k=20 is measurably better traces to an analytic blog argument with no
/// retrieval measurement behind it. The one real sweep found k=32 peaking at
/// nDCG@10 0.899 against k=60's 0.881, with everything from 10 to 100 inside
/// 0.868-0.899. A gentle hill. Default 20, tune against the gold set, and do
/// not spend much on it.
#[must_use]
pub fn rrf(lists: &[Vec<u64>], k: f32) -> Vec<(u64, f32)> {
    use std::collections::HashMap;
    let mut acc: HashMap<u64, f32> = HashMap::new();
    for list in lists {
        for (i, doc) in list.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let rank = i as f32 + 1.0;
            *acc.entry(*doc).or_insert(0.0) += 1.0 / (k + rank);
        }
    }
    let mut fused: Vec<(u64, f32)> = acc.into_iter().collect();
    // Tie-break on id so the output is deterministic.
    fused.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(text: &str) -> Vec<String> {
        let mut a = lore_analyzer().expect("analyzer builds");
        let mut s = a.token_stream(text);
        let mut out = Vec::new();
        while s.advance() {
            out.push(s.token().text.clone());
        }
        out
    }

    #[test]
    fn invented_proper_nouns_survive_whole() {
        let got = toks("Dr. Kal'tsit told Ifrit-Nian that W and Ch'en left. Kal'tsit.");
        assert_eq!(
            got,
            vec![
                "dr",
                "kal'tsit",
                "told",
                "ifrit-nian",
                "that",
                "w",
                "and",
                "ch'en",
                "left",
                "kal'tsit"
            ]
        );
    }

    #[test]
    fn curly_and_straight_apostrophes_unify() {
        assert_eq!(toks("Kal’tsit"), toks("Kal'tsit"));
        assert_eq!(toks("Kal’tsit"), vec!["kal'tsit".to_owned()]);
    }

    #[test]
    fn possessives_match_the_bare_name() {
        assert_eq!(
            toks("Amiya's cat"),
            vec!["amiya".to_owned(), "cat".to_owned()]
        );
        assert_eq!(
            toks("Amiya’s cat"),
            vec!["amiya".to_owned(), "cat".to_owned()]
        );
    }

    #[test]
    fn short_tokens_are_not_mangled_by_the_possessive_rule() {
        // "it's" is 4 chars; stripping would leave "it", which is intended.
        // A bare "'s" must not become empty.
        assert_eq!(toks("W's"), vec!["w's".to_owned()], "too short to strip");
    }

    #[test]
    fn schema_indexes_positions() {
        let s = lore_schema();
        let entry = s.schema.get_field_entry(s.body);
        let opts = entry
            .field_type()
            .get_index_record_option()
            .expect("body is indexed");
        assert_eq!(opts, IndexRecordOption::WithFreqsAndPositions);
    }

    #[test]
    fn rrf_ranks_agreement_above_a_single_first_place() {
        let fused = rrf(&[vec![3, 1, 2], vec![1, 4, 3]], 20.0);
        assert_eq!(fused[0].0, 1, "doc 1 is ranked 2 and 1");
        assert_eq!(fused[1].0, 3, "doc 3 is ranked 1 and 3");
    }

    #[test]
    fn analyze_round_trips_through_a_real_index() {
        let dir = tempdir();
        let index = open_index(&dir).expect("index opens");
        let got = analyze(&index, "Kal’tsit's report").expect("analyzer registered");
        assert_eq!(got, vec!["kal'tsit".to_owned(), "report".to_owned()]);

        // Reopening must not lose the tokenizer: this is the silent-failure
        // path that open_index exists to close.
        let reopened = open_index(&dir).expect("index reopens");
        assert!(reopened.tokenizers().get(LORE_TOKENIZER).is_some());
    }

    fn tempdir() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("trevor-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).expect("temp dir");
        p
    }
}
