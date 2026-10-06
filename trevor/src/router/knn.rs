//! The kNN router's loader: labelled examples embedded with the query embedder, vectors cached per model.

use std::collections::HashMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::corpus::embed::QueryEmbedder;
use crate::tools::Route;
use crate::tools::Tools;
use super::{Dict, Intent, KnnParams, decide, neighbours, vote_from};

/// The kNN router: labelled examples embedded with the query embedder (vectors cached per model under
/// `artifacts/router/`, since embedding 1,200 questions takes about half a minute).
pub struct Knn {
    embedder: QueryEmbedder,
    pub examples: Vec<(Vec<f32>, String)>,
    /// The real question each example is or paraphrases, so a held-out question also drops its paraphrases.
    pub origins: Vec<String>,
    /// Examples before this index are the real labelled questions, in file order; the rest are paraphrases.
    pub n_real: usize,
    pub params: KnnParams,
    pub dict: Dict,
}

impl Knn {
    /// # Errors
    /// Model load failures, or an unreadable intents file.
    pub fn load(model_dir: &Path, onnx: &str, intents: &Path, synth: Option<&Path>, params: KnnParams, tools: &Tools) -> Result<Self> {
        let tok = std::fs::read(model_dir.join("tokenizer.json")).with_context(|| format!("reading {}", model_dir.display()))?;
        let tok_sha = crate::util::sha_hex(&tok);
        let mut embedder = QueryEmbedder::load(model_dir, onnx, None, &tok_sha)?;
        let read = crate::util::read_jsonl_strict::<Intent>;
        let mut items = read(intents)?;
        let n_real = items.len();
        if let Some(p) = synth {
            // Only paraphrases the model router sent to their source's tool and target.
            items.extend(read(p)?.into_iter().filter(|i| i.verified == Some(true)));
        }
        let cache_path = PathBuf::from("artifacts/router").join(format!("{:.16}.jsonl", embedder.model_sha));
        let mut cache: HashMap<String, Vec<f32>> = std::fs::read_to_string(&cache_path).unwrap_or_default().lines()
            .filter_map(|l| serde_json::from_str::<(String, Vec<f32>)>(l).ok()).collect();
        let mut fresh = Vec::new();
        let mut examples = Vec::with_capacity(items.len());
        for it in &items {
            let v = if let Some(v) = cache.get(&it.q) { v.clone() } else {
                let v = embedder.embed(&it.q)?;
                cache.insert(it.q.clone(), v.clone());
                fresh.push((it.q.clone(), v.clone()));
                v
            };
            examples.push((v, it.tool.clone()));
        }
        if !fresh.is_empty() {
            std::fs::create_dir_all("artifacts/router")?;
            let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&cache_path)?;
            for x in &fresh {
                serde_json::to_writer(&mut f, x)?;
                f.write_all(b"\n")?;
            }
        }
        let origins = items.into_iter().map(|i| i.source_q.unwrap_or(i.q)).collect();
        Ok(Self { embedder, examples, origins, n_real, params, dict: Dict::new(tools) })
    }

    /// # Errors
    /// An embedding failure.
    pub fn embed(&mut self, q: &str) -> Result<Vec<f32>> {
        self.embedder.embed(q)
    }

    /// Route one question. `holdout` drops examples with exactly this text and their paraphrases, so a question
    /// that is itself a labelled example is routed by its neighbours, not by its own label (the route check).
    ///
    /// # Errors
    /// An embedding failure.
    pub fn route(&mut self, q: &str, holdout: bool) -> Result<(Route, super::Vote)> {
        let v = self.embed(q)?;
        let origins = &self.origins;
        let neigh = neighbours(&self.examples, &v, self.params.k, |i| holdout && origins[i] == q);
        let vt = vote_from(&self.examples, &neigh, self.params.k);
        Ok((decide(&vt, self.params, q, &self.dict), vt))
    }
}
