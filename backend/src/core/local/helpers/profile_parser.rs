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
        let parts: Vec<&str> = line.splitn(2, ']').collect();
        if parts.len() != 2 {
            continue;
        }

        let key = parts[0];
        let value = parts[1].trim();

        match key {
            "[Code Name" => info.code_name = value.to_string(),
            "[Gender" => info.gender = parse_gender(value),
            "[Combat Experience" => info.combat_experience = value.to_string(),
            "[Place of Birth" => info.place_of_birth = parse_birthplace(value),
            "[Date of Birth" => info.date_of_birth = value.to_string(),
            "[Race" => info.race = parse_race(value),
            "[Height" => info.height = value.to_string(),
            "[Infection Status" => info.infection_status = value.to_string(),
            _ => {}
        }
    }

    info
}

fn parse_physical_exam(text: &str) -> PhysicalExam {
    let mut exam = PhysicalExam::default();

    for line in text.lines() {
        let parts: Vec<&str> = line.splitn(2, ']').collect();
        if parts.len() != 2 {
            continue;
        }

        let key = parts[0];
        let value = parts[1].trim().to_string();

        match key {
            "[Physical Strength" => exam.physical_strength = value,
            "[Mobility" => exam.mobility = value,
            "[Physical Resilience" => exam.physical_resilience = value,
            "[Tactical Acumen" => exam.tactical_acumen = value,
            "[Combat Skill" => exam.combat_skill = value,
            "[Originium Arts Assimilation" => exam.originium_arts_assimilation = value,
            _ => {}
        }
    }

    exam
}

// TODO: use serde for full implementation
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
    // TODO: Use serde_json for parsing
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .unwrap_or(OperatorBirthPlace::Unknown)
}

fn parse_race(s: &str) -> OperatorRace {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .unwrap_or(OperatorRace::Unknown)
}
