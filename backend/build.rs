//! Emits `DPS_ENGINE_VERSION`, the hash of the DPS engine's sources that every
//! DPS cache key carries (`src/dps/engine_version.rs` says why).

#[path = "src/dps/engine_version.rs"]
mod engine_version;

fn main() {
    let root = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"),
    );
    // A directory is scanned whole, so a new engine file reruns this too.
    for input in engine_version::INPUTS {
        println!("cargo:rerun-if-changed={input}");
    }
    let version = engine_version::hash_inputs(&root).expect("the engine sources are readable");
    println!("cargo:rustc-env=DPS_ENGINE_VERSION={version}");
}
