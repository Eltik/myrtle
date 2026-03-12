use downloader::download::{Downloader, replace_last_ext};
use downloader::server::Server;

// ─── replace_last_ext ───────────────────────────────────────────────────────

#[test]
fn replace_ext_normal() {
    assert_eq!(replace_last_ext("foo.zip", "dat"), "foo.dat");
}

#[test]
fn replace_ext_no_extension() {
    assert_eq!(replace_last_ext("noext", "dat"), "noext.dat");
}

#[test]
fn replace_ext_multiple_dots() {
    assert_eq!(replace_last_ext("a.b.zip", "dat"), "a.b.dat");
}

#[test]
fn replace_ext_path_with_dots() {
    assert_eq!(
        replace_last_ext("path/to/file.ab.zip", "dat"),
        "path/to/file.ab.dat"
    );
}

// ─── build_url ──────────────────────────────────────────────────────────────

#[test]
fn build_url_replaces_slashes() {
    let dl = Downloader::new(Server::En, "v1".into(), 1);
    let url = dl.build_url("a/b/c.zip");
    // slashes become underscores
    assert!(url.contains("a_b_c.dat"), "url was: {url}");
}

#[test]
fn build_url_replaces_hash() {
    let dl = Downloader::new(Server::En, "v1".into(), 1);
    let url = dl.build_url("file#1.ab");
    // '#' becomes '__'
    assert!(url.contains("file__1.dat"), "url was: {url}");
}

#[test]
fn build_url_replaces_ext() {
    let dl = Downloader::new(Server::En, "v1".into(), 1);
    let url = dl.build_url("bundle.ab");
    assert!(url.ends_with("bundle.dat"), "url was: {url}");
}

#[test]
fn build_url_format() {
    let dl = Downloader::new(Server::En, "24-12-01-abc".into(), 1);
    let url = dl.build_url("arts/chararts/char.ab");

    let expected_base = Server::En.cdn_base_url();
    assert!(url.starts_with(expected_base), "url was: {url}");
    assert!(url.contains("24-12-01-abc"), "url was: {url}");
    assert!(url.ends_with("arts_chararts_char.dat"), "url was: {url}");
}
