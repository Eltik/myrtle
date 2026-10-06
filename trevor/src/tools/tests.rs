//! Unit tests of the table tools, on hand-made tables and (when present) the real ones.

use serde_json::json;

use super::*;
use super::text::episode_number;

fn tools() -> Tools {
    let mut t = Tools::default();
    for (id, name, ch) in [("act33side", "Babel", None), ("act21side", "IL Siracusano", None), ("main_7", "The Birth of Tragedy", Some(7)),
                           ("main_14", "Absolved Will Be the Seekers", Some(14)), ("act29side", "Zwillingstürme im Herbst", None),
                           ("act42side", "The Masses' Travels", None), ("story_x", "Babel Records", None)] {
        let mut g = json!({"groupId": id, "name": name});
        if let Some(c) = ch {
            g["chapter"] = json!(c);
        }
        t.groups.insert(id.to_owned(), g);
    }
    t.identities = vec![vec!["Kal'tsit".into(), "AMa-10".into(), "Louisa".into(), "Kal'tsit".into()]];
    t.attributes = Some(vec![
        json!({"name": "Texas", "charId": "c1", "race": "Lupo", "heightCm": 161, "nation": "Lungmen", "class": "Vanguard"}),
        json!({"name": "Texas the Omertosa", "charId": "c2", "race": "Lupo", "heightCm": 161, "class": "Specialist"}),
        json!({"name": "Kal'tsit", "charId": "c3", "race": "Feline", "heightCm": 162, "class": "Medic"}),
        json!({"name": "Flamebringer", "charId": "c4", "race": "Sarkaz", "heightCm": 185, "class": "Guard", "branch": "Dreadnought"}),
    ]);
    t
}

fn gid(t: &Tools, n: &str) -> Option<String> {
    t.resolve_event(n).ok().map(|g| g["groupId"].as_str().unwrap_or_default().to_owned())
}

#[test]
fn events_resolve_by_name_number_and_misspelling() {
    let t = tools();
    assert_eq!(gid(&t, "act33side").as_deref(), Some("act33side"));
    assert_eq!(gid(&t, "Babel").as_deref(), Some("act33side"));
    assert_eq!(gid(&t, "babel").as_deref(), Some("act33side"));
    assert_eq!(gid(&t, "Il Siracusano").as_deref(), Some("act21side"));
    assert_eq!(gid(&t, "Il Siracusanno").as_deref(), Some("act21side"));
    assert_eq!(gid(&t, "Siracusano").as_deref(), Some("act21side"));
    assert_eq!(gid(&t, "Episode 7").as_deref(), Some("main_7"));
    assert_eq!(gid(&t, "chapter 14").as_deref(), Some("main_14"));
    assert_eq!(gid(&t, "EP07").as_deref(), Some("main_7"));
    assert_eq!(gid(&t, "Birth of Tragedy").as_deref(), Some("main_7"));
    assert_eq!(gid(&t, "Zwillingsturme im Herbst").as_deref(), Some("act29side"));
    assert_eq!(gid(&t, "the Masses Travels").as_deref(), Some("act42side"));
    assert_eq!(gid(&t, "the Babel rerun").as_deref(), Some("act33side"));
    assert_eq!(gid(&t, "Masses Travel").as_deref(), Some("act42side"));
    assert_eq!(gid(&t, "masses travels").as_deref(), Some("act42side"));
    assert_eq!(gid(&t, "The Masse's Travel").as_deref(), Some("act42side"));
    assert_eq!(gid(&t, "ch 14").as_deref(), Some("main_14"));
    assert_eq!(gid(&t, "chapter 14").as_deref(), Some("main_14"));
}

#[test]
fn unknown_events_are_errors_not_guesses() {
    let t = tools();
    assert!(matches!(t.resolve_event("Episode 99"), Err(ToolError::NotFound { .. })));
    assert!(matches!(t.resolve_event("Rhodes Island"), Err(ToolError::NotFound { .. })));
    assert!(t.resolve_event("Babl").is_err(), "4 letters: no fuzzy budget");
}

