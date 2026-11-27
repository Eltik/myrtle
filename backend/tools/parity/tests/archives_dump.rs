#![allow(clippy::unwrap_used)]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use myrtle_engine_archives::profile::Lang;
use myrtle_engine_archives::{
    Archive, ArchiveAppearance, SceneIndex, build::file_ids, build_archive,
};
use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata::model::GameModel as Data;
use myrtle_gamedata_raw::tables::handbook_network::RawNetwork;
use myrtle_gamedata_raw::{DirSource, TableSource, load};
use myrtle_types::Server;

fn model(root: PathBuf, server: Server) -> Data {
    let raw = load(&DirSource { root }, server).unwrap();
    compile(raw, &CompileOptions::new(server)).unwrap().model
}

fn network(root: &Path) -> Option<RawNetwork> {
    let source = DirSource {
        root: root.to_path_buf(),
    };
    let force_map = source
        .read_gamedata("art/handbook_force_map_table.json.json")
        .ok()??;
    let positions = source
        .read_gamedata("art/handbookpos_table.json.json")
        .ok()??;
    RawNetwork::from_slices(&force_map, &positions).ok()
}

/// A scene index made from the file ids alone, so both sides build the same
/// one without the story engine: every operator speaks in three of 61 stories.
fn scenes(data: &Data) -> SceneIndex {
    file_ids(data)
        .into_iter()
        .enumerate()
        .map(|(i, id)| {
            let rows = [i % 61, (i * 7 + 3) % 61, (i * 13 + 11) % 61]
                .into_iter()
                .enumerate()
                .map(|(k, story)| ArchiveAppearance {
                    story_id: format!("story_{story:02}"),
                    name: format!("Story {story}"),
                    code: (story % 2 == 0).then(|| format!("ST-{story}")),
                    tag: (story % 3 == 0).then(|| "tag".to_string()),
                    group_id: format!("group_{}", story % 9),
                    group_name: format!("Group {}", story % 9),
                    #[allow(clippy::cast_precision_loss)]
                    lines: ((i % 5) as f64).mul_add(0.25, 1.0 + k as f64),
                })
                .collect();
            (id, rows)
        })
        .collect()
}

/// The archive's five parts, one JSON value per line, operators in id order.
fn lines(tag: &str, a: &Archive, text: &mut String) {
    writeln!(
        text,
        "{tag} network\t{}",
        serde_json::to_string(&a.network).unwrap()
    )
    .unwrap();
    writeln!(
        text,
        "{tag} dossiers\t{}",
        serde_json::to_string(&a.dossiers).unwrap()
    )
    .unwrap();
    writeln!(
        text,
        "{tag} demographics\t{}",
        serde_json::to_string(&a.demographics).unwrap()
    )
    .unwrap();
    writeln!(
        text,
        "{tag} hidden\t{}",
        serde_json::to_string(&a.hidden).unwrap()
    )
    .unwrap();
    let mut ids: Vec<&String> = a.operators.keys().collect();
    ids.sort();
    for id in ids {
        writeln!(
            text,
            "{tag} op {id}\t{}",
            serde_json::to_string(&a.operators[id]).unwrap()
        )
        .unwrap();
    }
}

fn dump(root: &Path, server: Server) -> String {
    let data = model(root.to_path_buf(), server);
    let net = network(root);
    let lang = Lang::for_server(server);
    let mut text = String::new();
    let bare = build_archive(
        &data,
        &data.factions,
        net.as_ref(),
        &SceneIndex::new(),
        lang,
    );
    lines("bare", &bare, &mut text);
    let full = build_archive(&data, &data.factions, net.as_ref(), &scenes(&data), lang);
    lines("scenes", &full, &mut text);
    let unread = build_archive(&data, &data.factions, None, &SceneIndex::new(), None);
    lines("unread", &unread, &mut text);
    text
}

#[test]
fn archives_dump() {
    let Ok(out) = std::env::var("ARCHIVES_DUMP") else {
        return;
    };
    let root = PathBuf::from(std::env::var("ARCHIVES_ROOT").unwrap());
    let out = PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    for (dir, server) in [
        ("en", Server::EN),
        ("cn", Server::CN),
        ("jp", Server::JP),
        ("kr", Server::KR),
    ] {
        let root = root.join(dir);
        if !root.join("gamedata/excel").is_dir() {
            continue;
        }
        std::fs::write(out.join(format!("{dir}.tsv")), dump(&root, server)).unwrap();
    }
}
