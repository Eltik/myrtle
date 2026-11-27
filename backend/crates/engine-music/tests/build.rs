#![allow(clippy::unwrap_used)]

use myrtle_engine_music::{
    ClipTree, MusicIndex, MusicKind, MusicPlaceItem, MusicPlaceKind, MusicRole, MusicSources,
    MusicTables, StoryScript, build_music_index, level_ids,
};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata::model::stage_index::StageIndexEntry;
use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_gamedata_raw::tables::audio::{MusicBanks, RawAudioData, RawBgmBank, RawMusic};
use myrtle_gamedata_raw::tables::roguelike::RoguelikeThemeGameData;
use myrtle_gamedata_raw::tables::stage::StageDifficulty;
use myrtle_gamedata_raw::tables::story::{StoryReviewGroup, StoryUnlockData, StoryVariables};
use myrtle_gamedata_raw::tables::zone::Zone;
use serde_json::{Value, json};

fn audio(banks: &[(&str, Option<&str>, &str)], musics: &[(&str, &str)]) -> MusicBanks {
    MusicBanks::from_raw(&RawAudioData {
        bgm_banks: banks
            .iter()
            .map(|(name, intro, loop_)| RawBgmBank {
                name: (*name).to_owned(),
                intro: intro.map(str::to_owned),
                loop_: Some((*loop_).to_owned()),
            })
            .collect(),
        musics: musics
            .iter()
            .map(|(id, bank)| RawMusic {
                id: (*id).to_owned(),
                name: String::new(),
                bank: (*bank).to_owned(),
            })
            .collect(),
        ..RawAudioData::default()
    })
}

fn tables(json: &[(&str, Value)]) -> MusicTables {
    let (tables, warnings) = MusicTables::parse(|name| {
        json.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| serde_json::to_vec(v).unwrap())
    });
    assert!(warnings.is_empty(), "{warnings:?}");
    tables
}

fn kinds_and_roles(index: &MusicIndex, id: &str) -> (Vec<MusicKind>, Vec<Option<MusicRole>>) {
    let t = index.tracks.iter().find(|t| t.id == id).unwrap();
    (t.kinds.clone(), t.places.iter().map(|p| p.role).collect())
}

#[test]
fn a_loop_keys_the_track_and_silence_is_not_one() {
    let gd = GameModel {
        music: audio(
            &[
                (
                    "battle.ON_GAME_READY.a",
                    None,
                    "Audio/Sound_Beta_2/Music/act27side/m_bat_act27side_1_loop",
                ),
                (
                    "sys.ON_MUSIC.silence",
                    None,
                    "Audio/Sound_Beta_2/Music/obt/empty",
                ),
                (
                    "sys.ON_MUSIC.climax",
                    Some("Audio/Sound_Beta_2/Music/obt/empty"),
                    "Audio/Sound_Beta_2/Music/obt/m_sys_ddd_climax",
                ),
            ],
            &[],
        ),
        ..GameModel::default()
    };
    let src = MusicSources {
        scripts: vec![script(
            "obt/x.txt.txt",
            b"[PlayMusic(key=\"Sound_Beta_2/Music/act27side/m_avg_SiestaCity\")]",
        )],
        clip_trees: vec![tree(None, &["obt/empty.ogg", "obt/m_sys_ddd_climax.ogg"])],
        ..MusicSources::default()
    };
    let index = build_music_index(&gd, &src);
    let ids: Vec<&str> = index.tracks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(
        ids,
        ["m_avg_siestacity", "m_bat_act27side_1", "m_sys_ddd_climax"]
    );
    let climax = &index.tracks[2];
    assert!(climax.loop_url.is_some());
    assert_eq!(climax.intro_url, None);
}

#[test]
fn the_folder_drops_bundle_revisions() {
    let mut gd = GameModel {
        music: audio(
            &[
                (
                    "battle.ON_GAME_READY.a",
                    None,
                    "Audio/Sound_Beta_2/Music/act18d5d0/m_sys/m_sys_ccs5_loop",
                ),
                (
                    "battle.ON_GAME_READY.b",
                    None,
                    "Audio/Sound_Beta_2/Music/m_loose_loop",
                ),
            ],
            &[],
        ),
        ..GameModel::default()
    };
    activity(&mut gd, "act18d5", "Code of Brawl", 1);
    let index = build_music_index(&gd, &MusicSources::default());
    let origins: Vec<_> = index
        .tracks
        .iter()
        .map(|t| (t.id.as_str(), t.origin.as_deref()))
        .collect();
    assert_eq!(
        origins,
        [("m_loose", None), ("m_sys_ccs5", Some("Code of Brawl"))]
    );
}

