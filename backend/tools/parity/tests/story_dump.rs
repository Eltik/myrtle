#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use myrtle_engine_story::parser::{parse, parse_with_stats, word_count};
use myrtle_engine_story::speakers::{
    PlateRule, SpeakerRules, SpeakerWalk, attribute_speakers, attribute_speakers_with,
    is_meaningful_name,
};
use myrtle_gamedata_raw::tables::story::StoryVariables;

/// Main reads the table by path and boots with an empty one when it is
/// missing or malformed; the same fallback here.
fn variables(root: &Path) -> StoryVariables {
    std::fs::read(root.join("gamedata/story/story_variables.json.json"))
        .ok()
        .and_then(|bytes| StoryVariables::parse(&bytes).ok())
        .unwrap_or_default()
}

/// FNV-1a over the bytes, enough to compare two dumps line by line.
fn digest(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn scripts(dir: &Path, rel: &str, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        if path.is_dir() {
            scripts(&path, &rel, out);
        } else if name.ends_with(".txt.txt") {
            out.push((rel, path));
        }
    }
}

/// A stand-in for the sprite index, the same on both sides: the name up to its
/// expression suffix is the folder, and one name in seven has no folder.
fn folder_of(raw: &str) -> Option<String> {
    let folder = raw
        .split(['#', '$'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    (!folder.is_empty() && !digest(&folder).is_multiple_of(7)).then_some(folder)
}

fn walk_line(tag: &str, walk: &SpeakerWalk, text: &mut String) {
    let named = walk
        .lines
        .iter()
        .filter(|l| is_meaningful_name(&l.speaker))
        .count();
    write!(
        text,
        "\t{tag} {} {named} {} {:016x}",
        walk.lines.len(),
        walk.placements.len(),
        digest(&format!("{walk:?}")),
    )
    .unwrap();
}

fn dump(root: &Path) -> String {
    let mut text = String::new();
    let vars = variables(root);
    let mut keys = BTreeMap::new();
    let table: serde_json::Value =
        std::fs::read(root.join("gamedata/story/story_variables.json.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
    for key in table.as_object().into_iter().flatten().map(|(k, _)| k) {
        keys.insert(
            key.clone(),
            vars.resolve(&format!("${key}")).map(str::to_owned),
        );
    }
    writeln!(
        text,
        "variables\t{}\t{:016x}",
        vars.len(),
        digest(&format!("{keys:?}"))
    )
    .unwrap();

    let mut files = Vec::new();
    scripts(&root.join("gamedata/story"), "", &mut files);
    files.sort();
    for (rel, path) in files {
        let script = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
        let (commands, stats) = parse_with_stats(&script);
        assert_eq!(commands, parse(&script), "{rel}");
        write!(
            text,
            "{rel}\t{} {} {:016x}\t{} {} {} {} {:016x}",
            commands.len(),
            word_count(&commands),
            digest(&serde_json::to_string(&commands).unwrap()),
            stats.continued_lines,
            stats.decoded_values,
            stats.dropped_arg_blocks,
            stats.diagnostics.len(),
            digest(&format!("{:?}", stats.diagnostics)),
        )
        .unwrap();
        walk_line(
            "fallback",
            &attribute_speakers(&commands, folder_of),
            &mut text,
        );
        for (tag, plates) in [
            ("exclusive", PlateRule::Exclusive),
            ("join", PlateRule::Join),
        ] {
            let walk = attribute_speakers_with(&commands, SpeakerRules { plates }, folder_of);
            walk_line(tag, &walk, &mut text);
        }
        text.push('\n');
    }
    text
}

#[test]
fn story_dump() {
    let Ok(out) = std::env::var("STORY_DUMP") else {
        return;
    };
    let root = PathBuf::from(std::env::var("STORY_ROOT").unwrap());
    let out = PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    for dir in ["en", "cn", "jp", "kr"] {
        let root = root.join(dir);
        if !root.join("gamedata/story").is_dir() {
            continue;
        }
        std::fs::write(out.join(format!("{dir}.tsv")), dump(&root)).unwrap();
    }
}
