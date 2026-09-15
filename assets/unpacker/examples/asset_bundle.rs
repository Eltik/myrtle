//! Which bundle carries an asset: `asset_bundle <manifest.idx> <path substring>`.
//! Prints every manifest asset whose path contains the substring with the
//! bundle that packs it, e.g. `shopskinportraits/char_4217` for a store
//! portrait.

use std::path::Path;

use unpacker::export::resource_manifest_generated::root_as_clz_torappu_resource_resource_manifest_unchecked;

const MANIFEST_HEADER: usize = 128;

fn main() {
    let idx = std::env::args().nth(1).expect("manifest .idx path");
    let needle = std::env::args().nth(2).expect("asset path substring");
    let data = std::fs::read(Path::new(&idx)).expect("read manifest");
    let manifest = unsafe {
        root_as_clz_torappu_resource_resource_manifest_unchecked(&data[MANIFEST_HEADER..])
    };
    let bundles = manifest.bundles().expect("bundles");
    let Some(assets) = manifest.assetToBundleList() else {
        return;
    };
    for i in 0..assets.len() {
        let asset = assets.get(i);
        let path = asset.path().unwrap_or("");
        if path.contains(&needle) {
            let bundle = bundles.get(asset.bundleIndex() as usize);
            println!("{path} -> {}", bundle.name().unwrap_or("?"));
        }
    }
}
