use crate::enrich::stage_index::{mode_stage_id, preview_keys};

#[test]
fn plain_stage_is_its_own_only_key() {
    assert_eq!(preview_keys("main_01-07", None, ""), vec!["main_01-07"]);
}

#[test]
fn challenge_mode_and_ex_variants_fall_back_to_the_base_stage() {
    assert_eq!(
        preview_keys("main_11-01#f#", None, ""),
        vec!["main_11-01#f#", "main_11-01"]
    );
    assert_eq!(
        preview_keys("lt_08_02_ex", None, ""),
        vec!["lt_08_02_ex", "lt_08_02"]
    );
    assert_eq!(
        preview_keys("act1_ex#f#", None, ""),
        vec!["act1_ex#f#", "act1_ex", "act1"]
    );
}

#[test]
fn procedural_nodes_use_their_level_file() {
    assert_eq!(
        preview_keys("ro4_n_1", Some("Obt/Roguelike/RO4/level_rogue4_b-4-c"), ""),
        vec!["ro4_n_1", "rogue4_b-4-c"]
    );
    assert_eq!(
        preview_keys("mem_x", Some("Obt/Memory/level_memory_gdglow_1"), ""),
        vec!["mem_x", "mem_gdglow_1", "memory_gdglow_1"]
    );
    assert_eq!(
        preview_keys("sb", Some("obt/sandbox/level_sandbox1_04"), ""),
        vec!["sb", "sandbox_1_04", "sandbox1_04"]
    );
    // Windows separators and case are normalized.
    assert_eq!(
        preview_keys("x", Some("Obt\\Main\\Level_Main_01-07"), ""),
        vec!["x", "main_01-07"]
    );
}

#[test]
fn a_contract_code_adds_the_crisis_key_last() {
    assert_eq!(
        preview_keys("cc_x", Some("obt/crisis/level_crisis_v2_01-02"), "01-02"),
        vec!["cc_x", "crisis_v2_01-02", "crisis_v2_01-02"]
    );
}

#[test]
fn mode_stage_id_is_the_last_path_segment() {
    assert_eq!(
        mode_stage_id("obt/roguelike/ro1/level_rogue1_6-1"),
        "level_rogue1_6-1"
    );
    assert_eq!(mode_stage_id("level_only"), "level_only");
    assert_eq!(mode_stage_id("trailing/"), "");
}
