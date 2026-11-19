#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::disallowed_methods
)]

use myrtle_engine_story::assets::{
    BodySize, FacePos, ImageSource, PlateRect, StoryAssetIndex, parse_sprite_name, served_rel,
    trim_index,
};
use myrtle_gamedata_raw::DirSource;
use tempfile::TempDir;

fn png(w: u32, h: u32) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n".to_vec();
    b.extend_from_slice(&13u32.to_be_bytes());
    b.extend_from_slice(b"IHDR");
    b.extend_from_slice(&w.to_be_bytes());
    b.extend_from_slice(&h.to_be_bytes());
    b
}

/// `paths` as the listing gives them: each tree's files relative to it, in the order given
fn listing(paths: &[&str]) -> impl Fn(&str) -> Vec<String> {
    let paths: Vec<String> = paths.iter().map(|p| (*p).to_owned()).collect();
    move |tree: &str| {
        let prefix = format!("{tree}/");
        paths
            .iter()
            .filter_map(|p| p.strip_prefix(&prefix).map(str::to_owned))
            .collect()
    }
}

/// `files` written for the reads, listed in path order
fn index(files: &[(String, Vec<u8>)]) -> (TempDir, StoryAssetIndex) {
    let dir = tempfile::tempdir().unwrap();
    for (rel, bytes) in files {
        let path = dir.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    let src = DirSource {
        root: dir.path().to_path_buf(),
    };
    let mut paths: Vec<&str> = files.iter().map(|(rel, _)| rel.as_str()).collect();
    paths.sort_unstable();
    let idx = StoryAssetIndex::build(&src, &src, &listing(&paths)).unwrap();
    (dir, idx)
}

/// Sprite folders of empty PNGs, each with an optional `hub.json`
fn sprites(folders: &[(&str, &[&str], Option<&str>)]) -> (TempDir, StoryAssetIndex) {
    let mut files = Vec::new();
    for (name, pngs, hub) in folders {
        for f in *pngs {
            files.push((format!("textures/avg/characters/{name}/{f}"), Vec::new()));
        }
        if let Some(hub) = hub {
            files.push((
                format!("textures/avg/characters/{name}/hub.json"),
                hub.as_bytes().to_vec(),
            ));
        }
    }
    index(&files)
}

const HAAK_HUB: &str = r#"{
  "groups": [{
    "facePos": {"x": 512.0, "y": 120.0},
    "faceSize": {"w": 51.0, "h": 71.0},
    "sprites": [
      {"name": "1$1", "alias": "", "isWholeBody": false},
      {"name": "3$1", "alias": "wry", "isWholeBody": false},
      {"name": "avg_225_haak_1$1", "alias": "", "isWholeBody": false}
    ]
  }],
  "legacy": false
}"#;

const AMIYA_HUB: &str = r#"{
  "groups": [{
    "facePos": {"x": 570.0, "y": 233.0},
    "faceSize": {"w": 117.0, "h": 95.0},
    "sprites": [
      {"name": "1$1", "alias": "", "isWholeBody": false, "size": {"w": 128.0, "h": 128.0}},
      {"name": "avg_1037_amiya3_1$1", "alias": "", "isWholeBody": true, "size": {"w": 1280.0, "h": 1280.0}}
    ]
  }],
  "legacy": false
}"#;

const HAAK: &[&str] = &["avg_225_haak_1$1.png", "1$1.png", "3$1.png"];

#[test]
fn sprite_name_parts() {
    let p = parse_sprite_name("avg_225_haak_1#3$2");
    assert_eq!(
        (p.base, p.face, p.body, p.alias),
        ("avg_225_haak_1", Some("3"), Some("2"), None)
    );
    let p = parse_sprite_name("char_002_amiya_1#5");
    assert_eq!(
        (p.base, p.face, p.body),
        ("char_002_amiya_1", Some("5"), None)
    );
    let p = parse_sprite_name("avg_npc_001");
    assert_eq!((p.base, p.face, p.body), ("avg_npc_001", None, None));
    let p = parse_sprite_name("char_102_texas_1#1 ");
    assert_eq!(
        (p.base, p.face, p.body),
        ("char_102_texas_1", Some("1"), None)
    );
    let p = parse_sprite_name("avg_1014_nearl2_1$1");
    assert_eq!(
        (p.base, p.face, p.body),
        ("avg_1014_nearl2_1", None, Some("1"))
    );
    let p = parse_sprite_name("char_002_amiya_1@open mouth");
    assert_eq!(
        (p.base, p.face, p.alias),
        ("char_002_amiya_1", None, Some("open mouth"))
    );
    let p = parse_sprite_name("avg_1037_amiya3_1@smile$2");
    assert_eq!(
        (p.base, p.alias, p.body),
        ("avg_1037_amiya3_1", Some("smile"), Some("2"))
    );
    assert_eq!(trim_index("00"), "0");
    assert_eq!(trim_index("10"), "10");
    assert_eq!(trim_index("01"), "1");
}

