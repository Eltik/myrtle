use unpacker::unity::compression::decompress;
use unpacker::unity::endian_reader::EndianReader;

fn asset_path() -> String {
    format!(
        "{}/tests/assets/ui_skin_groups.ab",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn decompress_block_info() {
    let data = match std::fs::read(asset_path()) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("skip: {e}");
            return;
        }
    };
    let mut r = EndianReader::new(&data, true);

    let sig = r.read_cstring().unwrap();
    assert_eq!(sig, "UnityFS");
    let version = r.read_u32().unwrap();
    let _version_player = r.read_cstring().unwrap();
    let _version_engine = r.read_cstring().unwrap();
    let _file_size = r.read_i64().unwrap();
    let compressed_size = r.read_u32().unwrap() as usize;
    let uncompressed_size = r.read_u32().unwrap() as usize;
    let flags = r.read_u32().unwrap();

    if version >= 7 {
        r.align(16);
    }

    let comp_type = flags & 0x3F;
    let compressed_data = r.read_bytes(compressed_size).unwrap();
    let block_info = decompress(&compressed_data, uncompressed_size, comp_type).unwrap();

    assert_eq!(block_info.len(), uncompressed_size);

    let mut bi = EndianReader::new(&block_info, true);
    let _hash = bi.read_bytes(16).unwrap();
    let block_count = bi.read_i32().unwrap();
    assert!(block_count > 0, "should have at least one block");

    let dir_count = {
        for _ in 0..block_count {
            let _uncomp = bi.read_u32().unwrap();
            let _comp = bi.read_u32().unwrap();
            let _bflags = bi.read_u16().unwrap();
        }
        bi.read_i32().unwrap()
    };
    assert!(dir_count > 0, "should have at least one file entry");
}

#[test]
fn decompress_first_data_block() {
    let data = match std::fs::read(asset_path()) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("skip: {e}");
            return;
        }
    };
    let mut r = EndianReader::new(&data, true);

    let sig = r.read_cstring().unwrap();
    assert_eq!(sig, "UnityFS");
    let version = r.read_u32().unwrap();
    let _vp = r.read_cstring().unwrap();
    let _ve = r.read_cstring().unwrap();
    let _file_size = r.read_i64().unwrap();
    let comp_bi_size = r.read_u32().unwrap() as usize;
    let uncomp_bi_size = r.read_u32().unwrap() as usize;
    let flags = r.read_u32().unwrap();
    if version >= 7 {
        r.align(16);
    }

    let comp_type = flags & 0x3F;
    let comp_bytes = r.read_bytes(comp_bi_size).unwrap();
    let bi_data = decompress(&comp_bytes, uncomp_bi_size, comp_type).unwrap();

    let mut bi = EndianReader::new(&bi_data, true);
    bi.read_bytes(16).unwrap(); // hash
    let _block_count = bi.read_i32().unwrap();
    let uncomp = bi.read_u32().unwrap();
    let comp = bi.read_u32().unwrap();
    let bflags = bi.read_u16().unwrap();

    r.align(16);

    let block_data = r.read_bytes(comp as usize).unwrap();
    let result = decompress(&block_data, uncomp as usize, (bflags & 0x3F) as u32);
    assert!(result.is_ok(), "first data block decompression failed");
    assert_eq!(result.unwrap().len(), uncomp as usize);
}
