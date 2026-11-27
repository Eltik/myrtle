use std::collections::HashMap;

use serde_json::Value;

use crate::story_sprites::*;

/// Folders that each hold a body spelled like the folder itself.
fn tree(names: &[&str]) -> HashMap<String, Vec<String>> {
    names
        .iter()
        .map(|n| ((*n).to_owned(), vec![(*n).to_owned()]))
        .collect()
}

fn folders(names: &[&str]) -> HashMap<String, String> {
    character_ids(&tree(names))
}

#[test]
fn a_bare_avg_npc_number_is_the_character_not_a_set() {
    let t = tree(&[
        "avg_npc_001",
        "avg_npc_061",
        "avg_npc_102",
        "avg_npc_102_2",
        "avg_npc_935_1",
        "avg_npc_935_2",
        "avg_doc_1",
        "npc_10002",
        "char_003_kalts_1",
        "char_242_mayer",
    ]);
    let ids = character_ids(&t);
    let id = |f: &str| ids[f].as_str();
    // Maria Nearl's folder is hers, not one set of a shared `avg_npc`.
    assert_eq!(id("avg_npc_061"), "avg_npc_061");
    // The folder the old rule kept keeps the id it was saved under.
    assert_eq!(id("avg_npc_001"), "avg_npc");
    // A bare folder whose number a set-numbered character already holds
    // is someone else: the sets keep the id, the bare folder takes `_0`.
    assert_eq!(id("avg_npc_102_2"), "avg_npc_102");
    assert_eq!(id("avg_npc_102"), "avg_npc_102_0");
    let sets = sets_by_character(&t, &ids);
    assert_eq!(sets["avg_npc_102"], vec![(2, "avg_npc_102_2")]);
    assert_eq!(sets["avg_npc_102_0"], vec![(102, "avg_npc_102")]);
    // Everything else cuts its set number as before.
    assert_eq!(id("avg_npc_935_2"), "avg_npc_935");
    assert_eq!(id("avg_doc_1"), "avg_doc");
    assert_eq!(id("npc_10002"), "npc");
    assert_eq!(id("char_003_kalts_1"), "char_003_kalts");
    assert_eq!(id("char_242_mayer"), "char_242_mayer");
    assert_eq!(sets.len(), 9);
}

#[test]
fn a_line_spoken_over_a_bare_avg_npc_votes_for_that_character() {
    let ids = folders(&[
        "avg_npc_001",
        "avg_npc_061",
        "avg_npc_120",
        "avg_npc_208",
        "avg_npc_208_1",
    ]);
    let mut votes = HashMap::new();
    vote_script(
        "[Character(name=\"avg_npc_061#2\",fadetime=1,block=true)]\n[name=\"Maria\"]  Hi, everyone!\n\
         [Character(name=\"avg_npc_120\", name2=\"avg_npc_061#7\", focus=2)]\n[name=\"Maria\"]  ......\n\
         [Character(name=\"avg_npc_208\")]\n[name=\"Cannot\"]  Huh.\n\
         [Character(name=\"avg_npc_208_1#2\")]\n[name=\"Monique\"]  Hm.\n",
        &ids,
        &mut votes,
    );
    assert_eq!(votes["avg_npc_061"]["Maria"], 2);
    assert!(!votes.contains_key("avg_npc"));
    assert_eq!(votes["avg_npc_208_0"]["Cannot"], 1);
    assert_eq!(votes["avg_npc_208"]["Monique"], 1);
}

