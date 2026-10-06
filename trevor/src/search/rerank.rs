//! Cross-encoder reranking with `cross-encoder/ettin-reranker-17m-v1`.
//!
//! The published ONNX export is the ModernBERT backbone only: its single
//! output is `last_hidden_state [batch, seq, 256]` (checked 2026-09-24, opset
//! 18). The sentence-transformers scoring head ships as three safetensors
//! files and is applied here:
//!
//! ```text
//! CLS token -> Dense(256->256, no bias) -> GELU (exact, erf)
//!           -> LayerNorm(256, eps 1e-5) -> Dense(256->1, bias) -> score
//! ```
//!
//! 66,305 parameters. The score is one unbounded logit, higher is better,
//! never a softmax over classes. Pairs are encoded by the tokenizer's own
//! post-processor as `[CLS] query [SEP] passage [SEP]`, which is exactly what
//! `transformers` produces for this model (0 mismatches on 33 pairs).
//!
//! The fp32 export is used, one pair per run. It is 67 MB, so there is no
//! INT8 accuracy question to answer per host (the gte INT8 export lost 6
//! points of cosine on x86), and one pair per run keeps scores independent
//! of which passages share a batch.

use std::path::Path;
use std::sync::Mutex;

use anyhow::{Context, Result, anyhow, bail};
use ort::session::Session;
use ort::value::Tensor;
use safetensors::SafeTensors;
use sha2::{Digest, Sha256};
use tokenizers::{Tokenizer, TruncationDirection, TruncationParams, TruncationStrategy};

/// The model's trained context (`model_max_length` 7999).
pub const RERANK_MAX_TOKENS: usize = 7999;
/// Longest query accepted. Truncation cuts only the passage, so an oversized
/// query would make every pair fail; refuse it up front instead.
pub const MAX_QUERY_TOKENS: usize = 256;

struct Head {
    hidden: usize,
    w1: Vec<f32>, // [hidden, hidden], row = output
    ln_w: Vec<f32>,
    ln_b: Vec<f32>,
    w2: Vec<f32>, // [hidden]
    b2: f32,
}

fn tensor(st: &SafeTensors<'_>, name: &str, shape: &[usize]) -> Result<Vec<f32>> {
    let t = st
        .tensor(name)
        .with_context(|| format!("safetensors has no {name}"))?;
    if t.dtype() != safetensors::Dtype::F32 {
        bail!("{name} is {:?}, expected F32", t.dtype());
    }
    if t.shape() != shape {
        bail!("{name} has shape {:?}, expected {shape:?}", t.shape());
    }
    Ok(t.data()
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}

impl Head {
    fn load(dir: &Path, hidden: usize) -> Result<Self> {
        let read = |p: &str| {
            std::fs::read(dir.join(p)).with_context(|| format!("reading {}", dir.join(p).display()))
        };
        let (d1, d2, d3) = (
            read("2_Dense/model.safetensors")?,
            read("3_LayerNorm/model.safetensors")?,
            read("4_Dense/model.safetensors")?,
        );
        let st1 = SafeTensors::deserialize(&d1)?;
        let st2 = SafeTensors::deserialize(&d2)?;
        let st3 = SafeTensors::deserialize(&d3)?;
        let b2 = tensor(&st3, "linear.bias", &[1])?;
        Ok(Self {
            hidden,
            w1: tensor(&st1, "linear.weight", &[hidden, hidden])?,
            ln_w: tensor(&st2, "norm.weight", &[hidden])?,
            ln_b: tensor(&st2, "norm.bias", &[hidden])?,
            w2: tensor(&st3, "linear.weight", &[1, hidden])?,
            b2: b2[0],
        })
    }

    /// Accumulates in f64: 256-term dot products in f32 drift by ~1e-6,
    /// enough to reorder near-ties between otherwise identical runs on two
    /// machines.
    fn score(&self, cls: &[f32]) -> f32 {
        let h = self.hidden;
        let mut a: Vec<f64> = (0..h)
            .map(|o| {
                let row = &self.w1[o * h..(o + 1) * h];
                let z: f64 = row
                    .iter()
                    .zip(cls)
                    .map(|(w, x)| f64::from(*w) * f64::from(*x))
                    .sum();
                gelu(z)
            })
            .collect();
        #[allow(clippy::cast_precision_loss)]
        let n = h as f64;
        let mean = a.iter().sum::<f64>() / n;
        let var = a.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n;
        let inv = 1.0 / (var + 1e-5).sqrt();
        for (i, x) in a.iter_mut().enumerate() {
            *x = (*x - mean) * inv * f64::from(self.ln_w[i]) + f64::from(self.ln_b[i]);
        }
        let s: f64 = a.iter().zip(&self.w2).map(|(x, w)| x * f64::from(*w)).sum();
        #[allow(clippy::cast_possible_truncation)]
        let out = (s + f64::from(self.b2)) as f32;
        out
    }
}

/// `torch.nn.GELU()` default: `0.5 x (1 + erf(x / sqrt 2))`.
fn gelu(x: f64) -> f64 {
    0.5 * x * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Abramowitz and Stegun 7.1.26, maximum absolute error 1.5e-7, which is
/// below f32 resolution for the activations this head sees. `f64::erf` is not
/// stable in std, and one function does not justify a dependency.
fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let y = 1.0
        - (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736) * t
            + 0.254_829_592)
            * t
            * (-x * x).exp();
    sign * y
}

