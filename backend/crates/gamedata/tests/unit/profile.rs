use crate::enrich::profile::*;

// `char_377_gdglow`'s basic-info section on three servers, 2026-10-06.
const EN: &str = "[Code Name] Goldenglow\n[Gender] Female\n[Combat Experience] None\n[Place of Birth] Victoria\n[Date of Birth] Jan 7\n[Race] Feline\n[Height] 159cm\n[Infection Status]\nConfirmed Infected by medical examination.";
const KR: &str = "[코드네임] 골든글로우\n[성별] 여\n[전투 경험] 없음\n[출신지] 빅토리아\n[생일] 1월 7일\n[종족] 필라인\n[신장] 159cm\n[광석병 감염 상황]\n의학 테스트 보고서 참고 결과 감염자로 확인.";
const JP: &str = "【コードネーム】ゴールデングロー\n【性別】女\n【戦闘経験】なし\n【出身地】ヴィクトリア\n【誕生日】1月7日\n【種族】フェリーン\n【身長】159cm\n【鉱石病感染状況】\nメディカルチェックの結果、感染者に認定。";

#[test]
fn unknown_labels_parse_to_nothing() {
    assert!(parse_basic_info(KR).code_name.is_empty());
    assert!(parse_basic_info(JP).code_name.is_empty());
}

#[test]
fn relabelled_text_keeps_the_targets_own_values() {
    let kr = parse_basic_info(&relabel(KR, EN).expect("same line count"));
    assert_eq!(kr.code_name, "골든글로우");
    assert_eq!(kr.date_of_birth, "1월 7일");
    assert_eq!(kr.height, "159cm");

    let jp = parse_basic_info(&relabel(JP, EN).expect("same line count"));
    assert_eq!(jp.code_name, "ゴールデングロー");
    assert_eq!(jp.combat_experience, "なし");
}

#[test]
fn a_line_count_mismatch_refuses_to_align() {
    assert!(relabel(KR, "[Code Name] Goldenglow\n[Gender] Female").is_none());
    assert!(relabel("[코드네임] 골든글로우", EN).is_none());
}
