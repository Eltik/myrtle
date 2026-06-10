#[must_use]
pub fn keep_for_operators(name: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "anon/",
        "chararts/",
        "skinpack/",
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
    name.ends_with(".idx") || PREFIXES.iter().any(|p| name.starts_with(p))
}

#[cfg(test)]
mod tests {
    use super::keep_for_operators;

    #[test]
    fn keeps_operator_assets_and_ui_icons() {
        assert!(keep_for_operators("chararts/char_002_amiya.ab"));
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
}
