use myrtle_engine_music::{ClipTree, MusicSources, MusicTables, StoryScript, bgm_event, level_ids};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::story::StoryVariables;
use myrtle_gamedata_raw::{DirSource, TableSource};

pub fn clip_paths(art: &DirSource) -> Vec<String> {
    art.list_assets()
        .unwrap()
        .into_iter()
        .filter(|p| p.starts_with("audio/"))
        .collect()
}

pub fn music_sources(
    gd: &GameModel,
    own: &DirSource,
    level_trees: &[&DirSource],
    clip_trees: Vec<ClipTree>,
) -> (MusicSources, Vec<String>) {
    let (tables, warnings) = MusicTables::parse(|name| {
        own.read_asset(&format!("gamedata/excel/{name}.json"))
            .unwrap()
    });
    let vars = own
        .read_asset("gamedata/story/story_variables.json.json")
        .unwrap()
        .and_then(|b| StoryVariables::parse(&b).ok())
        .unwrap_or_default();
    let scripts = own
        .list_gamedata("story")
        .unwrap()
        .into_iter()
        .filter(|p| p.ends_with(".txt.txt"))
        .map(|p| {
            let bytes = own.read_asset(&format!("gamedata/{p}")).unwrap().unwrap();
            StoryScript::read(&p["story/".len()..], &bytes, &vars)
        })
        .collect();
    let level_bgm = level_ids(gd)
        .into_iter()
        .filter_map(|id| {
            let rel = format!("gamedata/levels/{id}.json");
            let event = level_trees
                .iter()
                .find_map(|t| bgm_event(&t.read_asset(&rel).ok()??))?;
            Some((id, event))
        })
        .collect();
    let src = MusicSources {
        tables,
        scripts,
        level_bgm,
        clip_trees,
    };
    (src, warnings)
}