#[test]
fn stems_fold_doubled_extensions_and_urls_encode_hash() {
    let (_d, idx) = sprites(&[("avg_x_1", &["Avg_X_1$1.png.png", "1$1..png"], None)]);
    let s = idx.resolve_character("avg_x_1#1$1").unwrap();
    assert_eq!(
        s.body_url,
        "/textures/avg/characters/avg_x_1/Avg_X_1$1.png.png"
    );
    assert_eq!(
        s.face_url.as_deref(),
        Some("/textures/avg/characters/avg_x_1/1$1..png")
    );
    assert_eq!(served_rel("/a/b%231.png"), "a/b#1.png");
}

#[test]
fn the_body_texture_size_rides_beside_the_placement() {
    let (_d, idx) = sprites(&[(
        "avg_1037_amiya3_1",
        &["avg_1037_amiya3_1$1.png", "1$1.png"],
        Some(AMIYA_HUB),
    )]);
    let s = idx.resolve_character("avg_1037_amiya3_1#1$1").unwrap();
    let size = s.body_size.expect("bodySize");
    assert_eq!((size.w, size.h), (1280.0, 1280.0));
    let pos = s.face_pos.expect("facePos");
    assert!((pos.x / size.w - 0.445_312_5).abs() < 1e-6, "{pos:?}");
    assert!((pos.w / size.w - 0.091_406_25).abs() < 1e-6, "{pos:?}");
}

#[test]
fn a_hub_with_no_size_falls_back_to_the_png_header() {
    let hub = AMIYA_HUB
        .replace(", \"size\": {\"w\": 128.0, \"h\": 128.0}", "")
        .replace(", \"size\": {\"w\": 1280.0, \"h\": 1280.0}", "");
    let dir = "textures/avg/characters/avg_1037_amiya3_1";
    let (_d, idx) = index(&[
        (format!("{dir}/avg_1037_amiya3_1$1.png"), png(1280, 1280)),
        (format!("{dir}/1$1.png"), png(1280, 1280)),
        (format!("{dir}/hub.json"), hub.into_bytes()),
    ]);
    let s = idx.resolve_character("avg_1037_amiya3_1#1$1").unwrap();
    assert_eq!(s.body_size.map(|b| (b.w, b.h)), Some((1280.0, 1280.0)));
}

#[test]
fn the_prefab_plate_rides_on_every_sprite_of_the_folder() {
    let hub = HAAK_HUB.replace(
        "\"legacy\": false",
        "\"legacy\": false, \"root\": {\"x\": 0.0, \"y\": 150.0, \"w\": 955.0, \"h\": 955.0}",
    );
    let (_d, idx) = sprites(&[("avg_225_haak_1", HAAK, Some(&hub))]);
    let want = Some(PlateRect {
        x: 0.0,
        y: 150.0,
        w: 955.0,
        h: 955.0,
    });
    assert_eq!(
        idx.resolve_character("avg_225_haak_1#1$1").unwrap().plate,
        want
    );
    assert_eq!(
        idx.resolve_character("avg_225_haak_1#3$1").unwrap().plate,
        want
    );
    let (_d, plain) = sprites(&[("avg_225_haak_1", &["avg_225_haak_1$1.png"], Some(HAAK_HUB))]);
    assert_eq!(
        plain.resolve_character("avg_225_haak_1").unwrap().plate,
        None
    );
}

