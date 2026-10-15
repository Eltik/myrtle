//! `unpacker schema-sync --self-check`: the gate that lets the sync judge
//! candidates through the reflection verifier at all.
//!
//! Every gamedata `FlatBuffer` in each bundle dir is verified twice under the
//! schemas the CURRENT generated decoder was built from: once through the
//! generated code (`flatbuffers_decode::verify_table`, both schema sides) and
//! once through the vendored reflection verifier with the `.bfbs` flatc
//! compiles from the same `.fbs`. Both use `verifier_opts()`. Any buffer where
//! the two disagree is a defect in the reflection path or in bfbs generation,
//! and the sync's verdicts cannot be trusted until it is zero.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use anyhow::Context;

use super::reflect::{self, BfbsCache, Compiled};
use crate::export::gamedata::for_each_gamedata_flatbuffer;
use crate::export::manifest::{ResourceManifest, find_idx_file};
use crate::flatbuffers_decode::{self, YostarVerdict};

#[derive(Default)]
struct Row {
    buffers: usize,
    /// (generated passes, reflection passes, comparisons, agreements)
    cn: [usize; 4],
    yostar: [usize; 4],
    example: Option<String>,
}

fn tally(slot: &mut [usize; 4], generated_ok: bool, reflected: &Result<(), String>) -> bool {
    slot[0] += usize::from(generated_ok);
    slot[1] += usize::from(reflected.is_ok());
    slot[2] += 1;
    let agree = generated_ok == reflected.is_ok();
    slot[3] += usize::from(agree);
    agree
}

/// Runs the check over every `(server, dir)`. Returns the total number of
/// disagreeing verdicts.
pub fn run(
    store: &Path,
    bundles: &[(String, PathBuf)],
    cache: &BfbsCache,
) -> anyhow::Result<usize> {
    let mut compiled: HashMap<(bool, &'static str), Compiled> = HashMap::new();
    let mut total_disagree = 0usize;
    for (server, dir) in bundles {
        let idx = find_idx_file(dir)
            .with_context(|| format!("{server}: no .idx in {}", dir.display()))?;
        let manifest = ResourceManifest::load(&idx)?;
        let mut rows: BTreeMap<&'static str, Row> = BTreeMap::new();
        let mut unrecognised = 0usize;
        for_each_gamedata_flatbuffer(dir, &manifest, |buf| {
            let file_name = Path::new(&buf.path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(&buf.name)
                .to_string();
            let verdict = flatbuffers_decode::verify_table(&buf.data, &file_name);
            if verdict.table == "unknown" {
                unrecognised += 1;
                return;
            }
            // `generate-fbs` names every schema type after its store stem.
            let stem = verdict.table;
            let row = rows.entry(verdict.table).or_default();
            row.buffers += 1;
            let mut bfbs = |yostar: bool| -> Compiled {
                compiled
                    .entry((yostar, stem))
                    .or_insert_with(|| {
                        let side = if yostar { "yostar" } else { "cn" };
                        let path = store.join(side).join(format!("{stem}.fbs"));
                        std::fs::read(&path)
                            .map_err(|e| format!("{}: {e}", path.display()))
                            .and_then(|src| cache.compile(stem, &src))
                    })
                    .clone()
            };
            let check = |bfbs: Compiled| match bfbs {
                Ok(b) => reflect::verify(&b, &buf.data),
                Err(e) => Err(e),
            };
            let r_cn = check(bfbs(false));
            if !tally(&mut row.cn, verdict.cn.is_none(), &r_cn) {
                row.example.get_or_insert(format!(
                    "{} cn: generated {:?} reflection {:?}",
                    buf.path, verdict.cn, r_cn
                ));
            }
            let generated_yostar = match &verdict.yostar {
                YostarVerdict::NotRun => None,
                YostarVerdict::Verified => Some(true),
                YostarVerdict::Failed(_) => Some(false),
            };
            if let Some(gen_ok) = generated_yostar {
                let r_y = check(bfbs(true));
                if !tally(&mut row.yostar, gen_ok, &r_y) {
                    row.example.get_or_insert(format!(
                        "{} yostar: generated {gen_ok} reflection {:?}",
                        buf.path, r_y
                    ));
                }
            }
        });

        println!("== {server}");
        println!(
            "{:<26} {:>7} | {:>23} | {:>23}",
            "schema type", "buffers", "CN gen/refl/agree", "Yostar gen/refl/agree"
        );
        let (mut buffers, mut comparisons, mut agreements) = (0, 0, 0);
        for (table, r) in &rows {
            let cell = |s: &[usize; 4]| {
                if s[2] == 0 {
                    "n/a".to_string()
                } else {
                    format!("{}/{}/{}", s[0], s[1], s[3])
                }
            };
            println!(
                "{table:<26} {:>7} | {:>23} | {:>23}{}",
                r.buffers,
                cell(&r.cn),
                cell(&r.yostar),
                r.example
                    .as_ref()
                    .map_or_else(String::new, |e| format!("  DISAGREE {e}"))
            );
            buffers += r.buffers;
            comparisons += r.cn[2] + r.yostar[2];
            agreements += r.cn[3] + r.yostar[3];
        }
        let disagree = comparisons - agreements;
        total_disagree += disagree;
        println!(
            "{server}: {} schema types, {buffers} buffers, {comparisons} verdicts compared, {agreements} agree, {disagree} disagree, {unrecognised} unrecognised",
            rows.len()
        );
    }
    Ok(total_disagree)
}
