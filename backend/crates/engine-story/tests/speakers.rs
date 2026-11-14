#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_story::{StoryCommand, speakers::*};

fn cmd(kind: &str, args: &[(&str, &str)]) -> StoryCommand {
    StoryCommand {
        kind: kind.to_owned(),
        args: args
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect(),
        text: None,
        line: 0,
    }
}

fn say(name: &str) -> StoryCommand {
    cmd("name", &[("name", name)])
}

/// Every name resolves to its own base (`a#2$1` -> `a`), except `ghost`.
fn folder_of(raw: &str) -> Option<String> {
    let base = raw.split(['#', '$', '@']).next().unwrap_or(raw).trim();
    (base != "ghost").then(|| base.to_owned())
}

fn attributed(walk: &SpeakerWalk) -> Vec<Vec<(String, f64)>> {
    walk.lines.iter().map(|l| l.sprites.clone()).collect()
}

fn one(folder: &str) -> Vec<(String, f64)> {
    vec![(folder.to_owned(), 1.0)]
}

#[test]
fn character_focus_lights_the_named_slot_and_minus_one_is_one() {
    let walk = attribute_speakers(
        &[
            cmd(
                "character",
                &[("name", "a#1"), ("name2", "b#3"), ("focus", "1")],
            ),
            say("A"),
            cmd(
                "character",
                &[("name", "a#1"), ("name2", "b#3"), ("focus", "2")],
            ),
            say("B"),
            cmd(
                "character",
                &[("name", "a"), ("name2", "b"), ("focus", "-1")],
            ),
            say("A"),
            // 3 dims both, and two on stage with nothing lit names nobody.
            cmd(
                "character",
                &[("name", "a"), ("name2", "b"), ("focus", "3")],
            ),
            say("?"),
            // A lone sprite with no focus is lit.
            cmd("character", &[("name", "b")]),
            say("B"),
        ],
        folder_of,
    );
    assert_eq!(
        attributed(&walk),
        vec![one("a"), one("b"), one("a"), vec![], one("b")]
    );
}

#[test]
fn two_lit_sprites_split_a_line_in_half() {
    let walk = attribute_speakers(
        &[
            cmd("character", &[("name", "a"), ("name2", "b")]),
            say("Both"),
            cmd("charslot", &[("slot", "l"), ("name", "a")]),
            cmd("charslot", &[("slot", "r"), ("name", "b")]),
            say("Both again"),
        ],
        folder_of,
    );
    let half = vec![("a".to_owned(), 0.5), ("b".to_owned(), 0.5)];
    assert_eq!(attributed(&walk), vec![half.clone(), half]);
}

#[test]
fn charslot_focus_dims_the_others_and_none_keeps_a_lone_sprite() {
    let walk = attribute_speakers(
        &[
            cmd("charslot", &[("slot", "l"), ("name", "a#2$1")]),
            cmd("charslot", &[("slot", "r"), ("name", "b"), ("focus", "r")]),
            say("B"),
            // A focus with no name re-lights by the list.
            cmd("charslot", &[("focus", "l")]),
            say("A"),
            // Bare charslot empties the stage.
            cmd("charslot", &[]),
            cmd("charslot", &[("slot", "m"), ("name", "a"), ("focus", "n")]),
            say("A alone, dimmed"),
            // An unknown slot is ignored whole, focus included.
            cmd(
                "charslot",
                &[("slot", "x"), ("name", "b"), ("focus", "none")],
            ),
            say("A still"),
            // An unresolved sprite is never put up.
            cmd("charslot", &[("slot", "m"), ("name", "ghost")]),
            say("A after a ghost"),
        ],
        folder_of,
    );
    assert_eq!(
        attributed(&walk),
        vec![one("b"), one("a"), one("a"), one("a"), one("a")]
    );
    // Placements count every PUT, including a re-put of the same sprite.
    let placed: Vec<&str> = walk.placements.iter().map(|(f, _)| f.as_str()).collect();
    assert_eq!(placed, ["a", "b", "a"]);
}

#[test]
fn a_plate_speaks_when_the_stage_does_not() {
    let script = [
        cmd("charslot", &[("slot", "m"), ("name", "a")]),
        cmd("charactercutin", &[("name", "c#4")]),
        say("A over the call"),
        cmd("charslot", &[]),
        say("C, the stage empty"),
        cmd("charactercutin", &[]),
        say("Nobody"),
        cmd(
            "interlude",
            &[
                ("channel", "1"),
                ("maskid", "m1"),
                ("type", "3"),
                ("name", "w"),
            ],
        ),
        cmd(
            "interlude",
            &[
                ("channel", "2"),
                ("maskid", "m2"),
                ("type", "3"),
                ("name", "v"),
            ],
        ),
        say("W or V"),
        cmd("interlude", &[("channel", "2"), ("clear", "true")]),
        say("W"),
        cmd("interlude", &[("channel", "1"), ("clear", "true")]),
        // A character with no window to draw into is dropped.
        cmd(
            "interlude",
            &[("channel", "9"), ("type", "3"), ("name", "w")],
        ),
        say("Nobody again"),
    ];
    let walk = attribute_speakers(&script, folder_of);
    assert_eq!(
        attributed(&walk),
        vec![
            one("a"),
            one("c"),
            vec![],
            vec![("w".to_owned(), 0.5), ("v".to_owned(), 0.5)],
            one("w"),
            vec![],
        ]
    );
    let plates: Vec<u8> = walk.lines.iter().map(|l| l.plate).collect();
    assert_eq!(plates, [1, 1, 0, 2, 2, 0]);

    // The other two readings, on the first line: the cut-in over a lit A.
    let exclusive = attribute_speakers_with(
        &script,
        SpeakerRules {
            plates: PlateRule::Exclusive,
        },
        folder_of,
    );
    assert_eq!(exclusive.lines[0].sprites, one("c"));
    let joined = attribute_speakers_with(
        &script,
        SpeakerRules {
            plates: PlateRule::Join,
        },
        folder_of,
    );
    assert_eq!(
        joined.lines[0].sprites,
        vec![("c".to_owned(), 0.5), ("a".to_owned(), 0.5)]
    );
}

#[test]
fn masks_are_not_names() {
    assert!(!is_meaningful_name("???"));
    assert!(!is_meaningful_name("？？？"));
    assert!(!is_meaningful_name(" ... "));
    assert!(!is_meaningful_name(""));
    assert!(is_meaningful_name("Nine"));
    assert!(is_meaningful_name("??? (Nine)"));
}
