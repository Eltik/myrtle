use crate::core::local::types::handbook::{
    BasicInfo, HandbookStoryTextAudio, OperatorBirthPlace, OperatorGender, OperatorProfile,
    OperatorRace, PhysicalExam,
};

pub fn parse_operator_profile(
    story_text_audio: &[HandbookStoryTextAudio],
) -> Option<OperatorProfile> {
    if story_text_audio.len() < 2 {
        return None;
    }

    let basic_text = story_text_audio.get(0)?.stories.get(0)?.story_text.as_str();

    let physical_text = story_text_audio.get(1)?.stories.get(0)?.story_text.as_str();

    let basic_info = parse_basic_info(basic_text);
    let physical_exam = parse_physical_exam(physical_text);

    Some(OperatorProfile {
        basic_info,
        physical_exam,
    })
}

fn parse_basic_info(text: &str) -> BasicInfo {
    let mut info = BasicInfo::default();

    for line in text.lines() {
        // Try English format first: [Key] value
        if let Some((key, value)) = parse_bracketed_line(line, '[', ']') {
            match key {
                "Code Name" => info.code_name = value.to_string(),
                "Gender" => info.gender = parse_gender(value),
                "Combat Experience" => info.combat_experience = value.to_string(),
                "Place of Birth" => info.place_of_birth = parse_birthplace(value),
                "Date of Birth" => info.date_of_birth = value.to_string(),
                "Race" => info.race = parse_race(value),
                "Height" => info.height = value.to_string(),
                "Infection Status" => info.infection_status = value.to_string(),
                _ => {}
            }
        }
        // Try Chinese format: 【Key】value
        else if let Some((key, value)) = parse_bracketed_line(line, '【', '】') {
            match key {
                "代号" => info.code_name = value.to_string(),
                "性别" => info.gender = parse_gender_cn(value),
                "战斗经验" => info.combat_experience = value.to_string(),
                "出身地" => info.place_of_birth = parse_birthplace_cn(value),
                "生日" => info.date_of_birth = value.to_string(),
                "种族" => info.race = parse_race_cn(value),
                "身高" => info.height = value.to_string(),
                "矿石病感染情况" => info.infection_status = value.to_string(),
                _ => {}
            }
        }
    }

    info
}

fn parse_physical_exam(text: &str) -> PhysicalExam {
    let mut exam = PhysicalExam::default();

    for line in text.lines() {
        // Try English format first: [Key] value
        if let Some((key, value)) = parse_bracketed_line(line, '[', ']') {
            match key {
                "Physical Strength" => exam.physical_strength = value.to_string(),
                "Mobility" => exam.mobility = value.to_string(),
                "Physical Resilience" => exam.physical_resilience = value.to_string(),
                "Tactical Acumen" => exam.tactical_acumen = value.to_string(),
                "Combat Skill" => exam.combat_skill = value.to_string(),
                "Originium Arts Assimilation" => {
                    exam.originium_arts_assimilation = value.to_string()
                }
                _ => {}
            }
        }
        // Try Chinese format: 【Key】value
        else if let Some((key, value)) = parse_bracketed_line(line, '【', '】') {
            match key {
                "物理强度" => exam.physical_strength = value.to_string(),
                "战场机动" => exam.mobility = value.to_string(),
                "生理耐受" => exam.physical_resilience = value.to_string(),
                "战术规划" => exam.tactical_acumen = value.to_string(),
                "战斗技巧" => exam.combat_skill = value.to_string(),
                "源石技艺适应性" => exam.originium_arts_assimilation = value.to_string(),
                _ => {}
            }
        }
    }

    exam
}

/// Parse a line with bracket format: <open>key<close>value
fn parse_bracketed_line(line: &str, open: char, close: char) -> Option<(&str, &str)> {
    let line = line.trim();
    if !line.starts_with(open) {
        return None;
    }

    let close_pos = line.find(close)?;
    let key = &line[open.len_utf8()..close_pos];
    let value = line[close_pos + close.len_utf8()..].trim();

    Some((key, value))
}

// ============================================================================
// English Parsers
// ============================================================================

fn parse_gender(s: &str) -> OperatorGender {
    match s {
        "Female" => OperatorGender::Female,
        "Male" => OperatorGender::Male,
        "Male]" => OperatorGender::MaleBugged,
        "Conviction" => OperatorGender::Conviction,
        _ => OperatorGender::Unknown,
    }
}

fn parse_birthplace(s: &str) -> OperatorBirthPlace {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .unwrap_or(OperatorBirthPlace::Unknown)
}

