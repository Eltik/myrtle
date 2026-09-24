//! Find which bundle defines a given MonoBehaviour class, by searching the DECOMPRESSED serialized
//! files for the type-tree string. Type-tree strings carry the C# class name in `PPtr<$Name>` and in
//! the MonoBehaviour's own type entry, so a bundle that instantiates a script contains its name in
//! plain bytes once the LZ4/LZMA blocks are expanded.
//!
//! Usage: cargo run --release --example scan_typestr -- <needle> <bundle.ab>...
//! Paths may also be fed on stdin, one per line.
use rayon::prelude::*;
use std::io::BufRead;
use unpacker::unity::bundle::BundleFile;

fn main() {
    let mut args = std::env::args().skip(1);
    let needle = args.next().expect("needle");
    let mut paths: Vec<String> = args.collect();
    if paths.is_empty() {
        for l in std::io::stdin().lock().lines().map_while(Result::ok) {
            if !l.trim().is_empty() {
                paths.push(l.trim().to_string());
            }
        }
    }
    let n = needle.as_bytes();
    paths.par_iter().for_each(|p| {
        let Ok(d) = std::fs::read(p) else { return };
        let Ok(b) = BundleFile::parse(d) else { return };
        for e in &b.files {
            if e.data.windows(n.len()).any(|w| w == n) {
                println!("{p}  [{}]", e.path);
                return;
            }
        }
    });
}
