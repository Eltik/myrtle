use super::*;

fn words(s: &str) -> u32 {
    u32::try_from(s.split_whitespace().count()).unwrap_or(u32::MAX)
}

#[allow(clippy::cast_possible_truncation)]
fn _unused() {}

fn cmd(kind: &str, args: &[(&str, &str)], text: Option<&str>, line: u32) -> StoryCommand {
    StoryCommand {
        kind: kind.to_owned(),
        args: args
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect(),
        text: text.map(str::to_owned),
        line,
    }
}

fn say(speaker: &str, text: &str, line: u32) -> StoryCommand {
    cmd("name", &[("name", speaker)], Some(text), line)
}

fn small() -> ChunkConfig {
    ChunkConfig {
        target_min_tokens: 6,
        target_max_tokens: 12,
        ..ChunkConfig::default()
    }
}

#[test]
fn background_without_image_is_not_a_scene_break() {
    let cmds = vec![
        cmd("background", &[("image", "bg_a")], None, 1),
        say("Amiya", "one two three", 2),
        // a fade: fadetime only, no image
        cmd(
            "background",
            &[("fadetime", "1"), ("block", "false")],
            None,
            3,
        ),
        say("Amiya", "four five six", 4),
    ];
    let (turns, _) = turns_of(&cmds);
    let scenes: Vec<u32> = turns.iter().map(|t| t.scene).collect();
    assert_eq!(
        scenes,
        vec![1],
        "a fade must not open a scene, so both Amiya lines stay one turn"
    );
}

#[test]
fn repeated_identical_background_is_not_a_scene_break() {
    let cmds = vec![
        cmd("background", &[("image", "bg_a")], None, 1),
        say("W", "one two three", 2),
        cmd("background", &[("image", "bg_a")], None, 3),
        say("W", "four five six", 4),
        cmd("background", &[("image", "bg_b")], None, 5),
        say("W", "seven eight nine", 6),
    ];
    let (turns, _) = turns_of(&cmds);
    let scenes: Vec<u32> = turns.iter().map(|t| t.scene).collect();
    // The first two W lines merge (same speaker, same scene); bg_b opens
    // scene 2 and therefore a new turn.
    assert_eq!(scenes, vec![1, 2]);
}

#[test]
fn pacing_commands_are_inert() {
    let base = vec![
        say("Amiya", "alpha bravo charlie", 1),
        say("Kal'tsit", "delta echo foxtrot", 2),
    ];
    let mut noisy = vec![
        cmd("delay", &[("time", "1.3")], None, 1),
        say("Amiya", "alpha bravo charlie", 1),
        cmd("blocker", &[("a", "1")], None, 2),
        cmd("playsound", &[("key", "x")], None, 2),
        cmd("dialog", &[], None, 2),
        say("Kal'tsit", "delta echo foxtrot", 2),
    ];
    noisy.push(cmd("stopmusic", &[], None, 3));

    let a = chunk_story("s", "g", &base, &small(), &words);
    let b = chunk_story("s", "g", &noisy, &small(), &words);
    assert_eq!(
        a.iter().map(|c| &c.text).collect::<Vec<_>>(),
        b.iter().map(|c| &c.text).collect::<Vec<_>>(),
        "delay/blocker/dialog/playsound must never move a chunk edge"
    );
    assert_eq!(a, b, "and the whole chunk must be byte-identical");
}

#[test]
fn a_turn_is_never_split() {
    // One turn far wider than the ceiling.
    let long = "lorem ".repeat(200);
    let cmds = vec![say("Amiya", long.trim(), 1)];
    let out = chunk_story("s", "g", &cmds, &small(), &words);
    assert_eq!(out.len(), 1, "an oversized turn survives whole");
    assert!(out[0].token_count > small().target_max_tokens);
    assert!(out[0].text.starts_with("Amiya: lorem"));
}

#[test]
fn sub_minimum_story_is_one_chunk() {
    let cmds = vec![say("Dobermann", "short line", 1)];
    let out = chunk_story("s", "g", &cmds, &small(), &words);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].ordinal, 0);
    assert_eq!(out[0].chunk_id, "s#0000");
}

#[test]
fn concatenation_is_lossless_and_unduplicated() {
    let mut cmds = Vec::new();
    for i in 0..40u32 {
        if i % 7 == 0 {
            cmds.push(cmd(
                "background",
                &[("image", &format!("bg_{i}"))],
                None,
                i * 2,
            ));
        }
        cmds.push(say(
            if i % 2 == 0 { "Amiya" } else { "Kal'tsit" },
            &format!("line {i} alpha bravo charlie delta"),
            i * 2 + 1,
        ));
    }
    let out = chunk_story("s", "g", &cmds, &small(), &words);
    assert!(out.len() > 4, "expected several chunks, got {}", out.len());

    let (turns, _) = turns_of(&cmds);
    let expected: Vec<String> = turns.iter().map(Turn::render).collect();
    let got: Vec<String> = out
        .iter()
        .flat_map(|c| c.text.split('\n').map(str::to_owned))
        .collect();
    assert_eq!(got, expected, "no loss and no duplication at overlap 0");

    for w in out.windows(2) {
        assert!(
            w[0].line_end < w[1].line_start,
            "line spans must not overlap: {:?} then {:?}",
            w[0].line_end,
            w[1].line_start
        );
    }
    for (i, c) in out.iter().enumerate() {
        assert_eq!(c.ordinal as usize, i);
        assert!(c.line_start <= c.line_end);
    }
}