#[test]
fn map_signals_and_roles() {
    let mut gd = GameModel {
        music: audio(
            &[
                (
                    "sys.ON_ROGUELIKE_MAPBOSS.rogue_4",
                    None,
                    "Audio/Sound_Beta_2/Music/obt/m_a_loop",
                ),
                (
                    "sys.ON_ROGUELIKE_SHOP.rogue_1",
                    None,
                    "Audio/Sound_Beta_2/Music/obt/m_b_loop",
                ),
                (
                    "sys.ON_SANDBOX_DUNGEONMAP.sandbox_1",
                    None,
                    "Audio/Sound_Beta_2/Music/obt/m_c_loop",
                ),
                (
                    "battle.ON_CUSTOM_TRIGGER.rl5boss4_2",
                    None,
                    "Audio/Sound_Beta_2/Music/obt/m_d_loop",
                ),
                (
                    "sys.ON_ROGUELIKE_SECRET_EVENT.rogue_1",
                    None,
                    "Audio/Sound_Beta_2/Music/obt/m_e_loop",
                ),
            ],
            &[],
        ),
        ..GameModel::default()
    };
    for (id, name) in [("rogue_1", "Phantom"), ("rogue_4", "Sui")] {
        gd.roguelike.themes.insert(
            id.to_owned(),
            RoguelikeThemeGameData {
                theme_id: id.to_owned(),
                theme_name: name.to_owned(),
                ..RoguelikeThemeGameData::default()
            },
        );
    }
    let src = MusicSources {
        tables: tables(&[
            (
                "roguelike_topic_table",
                json!({"Details": [{"key": "rogue_4", "value": {"Zones": [
                    {"key": "z6", "value": {"Name": "Floor 6", "BgmSignal": "ON_ROGUELIKE_MAPBOSS"}}
                ]}}]}),
            ),
            (
                "sandbox_perm_table",
                json!({"BasicInfo": [{"key": "sandbox_1",
                    "value": {"TopicId": "sandbox_1", "TopicName": "Sand"}}]}),
            ),
        ]),
        ..MusicSources::default()
    };
    let index = build_music_index(&gd, &src);
    assert_eq!(
        kinds_and_roles(&index, "m_a"),
        (vec![MusicKind::Map], vec![Some(MusicRole::Map)])
    );
    let boss = &index.tracks[0].places[0];
    assert_eq!(
        (
            boss.kind,
            boss.name.as_deref(),
            boss.items[0].name.as_deref()
        ),
        (MusicPlaceKind::Map, Some("Sui"), Some("Floor 6"))
    );
    assert_eq!(
        kinds_and_roles(&index, "m_b"),
        (vec![MusicKind::Lobby], vec![Some(MusicRole::Shop)])
    );
    assert_eq!(
        kinds_and_roles(&index, "m_c"),
        (vec![MusicKind::Map], vec![Some(MusicRole::Map)])
    );
    assert_eq!(
        kinds_and_roles(&index, "m_d"),
        (vec![MusicKind::Battle], vec![])
    );
    assert_eq!(
        kinds_and_roles(&index, "m_e"),
        (vec![MusicKind::Lobby], vec![Some(MusicRole::Encounter)])
    );
}

fn activity(gd: &mut GameModel, id: &str, name: &str, start: i64) {
    gd.activities.insert(
        id.to_owned(),
        ActivityBasicInfo {
            id: id.to_owned(),
            name: name.to_owned(),
            start_time: start,
            ..ActivityBasicInfo::default()
        },
    );
}

fn script(path: &str, text: &[u8]) -> StoryScript {
    StoryScript::read(path, text, &StoryVariables::default())
}

fn not_utf8(text: &str) -> Vec<u8> {
    let mut bytes = text.as_bytes().to_vec();
    bytes.push(0xff);
    bytes
}

fn play(name: &str) -> String {
    format!("[PlayMusic(key=\"Sound_Beta_2/Music/obt/{name}_loop\")]")
}