pub struct Reranker {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
    head: Head,
    pub model_sha: String,
    pub max_tokens: usize,
}

impl Reranker {
    /// Load the backbone `onnx_file` (relative to `dir`), `tokenizer.json`,
    /// and the three head files. `max_tokens` caps the whole pair; the
    /// passage is cut, never the query.
    ///
    /// # Errors
    /// Missing files, head tensors of the wrong shape, or a backbone whose
    /// output is not `last_hidden_state`.
    pub fn load(
        dir: &Path,
        onnx_file: &str,
        max_tokens: usize,
        intra_threads: Option<usize>,
    ) -> Result<Self> {
        if !(MAX_QUERY_TOKENS + 8..=RERANK_MAX_TOKENS).contains(&max_tokens) {
            bail!(
                "rerank max tokens must be in {}..={RERANK_MAX_TOKENS}",
                MAX_QUERY_TOKENS + 8
            );
        }
        let onnx = std::fs::read(dir.join(onnx_file))
            .with_context(|| format!("reading {}", dir.join(onnx_file).display()))?;
        let model_sha = format!("{:x}", Sha256::digest(&onnx));
        let mut builder = Session::builder().map_err(|e| anyhow!("{e}"))?;
        if let Some(n) = intra_threads {
            builder = builder.with_intra_threads(n).map_err(|e| anyhow!("{e}"))?;
        }
        let session = builder
            .commit_from_memory(&onnx)
            .map_err(|e| anyhow!("loading the reranker: {e}"))?;
        let inputs: Vec<&str> = session
            .inputs()
            .iter()
            .map(ort::value::Outlet::name)
            .collect();
        if inputs != ["input_ids", "attention_mask"] {
            bail!("reranker inputs are {inputs:?}, expected [input_ids, attention_mask]");
        }
        let outputs: Vec<&str> = session
            .outputs()
            .iter()
            .map(ort::value::Outlet::name)
            .collect();
        if outputs != ["last_hidden_state"] {
            bail!(
                "reranker outputs are {outputs:?}; this loader applies the head itself and \
                 expects the backbone's last_hidden_state only"
            );
        }

        let mut tokenizer =
            Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| anyhow!("{e}"))?;
        tokenizer.with_padding(None);
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length: max_tokens,
                strategy: TruncationStrategy::OnlySecond,
                stride: 0,
                direction: TruncationDirection::Right,
            }))
            .map_err(|e| anyhow!("{e}"))?;

        let config: serde_json::Value = serde_json::from_slice(
            &std::fs::read(dir.join("config.json")).context("reading config.json")?,
        )?;
        let hidden = config["hidden_size"]
            .as_u64()
            .context("config.json has no hidden_size")?;
        let head = Head::load(dir, usize::try_from(hidden)?)?;
        Ok(Self {
            session: Mutex::new(session),
            tokenizer,
            head,
            model_sha,
            max_tokens,
        })
    }

    /// One relevance logit per passage, in input order.
    ///
    /// # Errors
    /// An over-long query, a tokenizer failure, or an inference failure.
    pub fn score(&self, query: &str, passages: &[&str]) -> Result<Vec<f32>> {
        let q_len = self
            .tokenizer
            .encode(query, false)
            .map_err(|e| anyhow!("{e}"))?
            .len();
        if q_len > MAX_QUERY_TOKENS {
            bail!("query is {q_len} tokens; the reranker accepts at most {MAX_QUERY_TOKENS}");
        }
        let mut session = self
            .session
            .lock()
            .map_err(|_| anyhow!("reranker session poisoned"))?;
        let mut out = Vec::with_capacity(passages.len());
        for p in passages {
            let enc = self
                .tokenizer
                .encode((query, *p), true)
                .map_err(|e| anyhow!("{e}"))?;
            let ids: Vec<i64> = enc.get_ids().iter().map(|&i| i64::from(i)).collect();
            let n = ids.len();
            let mask = vec![1i64; n];
            let outputs = session
                .run(ort::inputs![
                    "input_ids" => Tensor::from_array(([1usize, n], ids)).map_err(|e| anyhow!("{e}"))?,
                    "attention_mask" => Tensor::from_array(([1usize, n], mask)).map_err(|e| anyhow!("{e}"))?,
                ])
                .map_err(|e| anyhow!("reranker inference: {e}"))?;
            let (shape, data) = outputs["last_hidden_state"]
                .try_extract_tensor::<f32>()
                .map_err(|e| anyhow!("{e}"))?;
            let h = self.head.hidden;
            if shape.len() != 3 || usize::try_from(shape[2])? != h || data.len() < h {
                bail!("unexpected last_hidden_state shape {:?}", &**shape);
            }
            out.push(self.head.score(&data[..h]));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erf_matches_known_values() {
        // Reference values from the closed form tables.
        for (x, want) in [
            (0.0, 0.0),
            (0.5, 0.520_499_877_8),
            (1.0, 0.842_700_792_9),
            (2.0, 0.995_322_265),
            (-1.0, -0.842_700_792_9),
        ] {
            assert!(
                (erf(x) - want).abs() < 2e-7,
                "erf({x}) = {} want {want}",
                erf(x)
            );
        }
    }

    #[test]
    fn gelu_is_the_exact_form_not_tanh() {
        // exact GELU(1) = 0.841344746; the tanh approximation gives 0.841191990.
        assert!((gelu(1.0) - 0.841_344_746).abs() < 2e-7);
    }
}
