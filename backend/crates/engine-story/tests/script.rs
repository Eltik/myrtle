#![allow(clippy::unwrap_used, clippy::expect_used, clippy::disallowed_methods)]

use myrtle_engine_story::StoryAssetIndex;
use myrtle_engine_story::script::{
    ScriptError, load_script, load_synopsis, parse_story, resolve_refs, script_facts,
};
use myrtle_gamedata_raw::{DirSource, TableSource};
use tempfile::TempDir;

fn tree(files: &[(&str, &str)]) -> (TempDir, DirSource) {
    let dir = tempfile::tempdir().unwrap();
    for (rel, text) in files {
        let path = dir.path().join("gamedata/story").join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let src = DirSource {
        root: dir.path().to_path_buf(),
    };
    (dir, src)
}

#[test]
fn script_probe_prefers_primary_and_accepts_swapped() {
    let (_d, src) = tree(&[
        ("act/x/a.txt.txt", "A summary line.\n"),
        ("[uc]info/act/x/a.txt.txt", "[Dialog]\n"),
        ("act/x/b.txt.txt", "[HEADER] primary\n"),
        ("[uc]info/act/x/b.txt.txt", "[HEADER] info\n"),
        ("act/x/c.txt.txt", "Only prose.\n"),
    ]);
    assert_eq!(load_script(&src, "act/x/a").unwrap(), "[Dialog]\n");
    assert_eq!(load_script(&src, "act/x/b").unwrap(), "[HEADER] primary\n");
    assert!(matches!(
        load_script(&src, "act/x/c"),
        Err(ScriptError::NotAScript { .. })
    ));
    assert!(matches!(
        load_script(&src, "act/x/nope"),
        Err(ScriptError::Missing { .. })
    ));
}

#[test]
fn a_script_that_is_not_utf8_reads_lossily() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gamedata/story/act/x/a.txt.txt");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"[Dialog]\n\xff\n").unwrap();
    let src = DirSource {
        root: dir.path().to_path_buf(),
    };
    assert!(
        load_script(&src, "act/x/a")
            .unwrap()
            .starts_with("[Dialog]")
    );
}

#[test]
fn synopsis_probe_reads_uc_info_and_passes_over_a_script() {
    let (_d, src) = tree(&[
        ("act/x/a.txt.txt", "[Dialog]\n"),
        (
            "[uc]info/act/x/a.txt.txt",
            "\u{feff}First paragraph.\r\nSecond.\r\n",
        ),
        ("act/x/b.txt.txt", "Swapped summary.\n"),
        ("[uc]info/act/x/b.txt.txt", "[HEADER] x\n"),
        ("act/x/c.txt.txt", "[Dialog]\n"),
        ("[uc]info/act/x/c.txt.txt", "  \n"),
    ]);
    assert_eq!(
        load_synopsis(&src, "info/act/x/a").as_deref(),
        Some("First paragraph.\nSecond.")
    );
    assert_eq!(
        load_synopsis(&src, "info/act/x/b").as_deref(),
        Some("Swapped summary.")
    );
    assert_eq!(load_synopsis(&src, "info/act/x/c"), None);
    assert_eq!(load_synopsis(&src, "info/act/x/nope"), None);
    assert_eq!(load_synopsis(&src, ""), None);
}

#[test]
fn script_facts_list_distinct_names_in_first_appearance_order() {
    let (_d, src) = tree(&[(
        "act/x/a.txt.txt",
        concat!(
            "[Background(image=\"bg_two\")]\n",
            "[charslot(slot=\"l\", name=\"avg_1_a_1#03$1\")]\n",
            "[Image(image=\"cg_one\")]\n",
            "[Background(image=\"bg_two\")]\n",
            "[charslot(slot=\"l\", name=\"avg_1_a_1#11$2\")]\n",
            "[character(name=\"char_002_amiya_1\")]\n",
            "[largebg(imagegroup=\"bg_p1/bg_p2\")]\n",
            "[Video(id=\"v\")]\n",
        ),
    )]);
    let facts = script_facts(&src, "act/x/a", &StoryAssetIndex::default()).unwrap();
    assert_eq!(facts.backgrounds, ["bg_two", "bg_p1", "bg_p2"]);
    assert_eq!(facts.images, ["cg_one"]);
    assert_eq!(facts.first_background(), Some("bg_two"));
    assert!(facts.has_video);
    let bases: Vec<&str> = facts.sprites.iter().map(|s| s.base.as_str()).collect();
    assert_eq!(bases, ["avg_1_a_1", "char_002_amiya_1"]);
    assert_eq!(facts.sprites[0].faces, ["3", "11"]);
    assert_eq!(facts.sprites[1].faces, ["1"]);
    assert!(script_facts(&src, "act/x/none", &StoryAssetIndex::default()).is_none());
}

#[test]
fn parse_story_collects_only_referenced_and_resolved_assets() {
    let index = StoryAssetIndex::default();
    let s = parse_story(
        "id",
        "Name",
        "grp",
        "[Background(image=\"bg_x\")]\n[name=\"A\"] one two\n[PlayMusic(key=\"$m\")]\n",
        &index,
    );
    assert_eq!(s.commands.len(), 3);
    assert_eq!(s.word_count, 2);
    assert!(s.assets.backgrounds.is_empty());
    assert!(s.assets.music.is_empty());
    let refs = resolve_refs(&s.commands, &index);
    assert_eq!(refs.backgrounds.get("bg_x"), Some(&None));
    assert_eq!(refs.music.get("$m"), Some(&None));
}

#[test]
fn an_interlude_that_names_no_sprite_is_an_image() {
    let dir = tempfile::tempdir().unwrap();
    for rel in [
        "textures/avg/imgs/x/cg_panel.png",
        "textures/avg/characters/avg_a_1/avg_a_1.png",
        "audio/audio/sound_beta_2/music/m_intro.ogg",
        "audio/audio/sound_beta_2/music/m_loop.ogg",
    ] {
        let path = dir.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"").unwrap();
    }
    let src = DirSource {
        root: dir.path().to_path_buf(),
    };
    let index = StoryAssetIndex::build(&src, &src, &|tree| {
        DirSource {
            root: src.root.join(tree),
        }
        .list_assets()
        .unwrap_or_default()
    })
    .unwrap();
    let s = parse_story(
        "id",
        "Name",
        "grp",
        "[interlude(name=\"cg_panel\")]\n[interlude(name=\"avg_a_1\")]\n[PlayMusic(intro=\"m_intro\", key=\"m_loop\")]\n",
        &index,
    );
    assert!(s.assets.images.contains_key("cg_panel"));
    assert!(!s.assets.characters.contains_key("cg_panel"));
    assert!(s.assets.characters.contains_key("avg_a_1"));
    let cue = &s.assets.music["m_loop"];
    assert!(cue.intro.as_deref().unwrap().ends_with("m_intro.ogg"));
    assert!(cue.r#loop.as_deref().unwrap().ends_with("m_loop.ogg"));
}
