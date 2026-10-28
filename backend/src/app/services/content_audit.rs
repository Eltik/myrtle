//! Who did what to a tier list or a grid: the `content_audit_log` writes and
//! the admin feed that reads them.
//!
//! Writes are best effort. The action they describe has already happened, so
//! a failed audit write is logged and the request still succeeds.
//!
//! Edits coalesce. The tier list editor saves one request per changed tier
//! and placement, and autosaves every few seconds, so a row per request would
//! bury everything else. An `edit` instead folds into the actor's latest row
//! on the same target when that row is an `edit` touched within
//! [`EDIT_RUN_SECS`]: one row reads "moved 3 entries, renamed tier S -> S+"
//! for a whole sitting. Any other action (publish, delete, ...) is its own
//! row and ends the run.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::services::operator_notes::AuditLogActor;
use crate::app::state::AppState;
use crate::database::models::content_audit::{
    ContentAuditAction, ContentAuditDetails, ContentAuditRowWithPeople, ContentTargetKind,
};
use crate::database::models::tier_list::{EntityRef, TierList};
use crate::database::queries::content_audit as queries;
use crate::database::queries::content_audit::{ContentAuditFilter, NewContentAudit};

/// How long an edit run stays open after its last request.
pub const EDIT_RUN_SECS: f64 = 15.0 * 60.0;

/// `kind:id`, the form entry sets are stored in.
pub fn entry_key(entity: EntityRef<'_>) -> String {
    format!("{}:{}", entity.kind.as_str(), entity.id)
}

/// A length as a stored count.
pub fn count_of(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// An entry about `list`, with its current name, slug and owner.
pub fn tier_list_entry(
    list: &TierList,
    actor_id: Uuid,
    action: ContentAuditAction,
    details: ContentAuditDetails,
) -> NewContentAudit<'_> {
    NewContentAudit {
        actor_id,
        target_kind: ContentTargetKind::TierList,
        target_id: list.id,
        target_name: &list.name,
        target_slug: Some(&list.slug),
        owner_id: list.created_by,
        subject_id: None,
        action,
        details,
    }
}

/// Write `entry`, folding an `edit` into the open run. An `edit` that
/// changed nothing is not written.
pub async fn record(state: &AppState, entry: NewContentAudit<'_>) {
    if entry.action == ContentAuditAction::Edit && entry.details == ContentAuditDetails::default() {
        return;
    }
    if let Err(e) = write(state, &entry).await {
        tracing::warn!(
            error = %e,
            action = entry.action.as_str(),
            target_kind = entry.target_kind.as_str(),
            target_id = %entry.target_id,
            "content audit write failed"
        );
    }
}

async fn write(state: &AppState, entry: &NewContentAudit<'_>) -> Result<(), sqlx::Error> {
    let mut tx = state.db.begin().await?;
    if entry.action == ContentAuditAction::Edit
        && let Some(open) = queries::latest_locked(
            &mut tx,
            entry.actor_id,
            entry.target_kind,
            entry.target_id,
            EDIT_RUN_SECS,
        )
        .await?
        && open.action == ContentAuditAction::Edit.as_str()
    {
        let mut details = open.details.0;
        details.merge(entry.details.clone());
        if details.renamed_from.as_deref() == Some(entry.target_name) {
            details.renamed_from = None;
        }
        queries::update_coalesced(
            &mut tx,
            open.id,
            entry.target_name,
            entry.target_slug,
            &details,
        )
        .await?;
    } else {
        queries::insert(&mut tx, entry).await?;
    }
    tx.commit().await
}

/// An account a row names besides its actor. `uid` and `nickname` are `None`
/// once the account is gone.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Serialize)]
pub struct ContentAuditPerson {
    pub user_id: Uuid,
    pub uid: Option<String>,
    pub nickname: Option<String>,
}

/// One row of the content audit feed.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Serialize)]
pub struct ContentAuditEntry {
    #[ts(type = "number")]
    pub id: i64,
    pub action: ContentAuditAction,
    pub target_kind: ContentTargetKind,
    pub target_id: Uuid,
    /// The name when the row was last written. The target may be renamed or
    /// gone since.
    pub target_name: String,
    pub target_slug: Option<String>,
    pub actor: AuditLogActor,
    /// The target's owner at the time; differs from `actor` when staff acted
    /// on someone else's content. `None` when unknown or since deleted.
    pub owner: Option<ContentAuditPerson>,
    /// `grant` / `revoke`: the account the grant was about.
    pub subject: Option<ContentAuditPerson>,
    pub details: ContentAuditDetails,
    /// Requests folded into this row; 1 for anything but a coalesced edit.
    pub event_count: i32,
    pub first_at: chrono::DateTime<chrono::Utc>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Serialize)]
pub struct ContentAuditResponse {
    pub entries: Vec<ContentAuditEntry>,
    #[ts(type = "number")]
    pub total: i64,
}

/// A stored `snake_case` spelling back to its enum; `None` for a value this
/// build does not know.
fn parse<T: for<'de> Deserialize<'de>>(stored: &str) -> Option<T> {
    serde_json::from_value(serde_json::Value::String(stored.to_owned())).ok()
}

fn person(
    id: Option<Uuid>,
    uid: Option<String>,
    nickname: Option<String>,
) -> Option<ContentAuditPerson> {
    id.map(|user_id| ContentAuditPerson {
        user_id,
        uid,
        nickname,
    })
}

fn to_entry(r: ContentAuditRowWithPeople) -> Option<ContentAuditEntry> {
    let row = r.row;
    Some(ContentAuditEntry {
        id: row.id,
        action: parse(&row.action)?,
        target_kind: parse(&row.target_kind)?,
        target_id: row.target_id,
        target_name: row.target_name,
        target_slug: row.target_slug,
        actor: AuditLogActor {
            user_id: row.actor_id,
            uid: r.actor_uid,
            nickname: r.actor_nickname,
            secretary: r.actor_secretary,
            secretary_skin_id: r.actor_secretary_skin_id,
        },
        owner: person(row.owner_id, r.owner_uid, r.owner_nickname),
        subject: person(row.subject_id, r.subject_uid, r.subject_nickname),
        details: row.details.0,
        event_count: row.event_count,
        first_at: row.first_at,
        changed_at: row.changed_at,
    })
}

/// One page of the feed under `filter`, newest first, and the filter's total.
pub async fn feed(
    state: &AppState,
    filter: ContentAuditFilter,
    limit: i64,
    before: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<ContentAuditResponse, ApiError> {
    let limit = limit.clamp(1, 500);
    let (rows, total) = tokio::try_join!(
        queries::list(&state.db, filter, limit, before),
        queries::count(&state.db, filter),
    )?;
    Ok(ContentAuditResponse {
        // A row of a kind or action a newer build wrote is skipped, not a 500.
        entries: rows.into_iter().filter_map(to_entry).collect(),
        total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_spellings_parse_back() {
        assert_eq!(
            parse::<ContentAuditAction>("visibility"),
            Some(ContentAuditAction::Visibility)
        );
        assert_eq!(
            parse::<ContentTargetKind>("tier_list"),
            Some(ContentTargetKind::TierList)
        );
        assert_eq!(parse::<ContentAuditAction>("teleport"), None);
    }
}
