use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use myrtle_db::repos::content_audit::{
    self as repo, ContentAuditFeedRow, ContentAuditFilter, NewContentAudit,
};
use myrtle_db::{Db, DbError};
use myrtle_types::{ContentAuditAction, ContentTargetKind, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppError;
use crate::policy::{Action, Policy, Resource, Subject};

pub const EDIT_RUN_SECS: f64 = 15.0 * 60.0;
pub const AUDIT_SET_MAX: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditTierRef {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditTierRename {
    pub id: Uuid,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentAuditDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forked_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renamed_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_changed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cells_changed: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers_added: Option<Vec<AuditTierRef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers_renamed: Option<Vec<AuditTierRename>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers_reordered: Option<Vec<AuditTierRef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers_restyled: Option<Vec<AuditTierRef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers_deleted: Option<Vec<AuditTierRef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries_dropped: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries_added: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries_moved: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries_reordered: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries_removed: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes_edited: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changelog: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flair: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiers: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries: Option<u32>,
}

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
        }
        if let (Some(added), Some(renamed)) = (&mut self.tiers_added, &mut self.tiers_renamed) {
            renamed.retain(|r| match added.iter_mut().find(|t| t.id == r.id) {
                Some(tier) => {
                    tier.name.clone_from(&r.to);
                    false
                }
                None => true,
            });
        }
        if self.tiers_renamed.as_ref().is_some_and(Vec::is_empty) {
            self.tiers_renamed = None;
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

#[derive(Debug, Clone)]
pub struct ContentEvent<'a> {
    pub actor: UserId,
    pub target_kind: ContentTargetKind,
    pub target_id: Uuid,
    pub target_name: &'a str,
    pub target_slug: Option<&'a str>,
    pub owner: Option<UserId>,
    pub subject: Option<UserId>,
    pub action: ContentAuditAction,
    pub details: ContentAuditDetails,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditActor {
    pub user_id: Uuid,
    pub uid: Option<String>,
    pub nickname: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditPerson {
    pub user_id: Uuid,
    pub uid: Option<String>,
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentAuditEntry {
    pub id: i64,
    pub action: ContentAuditAction,
    pub target_kind: ContentTargetKind,
    pub target_id: Uuid,
    pub target_name: String,
    pub target_slug: Option<String>,
    pub actor: AuditActor,
    pub owner: Option<AuditPerson>,
    pub subject: Option<AuditPerson>,
    pub details: ContentAuditDetails,
    pub event_count: i32,
    pub first_at: DateTime<Utc>,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentAuditFeed {
    pub entries: Vec<ContentAuditEntry>,
    pub total: i64,
}

#[derive(Debug, Clone)]
pub struct Auditor {
    pub db: Db,
}

impl Auditor {
    pub async fn record(&self, event: ContentEvent<'_>) {
        if event.action == ContentAuditAction::Edit
            && event.details == ContentAuditDetails::default()
        {
            return;
        }
        if let Err(e) = self.write(&event).await {
            tracing::warn!(
                error = %e,
                action = event.action.as_str(),
                target_kind = event.target_kind.as_str(),
                target_id = %event.target_id,
                "content audit write failed"
            );
        }
    }

    async fn write(&self, event: &ContentEvent<'_>) -> Result<(), DbError> {
        let mut tx = self.db.begin().await?;
        let open = if event.action == ContentAuditAction::Edit {
            repo::latest_locked(
                &mut tx,
                event.actor,
                event.target_kind,
                event.target_id,
                EDIT_RUN_SECS,
            )
            .await?
            .filter(|open| open.action == ContentAuditAction::Edit.as_str())
        } else {
            None
        };
        if let Some(open) = open {
            let mut details: ContentAuditDetails =
                serde_json::from_value(open.details).unwrap_or_default();
            details.merge(event.details.clone());
            if details.renamed_from.as_deref() == Some(event.target_name) {
                details.renamed_from = None;
            }
            let details = serde_json::to_value(&details).unwrap_or_default();
            repo::update_coalesced(
                &mut tx,
                open.id,
                event.target_name,
                event.target_slug,
                &details,
            )
            .await?;
        } else {
            let details = serde_json::to_value(&event.details).unwrap_or_default();
            let entry = NewContentAudit {
                actor_id: event.actor,
                target_kind: event.target_kind,
                target_id: event.target_id,
                target_name: event.target_name,
                target_slug: event.target_slug,
                owner_id: event.owner,
                subject_id: event.subject,
                action: event.action,
                details: &details,
            };
            repo::insert(&mut tx, &entry).await?;
        }
        tx.commit().await
    }

    pub async fn feed(
        &self,
        policy: &Policy,
        subject: &Subject,
        filter: ContentAuditFilter,
        limit: i64,
        before: Option<DateTime<Utc>>,
    ) -> Result<ContentAuditFeed, AppError> {
        policy.require(subject, Action::ReadAudit, &Resource::Audit)?;
        let mut conn = self.db.conn().await?;
        let rows = repo::list(&mut conn, filter, limit.clamp(1, 500), before).await?;
        let total = repo::count(&mut conn, filter).await?;
        Ok(ContentAuditFeed {
            entries: rows.into_iter().filter_map(entry).collect(),
            total,
        })
    }
}

fn entry(r: ContentAuditFeedRow) -> Option<ContentAuditEntry> {
    let person = |id: Option<Uuid>, uid, nickname| {
        id.map(|user_id| AuditPerson {
            user_id,
            uid,
            nickname,
        })
    };
    let row = r.row;
    Some(ContentAuditEntry {
        id: row.id,
        action: ContentAuditAction::parse(&row.action)?,
        target_kind: ContentTargetKind::parse(&row.target_kind)?,
        target_id: row.target_id,
        target_name: row.target_name,
        target_slug: row.target_slug,
        actor: AuditActor {
            user_id: row.actor_id,
            uid: r.actor_uid,
            nickname: r.actor_nickname,
            secretary: r.actor_secretary,
            secretary_skin_id: r.actor_secretary_skin_id,
        },
        owner: person(row.owner_id, r.owner_uid, r.owner_nickname),
        subject: person(row.subject_id, r.subject_uid, r.subject_nickname),
        details: serde_json::from_value(row.details).unwrap_or_default(),
        event_count: row.event_count,
        first_at: row.first_at,
        changed_at: row.changed_at,
    })
}
