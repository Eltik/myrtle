#![allow(clippy::unwrap_used)]

use myrtle_engine_music::{
    MusicIndex, MusicPlaceKind, MusicRole, MusicSources, MusicTables, build_music_index,
};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_gamedata_raw::tables::audio::{MusicBanks, RawAudioData, RawBgmBank, RawMusic};
use serde_json::{Value, json};

fn model(bank: &str) -> GameModel {
    GameModel {
        music: MusicBanks::from_raw(&RawAudioData {
            bgm_banks: vec![RawBgmBank {
                name: bank.to_owned(),
                intro: None,
                loop_: Some("Audio/Sound_Beta_2/Music/obt/m_a_loop".to_owned()),
            }],
            musics: vec![RawMusic {
                id: "music_a".to_owned(),
                name: String::new(),
                bank: bank.to_owned(),
            }],
            ..RawAudioData::default()
        }),
        ..GameModel::default()
    }
}

fn index(gd: &GameModel, json: &[(&str, Value)]) -> (MusicIndex, Vec<String>) {
    let (tables, warnings) = MusicTables::parse(|name| {
        json.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| serde_json::to_vec(v).unwrap())
    });
    let src = MusicSources {
        tables,
        ..MusicSources::default()
    };
    (build_music_index(gd, &src), warnings)
}

#[test]
fn a_table_that_is_missing_or_broken_reads_empty() {
    let mut gd = model("sys.ON_ACTIVITY_LOADED.act45side");
    gd.activities.insert(
        "act45side".to_owned(),
        ActivityBasicInfo {
            id: "act45side".to_owned(),
            name: "Act 45".to_owned(),
            ..ActivityBasicInfo::default()
        },
    );
    let (tables, warnings) = MusicTables::parse(|name| match name {
        "crisis_table" => Some(b"{not json".to_vec()),
        "activity_table" => Some(
            serde_json::to_vec(&json!({"Activity": {
                "TYPE_ACT45SIDE": [{"key": "act45side", "value": {
                    "ConstData": {"EntryMusicId": "music_a"}, "MusicDataMap": {"odd": 1}}}],
                "TYPE_OTHER": [{"key": "act1", "value": {"ConstData": [1, 2]}}]}}))
            .unwrap(),
        ),
        _ => None,
    });
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].starts_with("crisis_table: "));
    let src = MusicSources {
        tables,
        ..MusicSources::default()
    };
    let index = build_music_index(&gd, &src);
    let places: Vec<_> = index.tracks[0]
        .places
        .iter()
        .map(|p| (p.kind, p.name.as_deref(), p.role))
        .collect();
    assert_eq!(
        places,
        [(MusicPlaceKind::Event, Some("Act 45"), Some(MusicRole::Menu))]
    );
    assert_eq!(index.tracks[0].origin.as_deref(), Some("Act 45"));
}

#[test]
fn a_plain_object_table_reads_in_key_order() {
    let gd = model("sys.ON_MUSIC.box");
    let furnitures: serde_json::Map<String, Value> = (0..32)
        .rev()
        .map(|i| {
            (
                format!("k{i:02}"),
                json!({"Name": format!("F{i:02}"), "MusicId": "music_a"}),
            )
        })
        .collect();
    let (index, warnings) = index(
        &gd,
        &[(
            "building_data",
            json!({"CustomData": {"Furnitures": furnitures}}),
        )],
    );
    assert!(warnings.is_empty());
    let names: Vec<String> = index.tracks[0]
        .places
        .iter()
        .map(|p| p.name.clone().unwrap())
        .collect();
    let want: Vec<String> = (0..32).map(|i| format!("F{i:02}")).collect();
    assert_eq!(names, want);
}
