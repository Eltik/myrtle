//! THROWAWAY: list a bundle's SerializedFile externals in m_FileID order, so a material's
//! `m_FileID: n` can be mapped to the bundle it depends on.
use unpacker::unity::{bundle::BundleFile, serialized_file::SerializedFile};
fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else { continue };
        let Ok(bundle) = BundleFile::parse(data) else { continue };
        for e in &bundle.files {
            let (name, blob) = (&e.path, &e.data);
            let Ok(sf) = SerializedFile::parse(blob.to_vec()) else { continue };
            if sf.externals.is_empty() { continue }
            println!("{name}");
            for (i, e) in sf.externals.iter().enumerate() {
                println!("  m_FileID={} -> {}", i + 1, e.path);
            }
        }
    }
}
