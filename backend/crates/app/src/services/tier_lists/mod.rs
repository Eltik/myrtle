mod edit;
pub mod entities;
mod social;

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use chrono::{DateTime, Utc};
use myrtle_db::Conn;
use myrtle_db::repos::tier_lists::{
    self as repo, AuthorRow, FlairRow, GrantRow, GrantedListRow, PermissionRow, PlacementRow,
    StatsRow, TierListRow, TierRow, VersionRow,
};
use myrtle_gamedata::model::GameModel;
use myrtle_types::{ContentAuditAction, ContentTargetKind, EntityKind, Server, TierListId, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::audit::{ContentAuditDetails, ContentEvent};
use crate::policy::{Action, Resource, Subject};
use crate::services::accounts;
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};
pub use edit::*;
use entities::{EntitySummary, StoryCatalogues};
pub use social::*;

const DETAIL_TTL: Duration = Duration::from_mins(10);

fn operator_only() -> Vec<EntityKind> {
    vec![EntityKind::Operator]
}

const fn operator_kind() -> EntityKind {
    EntityKind::Operator
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierList {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub list_type: String,
    pub created_by: Option<Uuid>,
    pub is_active: bool,
    pub is_listed: bool,
    pub flair_id: Option<i16>,
    #[serde(default = "operator_only")]
    pub entity_kinds: Vec<EntityKind>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TierListRow> for TierList {
    fn from(r: TierListRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            slug: r.slug,
            description: r.description,
            list_type: r.list_type,
            created_by: r.created_by,
            is_active: r.is_active,
            is_listed: r.is_listed,
            flair_id: r.flair_id,
            entity_kinds: r.entity_kinds,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub name: String,
    pub display_order: i16,
    pub color: Option<String>,
    pub description: Option<String>,
}

impl From<TierRow> for Tier {
    fn from(r: TierRow) -> Self {
        Self {
            id: r.id,
            tier_list_id: r.tier_list_id,
            name: r.name,
            display_order: r.display_order,
            color: r.color,
            description: r.description,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierPlacement {
    pub tier_id: Uuid,
    #[serde(default = "operator_kind")]
    pub entity_kind: EntityKind,
    #[serde(alias = "operator_id")]
    pub entity_id: String,
    pub sub_order: i16,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl From<PlacementRow> for TierPlacement {
    fn from(r: PlacementRow) -> Self {
        Self {
            tier_id: r.tier_id,
            entity_kind: r.entity_kind,
            entity_id: r.entity_id,
            sub_order: r.sub_order,
            description: r.description,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierListStats {
    pub tier_list_id: Uuid,
    pub view_count: i64,
    pub unique_view_count: i64,
    pub favorite_count: i32,
    pub share_count: i32,
    pub is_trending: bool,
    pub trending_score: f64,
    pub views_last_24h: i32,
    pub views_last_7d: i32,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub stats_updated_at: DateTime<Utc>,
}

impl From<StatsRow> for TierListStats {
    fn from(r: StatsRow) -> Self {
        Self {
            tier_list_id: r.tier_list_id,
            view_count: r.view_count,
            unique_view_count: r.unique_view_count,
            favorite_count: r.favorite_count,
            share_count: r.share_count,
            is_trending: r.is_trending,
            trending_score: r.trending_score,
            views_last_24h: r.views_last_24h,
            views_last_7d: r.views_last_7d,
            last_viewed_at: r.last_viewed_at,
            stats_updated_at: r.stats_updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierListFlair {
    pub id: i16,
    pub code: String,
    pub label: String,
    pub color: Option<String>,
    pub display_order: i16,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<FlairRow> for TierListFlair {
    fn from(r: FlairRow) -> Self {
        Self {
            id: r.id,
            code: r.code,
            label: r.label,
            color: r.color,
            display_order: r.display_order,
            is_active: r.is_active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierListAuthor {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub avatar_id: Option<String>,
}

impl From<AuthorRow> for TierListAuthor {
    fn from(r: AuthorRow) -> Self {
        Self {
            id: r.id,
            uid: r.uid,
            nickname: r.nickname,
            avatar_id: r.avatar_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierListVersion {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub version: i32,
    pub snapshot: serde_json::Value,
    pub changelog: Option<String>,
    pub published_by: Option<Uuid>,
    pub published_at: DateTime<Utc>,
}

impl From<VersionRow> for TierListVersion {
    fn from(r: VersionRow) -> Self {
        Self {
            id: r.id,
            tier_list_id: r.tier_list_id,
            version: r.version,
            snapshot: r.snapshot,
            changelog: r.changelog,
            published_by: r.published_by,
            published_at: r.published_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierListPermission {
    pub tier_list_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

impl From<PermissionRow> for TierListPermission {
    fn from(r: PermissionRow) -> Self {
        Self {
            tier_list_id: r.tier_list_id,
            user_id: r.user_id,
            permission: r.permission,
            granted_by: r.granted_by,
            granted_at: r.granted_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TierListGrant {
    pub tier_list_id: Uuid,
    pub slug: String,
    pub title: String,
    pub list_type: String,
    pub user_id: Uuid,
    pub user_uid: String,
    pub user_nickname: Option<String>,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub granted_by_nickname: Option<String>,
    pub granted_at: DateTime<Utc>,
}

impl From<GrantRow> for TierListGrant {
    fn from(r: GrantRow) -> Self {
        Self {
            tier_list_id: r.tier_list_id,
            slug: r.slug,
            title: r.title,
            list_type: r.list_type,
            user_id: r.user_id,
            user_uid: r.user_uid,
            user_nickname: r.user_nickname,
            permission: r.permission,
            granted_by: r.granted_by,
            granted_by_nickname: r.granted_by_nickname,
            granted_at: r.granted_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GrantedTierList {
    pub tier_list_id: Uuid,
    pub slug: String,
    pub title: String,
    pub list_type: String,
    pub permission: String,
}

impl From<GrantedListRow> for GrantedTierList {
    fn from(r: GrantedListRow) -> Self {
        Self {
            tier_list_id: r.tier_list_id,
            slug: r.slug,
            title: r.title,
            list_type: r.list_type,
            permission: r.permission,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierListDetail {
    #[serde(flatten)]
    pub list: TierList,
    pub tiers: Vec<TierDetail>,
    pub stats: Option<TierListStats>,
    pub flair: Option<TierListFlair>,
    pub author: Option<TierListAuthor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDetail {
    #[serde(flatten)]
    pub tier: Tier,
    pub placements: Vec<PlacementDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementDetail {
    #[serde(flatten)]
    pub placement: TierPlacement,
    pub entity: Option<EntitySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierSnapshot {
    #[serde(flatten)]
    pub tier: Tier,
    pub placements: Vec<TierPlacement>,
}

pub fn read_snapshot(snapshot: &serde_json::Value) -> Result<Vec<TierSnapshot>, serde_json::Error> {
    Vec::<TierSnapshot>::deserialize(snapshot)
}

#[must_use]
pub fn snapshot_of(tiers: &[TierDetail]) -> Vec<TierSnapshot> {
    tiers
        .iter()
        .map(|t| TierSnapshot {
            tier: t.tier.clone(),
            placements: t.placements.iter().map(|p| p.placement.clone()).collect(),
        })
        .collect()
}

#[must_use]
pub fn resolve_snapshot(
    snapshot: Vec<TierSnapshot>,
    model: &GameModel,
    story: &StoryCatalogues,
) -> Vec<TierDetail> {
    snapshot
        .into_iter()
        .map(|t| TierDetail {
            tier: t.tier,
            placements: t
                .placements
                .into_iter()
                .map(|p| resolve_placement(p, model, story))
                .collect(),
        })
        .collect()
}

fn resolve_placement(
    placement: TierPlacement,
    model: &GameModel,
    story: &StoryCatalogues,
) -> PlacementDetail {
    let entity = entities::resolve(model, story, placement.entity_kind, &placement.entity_id);
    PlacementDetail { placement, entity }
}

fn detail_key(slug: &str) -> String {
    format!("tierlist:{slug}")
}

pub async fn invalidate_detail(ctx: &AppContext, slug: &str) {
    ctx.cache.invalidate(&detail_key(slug)).await;
}

const fn list_id(list: &TierListRow) -> TierListId {
    TierListId(list.id)
}

async fn find(conn: &mut Conn, slug: &str) -> Result<TierListRow, AppError> {
    repo::find_by_slug(conn, slug)
        .await?
        .ok_or(AppError::NotFound(None))
}

fn actor(subject: &Subject) -> Result<UserId, AppError> {
    subject.user_id().ok_or(AppError::Unauthorized)
}

async fn authorize(
    ctx: &AppContext,
    conn: &mut Conn,
    subject: &Subject,
    action: Action,
    list: &TierListRow,
) -> Result<(), AppError> {
    let resource = Resource::TierList {
        id: list_id(list),
        owner: list.created_by.map(UserId),
    };
    if ctx.policy.can(subject, action, &resource).is_allowed() {
        return Ok(());
    }
    let subject = match subject {
        Subject::User {
            id,
            uid,
            server,
            role,
            ..
        } => Subject::User {
            id: *id,
            uid: uid.clone(),
            server: *server,
            role: *role,
            grants: accounts::grants(conn, *id).await?.into(),
        },
        other => other.clone(),
    };
    ctx.policy.require(&subject, action, &resource)
}

async fn find_and_authorize(
    ctx: &AppContext,
    conn: &mut Conn,
    subject: &Subject,
    slug: &str,
    action: Action,
) -> Result<(TierListRow, UserId), AppError> {
    let actor = actor(subject)?;
    let list = find(conn, slug).await?;
    authorize(ctx, conn, subject, action, &list).await?;
    Ok((list, actor))
}

fn audit_event(
    list: &TierListRow,
    actor: UserId,
    action: ContentAuditAction,
    details: ContentAuditDetails,
) -> ContentEvent<'_> {
    ContentEvent {
        actor,
        target_kind: ContentTargetKind::TierList,
        target_id: list.id,
        target_name: &list.name,
        target_slug: Some(&list.slug),
        owner: list.created_by.map(UserId),
        subject: None,
        action,
        details,
    }
}

fn count_of(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn server_of(ctx: &AppContext, server: Option<Server>) -> Server {
    server.unwrap_or_else(|| ctx.snapshots.default_server())
}

fn model_for(
    ctx: &AppContext,
    server: Option<Server>,
) -> Result<std::sync::Arc<Snapshot>, AppError> {
    ctx.snapshots
        .or_default(server.unwrap_or_else(|| ctx.snapshots.default_server()))
}

pub async fn get(
    ctx: &AppContext,
    slug: &str,
    server: Option<Server>,
) -> Result<TierListDetail, AppError> {
    let mut detail = load_detail(ctx, slug).await?;
    let snapshot = model_for(ctx, server)?;
    let kinds: Vec<EntityKind> = detail
        .tiers
        .iter()
        .flat_map(|t| t.placements.iter().map(|p| p.placement.entity_kind))
        .collect();
    let story = StoryCatalogues::gather(ctx, server_of(ctx, server), &kinds).await;
    for p in detail.tiers.iter_mut().flat_map(|t| &mut t.placements) {
        p.entity = entities::resolve(
            &snapshot.model,
            &story,
            p.placement.entity_kind,
            &p.placement.entity_id,
        );
    }
    Ok(detail)
}

pub async fn load_detail(ctx: &AppContext, slug: &str) -> Result<TierListDetail, AppError> {
    let key = detail_key(slug);
    if let Some(cached) = ctx.cache.get_json::<TierListDetail>(&key).await {
        return Ok(cached);
    }
    let mut conn = ctx.db.conn().await?;
    let list = find(&mut conn, slug).await?;
    let id = list_id(&list);
    let tiers = repo::get_tiers(&mut conn, id).await?;
    let stats = repo::get_stats(&mut conn, id).await?;
    let flair = match list.flair_id {
        Some(flair) => repo::get_flair_by_id(&mut conn, flair).await?,
        None => None,
    };
    let author = match list.created_by {
        Some(user) => repo::find_author(&mut conn, UserId(user)).await?,
        None => None,
    };
    let tier_ids: Vec<Uuid> = tiers.iter().map(|t| t.id).collect();
    let placements = repo::get_placements_for_tiers(&mut conn, &tier_ids).await?;
    drop(conn);

    let mut by_tier = group_by(placements, |p| p.tier_id);
    let tiers = tiers
        .into_iter()
        .map(|tier| TierDetail {
            placements: by_tier
                .remove(&tier.id)
                .unwrap_or_default()
                .into_iter()
                .map(|p| PlacementDetail {
                    placement: p.into(),
                    entity: None,
                })
                .collect(),
            tier: tier.into(),
        })
        .collect();
    let detail = TierListDetail {
        list: list.into(),
        tiers,
        stats: stats.map(Into::into),
        flair: flair.map(Into::into),
        author: author.map(Into::into),
    };
    ctx.cache.set_json(&key, &detail, DETAIL_TTL).await;
    Ok(detail)
}

fn group_by<T>(items: Vec<T>, key: impl Fn(&T) -> Uuid) -> HashMap<Uuid, Vec<T>> {
    let mut out: HashMap<Uuid, Vec<T>> = HashMap::new();
    for item in items {
        out.entry(key(&item)).or_default().push(item);
    }
    out
}

pub async fn list_details(
    ctx: &AppContext,
    limit: Option<i64>,
    author: Option<UserId>,
    server: Option<Server>,
) -> Result<Vec<TierListDetail>, AppError> {
    let limit = limit.unwrap_or(60).clamp(1, 200);
    let snapshot = model_for(ctx, server)?;
    let mut conn = ctx.db.conn().await?;
    let lists = repo::find_all_active_limited(&mut conn, limit, author).await?;
    let ids: Vec<TierListId> = lists.iter().map(list_id).collect();
    let flair_ids: Vec<i16> = lists
        .iter()
        .filter_map(|l| l.flair_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let author_ids: Vec<UserId> = lists
        .iter()
        .filter_map(|l| l.created_by.map(UserId))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let tiers = repo::get_tiers_for_lists(&mut conn, &ids).await?;
    let stats = repo::get_stats_for_lists(&mut conn, &ids).await?;
    let flairs = repo::get_flairs_by_ids(&mut conn, &flair_ids).await?;
    let authors = repo::find_authors(&mut conn, &author_ids).await?;
    let tier_ids: Vec<Uuid> = tiers.iter().map(|t| t.id).collect();
    let placements = repo::get_placements_for_tiers(&mut conn, &tier_ids).await?;
    drop(conn);

    let kinds: Vec<EntityKind> = placements.iter().map(|p| p.entity_kind).collect();
    let story = StoryCatalogues::gather(ctx, server_of(ctx, server), &kinds).await;
    let model = &snapshot.model;
    let mut tiers_by_list = group_by(tiers, |t| t.tier_list_id);
    let mut by_tier = group_by(placements, |p| p.tier_id);
    let mut stats: HashMap<Uuid, StatsRow> =
        stats.into_iter().map(|s| (s.tier_list_id, s)).collect();
    let flairs: HashMap<i16, FlairRow> = flairs.into_iter().map(|f| (f.id, f)).collect();
    let authors: HashMap<Uuid, AuthorRow> = authors.into_iter().map(|a| (a.id, a)).collect();
    Ok(lists
        .into_iter()
        .map(|list| {
            let tiers = tiers_by_list
                .remove(&list.id)
                .unwrap_or_default()
                .into_iter()
                .map(|tier| TierDetail {
                    placements: by_tier
                        .remove(&tier.id)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|p| resolve_placement(p.into(), model, &story))
                        .collect(),
                    tier: tier.into(),
                })
                .collect();
            TierListDetail {
                tiers,
                stats: stats.remove(&list.id).map(Into::into),
                flair: list
                    .flair_id
                    .and_then(|f| flairs.get(&f).cloned().map(Into::into)),
                author: list
                    .created_by
                    .and_then(|a| authors.get(&a).cloned().map(Into::into)),
                list: list.into(),
            }
        })
        .collect())
}

pub async fn list(ctx: &AppContext) -> Result<Vec<TierList>, AppError> {
    let rows = repo::find_all_active(&mut *ctx.db.conn().await?, None).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn mine(ctx: &AppContext, subject: &Subject) -> Result<Vec<TierList>, AppError> {
    let user = actor(subject)?;
    let rows = repo::find_by_user(&mut *ctx.db.conn().await?, user).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn favorites(ctx: &AppContext, subject: &Subject) -> Result<Vec<TierList>, AppError> {
    let user = actor(subject)?;
    let rows = repo::find_favorited_by_user(&mut *ctx.db.conn().await?, user).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn versions(
    ctx: &AppContext,
    slug: &str,
    server: Option<Server>,
) -> Result<Vec<TierListVersion>, AppError> {
    let rows = {
        let mut conn = ctx.db.conn().await?;
        let list = find(&mut conn, slug).await?;
        repo::get_versions(&mut conn, list_id(&list)).await?
    };
    let snapshot = model_for(ctx, server)?;
    let mut versions: Vec<TierListVersion> = rows.into_iter().map(Into::into).collect();
    let read: Vec<_> = versions
        .iter()
        .map(|v| read_snapshot(&v.snapshot))
        .collect();
    let kinds: Vec<EntityKind> = read
        .iter()
        .flatten()
        .flatten()
        .flat_map(|t| t.placements.iter().map(|p| p.entity_kind))
        .collect();
    let story = StoryCatalogues::gather(ctx, server_of(ctx, server), &kinds).await;
    for (v, tiers) in versions.iter_mut().zip(read) {
        match tiers {
            Ok(tiers) => {
                v.snapshot = serde_json::to_value(resolve_snapshot(tiers, &snapshot.model, &story))
                    .map_err(AppError::internal)?;
            }
            Err(e) => {
                tracing::warn!(slug, version = v.version, error = %e, "unreadable tier list snapshot");
            }
        }
    }
    Ok(versions)
}

pub async fn catalogue(
    ctx: &AppContext,
    kind: EntityKind,
    server: Option<Server>,
) -> Result<Vec<EntitySummary>, AppError> {
    let snapshot = model_for(ctx, server)?;
    let story = StoryCatalogues::gather(ctx, server_of(ctx, server), &[kind]).await;
    Ok(entities::catalogue(&snapshot.model, &story, kind))
}

pub async fn validate_entity(ctx: &AppContext, kind: EntityKind, id: &str) -> Result<(), AppError> {
    entities::validate_loaded(ctx, kind, id).await
}
