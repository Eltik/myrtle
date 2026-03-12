use std::collections::HashMap;
use std::path::PathBuf;

use unpacker::export::texture::{decode_texture, export_texture};
use unpacker::unity::bundle::BundleFile;
use unpacker::unity::object_reader::read_object;
use unpacker::unity::serialized_file::SerializedFile;

#[test]
fn test_export_texture() {
    let path = format!(
        "{}/tests/assets/chararts_char_003_kalts.ab",
        env!("CARGO_MANIFEST_DIR")
    );
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("skip: {e}");
            return;
        }
    };
    let bundle = BundleFile::parse(data).unwrap();

    let mut resources = HashMap::new();
    for entry in &bundle.files {
        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
            let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
            resources.insert(filename.to_string(), entry.data.clone());
        }
    }

    let sf = SerializedFile::parse(bundle.files[0].data.clone()).unwrap();
    let output_dir = PathBuf::from(format!(
        "{}/test_output/textures",
        env!("CARGO_MANIFEST_DIR")
    ));
    std::fs::create_dir_all(&output_dir).unwrap();

    let mut exported = 0;
    for obj in &sf.objects {
        if obj.class_id != 28 {
            continue;
        }
        let val = read_object(&sf, obj).unwrap();
        if export_texture(&val, &output_dir, &resources).is_ok() {
            exported += 1;
        }
    }
    assert!(exported > 0, "should export at least one texture");
}

#[test]
fn decode_alpha8() {
    let data = vec![0u8, 128, 255];
    let mut buf = vec![0u32; 3];
    decode_texture(&data, 3, 1, 1, &mut buf).expect("Alpha8 (format 1) should be supported");
    assert_eq!(buf[0] & 0xFF000000, 0x00000000, "first pixel alpha=0");
    assert_eq!(buf[1] >> 24, 128, "second pixel alpha=128");
    assert_eq!(buf[2] >> 24, 255, "third pixel alpha=255");
}

#[test]
fn decode_rgb565() {
    // Pure red in RGB565: 11111_000000_00000 = 0xF800
    let data = vec![0x00, 0xF8];
    let mut buf = vec![0u32; 1];
    decode_texture(&data, 1, 1, 7, &mut buf).expect("RGB565 (format 7) should be supported");
    let r = (buf[0] >> 16) & 0xFF;
    assert!(r > 240, "red channel should be near 255, got {r}");
}

#[test]
fn decode_astc_7x7_accepted() {
    let data = vec![0u8; 16];
    let mut buf = vec![0u32; 7 * 7];
    let result = decode_texture(&data, 7, 7, 51, &mut buf);
    assert!(
        !result
            .as_ref()
            .is_err_and(|e| e.to_string().contains("unsupported")),
        "ASTC 7x7 (format 51) should be supported, got: {result:?}"
    );
}

#[test]
fn export_texture_stream_data_out_of_bounds() {
    use serde_json::json;

    let mut resources = HashMap::new();
    resources.insert("test.resS".to_string(), vec![0u8; 100]);

    let obj = json!({
        "m_Name": "test_oob",
        "m_Width": 4,
        "m_Height": 4,
        "m_TextureFormat": 4,
        "image data": "",
        "m_StreamData": {
            "offset": 50,
            "size": 200,
            "path": "archive:/CAB-abc/test.resS"
        }
    });

    let dir = PathBuf::from(format!(
        "{}/test_output/texture_oob",
        env!("CARGO_MANIFEST_DIR")
    ));
    std::fs::create_dir_all(&dir).unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        export_texture(&obj, &dir, &resources)
    }));
    assert!(
        result.is_ok(),
        "export_texture should not panic on out-of-bounds stream data"
    );
    assert!(
        result.unwrap().is_ok(),
        "out-of-bounds stream data should be silently skipped"
    );
}

#[test]
fn export_texture_insufficient_data_skips_silently() {
    use serde_json::json;

    let mut resources = HashMap::new();
    resources.insert("small.resS".to_string(), vec![0u8; 16]);

    let obj = json!({
        "m_Name": "test_small_astc",
        "m_Width": 1024,
        "m_Height": 1024,
        "m_TextureFormat": 48,
        "image data": "",
        "m_StreamData": {
            "offset": 0,
            "size": 16,
            "path": "small.resS"
        }
    });

    let dir = PathBuf::from(format!(
        "{}/test_output/texture_small",
        env!("CARGO_MANIFEST_DIR")
    ));
    std::fs::create_dir_all(&dir).unwrap();

    let result = export_texture(&obj, &dir, &resources);
    assert!(
        result.is_ok(),
        "insufficient texture data should be silently skipped, got: {result:?}"
    );
    assert!(
        !dir.join("test_small_astc.png").exists(),
        "should not write a PNG for insufficient data"
    );
}

#[test]
fn decode_rgba32_overflow_no_panic() {
    // buf has 4 pixels, but data has 5 RGBA pixels (20 bytes)
    let data = vec![255u8; 20];
    let mut buf = vec![0u32; 4];
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        decode_texture(&data, 2, 2, 4, &mut buf)
    }));
    assert!(
        result.is_ok(),
        "decode_texture should not panic when data has more pixels than w*h"
    );
}
