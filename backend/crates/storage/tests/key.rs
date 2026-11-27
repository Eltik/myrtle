//! Object keys, public URLs and the served-roots check.

use myrtle_storage::key::{is_under_roots, normalize_rel, object_key, public_url, server_prefix};

fn roots() -> Vec<String> {
    ["textures", "spine", "audio", "video", "portraits"]
        .iter()
        .map(|r| (*r).to_owned())
        .collect()
}

#[test]
fn key_layout_matches_the_contract() {
    let rel = normalize_rel("spine/DynIllust/char_1012_skadi2/[scene].json").unwrap();
    assert_eq!(
        object_key("assets", "en", &rel),
        "assets/en/spine/DynIllust/char_1012_skadi2/[scene].json"
    );
    assert_eq!(object_key("", "cn", "textures/a.png"), "cn/textures/a.png");
}

#[test]
fn brackets_spaces_hash_and_percent_are_encoded() {
    let base = "https://assets.myrtle.moe";
    assert_eq!(
        public_url(
            base,
            "assets/en/spine/DynIllust/char_1012_skadi2/[scene].json"
        ),
        "https://assets.myrtle.moe/assets/en/spine/DynIllust/char_1012_skadi2/%5Bscene%5D.json"
    );
    assert_eq!(
        public_url(base, "en/textures/a b#c%d?.png"),
        "https://assets.myrtle.moe/en/textures/a%20b%23c%25d%3F.png"
    );
    // encodeURIComponent leaves these alone.
    assert_eq!(
        public_url(base, "en/audio/x-y_z.!~*'().ogg"),
        "https://assets.myrtle.moe/en/audio/x-y_z.!~*'().ogg"
    );
    assert_eq!(
        public_url("https://cdn.example/", "en/a+b,c.png"),
        "https://cdn.example/en/a%2Bb%2Cc.png"
    );
    assert_eq!(
        public_url(base, "en/textures/\u{30a2}.png"),
        "https://assets.myrtle.moe/en/textures/%E3%82%A2.png"
    );
}

#[test]
fn rel_normalization() {
    assert_eq!(
        normalize_rel("/textures/a.png").as_deref(),
        Some("textures/a.png")
    );
    assert_eq!(
        normalize_rel("./textures//./a.png").as_deref(),
        Some("textures/a.png")
    );
    assert_eq!(normalize_rel("textures/../gamedata/x.json"), None);
    assert_eq!(normalize_rel("../textures/a.png"), None);
    assert_eq!(normalize_rel("a\0.png"), None);
    assert_eq!(normalize_rel("/"), None);
    assert_eq!(normalize_rel(""), None);
}

#[test]
fn roots_check() {
    let roots = roots();
    assert!(is_under_roots("spine/x/[scene].json", &roots));
    assert!(is_under_roots("/textures/a.png", &roots));
    assert!(is_under_roots("./audio/a.ogg", &roots));
    assert!(!is_under_roots("gamedata/excel/a.json", &roots));
    assert!(!is_under_roots("derived/a.json", &roots));
    assert!(!is_under_roots("Textures/a.png", &roots));
    assert!(!is_under_roots("texturesx/a.png", &roots));
    assert!(!is_under_roots("textures", &roots));
    assert!(!is_under_roots("textures/../gamedata/a.json", &roots));
    assert!(!is_under_roots("", &roots));
}

#[test]
fn server_prefix_has_one_trailing_slash() {
    assert_eq!(server_prefix("assets", "en"), "assets/en/");
    assert_eq!(server_prefix("", "en"), "en/");
}
