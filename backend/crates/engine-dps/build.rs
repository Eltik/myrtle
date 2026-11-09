#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::io;
use std::path::{Path, PathBuf};

const INPUTS: &[&str] = &["src", "../dps-formulas/src", "../dps-formulas/data"];

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

fn collect(path: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            collect(&entry?.path(), out)?;
        }
    } else {
        out.push(path.to_path_buf());
    }
    Ok(())
}

fn hash_inputs(root: &Path) -> io::Result<String> {
    let mut files = Vec::new();
    for input in INPUTS {
        collect(&root.join(input), &mut files)?;
    }
    files.sort();
    let mut h = FNV_OFFSET;
    for file in &files {
        let rel = file.strip_prefix(root).unwrap_or(file);
        h = fnv(h, rel.to_string_lossy().replace('\\', "/").as_bytes());
        h = fnv(h, &[0]);
        h = fnv(h, &std::fs::read(file)?);
        h = fnv(h, &[0xff]);
    }
    Ok(format!("{h:016x}"))
}

fn main() {
    let root =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"));
    for input in INPUTS {
        println!("cargo:rerun-if-changed={input}");
    }
    let version = hash_inputs(&root).expect("the engine sources are readable");
    println!("cargo:rustc-env=DPS_ENGINE_VERSION={version}");
}
