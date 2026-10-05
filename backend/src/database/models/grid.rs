use serde::{Deserialize, Serialize};
use sqlx::types::{
    Json, Uuid,
    chrono::{DateTime, Utc},
};

use crate::database::models::tier_list::EntityKind;

/// One cell as `grids.cells` stores it. The kind stays a string here so a row
/// holding a kind this build does not know still loads: [`StoredCell::entity`]
/// drops the pick, never the grid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredCell {
    pub label: String,
    pub entity_kind: Option<String>,
    pub entity_id: Option<String>,
}

impl StoredCell {
    /// The cell's `(kind, id)` when both are set and the kind is known.
    pub fn entity(&self) -> Option<(EntityKind, &str)> {
        let (kind, id) = (self.entity_kind.as_deref()?, self.entity_id.as_deref()?);
        match kind.parse() {
            Ok(kind) => Some((kind, id)),
            Err(err) => {
                tracing::warn!(entity_id = %id, "grids.cells: dropping a pick, {err}");
                None
            }
        }
    }
}

/// A `grids` row with what every read shows beside it: the owner's display
/// name, how many grids were forked from it, and the grid it was forked from.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GridRow {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub rows: i16,
    pub cols: i16,
    pub cells: Json<Vec<StoredCell>>,
    pub is_listed: bool,
    /// The kinds a cell may hold, as stored; read through [`GridRow::kinds`].
    pub entity_kinds: Vec<String>,
    pub template_of: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub owner_name: String,
    pub fork_count: i64,
    pub template_slug: Option<String>,
    pub template_title: Option<String>,
    pub template_listed: Option<bool>,
    pub template_owner: Option<Uuid>,
}

impl GridRow {
    /// The allowed kinds this build knows, in canonical order. Lenient like
    /// [`StoredCell::entity`]: a kind this build does not know is skipped with
    /// a warning, and a set with nothing known left reads as every kind, so a
    /// hand edit or a rolled-back release never locks a grid's picks out.
    pub fn kinds(&self) -> Vec<EntityKind> {
        let mut kinds: Vec<EntityKind> = self
            .entity_kinds
            .iter()
            .filter_map(|s| match s.parse() {
                Ok(kind) => Some(kind),
                Err(err) => {
                    tracing::warn!(slug = %self.slug, "grids.entity_kinds: skipping {err}");
                    None
                }
            })
            .collect();
        kinds.sort_unstable();
        kinds.dedup();
        if kinds.is_empty() {
            EntityKind::ALL.to_vec()
        } else {
            kinds
        }
    }
}

/// A grid's stored document: what a create, a save or a fork writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridDocument {
    pub title: String,
    pub description: Option<String>,
    pub rows: i16,
    pub cols: i16,
    pub cells: Vec<StoredCell>,
    pub is_listed: bool,
    /// Allowed kinds, deduplicated and in canonical [`EntityKind`] order.
    pub entity_kinds: Vec<EntityKind>,
}

impl GridDocument {
    /// `entity_kinds` as the `text[]` column stores them.
    pub fn kind_names(&self) -> Vec<&'static str> {
        self.entity_kinds.iter().map(|k| k.as_str()).collect()
    }
}
