use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Chibi Types - Spine animation data for operators
// ============================================================================

/// Animation types for different views/poses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimationType {
    Front,
    Back,
    Dorm,
    Dynamic,
}

impl AnimationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnimationType::Front => "front",
            AnimationType::Back => "back",
            AnimationType::Dorm => "dorm",
            AnimationType::Dynamic => "dynamic",
        }
    }
}

/// Spine files for an animation type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpineFiles {
    pub atlas: Option<String>,
    pub skel: Option<String>,
    pub png: Option<String>,
}

impl SpineFiles {
    pub fn has_data(&self) -> bool {
        self.atlas.is_some() || self.skel.is_some() || self.png.is_some()
    }
}

/// Character skin with different animation types
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChibiSkin {
    pub name: String,
    pub path: String,
    pub has_spine_data: bool,
    pub animation_types: HashMap<String, SpineFiles>,
}

/// Processed character data for frontend
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChibiCharacter {
    pub operator_code: String,
    pub name: String,
    pub path: String,
    pub skins: Vec<ChibiSkin>,
}

// ============================================================================
// Internal crawling types (not serialized to frontend)
// ============================================================================

/// Content type for repo items
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Dir,
    File,
}

/// Repository item (internal structure for crawling)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoItem {
    pub name: String,
    pub path: String,
    pub content_type: ContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<RepoItem>>,
}

impl RepoItem {
    pub fn new_dir(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            content_type: ContentType::Dir,
            children: Some(Vec::new()),
        }
    }

    pub fn new_file(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            content_type: ContentType::File,
            children: None,
        }
    }
}

/// Cached chibi data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedChibiData {
    pub timestamp: i64,
    pub data: Vec<RepoItem>,
    pub version: u32,
}

// ============================================================================
// Container type
// ============================================================================

/// All chibi data
/// Uses Arc<ChibiCharacter> to share data between Vec and HashMap without cloning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChibiData {
    /// Raw repo items from crawling
    #[serde(skip)]
    pub raw_items: Vec<RepoItem>,
    /// Processed character data for frontend (uses Arc for zero-copy sharing)
    #[serde(serialize_with = "serialize_arc_vec")]
    pub characters: Vec<Arc<ChibiCharacter>>,
    /// Lookup by operator code (shares Arc with characters vec)
    #[serde(skip)]
    pub by_operator: HashMap<String, Arc<ChibiCharacter>>,
}

/// Custom serializer to serialize Vec<Arc<T>> as Vec<T>
fn serialize_arc_vec<S>(data: &[Arc<ChibiCharacter>], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(data.len()))?;
    for item in data {
        seq.serialize_element(item.as_ref())?;
    }
    seq.end()
}

impl ChibiData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_by_operator(&self, operator_code: &str) -> Option<&ChibiCharacter> {
        self.by_operator.get(operator_code).map(|arc| arc.as_ref())
    }

    pub fn is_loaded(&self) -> bool {
        !self.characters.is_empty()
    }
}