#[test]
fn runner_up_names_over_the_floor_are_aliases() {
    let votes = |v: &[(&str, u32)]| -> HashMap<String, u32> {
        v.iter().map(|(n, c)| ((*n).to_owned(), *c)).collect()
    };
    let names = |v: &[(&str, u32)]| pick_names(votes(v));
    // avg_npc_061 on EN: Maria wins, Blemishine (93 lines) clears 10% of 772.
    let maria = names(&[("Maria", 772), ("Blemishine", 93), ("Guard", 4)]).unwrap();
    assert_eq!(maria.speaker, "Maria");
    assert_eq!(maria.aliases, vec!["Blemishine".to_owned()]);
    // The floor is max(5, winner / 10), inclusive on both, unrounded.
    let small = names(&[("A", 20), ("B", 5), ("C", 4)]).unwrap();
    assert_eq!(small.aliases, vec!["B".to_owned()]);
    let big = names(&[("A", 101), ("B", 11), ("C", 10)]).unwrap();
    assert_eq!(big.aliases, vec!["B".to_owned()]);
    // Never the winner again, in any case or wrapping; never a letterless
    // `???`; most votes first, ties by name, at most four.
    let messy = names(&[
        ("Talulah", 500),
        ("talulah?", 90),
        ("???", 80),
        ("'Deathless Black Snake'", 60),
        ("E", 50),
        ("D", 50),
        ("F", 50),
        ("G", 50),
        ("H", 40),
    ])
    .unwrap();
    assert_eq!(
        messy.aliases,
        vec!["'Deathless Black Snake'", "D", "E", "F"]
    );
    // No winner, no aliases.
    assert_eq!(names(&[("A", 10), ("B", 9), ("C", 9)]), None);
    assert_eq!(names(&[("A", 1)]), None);
    assert!(names(&[("A", 30)]).unwrap().aliases.is_empty());
}

#[test]
fn a_folder_no_file_rule_spells_falls_back_to_its_hub() {
    // Ported from main: the hub is now the parsed `hub.json`, not its path.
    let hub: Value = serde_json::from_str(
        r#"{"groups":[{"facePos":{"x":-1,"y":-1},"sprites":[{"name":"char_2006_fmzuki_1"}]}]}"#,
    )
    .unwrap();
    let stems = vec!["char_2006_fmzuki_1".to_owned(), "hub".to_owned()];
    assert_eq!(default_body("char_2006_weiywfmzuki_1", &stems), None);
    assert_eq!(
        hub_body(Some(&hub), &stems),
        Some("char_2006_fmzuki_1".to_owned())
    );
    // A hub naming a file the folder lacks gives nothing.
    assert_eq!(hub_body(Some(&hub), &["other".to_owned()]), None);
}

#[test]
fn args_read_quoted_and_bare_values() {
    let a = args(r#"Character(name="char_003_kalts_1", name2="avg_npc_1#2", focus=2)"#);
    assert_eq!(
        a,
        vec![
            ("name".to_owned(), "char_003_kalts_1".to_owned()),
            ("name2".to_owned(), "avg_npc_1#2".to_owned()),
            ("focus".to_owned(), "2".to_owned()),
        ]
    );
    assert_eq!(
        args(r#"name="Kal'tsit""#),
        vec![("name".to_owned(), "Kal'tsit".to_owned())]
    );
}

#[test]
fn a_lone_or_focused_sprite_gets_the_vote() {
    let lf = folders(&["char_003_kalts_1", "avg_npc_7_1"]);
    let mut votes = HashMap::new();
    vote_script(
        "[Character(name=\"char_003_kalts_1\")]\n[name=\"Kal'tsit\"]  Hello.\n\
         [Character(name=\"char_003_kalts_1\", name2=\"avg_npc_7_1#3\", focus=2)]\n[name=\"Guard\"]  Halt.\n\
         [Character(name=\"char_003_kalts_1\", name2=\"avg_npc_7_1\")]\n[name=\"Nobody\"]  Unattributed.\n",
        &lf,
        &mut votes,
    );
    assert_eq!(votes["char_003_kalts"]["Kal'tsit"], 1);
    assert_eq!(votes["avg_npc_7"]["Guard"], 1);
    assert!(!votes["char_003_kalts"].contains_key("Nobody"));
}

#[test]
fn the_default_body_follows_the_reader_order() {
    let stems = |s: &[&str]| s.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
    assert_eq!(
        default_body(
            "avg_npc_935_1",
            &stems(&["1$1", "avg_npc_935_1$1", "avg_npc_935_1$1[alpha]"])
        ),
        Some("avg_npc_935_1$1".to_owned())
    );
    assert_eq!(
        default_body(
            "char_003_kalts_1",
            &stems(&["char_003_kalts_2", "char_003_kalts_1"])
        ),
        Some("char_003_kalts_1".to_owned())
    );
    assert_eq!(default_body("char_empty", &stems(&["1"])), None);
}
