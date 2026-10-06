//! BM25 over chunk text with the lore analyzer.
//!
//! Queries are analyzed with the index's own registered analyzer and turned
//! into a flat OR of `TermQuery`s. The query parser is bypassed on purpose: it
//! rejects a bare apostrophe (`kal'tsit` is a syntax error), and that is the
//! token class this corpus depends on. Repeated query terms are counted once,
//! so "Amiya and Amiya's sister" does not weight `amiya` twice.

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, Query, TermQuery};
use tantivy::schema::{IndexRecordOption, Value as _};
use tantivy::{IndexReader, TantivyDocument, Term, doc};

use super::store::ChunkStore;
use super::tokenizer::{LORE_PATTERN, analyze, lore_schema, open_index};

const META: &str = "trevor-bm25.json";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Bm25Meta {
    chunks_sha: String,
    analyzer_pattern: String,
    count: usize,
}

/// Build a fresh index at `dir` from every chunk, in row order.
///
/// One indexing thread, so documents land in row order and a rebuild from the
/// same chunks gives the same index. The build takes seconds; parallelism buys
/// nothing worth the nondeterminism.
///
/// # Errors
/// I/O and tantivy errors.
pub fn build(store: &ChunkStore, dir: &Path) -> Result<()> {
    if dir.exists() {
        std::fs::remove_dir_all(dir).with_context(|| format!("clearing {}", dir.display()))?;
    }
    let index = open_index(dir)?;
    let s = lore_schema();
    let mut writer = index.writer_with_num_threads::<TantivyDocument>(1, 50_000_000)?;
    for c in &store.chunks {
        writer.add_document(doc!(s.chunk_id => c.chunk_id.as_str(), s.body => c.indexed_text().as_ref()))?;
    }
    writer.commit()?;
    writer.wait_merging_threads()?;
    let meta = Bm25Meta {
        chunks_sha: store.chunks_sha.clone(),
        analyzer_pattern: LORE_PATTERN.to_owned(),
        count: store.len(),
    };
    std::fs::write(dir.join(META), serde_json::to_vec_pretty(&meta)?)?;
    Ok(())
}

pub struct Bm25 {
    index: tantivy::Index,
    reader: IndexReader,
    chunk_id: tantivy::schema::Field,
    body: tantivy::schema::Field,
}

impl Bm25 {
    /// Open an index built by [`build`], refusing one built from different
    /// chunks or with a different analyzer.
    ///
    /// # Errors
    /// A missing index, a stale index, or tantivy errors.
    pub fn open(dir: &Path, store: &ChunkStore) -> Result<Self> {
        let meta: Bm25Meta = serde_json::from_slice(
            &std::fs::read(dir.join(META))
                .with_context(|| format!("{} has no {META}; run build-index", dir.display()))?,
        )?;
        if meta.chunks_sha != store.chunks_sha {
            bail!(
                "BM25 index was built from chunks {:.16}, but chunks.jsonl is {:.16}; run build-index",
                meta.chunks_sha,
                store.chunks_sha
            );
        }
        if meta.analyzer_pattern != LORE_PATTERN {
            bail!("BM25 index was built with a different analyzer; run build-index");
        }
        let index = open_index(dir)?;
        let reader = index.reader()?;
        let s = lore_schema();
        Ok(Self {
            index,
            reader,
            chunk_id: s.chunk_id,
            body: s.body,
        })
    }

    /// Top `k` rows for `query`, best first, ties broken by row.
    ///
    /// # Errors
    /// tantivy errors, or a stored id missing from the store.
    pub fn search(&self, query: &str, k: usize, store: &ChunkStore) -> Result<Vec<(usize, f32)>> {
        let mut seen = HashSet::new();
        let terms: Vec<String> = analyze(&self.index, query)?
            .into_iter()
            .filter(|t| seen.insert(t.clone()))
            .collect();
        if terms.is_empty() || k == 0 {
            return Ok(Vec::new());
        }
        let clauses: Vec<(Occur, Box<dyn Query>)> = terms
            .iter()
            .map(|t| {
                let q: Box<dyn Query> = Box::new(TermQuery::new(
                    Term::from_field_text(self.body, t),
                    IndexRecordOption::WithFreqs,
                ));
                (Occur::Should, q)
            })
            .collect();
        let searcher = self.reader.searcher();
        let top = searcher.search(
            &BooleanQuery::new(clauses),
            &TopDocs::with_limit(k).order_by_score(),
        )?;
        let mut out = Vec::with_capacity(top.len());
        for (score, addr) in top {
            let d: TantivyDocument = searcher.doc(addr)?;
            let id = d
                .get_first(self.chunk_id)
                .and_then(|v| v.as_str())
                .context("document without chunk_id")?;
            let row = store
                .row(id)
                .with_context(|| format!("BM25 returned {id}, which is not in the store"))?;
            out.push((row, score));
        }
        out.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::chunk::Chunk;

    fn chunk(id: &str, text: &str) -> Chunk {
        Chunk {
            chunk_id: id.to_owned(),
            story_id: id.split('#').next().unwrap_or(id).to_owned(),
            group_id: "g".to_owned(),
            ordinal: 0,
            scene_ordinal: 0,
            line_start: 1,
            line_end: 2,
            text: text.to_owned(),
            speakers: vec![],
            on_screen: vec![],
            background: None,
            token_count: 10,
            content_sha: String::new(),
            prefix: None,
        }
    }

    fn tmp(name: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("trevor-bm25-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    fn store(sha: &str) -> ChunkStore {
        ChunkStore::from_chunks(
            vec![
                chunk("a#0000", "Kal’tsit: Amiya, stay behind me."),
                chunk(
                    "b#0000",
                    "Ch'en: Move out. The Lungmen Guard Department holds the line.",
                ),
                chunk("c#0000", "W: Heh. Who's fooling around with my grenades?"),
            ],
            sha.to_owned(),
        )
        .expect("store")
    }

    #[test]
    fn apostrophe_names_match_across_spellings_and_possessives() {
        let dir = tmp("names");
        let s = store("x");
        build(&s, &dir).expect("build");
        let bm = Bm25::open(&dir, &s).expect("open");
        let hits = bm
            .search("What did Kal'tsit's say?", 3, &s)
            .expect("search");
        assert_eq!(
            hits.first().map(|h| h.0),
            Some(0),
            "curly corpus, straight query"
        );
        let hits = bm.search("ch'en lungmen", 3, &s).expect("search");
        assert_eq!(hits.first().map(|h| h.0), Some(1));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_stale_index_is_refused() {
        let dir = tmp("stale");
        build(&store("old"), &dir).expect("build");
        let err = Bm25::open(&dir, &store("new")).err().expect("must refuse");
        assert!(err.to_string().contains("run build-index"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_all_punctuation_query_returns_nothing() {
        let dir = tmp("empty");
        let s = store("x");
        build(&s, &dir).expect("build");
        let bm = Bm25::open(&dir, &s).expect("open");
        assert!(bm.search("?!...", 5, &s).expect("search").is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
