use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct Grid {
    pub row: i32,
    pub col: i32,
}

// ============================================================================
// Range
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct Range {
    pub id: String,
    pub direction: i32,
    pub grids: Vec<Grid>,
}

// ============================================================================
// Container Types
// ============================================================================

pub type Ranges = HashMap<String, Range>;
