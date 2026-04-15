use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grid {
    pub row: i32,
    pub col: i32,
}

// ============================================================================
// Range
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub id: String,
    pub direction: i32,
    pub grids: Vec<Grid>,
}

// ============================================================================
// Container Types
// ============================================================================

pub type Ranges = HashMap<String, Range>;
