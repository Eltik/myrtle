//! `unpacker schema-sync`: resolve, from the live bundles, the schemas
//! `unpacker schema-gen` generated from the CN client and the vendored history
//! (`assets/schemas/history`), which schema every gamedata table needs, and
//! write the answer into the store (`assets/schemas`). No git, no network.
//!
//! The contract (candidate order, scoring, lock format, exit codes) is
//! `assets/schemas/README.md`. In short: CN takes the generated schema when it
//! verifies on the CN bundle, else the newest history version that does. The
//! Yostar side keeps nothing when the CN choice verifies on every supplied
//! Yostar server; otherwise it tries the CN choice, the generated schema, every
//! history version newest first, and the stored Yostar file LAST, and takes the
//! first that verifies everywhere, else the one verifying on the most servers
//! (EN breaks ties). Newest first because a newer schema that verifies carries
//! fields an older one lacks: appended fields are invisible to the verifier, so
//! a stability-first order silently drops data. "Verifies" is the vendored
//! reflection verifier under `verifier_opts()`, the options the generated
//! decoder uses.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context, bail};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::fields;
use super::reflect::{self, BfbsCache, Compiled, sha256_hex};
use super::store::{self, history, normalize_fbs};
use crate::export::gamedata::{for_each_gamedata_flatbuffer, rfc3339_utc};
use crate::export::manifest::{ResourceManifest, find_idx_file};

/// Exit code for a table that resolved in the previous lock and now resolves
/// under no candidate on a server where it did.
pub const EXIT_REGRESSION: i32 = 2;

pub struct Options {
    /// `schema-gen` output: `<stem>.fbs`, `client.json`, `schema-gen.json`.
    /// `None` runs on the stored `cn/` (a lock-only or Yostar-only run).
    pub generated: Option<PathBuf>,
    pub store: PathBuf,
    pub bundles: Vec<(String, PathBuf)>,
    pub report: Option<PathBuf>,
    pub levels_sample: usize,
    pub cache: Option<PathBuf>,
    pub compare_dir: Option<PathBuf>,
}

