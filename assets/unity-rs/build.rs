use std::env;
use std::path::PathBuf;

fn main() {
    // Get the manifest directory (project root)
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Determine the FMOD library path based on platform
    let fmod_lib_dir = manifest_dir.join("src/resources/FMOD");

    #[cfg(target_os = "macos")]
    let platform_dir = fmod_lib_dir.join("Darwin");

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let platform_dir = fmod_lib_dir.join("Linux/x86_64");

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let platform_dir = fmod_lib_dir.join("Linux/arm64");

    #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
    let platform_dir = fmod_lib_dir.join("Windows/x64");

    #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
    let platform_dir = fmod_lib_dir.join("Windows/x86");

    // Tell cargo to look for libraries in the platform-specific directory
    println!("cargo:rustc-link-search=native={}", platform_dir.display());

    // Add runtime library path (rpath) so executables can find FMOD library
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", platform_dir.display());

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", platform_dir.display());

    // Set runtime library path (for reference/debugging)
    println!("cargo:rustc-env=FMOD_LIB_PATH={}", platform_dir.display());

    // Re-run build script if FMOD files change
    println!("cargo:rerun-if-changed=src/resources/FMOD");
}
