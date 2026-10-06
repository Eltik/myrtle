//! The story names the prompts quote (from the backend, cached in `names.json`), and the generator's and
//! judge's user prompts.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use anyhow::{Context, Result};
use trevor::eval::goldset::Stratum;
use trevor::goldgen::sample::SampleItem;
use trevor::search::store::ChunkStore;


#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub(crate) struct Names {
    pub(crate) story: String,
    pub(crate) group: String,
}

pub(crate) fn load_names(work: &Path) -> HashMap<String, Names> {
    std::fs::read(work.join("names.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

pub(crate) fn label(names: &HashMap<String, Names>, story_id: &str) -> String {
    match names.get(story_id) {
        Some(n) if !n.story.is_empty() => format!("{} ({})", n.story, n.group),
        _ => story_id.to_owned(),
    }
}

pub(crate) fn user_prompt(
    item: &SampleItem,
    store: &ChunkStore,
    names: &HashMap<String, Names>,
) -> Result<String> {
    let chunk = |id: &str| -> Result<&trevor::corpus::chunk::Chunk> {
        Ok(&store.chunks[store
            .row(id)
            .with_context(|| format!("{id} not in chunks.jsonl"))?])
    };
    if item.stratum == Stratum::MultiHop {
        let (a, b) = (chunk(&item.chunk_ids[0])?, chunk(&item.chunk_ids[1])?);
        Ok(format!(
            "CHARACTER IN BOTH: {}\n\nPASSAGE A (from {}):\n{}\n\nPASSAGE B (from {}):\n{}",
            item.entity.as_deref().unwrap_or(""),
            label(names, &a.story_id),
            a.text,
            label(names, &b.story_id),
            b.text
        ))
    } else {
        let c = chunk(&item.chunk_ids[0])?;
        Ok(format!(
            "STORY: {}\nQUESTION TYPE: {}\n\nPASSAGE:\n{}",
            label(names, &c.story_id),
            item.stratum.name(),
            c.text
        ))
    }
}

pub(crate) async fn fetch_names(base: &str) -> Result<(HashMap<String, Names>, BTreeSet<String>)> {
    let v: serde_json::Value = reqwest::get(format!("{base}/api/story/index"))
        .await?
        .error_for_status()?
        .json()
        .await?;
    let mut out = HashMap::new();
    let operators: BTreeSet<String> = v["records"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| r["name"].as_str().map(str::to_owned))
        .collect();
    for g in v["groups"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(v["records"].as_array().into_iter().flatten())
    {
        let group = g["name"]
            .as_str()
            .or_else(|| g["charName"].as_str())
            .unwrap_or("")
            .to_owned();
        for s in g["stories"].as_array().into_iter().flatten() {
            if let Some(id) = s["id"].as_str() {
                out.entry(id.to_owned()).or_insert_with(|| Names {
                    story: s["name"].as_str().unwrap_or("").to_owned(),
                    group: group.clone(),
                });
            }
        }
    }
    Ok((out, operators))
}

pub(crate) fn passage_prompt(passage: &str, question: &str) -> String {
    format!("PASSAGE:\n{passage}\n\nQUESTION: {question}")
}
