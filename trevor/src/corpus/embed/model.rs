//! Loading the embedder, and the query embedder that must match the corpus's model and tokenizer.

use std::path::Path;

use anyhow::{Result, anyhow, bail};
use fastembed::{
    InitOptionsUserDefined, Pooling, QuantizationMode, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel,
};

use super::{MODEL_MAX_TOKENS, read};
use crate::util::sha_hex;

/// A loaded embedder and the hashes that identify it.
pub struct LoadedModel {
    pub embedding: TextEmbedding,
    pub model_sha: String,
    pub tokenizer_sha: String,
}

/// Load `onnx_file` from `model_dir` with the four tokenizer files beside it,
/// CLS pooling, and batching refused (see the module docs).
///
/// # Errors
/// Missing files or an ONNX Runtime load failure.
pub fn load_model(model_dir: &Path, onnx_file: &str, intra_threads: Option<usize>) -> Result<LoadedModel> {
    let tokenizer_file = read(model_dir, "tokenizer.json")?;
    let tokenizer_sha = sha_hex(&tokenizer_file);
    let onnx = read(model_dir, onnx_file)?;
    let model_sha = sha_hex(&onnx);
    let files = TokenizerFiles {
        tokenizer_file,
        config_file: read(model_dir, "config.json")?,
        special_tokens_map_file: read(model_dir, "special_tokens_map.json")?,
        tokenizer_config_file: read(model_dir, "tokenizer_config.json")?,
    };
    let model = UserDefinedEmbeddingModel::new(onnx, files)
        .with_pooling(Pooling::Cls)
        // Declared Dynamic so fastembed itself refuses to batch, which is the
        // property this module depends on.
        .with_quantization(QuantizationMode::Dynamic);
    let mut opts = InitOptionsUserDefined::new().with_max_length(MODEL_MAX_TOKENS as usize);
    if let Some(n) = intra_threads {
        opts = opts.with_intra_threads(n);
    }
    let embedding =
        TextEmbedding::try_new_from_user_defined(model, opts).map_err(|e| anyhow!("loading the embedder: {e}"))?;
    Ok(LoadedModel {
        embedding,
        model_sha,
        tokenizer_sha,
    })
}

/// Embeds queries the way the corpus was embedded: one text per call, CLS,
/// unit length.
///
/// The tokenizer may not differ from the corpus's; that is refused here. The
/// ONNX file may not differ either: [`check_query_model`] refuses a query
/// model other than the vectors' own.
pub struct QueryEmbedder {
    embedding: TextEmbedding,
    pub model_sha: String,
    pub onnx_file: String,
}

impl QueryEmbedder {
    /// # Errors
    /// Load failures, or a tokenizer other than the one the vectors used.
    pub fn load(
        model_dir: &Path,
        onnx_file: &str,
        intra_threads: Option<usize>,
        vectors_tokenizer_sha: &str,
    ) -> Result<Self> {
        let m = load_model(model_dir, onnx_file, intra_threads)?;
        if m.tokenizer_sha != vectors_tokenizer_sha {
            bail!(
                "query tokenizer {:.16} differs from the one the vectors used ({:.16})",
                m.tokenizer_sha,
                vectors_tokenizer_sha
            );
        }
        Ok(Self {
            embedding: m.embedding,
            model_sha: m.model_sha,
            onnx_file: onnx_file.to_owned(),
        })
    }

    /// # Errors
    /// An inference failure or an empty result.
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        self.embedding
            .embed([text], None)
            .map_err(|e| anyhow!("embedding the query: {e}"))?
            .pop()
            .ok_or_else(|| anyhow!("no vector for the query"))
    }
}

/// Setting this to `1` turns [`check_query_model`]'s refusal into a warning
/// (the kill switch: query and corpus models may then differ).
pub const ALLOW_QUERY_MODEL_MISMATCH_ENV: &str = "TREVOR_ALLOW_QUERY_MODEL_MISMATCH";

/// The query ONNX file to load: `requested`, or the vectors' own `modelFile`
/// when `requested` is `auto`.
#[must_use]
pub fn resolve_query_onnx<'a>(requested: &'a str, vectors_model_file: &'a str) -> &'a str {
    if requested == "auto" {
        vectors_model_file
    } else {
        requested
    }
}

/// Refuse a query embedder whose model sha differs from the one the corpus
/// vectors were built with (`vectors.meta.json` `modelFile`, `modelSha`):
/// INT8 vectors move with the CPU, so mixing models or machines silently
/// degrades dense retrieval. `allow_mismatch`, or
/// [`ALLOW_QUERY_MODEL_MISMATCH_ENV`]`=1`, logs the mismatch and allows it.
///
/// # Errors
/// The two shas differ and no override is set.
pub fn check_query_model(
    vectors_model_file: &str,
    vectors_model_sha: &str,
    query: &QueryEmbedder,
    allow_mismatch: bool,
) -> Result<()> {
    if query.model_sha == vectors_model_sha {
        return Ok(());
    }
    let msg = format!(
        "query model {} ({:.16}) is not the model the vectors were built with, {} ({:.16})",
        query.onnx_file, query.model_sha, vectors_model_file, vectors_model_sha
    );
    let env = crate::env::is(ALLOW_QUERY_MODEL_MISMATCH_ENV, "1");
    if allow_mismatch || env {
        eprintln!("warning: {msg}; allowed by override");
        return Ok(());
    }
    bail!("{msg}; pass the vectors' model file (or `auto`), or set {ALLOW_QUERY_MODEL_MISMATCH_ENV}=1")
}
