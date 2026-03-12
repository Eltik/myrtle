use unpacker::export::fsb5::{
    VORBIS_HEADERS, extract_block_flags, fsb5_to_ogg, lookup_vorbis_setup,
};

#[test]
fn vorbis_headers_loaded() {
    assert!(
        !VORBIS_HEADERS.is_empty(),
        "Vorbis headers table should not be empty"
    );
    assert!(
        lookup_vorbis_setup(3072374402).is_some(),
        "Should find setup_id from CN_004"
    );
}

#[test]
fn test_extract_block_flags() {
    let setup = lookup_vorbis_setup(3072374402).unwrap();
    let flags = extract_block_flags(setup.header_bytes, setup.seek_bit);
    assert!(!flags.is_empty(), "Should extract at least one block flag");
}

#[test]
fn fsb5_to_ogg_decode() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_output/audio/CN_004.bytes"
    );
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("skip: {e}");
            return;
        }
    };

    let results = fsb5_to_ogg(&data).unwrap();
    assert!(!results.is_empty(), "Should extract at least one sample");

    for (name, ogg) in &results {
        assert_eq!(&ogg[0..4], b"OggS", "Sample {name} should start with OggS");
    }
}