#[test]
fn names_resolve_exactly_then_through_identities_then_fuzzily() {
    let t = tools();
    let names = t.operator_names();
    assert_eq!(t.resolve_name("operator", "Texas", &names), Ok("Texas"));
    assert_eq!(t.resolve_name("operator", "texas the omertosa", &names), Ok("Texas the Omertosa"));
    assert_eq!(t.resolve_name("operator", "Kaltsit", &names), Ok("Kal'tsit"));
    assert_eq!(t.resolve_name("operator", "Louisa", &names), Ok("Kal'tsit"));
    assert_eq!(t.resolve_name("operator", "Flamebringr", &names), Ok("Flamebringer"));
    assert!(t.resolve_name("operator", "Amiya", &names).is_err());
    assert!(t.resolve_name("operator", "Texa", &names).is_err());
}

#[test]
fn tools_answer_and_fail_cleanly() {
    let t = tools();
    let r = t.run(&Route::new("operator_attribute", &[("operator", "texas"), ("field", "height")])).unwrap();
    assert!(r.contains("161 cm"), "{r}");
    assert!(matches!(t.run(&Route::new("operator_attribute", &[("operator", "Texas"), ("field", "birthday")])), Err(ToolError::NoData(_))));
    assert!(matches!(t.run(&Route::new("operator_attribute", &[("operator", "Texas"), ("field", "mood")])), Err(ToolError::BadArg { .. })));
    let r = t.run(&Route::new("operator_filter", &[("race", "Sarkaz"), ("sort", "tallest")])).unwrap();
    assert!(r.contains("Flamebringer: 185 cm"), "{r}");
    let r = t.run(&Route::new("operator_filter", &[("class", "medics")])).unwrap();
    assert!(r.starts_with("1 of the 4"), "{r}");
    assert!(t.run(&Route::new("operator_filter", &[])).is_err(), "no filter would list every operator");
    assert!(t.run(&Route::new("operator_filter", &[("race", "Elf")])).is_err());
    let r = t.run(&Route::new("operator_filter", &[("place", "Dreadnought")])).unwrap();
    assert!(r.contains("branch Dreadnought"), "{r}");
    let r = t.run(&Route::new("operator_filter", &[("branch", "Dreadnought guards")])).unwrap();
    assert!(r.contains("branch Dreadnought"), "{r}");
    assert!(matches!(t.run(&Route::new("death_of", &[("character", "W")])), Err(ToolError::Missing(_))));
    assert!(matches!(t.run(&Route::new("fly", &[])), Err(ToolError::UnknownTool(_))));
}

/// The shipped tables, when present (artifacts/ is git-ignored, so this passes vacuously without them).
#[test]
fn question_detectors_fire_on_their_questions_only() {
    let t = Tools::load(Path::new(env!("CARGO_MANIFEST_DIR")));
    if t.appearances.is_none() || t.canon.is_none() || !t.crossref.contains_key("bosses_playable") {
        return;
    }
    assert_eq!(t.appearance_question("What chapters does Elysium appear in?").as_deref(), Some("Elysium"));
    assert_eq!(t.appearance_question("When does Elysium first appear?"), None, "the first_appearance tool's question");
    assert_eq!(t.appearance_question("Who is Elysium?"), None);
    // The scoped scenes' detectors (2026-10-05 night): the event a question names, without "the" or the apostrophe,
    // and the characters of the appearance index who appear in it.
    let q = "what is the name of the npc that mon3tr suggests assassinating in masses travels";
    let g = t.event_in_question(q).expect("The Masses' Travels");
    assert_eq!(g["groupId"].as_str(), Some("act42side"));
    assert!(t.event_characters(q, "act42side", "The Masses' Travels").iter().any(|(n, _)| n == "Mon3tr"));
    assert_eq!(t.event_in_question("Who does Mandragora fight in Episode 10?").and_then(|g| g["chapter"].as_u64()), Some(10));
    assert!(t.event_in_question("What happened to Mandragora after Episode 10?").is_none(), "a time boundary is not the scope");
    assert!(t.event_in_question("Who is the best cook?").is_none());
    let a = t.appearances("Elysium").unwrap();
    assert!(a.contains("Episode 16") && a.contains("Stultifera Navis") && a.contains("Mentioned by name without speaking"), "{a}");
    assert_eq!(t.cross_ref_question("Which bosses later join Rhodes Island as playable operators?"), Some("bosses_playable"));
    assert_eq!(t.cross_ref_question("Who is the final boss of Episode 8?"), None);
    let (run, _, endings, k) = t.is_ending_question("What happens in IS4 Ending 2 (including its endbooks)?").unwrap();
    assert_eq!((run.as_str(), k), ("rogue_3", Some(2)));
    assert_eq!(endings.iter().map(|e| e.0).collect::<Vec<_>>(), vec![1, 2, 3, 4], "ro3_ending_c is not a numbered ending");
    assert_eq!(t.is_ending_question("what is the second ending of Integrated Strategies 3").map(|x| (x.0, x.3)), Some(("rogue_2".into(), Some(2))));
    assert!(t.is_ending_question("How does Episode 8 end?").is_none());
    assert_eq!(t.named_topic("Who are the three main factions in dossoles?").map(|p| p.topic).as_deref(), Some("Dossoles"));
    assert_eq!(t.place_nation("Dossoles", false).map(|p| p.topic).as_deref(), Some("Bolívar"));
    assert!(t.place_nation("Sarkaz", false).is_none(), "a race is not a place");
    assert!(t.place_nation("Sarkaz", true).is_none(), "a race is not a place");
    if t.place_nations.is_some() {
        assert_eq!(t.place_nation("Chernobog", true).map(|p| p.topic).as_deref(), Some("Ursus"));
        assert_eq!(t.place_nation("Dossoles", true).map(|p| p.topic).as_deref(), Some("Bolívar"));
    }
}

