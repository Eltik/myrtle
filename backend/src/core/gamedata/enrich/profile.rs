use crate::core::gamedata::types::{
    GameData,
    handbook::{
        BasicInfo, HandbookStoryTextAudio, OperatorBirthPlace, OperatorGender, OperatorProfile,
        OperatorRace, PhysicalExam,
    },
};
use crate::utils::env::switched_off;

pub fn parse_operator_profile(
    story_text_audio: &[HandbookStoryTextAudio],
) -> Option<OperatorProfile> {
    let basic_text = section_text(story_text_audio, 0)?;
    let physical_text = section_text(story_text_audio, 1)?;
    Some(OperatorProfile {
        basic_info: parse_basic_info(basic_text),
        physical_exam: parse_physical_exam(physical_text),
    })
}

/// Re-read every profile in `target` whose labels the parser does not know,
/// by position against the same operator's text in a reference server.
///
/// The parser matches EN and CN labels only, so a KR or JP profile parsed to
/// all defaults: KR `char_377_gdglow` read gender and race `Unknown` against
/// EN's `Female` and `Feline` (2026-10-06). The handbook lines come in the
/// same order on every server (`[Code Name] [Gender] ...` in EN is
/// `[코드네임] [성별] ...` in KR and `【コードネーム】【性別】...` in JP), so the
/// n-th bracketed line takes the reference's n-th label and is then parsed as
/// usual. That keeps the target's own wording for the free-text fields
/// (code name, height) without a label table per language.
///
/// `references` are tried in order, and only one whose own profile for the
/// operator parsed counts. Gender, race and place of birth each come from the
/// first such reference that does not read `Unknown` for that field: those
/// enums are mapped from the value's wording, which is as language-bound as
/// the label, and they are facts rather than text. The first one whose
/// bracketed lines number the same as the target's supplies the labels. EN alone matched 410 of 412 KR basic-info sections; Skadi (EN
/// drops the `[Weight]` line the other three servers carry) and Wind Chimes
/// (EN repeats `[Infection Status]`) match CN line for line, so passing CN
/// second closes both.
///
/// Left alone: an operator no reference parsed (the robots, whose labels are
/// a different schema on every server), and a profile the parser already read
/// (any server whose code name parsed), so EN and CN are unchanged by
/// construction. `PROFILE_ALIGN=0` turns the pass off.
///
/// Returns how many profiles were re-read.
pub fn align_profiles(target: &mut GameData, references: &[&GameData]) -> usize {
    if switched_off("PROFILE_ALIGN") {
        return 0;
    }
    let mut aligned = 0;
    for (id, op) in &mut target.operators {
        if op
            .profile
            .as_ref()
            .is_some_and(|p| !p.basic_info.code_name.is_empty())
        {
            continue;
        }
        let target_text = &op.handbook.story_text_audio;
        if section_text(target_text, 0).is_none() {
            continue;
        }
        let parsed: Vec<_> = references
            .iter()
            .filter_map(|reference| {
                let reference_op = reference.operators.get(id)?;
                let profile = reference_op.profile.as_ref()?;
                (!profile.basic_info.code_name.is_empty())
                    .then_some((&reference_op.handbook.story_text_audio, profile))
            })
            .collect();
        if parsed.is_empty() {
            continue;
        }
        let relabelled = |section: usize| {
            let text = section_text(target_text, section)?;
            parsed
                .iter()
                .find_map(|(reference, _)| relabel(text, section_text(reference, section)?))
        };

        let mut basic_info = relabelled(0)
            .map(|text| parse_basic_info(&text))
            .unwrap_or_default();
        // Per field, the first reference that knows the value: EN writes
        // Skadi's race as the literal "Unknown" where CN says 未公开.
        let fact = |known: &dyn Fn(&BasicInfo) -> bool| {
            parsed
                .iter()
                .map(|(_, p)| &p.basic_info)
                .find(|b| known(b))
                .unwrap_or(&parsed[0].1.basic_info)
        };
        basic_info.gender = fact(&|b| b.gender != OperatorGender::Unknown)
            .gender
            .clone();
        basic_info.race = fact(&|b| b.race != OperatorRace::Unknown).race.clone();
        basic_info.place_of_birth = fact(&|b| b.place_of_birth != OperatorBirthPlace::Unknown)
            .place_of_birth
            .clone();
        op.profile = Some(OperatorProfile {
            basic_info,
            physical_exam: relabelled(1)
                .map(|text| parse_physical_exam(&text))
                .unwrap_or_default(),
        });
        aligned += 1;
    }
    aligned
}