#[test]
fn same_speaker_lines_merge_and_masked_speakers_are_excluded() {
    let cmds = vec![
        say("Amiya", "one two", 1),
        say("Amiya", "three four", 2),
        say("???", "five six", 3),
        say("", "seven eight", 4),
    ];
    let out = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
    assert_eq!(out.len(), 1);
    assert!(
        out[0].text.starts_with("Amiya: one two three four"),
        "same-speaker lines merge: {:?}",
        out[0].text
    );
    assert_eq!(out[0].speakers, vec!["Amiya".to_owned()]);
}

#[test]
fn prose_arg_kinds_are_read_from_args() {
    let cmds = vec![
        cmd("subtitle", &[("text", "a narrated caption")], None, 1),
        cmd("sticker", &[("text", "a sticker line")], None, 2),
    ];
    let out = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
    assert_eq!(out.len(), 1);
    assert!(out[0].text.contains("a narrated caption"));
    assert!(out[0].text.contains("a sticker line"));
}

#[test]
fn metadata_is_populated() {
    let cmds = vec![
        cmd("background", &[("image", "bg_rhodes")], None, 1),
        cmd(
            "charslot",
            &[("slot", "l"), ("name", "avg_npc_032")],
            None,
            2,
        ),
        say("Amiya", "one two three four", 3),
    ];
    let out = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
    assert_eq!(out[0].background.as_deref(), Some("bg_rhodes"));
    assert_eq!(out[0].on_screen, vec!["avg_npc_032".to_owned()]);
    assert_eq!(out[0].scene_ordinal, 1);
    assert_eq!(out[0].content_sha.len(), 16);
}

#[test]
fn strip_backgrounds_named_by_imagegroup_open_scenes() {
    // gridbg / largebg / verticalbg never carry `image`, only `imagegroup`.
    let cmds = vec![
        cmd("background", &[("image", "bg_a")], None, 1),
        say("Amiya", "one two", 2),
        cmd(
            "gridbg",
            &[("imagegroup", "bg_beach_1/bg_beach_2")],
            None,
            3,
        ),
        say("W", "three four", 4),
        // same strip again: not a new scene
        cmd(
            "largebg",
            &[("imagegroup", "bg_beach_1/bg_beach_2")],
            None,
            5,
        ),
        say("W", "five six", 6),
    ];
    let (turns, scene_bg) = turns_of(&cmds);
    let scenes: Vec<u32> = turns.iter().map(|t| t.scene).collect();
    assert_eq!(scenes, vec![1, 2]);
    assert_eq!(
        scene_bg.last().and_then(|(_, b)| b.clone()).as_deref(),
        Some("bg_beach_1/bg_beach_2")
    );
}

#[test]
fn a_cg_is_an_overlay_not_a_scene_change() {
    let cmds = vec![
        cmd("background", &[("image", "bg_rhodes")], None, 1),
        say("Amiya", "before the cg", 2),
        cmd("image", &[("image", "avg_2_2")], None, 3),
        say("Amiya", "during the cg", 4),
        cmd("image", &[], None, 5),
        say("Kal'tsit", "after the cg", 6),
    ];
    let (turns, _) = turns_of(&cmds);
    assert!(
        turns.iter().all(|t| t.scene == 1),
        "a CG must not open a scene"
    );
    let out = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
    assert_eq!(out.len(), 1);
    assert_eq!(
        out[0].background.as_deref(),
        Some("bg_rhodes"),
        "the background field must name the location, never the CG"
    );
}

#[test]
fn a_cutin_joins_the_stage_without_clearing_it() {
    let cmds = vec![
        cmd("character", &[("name", "char_002_amiya")], None, 1),
        cmd("charactercutin", &[("name", "char_003_kalts")], None, 2),
        say("Kal'tsit", "one two", 3),
    ];
    let out = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
    assert_eq!(
        out[0].on_screen,
        vec!["char_002_amiya".to_owned(), "char_003_kalts".to_owned()]
    );
}

#[test]
fn packing_charges_sequence_overhead_and_joins() {
    // Four 3-word turns. With a 12-token ceiling and no overhead, all four
    // fit (12). Charging 2 per sequence and 1 per join, the real cost of
    // n turns is 2 + 3n + (n - 1), so only two fit (2 + 6 + 1 = 9; three
    // would be 13).
    let cmds = vec![
        say("Amiya", "a b", 1),
        say("W", "c d", 2),
        say("Amiya", "e f", 3),
        say("W", "g h", 4),
    ];
    let loose = ChunkConfig {
        target_min_tokens: 100,
        target_max_tokens: 12,
        ..ChunkConfig::default()
    };
    let exact = ChunkConfig {
        sequence_overhead_tokens: 2,
        joiner_tokens: 1,
        ..loose
    };
    assert_eq!(chunk_story("s", "g", &cmds, &loose, &words).len(), 1);
    let out = chunk_story("s", "g", &cmds, &exact, &words);
    assert_eq!(out.len(), 2);
    for c in &out {
        // stored count is the whole-text count plus the sequence overhead
        assert_eq!(c.token_count, words(&c.text) + 2);
        assert!(c.token_count <= 12);
    }
}

#[test]
fn deterministic_across_runs() {
    let cmds = vec![say("W", "alpha bravo", 1), say("Hoederer", "charlie", 2)];
    let a = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
    let b = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
    assert_eq!(a, b);
}
