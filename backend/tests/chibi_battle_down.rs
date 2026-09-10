//! `BattleDown` is the third battle facing, exported since 2026-09-09. It must reach the site
//! as an extra animation type on the token entries that already exist, not as a separate
//! character, and it must not disturb the four keys that were there before.
//!
//! Built from a fixture rather than the local export: `assets/output/*` predates the category,
//! so a test reading it would pass by skipping.

use std::fs;
use std::path::Path;

use backend::core::gamedata::enrich::chibi::init_chibi_data;

/// Write the `.atlas`/`.skel`/`.png` triple a spine set needs to count as complete.
fn write_set(root: &Path, category: &str, dir: &str, stem: &str) {
    let d = root.join("spine").join(category).join(dir);
    fs::create_dir_all(&d).expect("create fixture dir");
    for ext in ["atlas", "skel", "png"] {
        fs::write(d.join(format!("{stem}.{ext}")), b"x").expect("write fixture file");
    }
}

#[test]
fn battle_down_lands_as_a_fifth_animation_type_on_the_existing_entry() {
    let tmp = std::env::temp_dir().join(format!("chibi_down_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);

    // One token with all three battle facings plus a dorm pose, exactly the shape
    // `token_10027_ironmn_pile3` has.
    let token = "token_10027_ironmn_pile3";
    write_set(&tmp, "BattleFront", token, token);
    write_set(&tmp, "BattleBack", token, token);
    write_set(&tmp, "BattleDown", token, token);
    write_set(&tmp, "Building", token, &format!("build_{token}"));
    // A plain operator with no Down, to confirm nothing gains a spurious key.
    write_set(&tmp, "BattleFront", "char_101_sora", "char_101_sora");

    let data = init_chibi_data(&tmp);

    let tok = data
        .get_by_operator(token)
        .unwrap_or_else(|| panic!("{token} present"));
    assert_eq!(tok.skins.len(), 1, "one skin, not one per category");
    let types = &tok.skins[0].animation_types;
    let mut keys: Vec<&str> = types.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["back", "dorm", "down", "front"],
        "down merges onto the same skin beside the facings it already had"
    );
    let down = types.get("down").expect("down present");
    assert!(down.atlas.is_some() && down.skel.is_some() && down.png.is_some());
    assert_eq!(
        down.skel.as_deref().unwrap(),
        format!("/spine/BattleDown/{token}/{token}.skel"),
        "down is served from its own category directory"
    );

    // The token is not duplicated into a second character by the extra directory.
    let matching = data
        .characters
        .iter()
        .filter(|c| c.operator_code == token)
        .count();
    assert_eq!(matching, 1, "one character, not one per category");

    // An operator with no Down keeps exactly what it had.
    let sora = data
        .get_by_operator("char_101_sora")
        .expect("char_101_sora present");
    let sora_keys: Vec<&str> = sora.skins[0]
        .animation_types
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(sora_keys, ["front"], "no spurious down key");

    let _ = fs::remove_dir_all(&tmp);
}
