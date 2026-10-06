//! The profile Showcase: checking a saved showcase's referents, and serving it.
//!
//! A block stores only a pointer (entity ids, a grid or tier list slug, a plan
//! id). A save is refused when a pointer names nothing. What it names can still
//! go later (a grid deleted, a plan taken off the profile, an id the game data
//! drops), so every read checks again: a visitor never sees a block whose
//! referent is gone, and the owner sees it marked `removed` so they can fix it.
//!
//! The read check is three indexed lookups at most (grid slugs, tier list
//! slugs, plan ids, each one `= ANY` query, skipped when no block needs it)
//! plus in-memory game-data lookups. It runs on every visitor's `/get-user`,
//! after the cache, so the tab a visitor lands on never opens onto blocks that
//! all vanished since the profile row was cached.

use std::collections::HashSet;
use std::sync::atomic::Ordering;

use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::services::grid::Resolver;
use crate::app::services::story::gallery;
use crate::app::services::story::{StoryArtKind, story_art};
use crate::app::services::tier_entity::{self, EntitySummary};
use crate::app::services::tier_list::validate_entity;
use crate::app::state::{AppState, ServerData};
use crate::core::hypergryph::constants::Server;
use crate::database::models::profile_layout::{
    ProfileBackground, ProfileBackgroundKind, ProfileLayout, ProfileShowcase, ShowcaseBlock,
};
use crate::database::models::tier_list::EntityKind;

/// The showcase as `/get-user-showcase` serves it.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShowcaseView {
    pub blocks: Vec<ShowcaseBlockView>,
}

/// One block, with what it points at checked and, for favourites, resolved.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShowcaseBlockView {
    pub block: ShowcaseBlock,
    /// What the block points at is gone: a deleted grid or tier list, a plan
    /// deleted or no longer shown on the profile, or favourites none of whose
    /// ids the game data knows. Only the owner is ever sent such a block.
    pub removed: bool,
    /// Favourites only, one per id in `block`, in its order; empty for every
    /// other block. A visitor is sent only the ids that resolve.
    pub entities: Vec<ShowcaseEntity>,
}

/// One favourite, resolved against the reader's server first and then every
/// other loaded server, as a grid cell is.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShowcaseEntity {
    pub id: String,
    /// `None` when no loaded server's game data has the id.
    pub entity: Option<EntitySummary>,
    /// The server `entity` resolved on when the reader's does not know it,
    /// with `GridCell::entity_server`'s meaning.
    #[ts(type = "string | null")]
    #[schema(value_type = Option<String>)]
    pub entity_server: Option<Server>,
}

/// Which grids, tier lists and plans named by `blocks` exist: a live grid, an
/// active tier list, one of `owner`'s plans with `display_on_profile` set.
#[derive(Default)]
struct Existing {
    grids: HashSet<String>,
    tier_lists: HashSet<String>,
    plans: HashSet<Uuid>,
}

impl Existing {
    async fn load(
        state: &AppState,
        owner: Uuid,
        blocks: &[ShowcaseBlock],
    ) -> Result<Self, ApiError> {
        let mut grids: Vec<String> = Vec::new();
        let mut tier_lists: Vec<String> = Vec::new();
        let mut plans: Vec<Uuid> = Vec::new();
        for block in blocks {
            match block {
                ShowcaseBlock::Grid { slug } => grids.push(slug.clone()),
                ShowcaseBlock::TierList { slug } => tier_lists.push(slug.clone()),
                ShowcaseBlock::Plan { id } => plans.push(*id),
                ShowcaseBlock::Favourites { .. } => {}
            }
        }
        let mut out = Self::default();
        if !grids.is_empty() {
            out.grids =
                sqlx::query_scalar::<_, String>("SELECT slug FROM grids WHERE slug = ANY($1)")
                    .bind(&grids)
                    .fetch_all(&state.db)
                    .await?
                    .into_iter()
                    .collect();
        }
        if !tier_lists.is_empty() {
            out.tier_lists = sqlx::query_scalar::<_, String>(
                "SELECT slug FROM tier_lists WHERE slug = ANY($1) AND is_active",
            )
            .bind(&tier_lists)
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .collect();
        }
        if !plans.is_empty() {
            out.plans = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM operator_plans WHERE id = ANY($1) AND user_id = $2 AND display_on_profile",
            )
            .bind(&plans)
            .bind(owner)
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .collect();
        }
        Ok(out)
    }

    /// Whether a non-favourites block's referent exists. Favourites are
    /// checked id by id against the game data instead.
    fn has(&self, block: &ShowcaseBlock) -> bool {
        match block {
            ShowcaseBlock::Grid { slug } => self.grids.contains(slug),
            ShowcaseBlock::TierList { slug } => self.tier_lists.contains(slug),
            ShowcaseBlock::Plan { id } => self.plans.contains(id),
            ShowcaseBlock::Favourites { .. } => true,
        }
    }
}

