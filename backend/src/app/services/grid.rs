//! Grids: a titled R x C board whose cells each carry a free-text label and,
//! optionally, one tier-list entity.
//!
//! A grid is read and written whole. Cells are stored as `(kind, id)` only and
//! resolved per request against the reader's server, like tier list
//! placements. A pick that server does not know is tried on the other loaded
//! servers, CN first, so an operator only CN has released still shows, with
//! `entity_server` naming where it came from. A pick no loaded server knows
//! comes back with `entity: None` and its raw ref, and an edit round-trip
//! keeps it.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use ts_rs::TS;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::services::tier_entity::{self, EntitySummary, FacetValue};
use crate::app::services::tier_list::{generate_slug, validate_entity};
use crate::app::state::AppState;
use crate::app::validation::{validate_length, validate_opt_length};
use crate::core::auth::permissions::GlobalRole;
use crate::core::gamedata::assets::AssetIndex;
use crate::core::gamedata::types::GameData;
use crate::core::hypergryph::constants::Server;
use crate::database::models::grid::{GridDocument, GridRow, StoredCell};
use crate::database::models::tier_list::EntityKind;
use crate::database::queries::grids as queries;
use crate::database::queries::grids::ListOrder;

pub const SIZE_MIN: i32 = 1;
pub const SIZE_MAX: i32 = 10;
// `grids.title varchar(100)` (v031); raising it needs a migration.
pub const TITLE_MAX: usize = 100;
pub const DESCRIPTION_MAX: usize = 2000;
pub const LABEL_MAX: usize = 60;
pub const PER_PAGE_DEFAULT: i32 = 24;
pub const PER_PAGE_MAX: i32 = 60;
const MAX_GRIDS_PER_USER: i64 = 50;
/// Slug base length: the random suffix keeps it unique, so the rest is only
/// for the reader of the URL.
const SLUG_BASE_MAX: usize = 60;

/// One cell as the editor sends it. `entity_kind` and `entity_id` are set
/// together or not at all.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCellInput {
    pub label: String,
    pub entity_kind: Option<EntityKind>,
    pub entity_id: Option<String>,
}

/// A whole grid as the editor saves it. `cells` is row-major and exactly
/// `rows * cols` long, and every pick is of a kind in `entity_kinds`.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridInput {
    pub title: String,
    pub description: Option<String>,
    pub rows: i32,
    pub cols: i32,
    pub cells: Vec<GridCellInput>,
    pub is_listed: bool,
    /// The kinds a cell may hold: at least one. Stored deduplicated and in
    /// canonical order. A fork's set is its template's and cannot change.
    pub entity_kinds: Vec<EntityKind>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCell {
    pub label: String,
    pub entity_kind: Option<EntityKind>,
    pub entity_id: Option<String>,
    /// `None` when the cell is empty, or when no loaded server's game data has
    /// such an entity; the raw `entity_kind` and `entity_id` are still sent.
    pub entity: Option<EntitySummary>,
    /// The server `entity` was resolved on when the reader's server does not
    /// know it (an operator only CN has released), so its icon is fetched
    /// from that server. `None` when it resolved on the reader's server, or
    /// did not resolve.
    #[ts(type = "string | null")]
    #[schema(value_type = Option<String>)]
    pub entity_server: Option<Server>,
}

/// One filled cell of a card thumbnail: its art's path, the kind (which
/// decides how the art sits in the cell), and the server to fetch it from
/// when it is not the reader's, with [`GridCell::entity_server`]'s meaning.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridPreviewIcon {
    pub kind: EntityKind,
    /// Same form as [`EntitySummary::icon`].
    pub icon: String,
    #[ts(type = "string | null")]
    #[schema(value_type = Option<String>)]
    pub server: Option<Server>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridOwner {
    pub id: Uuid,
    /// The owner's in-game nickname, or their uid when it is empty.
    pub name: String,
}

