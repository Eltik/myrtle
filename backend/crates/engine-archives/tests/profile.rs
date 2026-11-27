#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_archives::profile::*;
use myrtle_types::Server;

const BASIC: &str = "[Code Name] Noir Corne\n[Gender] Male\n[Combat Experience] 8 years\n[Place of Birth] Higashi\n[Date of Birth] Aug. 30\n[Race] Oni\n[Height] 180cm\n[Infection Status]\nInfection confirmed by medical examination.";
const EXAM: &str = "[Physical Strength] Standard\n[Mobility] Normal\n[Physical Resilience] Excellent\n[Tactical Acumen] Normal\n[Combat Skill] Normal\n[Originium Arts Assilimation] Standard";
const CLINICAL: &str = "Radiological scan showed blurred outlines. The subject is confirmed to be infected. \n\n[Cell-Originium Assimilation] 11%\nNo major infection symptoms have yet appeared. \n\n[Blood Originium-Crystal Density] 0.22u/L\nSeverity of infection remains low.";

#[test]
fn reads_a_whole_file() {
    let p = parse_texts_in(&EN, &[BASIC, EXAM, "Profile text.", CLINICAL]);
    assert_eq!(p.code_name.as_deref(), Some("Noir Corne"));
    assert_eq!(p.gender.as_deref(), Some("Male"));
    assert_eq!(p.combat_years, Some(8.0));
    assert_eq!(p.birthplace.as_deref(), Some("Higashi"));
    assert_eq!((p.birth_month, p.birth_day), (Some(8), Some(30)));
    assert_eq!(p.race.as_deref(), Some("Oni"));
    assert_eq!(p.height_cm, Some(180.0));
    assert_eq!(
        p.infection.as_deref(),
        Some("Infection confirmed by medical examination.")
    );
    assert_eq!(p.infection_state, InfectionState::Infected);
    assert_eq!(p.assimilation_pct, Some(11.0));
    assert_eq!(p.blood_density_ul, Some(0.22));
    assert_eq!(p.exam.len(), 6);
    assert_eq!(p.exam[5].axis, ExamAxis::Arts);
    assert_eq!(p.exam[5].step, Some(ExamStep::Standard.rank()));
    assert!(p.extra.is_empty());
}

#[test]
fn heights() {
    assert_eq!(height_cm(&EN, "163 cm"), Some(163.0));
    assert_eq!(height_cm(&EN, "1.69m"), Some(169.0));
    assert_eq!(
        height_cm(
            &EN,
            "35cm (sitting) 75cm (stretching, including tail, approximate)"
        ),
        Some(35.0)
    );
    assert_eq!(height_cm(&EN, "Undisclosed"), None);
    assert_eq!(height_cm(&EN, "1G #qI3?JAYXFV ZHanG"), None);
}

#[test]
fn combat_experience() {
    assert_eq!(combat_years(&EN, "None"), Some(0.0));
    assert_eq!(combat_years(&EN, "None (self-declared)"), Some(0.0));
    assert_eq!(combat_years(&EN, "No combat experience"), Some(0.0));
    assert_eq!(combat_years(&EN, "Half a year"), Some(0.5));
    assert_eq!(
        combat_years(&EN, "Half-year mock combat training"),
        Some(0.5)
    );
    assert_eq!(combat_years(&EN, "6 Months"), Some(0.5));
    assert_eq!(combat_years(&EN, "Six months"), Some(0.5));
    assert_eq!(combat_years(&EN, "1 Month"), Some(0.08));
    assert_eq!(combat_years(&EN, "Seven Years"), Some(7.0));
    assert_eq!(combat_years(&EN, "15 years (Self-reported)"), Some(15.0));
    assert_eq!(combat_years(&EN, "70 years"), Some(70.0));
    assert_eq!(combat_years(&EN, "Undisclosed"), None);
    // The game's own two dashes, written as escapes.
    assert_eq!(combat_years(&EN, "\u{2014}\u{2014}"), None);
    assert_eq!(combat_years(&EN, "N/A"), None);
    assert_eq!(combat_years(&EN, "Classified"), None);
}

#[test]
fn birthdays() {
    assert_eq!(birthday(&EN, "Dec. 23"), Some((12, 23)));
    assert_eq!(birthday(&EN, "February 13"), Some((2, 13)));
    assert_eq!(birthday(&EN, "Jun. 1"), Some((6, 1)));
    assert_eq!(birthday(&CN, "12月23日"), Some((12, 23)));
    assert_eq!(birthday(&EN, "Unknown"), None);
    assert_eq!(birthday(&EN, "Reported as Unknown"), None);
}