fn parse_race(s: &str) -> OperatorRace {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .unwrap_or(OperatorRace::Unknown)
}

// ============================================================================
// Chinese Parsers
// ============================================================================

fn parse_gender_cn(s: &str) -> OperatorGender {
    match s {
        "女" => OperatorGender::Female,
        "男" => OperatorGender::Male,
        "男]" => OperatorGender::MaleBugged,
        "断罪" => OperatorGender::Conviction, // Conviction's special gender
        _ => OperatorGender::Unknown,
    }
}

fn parse_birthplace_cn(s: &str) -> OperatorBirthPlace {
    match s {
        "未公开" => OperatorBirthPlace::Undisclosed,
        "东国" | "東国" => OperatorBirthPlace::Higashi,
        "卡西米尔" => OperatorBirthPlace::Kazimierz,
        "维多利亚" => OperatorBirthPlace::Victoria,
        "雷姆必拓" => OperatorBirthPlace::RimBilliton,
        "莱塔尼亚" => OperatorBirthPlace::Leithanien,
        "玻利瓦尔" => OperatorBirthPlace::Bolivar,
        "萨尔贡" => OperatorBirthPlace::Sargon,
        "谢拉格" => OperatorBirthPlace::Kjerag,
        "哥伦比亚" => OperatorBirthPlace::Columbia,
        "萨米" => OperatorBirthPlace::Sami,
        "伊比利亚" => OperatorBirthPlace::Iberia,
        "卡兹戴尔" => OperatorBirthPlace::Kazdel,
        "米诺斯" => OperatorBirthPlace::Minos,
        "龙门" => OperatorBirthPlace::Lungmen,
        "叙拉古" => OperatorBirthPlace::Siracusa,
        "炎国" | "炎" => OperatorBirthPlace::Yan,
        "乌萨斯" => OperatorBirthPlace::Ursus,
        "汐斯塔" => OperatorBirthPlace::Siesta,
        "阿戈尔" => OperatorBirthPlace::Aegir,
        "杜林" => OperatorBirthPlace::Durin,
        "拉特兰" => OperatorBirthPlace::Laterano,
        "沃尔珀" => OperatorBirthPlace::Vouivre,
        "罗德岛" => OperatorBirthPlace::RhodesIsland,
        "远东" => OperatorBirthPlace::FarEast,
        _ => OperatorBirthPlace::Unknown,
    }
}

fn parse_race_cn(s: &str) -> OperatorRace {
    match s {
        "未公开" => OperatorRace::Undisclosed,
        "札拉克" => OperatorRace::Zalak,
        "鬼" => OperatorRace::Oni,
        "萨弗拉" => OperatorRace::Savra,
        "杜林" => OperatorRace::Durin,
        "库兰塔" => OperatorRace::Kuranta,
        "沃尔珀" => OperatorRace::Vouivre,
        "黎博利" => OperatorRace::Liberi,
        "菲林" => OperatorRace::Feline,
        "卡特斯" => OperatorRace::Cautus,
        "佩洛" => OperatorRace::Perro,
        "雷普罗巴" => OperatorRace::Reproba,
        "萨科塔" => OperatorRace::Sankta,
        "萨卡兹" => OperatorRace::Sarkaz,
        "瓦伊凡" => OperatorRace::Vulpo,
        "依拉菲亚" => OperatorRace::Elafia,
        "斐迪亚" => OperatorRace::Phidia,
        "阿戈尔" => OperatorRace::Aegir,
        "阿纳缇" => OperatorRace::Anaty,
        "依特拉" => OperatorRace::Itra,
        "古龙" => OperatorRace::Archosauria,
        "鲁珀" => OperatorRace::Lupo,
        "菲亚特" => OperatorRace::Forte,
        "乌萨斯" => OperatorRace::Ursus,
        "佩特拉姆" => OperatorRace::Petram,
        "角峰" => OperatorRace::Cerato,
        "卡普里尼" => OperatorRace::Caprinae,
        "德拉克" => OperatorRace::Draco,
        "阿努拉" => OperatorRace::Anura,
        "阿纳萨" => OperatorRace::Anasa,
        "卡特斯/奇美拉" => OperatorRace::CautusChimera,
        "麒麟" => OperatorRace::Kylin,
        "披毛" => OperatorRace::Pilosa,
        "曼提柯" => OperatorRace::Manticore,
        "龙" => OperatorRace::Lung,
        "阿斯兰" => OperatorRace::Aslan,
        "精灵" => OperatorRace::Elf,
        _ => OperatorRace::Unknown,
    }
}
