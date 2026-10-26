use serde::{Deserialize, Serialize};
#[cfg(feature = "schema")]
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRequest {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl PageRequest {
    pub const DEFAULT_LIMIT: u32 = 20;
    pub const MAX_LIMIT: u32 = 100;

    pub fn limit(&self) -> u32 {
        self.limit
            .unwrap_or(Self::DEFAULT_LIMIT)
            .min(Self::MAX_LIMIT)
    }

    pub fn offset(&self) -> u32 {
        self.offset.unwrap_or(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Translator,
    TierListEditor,
    TierListAdmin,
    SuperAdmin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ProfileTab {
    Showcase,
    Stats,
    Score,
    Roster,
    Plans,
    Inventory,
    Enemies,
    Optimizer,
}