// ---------------------------------------------------------------- lock file

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Lock {
    /// The CN client the generated schemas came from; `null` for a run
    /// without `--generated`.
    #[serde(default)]
    pub client: Option<LockClient>,
    /// Fields and enums of an existing table whose type differs between the
    /// stored `cn/` schema and the generated one (`fields::type_changes`).
    /// The verifier cannot see these, so they are left for a human. Kept from
    /// the previous lock on a run without `--generated`.
    #[serde(default)]
    pub type_changes: Vec<String>,
    pub flatc: String,
    pub servers: BTreeMap<String, LockServer>,
    pub tables: BTreeMap<String, LockTable>,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct LockClient {
    pub version: String,
    pub generated_at: String,
    pub game_assembly_sha256: String,
    pub metadata_sha256: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct LockServer {
    #[serde(rename = "resVersion")]
    pub res_version: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LockTable {
    pub cn: LockChoice,
    #[serde(default)]
    pub yostar: Option<LockChoice>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LockChoice {
    /// `client@<version>`, `history/<stem>/<label>.fbs`, `same-as-cn`,
    /// `store` (a stored file no history version matches) or `unresolved`.
    pub source: String,
    /// Of the file as stored; absent for `same-as-cn`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// Per supplied server where the table is present. Empty when no candidate
    /// verified anywhere (the previous file, if any, is kept).
    pub verified: BTreeMap<String, bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Lock {
    fn load(path: &Path) -> anyhow::Result<Option<Self>> {
        match std::fs::read_to_string(path) {
            Ok(text) => Ok(Some(
                serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?,
            )),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("read {}", path.display())),
        }
    }
}

// ---------------------------------------------------------------- generated

/// One `schema-gen` output directory.
pub struct Generated {
    pub version: String,
    pub client: LockClient,
    pub schemas: BTreeMap<String, Vec<u8>>,
    /// `schema-gen.json` `missing_roots`: stems the client no longer has a
    /// root type for.
    pub missing_roots: Vec<String>,
}

fn read_json(path: &Path) -> anyhow::Result<serde_json::Value> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

fn json_str(v: &serde_json::Value, pointer: &str) -> Option<String> {
    v.pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

/// Reads a `schema-gen` directory. `generated_at` is kept from the previous
/// lock while the client (version and both file hashes) is unchanged, so a
/// rerun on a re-downloaded copy of the same client leaves the lock
/// byte-identical.
fn load_generated(dir: &Path, prev: Option<&Lock>) -> anyhow::Result<Generated> {
    let client = read_json(&dir.join("client.json"))?;
    let gen_meta = read_json(&dir.join("schema-gen.json"))?;
    let version =
        json_str(&client, "/client_version").context("client.json has no client_version")?;
    if let Some(v) = json_str(&gen_meta, "/client_version")
        && v != version
    {
        bail!("client.json says {version}, schema-gen.json says {v}");
    }
    let sha = |file: &str| {
        client
            .get("files")
            .and_then(|f| f.get(file))
            .and_then(|f| f.get("sha256"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .with_context(|| format!("client.json has no files.\"{file}\".sha256"))
    };
    let mut lock_client = LockClient {
        version: version.clone(),
        generated_at: String::new(),
        game_assembly_sha256: sha("GameAssembly.dll")?,
        metadata_sha256: sha("global-metadata.dat")?,
    };
    lock_client.generated_at = prev
        .and_then(|p| p.client.as_ref())
        .filter(|p| {
            p.version == lock_client.version
                && p.game_assembly_sha256 == lock_client.game_assembly_sha256
                && p.metadata_sha256 == lock_client.metadata_sha256
        })
        .map(|p| p.generated_at.clone())
        .or_else(|| json_str(&gen_meta, "/generated_at"))
        .or_else(|| json_str(&client, "/downloaded_at"))
        .unwrap_or_else(|| rfc3339_utc(std::time::SystemTime::now()));

    let mut schemas = BTreeMap::new();
    for stem in store::list_stems(dir) {
        let path = dir.join(format!("{stem}.fbs"));
        schemas.insert(
            stem,
            std::fs::read(&path).with_context(|| format!("read {}", path.display()))?,
        );
    }
    if schemas.is_empty() {
        bail!("{}: no .fbs files", dir.display());
    }
    let missing_roots = gen_meta
        .get("missing_roots")
        .and_then(serde_json::Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Ok(Generated {
        version,
        client: lock_client,
        schemas,
        missing_roots,
    })
}

// ---------------------------------------------------------------- bundles

struct Buf {
    path: String,
    data: Vec<u8>,
}

struct Server {
    name: String,
    res_version: String,
    /// Table -> the buffers that must verify (sampled for directory tables).
    tables: BTreeMap<String, Vec<Buf>>,
    /// Table -> how many buffers the bundles hold in total.
    totals: BTreeMap<String, usize>,
    /// Gamedata `FlatBuffer` paths no stored schema claims.
    unrouted: Vec<String>,
}

fn load_server(
    name: &str,
    dir: &Path,
    tables: &BTreeSet<String>,
    sample: usize,
) -> anyhow::Result<Server> {
    let idx =
        find_idx_file(dir).with_context(|| format!("{name}: no .idx in {}", dir.display()))?;
    let manifest =
        ResourceManifest::load(&idx).with_context(|| format!("load {}", idx.display()))?;
    let res_version = std::fs::read_to_string(dir.join(".downloaded_version"))
        .map_or_else(|_| "unknown".to_string(), |s| s.trim().to_string());

    let mut whole: BTreeMap<String, Vec<Buf>> = BTreeMap::new();
    let mut sampled: BTreeMap<String, BTreeMap<String, Vec<u8>>> = BTreeMap::new();
    let mut totals: BTreeMap<String, usize> = BTreeMap::new();
    let mut unrouted = Vec::new();
    for_each_gamedata_flatbuffer(dir, &manifest, |buf| {
        let Some(table) = store::table_for_path(&buf.path, tables) else {
            unrouted.push(buf.path);
            return;
        };
        *totals.entry(table.clone()).or_default() += 1;
        if store::is_directory_table(&table) {
            // The first `sample` buffers in sorted path order: two runs on the
            // same bundles verify the same files.
            let keep = sampled.entry(table).or_default();
            keep.insert(buf.path, buf.data);
            if keep.len() > sample {
                keep.pop_last();
            }
        } else {
            whole.entry(table).or_default().push(Buf {
                path: buf.path,
                data: buf.data,
            });
        }
    });
    for (table, bufs) in sampled {
        whole.insert(
            table,
            bufs.into_iter()
                .map(|(path, data)| Buf { path, data })
                .collect(),
        );
    }
    for bufs in whole.values_mut() {
        bufs.sort_by(|a, b| a.path.cmp(&b.path));
    }
    unrouted.sort();
    Ok(Server {
        name: name.to_string(),
        res_version,
        tables: whole,
        totals,
        unrouted,
    })
}

// ---------------------------------------------------------------- candidates

#[derive(Clone)]
struct Cand {
    source: String,
    bytes: Arc<Vec<u8>>,
    key: String,
    bfbs: Compiled,
}

struct Ctx<'a> {
    cache: &'a BfbsCache,
    memo: Mutex<HashMap<(String, String, String), bool>>,
}

impl Ctx<'_> {
    fn cand(&self, table: &str, source: String, bytes: Vec<u8>) -> Cand {
        let key = self.cache.key(&bytes);
        let bfbs = self.cache.compile(table, &bytes);
        Cand {
            source,
            bytes: Arc::new(bytes),
            key,
            bfbs,
        }
    }

    /// `None` when the table is absent from the server's bundles.
    fn verifies(&self, cand: &Cand, server: &Server, table: &str) -> Option<bool> {
        let bufs = server.tables.get(table)?;
        let memo_key = (cand.key.clone(), server.name.clone(), table.to_string());
        if let Some(&hit) = self.memo.lock().expect("memo poisoned").get(&memo_key) {
            return Some(hit);
        }
        let ok = match &cand.bfbs {
            Ok(bfbs) => bufs.iter().all(|b| reflect::verify(bfbs, &b.data).is_ok()),
            Err(_) => false,
        };
        self.memo
            .lock()
            .expect("memo poisoned")
            .insert(memo_key, ok);
        Some(ok)
    }

    /// Per server in `servers` where the table is present.
    fn verdicts(&self, cand: &Cand, servers: &[&Server], table: &str) -> BTreeMap<String, bool> {
        servers
            .iter()
            .filter_map(|s| self.verifies(cand, s, table).map(|ok| (s.name.clone(), ok)))
            .collect()
    }
}

/// A candidate before compilation: where it came from and its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Spec {
    source: String,
    bytes: Vec<u8>,
}

impl Spec {
    fn of(c: &Cand) -> Self {
        Self {
            source: c.source.clone(),
            bytes: c.bytes.to_vec(),
        }
    }
}

/// Drops every spec whose normalised text an earlier one already had, so a
/// schema is compiled and verified once, under its first (best) label.
fn dedupe_specs(specs: impl IntoIterator<Item = Spec>) -> Vec<Spec> {
    let mut seen = HashSet::new();
    specs
        .into_iter()
        .filter(|s| seen.insert(normalize_fbs(&String::from_utf8_lossy(&s.bytes))))
        .collect()
}

/// CN order: the head (the generated schema, else the stored `cn/` file), then
/// history newest first.
fn cn_order(head: Option<Spec>, history: &[Spec]) -> Vec<Spec> {
    dedupe_specs(head.into_iter().chain(history.iter().cloned()))
}

/// Yostar order: the CN choice, the generated schema, history newest first,
/// and the stored `yostar/` file LAST, as the fallback of record.
fn yostar_order(
    cn: Option<Spec>,
    generated: Option<Spec>,
    history: &[Spec],
    stored: Option<Spec>,
) -> Vec<Spec> {
    dedupe_specs(
        cn.into_iter()
            .chain(generated)
            .chain(history.iter().cloned())
            .chain(stored),
    )
}

/// One table's history versions as candidates, newest first.
fn load_history(store_dir: &Path, table: &str) -> Vec<Spec> {
    history::list(store_dir, table)
        .into_iter()
        .filter_map(|e| match std::fs::read(&e.path) {
            Ok(bytes) => Some(Spec {
                source: e.source(table),
                bytes,
            }),
            Err(err) => {
                eprintln!("warning: cannot read {}: {err}", e.path.display());
                None
            }
        })
        .collect()
}

/// The lock label of stored bytes: the history version they equal, else
/// `store`.
fn label_of(history: &[Spec], bytes: &[u8]) -> String {
    let want = normalize_fbs(&String::from_utf8_lossy(bytes));
    history
        .iter()
        .find(|s| normalize_fbs(&String::from_utf8_lossy(&s.bytes)) == want)
        .map_or_else(|| "store".to_string(), |s| s.source.clone())
}

// ---------------------------------------------------------------- per table

enum YostarFile {
    Keep,
    Remove,
    Write(Arc<Vec<u8>>),
}

struct Outcome {
    table: String,
    cn_bytes: Option<Arc<Vec<u8>>>,
    cn_lock: LockChoice,
    /// The CN head did not verify and history supplied the pick (or nothing
    /// did): `head source -> chosen source`.
    cn_fallback: Option<String>,
    yostar_file: YostarFile,
    yostar_lock: Option<LockChoice>,
    /// What the store held before this run decides, per server.
    before: BTreeMap<String, Option<bool>>,
    after: BTreeMap<String, Option<bool>>,
    /// Per server: did ANY candidate verify. Drives the regression exit.
    any: BTreeMap<String, bool>,
    /// The effective Yostar schema bytes after the run (for `--compare-dir`).
    yostar_effective: Option<Arc<Vec<u8>>>,
}

struct Inputs<'a> {
    ctx: &'a Ctx<'a>,
    store: &'a Path,
    prev: Option<&'a Lock>,
    generated: Option<&'a Generated>,
    cn: Option<&'a Server>,
    yostar: Vec<&'a Server>,
}

fn read_opt(path: &Path) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

fn same_schema(a: &[u8], b: &[u8]) -> bool {
    normalize_fbs(&String::from_utf8_lossy(a)) == normalize_fbs(&String::from_utf8_lossy(b))
}

#[allow(clippy::too_many_lines)]
fn resolve_table(inp: &Inputs<'_>, table: &str) -> Outcome {
    let ctx = inp.ctx;
    let old_cn = read_opt(&inp.store.join("cn").join(format!("{table}.fbs")));
    let old_yostar = read_opt(&inp.store.join("yostar").join(format!("{table}.fbs")));
    let prev = inp.prev.and_then(|l| l.tables.get(table));
    let hist = load_history(inp.store, table);

    // ---- before: what the store as it stands would do on today's bundles.
    let mut before: BTreeMap<String, Option<bool>> = BTreeMap::new();
    if let Some(cn) = inp.cn
        && cn.tables.contains_key(table)
    {
        let v = old_cn.as_ref().and_then(|b| {
            let c = ctx.cand(table, "store".into(), b.clone());
            ctx.verifies(&c, cn, table)
        });
        before.insert(cn.name.clone(), v);
    }
    let old_effective_yostar = match (&old_yostar, &old_cn) {
        (Some(y), Some(c)) if !same_schema(y, c) => Some(y.clone()),
        (Some(y), None) => Some(y.clone()),
        (_, c) => c.clone(),
    };
    for s in &inp.yostar {
        if s.tables.contains_key(table) {
            let v = old_effective_yostar.as_ref().and_then(|b| {
                let c = ctx.cand(table, "store".into(), b.clone());
                ctx.verifies(&c, s, table)
            });
            before.insert(s.name.clone(), v);
        }
    }

    let mut after = BTreeMap::new();
    let mut any = BTreeMap::new();

    // ---- CN: the generated schema (else the stored file) first, history as
    // the fallback.
    let generated = inp.generated.and_then(|g| {
        g.schemas.get(table).map(|b| Spec {
            source: format!("client@{}", g.version),
            bytes: b.clone(),
        })
    });
    let stored_cn = old_cn.as_ref().map(|b| Spec {
        source: label_of(&hist, b),
        bytes: b.clone(),
    });
    let head = generated.clone().or_else(|| stored_cn.clone());
    let head_source = head.as_ref().map(|h| h.source.clone());
    let cn_specs = cn_order(head, &hist);
    let mk = |s: Spec| ctx.cand(table, s.source, s.bytes);

    // An unverified pick: the lock records why, with an empty `verified`.
    let unverified = |c: Option<Cand>, note: &str| {
        let lock = LockChoice {
            source: c
                .as_ref()
                .map_or_else(|| "none".into(), |c| c.source.clone()),
            sha256: c.as_ref().map(|c| sha256_hex(&c.bytes)),
            verified: BTreeMap::new(),
            note: Some(note.to_string()),
        };
        (c, lock)
    };

    let mut cn_fallback = None;
    let (cn_choice, cn_lock): (Option<Cand>, LockChoice) = match inp.cn {
        // No CN bundle supplied: nothing to verify against, so the stored file
        // stands (a generated schema reaches `cn/` only through CN
        // verification). A table new to the store takes the head unverified.
        None => match stored_cn {
            Some(s) => {
                let sha = sha256_hex(&s.bytes);
                let lock = match prev {
                    Some(p) if p.cn.sha256.as_deref() == Some(sha.as_str()) => p.cn.clone(),
                    // Edited by hand since the lock was written, or new: its
                    // verdicts do not describe this file.
                    _ => LockChoice {
                        source: s.source.clone(),
                        sha256: Some(sha),
                        verified: BTreeMap::new(),
                        note: Some(
                            "no cn bundle supplied; file differs from the previous lock".into(),
                        ),
                    },
                };
                (Some(mk(s)), lock)
            }
            None => unverified(cn_specs.into_iter().next().map(mk), "no cn bundle supplied"),
        },
        Some(cn) if !cn.tables.contains_key(table) => unverified(
            cn_specs.into_iter().next().map(mk),
            "absent from the cn bundles",
        ),
        Some(cn) => {
            let mut found = None;
            for c in cn_specs.into_iter().map(mk) {
                if ctx.verifies(&c, cn, table) == Some(true) {
                    found = Some(c);
                    break;
                }
            }
            any.insert(cn.name.clone(), found.is_some());
            if let Some(c) = found {
                after.insert(cn.name.clone(), Some(true));
                let fell_back = head_source.as_deref().is_some_and(|h| h != c.source);
                if fell_back {
                    cn_fallback = Some(format!(
                        "{} -> {}",
                        head_source.as_deref().unwrap_or("none"),
                        c.source
                    ));
                }
                let lock = LockChoice {
                    source: c.source.clone(),
                    sha256: Some(sha256_hex(&c.bytes)),
                    verified: BTreeMap::from([(cn.name.clone(), true)]),
                    note: fell_back.then(|| {
                        format!(
                            "{} does not verify on cn; history fallback",
                            head_source.as_deref().unwrap_or("none")
                        )
                    }),
                };
                (Some(c), lock)
            } else {
                after.insert(cn.name.clone(), Some(false));
                cn_fallback = Some(format!(
                    "{} -> nothing verifies",
                    head_source.as_deref().unwrap_or("none")
                ));
                // Nothing verifies: keep the stored file, else the head.
                let keep = stored_cn.or(generated.clone()).map(mk);
                unverified(keep, "no candidate verifies on cn")
            }
        }
    };
    let cn_bytes = cn_choice.as_ref().map(|c| c.bytes.clone());

    // ---- Yostar.
    let present: Vec<&Server> = inp
        .yostar
        .iter()
        .copied()
        .filter(|s| s.tables.contains_key(table))
        .collect();
    let (yostar_file, yostar_lock, yostar_effective) = if inp.yostar.is_empty() {
        // No Yostar bundle supplied: the stored side stands as it is.
        (
            YostarFile::Keep,
            prev.and_then(|p| p.yostar.clone()),
            old_effective_yostar.map(Arc::new),
        )
    } else if present.is_empty() {
        (YostarFile::Keep, None, None)
    } else {
        let stored = old_yostar.as_ref().map(|b| Spec {
            source: label_of(&hist, b),
            bytes: b.clone(),
        });
        let specs = yostar_order(cn_choice.as_ref().map(Spec::of), generated, &hist, stored);
        resolve_yostar(
            inp,
            table,
            &present,
            cn_choice.as_ref(),
            specs,
            old_yostar.as_ref(),
            &hist,
            &mut after,
            &mut any,
        )
    };

    Outcome {
        table: table.to_string(),
        cn_bytes,
        cn_lock,
        cn_fallback,
        yostar_file,
        yostar_lock,
        before,
        after,
        any,
        yostar_effective,
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_yostar(
    inp: &Inputs<'_>,
    table: &str,
    present: &[&Server],
    cn_choice: Option<&Cand>,
    specs: Vec<Spec>,
    old_yostar: Option<&Vec<u8>>,
    hist: &[Spec],
    after: &mut BTreeMap<String, Option<bool>>,
    any: &mut BTreeMap<String, bool>,
) -> (YostarFile, Option<LockChoice>, Option<Arc<Vec<u8>>>) {
    let ctx = inp.ctx;
    let all = |v: &BTreeMap<String, bool>| v.len() == present.len() && v.values().all(|ok| *ok);

    let same_as_cn = |verified: BTreeMap<String, bool>| LockChoice {
        source: "same-as-cn".into(),
        sha256: None,
        verified,
        note: None,
    };

    // The variant is gone when the CN choice already verifies everywhere.
    if let Some(cn) = cn_choice {
        let v = ctx.verdicts(cn, present, table);
        if all(&v) {
            for (s, ok) in &v {
                after.insert(s.clone(), Some(*ok));
                any.insert(s.clone(), true);
            }
            return (
                YostarFile::Remove,
                Some(same_as_cn(v)),
                Some(cn.bytes.clone()),
            );
        }
    }

    let mut best: Option<(Cand, BTreeMap<String, bool>)> = None;
    let score = |v: &BTreeMap<String, bool>| {
        (
            v.values().filter(|ok| **ok).count(),
            v.get("en").copied().unwrap_or(false),
        )
    };
    for spec in specs {
        let c = ctx.cand(table, spec.source, spec.bytes);
        let v = ctx.verdicts(&c, present, table);
        for (s, ok) in &v {
            if *ok {
                any.insert(s.clone(), true);
            }
        }
        let done = all(&v);
        if best.as_ref().is_none_or(|(_, bv)| score(&v) > score(bv)) {
            best = Some((c, v));
        }
        if done {
            break;
        }
    }
    for s in present {
        any.entry(s.name.clone()).or_insert(false);
    }

    let cn_key = cn_choice.map(|c| c.key.clone());
    match best {
        Some((c, v)) if v.values().any(|ok| *ok) => {
            for (s, ok) in &v {
                after.insert(s.clone(), Some(*ok));
            }
            if Some(&c.key) == cn_key.as_ref() {
                (YostarFile::Remove, Some(same_as_cn(v)), Some(c.bytes))
            } else {
                let lock = LockChoice {
                    source: c.source.clone(),
                    sha256: Some(sha256_hex(&c.bytes)),
                    verified: v,
                    note: None,
                };
                (
                    YostarFile::Write(c.bytes.clone()),
                    Some(lock),
                    Some(c.bytes),
                )
            }
        }
        _ => {
            for s in present {
                after.insert(s.name.clone(), Some(false));
            }
            // Resolved under nothing: the previous file (if any) is kept and
            // `extract` keeps skipping the table.
            let lock = LockChoice {
                source: old_yostar.map_or_else(|| "unresolved".to_string(), |b| label_of(hist, b)),
                sha256: old_yostar.map(|b| sha256_hex(b)),
                verified: BTreeMap::new(),
                note: Some(format!(
                    "no candidate verifies on {}",
                    present
                        .iter()
                        .map(|s| s.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            };
            let effective = old_yostar
                .cloned()
                .map(Arc::new)
                .or(cn_choice.map(|c| c.bytes.clone()));
            (YostarFile::Keep, Some(lock), effective)
        }
    }
}

// ---------------------------------------------------------------- run

/// A lock source for the console: `history/<stem>/<label>.fbs` prints as
/// `history:<label>`, an old-format `<branch>@<sha>` keeps 7 hex digits.
fn short(source: &str) -> String {
    if let Some(rest) = source.strip_prefix("history/")
        && let Some((_, file)) = rest.split_once('/')
    {
        return format!("history:{}", file.strip_suffix(".fbs").unwrap_or(file));
    }
    match source.split_once('@') {
        Some((branch, sha)) if sha.len() > 7 => format!("{branch}@{}", &sha[..7]),
        _ => source.to_string(),
    }
}

fn fmt_verified(v: &BTreeMap<String, bool>) -> String {
    if v.is_empty() {
        return "{}".to_string();
    }
    v.iter()
        .map(|(s, ok)| format!("{s}:{}", if *ok { "ok" } else { "FAIL" }))
        .collect::<Vec<_>>()
        .join(" ")
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> anyhow::Result<bool> {
    if std::fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(true)
}

/// Runs the sync. `Ok(0)` resolved, `Ok(2)` regression; errors are exit 1.
/// What a run did beyond the per-table outcomes, for the console and report.
#[derive(Default)]
struct RunNotes {
    /// Stems in `cn/` that the generated directory does not carry; kept as
    /// they are.
    not_regenerated: Vec<String>,
    /// Stems the generated directory adds to `cn/`.
    new_stems: Vec<String>,
    /// History files this run added.
    history_added: Vec<String>,
    /// History files removed as duplicates of an older version.
    history_deduped: usize,
}

/// Runs the sync. `Ok(0)` resolved, `Ok(2)` regression; errors are exit 1.
#[allow(clippy::too_many_lines)]
pub fn run(opts: &Options) -> anyhow::Result<i32> {
    let store_dir = opts.store.clone();
    let lock_path = store_dir.join("schemas.lock.json");
    let prev = Lock::load(&lock_path)?;

    let cache_dir = opts
        .cache
        .clone()
        .unwrap_or_else(reflect::default_cache_dir);
    let cache = BfbsCache::new(&cache_dir)?;
    let generated = opts
        .generated
        .as_deref()
        .map(|d| load_generated(d, prev.as_ref()))
        .transpose()?;

    let stored_stems = store::list_stems(&store_dir.join("cn"));
    let mut tables = stored_stems.clone();
    let mut notes = RunNotes::default();
    if let Some(g) = &generated {
        let gen_stems: BTreeSet<String> = g.schemas.keys().cloned().collect();
        notes.not_regenerated = stored_stems.difference(&gen_stems).cloned().collect();
        notes.new_stems = gen_stems.difference(&stored_stems).cloned().collect();
        tables.extend(gen_stems);
        println!(
            "client: cn {} ({} schemas; GameAssembly.dll {}, global-metadata.dat {}); \
             missing roots: {}",
            g.version,
            g.schemas.len(),
            &g.client.game_assembly_sha256[..12.min(g.client.game_assembly_sha256.len())],
            &g.client.metadata_sha256[..12.min(g.client.metadata_sha256.len())],
            if g.missing_roots.is_empty() {
                "none".to_string()
            } else {
                g.missing_roots.join(", ")
            }
        );
    } else {
        println!("client: none (no --generated); cn is the stored cn/");
    }
    // Before anything writes `cn/`: field types the verifier cannot check.
    let type_changes = generated.as_ref().map_or_else(
        || {
            prev.as_ref()
                .map(|p| p.type_changes.clone())
                .unwrap_or_default()
        },
        |g| field_type_changes(&store_dir, g, prev.as_ref()),
    );
    println!("field-type-changes: {}", type_changes.len());
    for c in &type_changes {
        println!("  {c}");
    }
    for t in &tables {
        notes.history_deduped += history::dedupe(&store_dir, t)?.len();
    }
    if let Some(g) = &generated {
        let today = rfc3339_utc(std::time::SystemTime::now());
        let base = history::label(&today[..10], &g.version);
        for (stem, bytes) in &g.schemas {
            if let Some(label) = history::add(&store_dir, stem, &base, bytes)? {
                notes.history_added.push(history::source(stem, &label));
            }
        }
    }
    println!(
        "history: {} version(s) added, {} duplicate(s) removed",
        notes.history_added.len(),
        notes.history_deduped
    );
    for orphan in store::list_stems(&store_dir.join("yostar")).difference(&tables) {
        eprintln!("warning: yostar/{orphan}.fbs has no cn/ counterpart and is left untouched");
    }

    let mut seen_names = HashSet::new();
    for (name, _) in &opts.bundles {
        if !seen_names.insert(name.clone()) {
            bail!("--bundles {name}= given twice");
        }
    }
    let servers: Vec<Server> = opts
        .bundles
        .par_iter()
        .map(|(name, dir)| load_server(name, dir, &tables, opts.levels_sample))
        .collect::<anyhow::Result<_>>()?;
    for s in &servers {
        let buffers: usize = s.totals.values().sum();
        println!(
            "{}: {} ({} tables, {buffers} buffers, {} unrouted)",
            s.name,
            s.res_version,
            s.tables.len(),
            s.unrouted.len()
        );
        if !s.unrouted.is_empty() {
            println!("  unrouted: {}", s.unrouted.join(" "));
        }
    }

    let ctx = Ctx {
        cache: &cache,
        memo: Mutex::new(HashMap::new()),
    };
    let inputs = Inputs {
        ctx: &ctx,
        store: &store_dir,
        prev: prev.as_ref(),
        generated: generated.as_ref(),
        cn: servers.iter().find(|s| s.name == "cn"),
        yostar: servers.iter().filter(|s| s.name != "cn").collect(),
    };
    let table_list: Vec<String> = tables.iter().cloned().collect();
    let outcomes: Vec<Outcome> = table_list
        .par_iter()
        .map(|t| resolve_table(&inputs, t))
        .collect();

    // ---- write the store.
    let mut lock = Lock {
        client: generated.as_ref().map(|g| g.client.clone()),
        type_changes,
        flatc: cache.flatc().to_string(),
        servers: servers
            .iter()
            .map(|s| {
                (
                    s.name.clone(),
                    LockServer {
                        res_version: s.res_version.clone(),
                    },
                )
            })
            .collect(),
        tables: BTreeMap::new(),
    };
    let mut written = notes.history_added.len();
    let mut removed = notes.history_deduped;
    for o in &outcomes {
        if let Some(bytes) = &o.cn_bytes
            && write_if_changed(
                &store_dir.join("cn").join(format!("{}.fbs", o.table)),
                bytes,
            )?
        {
            written += 1;
        }
        let ypath = store_dir.join("yostar").join(format!("{}.fbs", o.table));
        match &o.yostar_file {
            YostarFile::Keep => {}
            YostarFile::Remove => {
                if ypath.exists() {
                    std::fs::remove_file(&ypath)?;
                    removed += 1;
                }
            }
            YostarFile::Write(bytes) => {
                if write_if_changed(&ypath, bytes)? {
                    written += 1;
                }
            }
        }
        lock.tables.insert(
            o.table.clone(),
            LockTable {
                cn: o.cn_lock.clone(),
                yostar: o.yostar_lock.clone(),
            },
        );
        println!(
            "{:<26} cn: {:<24} {:<10} | yostar: {}",
            o.table,
            short(&o.cn_lock.source),
            fmt_verified(&o.cn_lock.verified),
            o.yostar_lock.as_ref().map_or_else(
                || "null".to_string(),
                |y| format!("{:<24} {}", short(&y.source), fmt_verified(&y.verified))
            )
        );
    }
    let json = serde_json::to_string_pretty(&lock)? + "\n";
    let lock_changed = write_if_changed(&lock_path, json.as_bytes())?;
    println!(
        "store: {written} file(s) written, {removed} removed, lock {} {}",
        if lock_changed {
            "rewritten"
        } else {
            "unchanged"
        },
        lock_path.display()
    );
    if !notes.not_regenerated.is_empty() {
        println!(
            "not regenerated (kept as stored): {}",
            notes.not_regenerated.join(", ")
        );
    }
    let fallbacks: Vec<String> = outcomes
        .iter()
        .filter_map(|o| o.cn_fallback.as_ref().map(|f| format!("{} {f}", o.table)))
        .collect();
    if !fallbacks.is_empty() {
        println!("cn history fallback: {}", fallbacks.join("; "));
    }

    // ---- regressions: verified in the previous lock, no candidate now.
    let now: Vec<(&str, &BTreeMap<String, bool>)> = outcomes
        .iter()
        .map(|o| (o.table.as_str(), &o.any))
        .collect();
    let regressions = prev
        .as_ref()
        .map_or_else(Vec::new, |p| find_regressions(p, &now));

    if let Some(path) = &opts.report {
        let md = report(
            &lock,
            prev.as_ref(),
            &outcomes,
            &servers,
            &regressions,
            &notes,
        );
        std::fs::write(path, md).with_context(|| format!("write {}", path.display()))?;
        println!("report: {}", path.display());
    }
    if let Some(dir) = &opts.compare_dir {
        compare(&ctx, dir, &outcomes, &servers);
    }

    let unresolved: Vec<String> = outcomes
        .iter()
        .flat_map(|o| {
            o.after
                .iter()
                .filter(|(_, v)| **v == Some(false))
                .map(move |(s, _)| format!("{} on {s}", o.table))
        })
        .collect();
    if !unresolved.is_empty() {
        println!(
            "warning: chosen schema does not verify: {}",
            unresolved.join(", ")
        );
    }
    if regressions.is_empty() {
        Ok(0)
    } else {
        eprintln!(
            "REGRESSION: resolved in the previous lock, no candidate verifies now: {}",
            regressions.join(", ")
        );
        Ok(EXIT_REGRESSION)
    }
}

/// Every field and enum type change between the stored `cn/` schemas and
/// the generated ones, per `fields::type_changes`, stems in order.
///
/// A stem whose stored file already IS the generated schema (a rerun after
/// this client was written) has nothing left to compare, so while the client
/// is the previous lock's (version and both hashes) its entries are carried
/// over from the previous lock: a rerun on the same generated directory
/// reproduces the list.
fn field_type_changes(store_dir: &Path, g: &Generated, prev: Option<&Lock>) -> Vec<String> {
    let same_client = prev.and_then(|p| p.client.as_ref()).is_some_and(|c| {
        c.version == g.client.version
            && c.game_assembly_sha256 == g.client.game_assembly_sha256
            && c.metadata_sha256 == g.client.metadata_sha256
    });
    let mut out = Vec::new();
    for (stem, new) in &g.schemas {
        let Some(old) = read_opt(&store_dir.join("cn").join(format!("{stem}.fbs"))) else {
            continue;
        };
        if same_schema(&old, new) {
            if same_client && let Some(p) = prev {
                let prefix = format!("{stem}: ");
                out.extend(
                    p.type_changes
                        .iter()
                        .filter(|c| c.starts_with(&prefix))
                        .cloned(),
                );
            }
            continue;
        }
        out.extend(fields::type_changes(
            stem,
            &String::from_utf8_lossy(&old),
            &String::from_utf8_lossy(new),
        ));
    }
    out
}

/// `"<table> on <server>"` for every (table, server) the previous lock marks
/// verified where no candidate verifies now. `now` carries, per table, whether
/// any candidate verified on each server where the table is present; a server
/// or table missing from `now` (not supplied, or gone from the bundles) is not
/// a regression.
fn find_regressions(prev: &Lock, now: &[(&str, &BTreeMap<String, bool>)]) -> Vec<String> {
    let mut out = Vec::new();
    for (table, any) in now {
        let Some(p) = prev.tables.get(*table) else {
            continue;
        };
        for side in std::iter::once(&p.cn).chain(p.yostar.as_ref()) {
            for (server, ok) in &side.verified {
                if *ok && any.get(server) == Some(&false) {
                    out.push(format!("{table} on {server}"));
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn report(
    lock: &Lock,
    prev: Option<&Lock>,
    outcomes: &[Outcome],
    servers: &[Server],
    regressions: &[String],
    notes: &RunNotes,
) -> String {
    let mut md = String::new();
    let _ = writeln!(md, "## Schema sync\n");
    match &lock.client {
        Some(c) => {
            let prev_version = prev
                .and_then(|p| p.client.as_ref())
                .map(|p| p.version.as_str());
            let version = match prev_version {
                Some(p) if p != c.version => format!("{p} -> {}", c.version),
                _ => c.version.clone(),
            };
            let _ = writeln!(
                md,
                "CN client `{version}` (GameAssembly.dll `{}`, global-metadata.dat `{}`, generated {}), flatc {}.\n",
                &c.game_assembly_sha256[..12.min(c.game_assembly_sha256.len())],
                &c.metadata_sha256[..12.min(c.metadata_sha256.len())],
                c.generated_at,
                lock.flatc
            );
        }
        None => {
            let _ = writeln!(
                md,
                "No generated schemas this run: CN is the stored `cn/`. flatc {}.\n",
                lock.flatc
            );
        }
    }
    if !lock.type_changes.is_empty() {
        let _ = writeln!(
            md,
            "## Field type changes ({})\n\n\
             An existing field or enum changed type between the stored schema and the \
             generated one. The verifier cannot see this; check each by hand before merging.\n",
            lock.type_changes.len()
        );
        for c in &lock.type_changes {
            let _ = writeln!(md, "- `{c}`");
        }
        md.push('\n');
    }
    let fallbacks: Vec<String> = outcomes
        .iter()
        .filter_map(|o| o.cn_fallback.as_ref().map(|f| format!("`{}` {f}", o.table)))
        .collect();
    for (label, list) in [
        ("Not regenerated (kept as stored)", &notes.not_regenerated),
        ("New in the client", &notes.new_stems),
        ("History fallback (cn)", &fallbacks),
        ("History added", &notes.history_added),
    ] {
        if !list.is_empty() {
            let _ = writeln!(md, "- {label} ({}): {}", list.len(), list.join(", "));
        }
    }
    if !(notes.not_regenerated.is_empty()
        && notes.new_stems.is_empty()
        && fallbacks.is_empty()
        && notes.history_added.is_empty())
    {
        md.push('\n');
    }
    let _ = writeln!(
        md,
        "| server | resVersion | tables | verified before | verified after | fixed | still failing | regressed | newly unresolved |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|---|---|---|---|");
    let mut details = String::new();
    for s in servers {
        let mut fixed = Vec::new();
        let mut failing = Vec::new();
        let mut regressed = Vec::new();
        let mut newly = Vec::new();
        let (mut n, mut vb, mut va) = (0, 0, 0);
        for o in outcomes {
            let Some(after) = o.after.get(&s.name).copied().flatten() else {
                continue;
            };
            n += 1;
            let before = o.before.get(&s.name).copied().flatten();
            vb += usize::from(before == Some(true));
            va += usize::from(after);
            match (before, after) {
                (Some(false), true) => fixed.push(o.table.as_str()),
                (Some(false), false) => failing.push(o.table.as_str()),
                (Some(true), false) => regressed.push(o.table.as_str()),
                (None, false) => newly.push(o.table.as_str()),
                _ => {}
            }
        }
        let prev_version = prev
            .and_then(|p| p.servers.get(&s.name))
            .map(|v| v.res_version.as_str());
        let version = match prev_version {
            Some(p) if p != s.res_version => format!("{p} -> {}", s.res_version),
            _ => s.res_version.clone(),
        };
        let _ = writeln!(
            md,
            "| {} | `{version}` | {n} | {vb} | {va} | {} | {} | {} | {} |",
            s.name,
            fixed.len(),
            failing.len(),
            regressed.len(),
            newly.len()
        );
        for (label, list) in [
            ("fixed", &fixed),
            ("still failing", &failing),
            ("regressed", &regressed),
            ("newly unresolved", &newly),
        ] {
            if !list.is_empty() {
                let _ = writeln!(details, "- {} {label}: {}", s.name, list.join(", "));
            }
        }
    }
    if !details.is_empty() {
        let _ = writeln!(md, "\n{details}");
    }
    if !regressions.is_empty() {
        let _ = writeln!(
            md,
            "\n**REGRESSION** (verified in the previous lock, no candidate verifies now): {}\n",
            regressions.join(", ")
        );
    }

    let mut changes = String::new();
    for (table, now) in &lock.tables {
        let old = prev.and_then(|p| p.tables.get(table));
        let cn_old = old.map(|o| o.cn.source.as_str());
        let y_old = old
            .and_then(|o| o.yostar.as_ref())
            .map(|y| y.source.as_str());
        let y_now = now.yostar.as_ref().map(|y| y.source.as_str());
        if cn_old != Some(now.cn.source.as_str())
            || (old.is_some() && y_old != y_now)
            || old.is_none()
        {
            let _ = writeln!(
                changes,
                "| {table} | {} -> {} | {} -> {} |",
                cn_old.map_or_else(|| "-".to_string(), short),
                short(&now.cn.source),
                y_old.map_or_else(|| "-".to_string(), short),
                y_now.map_or_else(|| "null".to_string(), short),
            );
        }
    }
    if !changes.is_empty() {
        let _ = writeln!(md, "\n| table | cn | yostar |\n|---|---|---|");
        md.push_str(&changes);
    }
    md
}

/// `--compare-dir`: our effective Yostar schema against another tree's, table
/// by table, with both verification matrices.
fn compare(ctx: &Ctx<'_>, dir: &Path, outcomes: &[Outcome], servers: &[Server]) {
    let yostar: Vec<&Server> = servers.iter().filter(|s| s.name != "cn").collect();
    let theirs = store::list_stems(dir);
    let (mut same, mut different) = (0, 0);
    println!("compare against {}:", dir.display());
    for stem in &theirs {
        let Ok(their_bytes) = std::fs::read(dir.join(format!("{stem}.fbs"))) else {
            continue;
        };
        let Some(o) = outcomes.iter().find(|o| &o.table == stem) else {
            println!("  {stem:<26} not a store table");
            continue;
        };
        let provenance = String::from_utf8_lossy(&their_bytes)
            .lines()
            .next()
            .and_then(|l| {
                l.strip_prefix("// https://github.com/MooncellWiki/OpenArknightsFBS/commit/")
            })
            .map_or_else(
                || "YoStar head (no comment)".to_string(),
                |c| c[..7.min(c.len())].to_string(),
            );
        let ours = o.yostar_effective.clone();
        let is_same = ours.as_ref().is_some_and(|b| same_schema(b, &their_bytes));
        let our_source = o.yostar_lock.as_ref().map_or("null", |y| y.source.as_str());
        if is_same {
            same += 1;
            println!("  {stem:<26} SAME      ours {}", short(our_source));
        } else {
            different += 1;
            let tc = ctx.cand(stem, "theirs".into(), their_bytes);
            let tv = ctx.verdicts(&tc, &yostar, stem);
            let ov = ours.map(|b| {
                let oc = ctx.cand(stem, "ours".into(), b.to_vec());
                ctx.verdicts(&oc, &yostar, stem)
            });
            println!(
                "  {stem:<26} DIFFERENT ours {} [{}] | theirs {provenance} [{}]",
                short(our_source),
                ov.as_ref().map_or_else(|| "-".to_string(), fmt_verified),
                fmt_verified(&tv)
            );
        }
    }
    println!("compare: {same} same, {different} different");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choice(verified: &[(&str, bool)]) -> LockChoice {
        LockChoice {
            source: "client@2.7.81".into(),
            sha256: None,
            verified: verified
                .iter()
                .map(|(s, v)| ((*s).to_string(), *v))
                .collect(),
            note: None,
        }
    }

    #[test]
    fn regression_needs_a_previous_pass_and_no_candidate_now() {
        let mut prev = Lock::default();
        prev.tables.insert(
            "stage_table".into(),
            LockTable {
                cn: choice(&[("cn", true)]),
                yostar: Some(choice(&[("en", true), ("jp", true), ("kr", false)])),
            },
        );
        let any: BTreeMap<String, bool> =
            [("cn", true), ("en", false), ("jp", true), ("kr", false)]
                .iter()
                .map(|(s, v)| ((*s).to_string(), *v))
                .collect();
        // en passed before and nothing verifies now: regression. kr failed
        // before: a warning, not a regression. jp still resolves.
        assert_eq!(
            find_regressions(&prev, &[("stage_table", &any)]),
            vec!["stage_table on en".to_string()]
        );
        // A server not supplied this run is not a regression.
        let partial: BTreeMap<String, bool> = std::iter::once(("cn".to_string(), true)).collect();
        assert!(find_regressions(&prev, &[("stage_table", &partial)]).is_empty());
    }

    fn spec(source: &str, text: &str) -> Spec {
        Spec {
            source: source.to_string(),
            bytes: text.as_bytes().to_vec(),
        }
    }

    fn sources(specs: &[Spec]) -> Vec<&str> {
        specs.iter().map(|s| s.source.as_str()).collect()
    }

    #[test]
    fn yostar_order_is_cn_generated_history_then_stored() {
        // History newest first as `history::list` returns it; the stored
        // Yostar file is an old version that history also holds.
        let hist = vec![
            spec(
                "history/t/2026-10-15_2.7.81",
                "table T { a: int; b: int; c: int; }",
            ),
            spec("history/t/2026-09-04_2.7.71", "table T { a: int; b: int; }"),
            spec("history/t/2025-06-05_2.5.80", "table T { a: int; }"),
        ];
        let cn = spec(
            "client@2.7.81",
            "// generated\ntable T { a: int; b: int; c: int; }",
        );
        let generated = spec("client@2.7.81", "table T { a: int; b: int; c: int; }");
        let stored = spec("history/t/2025-06-05_2.5.80", "table T {\n  a: int;\n}");
        let order = yostar_order(Some(cn), Some(generated), &hist, Some(stored.clone()));
        // The generated schema and the newest history version equal the CN
        // choice after normalisation and the stored file equals 2.5.80, so
        // each is tried once, at its first position.
        assert_eq!(
            sources(&order),
            vec![
                "client@2.7.81",
                "history/t/2026-09-04_2.7.71",
                "history/t/2025-06-05_2.5.80",
            ]
        );
        // A stored file that no history version holds is tried LAST.
        let stray = spec("store", "table T { a: long; }");
        let order = yostar_order(None, None, &hist, Some(stray));
        assert_eq!(
            sources(&order),
            vec![
                "history/t/2026-10-15_2.7.81",
                "history/t/2026-09-04_2.7.71",
                "history/t/2025-06-05_2.5.80",
                "store",
            ]
        );
    }

    #[test]
    fn cn_order_is_head_then_history() {
        let hist = vec![
            spec("history/t/2026-10-09_2.7.81", "table T { a: int; b: int; }"),
            spec("history/t/2026-09-04_2.7.71", "table T { a: int; }"),
        ];
        let head = spec("client@2.7.91", "table T { a: int; b: int; x: int; }");
        assert_eq!(
            sources(&cn_order(Some(head), &hist)),
            vec![
                "client@2.7.91",
                "history/t/2026-10-09_2.7.81",
                "history/t/2026-09-04_2.7.71",
            ]
        );
        assert_eq!(
            label_of(&hist, b"table T {\n a: int;\n}"),
            "history/t/2026-09-04_2.7.71"
        );
        assert_eq!(label_of(&hist, b"table U {}"), "store");
        assert_eq!(
            short("history/t/2026-09-04_2.7.71.fbs"),
            "history:2026-09-04_2.7.71"
        );
    }

    #[test]
    fn type_changes_compare_stored_cn_and_carry_over_on_rerun() {
        let dir = std::env::temp_dir().join(format!("myrtle-sync-tc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("cn")).unwrap();
        std::fs::write(dir.join("cn/t.fbs"), "table T {\n    a: int;\n}\n").unwrap();
        let client = LockClient {
            version: "2.7.81".into(),
            generated_at: String::new(),
            game_assembly_sha256: "aa".into(),
            metadata_sha256: "bb".into(),
        };
        let g = Generated {
            version: "2.7.81".into(),
            client: client.clone(),
            schemas: BTreeMap::from([
                ("t".to_string(), b"table T {\n    a: float;\n}\n".to_vec()),
                ("u".to_string(), b"table U {}\n".to_vec()),
            ]),
            missing_roots: Vec::new(),
        };
        let first = field_type_changes(&dir, &g, None);
        assert_eq!(first, vec!["t: T.a: int -> float".to_string()]);

        // The run wrote the generated schema to cn/: the rerun on the same
        // client carries the entry over, a new client does not.
        std::fs::write(dir.join("cn/t.fbs"), &g.schemas["t"]).unwrap();
        let prev = Lock {
            client: Some(client),
            type_changes: first.clone(),
            ..Lock::default()
        };
        assert_eq!(field_type_changes(&dir, &g, Some(&prev)), first);
        let mut newer = prev.clone();
        newer.client.as_mut().unwrap().version = "2.7.71".into();
        assert!(field_type_changes(&dir, &g, Some(&newer)).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn lock_round_trips_with_null_yostar() {
        let mut lock = Lock::default();
        lock.tables.insert(
            "arkvent_table".into(),
            LockTable {
                cn: choice(&[("cn", true)]),
                yostar: None,
            },
        );
        let json = serde_json::to_string(&lock).unwrap();
        assert!(json.contains("\"yostar\":null"));
        let back: Lock = serde_json::from_str(&json).unwrap();
        assert!(back.tables["arkvent_table"].yostar.is_none());
    }
}
