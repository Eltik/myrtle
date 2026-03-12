use unpacker::unity::endian_reader::EndianReader;

#[test]
fn read_u32_big_endian() {
    let data = vec![0x00, 0x00, 0x00, 0x2A]; // 42 in BE
    let mut r = EndianReader::new(&data, true);
    assert_eq!(r.read_u32().unwrap(), 42);
    assert_eq!(r.position(), 4);
}

#[test]
fn read_u32_little_endian() {
    let data = vec![0x2A, 0x00, 0x00, 0x00]; // 42 in LE
    let mut r = EndianReader::new(&data, false);
    assert_eq!(r.read_u32().unwrap(), 42);
}

#[test]
fn read_cstring() {
    let data = vec![b'U', b'n', b'i', b't', b'y', 0x00, 0xFF];
    let mut r = EndianReader::new(&data, true);
    assert_eq!(r.read_cstring().unwrap(), "Unity");
    assert_eq!(r.position(), 6);
}

#[test]
fn align() {
    let data = vec![0; 16];
    let mut r = EndianReader::new(&data, true);
    r.set_position(5);
    r.align(4);
    assert_eq!(r.position(), 8);
}

#[test]
fn read_past_end() {
    let data = vec![0x01];
    let mut r = EndianReader::new(&data, true);
    assert!(r.read_u32().is_err());
}

#[test]
fn read_string_negative_length_no_panic() {
    // i32 = -1 in little-endian = 0xFFFFFFFF, wraps to usize::MAX
    let data = vec![0xFF, 0xFF, 0xFF, 0xFF];
    let mut r = EndianReader::new(&data, false);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| r.read_string()));
    assert!(
        result.is_ok(),
        "read_string should not panic on negative length"
    );
    assert!(
        result.unwrap().is_err(),
        "read_string should return Err for negative length"
    );
}