#[test]
fn real_tables_resolve_player_names() {
    let t = Tools::load(Path::new(env!("CARGO_MANIFEST_DIR")));
    if t.groups.is_empty() || t.attributes.is_none() || t.real_names.is_none() || t.deaths.is_none() {
        return;
    }
    for (name, id) in [("Il Siracusano", "act21side"), ("Babel", "act33side"), ("Episode 7", "main_7"), ("chapter 14", "main_14"),
                       ("Zwillingsturme im Herbst", "act29side"), ("Such Is the Joy of Our Reunion", "act40side"),
                       ("Darknight Memoir", "act9d0"), ("the Masses Travels", "act42side")] {
        assert_eq!(gid(&t, name).as_deref(), Some(id), "{name}");
    }
    let run = |tool: &str, args: &[(&str, &str)]| t.run(&Route::new(tool, args));
    assert!(run("operator_attribute", &[("operator", "texas"), ("field", "height")]).unwrap().contains("161 cm"));
    assert!(run("real_name", &[("operator", "Bagpipe")]).unwrap().contains("Fiona Young"));
    assert!(matches!(run("real_name", &[("operator", "Amiya")]), Err(ToolError::NoData(_))));
    assert!(run("death_of", &[("character", "patriot")]).unwrap().contains("Patriot"));
    assert!(run("deaths_in_event", &[("event", "Babel"), ("who", "all")]).unwrap().contains("Theresa"));
    assert!(run("reading_event", &[("event", "Babel")]).unwrap().contains("Episode 13"));
    assert!(matches!(run("recap", &[("event", "Rhodes Island"), ("ending", "false")]), Err(ToolError::NotFound { .. })));
    assert!(matches!(run("operator_attribute", &[("operator", "Theresa"), ("field", "race")]), Err(ToolError::NotFound { .. })));
    if t.canon.is_some() {
        let mut t = Tools::load(Path::new(env!("CARGO_MANIFEST_DIR")));
        t.opts.canon_v1 = true;
        let run = |tool: &str, args: &[(&str, &str)]| t.run(&Route::new(tool, args));
        let r = run("canon", &[("run", "IS4")]).unwrap();
        assert!(r.contains("no later story refers to any of its endings"), "{r}");
        // The counts move with the evidence scheme (CANON_EVIDENCE in scripts/canon_refs.py); the framing does not.
        let r = run("canon", &[("run", "IS3")]).unwrap();
        assert!(r.starts_with("The game never labels") && r.contains("Stella Caerula"), "{r}");
    }
    if t.topics.is_some() {
        assert_eq!(t.topic("Sarkazs").unwrap().topic, "Sarkaz");
        assert!(!t.topic("Sami").unwrap().text.contains('['));
        assert!(t.topic("Pythia").is_err(), "Pythia's summary is the generator asking for passages");
    }
    if t.canon.as_ref().is_some_and(|c| c["runs"][0]["verdict"].is_object()) {
        for run_ in ["IS3", "IS2"] {
            let r = run("canon", &[("run", run_)]).unwrap();
            if std::env::var("TREVOR_PRINT_CANON").is_ok() {
                println!("=== {run_}\n{r}");
            }
            assert!(r.starts_with("The game never labels an ending canon; Trevor's reading of the stories:"), "{r}");
            assert!(!r.contains("grade") && !r.contains("[1]"), "{r}");
        }
    }
    if t.storylines.is_some() {
        let r = run("reading_compare", &[("event", "Masses Travel"), ("other", "chapter 15")]).unwrap();
        assert!(r.contains("Dissociative Recombination") && r.contains("Storylines view places"), "{r}");
        assert!(t.reading_guide().unwrap().contains("The game's Storylines view places Babel before Episode 14"));
    }
}

