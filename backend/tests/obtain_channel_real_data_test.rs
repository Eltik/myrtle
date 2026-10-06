//! Obtain channels and localized faction and subclass names on the real trees,
//! with KR and JP channels aligned against EN and CN.
//!
//! Skips when `assets/output/{en,cn,kr}` are absent, which is CI: the CI
//! `game-data` artifact carries EN only.

mod common;

use std::collections::BTreeMap;

use backend::core::gamedata::enrich::reference::align_reference_facts;
use backend::core::gamedata::types::GameData;
use backend::core::gamedata::types::handbook_team::NO_FACTION;
use backend::core::gamedata::types::obtain::{ObtainChannel, SkinChannel};
use backend::core::gamedata::types::operator::Operator;
use common::load_server_with_en_art as load;

fn operator_census(data: &GameData) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for op in data.operators.values() {
        *out.entry(format!("{:?}", op.obtain_channel)).or_default() += 1;
    }
    out
}

fn skin_census(data: &GameData) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for skin in data.skins.char_skins.values() {
        *out.entry(format!("{:?}", skin.obtain_channel)).or_default() += 1;
    }
    out
}

fn count(census: &BTreeMap<String, usize>, key: &str) -> usize {
    census.get(key).copied().unwrap_or(0)
}

#[test]
fn channels_and_names_read_the_same_on_every_server() {
    let (Some(en), Some(mut cn)) = (load("en"), load("cn")) else {
        eprintln!("skipping: assets/output/en or cn missing");
        return;
    };

    // EN and CN classify themselves; the pass must not move either.
    let en_ops = operator_census(&en);
    let en_skins = skin_census(&en);
    eprintln!("en operators {en_ops:?}");
    eprintln!("en skins {en_skins:?}");
    let cn_ops = operator_census(&cn);
    let cn_skins = skin_census(&cn);
    let cn_facts = align_reference_facts(&mut cn, &[&en]);
    eprintln!("cn operators {cn_ops:?}");
    eprintln!("cn skins {cn_skins:?}");
    eprintln!("cn facts from reference {cn_facts:?}");
    assert_eq!(cn_facts.operator_channels, 0);
    assert_eq!(cn_facts.skin_channels, 0);
    assert_eq!(operator_census(&cn), cn_ops);
    assert_eq!(skin_census(&cn), cn_skins);
    // 313 "Recruitment & Headhunting" plus the one with Pinboard Missions.
    assert!(count(&en_ops, "Some(Headhunting)") >= 314);
    assert_eq!(count(&en_skins, "Other"), 0);

    for server in ["kr", "jp"] {
        let Some(mut data) = load(server) else {
            eprintln!("skipping {server}: not on disk");
            continue;
        };
        let ops_before = operator_census(&data);
        let skins_before = skin_census(&data);
        let facts = align_reference_facts(&mut data, &[&en, &cn]);
        let ops = operator_census(&data);
        let skins = skin_census(&data);
        eprintln!("{server} operators before {ops_before:?}");
        eprintln!("{server} operators after {ops:?}");
        eprintln!("{server} skins before {skins_before:?}");
        eprintln!("{server} skins after {skins:?}");
        eprintln!(
            "{server}: operators from reference {}, still Other {}, None {}; skins from reference {}, still Other {}",
            facts.operator_channels,
            count(&ops, "Some(Other)"),
            count(&ops, "None"),
            facts.skin_channels,
            count(&skins, "Other"),
        );

        let op = |id: &str| data.operators[id].obtain_channel;
        assert_eq!(
            op("char_002_amiya"),
            en.operators["char_002_amiya"].obtain_channel
        );
        assert_eq!(op("char_263_skadi"), Some(ObtainChannel::Headhunting));
        assert!(count(&ops, "Some(Headhunting)") >= 314);
        assert!(count(&skins, "Collab") >= 30);
        assert_eq!(
            data.skins
                .char_skins
                .values()
                .filter(|s| s.obtain_channel == SkinChannel::Other)
                .count(),
            count(&skins, "Other"),
        );

        let named =
            |f: &dyn Fn(&Operator) -> bool| data.operators.values().filter(|op| f(op)).count();
        let has_faction =
            |id: Option<&str>| id.is_some_and(|id| !id.is_empty() && id != NO_FACTION);
        eprintln!(
            "{server} names: {} operators, sub_profession_name {}, nation_name {} of {} with a nation id, group_name {} of {}, team_name {} of {}",
            data.operators.len(),
            named(&|op| op.sub_profession_name.is_some()),
            named(&|op| op.nation_name.is_some()),
            named(&|op| has_faction(Some(op.nation_id.as_str()))),
            named(&|op| op.group_name.is_some()),
            named(&|op| has_faction(op.group_id.as_deref())),
            named(&|op| op.team_name.is_some()),
            named(&|op| has_faction(op.team_id.as_deref())),
        );
        assert_eq!(
            named(&|op| op.sub_profession_name.is_some()),
            data.operators.len()
        );
        eprintln!(
            "{server} sample: skadi {:?} / {:?}",
            data.operators["char_263_skadi"].sub_profession_name,
            data.operators["char_263_skadi"].nation_name
        );
    }
}
