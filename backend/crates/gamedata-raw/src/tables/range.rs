use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grid {
    pub row: i32,
    pub col: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub id: String,
    pub direction: i32,
    pub grids: Vec<Grid>,
}

pub type Ranges = HashMap<String, Range>;
