//! The sidecar patch rebuilds `gacha` and `event_shops` through
//! `gamedata::load_sidecar_parts` instead of a full load. Pins that on the real
//! EN tree the two are IDENTICAL to what the full load builds, so the patch
//! path can never serve banners or shops the full reload would not.
//!
//! Skipped (passes vacuously) when `../assets/output/en` is absent.

mod common;

use std::path::Path;

#[test]
fn sidecar_parts_match_the_full_load_on_en() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/output/en");
    if !assets.join("gamedata/excel").is_dir() {
        eprintln!("EN tree absent; skipped");
        return;
    }
    let full = common::load_game_data();
    let parts =
        backend::core::gamedata::load_sidecar_parts(&assets.join("gamedata/excel"), &assets);

    let full_gacha = serde_json::to_value(&full.gacha).expect("gacha");
    let part_gacha = serde_json::to_value(&parts.gacha).expect("gacha");
    assert_eq!(
        full_gacha, part_gacha,
        "gacha differs between full load and patch"
    );
    assert_eq!(full.event_shops, parts.event_shops, "event shops differ");

    let pools = parts.gacha.gacha_pool_client.len();
    eprintln!(
        "parity: {pools} banners, {} event shops, {} sidecar warnings",
        parts.event_shops.len(),
        parts.warnings.len()
    );
    assert!(pools > 0, "EN carries banners");
}
