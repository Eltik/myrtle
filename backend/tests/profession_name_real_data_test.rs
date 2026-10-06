//! Class names in each server's own wording, read off the gacha class tags,
//! on the real trees.
//!
//! Skips a server whose `assets/output/<server>` is absent, which on CI is
//! every server but EN.

mod common;

use backend::core::gamedata::types::gacha::PROFESSION_TAG_IDS;
use common::load_server_with_en_art as load;

#[test]
fn every_playable_operator_gets_its_servers_class_name() {
    for server in ["en", "cn", "kr", "jp"] {
        let Some(data) = load(server) else {
            eprintln!("skipping {server}: not on disk");
            continue;
        };
        let playable = |code: &str| PROFESSION_TAG_IDS.iter().any(|(c, _)| *c == code);
        let classed = data
            .operators
            .values()
            .filter(|op| playable(op.profession.to_raw_str()))
            .count();
        let named = data
            .operators
            .values()
            .filter(|op| op.profession_name.is_some())
            .count();
        let names: Vec<&str> = PROFESSION_TAG_IDS
            .iter()
            .map(|(code, _)| {
                data.profession_names
                    .get(*code)
                    .map_or("<missing>", String::as_str)
            })
            .collect();
        eprintln!(
            "{server}: {} operators, {classed} in a playable class, {named} with profession_name; {names:?}",
            data.operators.len()
        );

        // All 8 ids are on every server checked (2026-10-06), so every
        // operator in a playable class is named, and nothing else is.
        assert_eq!(data.profession_names.len(), PROFESSION_TAG_IDS.len());
        assert_eq!(named, classed);
        assert!(names.iter().all(|name| !name.is_empty()));
        if server == "en" {
            assert_eq!(
                names,
                [
                    "Guard",
                    "Sniper",
                    "Defender",
                    "Medic",
                    "Supporter",
                    "Caster",
                    "Specialist",
                    "Vanguard"
                ]
            );
        }
    }
}