fn section_text(story_text_audio: &[HandbookStoryTextAudio], section: usize) -> Option<&str> {
    Some(
        story_text_audio
            .get(section)?
            .stories
            .first()?
            .story_text
            .as_str(),
    )
}

/// `target` with each bracketed line's label replaced by the label of the
/// reference's bracketed line at the same position. `None` when the two
/// disagree on how many bracketed lines there are, which is the one signal
/// that the positions do not correspond.
fn relabel(target: &str, reference: &str) -> Option<String> {
    let mut labels = reference
        .lines()
        .filter_map(|line| parse_bracketed(line).map(|(key, _)| key));
    let mut out = String::with_capacity(target.len());
    for line in target.lines() {
        match parse_bracketed(line) {
            Some((_, value)) => {
                let label = labels.next()?;
                out.push('[');
                out.push_str(label);
                out.push_str("] ");
                out.push_str(value);
            }
            None => out.push_str(line),
        }
        out.push('\n');
    }
    labels.next().is_none().then_some(out)
}

fn parse_basic_info(text: &str) -> BasicInfo {
    let mut info = BasicInfo::default();

    for line in text.lines() {
        let Some((key, value)) = parse_bracketed(line) else {
            continue;
        };
        match key {
            "Code Name" | "Codename" | "代号" => info.code_name = value.to_owned(),
            "Gender" | "性别" => info.gender = parse_gender(value),
            "Combat Experience" | "战斗经验" => info.combat_experience = value.to_owned(),
            "Place of Birth" | "出身地" => info.place_of_birth = parse_birthplace(value),
            "Date of Birth" | "生日" => info.date_of_birth = value.to_owned(),
            "Race" | "种族" => info.race = parse_race(value),
            "Height" | "身高" => info.height = value.to_owned(),
            "Infection Status" | "矿石病感染情况" => {
                info.infection_status = value.to_owned();
            }
            _ => {}
        }
    }
    info
}

fn parse_physical_exam(text: &str) -> PhysicalExam {
    let mut exam = PhysicalExam::default();

    for line in text.lines() {
        let Some((key, value)) = parse_bracketed(line) else {
            continue;
        };
        match key {
            "Physical Strength" | "物理强度" => exam.physical_strength = value.to_owned(),
            "Mobility" | "战场机动" => exam.mobility = value.to_owned(),
            "Physical Resilience" | "生理耐受" => exam.physical_resilience = value.to_owned(),
            "Tactical Acumen" | "战术规划" => exam.tactical_acumen = value.to_owned(),
            "Combat Skill" | "Combat Skills" | "战斗技巧" => {
                exam.combat_skill = value.to_owned();
            }
            // Both misspellings are EN's own, on 3 operators (2026-10-06).
            "Originium Arts Assimilation"
            | "Originium Arts Assilimation"
            | "Originium ArtsAssimilation"
            | "源石技艺适应性" => {
                exam.originium_arts_assimilation = value.to_owned();
            }
            _ => {}
        }
    }
    exam
}

fn parse_bracketed(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.starts_with('[') {
        let close = line.find(']')?;
        Some((&line[1..close], line[close + 1..].trim()))
    } else if line.starts_with('【') {
        let close = line.find('】')?;
        let open_len = '【'.len_utf8();
        let close_len = '】'.len_utf8();
        Some((&line[open_len..close], line[close + close_len..].trim()))
    } else {
        None
    }
}

fn parse_gender(s: &str) -> OperatorGender {
    match s {
        "Female" | "女" => OperatorGender::Female,
        "Male" | "男" => OperatorGender::Male,
        "Male]" | "男]" => OperatorGender::MaleBugged,
        "Conviction" | "断罪" => OperatorGender::Conviction,
        _ => OperatorGender::Unknown,
    }
}

