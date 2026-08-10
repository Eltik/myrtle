//! THROWAWAY diagnostic: does any spine renderer carry SEPARATOR SLOTS?
//!
//! Motivation: Virtuosa's `bg_*` haze sheets are authored ABOVE `characterSort`, so Unity
//! draws them in FRONT of the spine — but the game does not wash her body. That is only
//! expressible if the skeleton's draw is SPLIT and the sheet is interleaved between the
//! parts. Spine-Unity does exactly that via `SkeletonRenderSeparator` + the renderer's
//! `separatorSlotNames`, and both classes exist in the IL2CPP dump. If those names are
//! serialized on the prefab, they ARE the authoritative background/character split point
//! that no bone hierarchy provides (all 328 of cello's slots share the root bone `root`).
//!
//! Usage: cargo run --release --example probe_separator -- <bundle.ab>
use serde_json::Value;
use std::collections::HashMap;
use unpacker::unity::{bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile};

fn main() {
    let path = std::env::args().nth(1).expect("usage: probe_separator <bundle.ab>");
    let data = std::fs::read(&path).expect("read");
    let bundle = BundleFile::parse(data).expect("bundle");
    for entry in &bundle.files {
        let lower = entry.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else { continue };
        let mut all: HashMap<i64, (i32, Value)> = HashMap::new();
        for o in &sf.objects {
            if let Ok(v) = read_object(&sf, o) {
                all.insert(o.path_id, (o.class_id, v));
            }
        }
        for (pid, (cid, v)) in &all {
            if *cid != 114 {
                continue;
            }
            let has_skel = v.get("skeletonDataAsset").is_some();
            // Any field whose name mentions a separator, on ANY MonoBehaviour.
            let sep: Vec<(&String, &Value)> = v
                .as_object()
                .map(|m| m.iter().filter(|(k, _)| k.to_lowercase().contains("separator") || k.to_lowercase().contains("partsrenderer")).collect())
                .unwrap_or_default();
            if !sep.is_empty() {
                println!("  pid {pid} skeletonDataAsset={has_skel} go={:?}", v.get("m_GameObject").and_then(|g| g.get("m_PathID")).and_then(Value::as_i64));
                for (k, val) in sep {
                    println!("      {k} = {}", serde_json::to_string(val).unwrap_or_default());
                    // Resolve each SkeletonPartsRenderer to its GameObject's MeshRenderer
                    // sortingOrder: the PARTS have their own depths, and a sheet belongs in the
                    // gap exactly when its own sort falls BETWEEN two consecutive parts.
                    if k == "partsRenderers" {
                        for e in val.as_array().into_iter().flatten() {
                            let Some(rp) = e.get("m_PathID").and_then(Value::as_i64) else { continue };
                            let Some((_, rv)) = all.get(&rp) else { println!("        part pid {rp}: NOT FOUND"); continue };
                            let go = rv.get("m_GameObject").and_then(|g| g.get("m_PathID")).and_then(Value::as_i64);
                            let mut order = None;
                            let mut layer = None;
                            if let Some(go) = go {
                                for (_, (cid2, v2)) in all.iter() {
                                    if *cid2 == 23
                                        && v2.get("m_GameObject").and_then(|g| g.get("m_PathID")).and_then(Value::as_i64) == Some(go)
                                    {
                                        order = v2.get("m_SortingOrder").and_then(Value::as_i64);
                                        layer = v2.get("m_SortingLayerID").and_then(Value::as_i64);
                                    }
                                }
                            }
                            println!("        part pid {rp} go={go:?} sortingOrder={order:?} sortingLayer={layer:?}");
                        }
                    }
                }
            } else if has_skel {
                let keys: Vec<&String> = v.as_object().map(|m| m.keys().collect()).unwrap_or_default();
                let _ = keys; println!("  pid {pid} SKELETON MonoBehaviour (no separator field) go={:?}", v.get("m_GameObject").and_then(|g| g.get("m_PathID")).and_then(Value::as_i64));
            }
        }
    }
}
