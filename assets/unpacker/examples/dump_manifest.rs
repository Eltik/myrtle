//! Dump `assetToBundleList` rows of a `.idx` resource manifest whose asset `path`
//! or `name` contains a needle, with the bundle each row points at.
//!
//! Usage: cargo run --release --example dump_manifest -- <manifest.idx> <needle>...
use unpacker::export::resource_manifest_generated::root_as_clz_torappu_resource_resource_manifest_unchecked;

fn main() {
    let mut args = std::env::args().skip(1);
    let idx = args.next().expect("idx path");
    let needles: Vec<String> = args.collect();
    let data = std::fs::read(&idx).expect("read idx");
    let m = unsafe { root_as_clz_torappu_resource_resource_manifest_unchecked(&data[128..]) };
    let bundles = m.bundles();
    let assets = m.assetToBundleList().expect("assets");
    let mut hits = 0usize;
    for i in 0..assets.len() {
        let a = assets.get(i);
        let name = a.name().unwrap_or("");
        let path = a.path().unwrap_or("");
        let asset_name = a.assetName().unwrap_or("");
        if !needles.iter().any(|n| {
            path.contains(n.as_str())
                || name.contains(n.as_str())
                || asset_name.contains(n.as_str())
        }) {
            continue;
        }
        let bi = a.bundleIndex();
        let bname = bundles
            .as_ref()
            .and_then(|b| {
                if bi >= 0 && (bi as usize) < b.len() {
                    b.get(bi as usize)
                        .name()
                        .map(std::string::ToString::to_string)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        println!("row {i}\tname={name}\tassetName={asset_name}\tpath={path}\tbundle[{bi}]={bname}");
        hits += 1;
    }
    eprintln!("{hits} rows over {} assets", assets.len());
}