fn story(id: &str, txt: &str, sort: i32) -> StoryUnlockData {
    StoryUnlockData {
        story_id: id.to_owned(),
        story_name: format!("Story {id}"),
        story_code: Some(format!("C-{sort}")),
        story_sort: sort,
        story_txt: txt.to_owned(),
        ..StoryUnlockData::default()
    }
}

fn story_model() -> GameModel {
    let mut gd = GameModel::default();
    gd.story_reviews.insert(
        "main_0".to_owned(),
        StoryReviewGroup {
            id: "main_0".to_owned(),
            name: "Prologue".to_owned(),
            act_type: "MAIN_STORY".to_owned(),
            start_time: 1,
            info_unlock_datas: vec![
                story("s2", "Obt/Main/level_main_00-02", 2),
                story("s1", "Obt/Main/level_main_00-01", 1),
                story("s3", "obt/main/level_missing", 3),
            ],
            ..StoryReviewGroup::default()
        },
    );
    gd.roguelike.themes.insert(
        "rogue_1".to_owned(),
        RoguelikeThemeGameData {
            theme_id: "rogue_1".to_owned(),
            theme_name: "Phantom".to_owned(),
            ..RoguelikeThemeGameData::default()
        },
    );
    gd
}

fn places(index: &MusicIndex, id: &str) -> Vec<(Option<String>, Vec<MusicPlaceItem>)> {
    index
        .tracks
        .iter()
        .find(|t| t.id == id)
        .unwrap_or_else(|| panic!("no track {id}"))
        .places
        .iter()
        .map(|p| (p.name.clone(), p.items.clone()))
        .collect()
}

fn item(id: Option<&str>, code: Option<&str>, name: &str) -> MusicPlaceItem {
    MusicPlaceItem {
        id: id.map(str::to_owned),
        code: code.map(str::to_owned),
        name: Some(name.to_owned()),
    }
}

#[test]
fn listed_scripts_read_by_story_order_and_unlisted_ones_by_path() {
    let gd = story_model();
    let src = MusicSources {
        scripts: vec![
            script("obt/rogue/rogue_1/zz_end.txt.txt", play("m_b").as_bytes()),
            script("Obt/Main/level_main_00-02.txt.txt", play("m_a").as_bytes()),
            script("Obt/Main/level_main_00-01.txt.txt", b"not a script"),
            script(
                "[uc]info/Obt/Main/level_main_00-01.txt.txt",
                play("m_a").as_bytes(),
            ),
            script("obt/rogue/rogue_1/aa_start.txt.txt", play("m_b").as_bytes()),
            script("obt/loose/bad.txt.txt", &not_utf8(&play("m_bad"))),
            script("[uc]info/obt/info_only.txt.txt", play("m_c").as_bytes()),
            script("obt/notes.txt", play("m_c").as_bytes()),
        ],
        ..MusicSources::default()
    };
    let index = build_music_index(&gd, &src);
    let ids: Vec<&str> = index.tracks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["m_a", "m_b"]);
    assert_eq!(
        places(&index, "m_a"),
        [(
            Some("Prologue".to_owned()),
            vec![
                item(Some("s1"), Some("C-1"), "Story s1"),
                item(Some("s2"), Some("C-2"), "Story s2"),
            ]
        )]
    );
    assert_eq!(
        places(&index, "m_b"),
        [(
            Some("Phantom".to_owned()),
            vec![item(None, None, "aa_start"), item(None, None, "zz_end")]
        )]
    );
    let m_a = index.tracks.iter().find(|t| t.id == "m_a").unwrap();
    assert_eq!(m_a.origin.as_deref(), Some("Prologue"));
    assert_eq!(m_a.kinds, [MusicKind::Story]);
}

#[test]
fn a_listed_script_is_not_read_again_as_unlisted() {
    let gd = story_model();
    let src = MusicSources {
        scripts: vec![script(
            "obt/main/level_missing.txt.txt",
            b"plain text [PlayMusic(key=\"Sound_Beta_2/Music/obt/m_x_loop\")]",
        )],
        ..MusicSources::default()
    };
    assert!(build_music_index(&gd, &src).tracks.is_empty());
}

#[test]
fn unlisted_scripts_sort_by_path_components() {
    let mut gd = story_model();
    gd.story_reviews.clear();
    let src = MusicSources {
        scripts: vec![
            script("a-b/second.txt.txt", play("m_a").as_bytes()),
            script("a/b/first.txt.txt", play("m_a").as_bytes()),
        ],
        ..MusicSources::default()
    };
    let index = build_music_index(&gd, &src);
    assert_eq!(
        places(&index, "m_a"),
        [(
            None,
            vec![item(None, None, "first"), item(None, None, "second")]
        )]
    );
}

