use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::io::{Read, Write};

/// Store pre-compressed bytes in Redis
pub fn compress_json(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

#[allow(dead_code)]
pub fn decompress(data: &[u8]) -> Vec<u8> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
}
