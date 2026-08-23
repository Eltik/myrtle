//! THROWAWAY: does a shader object's RAW bytes contain a given string anywhere?
//! Guards against a property-block parser false-negative.
//! Usage: probe_rawstr <shaders.ab> <shader-name-substr> <needle> [needle...]
use unpacker::unity::{bundle::BundleFile, serialized_file::SerializedFile};

fn scan_name(seg: &[u8]) -> Option<String> {
    let ok = |b: u8| b.is_ascii_graphic() || b == b' ';
    let mut i = 0;
    while i < seg.len() {
        if seg[i].is_ascii_uppercase() {
            let s = &seg[i..];
            let e = s.iter().position(|&b| !ok(b)).unwrap_or(s.len());
            let run = &s[..e];
            if run.len() >= 10
                && run.contains(&b'/')
                && (run.starts_with(b"Torappu")
                    || run.starts_with(b"Hidden")
                    || run.starts_with(b"Custom"))
            {
                return Some(String::from_utf8_lossy(run).to_string());
            }
            i += e.max(1);
        } else {
            i += 1
        }
    }
    None
}

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().unwrap();
    let filt = a.next().unwrap();
    let needles: Vec<String> = a.collect();
    let data = std::fs::read(&path).unwrap();
    let bundle = BundleFile::parse(data).unwrap();
    for entry in &bundle.files {
        let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            if obj.class_id != 48 {
                continue;
            }
            let st = obj.byte_start as usize;
            let en = (st + obj.byte_size as usize).min(sf.data.len());
            if st >= en {
                continue;
            }
            let seg = &sf.data[st..en];
            let Some(name) = scan_name(seg) else { continue };
            if !name.contains(&filt) {
                continue;
            }
            print!("{name}  ({} bytes)", seg.len());
            for n in &needles {
                let c = seg.windows(n.len()).filter(|w| *w == n.as_bytes()).count();
                print!("   {n}={c}");
            }
            println!();
        }
    }
}
