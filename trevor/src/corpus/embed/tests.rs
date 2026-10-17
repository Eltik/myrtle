use super::*;
use super::checkpoint::parse_checkpoint;

fn record(key: &str, v: &[f32]) -> Vec<u8> {
    let mut b = key.as_bytes().to_vec();
    b.extend_from_slice(&u32::try_from(v.len()).unwrap().to_le_bytes());
    for x in v {
        b.extend_from_slice(&x.to_le_bytes());
    }
    b
}

#[test]
fn checkpoint_keeps_the_valid_prefix() {
    let a = record("0123456789abcdef", &[0.6, 0.8]);
    let b = record("fedcba9876543210", &[1.0, 0.0]);
    let mut bytes = [a.clone(), b.clone()].concat();
    let (rows, valid, dim) = parse_checkpoint(&bytes);
    assert_eq!((rows.len(), valid, dim), (2, bytes.len(), Some(2)));
    assert_eq!(rows["fedcba9876543210"], b[20..]);
    // A torn last record is cut, not read.
    bytes.extend_from_slice(&record("00000000000000aa", &[0.0, 1.0])[..23]);
    let (rows, valid, _) = parse_checkpoint(&bytes);
    assert_eq!((rows.len(), valid), (2, a.len() + b.len()));
    // Zeros (a crash after the size grew, before the data) end it too.
    let z = [a.clone(), vec![0u8; 28]].concat();
    assert_eq!(parse_checkpoint(&z).1, a.len());
    // So do a non-unit row, a non-hex key and a changed dim.
    assert_eq!(parse_checkpoint(&[a.clone(), record("1111111111111111", &[0.5, 0.5])].concat()).1, a.len());
    assert_eq!(parse_checkpoint(&[a.clone(), record("XYZ1111111111111", &[1.0, 0.0])].concat()).1, a.len());
    assert_eq!(parse_checkpoint(&[a.clone(), record("1111111111111111", &[1.0, 0.0, 0.0])].concat()).1, a.len());
}

#[test]
fn query_onnx_auto_is_the_vectors_file() {
    assert_eq!(resolve_query_onnx("auto", "model.onnx"), "model.onnx");
    assert_eq!(resolve_query_onnx("model_int8.onnx", "model.onnx"), "model_int8.onnx");
}