#[test]
fn canon_labels_strength_splits_and_ties_from_the_counts() {
    let mut t = Tools::default();
    let q = |st: &str| json!({"story": st, "released": "2025-01-01", "quote": format!("line from {st}")});
    let e = |name: &str, sts: &[&str]| json!({"name": name, "referencedBy": sts.iter().map(|x| json!({"story": x})).collect::<Vec<_>>(),
                                               "quotes": sts.iter().map(|x| q(x)).collect::<Vec<_>>()});
    t.canon = Some(json!({"runs": [
        {"run": "rogue_1", "runName": "Phantom & Crimson Solitaire", "endings": [e("A", &["Act or Die"]), e("B", &["Act or Die"]), e("C", &[])]},
        {"run": "rogue_2", "runName": "Mizuki & Caerula Arbor", "endings": [e("Stella", &["module", "Episode 15"]), e("D", &[])]},
    ]}));
    let r1 = t.canon(Some("IS2")).unwrap();
    assert!(r1.starts_with("The game never labels"), "{r1}");
    assert!(r1.contains("No ending is favored") && r1.contains("A: one later story; Act or Die is credited to B as well"), "{r1}");
    assert!(r1.contains("C: no later story refers to it"), "{r1}");
    let r2 = t.canon(Some("mizuki")).unwrap();
    assert!(r2.contains("Most continued: Stella (2 later stories)") && r2.contains("Stella: continued by 2 later stories"), "{r2}");
    assert!(t.canon(Some("Integrated Strategies #3")).unwrap() == r2);
    assert!(t.canon(Some("rogue_2")).unwrap() == r2, "a data id is not shifted");
    assert!(t.canon(Some("IS1")).unwrap().contains("Ceobe's Fungimist, is not in the game data"));
    assert!(t.canon(Some("Fungimist")).unwrap().starts_with("Integrated Strategies #1"));
    assert!(matches!(t.canon(Some("IS9")), Err(ToolError::NotFound { .. })));
    let all = t.canon(None).unwrap();
    assert!(all.contains("Phantom & Crimson Solitaire (Integrated Strategies #2)") && all.contains("(Integrated Strategies #3)"), "{all}");
    t.opts.is_offset = false;
    assert!(t.canon(None).unwrap().contains("Phantom & Crimson Solitaire (Integrated Strategies #1)"), "the old numbering");
    assert!(t.canon(Some("IS1")).unwrap().contains("Phantom"));
}

#[test]
fn topics_resolve_by_name_then_alias_and_lose_their_citations() {
    let t = Tools { topics: Some(vec![json!({"topic": "Beast Lords", "aliases": ["Beast Lord", "Feranmut"], "summary": format!("Old [1]. Very old [2, 3].{}", " x".repeat(100))}),
                         json!({"topic": "Feranmut", "aliases": ["Feranmut", "Sui", "Beast Lord"], "summary": format!("Sui [4].{}", " x".repeat(100))}),
                         json!({"topic": "Pythia", "aliases": ["Pythia"], "summary": "Please provide the numbered passages [1]."}),
                         json!({"topic": "Sami", "aliases": ["Sami"], "summary": format!("Cold [x] land [9].{}", " snow".repeat(100))})]), ..Tools::default() };
    assert_eq!(t.topic("beast lord").unwrap().topic, "Beast Lords");
    assert_eq!(t.topic("Sui").unwrap().topic, "Feranmut");
    assert!(t.topic("Beast Lords").unwrap().text.starts_with("Old. Very old. x"));
    assert!(matches!(t.topic("Pythia"), Err(ToolError::NoData(_))), "a summary under 100 words is no summary");
    assert!(t.topic("sami").unwrap().text.starts_with("Cold [x] land. snow"));
    assert!(t.topic("Kazimierz").is_err());
}