/// The grid a fork was made from.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridTemplateRef {
    pub slug: String,
    pub title: String,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub rows: i32,
    pub cols: i32,
    /// Row-major: cell `(r, c)` is `cells[r * cols + c]`.
    pub cells: Vec<GridCell>,
    pub is_listed: bool,
    /// The kinds a cell may hold, in canonical order.
    pub entity_kinds: Vec<EntityKind>,
    /// `entity_kinds` is set by the template this grid was forked from and a
    /// save cannot change it. True for every fork, including one whose
    /// `template_of` is hidden from this viewer; false once the source is
    /// deleted, which clears the link.
    pub kinds_locked: bool,
    pub owner: GridOwner,
    /// `None` for an original, or when the source has been deleted.
    pub template_of: Option<GridTemplateRef>,
    #[ts(type = "number")]
    pub fork_count: i64,
    /// The caller may save or delete it: its owner, or a site admin.
    pub can_edit: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSummary {
    pub slug: String,
    pub title: String,
    pub rows: i32,
    pub cols: i32,
    pub owner: GridOwner,
    #[ts(type = "number")]
    pub fork_count: i64,
    pub is_listed: bool,
    /// The kinds a cell may hold, in canonical order.
    pub entity_kinds: Vec<EntityKind>,
    pub updated_at: DateTime<Utc>,
    /// The board in miniature for a card thumbnail: one entry per cell,
    /// row-major like [`Grid::cells`], `rows * cols` long. `None` for an
    /// empty cell, and for a pick no loaded server knows or that has no art.
    pub preview: Vec<Option<GridPreviewIcon>>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridListResponse {
    pub items: Vec<GridSummary>,
    #[ts(type = "number")]
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// Who is reading: their user id and role, when signed in as a user.
pub type Viewer = Option<(Uuid, GlobalRole)>;

/// The grid as `GET /grids/{slug}` serves it, resolved against `server`.
pub async fn get(
    state: &AppState,
    slug: &str,
    server: Server,
    viewer: Viewer,
) -> Result<Grid, ApiError> {
    let row = load(state, slug).await?;
    Ok(to_grid(row, viewer, &Resolver::new(state, server)))
}

/// One page of listed grids, `q` matching anywhere in the title.
pub async fn list(
    state: &AppState,
    order: ListOrder,
    q: Option<&str>,
    page: Option<i32>,
    per_page: Option<i32>,
    server: Server,
) -> Result<GridListResponse, ApiError> {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(PER_PAGE_DEFAULT).clamp(1, PER_PAGE_MAX);
    let offset = i64::from(page - 1) * i64::from(per_page);
    let pattern = q.and_then(title_pattern);
    let (rows, total) = queries::find_listed(
        &state.db,
        pattern.as_deref(),
        order,
        i64::from(per_page),
        offset,
    )
    .await?;
    Ok(GridListResponse {
        items: summaries(state, rows, server),
        total,
        page,
        per_page,
    })
}

/// Every grid `user_id` owns, listed or not, latest edit first.
pub async fn mine(
    state: &AppState,
    user_id: Uuid,
    server: Server,
) -> Result<Vec<GridSummary>, ApiError> {
    let rows = queries::find_by_user(&state.db, user_id).await?;
    Ok(summaries(state, rows, server))
}

pub async fn create(
    state: &AppState,
    user_id: Uuid,
    role: GlobalRole,
    input: GridInput,
    server: Server,
) -> Result<Grid, ApiError> {
    let doc = checked(input)?;
    validate_entities(state, &doc.cells, &[])?;
    let slug = insert_capped(state, &doc, user_id, None).await?;
    get(state, &slug, server, Some((user_id, role))).await
}

/// Replace the grid's document. A pick the stored grid already holds is kept
/// without re-validation, so a grid whose entity has left the game data still
/// saves.
pub async fn update(
    state: &AppState,
    slug: &str,
    user_id: Uuid,
    role: GlobalRole,
    input: GridInput,
    server: Server,
) -> Result<Grid, ApiError> {
    let row = load_editable(state, slug, user_id, role).await?;
    let doc = checked(input)?;
    check_locked_kinds(&row, &doc.entity_kinds)?;
    validate_entities(state, &doc.cells, &row.cells.0)?;
    queries::update(&state.db, row.id, &doc).await?;
    get(state, slug, server, Some((user_id, role))).await
}

pub async fn delete(
    state: &AppState,
    slug: &str,
    user_id: Uuid,
    role: GlobalRole,
) -> Result<(), ApiError> {
    let row = load_editable(state, slug, user_id, role).await?;
    queries::delete(&state.db, row.id).await?;
    Ok(())
}

/// A new grid owned by `user_id` with the source's title, description, size
/// and labels, and no picks.
pub async fn fork(
    state: &AppState,
    slug: &str,
    user_id: Uuid,
    role: GlobalRole,
    server: Server,
) -> Result<Grid, ApiError> {
    let source = load(state, slug).await?;
    let doc = template_of(&source);
    let new_slug = insert_capped(state, &doc, user_id, Some(source.id)).await?;
    get(state, &new_slug, server, Some((user_id, role))).await
}

async fn load(state: &AppState, slug: &str) -> Result<GridRow, ApiError> {
    queries::find_by_slug(&state.db, slug)
        .await?
        .ok_or(ApiError::NotFound)
}

/// The grid by `slug` (404), if `user_id` may edit it (403).
async fn load_editable(
    state: &AppState,
    slug: &str,
    user_id: Uuid,
    role: GlobalRole,
) -> Result<GridRow, ApiError> {
    let row = load(state, slug).await?;
    if may_edit(row.created_by, Some((user_id, role))) {
        Ok(row)
    } else {
        Err(ApiError::Forbidden)
    }
}

/// Insert the grid under a fresh slug and return it, or `409` when `user_id`
/// already owns the most one account can keep. The cap is checked inside the
/// insert's transaction.
async fn insert_capped(
    state: &AppState,
    doc: &GridDocument,
    user_id: Uuid,
    template_of: Option<Uuid>,
) -> Result<String, ApiError> {
    let slug = slug_for(&doc.title);
    if !queries::create_capped(
        &state.db,
        &slug,
        doc,
        user_id,
        template_of,
        MAX_GRIDS_PER_USER,
    )
    .await?
    {
        return Err(ApiError::Conflict(format!(
            "you already have {MAX_GRIDS_PER_USER} grids, the most one account can keep: delete one to make another"
        )));
    }
    Ok(slug)
}

/// Tier-list admins moderate community content site-wide, grids included.
fn may_edit(owner: Uuid, viewer: Viewer) -> bool {
    viewer.is_some_and(|(id, role)| id == owner || role.is_tier_list_admin())
}

/// The input as stored: title and labels trimmed, an empty description
/// dropped, the allowed kinds normalized, and every shape, length and kind
/// rule checked. Entity refs are checked separately by
/// [`validate_entities`], which needs the loaded game data.
///
/// # Errors
/// `400` naming the first rule broken.
fn checked(input: GridInput) -> Result<GridDocument, ApiError> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(ApiError::BadRequest("title is required".into()));
    }
    validate_length("title", title, TITLE_MAX)?;
    let description = input
        .description
        .map(|d| d.trim().to_owned())
        .filter(|d| !d.is_empty());
    validate_opt_length("description", description.as_deref(), DESCRIPTION_MAX)?;

    let entity_kinds = allowed_kinds(&input.entity_kinds)?;
    let rows = size("rows", input.rows)?;
    let cols = size("cols", input.cols)?;
    let expected = cell_count(rows, cols);
    if input.cells.len() != expected {
        return Err(ApiError::BadRequest(format!(
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

/// `kinds` deduplicated and in canonical [`EntityKind`] order.
///
/// Unlike a tier list's offered kinds, which keep the editor's order, a grid's
/// set is compared whole (a fork's lock), so it has one stored form.
///
/// # Errors
/// `400` when the set is empty: a grid allows at least one kind.
fn allowed_kinds(kinds: &[EntityKind]) -> Result<Vec<EntityKind>, ApiError> {
    let mut out = kinds.to_vec();
    out.sort_unstable();
    out.dedup();
    if out.is_empty() {
        return Err(ApiError::BadRequest(
            "entity_kinds: a grid allows at least one type".into(),
        ));
    }
    Ok(out)
}

/// A fork's allowed kinds are its template's: a save may resend them, in any
/// order, but not change them. Keyed on the stored link, not on whether the
/// viewer may see the template.
///
/// # Errors
/// `400` when `row` is a fork and `kinds` is a different set.
fn check_locked_kinds(row: &GridRow, kinds: &[EntityKind]) -> Result<(), ApiError> {
    if row.template_of.is_some() && row.kinds() != kinds {
        return Err(ApiError::BadRequest(
            "entity_kinds: allowed types are set by the template".into(),
        ));
    }
    Ok(())
}

/// The cells a `rows` x `cols` board holds. Stored sizes are 1 to 10, so the
/// sign never matters; `unsigned_abs` only keeps the cast lossless.
fn cell_count(rows: i16, cols: i16) -> usize {
    usize::from(rows.unsigned_abs()) * usize::from(cols.unsigned_abs())
}

fn size(field: &str, value: i32) -> Result<i16, ApiError> {
    if (SIZE_MIN..=SIZE_MAX).contains(&value)
        && let Ok(v) = i16::try_from(value)
    {
        return Ok(v);
    }
    Err(ApiError::BadRequest(format!(
        "{field} must be {SIZE_MIN} to {SIZE_MAX}, got {value}"
    )))
}

fn checked_cell(
    index: usize,
    cell: GridCellInput,
    allowed: &[EntityKind],
) -> Result<StoredCell, ApiError> {
    let label = cell.label.trim();
    validate_length(&format!("cell {index} label"), label, LABEL_MAX)?;
    let (entity_kind, entity_id) = match (cell.entity_kind, cell.entity_id) {
        (Some(kind), Some(_)) if !allowed.contains(&kind) => {
            return Err(ApiError::BadRequest(format!(
                "cell {index}: `{}` is not one of this grid's allowed types",
                kind.as_str()
            )));
        }
        (Some(kind), Some(id)) => (Some(kind.as_str().to_owned()), Some(id)),
        (None, None) => (None, None),
        _ => {
            return Err(ApiError::BadRequest(format!(
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

/// Every pick some loaded server knows, except those `kept` already holds.
fn validate_entities(
    state: &AppState,
    cells: &[StoredCell],
    kept: &[StoredCell],
) -> Result<(), ApiError> {
    let held: Vec<_> = kept.iter().filter_map(StoredCell::entity).collect();
    for (kind, id) in cells.iter().filter_map(StoredCell::entity) {
        if !held.contains(&(kind, id)) {
            validate_entity(state, kind, id)?;
        }
    }
    Ok(())
}

/// What a fork starts from: the source's labels and allowed kinds, no picks,
/// listed only when the source is, so a fork never publishes an unlisted grid.
fn template_of(source: &GridRow) -> GridDocument {
    GridDocument {
        title: source.title.clone(),
        description: source.description.clone(),
        rows: source.rows,
        cols: source.cols,
        cells: source
            .cells
            .0
            .iter()
            .map(|cell| StoredCell {
                label: cell.label.clone(),
                entity_kind: None,
                entity_id: None,
            })
            .collect(),
        is_listed: source.is_listed,
        entity_kinds: source.kinds(),
    }
}

/// `q` as a case-insensitive substring pattern for `ILIKE ... ESCAPE '\'`,
/// or `None` when it is blank.
fn title_pattern(q: &str) -> Option<String> {
    let q = q.trim();
    if q.is_empty() {
        return None;
    }
    let mut pattern = String::with_capacity(q.len() + 2);
    pattern.push('%');
    for c in q.chars() {
        if matches!(c, '\\' | '%' | '_') {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    pattern.push('%');
    Some(pattern)
}

fn slug_for(title: &str) -> String {
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

fn to_grid(row: GridRow, viewer: Viewer, resolver: &Resolver) -> Grid {
    let owner = owner_of(&row);
    let can_edit = may_edit(row.created_by, viewer);
    let entity_kinds = row.kinds();
    let kinds_locked = row.template_of.is_some();
    let template_visible = template_visible(row.template_listed, row.template_owner, viewer);
    let template_of = row
        .template_slug
        .zip(row.template_title)
        .filter(|_| template_visible)
        .map(|(slug, title)| GridTemplateRef { slug, title });
    let cells = row
        .cells
        .0
        .into_iter()
        .map(|cell| to_cell(cell, resolver))
        .collect();
    Grid {
        id: row.id,
        slug: row.slug,
        title: row.title,
        description: row.description,
        rows: i32::from(row.rows),
        cols: i32::from(row.cols),
        cells,
        is_listed: row.is_listed,
        entity_kinds,
        kinds_locked,
        owner,
        template_of,
        fork_count: row.fork_count,
        can_edit,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn to_cell(cell: StoredCell, resolver: &Resolver) -> GridCell {
    // A kind this build does not know is dropped with its id, so the editor
    // never sends back half a pick.
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

/// Game data to resolve picks against: the reader's server first, then every
/// other loaded server in [`fallback_order`].
pub(crate) struct Resolver {
    /// `None` marks the reader's own server.
    sources: Vec<(Option<Server>, Arc<GameData>, Arc<AssetIndex>)>,
}

impl Resolver {
    pub(crate) fn new(state: &AppState, server: Server) -> Self {
        let own = state.server_data(server);
        let mut tried = vec![Arc::clone(&own)];
        let mut sources = vec![(None, own.game_data.load_full(), own.asset_index.load_full())];
        for other in fallback_order(server, &state.config.servers) {
            // Bilibili shares CN's cell; a server already tried adds nothing.
            let Some(data) = state
                .servers
                .get(&other)
                .filter(|d| d.loaded.load(Ordering::Acquire))
            else {
                continue;
            };
            if tried.iter().any(|t| Arc::ptr_eq(t, data)) {
                continue;
            }
            tried.push(Arc::clone(data));
            sources.push((
                Some(other),
                data.game_data.load_full(),
                data.asset_index.load_full(),
            ));
        }
        Self { sources }
    }

    /// The pick's summary and, when it is not the reader's, the server that
    /// knew it.
    pub(crate) fn resolve(
        &self,
        kind: EntityKind,
        id: &str,
    ) -> Option<(EntitySummary, Option<Server>)> {
        first_resolved(self.sources.iter(), |(server, gd, assets)| {
            tier_entity::resolve(gd, assets, kind, id)
                .map(|e| (romanize_cn_operator(e, *server), *server))
        })
    }
}

/// An operator only CN-family data knows, read by a viewer on another server,
/// is named by its `appellation` (`char_1015_aglna2`: `Angelina the Mellow
/// Wish`, not `予愿安洁莉娜`). The appellation is the romanized name, or for a
/// few the Cyrillic one (`Вий`), and either reads better than Chinese outside
/// CN. `from` is [`Resolver`]'s marker: `None` for the viewer's own server, so
/// a CN viewer keeps the Chinese name.
fn romanize_cn_operator(mut entity: EntitySummary, from: Option<Server>) -> EntitySummary {
    if entity.kind == EntityKind::Operator
        && matches!(from, Some(Server::CN | Server::Bilibili))
        && let Some(FacetValue::One(appellation)) = entity.facets.get("appellation")
        && !appellation.trim().is_empty()
    {
        entity.name = appellation.clone();
    }
    entity
}

/// The servers a pick the reader's `server` does not know is tried on: CN
/// first, since it releases everything first, then the rest of `configured`
/// in its order, without `server` and without repeats.
fn fallback_order(server: Server, configured: &[Server]) -> Vec<Server> {
    let mut out = Vec::with_capacity(configured.len());
    let cn_first = configured
        .contains(&Server::CN)
        .then_some(Server::CN)
        .into_iter();
    for s in cn_first.chain(configured.iter().copied()) {
        if s != server && !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

/// The first source, in order, that `resolve` answers for.
fn first_resolved<S, T>(
    sources: impl IntoIterator<Item = S>,
    resolve: impl FnMut(S) -> Option<T>,
) -> Option<T> {
    sources.into_iter().find_map(resolve)
}

/// An unlisted source's slug is shown only to its owner.
fn template_visible(listed: Option<bool>, owner: Option<Uuid>, viewer: Viewer) -> bool {
    listed.unwrap_or(false) || viewer.is_some_and(|(id, _)| Some(id) == owner)
}

/// `len` thumbnail entries, one per cell in cell order: the pick's art as
/// `resolve` finds it, `None` where there is no pick, it does not resolve, or
/// it resolves without art. Saves keep the cell count at `rows * cols`; a
/// stored list that is not is cut or padded with empty cells, so the card
/// always draws the grid's shape.
fn preview_of(
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

/// Card summaries of `rows`, previews resolved against `server` first.
fn summaries(state: &AppState, rows: Vec<GridRow>, server: Server) -> Vec<GridSummary> {
    let resolver = Resolver::new(state, server);
    rows.into_iter()
        .map(|row| to_summary(row, &resolver))
        .collect()
}

fn to_summary(row: GridRow, resolver: &Resolver) -> GridSummary {
    let owner = owner_of(&row);
    let entity_kinds = row.kinds();
    let preview = preview_of(&row.cells.0, cell_count(row.rows, row.cols), |kind, id| {
        resolver.resolve(kind, id)
    });
    GridSummary {
        slug: row.slug,
        title: row.title,
        rows: i32::from(row.rows),
        cols: i32::from(row.cols),
        owner,
        fork_count: row.fork_count,
        is_listed: row.is_listed,
        entity_kinds,
        updated_at: row.updated_at,
        preview,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(label: &str) -> GridCellInput {
        GridCellInput {
            label: label.into(),
            entity_kind: None,
            entity_id: None,
        }
    }

    fn input(rows: i32, cols: i32) -> GridInput {
        let n = usize::try_from(rows * cols).unwrap_or(0);
        GridInput {
            title: "  Favourites  ".into(),
            description: Some("   ".into()),
            rows,
            cols,
            cells: (0..n).map(|_| cell("Favorite Vanguard")).collect(),
            is_listed: true,
            entity_kinds: EntityKind::ALL.to_vec(),
        }
    }

    fn bad_request<T: std::fmt::Debug>(result: Result<T, ApiError>) -> String {
        match result {
            Err(ApiError::BadRequest(msg)) => msg,
            other => panic!("expected a 400, got {other:?}"),
        }
    }

    fn row(template_of: Option<Uuid>, kinds: &[&str]) -> GridRow {
        let now = Utc::now();
        GridRow {
            id: Uuid::from_u128(10),
            slug: "g-abcdef".into(),
            title: "G".into(),
            description: None,
            created_by: Uuid::from_u128(1),
            rows: 1,
            cols: 1,
            cells: sqlx::types::Json(vec![]),
            is_listed: true,
            entity_kinds: kinds.iter().map(|k| (*k).to_owned()).collect(),
            template_of,
            created_at: now,
            updated_at: now,
            owner_name: "o".into(),
            fork_count: 0,
            template_slug: None,
            template_title: None,
            template_listed: None,
            template_owner: None,
        }
    }

    #[test]
    fn a_valid_grid_is_trimmed_and_kept() {
        let doc = checked(input(6, 6)).expect("valid");
        assert_eq!(doc.title, "Favourites");
        assert_eq!(doc.description, None);
        assert_eq!((doc.rows, doc.cols), (6, 6));
        assert_eq!(doc.cells.len(), 36);
        assert!(doc.cells.iter().all(|c| c.entity_kind.is_none()));
    }

    #[test]
    fn sizes_are_one_to_ten() {
        assert!(checked(input(1, 1)).is_ok());
        assert!(checked(input(10, 10)).is_ok());
        for (rows, cols) in [(0, 3), (3, 0), (11, 3), (3, 11), (-1, 2)] {
            let mut grid = input(3, 3);
            grid.rows = rows;
            grid.cols = cols;
            assert!(bad_request(checked(grid)).contains("must be 1 to 10"));
        }
    }

    #[test]
    fn the_cell_count_must_match_the_size() {
        let mut grid = input(3, 4);
        grid.cells.pop();
        assert!(bad_request(checked(grid)).contains("has 12 cells, got 11"));
        let mut grid = input(3, 4);
        grid.cells.push(cell(""));
        assert!(bad_request(checked(grid)).contains("got 13"));
    }

    #[test]
    fn a_half_specified_entity_is_refused() {
        let mut grid = input(2, 2);
        grid.cells[1].entity_kind = Some(EntityKind::Operator);
        assert!(bad_request(checked(grid)).starts_with("cell 1:"));
        let mut grid = input(2, 2);
        grid.cells[3].entity_id = Some("char_002_amiya".into());
        assert!(bad_request(checked(grid)).starts_with("cell 3:"));
    }

    #[test]
    fn a_full_entity_is_stored_by_its_kind_name() {
        let mut grid = input(1, 2);
        grid.cells[0].entity_kind = Some(EntityKind::StrongholdBond);
        grid.cells[0].entity_id = Some("bond_x".into());
        let doc = checked(grid).expect("valid");
        assert_eq!(doc.cells[0].entity_kind.as_deref(), Some("stronghold_bond"));
        assert_eq!(
            doc.cells[0].entity(),
            Some((EntityKind::StrongholdBond, "bond_x"))
        );
    }

    #[test]
    fn lengths_are_capped_in_chars() {
        let mut grid = input(1, 1);
        grid.cells[0].label = "あ".repeat(LABEL_MAX);
        assert!(checked(grid).is_ok());
        let mut grid = input(1, 1);
        grid.cells[0].label = "a".repeat(LABEL_MAX + 1);
        assert!(bad_request(checked(grid)).contains("cell 0 label"));

        let mut grid = input(1, 1);
        grid.title = "t".repeat(TITLE_MAX + 1);
        assert!(bad_request(checked(grid)).contains("title"));
        let mut grid = input(1, 1);
        grid.title = "   ".into();
        assert_eq!(bad_request(checked(grid)), "title is required");
        let mut grid = input(1, 1);
        grid.description = Some("d".repeat(DESCRIPTION_MAX + 1));
        assert!(bad_request(checked(grid)).contains("description"));
    }

    #[test]
    fn a_grid_allows_at_least_one_kind() {
        let mut grid = input(1, 1);
        grid.entity_kinds = vec![];
        assert!(bad_request(checked(grid)).contains("at least one type"));
    }

    #[test]
    fn allowed_kinds_are_deduplicated_in_canonical_order() {
        let mut grid = input(1, 1);
        grid.entity_kinds = vec![
            EntityKind::Skin,
            EntityKind::Operator,
            EntityKind::Skin,
            EntityKind::Enemy,
        ];
        let doc = checked(grid).expect("valid");
        assert_eq!(
            doc.entity_kinds,
            vec![EntityKind::Operator, EntityKind::Enemy, EntityKind::Skin]
        );
        assert_eq!(doc.kind_names(), vec!["operator", "enemy", "skin"]);
    }

    #[test]
    fn a_pick_outside_the_allowed_kinds_is_refused() {
        let mut grid = input(1, 2);
        grid.entity_kinds = vec![EntityKind::Operator, EntityKind::Skin];
        grid.cells[1].entity_kind = Some(EntityKind::Module);
        grid.cells[1].entity_id = Some("uniequip_002_amiya".into());
        let msg = bad_request(checked(grid));
        assert!(msg.starts_with("cell 1:"), "{msg}");
        assert!(msg.contains("`module`"), "{msg}");

        let mut grid = input(1, 2);
        grid.entity_kinds = vec![EntityKind::Operator];
        grid.cells[0].entity_kind = Some(EntityKind::Operator);
        grid.cells[0].entity_id = Some("char_002_amiya".into());
        assert!(checked(grid).is_ok());
    }

    #[test]
    fn a_forks_kinds_cannot_change() {
        let fork = row(Some(Uuid::from_u128(9)), &["skin", "operator"]);
        assert_eq!(
            bad_request(check_locked_kinds(&fork, &[EntityKind::Operator])),
            "entity_kinds: allowed types are set by the template"
        );
        assert!(
            check_locked_kinds(
                &fork,
                &[EntityKind::Operator, EntityKind::Skin, EntityKind::Enemy]
            )
            .is_err()
        );
    }

    #[test]
    fn a_forks_unchanged_kinds_are_accepted() {
        let fork = row(Some(Uuid::from_u128(9)), &["skin", "operator"]);
        let mut grid = input(1, 1);
        grid.entity_kinds = vec![EntityKind::Skin, EntityKind::Operator, EntityKind::Skin];
        let doc = checked(grid).expect("valid");
        assert!(check_locked_kinds(&fork, &doc.entity_kinds).is_ok());
    }

    #[test]
    fn an_original_may_change_its_kinds() {
        let original = row(None, &["operator"]);
        assert!(check_locked_kinds(&original, &[EntityKind::Enemy]).is_ok());
    }

    #[test]
    fn stored_kinds_read_leniently() {
        assert_eq!(
            row(None, &["skin", "boss", "operator", "skin"]).kinds(),
            vec![EntityKind::Operator, EntityKind::Skin]
        );
        assert_eq!(row(None, &["boss"]).kinds(), EntityKind::ALL.to_vec());
    }

    #[test]
    fn a_fork_copies_the_allowed_kinds() {
        let source = row(None, &["operator", "skin"]);
        assert_eq!(
            template_of(&source).entity_kinds,
            vec![EntityKind::Operator, EntityKind::Skin]
        );
    }

    /// v032 backfilled every grid with the kinds known then, and `kinds()`
    /// sorts by `Ord`, so the default must name real kinds and `ALL` must be in
    /// `Ord` order.
    #[test]
    fn the_v032_default_names_known_kinds_in_canonical_order() {
        let sql = include_str!("../../database/migrations/v032_grid_entity_kinds.sql");
        let start = sql.find("DEFAULT '{").expect("default") + "DEFAULT '{".len();
        let end = start + sql[start..].find('}').expect("closing brace");
        let kinds: Vec<EntityKind> = sql[start..end]
            .split(',')
            .map(|s| s.parse().expect("a known kind"))
            .collect();
        assert!(kinds.windows(2).all(|w| w[0] < w[1]));
        assert!(EntityKind::ALL.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn an_empty_label_is_allowed() {
        let mut grid = input(1, 1);
        grid.cells[0].label = String::new();
        assert_eq!(checked(grid).expect("valid").cells[0].label, "");
    }

    #[test]
    fn an_unknown_stored_kind_drops_the_pick_only() {
        let stored = StoredCell {
            label: "Boss".into(),
            entity_kind: Some("boss".into()),
            entity_id: Some("x".into()),
        };
        assert_eq!(stored.entity(), None);
    }

    #[test]
    fn title_search_escapes_like_wildcards() {
        assert_eq!(title_pattern("  "), None);
        assert_eq!(title_pattern(" Top ").as_deref(), Some("%Top%"));
        assert_eq!(
            title_pattern(r"100%_a\b").as_deref(),
            Some(r"%100\%\_a\\b%")
        );
    }

    #[test]
    fn only_owners_and_admins_edit() {
        let owner = Uuid::from_u128(1);
        let other = Uuid::from_u128(2);
        assert!(may_edit(owner, Some((owner, GlobalRole::User))));
        assert!(!may_edit(owner, Some((other, GlobalRole::User))));
        assert!(!may_edit(owner, Some((other, GlobalRole::TierListEditor))));
        assert!(may_edit(owner, Some((other, GlobalRole::TierListAdmin))));
        assert!(may_edit(owner, Some((other, GlobalRole::SuperAdmin))));
        assert!(!may_edit(owner, None));
    }

    #[test]
    fn a_cn_only_operator_is_named_by_its_appellation_off_cn() {
        let summary = |kind, appellation: Option<&str>| EntitySummary {
            kind,
            id: "char_1015_aglna2".into(),
            name: "予愿安洁莉娜".into(),
            icon: None,
            href: None,
            facets: appellation
                .map(|a| ("appellation".to_owned(), FacetValue::One(a.to_owned())))
                .into_iter()
                .collect(),
        };
        let mellow = Some("Angelina the Mellow Wish");
        let name = |e: EntitySummary, from| romanize_cn_operator(e, from).name;
        assert_eq!(
            name(summary(EntityKind::Operator, mellow), Some(Server::CN)),
            "Angelina the Mellow Wish"
        );
        assert_eq!(
            name(
                summary(EntityKind::Operator, mellow),
                Some(Server::Bilibili)
            ),
            "Angelina the Mellow Wish"
        );
        // The viewer's own server (a CN viewer), another fallback server, a
        // missing or blank appellation and a non-operator keep the name.
        for (e, from) in [
            (summary(EntityKind::Operator, mellow), None),
            (summary(EntityKind::Operator, mellow), Some(Server::JP)),
            (summary(EntityKind::Operator, None), Some(Server::CN)),
            (summary(EntityKind::Operator, Some(" ")), Some(Server::CN)),
            (summary(EntityKind::Skin, mellow), Some(Server::CN)),
        ] {
            assert_eq!(name(e, from), "予愿安洁莉娜");
        }
    }

    fn stored(kind: Option<&str>, id: Option<&str>) -> StoredCell {
        StoredCell {
            label: String::new(),
            entity_kind: kind.map(str::to_owned),
            entity_id: id.map(str::to_owned),
        }
    }

    /// Knows `amiya` (EN art), `aglna2` (CN art only) and `noart` (no art).
    fn preview_resolve(kind: EntityKind, id: &str) -> Option<(EntitySummary, Option<Server>)> {
        let (icon, server) = match id {
            "amiya" => (Some("/avatar/amiya"), None),
            "aglna2" => (Some("/avatar/aglna2"), Some(Server::CN)),
            "noart" => (None, None),
            _ => return None,
        };
        let entity = EntitySummary {
            kind,
            id: id.into(),
            name: id.into(),
            icon: icon.map(str::to_owned),
            href: None,
            facets: std::collections::BTreeMap::new(),
        };
        Some((entity, server))
    }

    #[test]
    fn the_preview_is_every_cell_in_order() {
        let cells = [
            stored(Some("operator"), Some("amiya")),
            stored(None, None),
            stored(Some("operator"), Some("aglna2")),
            stored(Some("operator"), Some("gone")),
            stored(Some("operator"), Some("noart")),
            stored(Some("not_a_kind"), Some("amiya")),
        ];
        let preview = preview_of(&cells, 6, preview_resolve);
        let icon = |icon: &str, server| {
            Some(GridPreviewIcon {
                kind: EntityKind::Operator,
                icon: icon.into(),
                server,
            })
        };
        assert_eq!(
            preview,
            vec![
                icon("/avatar/amiya", None),
                None,
                icon("/avatar/aglna2", Some(Server::CN)),
                None,
                None,
                None,
            ]
        );
    }

    #[test]
    fn the_preview_has_the_grids_shape_whatever_was_stored() {
        let filled = stored(Some("operator"), Some("amiya"));
        // A 6 x 6 grid draws 36 cells, not the first four picks.
        let full = vec![filled.clone(); 36];
        assert_eq!(preview_of(&full, 36, preview_resolve).len(), 36);
        assert!(
            preview_of(&full, 36, preview_resolve)
                .iter()
                .all(Option::is_some)
        );
        // An empty grid is all empty cells.
        let empty = vec![stored(None, None); 9];
        assert_eq!(preview_of(&empty, 9, preview_resolve), vec![None; 9]);
        // A stored list off its size is padded or cut to rows * cols.
        let short = preview_of(std::slice::from_ref(&filled), 4, preview_resolve);
        assert_eq!(short.len(), 4);
        assert!(short[0].is_some() && short[1..].iter().all(Option::is_none));
        assert_eq!(preview_of(&full, 2, preview_resolve).len(), 2);
    }

    #[test]
    fn an_unlisted_template_is_shown_to_its_owner_only() {
        let owner = Uuid::from_u128(1);
        let other = Uuid::from_u128(2);
        assert!(template_visible(Some(true), Some(owner), None));
        assert!(!template_visible(Some(false), Some(owner), None));
        assert!(!template_visible(
            Some(false),
            Some(owner),
            Some((other, GlobalRole::User))
        ));
        assert!(template_visible(
            Some(false),
            Some(owner),
            Some((owner, GlobalRole::User))
        ));
    }

    #[test]
    fn fallback_tries_cn_first_then_the_configured_order() {
        use Server::{CN, EN, JP, KR};
        assert_eq!(fallback_order(EN, &[EN, JP, CN, KR]), vec![CN, JP, KR]);
        assert_eq!(fallback_order(JP, &[EN, JP]), vec![EN]);
        assert_eq!(fallback_order(CN, &[EN, CN, KR]), vec![EN, KR]);
        assert_eq!(fallback_order(EN, &[EN]), Vec::<Server>::new());
        assert_eq!(fallback_order(EN, &[EN, CN, CN]), vec![CN]);
    }

    #[test]
    fn the_first_server_that_knows_a_pick_wins() {
        let sources = [
            (None, "en"),
            (Some(Server::CN), "cn"),
            (Some(Server::JP), "jp"),
        ];
        let knows = |set: &'static [&'static str]| {
            move |&(server, name): &(Option<Server>, &'static str)| {
                set.contains(&name).then_some((name, server))
            }
        };
        assert_eq!(
            first_resolved(sources.iter(), knows(&["en", "cn"])),
            Some(("en", None))
        );
        assert_eq!(
            first_resolved(sources.iter(), knows(&["cn", "jp"])),
            Some(("cn", Some(Server::CN)))
        );
        assert_eq!(
            first_resolved(sources.iter(), knows(&["jp"])),
            Some(("jp", Some(Server::JP)))
        );
        assert_eq!(first_resolved(sources.iter(), knows(&[])), None);
    }

    #[test]
    fn a_slug_always_has_a_readable_base() {
        assert!(slug_for("Favourite Vanguards").starts_with("favourite-vanguards-"));
        assert!(slug_for("!!!").starts_with("grid-"));
        let long = slug_for(&"a".repeat(TITLE_MAX));
        assert_eq!(long.len(), SLUG_BASE_MAX + 7);
    }
}