#[test]
fn image_sizes_read_the_sprites_json_and_fall_back_to_the_png_header() {
    let dir = "textures/avg/bg/avg_bkg_h1_bg_ch_0";
    let (_d, idx) = index(&[
        (format!("{dir}/bg_cher_1.png"), png(1024, 576)),
        (format!("{dir}/bg_nosprite.png"), png(512, 288)),
        (
            format!("{dir}/sprites.json"),
            br#"{"bg_cher_1": {"w": 1024.0, "h": 576.0, "ppu": 68.2464, "pivot": {"x": 0.5, "y": 0.5}}}"#
                .to_vec(),
        ),
    ]);
    let cher = idx.resolve_background_size("bg_cher_1").unwrap();
    assert_eq!((cher.w, cher.h), (1024.0, 576.0));
    assert!((cher.ppu - 68.2464).abs() < 1e-4, "{cher:?}");
    assert_eq!(cher.ppu_assumed, None);
    assert!((cher.w * 100.0 / cher.ppu - 1500.4).abs() < 0.1);
    let other = idx.resolve_background_size("bg_nosprite").unwrap();
    assert_eq!((other.w, other.h, other.ppu), (512.0, 288.0, 100.0));
    assert_eq!(other.ppu_assumed, Some(true));
    assert_eq!(idx.resolve_background_size("bg_missing"), None);
}

#[test]
fn no_hub_size_and_no_png_leaves_body_size_absent() {
    let (_d, idx) = sprites(&[("avg_225_haak_1", HAAK, Some(HAAK_HUB))]);
    let s = idx.resolve_character("avg_225_haak_1#3$1").unwrap();
    assert_eq!(s.body_size, None);
    assert!(s.face_pos.is_some());
}

#[test]
fn hub_face_pos_rides_the_face_patch() {
    let (_d, idx) = sprites(&[("avg_225_haak_1", HAAK, Some(HAAK_HUB))]);
    let s = idx.resolve_character("avg_225_haak_1#3$1").unwrap();
    assert_eq!(
        s.face_pos,
        Some(FacePos {
            x: 512.0,
            y: 120.0,
            w: 51.0,
            h: 71.0
        })
    );
    let s = idx.resolve_character("avg_225_haak_1").unwrap();
    assert!(s.face_url.as_deref().unwrap().ends_with("1$1.png"));
    assert!(s.face_pos.is_some());
    let s = idx.resolve_character("avg_225_haak_1@wry$1").unwrap();
    assert!(s.face_url.as_deref().unwrap().ends_with("3$1.png"));
    assert!(s.face_pos.is_some());
    let s = idx.resolve_character("avg_225_haak_1#9$1").unwrap();
    assert_eq!(s.face_url, None);
    assert_eq!(s.face_pos, None);
}

#[test]
fn a_sentinel_hub_and_a_whole_body_sprite_carry_no_face_pos() {
    let sentinel = r#"{"groups": [{"facePos": {"x": -1.0, "y": -1.0}, "faceSize": {"w": 0.0, "h": 0.0},
        "sprites": [{"name": "char_002_amiya_1", "alias": "normal", "isWholeBody": false},
                    {"name": "char_002_amiya_5", "alias": "angry", "isWholeBody": false}]}], "legacy": true}"#;
    let (_d, idx) = sprites(&[(
        "char_002_amiya_1",
        &["char_002_amiya_1.png", "char_002_amiya_5.png"],
        Some(sentinel),
    )]);
    let s = idx.resolve_character("char_002_amiya_1#5").unwrap();
    assert!(s.body_url.ends_with("char_002_amiya_5.png"));
    assert_eq!(s.face_pos, None);

    let whole = r#"{"groups": [{"facePos": {"x": 100.0, "y": 50.0}, "faceSize": {"w": 60.0, "h": 40.0},
        "sprites": [{"name": "1", "alias": "", "isWholeBody": true},
                    {"name": "avg_x_1", "alias": "", "isWholeBody": false}]}], "legacy": true}"#;
    let (_d, idx) = sprites(&[("avg_x_1", &["avg_x_1.png", "1.png"], Some(whole))]);
    let s = idx.resolve_character("avg_x_1#1").unwrap();
    assert!(s.body_url.ends_with("/1.png"), "{}", s.body_url);
    assert_eq!(s.face_url, None);
    assert_eq!(s.face_pos, None);
}