#[test]
fn deep_entries_replace_the_summary_only_when_switched_on_and_long_enough() {
    let mut t = Tools { topics: Some(vec![json!({"topic": "Sami", "aliases": ["Sami"], "summary": format!("Cold [1].{}", " snow".repeat(100))}),
                                     json!({"topic": "Sarkaz", "aliases": ["Sarkaz"], "summary": format!("Horned [2].{}", " x".repeat(100))})]),
                        ..Tools::default() };
    t.deep_topics.insert("Sami".into(), format!("What it is\nA northern land [1, 2].{}", " ice".repeat(100)));
    t.deep_topics.insert("Sarkaz".into(), "What it is\nToo short [1].".into());
    assert!(t.topic("sami").unwrap().text.starts_with("Cold. snow"), "off: the summary, as before");
    t.topics_deep = true;
    assert!(t.topic("sami").unwrap().text.starts_with("What it is\nA northern land. ice"));
    assert!(t.topic("Sarkaz").unwrap().text.starts_with("Horned."), "a deep entry under 100 words falls back to the summary");
    assert!(t.deep_entry("Sarkaz").is_none() && t.deep_entry("Sami").is_some());
    // The 2026-10-05 gate: only a question about the topic as a whole.
    assert!(t.broad_topic_question("Who are the sarkaz?") && t.broad_topic_question("What is sami guarding?"));
    assert!(!t.broad_topic_question("Do Sarkaz, as a culture, give their horns as gifts?"));
    assert!(!t.broad_topic_question("Who is Amiya?"), "names no topic");
    let g = Tools { topics: Some(vec![json!({"topic": "Aegir", "kind": "race", "aliases": ["Ægir"]}), json!({"topic": "Arts", "kind": "concept", "aliases": []})]),
                    ..Tools::default() };
    assert!(g.names_group_topic("Are Petrams related to Aegirs?") && !g.names_group_topic("Is Kristen related to Friston?"));
    assert!(!g.names_group_topic("Are Arts related to Originium?"), "a concept topic is not a group of people");
}

#[test]
fn answer_check_reads_the_category_from_the_question_and_the_names_from_the_data() {
    let a = |n: &str, nation: &str, birth: &str| json!({"name": n, "nation": nation, "birthplace": birth});
    let t = Tools { attributes: Some(vec![a("Sakiko Togawa", "", "Unknown"), a("Iris", "Victoria", "Victoria"), a("Rosmontis", "Rhodes Island", "Victoria"),
                                          a("Amiya", "Rhodes Island", "Rim Billiton")]),
                    real_names: Some(vec![json!({"name": "Rosmontis", "realName": "Rosa"})]),
                    topics: Some(vec![json!({"topic": "Rhodes Island", "kind": "organization", "aliases": []})]),
                    identity_links: vec![("Mabel".into(), "Bluishsilver".into(), "former_name".into(), String::new(), String::new(), true)],
                    ..Tools::default() };
    let q = "Which Rhodes Island Operator (not playable unit) had visited Iris' castle of dreams?";
    assert_eq!(t.answer_constraint(q), Some(AnswerConstraint::NonPlayable));
    assert_eq!(t.answer_constraint("Which operator/Rhodes Island NPC comes closest to being a Psychologist?"), None, "operator or NPC");
    assert_eq!(t.answer_constraint("Which Rhodes Island operator won the chess tournament?"), None, "an organization is no nation");
    assert_eq!(t.answer_constraint("Which operator from Victoria plays the harp?"), Some(AnswerConstraint::Nation("Victoria".into())));
    let c = AnswerConstraint::NonPlayable;
    assert_eq!(t.failing_answer_name(q, "Sakiko Togawa visited it [3].", &c).map(|x| x.0).as_deref(), Some("Sakiko Togawa"));
    assert_eq!(t.failing_answer_name(q, "Rosa visited it, then Mabel.", &c).map(|x| x.1).as_deref(), Some("Rosa is the playable operator Rosmontis"));
    assert!(t.failing_answer_name(q, "Mabel, the former Bluishsilver, visited Iris.", &c).is_none(), "an NPC passes; Iris is in the question");
    assert!(t.failing_answer_name(q, "The passages do not say.", &c).is_none());
    assert!(t.failing_answer_name(q, "The passages do not identify one. They mention Rosmontis [1].", &c).is_none(), "context, not the answer");
    let v = AnswerConstraint::Nation("Victoria".into());
    assert!(t.failing_answer_name("Which operator from Victoria?", "Rosmontis.", &v).is_none(), "birthplace Victoria");
    assert!(t.failing_answer_name("Which operator from Victoria?", "Amiya.", &v).is_some());
}