#[test]
fn a_listed_script_reads_lossily() {
    let gd = story_model();
    let src = MusicSources {
        scripts: vec![script(
            "obt/main/level_missing.txt.txt",
            &not_utf8(&play("m_l")),
        )],
        ..MusicSources::default()
    };
    let index = build_music_index(&gd, &src);
    assert_eq!(
        places(&index, "m_l"),
        [(
            Some("Prologue".to_owned()),
            vec![item(Some("s3"), Some("C-3"), "Story s3")]
        )]
    );
}

#[test]
fn an_unlisted_script_need_not_open_with_a_command() {
    let gd = story_model();
    let text = format!("prose first\n{}", play("m_p"));
    let src = MusicSources {
        scripts: vec![script("obt/loose/prose.txt.txt", text.as_bytes())],
        ..MusicSources::default()
    };
    let index = build_music_index(&gd, &src);
    assert_eq!(
        places(&index, "m_p"),
        [(None, vec![item(None, None, "prose")])]
    );
}

#[test]
fn the_primary_path_wins_over_uc_info_when_both_are_scripts() {
    let gd = story_model();
    let src = MusicSources {
        scripts: vec![
            script(
                "[uc]info/Obt/Main/level_main_00-02.txt.txt",
                play("m_u").as_bytes(),
            ),
            script("Obt/Main/level_main_00-02.txt.txt", play("m_a").as_bytes()),
        ],
        ..MusicSources::default()
    };
    let index = build_music_index(&gd, &src);
    let ids: Vec<&str> = index.tracks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["m_a"]);
}

#[test]
fn backslash_paths_read_as_slashes() {
    let gd = story_model();
    let src = MusicSources {
        scripts: vec![
            script(
                "Obt\\Main\\level_main_00-01.txt.txt",
                play("m_a").as_bytes(),
            ),
            script("obt\\rogue\\rogue_1\\x_end.txt.txt", play("m_b").as_bytes()),
        ],
        ..MusicSources::default()
    };
    assert_eq!(src.scripts[0].path, "Obt/Main/level_main_00-01.txt.txt");
    let index = build_music_index(&gd, &src);
    assert_eq!(
        places(&index, "m_a"),
        [(
            Some("Prologue".to_owned()),
            vec![item(Some("s1"), Some("C-1"), "Story s1")]
        )]
    );
    assert_eq!(
        places(&index, "m_b"),
        [(Some("Phantom".to_owned()), vec![item(None, None, "x_end")])]
    );
}

fn stage(stage_id: &str, level: &str, code: &str) -> StageIndexEntry {
    StageIndexEntry {
        stage_id: stage_id.to_owned(),
        level_id: Some(level.to_owned()),
        code: code.to_owned(),
        name: Some(format!("Stage {code}")),
        zone_id: "main_0".to_owned(),
        zone_name: Some("Evil Time Part 1".to_owned()),
        zone_order: 0,
        group: String::new(),
        category: String::new(),
        ap_cost: 6,
        boss: false,
        originite_prime: 0,
        is_hard: false,
        difficulty: StageDifficulty::Normal,
        can_view: true,
        preview: None,
        banner: None,
    }
}

fn bank(name: &str, intro: Option<&str>, loop_: &str) -> RawBgmBank {
    RawBgmBank {
        name: name.to_owned(),
        intro: intro.map(|i| format!("Audio/Sound_Beta_2/Music/obt/{i}")),
        loop_: Some(format!("Audio/Sound_Beta_2/Music/obt/{loop_}")),
    }
}

fn tree(server: Option<&str>, paths: &[&str]) -> ClipTree {
    ClipTree {
        server: server.map(str::to_owned),
        paths: paths
            .iter()
            .map(|p| format!("audio/audio/sound_beta_2/music/{p}"))
            .collect(),
    }
}