#[test]
fn a_whole_body_face_replaces_the_hub_body() {
    let hub = r#"{"groups": [{"facePos": {"x": 0.0, "y": 0.0}, "faceSize": {"w": 0.0, "h": 0.0},
        "sprites": [
          {"name": "1$1", "alias": "", "isWholeBody": true, "size": {"w": 1124.0, "h": 1124.0}},
          {"name": "5$1", "alias": "", "isWholeBody": true, "size": {"w": 1124.0, "h": 1124.0}},
          {"name": "avg_npc_1730_1$1", "alias": "", "isWholeBody": false, "size": {"w": 1124.0, "h": 1124.0}}
        ]}], "legacy": false}"#;
    let (_d, idx) = sprites(&[(
        "avg_npc_1730_1",
        &["avg_npc_1730_1$1.png", "1$1.png", "5$1.png"],
        Some(hub),
    )]);
    let s = idx.resolve_character("avg_npc_1730_1#5$1").unwrap();
    assert!(s.body_url.ends_with("/5$1.png"), "{}", s.body_url);
    assert_eq!(s.face_url, None);
    assert_eq!(s.face_pos, None);
    assert_eq!(
        s.body_size,
        Some(BodySize {
            w: 1124.0,
            h: 1124.0
        })
    );
    let s = idx.resolve_character("avg_npc_1730_1#1$1").unwrap();
    assert!(s.body_url.ends_with("/1$1.png"), "{}", s.body_url);
}

#[test]
fn the_body_index_picks_the_group() {
    let two = r#"{"groups": [
        {"facePos": {"x": 570.0, "y": 233.0}, "faceSize": {"w": 117.0, "h": 95.0},
         "sprites": [{"name": "1$1"}, {"name": "avg_a_1$1"}]},
        {"facePos": {"x": 300.0, "y": 111.0}, "faceSize": {"w": 80.0, "h": 60.0},
         "sprites": [{"name": "1$2"}, {"name": "avg_a_1$2"}]}
      ], "legacy": false}"#;
    let (_d, idx) = sprites(&[(
        "avg_a_1",
        &["avg_a_1$1.png", "avg_a_1$2.png", "1$1.png", "1$2.png"],
        Some(two),
    )]);
    let a = idx.resolve_character("avg_a_1#1$1").unwrap();
    let b = idx.resolve_character("avg_a_1#1$2").unwrap();
    assert_eq!(a.face_pos.unwrap().x, 570.0);
    assert_eq!(b.face_pos.unwrap().x, 300.0);
    assert_eq!(b.face_pos.unwrap().h, 60.0);
}

#[test]
fn a_folder_with_no_hub_resolves_by_file_name() {
    let (_d, idx) = sprites(&[(
        "avg_225_haak_1",
        &[
            "avg_225_haak_1$1.png",
            "1$1.png",
            "3$1.png",
            "avg_225_haak_1$1[alpha].png",
        ],
        None,
    )]);
    let s = idx.resolve_character("avg_225_haak_1#3$1").unwrap();
    assert_eq!(
        s.body_url,
        "/textures/avg/characters/avg_225_haak_1/avg_225_haak_1$1.png"
    );
    assert_eq!(
        s.face_url.as_deref(),
        Some("/textures/avg/characters/avg_225_haak_1/3$1.png")
    );
    assert_eq!(s.face_pos, None);
    let s = idx.resolve_character("avg_225_haak_1").unwrap();
    assert_eq!(
        s.face_url.as_deref(),
        Some("/textures/avg/characters/avg_225_haak_1/1$1.png")
    );
    assert_eq!(
        idx.resolve_character("avg_225_haak_1#9$1")
            .unwrap()
            .face_url,
        None
    );
    assert!(idx.resolve_character("AVG_225_Haak_1#1$1").is_some());
    assert_eq!(
        idx.sprite_folder_name("avg_225_haak#2"),
        Some("avg_225_haak_1")
    );
    assert_eq!(
        idx.sprite_variant_keys("avg_225_haak_1"),
        ["#1$1", "#2$1", "#3$1"]
    );
}