#[test]
fn game_data_for_trait_questions_about_named_characters_and_families() {
    let a = |n: &str, race: &str| json!({"name": n, "race": race, "birthplace": "Kazimierz", "nation": "Kazimierz", "group": null, "heightCm": 170});
    let t = Tools { attributes: Some(vec![a("Nearl", "Kuranta"), a("Nearl the Radiant Knight", "Kuranta"), a("Angelina", "Vulpo"), a("Mountain", "Feline")]),
                    topics: Some(vec![json!({"topic": "Kuranta", "aliases": ["Kuranta"], "summary": format!("Horse-like [1].{}", " x".repeat(100)),
                                             "summaryV2": format!("Equine.{}", " y".repeat(100))})]),
                    game_data: true, topics_v2_text: true, ..Tools::default() };
    let g = t.game_data("Do the Nearls have wings?").unwrap();
    assert_eq!(g.topic, "Nearl, Nearl the Radiant Knight");
    assert!(g.text.contains("race Kuranta") && g.text.contains("Race Kuranta (Trevor's topic summary, start): Equine."));
    assert!(g.text.contains("Races whose topic summaries mention wings: none"));
    assert_eq!(t.game_data("how old is angelina").unwrap().topic, "Angelina");
    assert!(t.game_data("What did Nearl do in Kazimierz?").is_none(), "not a trait question");
    // A trade, kept knowingly: folded matching of 6 letters or more lets a common word through on a trait question.
    assert_eq!(t.game_data("how tall is the mountain").unwrap().topic, "Mountain");
    assert!(Tools { game_data: false, ..t }.game_data("Do the Nearls have wings?").is_none());
}

#[test]
fn new_dossiers_match_by_own_name_long_folded_short_as_written() {
    let t = Tools { new_dossiers: vec![("Rosa".into(), "r".into()), ("Innkeeper Zheng".into(), "z".into()), ("Pole-Carrier".into(), "p".into())],
                    ..Tools::default() };
    assert_eq!(t.new_dossier("Who is Rosa's mentor?").unwrap().topic, "Rosa");
    assert!(t.new_dossier("a rosa flower").is_none(), "a one-word name matches only as written");
    assert!(t.new_dossier("Rosalind says").is_none());
    assert_eq!(t.new_dossier("what does innkeeper zheng sell").unwrap().topic, "Innkeeper Zheng");
    let m = Tools { new_dossiers: vec![("Mountain".into(), "m".into())], ..Tools::default() };
    assert!(m.new_dossier("the tree he planted on the mountain").is_none());
    assert!(m.new_dossier("Is Mountain strong?").is_some());
    let l = Tools { new_dossiers: vec![("Lin".into(), "l".into())], ..Tools::default() };
    assert!(l.new_dossier("where Lin Qingyan was first brought").is_none(), "the start of a longer name");
    assert!(l.new_dossier("Is Lin a mob boss?").is_none(), "under 4 letters");
    assert_eq!(t.new_dossier("who is the pole carrier").unwrap().topic, "Pole-Carrier");
    assert!(Tools::default().new_dossier("Who is Red?").is_none(), "v1 loads none");
}

#[test]
fn topics_from_data_resolve_by_merged_names_and_drop_under_v1() {
    let long = |w: &str| format!("{w}.{}", " x".repeat(100));
    let rows = vec![json!({"topic": "Victoria", "aliases": ["Victoria"], "names": ["Victorian", "Victorians"], "summary": long("Old")}),
                    json!({"topic": "Yan-Lungmen", "aliases": ["Yan-Lungmen", "Lungmen"], "source": "table", "summary": long("City")}),
                    json!({"topic": "Kylin", "aliases": ["Kylin"], "source": "table", "thin": "passages: 2 of 2 name it"})];
    let mut t = Tools { topics: Some(served_topics(rows)), ..Tools::default() };
    assert_eq!(t.topic("victorians").unwrap().topic, "Victoria");
    assert_eq!(t.topic("Lungmen").unwrap().topic, "Yan-Lungmen");
    assert!(matches!(t.topic("Kylin"), Err(ToolError::NotFound { .. })), "a thin topic is not served");
    t.topics_v1();
    assert_eq!(t.topics.as_ref().unwrap().len(), 1);
    assert!(t.topics.as_ref().unwrap()[0].get("names").is_none(), "v1 has no merged names");
    assert!(t.topic("Lungmen").is_err());
}

