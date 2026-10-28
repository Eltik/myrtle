use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};
use ts_rs::TS;

/// What a `content_audit_log` row is about.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentTargetKind {
    TierList,
    Grid,
}

impl ContentTargetKind {
    /// The stored spelling, the same string serde reads and writes.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TierList => "tier_list",
            Self::Grid => "grid",
        }
    }
}

/// What was done. `Edit` rows coalesce (see `services::content_audit`);
/// every other action is one row.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentAuditAction {
    Create,
    /// A grid made from another as a template.
    Fork,
    /// Name, description, kinds, tiers, placements or cells.
    Edit,
    /// A tier list version frozen.
    Publish,
    /// Shown in or hidden from the public index.
    Visibility,
    /// A flair set or cleared.
    Flair,
    /// An edit grant given on a tier list.
    Grant,
    /// An edit grant withdrawn.
    Revoke,
    Delete,
}

impl ContentAuditAction {
    /// The stored spelling, the same string serde reads and writes.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Fork => "fork",
            Self::Edit => "edit",
            Self::Publish => "publish",
            Self::Visibility => "visibility",
            Self::Flair => "flair",
            Self::Grant => "grant",
            Self::Revoke => "revoke",
            Self::Delete => "delete",
        }
    }
}

/// One tier, by id and its name at the time.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditTierRef {
    pub id: Uuid,
    pub name: String,
}

/// A tier's name before and after a run of edits.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditTierRename {
    pub id: Uuid,
    pub from: String,
    pub to: String,
}

/// Most entries of one kind of change a row keeps. Past it the set stops
/// growing and the count reads as "at least".
pub const AUDIT_SET_MAX: usize = 500;

/// What one row records, every field optional so the stored JSON holds only
/// what happened. An `edit` row accumulates: sets of entries and tiers are
/// deduplicated (moving one operator five times is one entry moved), renames
/// keep the first `from` and the last `to`.
///
/// Entries are `kind:id` strings (`operator:char_002_amiya`).
#[derive(TS, utoipa::ToSchema)]
#[ts(export, optional_fields)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentAuditDetails {
    /// `create`: the tier list's type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_type: Option<String>,
    /// `fork`: the source grid's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forked_from: Option<String>,
    /// `edit`: the name before the run's first rename; the row's
    /// `target_name` is the name after.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renamed_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_changed: Option<bool>,
    /// `edit`: the kinds the editor offers, after the change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<String>>,
    /// `visibility`, or a grid `edit` that changed its listing: listed after.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listed: Option<bool>,
    /// Grid `edit`: `rows x cols` after a resize.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Grid `edit`: cells whose label or pick changed, summed over the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cells_changed: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers_added: Option<Vec<AuditTierRef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers_renamed: Option<Vec<AuditTierRename>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers_reordered: Option<Vec<AuditTierRef>>,
    /// Colour or description changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers_restyled: Option<Vec<AuditTierRef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers_deleted: Option<Vec<AuditTierRef>>,
    /// Placements removed with a deleted tier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries_dropped: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries_added: Option<Vec<String>>,
    /// Moved to another tier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries_moved: Option<Vec<String>>,
    /// Moved within their tier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries_reordered: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries_removed: Option<Vec<String>>,
    /// A placement's note changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes_edited: Option<Vec<String>>,
    /// `publish`: the new version number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changelog: Option<String>,
    /// `flair`: the flair's label, absent when it was cleared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flair: Option<String>,
    /// `grant` / `revoke`: the permission level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<String>,
    /// `delete`: how many tiers and placements went with it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries: Option<u32>,
}

/// `into` with `more` appended, first sighting kept, capped at
/// [`AUDIT_SET_MAX`]. `None` stays `None` when there is nothing to add.
fn union_strings(into: &mut Option<Vec<String>>, more: Option<Vec<String>>) {
    let Some(more) = more else { return };
    let list = into.get_or_insert_with(Vec::new);
    let mut seen: BTreeSet<String> = list.iter().cloned().collect();
    for s in more {
        if list.len() >= AUDIT_SET_MAX {
            break;
        }
        if seen.insert(s.clone()) {
            list.push(s);
        }
    }
}

