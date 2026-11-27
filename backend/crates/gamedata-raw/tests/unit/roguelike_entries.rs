use crate::tables::roguelike_entries::{is_band_variant, natural_cmp, strip_variant};

#[test]
fn variant_suffixes_are_cut() {
    assert_eq!(strip_variant("rogue_5_wrath_1_a"), "rogue_5_wrath_1");
    assert_eq!(strip_variant("rogue_5_wrath_1_a_3"), "rogue_5_wrath_1");
    assert_eq!(
        strip_variant("rogue_5_copper_B_01_a"),
        "rogue_5_copper_B_01"
    );
    assert_eq!(strip_variant("rogue_1_band_1"), "rogue_1_band_1");
    assert!(is_band_variant("rogue_4_band_22_7"));
    assert!(!is_band_variant("rogue_4_band_22"));
}

#[test]
fn ids_sort_by_number() {
    let mut ids = vec!["band_10", "band_2", "band_1"];
    ids.sort_by(|a, b| natural_cmp(a, b));
    assert_eq!(ids, ["band_1", "band_2", "band_10"]);
}