#[test]
fn a_folder_named_unlike_its_only_file_resolves_through_the_hub() {
    let hub = r#"{"groups":[{"facePos":{"x":-1.0,"y":-1.0},"faceSize":{"w":0.0,"h":0.0},"sprites":[{"name":"char_2006_fmzuki_1","alias":"","isWholeBody":false,"size":{"w":1024.0,"h":1024.0}}]}],"legacy":true,"root":{"x":0.0,"y":160.0,"w":1024.0,"h":1024.0}}"#;
    let (_d, idx) = sprites(&[(
        "char_2006_weiywfmzuki_1",
        &["char_2006_fmzuki_1.png"],
        Some(hub),
    )]);
    for raw in ["char_2006_weiywfmzuki_1", "char_2006_weiywfmzuki_1#3"] {
        assert_eq!(
            idx.resolve_character(raw).unwrap().body_url,
            "/textures/avg/characters/char_2006_weiywfmzuki_1/char_2006_fmzuki_1.png"
        );
    }
}

#[test]
fn legacy_hub_order_decides_the_face_index_not_the_file_suffix() {
    let hub = r#"{"groups":[{"facePos":{"x":-1.0,"y":-1.0},"faceSize":{"w":0.0,"h":0.0},"sprites":[{"name":"avg_npc_043_2"},{"name":"avg_npc_043_1"}]}],"legacy":true}"#;
    let (_d, idx) = sprites(&[(
        "avg_npc_043_1",
        &["avg_npc_043_1.png", "avg_npc_043_2.png"],
        Some(hub),
    )]);
    let body = |raw: &str| idx.resolve_character(raw).unwrap().body_url;
    let base = "/textures/avg/characters/avg_npc_043_1/";
    assert_eq!(body("avg_npc_043_1#2"), format!("{base}avg_npc_043_1.png"));
    assert_eq!(body("avg_npc_043_1#1"), format!("{base}avg_npc_043_2.png"));
    assert_eq!(body("avg_npc_043_1"), format!("{base}avg_npc_043_2.png"));
    assert_eq!(body("avg_npc_043_1#7"), format!("{base}avg_npc_043_1.png"));
}

#[test]
fn legacy_sprite_resolves_by_face_index() {
    let (_d, idx) = sprites(&[
        (
            "char_002_amiya_1",
            &["char_002_amiya_1.png", "char_002_amiya_5.png"],
            None,
        ),
        (
            "char_362_saga",
            &["char_362_saga#1.png", "char_362_saga#6.png"],
            None,
        ),
        (
            "char_130_doberm_ex",
            &["char_130_doberm_ex.png", "char_130_doberm_ex_2.png"],
            None,
        ),
    ]);
    let s = idx.resolve_character("char_002_amiya_1#5").unwrap();
    assert_eq!(
        s.body_url,
        "/textures/avg/characters/char_002_amiya_1/char_002_amiya_5.png"
    );
    assert_eq!(s.face_url, None);
    for raw in ["char_002_amiya_1", "char_002_amiya_1#9"] {
        assert_eq!(
            idx.resolve_character(raw).unwrap().body_url,
            "/textures/avg/characters/char_002_amiya_1/char_002_amiya_1.png"
        );
    }
    assert_eq!(
        idx.resolve_character("char_362_Saga#6").unwrap().body_url,
        "/textures/avg/characters/char_362_saga/char_362_saga%236.png"
    );
    assert!(
        idx.resolve_character("char_002_amiya#5")
            .unwrap()
            .body_url
            .ends_with("char_002_amiya_5.png")
    );
    assert!(
        idx.resolve_character("char_130_doberm_ex#2")
            .unwrap()
            .body_url
            .ends_with("char_130_doberm_ex_2.png")
    );
    assert!(idx.resolve_character("nobody_here").is_none());
    assert!(idx.resolve_character("").is_none());
}

