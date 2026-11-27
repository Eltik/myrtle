use myrtle_db::repos::tier_lists::{self as repo, TierRow};
use myrtle_types::{ContentAuditAction, EntityKind, TierListType};
use serde::Deserialize;
use uuid::Uuid;

use super::{
    Tier, TierList, TierListVersion, TierPlacement, actor, audit_event, count_of,
    find_and_authorize, invalidate_detail, list_id, load_detail, snapshot_of, validate_entity,
};
use crate::audit::{AuditTierRef, AuditTierRename, ContentAuditDetails};
use crate::policy::{Action, Resource, Subject};
use crate::{AppContext, AppError};

pub const TIER_NAME_MAX: usize = 40;
pub const TIER_DESCRIPTION_MAX: usize = 1000;
pub const LIST_NAME_MAX: usize = 80;
pub const LIST_DESCRIPTION_MAX: usize = 4000;
pub const PLACEMENT_DESCRIPTION_MAX: usize = 1000;
pub const MAX_LISTS_PER_USER: i64 = 10;
const CHANGELOG_AUDIT_MAX: usize = 280;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateListRequest {
    pub name: String,
    pub description: Option<String>,
    pub list_type: TierListType,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateListRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub entity_kinds: Option<Vec<EntityKind>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TierRequest {
    pub name: String,
    pub display_order: i16,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddPlacementRequest {
    pub tier_id: Uuid,
    pub entity_kind: EntityKind,
    pub entity_id: String,
    pub sub_order: Option<i16>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MovePlacementRequest {
    pub new_tier_id: Uuid,
    pub sub_order: Option<i16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublishRequest {
    pub changelog: Option<String>,
}

pub fn validate_hex_color(value: Option<&str>) -> Result<(), AppError> {
    let Some(color) = value else { return Ok(()) };
    let bytes = color.as_bytes();
    let valid =
        bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(u8::is_ascii_hexdigit);
    if valid {
        Ok(())
    } else {
        Err(AppError::bad_request("invalid color"))
    }
}

pub fn validate_length(field: &str, value: Option<&str>, max: usize) -> Result<(), AppError> {
    match value {
        Some(v) if v.chars().count() > max => {
            Err(AppError::BadRequest(format!("{field} max {max} chars")))
        }
        _ => Ok(()),
    }
}

fn validate_list(name: &str, description: Option<&str>) -> Result<(), AppError> {
    validate_length("list name", Some(name), LIST_NAME_MAX)?;
    validate_length("list description", description, LIST_DESCRIPTION_MAX)
}

fn validate_tier(tier: &TierRequest) -> Result<(), AppError> {
    validate_length("tier name", Some(&tier.name), TIER_NAME_MAX)?;
    validate_length(
        "tier description",
        tier.description.as_deref(),
        TIER_DESCRIPTION_MAX,
    )?;
    validate_hex_color(tier.color.as_deref())
}

#[must_use]
pub fn generate_slug(name: &str) -> String {
    let base = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let suffix: String = (0..6)
        .map(|_| {
            let n = rand::random::<u8>() % 36;
            char::from(if n < 10 { b'0' + n } else { b'a' + n - 10 })
        })
        .collect();
    format!("{base}-{suffix}")
}

pub fn offered_kinds(kinds: &[EntityKind]) -> Result<Vec<EntityKind>, AppError> {
    let mut out: Vec<EntityKind> = Vec::with_capacity(kinds.len());
    for kind in kinds {
        if !out.contains(kind) {
            out.push(*kind);
        }
    }
    if out.is_empty() {
        return Err(AppError::bad_request(
            "entity_kinds: a list offers at least one kind",
        ));
    }
    Ok(out)
}

pub async fn create(
    ctx: &AppContext,
    subject: &Subject,
    req: &CreateListRequest,
) -> Result<TierList, AppError> {
    validate_list(&req.name, req.description.as_deref())?;
    let user = actor(subject)?;
    let kind = req.list_type;
    ctx.policy.require(
        subject,
        Action::CreateTierList,
        &Resource::NewTierList { kind },
    )?;
    let staff = ctx
        .policy
        .can(
            subject,
            Action::CreateTierList,
            &Resource::NewTierList {
                kind: TierListType::Official,
            },
        )
        .is_allowed();
    let mut conn = ctx.db.conn().await?;
    if !staff && repo::count_by_user(&mut conn, user, None).await? >= MAX_LISTS_PER_USER {
        return Err(AppError::Conflict(format!(
            "maximum {MAX_LISTS_PER_USER} tier lists per user"
        )));
    }
    let slug = generate_slug(&req.name);
    let list = repo::create(
        &mut conn,
        &req.name,
        &slug,
        req.description.as_deref(),
        kind,
        user,
    )
    .await?;
    repo::ensure_stats_row(&mut conn, list_id(&list)).await?;
    drop(conn);
    let details = ContentAuditDetails {
        list_type: Some(kind.as_str().to_owned()),
        ..Default::default()
    };
    ctx.audit
        .record(audit_event(
            &list,
            user,
            ContentAuditAction::Create,
            details,
        ))
        .await;
    Ok(list.into())
}

pub async fn update(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    req: &UpdateListRequest,
) -> Result<TierList, AppError> {
    validate_list(&req.name, req.description.as_deref())?;
    let kinds = req.entity_kinds.as_deref().map(offered_kinds).transpose()?;
    let mut conn = ctx.db.conn().await?;
    let (list, user) =
        find_and_authorize(ctx, &mut conn, subject, slug, Action::EditTierList).await?;
    let updated = repo::update(
        &mut conn,
        list_id(&list),
        &req.name,
        req.description.as_deref(),
        kinds.as_deref(),
    )
    .await?;
    drop(conn);
    invalidate_detail(ctx, slug).await;
    let details = ContentAuditDetails {
        renamed_from: (list.name != updated.name).then(|| list.name.clone()),
        description_changed: (list.description != updated.description).then_some(true),
        kinds: (list.entity_kinds != updated.entity_kinds).then(|| {
            updated
                .entity_kinds
                .iter()
                .map(|k| k.as_str().to_owned())
                .collect()
        }),
        ..Default::default()
    };
    ctx.audit
        .record(audit_event(
            &updated,
            user,
            ContentAuditAction::Edit,
            details,
        ))
        .await;
    Ok(updated.into())
}

pub async fn delete(ctx: &AppContext, subject: &Subject, slug: &str) -> Result<(), AppError> {
    let mut conn = ctx.db.conn().await?;
    let (list, user) =
        find_and_authorize(ctx, &mut conn, subject, slug, Action::AdministerTierList).await?;
    let tier_ids: Vec<Uuid> = repo::get_tiers(&mut conn, list_id(&list))
        .await?
        .iter()
        .map(|t| t.id)
        .collect();
    let entries = repo::get_placements_for_tiers(&mut conn, &tier_ids)
        .await?
        .len(); // counted before the cascade takes them
    repo::delete_list(&mut conn, list_id(&list)).await?;
    drop(conn);
    invalidate_detail(ctx, slug).await;
    let details = ContentAuditDetails {
        tiers: Some(count_of(tier_ids.len())),
        entries: Some(count_of(entries)),
        ..Default::default()
    };
    ctx.audit
        .record(audit_event(
            &list,
            user,
            ContentAuditAction::Delete,
            details,
        ))
        .await;
    Ok(())
}

fn tier_ref(tier: &TierRow) -> AuditTierRef {
    AuditTierRef {
        id: tier.id,
        name: tier.name.clone(),
    }
}

pub async fn create_tier(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    req: &TierRequest,
) -> Result<Tier, AppError> {
    validate_tier(req)?;
    let mut conn = ctx.db.conn().await?;
    let (list, user) =
        find_and_authorize(ctx, &mut conn, subject, slug, Action::EditTierList).await?;
    let tier = repo::create_tier(
        &mut conn,
        list_id(&list),
        &req.name,
        req.display_order,
        req.color.as_deref(),
        req.description.as_deref(),
    )
    .await?;
    drop(conn);
    invalidate_detail(ctx, slug).await;
    let details = ContentAuditDetails {
        tiers_added: Some(vec![tier_ref(&tier)]),
        ..Default::default()
    };
    ctx.audit
        .record(audit_event(&list, user, ContentAuditAction::Edit, details))
        .await;
    Ok(tier.into())
}

fn tier_update_details(before: &TierRow, after: &TierRow) -> ContentAuditDetails {
    let changed = |yes: bool| yes.then(|| vec![tier_ref(after)]);
    ContentAuditDetails {
        tiers_renamed: (before.name != after.name).then(|| {
            vec![AuditTierRename {
                id: after.id,
                from: before.name.clone(),
                to: after.name.clone(),
            }]
        }),
        tiers_reordered: changed(before.display_order != after.display_order),
        tiers_restyled: changed(
            before.color != after.color || before.description != after.description,
        ),
        ..Default::default()
    }
}

pub async fn update_tier(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    tier_id: Uuid,
    req: &TierRequest,
) -> Result<Tier, AppError> {
    validate_tier(req)?;
    let mut conn = ctx.db.conn().await?;
    let (list, user) =
        find_and_authorize(ctx, &mut conn, subject, slug, Action::EditTierList).await?;
    let before = repo::get_tiers(&mut conn, list_id(&list))
        .await?
        .into_iter()
        .find(|t| t.id == tier_id);
    let tier = repo::update_tier(
        &mut conn,
        list_id(&list),
        tier_id,
        &req.name,
        req.display_order,
        req.color.as_deref(),
        req.description.as_deref(),
    )
    .await?
    .ok_or(AppError::NotFound(None))?;
    drop(conn);
    invalidate_detail(ctx, slug).await;
    if let Some(before) = before {
        let details = tier_update_details(&before, &tier);
        ctx.audit
            .record(audit_event(&list, user, ContentAuditAction::Edit, details))
            .await;
    }
    Ok(tier.into())
}

pub async fn delete_tier(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    tier_id: Uuid,
) -> Result<(), AppError> {
    let mut conn = ctx.db.conn().await?;
    let (list, user) =
        find_and_authorize(ctx, &mut conn, subject, slug, Action::AdministerTierList).await?;
    let before = repo::get_tiers(&mut conn, list_id(&list))
        .await?
        .into_iter()
        .find(|t| t.id == tier_id);
    let dropped = match &before {
        Some(tier) => repo::get_placements(&mut conn, tier.id).await?.len(),
        None => 0,
    };
    if !repo::delete_tier(&mut conn, list_id(&list), tier_id).await? {
        return Err(AppError::NotFound(None));
    }
    drop(conn);
    invalidate_detail(ctx, slug).await;
    if let Some(tier) = before {
        let details = ContentAuditDetails {
            tiers_deleted: Some(vec![tier_ref(&tier)]),
            entries_dropped: (dropped > 0).then(|| count_of(dropped)),
            ..Default::default()
        };
        ctx.audit
            .record(audit_event(&list, user, ContentAuditAction::Edit, details))
            .await;
    }
    Ok(())
}

fn entry_key(kind: EntityKind, id: &str) -> Vec<String> {
    vec![format!("{}:{id}", kind.as_str())]
}

pub async fn add_placement(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    req: &AddPlacementRequest,
) -> Result<TierPlacement, AppError> {
    validate_length(
        "placement description",
        req.description.as_deref(),
        PLACEMENT_DESCRIPTION_MAX,
    )?;
    let (list, user) = {
        let mut conn = ctx.db.conn().await?;
        find_and_authorize(ctx, &mut conn, subject, slug, Action::EditTierList).await?
    };
    validate_entity(ctx, req.entity_kind, &req.entity_id).await?;
    let mut conn = ctx.db.conn().await?;
    let placement = repo::add_placement(
        &mut conn,
        list_id(&list),
        req.tier_id,
        req.entity_kind,
        &req.entity_id,
        req.sub_order.unwrap_or(0),
        req.description.as_deref(),
    )
    .await?
    .ok_or(AppError::NotFound(None))?;
    drop(conn);
    invalidate_detail(ctx, slug).await;
    let details = ContentAuditDetails {
        entries_added: Some(entry_key(req.entity_kind, &req.entity_id)),
        ..Default::default()
    };
    ctx.audit
        .record(audit_event(&list, user, ContentAuditAction::Edit, details))
        .await;
    Ok(placement.into())
}

pub async fn set_placement_description(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    kind: EntityKind,
    id: &str,
    description: Option<&str>,
) -> Result<TierPlacement, AppError> {
    validate_length(
        "placement description",
        description,
        PLACEMENT_DESCRIPTION_MAX,
    )?;
    let mut conn = ctx.db.conn().await?;
    let (list, user) =
        find_and_authorize(ctx, &mut conn, subject, slug, Action::EditTierList).await?;
    let placement =
        repo::set_placement_description(&mut conn, list_id(&list), kind, id, description)
            .await?
            .ok_or(AppError::NotFound(None))?;
    drop(conn);
    invalidate_detail(ctx, slug).await;
    let details = ContentAuditDetails {
        notes_edited: Some(entry_key(kind, id)),
        ..Default::default()
    };
    ctx.audit
        .record(audit_event(&list, user, ContentAuditAction::Edit, details))
        .await;
    Ok(placement.into())
}

pub async fn remove_placement(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    kind: EntityKind,
    id: &str,
) -> Result<(), AppError> {
    let mut conn = ctx.db.conn().await?;
    let (list, user) =
        find_and_authorize(ctx, &mut conn, subject, slug, Action::EditTierList).await?;
    let Some(tier) = holding_tier(&mut conn, list_id(&list), kind, id).await? else {
        return Ok(());
    };
    repo::remove_placement(&mut conn, list_id(&list), tier, kind, id).await?;
    drop(conn);
    invalidate_detail(ctx, slug).await;
    let details = ContentAuditDetails {
        entries_removed: Some(entry_key(kind, id)),
        ..Default::default()
    };
    ctx.audit
        .record(audit_event(&list, user, ContentAuditAction::Edit, details))
        .await;
    Ok(())
}

pub async fn move_placement(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    kind: EntityKind,
    id: &str,
    req: &MovePlacementRequest,
) -> Result<TierPlacement, AppError> {
    let mut conn = ctx.db.conn().await?;
    let (list, user) =
        find_and_authorize(ctx, &mut conn, subject, slug, Action::EditTierList).await?;
    let tier = holding_tier(&mut conn, list_id(&list), kind, id)
        .await?
        .ok_or(AppError::NotFound(None))?;
    let moved = repo::move_placement(
        &mut conn,
        list_id(&list),
        tier,
        req.new_tier_id,
        kind,
        id,
        req.sub_order.unwrap_or(0),
    )
    .await?
    .ok_or_else(|| AppError::bad_request("destination tier is not on this tier list"))?;
    drop(conn);
    invalidate_detail(ctx, slug).await;
    let entry = Some(entry_key(kind, id));
    let details = if tier == req.new_tier_id {
        ContentAuditDetails {
            entries_reordered: entry,
            ..Default::default()
        }
    } else {
        ContentAuditDetails {
            entries_moved: entry,
            ..Default::default()
        }
    };
    ctx.audit
        .record(audit_event(&list, user, ContentAuditAction::Edit, details))
        .await;
    Ok(moved.into())
}

async fn holding_tier(
    conn: &mut myrtle_db::Conn,
    list: myrtle_types::TierListId,
    kind: EntityKind,
    id: &str,
) -> Result<Option<Uuid>, AppError> {
    let tiers: Vec<Uuid> = repo::get_tiers(conn, list)
        .await?
        .iter()
        .map(|t| t.id)
        .collect();
    Ok(repo::get_placements_for_tiers(conn, &tiers)
        .await?
        .into_iter()
        .find(|p| p.entity_kind == kind && p.entity_id == id)
        .map(|p| p.tier_id))
}

pub async fn publish(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    req: &PublishRequest,
) -> Result<TierListVersion, AppError> {
    let (list, user) = {
        let mut conn = ctx.db.conn().await?;
        find_and_authorize(ctx, &mut conn, subject, slug, Action::PublishTierList).await?
    };
    let detail = load_detail(ctx, slug).await?;
    let snapshot = serde_json::to_value(snapshot_of(&detail.tiers)).map_err(AppError::internal)?;
    let mut conn = ctx.db.conn().await?;
    let next = repo::latest_version(&mut conn, list_id(&list))
        .await?
        .unwrap_or(0)
        + 1;
    let version = repo::create_version(
        &mut conn,
        list_id(&list),
        next,
        &snapshot,
        req.changelog.as_deref(),
        user,
    )
    .await?;
    drop(conn);
    invalidate_detail(ctx, slug).await;
    let details = ContentAuditDetails {
        version: Some(next),
        changelog: req
            .changelog
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .map(|c| c.chars().take(CHANGELOG_AUDIT_MAX).collect()),
        ..Default::default()
    };
    ctx.audit
        .record(audit_event(
            &list,
            user,
            ContentAuditAction::Publish,
            details,
        ))
        .await;
    Ok(version.into())
}
