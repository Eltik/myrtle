use crate::enrich::chibi::is_start_companion;

#[test]
fn start_marker_with_and_without_skin_suffix() {
    assert!(is_start_companion("char_003_kalts_boc#6_Start"));
    assert!(is_start_companion("char_2023_ling_nian_Start#12"));
    assert!(is_start_companion("char_2023_ling_nian_start#12"));
    assert!(!is_start_companion("char_2023_ling_nian#12"));
    assert!(!is_start_companion("char_2023_ling_nian_Start#"));
    assert!(!is_start_companion("char_2023_ling_nian_Starter#12"));
}
