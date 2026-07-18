/// Index files (`hot_update_list.idx`, the resource-manifest `<md5>.idx`) are
/// kept by every profile — they drive incremental pulls and dependency
/// resolution.
fn is_idx(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("idx"))
}

/// Bundles the Stage Viewer needs: every stage's level scene (for the map/enemy
/// renderer and clickability) plus the map-preview thumbnails and zone/event/IS
/// banner art.
///
/// The `operators` profile omits all of these, so newer events (e.g. a
/// just-released side story) and mode art never arrive on an incremental pull.
#[must_use]
pub fn keep_for_stages(name: &str) -> bool {
    const PREFIXES: &[&str] = &[
        // Stage level scenes -> gamedata/levels (all modes + events).
        "scenes/activities/",
        "scenes/obt/",
        // Map-preview thumbnails (main / event / SSS / Annihilation / IS `ro*` /
        // CC / Paradox / Rune / RA-in-arts).
        "arts/ui/stage_mappreview_h2_",
        // Reclamation Algorithm previews live in the sprite pack instead.
        "spritepack/sandbox_1_stage_mappreview_",
        // Zone / event / mode banner key-art.
        "spritepack/ui_home_act_banner_zone_",
        "spritepack/ui_zone_home_theme_",
        // Integrated Strategies season KV banners (avg image bundles).
        "avg/imgs/avg_img_rogue_",
        "avg/imgs/asp_rl",
    ];
    is_idx(name) || PREFIXES.iter().any(|p| name.starts_with(p))
}

#[must_use]
pub fn keep_for_operators(name: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "anon/",
        "chararts/",
        "skinpack/",
        "arts/dynchars/",
        "arts/dynavatars",
        "[uc]shaders",
        "refs/fx/",
        "spritepack/char_portrait_",
        "arts/charportraits/",
        "spritepack/ui_char_avatar_",
        "spritepack/skill_icons_",
        "spritepack/ui_equip_",
        "arts/ui_item_icons_",
        "arts/items/item_icons_",
        "spritepack/building_ui_buff_skills_",
        "spritepack/ui_sub_profession_icon_hub_",
        "arts/elite_hub",
        "arts/potential_hub",
        "spritepack/ui_camp_logo_",
        "audio/sound_beta_2/player",
        "audio/sound_beta_2/btl_snd_",
        "audio/sound_beta_2/voice",
    ];
    is_idx(name) || PREFIXES.iter().any(|p| name.starts_with(p))
}

#[cfg(test)]
mod tests {
    use super::keep_for_operators;

    #[test]
    fn keeps_operator_assets_and_ui_icons() {
        assert!(keep_for_operators("chararts/char_002_amiya.ab"));
        assert!(keep_for_operators(
            "arts/dynchars/char_1012_skadi2_boc#4.ab"
        ));
        assert!(keep_for_operators("arts/dynavatars_0.ab"));
        assert!(keep_for_operators("spritepack/skill_icons_h1_0.ab"));
        assert!(keep_for_operators("anon/foo.ab"));
        assert!(keep_for_operators("hot_update_list.idx"));
        assert!(keep_for_operators("arts/ui_item_icons_5.ab"));
        assert!(keep_for_operators("arts/items/item_icons_no_tiny_hub.ab"));
        assert!(keep_for_operators(
            "spritepack/building_ui_buff_skills_h1_0.ab"
        ));
        assert!(keep_for_operators(
            "spritepack/ui_sub_profession_icon_hub_h2_0.ab"
        ));
        assert!(keep_for_operators("arts/elite_hub.ab"));
        assert!(keep_for_operators("arts/potential_hub.ab"));
        assert!(keep_for_operators("spritepack/ui_camp_logo_0.ab"));
        assert!(keep_for_operators("audio/sound_beta_2/player_0.ab"));
        assert!(keep_for_operators("audio/sound_beta_2/btl_snd_0.ab"));
        assert!(keep_for_operators(
            "audio/sound_beta_2/voice/char_002_amiya/cn_001.ab"
        ));
        assert!(keep_for_operators(
            "audio/sound_beta_2/voice_cn/char_1052_kalts2/cn_001.ab"
        ));
        assert!(!keep_for_operators("arts/ui/[uc]loadingbg.ab"));
        assert!(!keep_for_operators("audio/sound_beta_2/music_0.ab"));
        assert!(!keep_for_operators("audio/sound_beta_2/enmy_snd_atk_0.ab"));
    }

    #[test]
    fn keeps_stage_viewer_assets() {
        use super::keep_for_stages;
        // level scenes (map viewer + clickability)
        assert!(keep_for_stages(
            "scenes/activities/act45side/level_act45side_02/level_act45side_02.ab"
        ));
        assert!(keep_for_stages(
            "scenes/obt/roguelike/ro1/level_rogue1_4-5/level_rogue1_4-5.ab"
        ));
        // previews
        assert!(keep_for_stages("arts/ui/stage_mappreview_h2_main_09_0.ab"));
        assert!(keep_for_stages("arts/ui/stage_mappreview_h2_ro1_n_0.ab"));
        assert!(keep_for_stages(
            "spritepack/sandbox_1_stage_mappreview_0.ab"
        ));
        // banners
        assert!(keep_for_stages(
            "spritepack/ui_zone_home_theme_act48side.ab"
        ));
        assert!(keep_for_stages("spritepack/ui_home_act_banner_zone_0.ab"));
        // IS KV banners
        assert!(keep_for_stages("avg/imgs/avg_img_rogue_2p2_0.ab"));
        assert!(keep_for_stages("avg/imgs/asp_rl3_6_0.ab"));
        assert!(keep_for_stages("hot_update_list.idx"));
        // not stage-viewer content
        assert!(!keep_for_stages("chararts/char_002_amiya.ab"));
        assert!(!keep_for_stages("audio/sound_beta_2/music_0.ab"));
    }
}