/// Refuse a showcase that points at anything missing: an entity id no loaded
/// server knows for its kind, a grid or tier list slug with no live list
/// behind it, a plan that is not one of `owner`'s shown on their profile.
///
/// # Errors
/// `400` naming the first broken block, counted from 1.
pub async fn validate(
    state: &AppState,
    owner: Uuid,
    showcase: &ProfileShowcase,
) -> Result<(), ApiError> {
    let existing = Existing::load(state, owner, &showcase.blocks).await?;
    for (i, block) in showcase.blocks.iter().enumerate() {
        let n = i + 1;
        match block {
            ShowcaseBlock::Favourites {
                entity_kind, ids, ..
            } => {
                let context = format!("showcase block {n}");
                for id in ids {
                    validate_entity(state, *entity_kind, id).map_err(|e| prefixed(&context, e))?;
                }
            }
            _ if existing.has(block) => {}
            ShowcaseBlock::Grid { slug } => {
                return Err(ApiError::BadRequest(format!(
                    "showcase block {n}: no grid `{slug}`"
                )));
            }
            ShowcaseBlock::TierList { slug } => {
                return Err(ApiError::BadRequest(format!(
                    "showcase block {n}: no tier list `{slug}`"
                )));
            }
            ShowcaseBlock::Plan { id } => {
                return Err(ApiError::BadRequest(format!(
                    "showcase block {n}: plan `{id}` is not one of your plans shown on your profile"
                )));
            }
        }
    }
    Ok(())
}

/// `e` with `context` ahead of its message when it is a 400; any other error
/// as it is.
fn prefixed(context: &str, e: ApiError) -> ApiError {
    match e {
        ApiError::BadRequest(m) => ApiError::BadRequest(format!("{context}: {m}")),
        other => other,
    }
}

/// Refuse a background whose id no loaded server knows for its kind: the
/// outfit or operator lookup a favourites block of that kind goes through,
/// the gallery table for a gallery picture, the story art catalogue for a
/// story CG or scene. A skin id without `@` is refused too: `#1` and `#2` are an operator's
/// default and elite 2 art, which `skin_table` lists but the outfit pool
/// (`tier_entity::skin::is_outfit`) does not offer and the client draws no
/// skin art for; the operator kind is the way to pick that art. An operator
/// background that asks for elite 2 art is refused when the operator has no
/// elite 2.
///
/// # Errors
/// `400` naming the background.
pub async fn validate_background(
    state: &AppState,
    background: &ProfileBackground,
) -> Result<(), ApiError> {
    if background.kind == ProfileBackgroundKind::Skin && !background.id.contains('@') {
        return Err(ApiError::BadRequest(format!(
            "background: `{}` is not an outfit",
            background.id
        )));
    }
    if let Some(kind) = background.kind.entity_kind() {
        validate_entity(state, kind, &background.id).map_err(|e| prefixed("background", e))?;
        if background.elite == Some(2) && !has_elite_2(state, &background.id) {
            return Err(ApiError::BadRequest(format!(
                "background: `{}` has no elite 2 art",
                background.id
            )));
        }
        return Ok(());
    }
    if background_known(state, background).await {
        return Ok(());
    }
    let noun = match background.kind {
        ProfileBackgroundKind::StoryCg => "story CG",
        ProfileBackgroundKind::StoryScene => "story scene",
        // Skins and operators returned above.
        _ => "gallery picture",
    };
    Err(ApiError::BadRequest(format!(
        "background: `{}` is not a {noun}",
        background.id
    )))
}

/// Whether any loaded server's game data knows the background's id for its
/// kind: the entity lookup for a skin or operator, the gallery table for a
/// gallery picture, the story art catalogue for a story CG or scene.
async fn background_known(state: &AppState, background: &ProfileBackground) -> bool {
    let id = background.id.as_str();
    match background.kind {
        ProfileBackgroundKind::Skin | ProfileBackgroundKind::Operator => background
            .kind
            .entity_kind()
            .is_some_and(|kind| entity_known(state, kind, id)),
        ProfileBackgroundKind::ArchivePic => {
            any_loaded(state, |sd| gallery::known(&sd.game_data.load(), id))
        }
        ProfileBackgroundKind::StoryCg => {
            story_art::story_art_known(state, StoryArtKind::Cg, id).await
        }
        ProfileBackgroundKind::StoryScene => {
            story_art::story_art_known(state, StoryArtKind::Scene, id).await
        }
    }
}