#[test]
fn originium_numbers() {
    assert_eq!(number_before("0.2 u/L", &["u/l"], &ASCII_BOUNDS), Some(0.2));
    assert_eq!(number_before("Undisclosed", &["u/l"], &ASCII_BOUNDS), None);
    assert_eq!(number_before("<1%", &["%"], &ASCII_BOUNDS), None);
    assert_eq!(number_before("0%.", &["%"], &ASCII_BOUNDS), Some(0.0));
    assert_eq!(
        number_before(
            "0% Operator Degenbrecher shows no signs of Originium infection.",
            &["%"],
            &ASCII_BOUNDS
        ),
        Some(0.0)
    );
    assert_eq!(number_before("7.4%", &["%"], &ASCII_BOUNDS), Some(7.4));
    assert_eq!(number_before("?", &["%"], &ASCII_BOUNDS), None);
}

#[test]
fn en_infection_condition_label() {
    let p = parse_texts_in(
        &EN,
        &[
            "[Code Name] Breeze\n[Originium Infection Condition] \nMedical tests have confirmed that no infection is present.",
        ],
    );
    assert_eq!(p.infection_state, InfectionState::Clear);
}

#[test]
fn infection_sentences() {
    use InfectionState::{Clear, Infected, Unclear, Unstated};
    let cases = [
        (
            "Medical tests have confirmed that no infection is present.",
            Clear,
        ),
        ("Confirmed Infected by medical examination.", Infected),
        ("Non-infected as confirmed by medical report.", Clear),
        ("Confirmed uninfected by medical examination.", Clear),
        (
            "Originium crystals have not been found on the surface of the body, confirming there is no infection.",
            Clear,
        ),
        (
            "Showed no outward symptoms of Originium Infection, but Infection was confirmed by medical examination.",
            Infected,
        ),
        (
            "Traces of Originium Crystals have been detected in her body, confirming she is infected.",
            Infected,
        ),
        ("Medical tests confirm she is infected.", Infected),
        ("Medical tests confirm uninfected status.", Clear),
        ("Infection confirmed through clinical diagnosis.", Infected),
        ("Undisclosed", Unclear),
        ("", Unstated),
    ];
    for (text, want) in cases {
        assert_eq!(infection_state(&EN, text), want, "{text}");
    }
}

#[test]
fn exam_steps_refuse_splits_and_blocks() {
    assert_eq!(exam_step(&EN, "Standard"), Some(2));
    assert_eq!(exam_step(&CN, "缺陷"), Some(0));
    assert_eq!(exam_step(&EN, "Feeble"), exam_step(&EN, "Flawed"));
    assert_eq!(exam_step(&EN, "Defective"), exam_step(&CN, "缺陷"));
    assert_eq!(exam_step(&EN, "Exceptional"), exam_step(&CN, "卓越"));
    assert_eq!(exam_step(&EN, "Standard/Excellent"), None);
    assert_eq!(exam_step(&EN, "■■"), None);
    let p = parse_texts_in(
        &EN,
        &["[Code Name] X", "[Physical Strength] ■■\n[Mobility] 24km/h"],
    );
    assert!(p.exam[0].redacted);
    assert_eq!(p.exam[1].step, None);
}

#[test]
fn unknown_basic_labels_are_kept_as_extras() {
    let p = parse_texts_in(
        &EN,
        &[
            "[Code Name] Lancet-2\n[Model] Medical robot\n[Manufacturer] Rhodes Island\n[Gender] Male]",
        ],
    );
    assert_eq!(p.manufacturer.as_deref(), Some("Rhodes Island"));
    assert_eq!(p.gender.as_deref(), Some("Male"));
    assert_eq!(
        p.extra,
        vec![ProfileField {
            label: "Model".into(),
            value: "Medical robot".into()
        }]
    );
}

const JA_BASIC: &str = "【コードネーム】ノイルホーン\n【性別】男性\n【戦闘経験】八年\n【出身地】極東\n【誕生日】8月30日\n【種族】鬼\n【身長】180cm\n【鉱石病感染状況】\nメディカルチェックの結果、感染者に認定。";
const JA_EXAM: &str = "【物理強度】標準\n【戦場機動】普通\n【生理的耐性】優秀\n【戦術立案】普通\n【戦闘技術】普通\n【アーツ適性】標準";
const JA_CLINICAL: &str = "以上の結果から、鉱石病感染者と判定。\n\n【源石融合率】11%\n現状体表に鉱石病の兆候は見られない。\n\n【血液中源石密度】0.22u/L\n感染状況は浅い。";
const KO_BASIC: &str = "[코드네임] 노이르 코른\n[성별] 남\n[전투 경험] 8년\n[출신지] 극동\n[생일] 8월 30일\n[종족] 오니\n[신장] 180cm\n[광석병 감염 상황]\n의학 테스트 보고서 참고 결과 감염자로 확인.";
const KO_EXAM: &str = "[물리적 강도] 표준\n[전장 기동력] 보통\n[생체 인내도] 우수\n[전술 계획력] 보통\n[전투 기술력] 보통\n[오리지늄 아츠 적응성] 표준";
const KO_CLINICAL: &str = "[체세포와 오리지늄 융합률] 11%\n\n[혈중 오리지늄 결정 밀도] 0.22u/L";

