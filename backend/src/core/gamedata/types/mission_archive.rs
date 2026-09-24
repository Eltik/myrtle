//! `activity_table.MissionArchives`: the RECORDINGS shelf, a set of voice
//! clips an act unlocks as the player clears its missions.
//!
//! EN carries exactly one, `mission_archive_main_14`, which belongs to the
//! mainline chapter `main_14` rather than to an event: 5 nodes of 35 clips
//! plus 8 hidden ones, every clip Cetsyr's (`char_4134_cetsyr`). It is not
//! part of `story_review_meta_table` at all, so it is read here and served
//! beside the archive sections of the group its `Zones` names.
//!
//! The table is read from `activity_table.json` a SECOND time, with only this
//! field kept, because `ActivityTableFile` does not carry it and that type
//! belongs to the event code.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::core::gamedata::tables::SanitizingReader;

use super::serde_helpers::deserialize_fb_map_or_default;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MissionArchiveTableFile {
    #[serde(
        alias = "MissionArchives",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub mission_archives: HashMap<String, MissionArchive>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MissionArchive {
    #[serde(alias = "TopicId", default)]
    pub topic_id: String,
    #[serde(alias = "UnlockDesc", default)]
    pub unlock_desc: String,
    /// The zones the shelf belongs to; `main_14` is also the story group's id.
    #[serde(alias = "Zones", default)]
    pub zones: Vec<String>,
    #[serde(alias = "Nodes", default)]
    pub nodes: Vec<MissionArchiveNode>,
    #[serde(alias = "HiddenClips", default)]
    pub hidden_clips: Vec<MissionArchiveClip>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MissionArchiveNode {
    #[serde(alias = "NodeId", default)]
    pub node_id: String,
    #[serde(alias = "Title", default)]
    pub title: String,
    #[serde(alias = "UnlockDesc", default)]
    pub unlock_desc: String,
    #[serde(alias = "Clips", default)]
    pub clips: Vec<MissionArchiveClip>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MissionArchiveClip {
    #[serde(alias = "CharId", default)]
    pub char_id: String,
    #[serde(alias = "VoiceId", default)]
    pub voice_id: String,
    #[serde(alias = "Index", default)]
    pub index: i32,
}

/// Read `MissionArchives` out of `activity_table.json`. A missing or
/// malformed file is a WARNING and an empty map, never a boot failure, by the
/// same rule `load_table_or_warn` follows for every other table.
pub fn load_mission_archives(
    data_dir: &Path,
    warnings: &mut Vec<String>,
) -> HashMap<String, MissionArchive> {
    let path = data_dir.join("activity_table.json");
    let file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            warnings.push(format!("mission_archives: {e}"));
            return HashMap::new();
        }
    };
    let reader = SanitizingReader::new(std::io::BufReader::new(file));
    match serde_json::from_reader::<_, MissionArchiveTableFile>(reader) {
        Ok(t) => t.mission_archives,
        Err(e) => {
            warnings.push(format!("mission_archives: {e}"));
            HashMap::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_archive_reads_its_nodes_and_hidden_clips() {
        let raw = r#"{"MissionArchives": [{"key": "mission_archive_main_14", "value": {
            "TopicId": "mission_archive_main_14",
            "UnlockDesc": "Unlocks after clearing 14-23",
            "Zones": ["main_14"],
            "Nodes": [{"NodeId": "main_node_1", "Title": "In Commemoration of Setting Sail",
                "UnlockDesc": "Clear H14-1", "Clips": [
                    {"CharId": "char_4134_cetsyr", "VoiceId": "EX_CN_101", "Index": 1}
                ]}],
            "HiddenClips": [{"CharId": "char_4134_cetsyr", "VoiceId": "EX_CN_601", "Index": 1}]
        }}]}"#;
        let file: MissionArchiveTableFile = serde_json::from_str(raw).unwrap();
        let a = &file.mission_archives["mission_archive_main_14"];
        assert_eq!(a.zones, ["main_14"]);
        assert_eq!(a.nodes.len(), 1);
        assert_eq!(a.nodes[0].clips[0].voice_id, "EX_CN_101");
        assert_eq!(a.hidden_clips.len(), 1);
    }

    #[test]
    fn a_table_without_the_field_is_empty() {
        let file: MissionArchiveTableFile = serde_json::from_str(r#"{"Activity": []}"#).unwrap();
        assert!(file.mission_archives.is_empty());
    }
}
