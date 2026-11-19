#![allow(clippy::unwrap_used, clippy::expect_used)]

use image::{Rgba, RgbaImage};
use myrtle_engine_story::assets::{BodySize, CharacterSprite, FacePos};
use myrtle_engine_story::dto::{StoryArtKind, StoryCategory};
use myrtle_engine_story::gallery::{GallerySize, cache_rel_path, render_jpeg};
use myrtle_engine_story::index::group_category;
use myrtle_engine_story::story_art::{self, Exclusion, MIN_BYTES_PER_PX, header_fit};
use myrtle_engine_story::thumbs::{
    THUMB_H, ThumbFormat, compose, crop_rect, head_rel_path, is_blank, render_head,
    variant_file_stem, variant_rel_path,
};
use myrtle_gamedata_raw::tables::story::{StoryReviewGroup, StoryUnlockData};

fn png(img: &RgbaImage) -> Vec<u8> {
    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(img.clone())
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn plate(w: u32, h: u32, figure: impl Fn(u32, u32) -> bool) -> RgbaImage {
    RgbaImage::from_fn(w, h, |x, y| {
        if figure(x, y) {
            Rgba([200, 100, 50, 255])
        } else {
            Rgba([0, 255, 0, 0])
        }
    })
}

#[test]
fn categories_derive_from_entry_and_act_type_then_prefix() {
    let cat = |entry: &str, act: &str, txt: &str| {
        group_category(&StoryReviewGroup {
            entry_type: entry.to_owned(),
            act_type: act.to_owned(),
            info_unlock_datas: vec![StoryUnlockData {
                story_txt: txt.to_owned(),
                ..StoryUnlockData::default()
            }],
            ..StoryReviewGroup::default()
        })
    };
    assert_eq!(
        cat("MAINLINE", "MAIN_STORY", "obt/main/x"),
        StoryCategory::Main
    );
    assert_eq!(
        cat("ACTIVITY", "ACTIVITY_STORY", "activities/a/x"),
        StoryCategory::Side
    );
    assert_eq!(
        cat("MINI_ACTIVITY", "MINI_STORY", "activities/a/x"),
        StoryCategory::Vignette
    );
    assert_eq!(cat("NONE", "NONE", "obt/memory/x"), StoryCategory::Record);
    assert_eq!(cat("NONE", "NONE", "obt/rogue/x"), StoryCategory::Is);
    assert_eq!(cat("NONE", "NONE", "obt/roguelike/x"), StoryCategory::Is);
    assert_eq!(
        cat("NONE", "NONE", "obt/sandboxperm/x"),
        StoryCategory::Reclamation
    );
    assert_eq!(
        cat("NONE", "NONE", "obt/legion/x"),
        StoryCategory::SideContent
    );
    assert_eq!(StoryCategory::SideContent.as_str(), "sideContent");
    assert_eq!(
        serde_json::to_string(&StoryCategory::SideContent).unwrap(),
        "\"sideContent\""
    );
}

#[test]
fn the_header_band_takes_16_9_and_the_1280x712_cgs_only() {
    let fit = |w, h| header_fit(w, h, None, MIN_BYTES_PER_PX);
    assert_eq!(fit(1600, 900), Ok(()));
    assert_eq!(fit(1280, 712), Ok(()));
    assert_eq!(fit(1024, 576), Ok(()));
    assert_eq!(fit(1023, 576), Err(Exclusion::Narrow));
    assert_eq!(fit(512, 1024), Err(Exclusion::Narrow));
    assert_eq!(fit(1024, 768), Err(Exclusion::Aspect));
    assert_eq!(fit(2048, 1024), Err(Exclusion::Aspect));
    assert_eq!(fit(1024, 0), Err(Exclusion::Aspect));
}

#[test]
fn a_title_card_is_too_sparse_and_the_floor_switches_off() {
    let px = 1024 * 576;
    assert_eq!(
        header_fit(1024, 576, Some(12_790), MIN_BYTES_PER_PX),
        Err(Exclusion::Sparse)
    );
    assert_eq!(
        header_fit(1024, 576, Some(px / 5 + 1), MIN_BYTES_PER_PX),
        Ok(())
    );
    assert_eq!(header_fit(1024, 576, Some(12_790), 0.0), Ok(()));
    assert_eq!(header_fit(1024, 576, None, MIN_BYTES_PER_PX), Ok(()));
    assert_eq!(
        header_fit(500, 500, Some(1), MIN_BYTES_PER_PX),
        Err(Exclusion::Narrow)
    );
}

#[test]
fn ids_land_inside_their_cache() {
    assert_eq!(
        cache_rel_path("pic_rogue_1_KV1", GallerySize::Thumb),
        "derived/archive-pics/thumb/pic_rogue_1_kv1.jpg"
    );
    assert_eq!(
        cache_rel_path("../x/y", GallerySize::Header),
        "derived/archive-pics/header/___x_y.jpg"
    );
    assert_eq!(
        story_art::cache_rel_path(StoryArtKind::Cg, "avg_1_1", GallerySize::Thumb),
        "derived/story-art/cg/thumb/avg_1_1.jpg"
    );
    assert_eq!(
        story_art::cache_rel_path(StoryArtKind::Scene, "../bg#2", GallerySize::Header),
        "derived/story-art/scene/header/___bg_2.jpg"
    );
    assert_eq!(variant_file_stem("#2$1"), "f2b1");
    assert_eq!(variant_file_stem("@smile"), "asmile");
    assert_eq!(variant_file_stem("#1/../x"), "f1____x");
    assert_eq!(
        variant_rel_path("AVG_NPC_043_1", "#2$1", ThumbFormat::Webp),
        "derived/sprite-thumbs/avg_npc_043_1/f2b1.webp"
    );
    assert_ne!(
        head_rel_path("avg_npc_1", ThumbFormat::Webp),
        head_rel_path("avg_npc_1", ThumbFormat::Png)
    );
}

#[test]
fn a_gallery_variant_keeps_the_aspect_and_never_upscales() {
    let src = RgbaImage::from_pixel(1600, 900, Rgba([200, 100, 50, 255]));
    let size = |bytes: Vec<u8>| {
        let img = image::load_from_memory(&bytes).unwrap();
        (img.width(), img.height())
    };
    assert_eq!(size(render_jpeg(&png(&src), 320).unwrap()), (320, 180));
    assert_eq!(size(render_jpeg(&png(&src), 1600).unwrap()), (1600, 900));
    let small = RgbaImage::from_pixel(800, 450, Rgba([0, 0, 0, 255]));
    assert_eq!(size(render_jpeg(&png(&small), 1600).unwrap()), (800, 450));
}

#[test]
fn gallery_transparency_lands_on_black() {
    let src = RgbaImage::from_pixel(64, 36, Rgba([255, 255, 255, 0]));
    let out = image::load_from_memory(&render_jpeg(&png(&src), 64).unwrap())
        .unwrap()
        .to_rgb8();
    assert!(out.pixels().all(|p| p[0] < 8 && p[1] < 8 && p[2] < 8));
}

fn sprite(face: Option<FacePos>, body: (f32, f32)) -> CharacterSprite {
    CharacterSprite {
        body_url: "/b.png".to_owned(),
        face_url: face.map(|_| "/f.png".to_owned()),
        face_pos: face,
        body_size: Some(BodySize {
            w: body.0,
            h: body.1,
        }),
        plate: None,
    }
}

#[test]
fn the_face_lands_at_face_pos_in_texture_pixels() {
    let body = RgbaImage::from_pixel(200, 200, Rgba([0, 0, 255, 255]));
    let face = RgbaImage::from_pixel(64, 64, Rgba([255, 0, 0, 255]));
    let s = sprite(
        Some(FacePos {
            x: 100.0,
            y: 40.0,
            w: 50.0,
            h: 50.0,
        }),
        (400.0, 400.0),
    );
    let out = compose(&png(&body), Some(&png(&face)), &s, 200).unwrap();
    assert_eq!(out.dimensions(), (200, 200));
    assert_eq!(out.get_pixel(60, 30)[0], 255, "inside the patch");
    assert_eq!(out.get_pixel(40, 30)[2], 255, "left of the patch is body");
    assert_eq!(out.get_pixel(60, 50)[2], 255, "below the patch is body");
}

#[test]
fn the_height_is_fixed_and_the_width_follows_the_plate() {
    let body = RgbaImage::from_pixel(1280, 1024, Rgba([10, 20, 30, 255]));
    let out = compose(&png(&body), None, &sprite(None, (1280.0, 1024.0)), THUMB_H).unwrap();
    assert_eq!(out.dimensions(), (400, 320));
}

#[test]
fn a_hub_face_centres_the_crop_inside_the_plate() {
    let img = plate(1000, 1000, |_, _| true);
    assert_eq!(crop_rect(&img, Some((0.6, 0.15))), (455, 40, 290));
    let (l, t, s) = crop_rect(&img, Some((1.0, 1.0)));
    assert_eq!((l + s, t + s), (1000, 1000));
}

#[test]
fn without_a_hub_face_the_crop_finds_the_head_in_the_alpha() {
    let img = plate(1000, 1000, |x, y| y >= 100 && (250..350).contains(&x));
    assert_eq!(crop_rect(&img, None), (155, 109, 290));
}

#[test]
fn the_head_resize_does_not_bleed_transparent_colour_into_the_figure() {
    let src = png(&plate(400, 400, |x, _| x < 200));
    let out = image::load_from_memory(
        &render_head(&src, Some((0.5, 0.5)), 64, ThumbFormat::Webp).unwrap(),
    )
    .unwrap()
    .to_rgba8();
    assert_eq!(out.dimensions(), (64, 64));
    for p in out.pixels().filter(|p| p[3] > 0) {
        assert!(p[1] < 140, "green bled into the edge: {p:?}");
    }
}

#[test]
fn png_and_webp_carry_the_same_pixels() {
    let src = png(&plate(400, 400, |x, y| x < 200 && y > 50));
    let webp = render_head(&src, None, 64, ThumbFormat::Webp).unwrap();
    let out_png = render_head(&src, None, 64, ThumbFormat::Png).unwrap();
    assert_eq!(
        image::guess_format(&out_png).unwrap(),
        image::ImageFormat::Png
    );
    assert_eq!(
        image::load_from_memory(&webp).unwrap().to_rgba8(),
        image::load_from_memory(&out_png).unwrap().to_rgba8()
    );
}

#[test]
fn a_blank_plate_is_all_transparent() {
    assert!(is_blank(&png(&plate(24, 24, |_, _| false))));
    assert!(!is_blank(&png(&plate(24, 24, |x, _| x == 3))));
    assert!(!is_blank(b"not a png"));
}