/// Whether any loaded server lists elite 2 for operator `id`: a third phase in
/// `character_table`, the phase whose art is `<id>_2.png`. An operator that
/// stops at elite 1 (three stars and lower) has no such art to draw.
fn has_elite_2(state: &AppState, id: &str) -> bool {
    any_loaded(state, |sd| {
        sd.game_data
            .load()
            .operators
            .get(id)
            .is_some_and(|op| op.phases.len() >= 3)
    })
}

/// Whether `pred` holds for any loaded server.
fn any_loaded(state: &AppState, pred: impl Fn(&ServerData) -> bool) -> bool {
    state
        .servers
        .values()
        .filter(|sd| sd.loaded.load(Ordering::Acquire))
        .any(|sd| pred(sd))
}

/// Whether any loaded server's game data knows `(kind, id)`, the read-side
/// twin of [`validate_entity`].
fn entity_known(state: &AppState, kind: EntityKind, id: &str) -> bool {
    any_loaded(state, |sd| {
        tier_entity::known(&sd.game_data.load(), &sd.asset_index.load(), kind, id)
    })
}

/// A visitor's layout with every block whose referent is gone taken out, and
/// every unknown id taken out of a favourites block (the block too, when none
/// is left). A background whose id the game data no longer knows goes too,
/// so the header falls back to its plain look rather than a missing image.
/// Run it on the [`ProfileLayout::visible_only`] projection.
///
/// # Errors
/// A database error from the existence lookups.
pub async fn prune_for_visitor(
    state: &AppState,
    owner: Uuid,
    layout: &mut ProfileLayout,
) -> Result<(), ApiError> {
    if let Some(bg) = layout.background.as_ref()
        && !background_known(state, bg).await
    {
        layout.background = None;
    }
    let Some(showcase) = layout.showcase.as_mut() else {
        return Ok(());
    };
    if showcase.blocks.is_empty() {
        return Ok(());
    }
    let existing = Existing::load(state, owner, &showcase.blocks).await?;
    showcase.blocks.retain_mut(|block| match block {
        ShowcaseBlock::Favourites {
            entity_kind, ids, ..
        } => {
            let kind = *entity_kind;
            ids.retain(|id| entity_known(state, kind, id));
            !ids.is_empty()
        }
        other => existing.has(other),
    });
    Ok(())
}

/// The showcase of `layout` for one reader, favourites resolved against
/// `server`. The owner (`own`) gets every block, the broken ones `removed`; a
/// visitor gets the [`ProfileLayout::visible_only`] blocks that still resolve.
/// A NULL layout, or one with no showcase, is an empty showcase.
///
/// # Errors
/// A database error from the existence lookups.
pub async fn view(
    state: &AppState,
    owner: Uuid,
    layout: Option<&ProfileLayout>,
    own: bool,
    server: Server,
) -> Result<ShowcaseView, ApiError> {
    let projected = layout.map(|l| if own { l.clone() } else { l.visible_only() });
    let blocks = projected
        .as_ref()
        .map_or(&[][..], ProfileLayout::showcase_blocks);
    if blocks.is_empty() {
        return Ok(ShowcaseView { blocks: Vec::new() });
    }
    let existing = Existing::load(state, owner, blocks).await?;
    let resolver = Resolver::new(state, server);
    let mut out = Vec::with_capacity(blocks.len());
    for block in blocks {
        let view = match block {
            ShowcaseBlock::Favourites {
                entity_kind,
                ids,
                title,
            } => {
                let mut entities: Vec<ShowcaseEntity> = ids
                    .iter()
                    .map(|id| {
                        let resolved = resolver.resolve(*entity_kind, id);
                        ShowcaseEntity {
                            id: id.clone(),
                            entity_server: resolved.as_ref().and_then(|(_, s)| *s),
                            entity: resolved.map(|(e, _)| e),
                        }
                    })
                    .collect();
                if !own {
                    entities.retain(|e| e.entity.is_some());
                }
                let removed = entities.iter().all(|e| e.entity.is_none());
                ShowcaseBlockView {
                    block: ShowcaseBlock::Favourites {
                        entity_kind: *entity_kind,
                        ids: entities.iter().map(|e| e.id.clone()).collect(),
                        title: title.clone(),
                    },
                    removed,
                    entities,
                }
            }
            other => ShowcaseBlockView {
                block: other.clone(),
                removed: !existing.has(other),
                entities: Vec::new(),
            },
        };
        if own || !view.removed {
            out.push(view);
        }
    }
    Ok(ShowcaseView { blocks: out })
}
