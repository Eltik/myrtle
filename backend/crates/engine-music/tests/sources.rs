#![allow(clippy::unwrap_used)]

use myrtle_engine_music::{StoryScript, bgm_event, clip_index, script_music};
use myrtle_gamedata_raw::tables::story::StoryVariables;

#[test]
fn play_music_reads_both_halves_through_the_variables() {
    let vars = StoryVariables::from_map(
        [
            (
                "escape_intro".to_owned(),
                "Sound_Beta_2/Music/AVG/m_avg_escape_intro".to_owned(),
            ),
            (
                "escape_loop".to_owned(),
                "Sound_Beta_2/Music/AVG/m_avg_escape_loop".to_owned(),
            ),
        ]
        .into(),
    );
    let text = "[PlayMusic(intro=\"$escape_intro\", key=\"$escape_loop\", volume=0.8)]\n[playmusic(key=\"$missing\")]\n[PlayMusic(key=\"Sound_Beta_2/Music/act15side/m_sys_bitw_loop\")]";
    let found = script_music(text, &vars);
    assert_eq!(found.len(), 2);
    assert_eq!(
        found[0].loop_path,
        "Sound_Beta_2/Music/AVG/m_avg_escape_loop"
    );
    assert_eq!(
        found[0].intro.as_deref(),
        Some("Sound_Beta_2/Music/AVG/m_avg_escape_intro")
    );
    assert_eq!(
        found[1].loop_path,
        "Sound_Beta_2/Music/act15side/m_sys_bitw_loop"
    );
    assert_eq!(found[1].intro, None);
}

#[test]
fn a_script_records_whether_it_is_utf8_and_opens_with_a_command() {
    let vars = StoryVariables::default();
    let bom = StoryScript::read("a.txt.txt", "\u{feff}  [Dialog]".as_bytes(), &vars);
    assert!(bom.utf8 && bom.is_script && bom.plays.is_empty());
    let prose = StoryScript::read("b.txt.txt", b"prose", &vars);
    assert!(prose.utf8 && !prose.is_script);
    let lossy = StoryScript::read(
        "c.txt.txt",
        b"[PlayMusic(key=\"Sound_Beta_2/Music/obt/m_x_loop\")]\xff",
        &vars,
    );
    assert!(!lossy.utf8 && lossy.is_script);
    assert_eq!(lossy.plays[0].loop_path, "Sound_Beta_2/Music/obt/m_x_loop");
}

#[test]
fn a_level_event_is_read_without_a_parse() {
    assert_eq!(
        bgm_event(b"{\"BgmEvent\": \"calamity\", \"Enemies\": []}").as_deref(),
        Some("calamity")
    );
    assert_eq!(
        bgm_event(b"{\"BgmEvent\" :  \" indust \"}").as_deref(),
        Some("indust")
    );
    assert_eq!(bgm_event(b"{\"BgmEvent\": \"  \"}"), None);
    assert_eq!(bgm_event(b"{\"BgmEvent\": null}"), None);
    assert_eq!(bgm_event(b"{\"Enemies\": []}"), None);
    assert_eq!(bgm_event(b"{\"BgmEvent\": \"a\xff\"}"), None);
}

#[test]
fn the_first_root_wins_then_path_component_order() {
    let paths: Vec<String> = [
        "audio/anon/m_a.ogg",
        "audio/audio/sound_beta_2/x/a-b/m_b.ogg",
        "audio/audio/sound_beta_2/x/a/b/M_B.ogg",
        "audio/audio/sound_beta_2/x/m_a.ogg",
        "audio/audio/sound_beta_2/x/m_c.wav",
        "audio/audio/sound_beta_2/x/.ogg",
        "audio/audio/sound_beta_2_extra/m_d.ogg",
        "textures/m_e.ogg",
    ]
    .map(str::to_owned)
    .into();
    let index = clip_index(&paths);
    assert_eq!(index["m_a"], "/audio/audio/sound_beta_2/x/m_a.ogg");
    let windows = clip_index(&["audio\\anon\\m_w.ogg".to_owned()]);
    assert_eq!(windows["m_w"], "/audio/anon/m_w.ogg");
    assert_eq!(index["m_b"], "/audio/audio/sound_beta_2/x/a/b/M_B.ogg");
    assert_eq!(index.len(), 2);
}