#[test]
fn short_names_expand_only_unique_prefixes() {
    let mut t = tools();
    t.identities.push(vec!["Bibeak".into()]);
    assert_eq!(t.short_names("What did bibeak design for kal"), vec![("kal".to_owned(), "Kal'tsit".to_owned())]);
    assert!(t.short_names("Is Texas a Lupo?").is_empty(), "a whole name is left alone");
    // Texas and Texas the Omertosa share a first word, so "tex" is one target, the shorter name; "flame" also
    // matches Flamebringer here, which is why the caller checks corpus frequency (flame is a common word).
    assert_eq!(t.short_names("tex or flame"), vec![("tex".to_owned(), "Texas".to_owned()), ("flame".to_owned(), "Flamebringer".to_owned())]);
}

#[test]
fn cross_ref_lists_bosses_by_operator_and_off_topic_questions_name_nothing() {
    let mut t = tools();
    assert!(matches!(t.cross_ref("bosses_playable"), Err(ToolError::Missing(_))));
    t.crossref.insert("bosses_playable".into(), json!({"bosses": 229, "operators": 407, "rows": [
        {"operator": "W", "boss": "W", "matchedBy": "codename"},
        {"operator": "Amiya", "boss": "'Amiya'", "matchedBy": "codename"},
        {"operator": "Amiya", "boss": "'Amiya', Furnace's Finale", "matchedBy": "codename"}]}));
    let out = t.cross_ref("bosses_playable").unwrap();
    assert!(out.contains("229 boss") && out.contains(": 2 operators") && out.contains("- Amiya: \"'Amiya'\" (matched by codename); \"'Amiya', Furnace's Finale\""));
    assert!(matches!(t.cross_ref("everything"), Err(ToolError::BadArg { .. })));
    assert!(!t.names_anything("give me instructions for baking a tray of brownies"));
    assert!(t.names_anything("how tall is texas?") && t.names_anything("what happens in babel") && t.names_anything("who is Louisa"));
    assert!(t.names_anything("is this in arknights?") && t.names_anything("what's texas's hobby?"));
}

#[test]
fn design_deduce_reads_forward_and_reverse_and_shortlists_by_rare_features() {
    let mut t = tools();
    assert!(t.design_deduce_question("what animal is kirara based on").is_none());
    let row = |n: &str, text: &str| json!({"charId": n, "operator": n, "race": "", "alters": [], "text": text});
    t.design_evidence = Some(vec![
        row("Lucilla", "Art: dark ribbon-like tentacles and a translucent bell above her, blue water."),
        row("Phantom", "Art: a black cat, blue light."),
        row("Texas", "Art: wolf ears, blue sword."),
        row("Texas the Omertosa", "Art: a black wolf companion, blue."),
        row("Kirara", "Art: coral, blue screens.")]);
    // ian11 asks for an operator: forward, even though "phantom" is an operator's name.
    let (fwd, _) = t.design_deduce_question("what operator is based off an animal with phantom as part of its name").unwrap();
    assert!(fwd);
    let (fwd, named) = t.design_deduce_question("What animal is Kirara based on?").unwrap();
    assert!(!fwd && named.len() == 1 && named[0]["operator"] == "Kirara");
    // the longest name wins over the name it contains
    let (_, named) = t.design_deduce_question("What is Texas the Omertosa based on?").unwrap();
    assert_eq!(named.iter().map(|r| r["operator"].as_str().unwrap()).collect::<Vec<_>>(), vec!["Texas the Omertosa"]);
    assert!(t.design_deduce_question("who is phantom").is_none());
    // "tentacles" and "bell" are rare, "blue" is everywhere: Lucilla first, and an operator sharing only "blue" scores 0
    let short = t.design_shortlist(&[("giant phantom jelly".to_owned(), 2.0), ("long ribbon-like tentacles".to_owned(), 1.0),
                                     ("bell".to_owned(), 1.0), ("blue".to_owned(), 1.0)], 3);
    assert_eq!(short.first().map(|r| r["operator"].as_str().unwrap()), Some("Lucilla"));
    assert!(short.iter().all(|r| r["operator"] != "Kirara"));
}

