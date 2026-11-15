//! Music index per server into `$MUSIC_DUMP/<server>.json`; `MUSIC_OWN_ART=1` gives every server its own art tree

#![allow(clippy::unwrap_used)]

#[path = "../../engine-music/tests/common/mod.rs"]
mod common;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use myrtle_engine_music::{ClipTree, MusicIndex, build_music_index};
use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::{DirSource, load};
use myrtle_types::Server;

const SERVERS: [(&str, Server); 4] = [
    ("en", Server::EN),
    ("cn", Server::CN),
    ("jp", Server::JP),
    ("kr", Server::KR),
];

/// Production fallback order: the default server, then the rest
const FALLBACKS: [&str; 4] = ["en", "jp", "kr", "cn"];

fn model(root: &Path, server: Server) -> GameModel {
    let raw = load(
        &DirSource {
            root: root.to_path_buf(),
        },
        server,
    )
    .unwrap();
    compile(raw, &CompileOptions::new(server)).unwrap().model
}

fn present(root: &Path, dir: &str) -> bool {
    root.join(dir).join("gamedata/excel").is_dir()
}

/// The default server's art tree stands in for a tree without `portraits`
fn art_dir(root: &Path, dir: &str) -> PathBuf {
    let own = root.join(dir);
    let own_art = std::env::var_os("MUSIC_OWN_ART").is_some();
    if dir == "en" || own_art || own.join("portraits").is_dir() {
        own
    } else {
        root.join("en")
    }
}

fn dump(
    root: &Path,
    dir: &str,
    server: Server,
    clips: &mut HashMap<PathBuf, Vec<String>>,
) -> MusicIndex {
    let gd = model(&root.join(dir), server);
    let others: Vec<&str> = FALLBACKS
        .into_iter()
        .filter(|d| *d != dir && present(root, d))
        .collect();
    let trees: Vec<DirSource> = std::iter::once(dir)
        .chain(others.iter().copied())
        .map(|d| DirSource { root: root.join(d) })
        .collect();
    let mut arts = vec![(None, art_dir(root, dir))];
    for o in &others {
        let art = art_dir(root, o);
        if !arts.iter().any(|(_, a)| *a == art) {
            arts.push((Some((*o).to_owned()), art));
        }
    }
    let clip_trees = arts
        .into_iter()
        .map(|(server, art)| ClipTree {
            server,
            paths: clips
                .entry(art.clone())
                .or_insert_with(|| common::clip_paths(&DirSource { root: art }))
                .clone(),
        })
        .collect();
    let level_trees: Vec<&DirSource> = trees.iter().collect();
    let (src, warnings) = common::music_sources(&gd, &trees[0], &level_trees, clip_trees);
    for w in warnings {
        println!("[{dir}] {w}");
    }
    build_music_index(&gd, &src)
}

#[test]
fn music_dump() {
    let Ok(out) = std::env::var("MUSIC_DUMP") else {
        return;
    };
    let root = PathBuf::from(std::env::var("MUSIC_ROOT").unwrap());
    let out = PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    let mut clips = HashMap::new();
    for (dir, server) in SERVERS {
        if !present(&root, dir) {
            continue;
        }
        let index = dump(&root, dir, server, &mut clips);
        let t = &index.totals;
        println!(
            "[{dir}] tracks={} titled={} with_audio={} unavailable={} from_other_server={}",
            t.tracks, t.titled, t.with_audio, t.unavailable, t.from_other_server
        );
        std::fs::write(
            out.join(format!("{dir}.json")),
            serde_json::to_vec_pretty(&index).unwrap(),
        )
        .unwrap();
    }
}
