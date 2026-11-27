use std::sync::Arc;

use chrono::{DateTime, Utc};
use myrtle_db::Conn;
use myrtle_db::repos::grids::{self as repo, GridDoc, GridRow, ListOrder};
use myrtle_gamedata::model::GameModel;
use myrtle_types::{ContentAuditAction, ContentTargetKind, EntityKind, GridId, Server, UserId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::super::tier_lists::entities::{self, EntitySummary, FacetValue, StoryCatalogues};
use super::super::tier_lists::{generate_slug, validate_length};
use crate::audit::{ContentAuditDetails, ContentEvent};
use crate::policy::{Action, Resource, Subject};
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

pub const SIZE_MIN: i32 = 1;
pub const SIZE_MAX: i32 = 10;
pub const TITLE_MAX: usize = 100;
pub const DESCRIPTION_MAX: usize = 2000;
pub const LABEL_MAX: usize = 60;
pub const PER_PAGE_DEFAULT: i32 = 24;
pub const PER_PAGE_MAX: i32 = 60;
pub const MAX_GRIDS_PER_USER: i64 = 50;
pub const SLUG_BASE_MAX: usize = 60;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GridCellInput {
    pub label: String,
    pub entity_kind: Option<EntityKind>,
    pub entity_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GridInput {
    pub title: String,
    pub description: Option<String>,
    pub rows: i32,
    pub cols: i32,
    pub cells: Vec<GridCellInput>,
    pub is_listed: bool,
    pub entity_kinds: Vec<EntityKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GridCell {
    pub label: String,
    pub entity_kind: Option<EntityKind>,
    pub entity_id: Option<String>,
    pub entity: Option<EntitySummary>,
    #[ts(type = "string | null")]
    pub entity_server: Option<Server>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GridPreviewIcon {
    pub kind: EntityKind,
    pub icon: String,
    #[ts(type = "string | null")]
    pub server: Option<Server>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GridOwner {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GridTemplateRef {
    pub slug: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Grid {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub rows: i32,
    pub cols: i32,
    pub cells: Vec<GridCell>,
    pub is_listed: bool,
    pub entity_kinds: Vec<EntityKind>,
    pub kinds_locked: bool,
    pub owner: GridOwner,
    pub template_of: Option<GridTemplateRef>,
    #[ts(type = "number")]
    pub fork_count: i64,
    pub can_edit: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GridSummary {
    pub slug: String,
    pub title: String,
    pub rows: i32,
    pub cols: i32,
    pub owner: GridOwner,
    #[ts(type = "number")]
    pub fork_count: i64,
    pub is_listed: bool,
    pub entity_kinds: Vec<EntityKind>,
    pub updated_at: DateTime<Utc>,
    pub preview: Vec<Option<GridPreviewIcon>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GridListResponse {
    pub items: Vec<GridSummary>,
    #[ts(type = "number")]
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredCell {
    pub label: String,
    pub entity_kind: Option<String>,
    pub entity_id: Option<String>,
}

impl StoredCell {
    #[must_use]
    pub fn entity(&self) -> Option<(EntityKind, &str)> {
        let (kind, id) = (self.entity_kind.as_deref()?, self.entity_id.as_deref()?);
        let parsed = EntityKind::parse(kind);
        if parsed.is_none() {
            tracing::warn!(
                entity_id = id,
                kind,
                "grids.cells: dropping a pick of unknown kind"
            );
        }
        Some((parsed?, id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridDocument {
    pub title: String,
    pub description: Option<String>,
    pub rows: i16,
    pub cols: i16,
    pub cells: Vec<StoredCell>,
    pub is_listed: bool,
    pub entity_kinds: Vec<EntityKind>,
}

impl GridDocument {
    fn write<'a>(&'a self, cells: &'a Value) -> GridDoc<'a> {
        GridDoc {
            title: &self.title,
            description: self.description.as_deref(),
            rows: self.rows,
            cols: self.cols,
            cells,
            is_listed: self.is_listed,
            entity_kinds: &self.entity_kinds,
        }
    }
}

#[must_use]
pub fn stored_cells(row: &GridRow) -> Vec<StoredCell> {
    Vec::<StoredCell>::deserialize(&row.cells).unwrap_or_else(|e| {
        tracing::warn!(slug = %row.slug, error = %e, "grids.cells unreadable");
        Vec::new()
    })
}

#[must_use]
pub fn kinds_of(row: &GridRow) -> Vec<EntityKind> {
    let mut kinds: Vec<EntityKind> = row
        .entity_kinds
        .iter()
        .filter_map(|s| {
            let kind = EntityKind::parse(s);
            if kind.is_none() {
                tracing::warn!(slug = %row.slug, kind = %s, "grids.entity_kinds: skipping unknown kind");
            }
            kind
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

fn server_or_default(ctx: &AppContext, server: Option<Server>) -> Server {
    server.unwrap_or_else(|| ctx.snapshots.default_server())
}

fn actor(subject: &Subject) -> Result<UserId, AppError> {
    subject.user_id().ok_or(AppError::Unauthorized)
}

pub async fn get(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    server: Option<Server>,
) -> Result<Grid, AppError> {
    let row = load(&mut *ctx.db.conn().await?, slug).await?;
    let kinds = picked_kinds(&stored_cells(&row));
    let resolver = Resolver::new(ctx, server_or_default(ctx, server), &kinds).await;
    Ok(to_grid(ctx, row, subject, &resolver))
}

pub async fn list(
    ctx: &AppContext,
    order: ListOrder,
    q: Option<&str>,
    page: Option<i32>,
    per_page: Option<i32>,
    server: Option<Server>,
) -> Result<GridListResponse, AppError> {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(PER_PAGE_DEFAULT).clamp(1, PER_PAGE_MAX);
    let offset = i64::from(page - 1) * i64::from(per_page);
    let pattern = q
        .map(str::trim)
        .filter(|q| !q.is_empty())
        .map(myrtle_db::sql::contains_pattern);
    let (rows, total) = repo::find_listed(
        &mut *ctx.db.conn().await?,
        pattern.as_deref(),
        order,
        i64::from(per_page),
        offset,
    )
    .await?;
    let resolver = Resolver::new(ctx, server_or_default(ctx, server), &rows_kinds(&rows)).await;
    Ok(GridListResponse {
        items: rows.iter().map(|r| to_summary(r, &resolver)).collect(),
        total,
        page,
        per_page,
    })
}

pub async fn mine(
    ctx: &AppContext,
    subject: &Subject,
    server: Option<Server>,
) -> Result<Vec<GridSummary>, AppError> {
    let user = actor(subject)?;
    let rows = repo::find_by_user(&mut *ctx.db.conn().await?, user).await?;
    let resolver = Resolver::new(ctx, server_or_default(ctx, server), &rows_kinds(&rows)).await;
    Ok(rows.iter().map(|r| to_summary(r, &resolver)).collect())
}

pub async fn create(
    ctx: &AppContext,
    subject: &Subject,
    input: GridInput,
    server: Option<Server>,
) -> Result<Grid, AppError> {
    let user = actor(subject)?;
    let doc = checked(input)?;
    let kinds = picked_kinds(&doc.cells);
    let resolver = Resolver::new(ctx, server_or_default(ctx, server), &kinds).await;
    validate_entities(&resolver, &doc.cells, &[])?;
    let mut conn = ctx.db.conn().await?;
    let slug = insert_capped(&mut conn, &doc, user, None).await?;
    let row = load(&mut conn, &slug).await?;
    drop(conn);
    let grid = to_grid(ctx, row, subject, &resolver);
    let event = audit_event(
        &grid,
        user,
        ContentAuditAction::Create,
        ContentAuditDetails::default(),
    );
    ctx.audit.record(event).await;
    Ok(grid)
}

pub async fn update(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    input: GridInput,
    server: Option<Server>,
) -> Result<Grid, AppError> {
    let (before, user) = load_editable(ctx, &mut *ctx.db.conn().await?, subject, slug).await?;
    let doc = checked(input)?;
    if before.template_of.is_some() && kinds_of(&before) != doc.entity_kinds {
        return Err(AppError::bad_request(
            "entity_kinds: allowed types are set by the template",
        ));
    }
    let before_cells = stored_cells(&before);
    let kinds = picked_kinds(&doc.cells);
    let resolver = Resolver::new(ctx, server_or_default(ctx, server), &kinds).await;
    validate_entities(&resolver, &doc.cells, &before_cells)?;
    let cells = serde_json::to_value(&doc.cells).map_err(AppError::internal)?;
    let mut conn = ctx.db.conn().await?;
    repo::update(&mut conn, GridId(before.id), doc.write(&cells)).await?;
    let row = load(&mut conn, slug).await?;
    drop(conn);
    let grid = to_grid(ctx, row, subject, &resolver);
    let details = update_details(&before, &before_cells, &doc);
    ctx.audit
        .record(audit_event(&grid, user, ContentAuditAction::Edit, details))
        .await;
    Ok(grid)
}

pub async fn delete(ctx: &AppContext, subject: &Subject, slug: &str) -> Result<(), AppError> {
    let mut conn = ctx.db.conn().await?;
    let (row, user) = load_editable(ctx, &mut conn, subject, slug).await?;
    repo::delete(&mut conn, GridId(row.id)).await?;
    drop(conn);
    ctx.audit
        .record(ContentEvent {
            actor: user,
            target_kind: ContentTargetKind::Grid,
            target_id: row.id,
            target_name: &row.title,
            target_slug: Some(&row.slug),
            owner: Some(UserId(row.created_by)),
            subject: None,
            action: ContentAuditAction::Delete,
            details: ContentAuditDetails::default(),
        })
        .await;
    Ok(())
}

pub async fn fork(
    ctx: &AppContext,
    subject: &Subject,
    slug: &str,
    server: Option<Server>,
) -> Result<Grid, AppError> {
    let user = actor(subject)?;
    let mut conn = ctx.db.conn().await?;
    let source = load(&mut conn, slug).await?;
    let doc = template_of(&source);
    let new_slug = insert_capped(&mut conn, &doc, user, Some(GridId(source.id))).await?;
    let row = load(&mut conn, &new_slug).await?;
    drop(conn);
    let resolver = Resolver::new(ctx, server_or_default(ctx, server), &[]).await;
    let grid = to_grid(ctx, row, subject, &resolver);
    let details = ContentAuditDetails {
        forked_from: Some(source.title),
        ..Default::default()
    };
    ctx.audit
        .record(audit_event(&grid, user, ContentAuditAction::Fork, details))
        .await;
    Ok(grid)
}

async fn load(conn: &mut Conn, slug: &str) -> Result<GridRow, AppError> {
    repo::find_by_slug(conn, slug)
        .await?
        .ok_or(AppError::NotFound(None))
}

async fn load_editable(
    ctx: &AppContext,
    conn: &mut Conn,
    subject: &Subject,
    slug: &str,
) -> Result<(GridRow, UserId), AppError> {
    let user = actor(subject)?;
    let row = load(conn, slug).await?;
    let resource = Resource::Grid {
        owner: UserId(row.created_by),
    };
    ctx.policy.require(subject, Action::EditGrid, &resource)?;
    Ok((row, user))
}

async fn insert_capped(
    conn: &mut Conn,
    doc: &GridDocument,
    user: UserId,
    template: Option<GridId>,
) -> Result<String, AppError> {
    let slug = slug_for(&doc.title);
    let cells = serde_json::to_value(&doc.cells).map_err(AppError::internal)?;
    if !repo::create_capped(
        conn,
        &slug,
        doc.write(&cells),
        user,
        template,
        MAX_GRIDS_PER_USER,
    )
    .await?
    {
        return Err(AppError::Conflict(format!(
            "you already have {MAX_GRIDS_PER_USER} grids, the most one account can keep: delete one to make another"
        )));
    }
    Ok(slug)
}

fn audit_event(
    grid: &Grid,
    actor: UserId,
    action: ContentAuditAction,
    details: ContentAuditDetails,
) -> ContentEvent<'_> {
    ContentEvent {
        actor,
        target_kind: ContentTargetKind::Grid,
        target_id: grid.id,
        target_name: &grid.title,
        target_slug: Some(&grid.slug),
        owner: Some(UserId(grid.owner.id)),
        subject: None,
        action,
        details,
    }
}

fn update_details(
    before: &GridRow,
    old: &[StoredCell],
    after: &GridDocument,
) -> ContentAuditDetails {
    let cells_changed = (0..old.len().max(after.cells.len()))
        .filter(|&i| old.get(i) != after.cells.get(i))
        .count();
    ContentAuditDetails {
        renamed_from: (before.title != after.title).then(|| before.title.clone()),
        description_changed: (before.description != after.description).then_some(true),
        listed: (before.is_listed != after.is_listed).then_some(after.is_listed),
        size: (before.rows != after.rows || before.cols != after.cols)
            .then(|| format!("{}x{}", after.rows, after.cols)),
        kinds: (kinds_of(before) != after.entity_kinds).then(|| {
            after
                .entity_kinds
                .iter()
                .map(|k| k.as_str().to_owned())
                .collect()
        }),
        cells_changed: (cells_changed > 0)
            .then(|| u32::try_from(cells_changed).unwrap_or(u32::MAX)),
        ..Default::default()
    }
}

pub fn checked(input: GridInput) -> Result<GridDocument, AppError> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(AppError::bad_request("title is required"));
    }
    validate_length("title", Some(title), TITLE_MAX)?;
    let description = input
        .description
        .map(|d| d.trim().to_owned())
        .filter(|d| !d.is_empty());
    validate_length("description", description.as_deref(), DESCRIPTION_MAX)?;

    let mut entity_kinds = input.entity_kinds;
    entity_kinds.sort_unstable();
    entity_kinds.dedup();
    if entity_kinds.is_empty() {
        return Err(AppError::bad_request(
            "entity_kinds: a grid allows at least one type",
        ));
    }
    let rows = size("rows", input.rows)?;
    let cols = size("cols", input.cols)?;
    let expected = cell_count(rows, cols);
    if input.cells.len() != expected {
        return Err(AppError::BadRequest(format!(
            "cells: a {rows} x {cols} grid has {expected} cells, got {}",
            input.cells.len()
        )));
    }
    let cells = input
        .cells
        .into_iter()
        .enumerate()
        .map(|(i, cell)| checked_cell(i, cell, &entity_kinds))
        .collect::<Result<_, _>>()?;
    Ok(GridDocument {
        title: title.to_owned(),
        description,
        rows,
        cols,
        cells,
        is_listed: input.is_listed,
        entity_kinds,
    })
}

const fn cell_count(rows: i16, cols: i16) -> usize {
    rows.unsigned_abs() as usize * cols.unsigned_abs() as usize
}

fn size(field: &str, value: i32) -> Result<i16, AppError> {
    if (SIZE_MIN..=SIZE_MAX).contains(&value)
        && let Ok(v) = i16::try_from(value)
    {
        return Ok(v);
    }
    Err(AppError::BadRequest(format!(
        "{field} must be {SIZE_MIN} to {SIZE_MAX}, got {value}"
    )))
}

fn checked_cell(
    index: usize,
    cell: GridCellInput,
    allowed: &[EntityKind],
) -> Result<StoredCell, AppError> {
    let label = cell.label.trim();
    validate_length(&format!("cell {index} label"), Some(label), LABEL_MAX)?;
    let (entity_kind, entity_id) = match (cell.entity_kind, cell.entity_id) {
        (Some(kind), Some(_)) if !allowed.contains(&kind) => {
            return Err(AppError::BadRequest(format!(
                "cell {index}: `{}` is not one of this grid's allowed types",
                kind.as_str()
            )));
        }
        (Some(kind), Some(id)) => (Some(kind.as_str().to_owned()), Some(id)),
        (None, None) => (None, None),
        _ => {
            return Err(AppError::BadRequest(format!(
                "cell {index}: entity_kind and entity_id are set together or not at all"
            )));
        }
    };
    Ok(StoredCell {
        label: label.to_owned(),
        entity_kind,
        entity_id,
    })
}

fn validate_entities(
    resolver: &Resolver,
    cells: &[StoredCell],
    kept: &[StoredCell],
) -> Result<(), AppError> {
    let held: Vec<_> = kept.iter().filter_map(StoredCell::entity).collect();
    for (kind, id) in cells.iter().filter_map(StoredCell::entity) {
        if !held.contains(&(kind, id)) {
            entities::validate(resolver.sources(), kind, id)?;
        }
    }
    Ok(())
}

fn template_of(source: &GridRow) -> GridDocument {
    GridDocument {
        title: source.title.clone(),
        description: source.description.clone(),
        rows: source.rows,
        cols: source.cols,
        cells: stored_cells(source)
            .into_iter()
            .map(|cell| StoredCell {
                label: cell.label,
                entity_kind: None,
                entity_id: None,
            })
            .collect(),
        is_listed: source.is_listed,
        entity_kinds: kinds_of(source),
    }
}

#[must_use]
pub fn slug_for(title: &str) -> String {
    let base: String = title.chars().take(SLUG_BASE_MAX).collect();
    if base.chars().any(char::is_alphanumeric) {
        generate_slug(&base)
    } else {
        generate_slug("grid")
    }
}

fn owner_of(row: &GridRow) -> GridOwner {
    GridOwner {
        id: row.created_by,
        name: row.owner_name.clone(),
    }
}

fn to_grid(ctx: &AppContext, row: GridRow, subject: &Subject, resolver: &Resolver) -> Grid {
    let can_edit = ctx
        .policy
        .can(
            subject,
            Action::EditGrid,
            &Resource::Grid {
                owner: UserId(row.created_by),
            },
        )
        .is_allowed();
    let template_visible = row.template_listed.unwrap_or(false)
        || subject
            .user_id()
            .is_some_and(|u| Some(u.0) == row.template_owner);
    let cells = stored_cells(&row)
        .into_iter()
        .map(|cell| to_cell(cell, resolver))
        .collect();
    Grid {
        owner: owner_of(&row),
        entity_kinds: kinds_of(&row),
        kinds_locked: row.template_of.is_some(),
        template_of: row
            .template_slug
            .zip(row.template_title)
            .filter(|_| template_visible)
            .map(|(slug, title)| GridTemplateRef { slug, title }),
        id: row.id,
        slug: row.slug,
        title: row.title,
        description: row.description,
        rows: i32::from(row.rows),
        cols: i32::from(row.cols),
        cells,
        is_listed: row.is_listed,
        fork_count: row.fork_count,
        can_edit,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn to_cell(cell: StoredCell, resolver: &Resolver) -> GridCell {
    let pick = cell.entity().map(|(kind, id)| (kind, id.to_owned()));
    let (entity, entity_server) = pick
        .as_ref()
        .and_then(|(kind, id)| resolver.resolve(*kind, id))
        .unzip();
    let (entity_kind, entity_id) = pick.unzip();
    GridCell {
        label: cell.label,
        entity_kind,
        entity_id,
        entity,
        entity_server: entity_server.flatten(),
    }
}

fn to_summary(row: &GridRow, resolver: &Resolver) -> GridSummary {
    GridSummary {
        slug: row.slug.clone(),
        title: row.title.clone(),
        rows: i32::from(row.rows),
        cols: i32::from(row.cols),
        owner: owner_of(row),
        fork_count: row.fork_count,
        is_listed: row.is_listed,
        entity_kinds: kinds_of(row),
        updated_at: row.updated_at,
        preview: preview_of(
            &stored_cells(row),
            cell_count(row.rows, row.cols),
            |kind, id| resolver.resolve(kind, id),
        ),
    }
}

pub fn preview_of(
    cells: &[StoredCell],
    len: usize,
    resolve: impl Fn(EntityKind, &str) -> Option<(EntitySummary, Option<Server>)>,
) -> Vec<Option<GridPreviewIcon>> {
    let mut preview: Vec<_> = cells
        .iter()
        .take(len)
        .map(|cell| {
            let (kind, id) = cell.entity()?;
            let (entity, server) = resolve(kind, id)?;
            Some(GridPreviewIcon {
                kind: entity.kind,
                icon: entity.icon?,
                server,
            })
        })
        .collect();
    preview.resize(len, None);
    preview
}

#[must_use]
pub fn picked_kinds(cells: &[StoredCell]) -> Vec<EntityKind> {
    cells
        .iter()
        .filter_map(StoredCell::entity)
        .map(|(kind, _)| kind)
        .collect()
}

fn rows_kinds(rows: &[GridRow]) -> Vec<EntityKind> {
    rows.iter()
        .flat_map(|r| picked_kinds(&stored_cells(r)))
        .collect()
}

pub struct Resolver {
    sources: Vec<(Option<Server>, Arc<Snapshot>, StoryCatalogues)>,
}

impl Resolver {
    pub async fn new(ctx: &AppContext, server: Server, kinds: &[EntityKind]) -> Self {
        let server = server.data_server();
        let snapshots = &ctx.snapshots;
        let mut found: Vec<(Option<Server>, Server, Arc<Snapshot>)> = snapshots
            .get(server)
            .map(|own| (None, server, own))
            .or_else(|| {
                let default = snapshots.default_server();
                snapshots.get(default).map(|own| (None, default, own))
            })
            .into_iter()
            .collect();
        for other in fallback_order(server, &ctx.config.server.servers) {
            if let Some(data) = snapshots.get(other)
                && !found.iter().any(|(_, _, s)| Arc::ptr_eq(s, &data))
            {
                found.push((Some(other), other, data));
            }
        }
        let mut sources = Vec::with_capacity(found.len());
        for (marker, server, data) in found {
            let story = StoryCatalogues::gather(ctx, server, kinds).await;
            sources.push((marker, data, story));
        }
        Self { sources }
    }

    #[must_use]
    pub fn resolve(&self, kind: EntityKind, id: &str) -> Option<(EntitySummary, Option<Server>)> {
        self.sources.iter().find_map(|(server, data, story)| {
            entities::resolve(&data.model, story, kind, id)
                .map(|e| (romanize_cn_operator(e, *server), *server))
        })
    }

    pub fn sources(&self) -> impl Iterator<Item = (&GameModel, &StoryCatalogues)> {
        self.sources
            .iter()
            .map(|(_, data, story)| (&data.model, story))
    }
}

#[must_use]
pub fn romanize_cn_operator(mut entity: EntitySummary, from: Option<Server>) -> EntitySummary {
    if entity.kind == EntityKind::Operator
        && from.is_some_and(|s| s.data_server() == Server::CN)
        && let Some(FacetValue::One(appellation)) = entity.facets.get("appellation")
        && !appellation.trim().is_empty()
    {
        entity.name = appellation.clone();
    }
    entity
}

#[must_use]
pub fn fallback_order(server: Server, configured: &[Server]) -> Vec<Server> {
    let server = server.data_server();
    let configured = configured.iter().map(|s| s.data_server());
    let cn_first = configured
        .clone()
        .any(|s| s == Server::CN)
        .then_some(Server::CN);
    let mut out = Vec::new();
    for s in cn_first.into_iter().chain(configured) {
        if s != server && !out.contains(&s) {
            out.push(s);
        }
    }
    out
}
