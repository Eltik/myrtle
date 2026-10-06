//! KR and JP operator profiles, aligned against EN and CN, on the real trees.
//!
//! Skips when `assets/output/{kr,cn}` are absent, which is CI: the CI
//! `game-data` artifact carries EN only.

mod common;

use backend::core::gamedata::enrich::profile::align_profiles;
use backend::core::gamedata::types::GameData;
use backend::core::gamedata::types::handbook::{OperatorBirthPlace, OperatorGender, OperatorRace};
use common::load_server_with_en_art as load;

fn unknown_gender(data: &GameData) -> usize {
    data.operators
        .values()
        .filter_map(|op| op.profile.as_ref())
        .filter(|p| p.basic_info.gender == OperatorGender::Unknown)
        .count()
}

#[test]
fn kr_and_jp_profiles_read_like_en() {
    let (Some(en), Some(cn)) = (load("en"), load("cn")) else {
        eprintln!("skipping: assets/output/en or cn missing");
        return;
    };
    let empty_code_name = en
        .operators
        .values()
        .filter_map(|op| op.profile.as_ref())
        .filter(|p| p.basic_info.code_name.is_empty())
        .count();
    eprintln!(
        "en: unknown gender {}, empty code name {empty_code_name}",
        unknown_gender(&en)
    );

    for server in ["kr", "jp"] {
        let Some(mut data) = load(server) else {
            eprintln!("skipping {server}: not on disk");
            continue;
        };
        let before = unknown_gender(&data);
        let aligned = align_profiles(&mut data, &[&en, &cn]);
        eprintln!(
            "{server}: aligned {aligned}, unknown gender {before} -> {}",
            unknown_gender(&data)
        );

        let profile = |id: &str| data.operators[id].profile.clone().expect("profile");
        let gdglow = profile("char_377_gdglow");
        assert_eq!(gdglow.basic_info.gender, OperatorGender::Female);
        assert_eq!(gdglow.basic_info.race, OperatorRace::Feline);

        // EN's text for these two does not line up with KR/JP; CN's does.
        let skadi = profile("char_263_skadi");
        let chimes = profile("char_4083_chimes");
        eprintln!("{server}: skadi {:?}", skadi.basic_info);
        eprintln!("{server}: chimes {:?}", chimes.basic_info);
        assert!(!skadi.basic_info.code_name.is_empty());
        assert!(!skadi.basic_info.height.is_empty());
        // EN spells it `Ægir`; EN's race is the literal "Unknown", CN's 未公开.
        assert_eq!(skadi.basic_info.place_of_birth, OperatorBirthPlace::Aegir);
        assert_eq!(skadi.basic_info.race, OperatorRace::Undisclosed);
        assert!(!chimes.basic_info.code_name.is_empty());
        assert!(!chimes.basic_info.date_of_birth.is_empty());
        assert_eq!(chimes.basic_info.gender, OperatorGender::Female);
    }
}
