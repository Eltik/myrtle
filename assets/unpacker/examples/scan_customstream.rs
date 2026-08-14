//! Corpus scan: which `m_VertexStreams` ids accompany which `CustomData` SLOT/COMPONENT that
//! actually carries a curve?
//!
//! The exporter maps `30..=33 -> slot 0 (Custom1)` and `34..=37 -> slot 1 (Custom2)`. That mapping
//! was validated on Civilight Eterna, but Wiš'adel's `stroke_01 (1)` sends `[0,1,3,4,34,38]` while
//! its only curve lives in slot 0 component 2 — under the current mapping the exporter looks in
//! slot 1, finds constants, and emits no `ramDissolveCurve` at all, so the stroke never wipes.
//!
//! Rather than pick a Unity enum from memory, print the joint distribution and let the corpus say
//! which offset is consistent with every skin at once.
//!
//! Usage: cargo run --release --example `scan_customstream` -- <bundle.ab> [<bundle.ab>...]
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::or_fun_call,
    clippy::too_many_lines
)]

use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn b(v: &Value, k: &str) -> bool {
    v.get(k).and_then(Value::as_bool).unwrap_or(false)
        || v.get(k).and_then(Value::as_i64).unwrap_or(0) != 0
}

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    // (custom stream ids present, slot, component) -> count
    let mut joint: BTreeMap<(String, usize, usize), usize> = BTreeMap::new();
    let mut examples: BTreeMap<(String, usize, usize), String> = BTreeMap::new();
    // PAYLOAD POSITION (1-indexed, as the Ram vertex program consumes it) -> count
    let mut payload_hits: BTreeMap<usize, usize> = BTreeMap::new();
    let mut payload_by_skin: BTreeMap<(String, usize), usize> = BTreeMap::new();

    for path in &paths {
        let short = path.rsplit('/').next().unwrap_or(path).to_string();
        let Ok(data) = std::fs::read(path) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        for entry in &bundle.files {
            let lower = entry.path.to_ascii_lowercase();
            if lower.ends_with(".ress") || lower.ends_with(".resource") {
                continue;
            }
            let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                continue;
            };
            let skip: HashSet<i32> = [28, 43, 48, 49, 83, 128, 213].into_iter().collect();
            let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
            for obj in &sf.objects {
                if skip.contains(&obj.class_id) {
                    continue;
                }
                if let Ok(v) = read_object(&sf, obj) {
                    all.insert(obj.path_id, (obj.class_id, v));
                }
            }
            // GameObject -> its ParticleSystemRenderer vertex streams
            let mut streams_by_go: HashMap<i64, Vec<i64>> = HashMap::new();
            for (cid, v) in all.values() {
                if *cid != 199 {
                    continue; // ParticleSystemRenderer
                }
                let Some(go) = v
                    .get("m_GameObject")
                    .and_then(|g| g.get("m_PathID"))
                    .and_then(Value::as_i64)
                else {
                    continue;
                };
                let ids: Vec<i64> = v
                    .get("m_VertexStreams")
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(Value::as_i64).collect())
                    .unwrap_or_default();
                streams_by_go.insert(go, ids);
            }
            for (cid, v) in all.values() {
                if *cid != 198 {
                    continue; // ParticleSystem
                }
                let Some(cdm) = v.get("CustomDataModule") else {
                    continue;
                };
                if !b(cdm, "enabled") {
                    continue;
                }
                let Some(go) = v
                    .get("m_GameObject")
                    .and_then(|g| g.get("m_PathID"))
                    .and_then(Value::as_i64)
                else {
                    continue;
                };
                let ids = streams_by_go.get(&go).cloned().unwrap_or_default();
                // only the ids above the fixed built-ins are interesting
                let custom: Vec<i64> = ids.iter().copied().filter(|&x| x >= 20).collect();
                let key_ids = format!("{custom:?}");
                let name = v
                    .get("m_Name")
                    .and_then(Value::as_str)
                    .unwrap_or("?")
                    .to_string();
                for slot in 0..2usize {
                    if cdm
                        .get(format!("mode{slot}").as_str())
                        .and_then(Value::as_i64)
                        .unwrap_or(0)
                        != 1
                    {
                        continue;
                    }
                    for comp in 0..4usize {
                        let Some(vc) = cdm.get(format!("vector{slot}_{comp}").as_str()) else {
                            continue;
                        };
                        let n = vc
                            .get("maxCurve")
                            .and_then(|c| c.get("m_Curve"))
                            .and_then(Value::as_array)
                            .map_or(0, Vec::len);
                        if n > 1 {
                            let k = (key_ids.clone(), slot, comp);
                            *joint.entry(k.clone()).or_default() += 1;
                            examples.entry(k).or_insert(format!("{short}:{name}"));
                            // Where does this (slot, comp) LAND in the texcoord payload?
                            let mut pos = 0usize;
                            let mut found = None;
                            for id in ids.iter().copied() {
                                let (s2, n2) = match id {
                                    31..=34 => (0usize, (id - 30) as usize),
                                    35..=38 => (1usize, (id - 34) as usize),
                                    _ => continue,
                                };
                                for c2 in 0..n2 {
                                    pos += 1;
                                    if s2 == slot && c2 == comp {
                                        found = Some(pos);
                                    }
                                }
                            }
                            if let Some(pp) = found {
                                *payload_hits.entry(pp).or_default() += 1;
                                *payload_by_skin.entry((short.clone(), pp)).or_default() += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("== PAYLOAD POSITION (what the Ram vertex program does with it) ==");
    let what = |p: usize| match p {
        1 | 2 => "main UV offset",
        3 | 4 => "dissolve UV offset  [SHIPPED]",
        5 => "dissolve threshold  [read]",
        6 => "disturb intensity",
        7 | 8 => "disturb UV offset",
        _ => "beyond TEXCOORD2",
    };
    for (pp, n) in &payload_hits {
        println!("   payload {pp}: {n:5} systems   {}", what(*pp));
    }
    println!("\n== per REFERENCE skin ==");
    let refs = [
        "char_1012_skadi2_iteration#2",
        "char_1032_excu2_sale#12",
        "char_245_cello_sale#12",
        "char_4064_mlynar_epoque#28",
        "char_249_mlyss_boc#8",
        "char_1016_agoat2_epoque#34",
        "char_4134_cetsyr_epoque#50",
        "char_1035_wisdel_sale#14",
        "char_1038_whitw2_sale#15",
    ];
    for r in refs {
        let f = format!("{r}.ab");
        let row: Vec<String> = (1..=8)
            .filter_map(|pp| {
                payload_by_skin
                    .get(&(f.clone(), pp))
                    .map(|n| format!("p{pp}={n}"))
            })
            .collect();
        if !row.is_empty() {
            println!("   {:34} {}", r, row.join("  "));
        }
    }
    println!();
    println!(
        "{:<22} {:>5} {:>5} {:>7}   example",
        "customVertexStreams", "slot", "comp", "count"
    );
    for ((ids, slot, comp), n) in &joint {
        println!(
            "{:<22} {:>5} {:>5} {:>7}   {}",
            ids,
            slot,
            comp,
            n,
            examples
                .get(&(ids.clone(), *slot, *comp))
                .cloned()
                .unwrap_or_default()
        );
    }
}