#[test]
fn old_hub_layout_serves_the_body_with_a_bare_face_patch() {
    let (_d, idx) = sprites(&[
        (
            "avg_1013_spchen_1",
            &[
                "avg_1013_spchen_1.png",
                "1.png",
                "2.png",
                "alpha.png",
                "avg_1013_spchen_1[alpha].png",
            ],
            None,
        ),
        (
            "avg_npc_122",
            &["avg_npc_122_1.png", "1.png", "2.png", "alpha.png"],
            None,
        ),
        ("avg_only_patches", &["1.png", "2.png"], None),
        (
            "avg_npc_139",
            &["avg_npc_139#1.png", "1.png", "2.png", "alpha.png"],
            None,
        ),
        (
            "avg_npc_135",
            &["avg_npc_135_1#1.png", "avg_npc_135_1#3.png"],
            None,
        ),
    ]);
    let pair = |raw: &str| {
        let s = idx.resolve_character(raw).unwrap();
        (s.body_url, s.face_url)
    };
    let at = |folder: &str, file: &str| format!("/textures/avg/characters/{folder}/{file}");
    assert_eq!(
        pair("avg_npc_139#2"),
        (
            at("avg_npc_139", "avg_npc_139%231.png"),
            Some(at("avg_npc_139", "2.png"))
        )
    );
    assert_eq!(
        pair("avg_npc_135#3"),
        (at("avg_npc_135", "avg_npc_135_1%233.png"), None)
    );
    assert!(pair("avg_npc_135").0.ends_with("avg_npc_135_1%231.png"));
    assert_eq!(
        pair("avg_1013_spchen_1#2"),
        (
            at("avg_1013_spchen_1", "avg_1013_spchen_1.png"),
            Some(at("avg_1013_spchen_1", "2.png"))
        )
    );
    assert_eq!(
        pair("avg_npc_122#2"),
        (
            at("avg_npc_122", "avg_npc_122_1.png"),
            Some(at("avg_npc_122", "2.png"))
        )
    );
    assert_eq!(pair("avg_npc_122#9").1, None);
    assert!(idx.resolve_character("avg_only_patches#1").is_none());
}

#[test]
fn zero_padded_indices_resolve() {
    let (_d, idx) = sprites(&[(
        "avg_172_svrash_1",
        &["avg_172_svrash_1$1.png", "1$1.png", "2$1.png"],
        None,
    )]);
    let base = "/textures/avg/characters/avg_172_svrash_1/";
    let s = idx.resolve_character("avg_172_svrash_1#01$1").unwrap();
    assert_eq!(s.face_url, Some(format!("{base}1$1.png")));
    let s = idx.resolve_character("avg_172_svrash_1#2$01").unwrap();
    assert_eq!(s.body_url, format!("{base}avg_172_svrash_1$1.png"));
    assert_eq!(s.face_url, Some(format!("{base}2$1.png")));
}

#[test]
fn video_index_keys_by_lowercased_relative_path() {
    let (_d, idx) = index(&[
        ("video/main_10/main_10_enter.webm".to_owned(), b"w".to_vec()),
        ("video/main_10/main_10_enter.mp4".to_owned(), b"m".to_vec()),
    ]);
    let v = idx
        .resolve_video("video/main_10/MAIN_10_ENTER.mp4")
        .unwrap();
    assert_eq!(
        v.webm_url.as_deref(),
        Some("/video/main_10/main_10_enter.webm")
    );
    assert_eq!(
        v.mp4_url.as_deref(),
        Some("/video/main_10/main_10_enter.mp4")
    );
    assert!(idx.resolve_video("main_10/main_10_enter.mp4").is_some());
    assert!(idx.resolve_video("video/act15side/IW01.mp4").is_none());
    assert!(idx.resolve_video("").is_none());
}

#[test]
fn audio_resolves_through_variables_and_bare_paths() {
    let (_d, idx) = index(&[
        (
            "audio/audio/sound_beta_2/music/act25side/m_avg_decisivebattle/m_avg_DecisiveBattle_loop.ogg".to_owned(),
            Vec::new(),
        ),
        ("audio/audio/sound_beta_2/avg_se_3/d_avg_devicebeep.ogg".to_owned(), Vec::new()),
        (
            "gamedata/story/story_variables.json.json".to_owned(),
            br#"{"DecisiveBattle_loop": "Sound_Beta_2/Music/act25side/m_avg_DecisiveBattle_loop", "avatar_amiya": "char_002_amiya"}"#.to_vec(),
        ),
        (
            "textures/spritepack/ui_char_avatar_0/char_002_amiya_1+.png".to_owned(),
            Vec::new(),
        ),
        ("audio/audio/sound_beta_2/voice_en/char_002_amiya/CN_001.ogg".to_owned(), Vec::new()),
    ]);
    assert!(idx.resolve_audio("$DecisiveBattle_loop").is_some());
    assert!(idx.resolve_audio("$missing").is_none());
    assert!(
        idx.resolve_audio("Sound_Beta_2/AVG/d_avg_devicebeep")
            .is_some()
    );
    assert!(idx.resolve_audio("d_avg_DeviceBeep").is_some());
    assert!(idx.resolve_audio("").is_none());
    assert_eq!(
        idx.resolve_avatar("$avatar_amiya").as_deref(),
        Some("/textures/spritepack/ui_char_avatar_0/char_002_amiya_1+.png")
    );
    assert!(idx.has_voice("voice_en", "char_002_amiya", "CN_001"));
    assert!(!idx.has_voice("voice", "char_002_amiya", "CN_001"));
}

