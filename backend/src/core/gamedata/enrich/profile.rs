use crate::core::gamedata::types::handbook::{
    BasicInfo, HandbookStoryTextAudio, OperatorBirthPlace, OperatorGender, OperatorProfile,
    OperatorRace, PhysicalExam,
};

pub fn parse_operator_profile(
    story_text_audio: &[HandbookStoryTextAudio],
) -> Option<OperatorProfile> {
    let basic_text = story_text_audio
        .first()?
        .stories
        .first()?
        .story_text
        .as_str();
    let physical_text = story_text_audio
        .get(1)?
        .stories
        .first()?
        .story_text
        .as_str();

    Some(OperatorProfile {
        basic_info: parse_basic_info(basic_text),
        physical_exam: parse_physical_exam(physical_text),
    })
}

fn parse_basic_info(text: &str) -> BasicInfo {
    let mut info = BasicInfo::default();

    for line in text.lines() {
        let Some((key, value)) = parse_bracketed(line) else {
            continue;
        };
        match key {
            // English
            "Code Name" | "代号" => info.code_name = value.to_owned(),
            "Gender" | "性别" => info.gender = parse_gender(value),
            "Combat Experience" | "战斗经验" => info.combat_experience = value.to_owned(),
            "Place of Birth" | "出身地" => info.place_of_birth = parse_birthplace(value),
            "Date of Birth" | "生日" => info.date_of_birth = value.to_owned(),
            "Race" | "种族" => info.race = parse_race(value),
            "Height" | "身高" => info.height = value.to_owned(),
            "Infection Status" | "矿石病感染情况" => {
                info.infection_status = value.to_owned()
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
            "Combat Skill" | "战斗技巧" => exam.combat_skill = value.to_owned(),
            "Originium Arts Assimilation" | "源石技艺适应性" => {
                exam.originium_arts_assimilation = value.to_owned()
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
    // Try CN lookup first (most common in raw data), then fall back to serde for EN
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
        "阿戈尔" | "Aegir" => OperatorBirthPlace::Aegir,
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
        "阿戈尔" | "Aegir" => OperatorRace::Aegir,
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
