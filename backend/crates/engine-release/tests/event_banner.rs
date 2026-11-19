#![allow(clippy::unwrap_used, clippy::expect_used)]

use image::{ImageEncoder, Rgba, RgbaImage};
use myrtle_engine_release::event_banner::{HEIGHT, WIDTH, cache_rel_path, render};

fn png(img: &RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    bytes
}

#[test]
fn the_banner_takes_the_left_of_the_key_visual_and_lays_the_title_right() {
    let bg = RgbaImage::from_fn(1660, 936, |x, _| {
        if x < 830 {
            Rgba([255, 0, 0, 255])
        } else {
            Rgba([0, 0, 255, 255])
        }
    });
    let title = RgbaImage::from_fn(516, 260, |x, y| {
        if (100..416).contains(&x) && (80..180).contains(&y) {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([0, 0, 0, 0])
        }
    });
    let jpeg = render(&png(&bg), &png(&title)).unwrap();
    let out = image::load_from_memory(&jpeg).unwrap().to_rgb8();
    assert_eq!(out.dimensions(), (WIDTH, HEIGHT));
    let left = out.get_pixel(10, HEIGHT / 2);
    assert!(left[0] > 200 && left[2] < 60, "{left:?}");
    let title_px = out.get_pixel(WIDTH - 60, HEIGHT / 2);
    assert!(title_px.0.iter().all(|&c| c > 200), "{title_px:?}");
    // trimmed to its ink, the 316 px title fills the width budget rather than its 516 px canvas
    let ink_left = (0..WIDTH)
        .find(|&x| out.get_pixel(x, HEIGHT / 2).0.iter().all(|&c| c > 200))
        .unwrap();
    assert!(ink_left < WIDTH / 2 + 20, "title starts at {ink_left}");
}

#[test]
fn a_fully_transparent_title_leaves_the_key_visual_alone() {
    let bg = RgbaImage::from_pixel(1660, 936, Rgba([0, 255, 0, 255]));
    let title = RgbaImage::new(516, 260);
    let out = image::load_from_memory(&render(&png(&bg), &png(&title)).unwrap())
        .unwrap()
        .to_rgb8();
    let px = out.get_pixel(WIDTH - 60, HEIGHT / 2);
    assert!(px[1] > 200 && px[0] < 60, "{px:?}");
}

#[test]
fn an_empty_source_does_not_render() {
    assert!(render(&[], &[]).is_err());
}

#[test]
fn the_cache_path_is_versioned_under_derived() {
    assert_eq!(
        cache_rel_path("act17side"),
        "derived/event-banners/v1/act17side.jpg"
    );
}
