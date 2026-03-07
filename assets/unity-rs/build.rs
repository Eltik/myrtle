use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let fmod_lib_dir = manifest_dir.join("src/resources/FMOD");

    println!("cargo:rustc-link-search=native={}", fmod_lib_dir.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", fmod_lib_dir.display());
    println!("cargo:rustc-env=FMOD_LIB_PATH={}", fmod_lib_dir.display());

    println!("cargo:rerun-if-changed=src/resources/FMOD");
}
