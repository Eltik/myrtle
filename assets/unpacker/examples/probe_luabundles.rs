//! Resolve `lua/` asset paths to their downloadable bundles via the master `.idx`.
//!
//! Motivation: `Torappu.UI.DynIllustStartMgr` is xLua-hooked wholesale (every method is a short
//! stub tail-calling `DelegateBridge`), so the entrance-end timing that five data sources failed to
//! predict lives in Lua script, not in the binary or the scene JSON. The scripts are not in the apk
//! — it ships only `libxlua.so` — but the `.idx` carries a `lua/` namespace of ~333 entries.
//!
//! Prints each `lua/` asset's bundle, and a bundle->count rollup so the fetch list is obvious.
//!
//! Usage: cargo run --release --example `probe_luabundles` -- <path-to.idx> [path-substring]
#![allow(clippy::cast_sign_loss)]

use std::collections::BTreeMap;

use unpacker::export::resource_manifest_generated::root_as_clz_torappu_resource_resource_manifest_unchecked as root_as_manifest;

fn main() {
    let mut args = std::env::args().skip(1);
    let idx = args.next().expect("path to .idx");
    let want = args.next().unwrap_or_else(|| "lua/".into());

    let data = std::fs::read(&idx).expect("read idx");
    assert!(data.len() > 128, "idx shorter than its RSA signature");
    // Skip the 128-byte RSA signature (same as `ResourceManifest::load`).
    let m = unsafe { root_as_manifest(&data[128..]) };

    let bundles: Vec<String> = m
        .bundles()
        .map(|b| {
            b.iter()
                .map(|x| x.name().unwrap_or("?").to_string())
                .collect()
        })
        .unwrap_or_default();
    println!("bundles in manifest: {}", bundles.len());

    let Some(list) = m.assetToBundleList() else {
        eprintln!("no assetToBundleList");
        return;
    };
    let mut per_bundle: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut total = 0usize;
    for a in list {
        let path = a.path().unwrap_or("");
        if !path.contains(&want) {
            continue;
        }
        // `luaui/` entries are UI PREFABS driven by Lua, not scripts — exclude them so the
        // fetch list is only script bundles.
        if want == "lua/" && path.contains("luaui/") {
            continue;
        }
        total += 1;
        let bi = a.bundleIndex() as usize;
        let bn = bundles
            .get(bi)
            .cloned()
            .unwrap_or_else(|| format!("<oob {bi}>"));
        per_bundle.entry(bn).or_default().push(path.to_string());
    }
    println!(
        "matching assets: {total}   distinct bundles: {}\n",
        per_bundle.len()
    );
    for (b, paths) in &per_bundle {
        println!("{b}   ({} assets)", paths.len());
        for p in paths.iter().take(6) {
            println!("      {p}");
        }
        if paths.len() > 6 {
            println!("      … and {} more", paths.len() - 6);
        }
    }
}