#[test]
fn image_lookup_order() {
    let (_d, idx) = index(&[
        ("textures/avg/bg/x/shared.png".to_owned(), Vec::new()),
        ("textures/avg/imgs/y/shared.png".to_owned(), Vec::new()),
        ("textures/avg/imgs/y/cg_only.png".to_owned(), Vec::new()),
        (
            "textures/avg/imgs/y/cg_only[alpha].png".to_owned(),
            Vec::new(),
        ),
    ]);
    assert_eq!(
        idx.resolve_background("Shared").unwrap().1,
        ImageSource::Background
    );
    assert_eq!(idx.resolve_image("shared").unwrap().1, ImageSource::Image);
    assert_eq!(
        idx.resolve_background("cg_only").unwrap().1,
        ImageSource::Image
    );
    assert!(idx.resolve_image("nothing").is_none());
}

#[test]
fn the_first_listed_file_of_a_stem_wins() {
    let src = DirSource {
        root: std::path::PathBuf::from("/nonexistent"),
    };
    let order = |paths: &[&str]| {
        let idx = StoryAssetIndex::build(&src, &src, &listing(paths)).unwrap();
        (
            idx.resolve_background("bg_x").unwrap().0,
            idx.resolve_avatar("char_1").unwrap(),
        )
    };
    let nested = "textures/avg/bg/bg_x/bg_x.png";
    let top = "textures/avg/bg/bg_x.png";
    let (a, b) = (
        "textures/spritepack/ui_char_avatar_0/char_1.png",
        "textures/spritepack/ui_char_avatar_1/char_1.png",
    );
    assert_eq!(
        order(&[nested, top, a, b]),
        (format!("/{nested}"), format!("/{a}"))
    );
    assert_eq!(
        order(&[top, nested, b, a]),
        (format!("/{top}"), format!("/{b}"))
    );
}

#[test]
fn a_tree_the_listing_leaves_empty_costs_only_itself() {
    let src = DirSource {
        root: std::path::PathBuf::from("/nonexistent"),
    };
    let paths = [
        "textures/avg/bg/bg_cher_1.png",
        "textures/spritepack/ui_char_avatar_0/char_1.png",
    ];
    let full = listing(&paths);
    let without_characters = |tree: &str| {
        if tree == "textures/avg/characters" {
            Vec::new()
        } else {
            full(tree)
        }
    };
    let idx = StoryAssetIndex::build(&src, &src, &without_characters).unwrap();
    assert!(idx.resolve_background("bg_cher_1").is_some());
    assert!(idx.resolve_avatar("char_1").is_some());
    assert!(idx.resolve_character("char_x").is_none());
}

#[test]
fn a_sprite_header_is_read_when_first_asked_for() {
    let rel = "textures/avg/characters/char_y/char_y.png";
    let (dir, idx) = index(&[(rel.to_owned(), png(10, 10))]);
    std::fs::write(dir.path().join(rel), png(640, 480)).unwrap();
    let size = idx.resolve_character("char_y").unwrap().body_size;
    assert_eq!(size, Some(BodySize { w: 640.0, h: 480.0 }));
    std::fs::write(dir.path().join(rel), png(20, 20)).unwrap();
    let again = idx.resolve_character("char_y").unwrap().body_size;
    assert_eq!(again, size, "read once, then cached");
}

#[test]
fn a_failed_sprite_header_read_is_retried() {
    let rel = "textures/avg/characters/char_z/char_z.png";
    let (dir, idx) = index(&[(rel.to_owned(), b"not yet a png".to_vec())]);
    assert_eq!(idx.resolve_character("char_z").unwrap().body_size, None);
    std::fs::write(dir.path().join(rel), png(64, 32)).unwrap();
    assert_eq!(
        idx.resolve_character("char_z").unwrap().body_size,
        Some(BodySize { w: 64.0, h: 32.0 })
    );
}