/// [`union_strings`] for tiers, keyed by id; a later sighting updates the name.
fn union_tiers(into: &mut Option<Vec<AuditTierRef>>, more: Option<Vec<AuditTierRef>>) {
    let Some(more) = more else { return };
    let list = into.get_or_insert_with(Vec::new);
    for tier in more {
        if let Some(existing) = list.iter_mut().find(|t| t.id == tier.id) {
            existing.name = tier.name;
        } else if list.len() < AUDIT_SET_MAX {
            list.push(tier);
        }
    }
}

fn add_count(into: &mut Option<u32>, more: Option<u32>) {
    if let Some(more) = more {
        *into = Some(into.unwrap_or(0).saturating_add(more));
    }
}

impl ContentAuditDetails {
    /// Fold a later `edit` into this one.
    ///
    /// A tier renamed back to its first name drops out of `tiers_renamed`,
    /// and a list renamed back drops `renamed_from` (the caller compares it
    /// with the current name, which this struct does not hold).
    pub fn merge(&mut self, later: Self) {
        if self.renamed_from.is_none() {
            self.renamed_from = later.renamed_from;
        }
        if later.description_changed == Some(true) {
            self.description_changed = Some(true);
        }
        if later.kinds.is_some() {
            self.kinds = later.kinds;
        }
        if later.listed.is_some() {
            self.listed = later.listed;
        }
        if later.size.is_some() {
            self.size = later.size;
        }
        add_count(&mut self.cells_changed, later.cells_changed);

        union_tiers(&mut self.tiers_added, later.tiers_added);
        if let Some(renames) = later.tiers_renamed {
            let list = self.tiers_renamed.get_or_insert_with(Vec::new);
            for rename in renames {
                if let Some(existing) = list.iter_mut().find(|r| r.id == rename.id) {
                    existing.to = rename.to;
                } else if list.len() < AUDIT_SET_MAX {
                    list.push(rename);
                }
            }
            list.retain(|r| r.from != r.to);
            if list.is_empty() {
                self.tiers_renamed = None;
            }
        }
        // A tier added in this run then renamed reads as added under its
        // latest name, not as a rename.
        if let (Some(added), Some(renamed)) = (&mut self.tiers_added, &mut self.tiers_renamed) {
            renamed.retain(|r| match added.iter_mut().find(|t| t.id == r.id) {
                Some(tier) => {
                    tier.name.clone_from(&r.to);
                    false
                }
                None => true,
            });
            if renamed.is_empty() {
                self.tiers_renamed = None;
            }
        }
        union_tiers(&mut self.tiers_reordered, later.tiers_reordered);
        union_tiers(&mut self.tiers_restyled, later.tiers_restyled);
        union_tiers(&mut self.tiers_deleted, later.tiers_deleted);
        add_count(&mut self.entries_dropped, later.entries_dropped);

        union_strings(&mut self.entries_added, later.entries_added);
        union_strings(&mut self.entries_moved, later.entries_moved);
        union_strings(&mut self.entries_reordered, later.entries_reordered);
        union_strings(&mut self.entries_removed, later.entries_removed);
        union_strings(&mut self.notes_edited, later.notes_edited);
    }
}

/// One `content_audit_log` row as stored.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ContentAuditRow {
    pub id: i64,
    pub actor_id: Uuid,
    pub target_kind: String,
    pub target_id: Uuid,
    pub target_name: String,
    pub target_slug: Option<String>,
    pub owner_id: Option<Uuid>,
    pub subject_id: Option<Uuid>,
    pub action: String,
    pub details: sqlx::types::Json<ContentAuditDetails>,
    pub event_count: i32,
    pub first_at: DateTime<Utc>,
    pub changed_at: DateTime<Utc>,
}