#[test]
fn jp_and_kr_files_read_like_en() {
    let en = parse_texts_in(&EN, &[BASIC, EXAM, "Profile text.", CLINICAL]);
    for (basic, exam, clinical, lang) in [
        (JA_BASIC, JA_EXAM, JA_CLINICAL, Lang::Ja),
        (KO_BASIC, KO_EXAM, KO_CLINICAL, Lang::Ko),
    ] {
        let w = lang.vocab();
        let p = parse_texts_in(w, &[basic, exam, "Profile.", clinical]);
        assert!(p.code_name.is_some(), "{lang:?}");
        assert_eq!(
            p.gender.as_deref().and_then(|g| ArchiveGender::read(w, g)),
            Some(ArchiveGender::Male)
        );
        assert_eq!(p.combat_years, en.combat_years, "{lang:?}");
        assert_eq!((p.birth_month, p.birth_day), (Some(8), Some(30)));
        assert_eq!(p.height_cm, en.height_cm);
        assert_eq!(p.infection_state, InfectionState::Infected);
        assert_eq!(p.assimilation_pct, en.assimilation_pct);
        assert_eq!(p.blood_density_ul, en.blood_density_ul);
        let steps = |p: &ArchiveProfile| p.exam.iter().map(|r| r.step).collect::<Vec<_>>();
        assert_eq!(steps(&p), steps(&en), "{lang:?}");
        assert!(p.extra.is_empty(), "{lang:?} {:?}", p.extra);
    }
    assert_eq!(Lang::for_server(Server::EN), Some(Lang::En));
    assert_eq!(Lang::for_server(Server::Bilibili), Some(Lang::Cn));
    assert_eq!(Lang::for_server(Server::TW), None);
}

#[test]
fn jp_and_kr_vocabulary() {
    assert_eq!(combat_years(&JA, "二十五年"), Some(25.0));
    assert_eq!(combat_years(&JA, "十二年"), Some(12.0));
    assert_eq!(combat_years(&JA, "七十年"), Some(70.0));
    assert_eq!(combat_years(&JA, "十五年（自称）"), Some(15.0));
    assert_eq!(combat_years(&JA, "一ヶ月"), Some(0.08));
    assert_eq!(combat_years(&JA, "なし（自称）"), Some(0.0));
    assert_eq!(combat_years(&JA, "模擬訓練半年"), Some(0.5));
    assert_eq!(combat_years(&JA, "非公開"), None);
    assert_eq!(combat_years(&KO, "15년(자칭)"), Some(15.0));
    assert_eq!(combat_years(&KO, "6개월"), Some(0.5));
    assert_eq!(combat_years(&KO, "반 년의 비실전 훈련"), Some(0.5));
    assert_eq!(combat_years(&KO, "없음 (본인 주장)"), Some(0.0));
    assert_eq!(combat_years(&KO, "기밀"), None);
    assert_eq!(combat_years(&CN, "五年"), Some(5.0));
    assert_eq!(combat_years(&CN, "两年"), Some(2.0));
    assert_eq!(combat_years(&CN, "四十四年"), Some(44.0));
    assert_eq!(combat_years(&CN, "没有战斗经验（本人声称）"), Some(0.0));
    assert_eq!(combat_years(&CN, "半年非实战训练"), Some(0.5));
    assert_eq!(combat_years(&CN, "未公开"), None);
    assert_eq!(birthday(&KO, "8월22일"), Some((8, 22)));
    assert_eq!(birthday(&JA, "本人は忘れたと主張"), None);
    assert_eq!(height_cm(&JA, "185cm前後"), Some(185.0));
    assert_eq!(number_before("＜1%", JA.percent, JA.bounds), None);
    assert_eq!(number_before("14.5％", JA.percent, JA.bounds), Some(14.5));
    assert_eq!(number_before("0.33u/n", KO.density, KO.bounds), Some(0.33));
    assert_eq!(
        infection_state(&JA, "体表に源石結晶の分布なし。非感染者に認定。"),
        InfectionState::Clear
    );
    assert_eq!(
        infection_state(
            &KO,
            "체표면에 오리지늄 결정이 없으나, 의학 테스트 보고서 참고 결과 감염자로 확인."
        ),
        InfectionState::Infected
    );
    assert_eq!(infection_state(&KO, "미공개"), InfectionState::Unclear);
    assert_eq!(exam_step(&JA, "欠落"), Some(0));
    assert_eq!(exam_step(&KO, "탁월"), Some(4));
    assert_eq!(exam_step(&KO, "표준/우수"), None);
    for (w, word, want) in [
        (&JA, "女性", ArchiveGender::Female),
        (&KO, "여", ArchiveGender::Female),
        (&CN, "男", ArchiveGender::Male),
        (&EN, "Male", ArchiveGender::Male),
    ] {
        assert_eq!(ArchiveGender::read(w, word), Some(want), "{word}");
    }
    assert_eq!(ArchiveGender::read(&CN, "断罪"), None);
    assert_eq!(ArchiveGender::read(&EN, "女"), None);
}
