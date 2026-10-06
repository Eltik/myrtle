//! Unit tests of the question tests, the term helpers and the chronology lines.

use trevor::tools::Tools;

use crate::batch::{batch_items, qid_question};
use crate::chrono::{Chrono, chronology_for, time_evidence_block};
use crate::detect::{asks_identity, asks_origin, asks_relation, asks_scene, asks_term, asks_when, term_of};
use crate::retrieval::{Names, one_operator, one_operator_wide, story_label, typed_owner};
use crate::terms::{compact, count_word, first_sentences};

#[test]
fn term_lookup_helpers_find_the_term_and_its_spelling_parts() {
    assert_eq!(term_of("What is the khagan quest?").as_deref(), Some("khagan quest"));
    assert_eq!(term_of("who are the sarkaz?").as_deref(), Some("sarkaz"));
    assert_eq!(term_of("What is Amiya's power?"), None, "a possessive is not a term question");
    assert_eq!(compact("Khagan quest"), compact("Khaganquest"));
    assert_eq!(compact("Kal'tsit"), "kaltsit");
    assert_eq!(count_word("Iris looked for Mabel. Iris'", "Iris"), 2);
    assert_eq!(count_word("Irisa and Iris", "Iris"), 1);
    assert_eq!(first_sentences("What it is\nA land of ice. It is cold. It is far.", 2), "A land of ice. It is cold.");
}

#[test]
fn rule_triggers_fire_on_their_questions_only() {
    assert!(asks_origin("Why did swire join the LGD?") && asks_origin("When did the abyssal hunter project start?"));
    assert!(!asks_origin("Why is Kal'tsit so old?") && !asks_origin("What did Swire join?"));
    assert!(asks_relation("What is the relationship between Kristen and Friston?") && asks_relation("Which elite op is Lava a fan of / idolises?"));
    assert!(!asks_relation("Who is Ed?"));
    assert!(asks_when("When was Rhodes Island founded?") && asks_when("in what year did X die") && !asks_when("Who founded Rhodes Island?"));
    assert_eq!(typed_owner("Operator art: Pallas, E2 (Elite 2) art").as_deref(), Some("Pallas"));
    assert_eq!(typed_owner("Voice lines: Aroma").as_deref(), Some("Aroma"));
    assert_eq!(typed_owner("Module story: New Friends (Mutsumi Wakaba's module, PUM-Y)").as_deref(), Some("Mutsumi Wakaba"));
    assert!(asks_term("What is the khagan quest?") && asks_term("Who are the sarkaz?") && asks_term("What is sami guarding?"));
    assert!(!asks_term("What is Kal'tsit's real name?") && !asks_term("What is the name of the operator who visited Iris's castle of dreams?"));
    assert!(asks_identity("Which Rhodes Island Operator had visited Iris' castle of dreams?") && !asks_identity("Why did swire join the LGD?"));
    let mon3tr = ["mon3tr".to_owned()];
    assert!(asks_scene("During which event Mon3tr suggested assassinating a child", &mon3tr));
    assert!(asks_scene("What did Mon3tr say to the Doctor?", &mon3tr) && asks_scene("In which story Mon3tr ambushed a Sarkaz?", &mon3tr));
    assert!(!asks_scene("Why does Mon3tr's hat say KEE?", &mon3tr) && !asks_scene("What does the captain say about Mon3tr?", &mon3tr));
    assert!(!asks_scene("Is Mon3tr related to the Aggeloi?", &mon3tr));
}

#[test]
fn chronology_lines_name_the_subject_and_one_word_names_widen_only_when_unique() {
    let c = Chrono { history: vec![(1086.0, "1086: After excavating the Rhodes Island landship, Babel sought an engineer.".into()),
                                   (1094.0, "1094 (scene dated): Babelite detonates explosives on Rhodes Island.".into()),
                                   (1097.0, "1097: Ch'en leaves Lungmen.".into())], ..Chrono::default() };
    let out = chronology_for("When was Rhodes Island founded?", &c);
    assert!(out.contains("RHODES ISLAND") && out.contains("1086") && out.contains("1094") && !out.contains("Lungmen"));
    assert!(chronology_for("when was it founded?", &c).is_empty());
    let te = time_evidence_block(&out);
    assert!(te.starts_with("\nTIME EVIDENCE") && te.contains("T1. 1086") && te.contains("T2. 1094") && !te.contains("T3."));
    let t = Tools::load(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
    if t.attributes.is_some() {
        assert_eq!(one_operator_wide("who helped Mutsumi care for the plants?", &t).as_deref(), Some("Mutsumi Wakaba"));
        assert_eq!(one_operator("who helped Mutsumi care for the plants?", &t), None);
    }
}

#[test]
fn story_labels_and_batch_items_keep_their_shapes() {
    let mut names = std::collections::HashMap::new();
    names.insert("main_00-01".to_owned(), Names { story: "Prologue".into(), group: "Episode 00".into() });
    assert_eq!(story_label(&names, "main_00-01"), "story: Episode 00, Prologue");
    assert_eq!(story_label(&names, "act1_x"), "story: act1_x");
    let p = std::env::temp_dir().join(format!("trevor-ask-batch-test-{}.jsonl", std::process::id()));
    std::fs::write(&p, "{\"qid\": \"a\", \"question\": \"Q?\"}\n\n{\"question\": \"R?\"}\nnot json\n").unwrap();
    let items = batch_items(&p).unwrap();
    assert_eq!(items.len(), 3, "blank lines are skipped, malformed ones kept as errors");
    assert_eq!(qid_question(items[0].as_ref().unwrap()), ("a", "Q?"));
    assert_eq!(qid_question(items[1].as_ref().unwrap()), ("", "R?"));
    assert!(items[2].is_err());
    std::fs::remove_file(&p).unwrap();
}