/// A row with the people it names, for the admin feed.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ContentAuditRowWithPeople {
    #[sqlx(flatten)]
    pub row: ContentAuditRow,
    pub actor_uid: Option<String>,
    pub actor_nickname: Option<String>,
    pub actor_secretary: Option<String>,
    pub actor_secretary_skin_id: Option<String>,
    pub owner_uid: Option<String>,
    pub owner_nickname: Option<String>,
    pub subject_uid: Option<String>,
    pub subject_nickname: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tier(n: u128, name: &str) -> AuditTierRef {
        AuditTierRef {
            id: Uuid::from_u128(n),
            name: name.into(),
        }
    }

    fn strings(items: &[&str]) -> Option<Vec<String>> {
        Some(items.iter().map(|s| (*s).to_owned()).collect())
    }

    /// Autosave sends one request per placement: a run of them reads as the
    /// distinct entries touched, not the requests.
    #[test]
    fn entries_moved_twice_count_once() {
        let mut run = ContentAuditDetails {
            entries_moved: strings(&["operator:a", "operator:b"]),
            ..Default::default()
        };
        run.merge(ContentAuditDetails {
            entries_moved: strings(&["operator:b", "operator:c"]),
            entries_added: strings(&["operator:d"]),
            ..Default::default()
        });
        assert_eq!(
            run.entries_moved,
            strings(&["operator:a", "operator:b", "operator:c"])
        );
        assert_eq!(run.entries_added, strings(&["operator:d"]));
        assert_eq!(run.entries_removed, None);
    }

    #[test]
    fn a_rename_chain_keeps_the_first_from_and_the_last_to() {
        let id = Uuid::from_u128(1);
        let rename = |from: &str, to: &str| ContentAuditDetails {
            tiers_renamed: Some(vec![AuditTierRename {
                id,
                from: from.into(),
                to: to.into(),
            }]),
            ..Default::default()
        };
        let mut run = rename("S", "S+");
        run.merge(rename("S+", "SS"));
        assert_eq!(run.tiers_renamed.as_ref().unwrap()[0].from, "S");
        assert_eq!(run.tiers_renamed.as_ref().unwrap()[0].to, "SS");
        run.merge(rename("SS", "S"));
        assert_eq!(run.tiers_renamed, None, "renamed back is no rename");
    }

    #[test]
    fn a_tier_added_then_renamed_reads_as_added() {
        let mut run = ContentAuditDetails {
            tiers_added: Some(vec![tier(7, "New tier")]),
            ..Default::default()
        };
        run.merge(ContentAuditDetails {
            tiers_renamed: Some(vec![AuditTierRename {
                id: Uuid::from_u128(7),
                from: "New tier".into(),
                to: "EX".into(),
            }]),
            ..Default::default()
        });
        assert_eq!(run.tiers_added, Some(vec![tier(7, "EX")]));
        assert_eq!(run.tiers_renamed, None);
    }

    #[test]
    fn a_reorder_parks_and_settles_once() {
        // The editor parks a tier at 10000 + i, then settles it: two requests.
        let mut run = ContentAuditDetails {
            tiers_reordered: Some(vec![tier(1, "A")]),
            ..Default::default()
        };
        run.merge(ContentAuditDetails {
            tiers_reordered: Some(vec![tier(1, "A"), tier(2, "B")]),
            ..Default::default()
        });
        assert_eq!(run.tiers_reordered, Some(vec![tier(1, "A"), tier(2, "B")]));
    }

    #[test]
    fn counts_and_flags_accumulate() {
        let mut run = ContentAuditDetails {
            cells_changed: Some(2),
            renamed_from: Some("Old".into()),
            ..Default::default()
        };
        run.merge(ContentAuditDetails {
            cells_changed: Some(3),
            renamed_from: Some("Middle".into()),
            description_changed: Some(true),
            ..Default::default()
        });
        assert_eq!(run.cells_changed, Some(5));
        assert_eq!(run.renamed_from.as_deref(), Some("Old"));
        assert_eq!(run.description_changed, Some(true));
    }

    #[test]
    fn sets_stop_at_the_cap() {
        let many: Vec<String> = (0..AUDIT_SET_MAX + 10)
            .map(|i| format!("operator:{i}"))
            .collect();
        let mut run = ContentAuditDetails::default();
        run.merge(ContentAuditDetails {
            entries_added: Some(many),
            ..Default::default()
        });
        assert_eq!(run.entries_added.unwrap().len(), AUDIT_SET_MAX);
    }

    #[test]
    fn an_empty_row_stores_as_an_empty_object() {
        let json = serde_json::to_string(&ContentAuditDetails::default()).unwrap();
        assert_eq!(json, "{}");
    }
}