#[test]
fn stages_follow_the_level_bgm_and_clips_the_tree_holding_the_loop() {
    let gd = GameModel {
        music: MusicBanks::from_raw(&RawAudioData {
            bgm_banks: vec![
                bank(
                    "battle.ON_GAME_READY.indust",
                    Some("m_bat_indust_intro"),
                    "m_bat_indust_loop",
                ),
                bank("battle.ON_CUSTOM_TRIGGER.boss", None, "m_bat_boss_loop"),
                bank(
                    "battle.ON_GAME_READY.calm",
                    Some("m_bat_calm_intro"),
                    "m_bat_calm_loop",
                ),
            ],
            ..RawAudioData::default()
        }),
        stage_index: vec![
            stage("main_00-01", "/Obt/Main/level_main_00-01", "0-1"),
            stage("main_00-02", "obt/main/level_main_00-02", "0-2"),
            stage("main_00-03", "obt/main/level_main_00-03", "0-3"),
        ],
        ..GameModel::default()
    };
    let src = MusicSources {
        level_bgm: [
            ("obt/main/level_main_00-01", "indust"),
            ("obt/main/level_main_00-02", "battle.ON_CUSTOM_TRIGGER.boss"),
            ("obt/main/level_main_00-03", "no_such_bank"),
        ]
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .into(),
        clip_trees: vec![
            tree(
                None,
                &["own/m_bat_indust_intro.ogg", "obt/m_bat_calm_loop.ogg"],
            ),
            tree(
                Some("cn"),
                &[
                    "obt/m_bat_indust_loop.ogg",
                    "obt/m_bat_indust_intro.ogg",
                    "obt/m_bat_calm_intro.ogg",
                ],
            ),
        ],
        ..MusicSources::default()
    };
    assert_eq!(
        level_ids(&gd),
        [
            "obt/main/level_main_00-01",
            "obt/main/level_main_00-02",
            "obt/main/level_main_00-03"
        ]
    );
    let index = build_music_index(&gd, &src);
    let ids: Vec<&str> = index.tracks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["m_bat_boss", "m_bat_calm", "m_bat_indust"]);
    let [boss, calm, indust] = &index.tracks[..] else {
        unreachable!()
    };
    assert_eq!(indust.server.as_deref(), Some("cn"));
    assert_eq!(
        indust.intro_url.as_deref(),
        Some("/audio/audio/sound_beta_2/music/obt/m_bat_indust_intro.ogg")
    );
    assert_eq!(indust.places[0].items[0].code.as_deref(), Some("0-1"));
    assert_eq!(boss.places[0].kind, MusicPlaceKind::Stage);
    assert_eq!(boss.places[0].items[0].code.as_deref(), Some("0-2"));
    assert_eq!(calm.server, None);
    assert_eq!(calm.intro_url, None);
    assert!(calm.places.is_empty());
    assert_eq!(index.totals.from_other_server, 1);
}

const BANK: &str = "battle.ON_GAME_READY.x";

fn origin_model(loop_path: &str, extra_banks: &[&str]) -> GameModel {
    let mut banks = vec![(BANK, None, loop_path)];
    banks.extend(extra_banks.iter().map(|b| (*b, None, loop_path)));
    let mut gd = GameModel {
        music: audio(&banks, &[("music_a", BANK), ("music_b", BANK)]),
        ..GameModel::default()
    };
    for (id, name, start) in [
        ("actc", "Claim", 5),
        ("act1side", "Folder Event", 10),
        ("act2", "Menu Event", 50),
        ("act3", "Stage Event", 30),
        ("act4", "Story Event", 40),
        ("act5", "Late Event", 90),
    ] {
        activity(&mut gd, id, name, start);
    }
    for (id, name) in [("main_a", "Chapter A"), ("main_b", "Chapter B")] {
        gd.zones.insert(
            id.to_owned(),
            Zone {
                zone_id: id.to_owned(),
                zone_name_second: Some(name.to_owned()),
                ..Zone::default()
            },
        );
    }
    gd.zone_open_times.insert("main_a".to_owned(), 100);
    gd
}

fn story_groups(gd: &mut GameModel, loop_path: &str, act4_plays: usize) -> Vec<StoryScript> {
    let mut scripts = Vec::new();
    for (group, start, plays) in [("act4", 40, act4_plays), ("act5", 90, 1)] {
        let stories = (0..plays)
            .map(|i| {
                let txt = format!("activities/{group}/s{i}");
                let text = format!("[PlayMusic(key=\"{loop_path}\")]");
                scripts.push(script(&format!("{txt}.txt.txt"), text.as_bytes()));
                story(&format!("{group}_{i}"), &txt, i as i32)
            })
            .collect();
        gd.story_reviews.insert(
            group.to_owned(),
            StoryReviewGroup {
                id: group.to_owned(),
                act_type: "ACTIVITY_STORY".to_owned(),
                start_time: start,
                info_unlock_datas: stories,
                ..StoryReviewGroup::default()
            },
        );
    }
    scripts
}

