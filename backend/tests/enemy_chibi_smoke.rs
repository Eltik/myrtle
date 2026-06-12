//! Smoke test for enemy chibi data loading. Runs against the local extracted
//! assets when present (`assets/output/en/spine/Enemy`) and skips otherwise.

use std::path::Path;

use backend::core::gamedata::enrich::chibi::init_enemy_chibi_data;
use backend::core::gamedata::types::enemy::EnemyHandbook;

#[test]
fn enemy_chibi_data_loads_from_local_assets() {
    let assets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/output/en");
    if !assets_dir.join("spine/Enemy").is_dir() {
        eprintln!("skipping: no local enemy spine assets");
        return;
    }

    let data = init_enemy_chibi_data(&assets_dir, &EnemyHandbook::default());

    assert!(
        data.characters.len() > 1400,
        "expected >1400 enemy chibis (one per spine set), got {}",
        data.characters.len()
    );

    // `_N`-suffixed ids are distinct handbook enemies (Hound / Hound Pro /
    // Rabid Hound Pro), each its own character with a single default skin —
    // even though they share one directory on disk.
    for id in [
        "enemy_1000_gopro",
        "enemy_1000_gopro_2",
        "enemy_1000_gopro_3",
    ] {
        let enemy = data
            .get_by_operator(id)
            .unwrap_or_else(|| panic!("{id} present"));
        assert_eq!(enemy.skins.len(), 1, "{id} has exactly one skin");
        let skin = &enemy.skins[0];
        assert_eq!(skin.name, "default");

        let front = skin
            .animation_types
            .get("front")
            .expect("front animation present");
        assert!(front.atlas.is_some() && front.skel.is_some() && front.png.is_some());
        assert_eq!(
            front.skel.as_deref().unwrap(),
            format!("/spine/Enemy/enemy_1000_gopro/{id}.skel"),
            "skel URL points into the shared directory"
        );
    }

    // Serialization shape consumed by the frontend: {"characters":[...]}
    let json = serde_json::to_value(&data).unwrap();
    let characters = json
        .get("characters")
        .and_then(|c| c.as_array())
        .expect("characters array");
    let first = &characters[0];
    assert!(first.get("operatorCode").is_some());
    assert!(first.get("skins").is_some());
}