#[test]
fn design_basis_answers_both_readings_and_only_design_questions() {
    let mut t = tools();
    assert_eq!(t.design_basis("what operator is based off an animal with phantom as part of its name"), None);
    t.attributes = Some(vec![json!({"name": "Lucilla"}), json!({"name": "Phantom"}), json!({"name": "Tragodia"}), json!({"name": "Kirara"})]);
    let sub = |x: &str| json!({"subject": x, "category": "animal", "why": "clues.", "confidence": "medium",
                               "evidence": [{"id": "E3", "source": "operator file, Profile", "text": "a line"}]});
    t.design_basis = Some(vec![
        json!({"operator": "Kirara", "alters": [], "subjects": [sub("phantom jellyfish")]}),
        json!({"operator": "Lucilla", "alters": [], "subjects": []}),
        json!({"operator": "Phantom", "alters": ["Tragodia"], "subjects": [sub("The Phantom of the Opera")]}),
        json!({"operator": "Tragodia", "alters": ["Phantom"], "subjects": [sub("Dionysus")]})]);
    let out = t.design_basis("what operator is based off an animal with phantom as part of its name").unwrap();
    assert!(out.starts_with("Trevor's inference from the game data") && !out.contains("wiki"));
    assert!(out.contains("- Kirara: likely draws on phantom jellyfish (animal, medium confidence)"));
    assert!(out.contains("- Phantom: likely draws on The Phantom of the Opera") && out.contains("(the alternate operator of Phantom)"));
    assert_eq!(out.matches("- Phantom:").count(), 1);
    assert_eq!(t.design_basis("who is phantom"), None);
    assert_eq!(t.design_basis("what is the kitchen of rhodes island based on"), None);
    // An operator with nothing inferred says so instead of guessing.
    assert!(t.design_basis("what animal is lucilla based on").unwrap().contains("- Lucilla: the game data does not point"));
    // The kill switch is the wiki answer, read only from the rows the legacy loader gives it.
    t.opts.wiki_legacy = true;
    assert_eq!(t.design_basis("what animal is lucilla based on"), None);
    t.design_basis_wiki = Some(vec![json!({"operator": "Lucilla", "basis": ["giant phantom jelly"], "sentence": "s.", "alters": [], "url": "u1"})]);
    assert!(t.design_basis("what animal is lucilla based on").unwrap().starts_with("From the Arknights wiki's trivia pages"));
}

/// The wiki is an eval reference only (2026-10-04): no serving code names its directory except `crate::reference`,
/// whose loader only `ask --wiki-legacy` calls, and loading the tables never reads it.
#[test]
fn serving_code_never_opens_eval_reference() {
    let dir = concat!("eval/", "reference");
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut stack = vec![src.clone()];
    let mut offenders = Vec::new();
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") && p != src.join("reference.rs") {
                let text = std::fs::read_to_string(&p).unwrap();
                if text.contains(dir) || text.contains(concat!("wiki_", "cache")) || text.contains(concat!("design_basis", ".jsonl"))
                    || text.contains(concat!("wiki_", "deceased")) || text.contains(concat!("wiki_", "status")) {
                    offenders.push(p.display().to_string());
                }
            }
        }
    }
    assert!(offenders.is_empty(), "serving code names the wiki eval references: {offenders:?}");
    let root = std::env::temp_dir().join(format!("trevor-ref-test-{}", std::process::id()));
    std::fs::create_dir_all(root.join(dir)).unwrap();
    std::fs::write(root.join(dir).join(concat!("design_basis", ".jsonl")), "{\"operator\": \"Lucilla\", \"basis\": [\"jelly\"]}\n").unwrap();
    let t = Tools::load(&root);
    assert!(t.design_basis.is_none() && t.design_basis_wiki.is_none() && !t.opts.wiki_legacy);
    assert!(crate::reference::wiki_design_basis(&root).is_some());
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn norm_folds_case_accents_and_apostrophes() {
    assert_eq!(norm("Ch'en, the Holungday!"), "chen the holungday");
    assert_eq!(norm("Zwillingstürme"), "zwillingsturme");
    assert_eq!(episode_number("episode 7"), Some(7));
    assert_eq!(episode_number("ch14"), Some(14));
    assert_eq!(episode_number("babel"), None);
}

#[test]
fn whole_word_matches_skip_matches_inside_words() {
    assert_eq!(whole_word_matches("Iris and Irisa met Iris.", "Iris").collect::<Vec<_>>(), vec![0, 19]);
    assert_eq!(whole_word_matches("Rhodes Island's Islands", "Island").count(), 1);
}