fn origin(gd: &GameModel, src: &MusicSources) -> Option<String> {
    let index = build_music_index(gd, src);
    assert_eq!(index.tracks.len(), 1);
    index.tracks[0].origin.clone()
}

#[test]
fn origin_takes_the_first_source_in_chain_order() {
    let expected = [
        Some("Chapter A"),
        Some("Claim"),
        Some("Folder Event"),
        Some("Menu Event"),
        Some("Stage Event"),
        Some("Story Event"),
        None,
    ];
    for (from, want) in expected.into_iter().enumerate() {
        let has = |step: usize| from <= step;
        let loop_path = if has(2) {
            "Audio/Sound_Beta_2/Music/act1side/m_x_loop"
        } else {
            "Audio/Sound_Beta_2/Music/m_x_loop"
        };
        let menu: &[&str] = if has(3) {
            &["sys.ON_ACTIVITY_LOADED.act2"]
        } else {
            &[]
        };
        let mut gd = origin_model(loop_path, menu);
        let mut json = Vec::new();
        if has(0) {
            json.push((
                "zone_table",
                json!({"MainlineAdditionInfo": [{"key": "main_a",
                    "value": {"ZoneId": "main_a", "GameMusicId": "music_a"}}]}),
            ));
        }
        if has(1) {
            json.push((
                "stage_table",
                json!({"StorylineStorySets": [{"key": "set",
                    "value": {"GameMusicId": "music_a", "RelevantActivityId": "actc"}}]}),
            ));
        }
        let mut level_bgm = std::collections::HashMap::new();
        if has(4) {
            let mut entry = stage("act3_01", "obt/act3/level_01", "AC-1");
            entry.zone_id = "act3".to_owned();
            gd.stage_index.push(entry);
            level_bgm.insert("obt/act3/level_01".to_owned(), "x".to_owned());
        }
        let scripts = if has(5) {
            story_groups(&mut gd, loop_path, 2)
        } else {
            Vec::new()
        };
        let src = MusicSources {
            tables: tables(&json),
            scripts,
            level_bgm,
            ..MusicSources::default()
        };
        assert_eq!(origin(&gd, &src).as_deref(), want, "chain from step {from}");
    }
}

#[test]
fn origin_steps_take_the_earliest_and_undated_sorts_last() {
    let loop_path = "Audio/Sound_Beta_2/Music/m_x_loop";
    let gd = origin_model(loop_path, &[]);
    let src = MusicSources {
        tables: tables(&[(
            "zone_table",
            json!({
                "MainlineAdditionInfo": [
                    {"key": "main_a", "value": {"ZoneId": "main_a", "GameMusicId": "music_a"}},
                    {"key": "main_b", "value": {"ZoneId": "main_b", "GameMusicId": "music_b"}}
                ],
                "MainlineZoneIdList": ["main_a", "main_b"]
            }),
        )]),
        ..MusicSources::default()
    };
    assert_eq!(origin(&gd, &src).as_deref(), Some("Chapter B"));

    let sandbox = tables(&[(
        "sandbox_perm_table",
        json!({"BasicInfo": [{"key": "sandbox_1",
            "value": {"TopicId": "sandbox_1", "TopicName": "Sand"}}]}),
    )]);
    let src = MusicSources {
        tables: sandbox,
        ..MusicSources::default()
    };
    let gd = origin_model(
        loop_path,
        &[
            "sys.ON_SANDBOX_MAP.sandbox_1",
            "sys.ON_ACTIVITY_LOADED.act5",
        ],
    );
    assert_eq!(origin(&gd, &src).as_deref(), Some("Late Event"));
    let gd = origin_model(loop_path, &["sys.ON_SANDBOX_MAP.sandbox_1"]);
    assert_eq!(origin(&gd, &src).as_deref(), Some("Sand"));

    let mut gd = origin_model(loop_path, &[]);
    let src = MusicSources {
        scripts: story_groups(&mut gd, loop_path, 1),
        ..MusicSources::default()
    };
    assert_eq!(origin(&gd, &src).as_deref(), Some("Story Event"));
}
