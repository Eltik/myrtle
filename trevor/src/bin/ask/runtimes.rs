//! The loaded corpora: the main corpus, P4 loaded on first need, and the art corpus the typed fallback reads.

use std::path::PathBuf;

use anyhow::Result;
use trevor::search::runtime::{Runtime, RuntimeArgs};


/// The loaded corpus, and P4 loaded on first need when `--corpus` was left at its default (a batch can mix both).
pub(crate) struct Runtimes {
    pub(crate) main: Runtime,
    pub(crate) p4: Option<Runtime>,
    /// P4 plus the operator art units, loaded on the first typed-fallback ask for an operator's art.
    pub(crate) art: Option<Runtime>,
    /// The P4 directory of the lore served (`--lore`).
    pub(crate) p4_dir: &'static str,
    pub(crate) switch: bool,
    pub(crate) args: RuntimeArgs,
    /// The spoiler horizon of the current `--serve` job, given to every corpus `pick` returns. None outside serve.
    pub(crate) horizon: Option<std::sync::Arc<trevor::search::horizon::Horizon>>,
    /// Load the dense index and the query embedder (`--retrieval hybrid`, the default); false is BM25-only serving.
    pub(crate) dense: bool,
}

/// A corpus as `ask` loads it: BM25 always, the dense index and query embedder unless BM25-only (`dense` false); the
/// reranker loads on first use (`scenes::ensure_reranker`); the query embedder of `share` is reused when it fits
/// (`Runtime::load_sharing`).
/// `TREVOR_LOAD_DEBUG=1` prints the load time.
pub(crate) fn load_runtime(a: &RuntimeArgs, dense: bool, share: Option<trevor::search::runtime::SharedEmbedder>)
    -> Result<Runtime> {
    let t = std::time::Instant::now();
    let mut rt = Runtime::load_sharing(a, dense, true, false, share)?;
    rt.bm25_only = !dense;
    if trevor::env::debug_on("TREVOR_LOAD_DEBUG") {
        eprintln!("loaded {} in {:.2} s (dense {dense})", a.corpus.display(), t.elapsed().as_secs_f64());
    }
    Ok(rt)
}

impl Runtimes {
    pub(crate) fn pick(&mut self, p4: Option<bool>) -> Result<&mut Runtime> {
        if self.switch && p4 == Some(true) && self.args.corpus != std::path::Path::new(self.p4_dir) {
            if self.p4.is_none() {
                let mut a = self.args.clone();
                a.corpus = PathBuf::from(self.p4_dir);
                self.p4 = Some(load_runtime(&a, self.dense, self.main.shared_embedder())?);
            }
            let rt = self.p4.as_mut().expect("loaded above");
            rt.horizon.clone_from(&self.horizon);
            return Ok(rt);
        }
        self.main.horizon.clone_from(&self.horizon);
        Ok(&mut self.main)
    }

    /// Whether a spoiler horizon is set (`--serve` jobs only): table answers, generated passages and the chronology
    /// notes are then left out (they are built from every story, later ones included).
    pub(crate) fn horizon_set(&self) -> bool {
        self.horizon.is_some()
    }

    /// Set the horizon on every loaded corpus (and on the ones `pick` loads later).
    pub(crate) fn set_horizon(&mut self, h: Option<std::sync::Arc<trevor::search::horizon::Horizon>>) {
        for rt in std::iter::once(&mut self.main).chain(self.p4.as_mut()).chain(self.art.as_mut()) {
            rt.horizon.clone_from(&h);
        }
        self.horizon = h;
    }

    /// The corpus for the "art" source: `ART_DIR` (P4 plus the art units), P4 when it has not been built.
    pub(crate) fn pick_art(&mut self) -> Result<&mut Runtime> {
        if !self.switch || !std::path::Path::new(ART_DIR).join("chunks.jsonl").exists() {
            return self.pick(Some(true));
        }
        if self.art.is_none() {
            let mut a = self.args.clone();
            a.corpus = PathBuf::from(ART_DIR);
            self.art = Some(load_runtime(&a, self.dense, self.main.shared_embedder())?);
        }
        let rt = self.art.as_mut().expect("loaded above");
        rt.horizon.clone_from(&self.horizon);
        Ok(rt)
    }
}

/// P4 plus one unit per operator art image (`scripts/art_captions.py units`, then `build-units --units
/// artifacts/art/p4_art_units.jsonl --out artifacts/p4-art`). Read only by the typed fallback's "art" kind, when built,
/// so P4 and every other answer stay as they were.
pub(crate) const ART_DIR: &str = "artifacts/p4-art";