fn parse_birthplace(s: &str) -> OperatorBirthPlace {
    match s {
        "未公开" | "Undisclosed" => OperatorBirthPlace::Undisclosed,
        "东国" | "東国" | "Higashi" => OperatorBirthPlace::Higashi,
        "卡西米尔" | "Kazimierz" => OperatorBirthPlace::Kazimierz,
        "维多利亚" | "Victoria" => OperatorBirthPlace::Victoria,
        "雷姆必拓" | "Rim Billiton" => OperatorBirthPlace::RimBilliton,
        "莱塔尼亚" | "Leithanien" => OperatorBirthPlace::Leithanien,
        "玻利瓦尔" | "Bolívar" | "Bolivar" => OperatorBirthPlace::Bolivar,
        "萨尔贡" | "Sargon" => OperatorBirthPlace::Sargon,
        "谢拉格" | "Kjerag" => OperatorBirthPlace::Kjerag,
        "哥伦比亚" | "Columbia" => OperatorBirthPlace::Columbia,
        "萨米" | "Sami" => OperatorBirthPlace::Sami,
        "伊比利亚" | "Iberia" => OperatorBirthPlace::Iberia,
        "卡兹戴尔" | "Kazdel" => OperatorBirthPlace::Kazdel,
        "米诺斯" | "Minos" => OperatorBirthPlace::Minos,
        "龙门" | "Lungmen" => OperatorBirthPlace::Lungmen,
        "叙拉古" | "Siracusa" => OperatorBirthPlace::Siracusa,
        "炎国" | "炎" | "Yan" => OperatorBirthPlace::Yan,
        "乌萨斯" | "Ursus" => OperatorBirthPlace::Ursus,
        "汐斯塔" | "Siesta" => OperatorBirthPlace::Siesta,
        "阿戈尔" | "Aegir" | "Ægir" => OperatorBirthPlace::Aegir,
        "杜林" | "Durin" => OperatorBirthPlace::Durin,
        "拉特兰" | "Laterano" => OperatorBirthPlace::Laterano,
        "沃尔珀" | "Vouivre" => OperatorBirthPlace::Vouivre,
        "罗德岛" | "Rhodes Island" => OperatorBirthPlace::RhodesIsland,
        "远东" | "Far East" => OperatorBirthPlace::FarEast,
        _ => OperatorBirthPlace::Unknown,
    }
}

fn parse_race(s: &str) -> OperatorRace {
    match s {
        "未公开" | "Undisclosed" => OperatorRace::Undisclosed,
        "札拉克" | "Zalak" => OperatorRace::Zalak,
        "鬼" | "Oni" => OperatorRace::Oni,
        "萨弗拉" | "Savra" => OperatorRace::Savra,
        "杜林" | "Durin" => OperatorRace::Durin,
        "库兰塔" | "Kuranta" => OperatorRace::Kuranta,
        "沃尔珀" | "Vouivre" => OperatorRace::Vouivre,
        "黎博利" | "Liberi" => OperatorRace::Liberi,
        "菲林" | "Feline" => OperatorRace::Feline,
        "卡特斯" | "Cautus" => OperatorRace::Cautus,
        "佩洛" | "Perro" => OperatorRace::Perro,
        "雷普罗巴" | "Reproba" => OperatorRace::Reproba,
        "萨科塔" | "Sankta" => OperatorRace::Sankta,
        "萨卡兹" | "Sarkaz" => OperatorRace::Sarkaz,
        "瓦伊凡" | "Vulpo" => OperatorRace::Vulpo,
        "依拉菲亚" | "Elafia" => OperatorRace::Elafia,
        "斐迪亚" | "Phidia" => OperatorRace::Phidia,
        "阿戈尔" | "Aegir" | "Ægir" => OperatorRace::Aegir,
        "阿纳缇" | "Anaty" => OperatorRace::Anaty,
        "依特拉" | "Itra" => OperatorRace::Itra,
        "古龙" | "Archosauria" => OperatorRace::Archosauria,
        "鲁珀" | "Lupo" => OperatorRace::Lupo,
        "菲亚特" | "Forte" => OperatorRace::Forte,
        "乌萨斯" | "Ursus" => OperatorRace::Ursus,
        "佩特拉姆" | "Petram" => OperatorRace::Petram,
        "角峰" | "Cerato" => OperatorRace::Cerato,
        "卡普里尼" | "Caprinae" => OperatorRace::Caprinae,
        "德拉克" | "Draco" => OperatorRace::Draco,
        "阿努拉" | "Anura" => OperatorRace::Anura,
        "阿纳萨" | "Anasa" => OperatorRace::Anasa,
        "卡特斯/奇美拉" | "Cautus/Chimera" => OperatorRace::CautusChimera,
        "麒麟" | "Kylin" => OperatorRace::Kylin,
        "披毛" | "Pilosa" => OperatorRace::Pilosa,
        "曼提柯" | "Manticore" => OperatorRace::Manticore,
        "龙" | "Lung" => OperatorRace::Lung,
        "阿斯兰" | "Aslan" => OperatorRace::Aslan,
        "精灵" | "Elf" => OperatorRace::Elf,
        _ => OperatorRace::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
