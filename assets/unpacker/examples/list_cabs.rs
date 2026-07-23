//! list CAB entries of bundles
use unpacker::unity::bundle::BundleFile;
fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Ok(b) = BundleFile::parse(data) else {
            continue;
        };
        for e in &b.files {
            println!("{}\t{}", path, e.path);
        }
    }
}
