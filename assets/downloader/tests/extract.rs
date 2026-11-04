use downloader::extract::extract_zip;
use std::fs;
use std::io::Write;

fn create_test_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        for (name, content) in entries {
            writer.start_file(name.to_string(), options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap();
    }
    buf
}

#[test]
fn extract_simple_zip() {
    let dir = tempfile::tempdir().unwrap();
    let zip_path = dir.path().join("test.zip");
    let out_dir = dir.path().join("output");
    fs::create_dir_all(&out_dir).unwrap();

    let zip_data = create_test_zip(&[("hello.txt", b"Hello, world!")]);
    fs::write(&zip_path, &zip_data).unwrap();

    extract_zip(&zip_path, &out_dir).unwrap();

    let content = fs::read_to_string(out_dir.join("hello.txt")).unwrap();
    assert_eq!(content, "Hello, world!");
}

#[test]
fn extract_nested_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let zip_path = dir.path().join("nested.zip");
    let out_dir = dir.path().join("output");
    fs::create_dir_all(&out_dir).unwrap();

    let zip_data = create_test_zip(&[
        ("top.txt", b"top level"),
        ("sub/file.txt", b"nested file"),
        ("sub/deep/leaf.txt", b"deeply nested"),
    ]);
    fs::write(&zip_path, &zip_data).unwrap();

    extract_zip(&zip_path, &out_dir).unwrap();

    assert_eq!(
        fs::read_to_string(out_dir.join("top.txt")).unwrap(),
        "top level"
    );
    assert_eq!(
        fs::read_to_string(out_dir.join("sub/file.txt")).unwrap(),
        "nested file"
    );
    assert_eq!(
        fs::read_to_string(out_dir.join("sub/deep/leaf.txt")).unwrap(),
        "deeply nested"
    );
}

#[test]
fn extract_invalid_file() {
    let dir = tempfile::tempdir().unwrap();
    let bad_path = dir.path().join("not_a_zip.dat");
    let out_dir = dir.path().join("output");
    fs::create_dir_all(&out_dir).unwrap();

    fs::write(&bad_path, b"this is not a zip file").unwrap();

    let result = extract_zip(&bad_path, &out_dir);
    assert!(result.is_err());
}

/// RUSTSEC-2026-0368: zip 8 let `extract` create a symlink whose target is
/// outside the destination, so a later entry written through it lands outside
/// too. Whatever zip does with such an archive (refuse it, or confine the
/// link), nothing may appear outside `out_dir`.
#[cfg(unix)]
#[test]
fn extract_keeps_symlink_entries_inside_the_destination() {
    let dir = tempfile::tempdir().unwrap();
    let zip_path = dir.path().join("evil.zip");
    let out_dir = dir.path().join("output");
    let outside = dir.path().join("outside");
    fs::create_dir_all(&out_dir).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let mut buf = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer
            .add_symlink("link", outside.to_str().unwrap(), options)
            .unwrap();
        writer.start_file("link/pwned.txt", options).unwrap();
        writer.write_all(b"escaped").unwrap();
        writer.finish().unwrap();
    }
    fs::write(&zip_path, &buf).unwrap();

    let result = extract_zip(&zip_path, &out_dir);

    assert!(
        !outside.join("pwned.txt").exists(),
        "an entry was written outside the destination through a symlink"
    );
    let link = out_dir.join("link");
    if let Ok(meta) = fs::symlink_metadata(&link)
        && meta.file_type().is_symlink()
    {
        let target = fs::canonicalize(&link).unwrap();
        assert!(
            target.starts_with(fs::canonicalize(&out_dir).unwrap()),
            "extract left a symlink pointing outside the destination: {} -> {} ({result:?})",
            link.display(),
            target.display()
        );
    }
}
