use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use myrtle_db::repos::{grids, planner, tier_lists};
use myrtle_gamedata::model::GameModel;
use myrtle_types::{EntityKind, Server, UserId};
use serde::Serialize;
use uuid::Uuid;

use super::super::tier_lists::entities::{self, EntitySummary, StoryCatalogues};
use super::grids::Resolver;
use crate::policy::Subject;
use crate::privacy::layout::{
    ProfileBackground, ProfileBackgroundKind, ProfileLayout, ProfileShowcase, ProfileTabId,
    ShowcaseBlock,
};
use crate::privacy::{self, UpdateSettingsRequest};
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ShowcaseView {
    pub blocks: Vec<ShowcaseBlockView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ShowcaseBlockView {
    pub block: ShowcaseBlock,
    pub removed: bool,
    pub entities: Vec<ShowcaseEntity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ShowcaseEntity {
    pub id: String,
    pub entity: Option<EntitySummary>,
    #[ts(type = "string | null")]
    pub entity_server: Option<Server>,
}

#[derive(Default)]
struct Existing {
    grids: HashSet<String>,
    tier_lists: HashSet<String>,
    plans: HashSet<Uuid>,
    plan_by_operator: HashMap<String, Uuid>,
}

impl Existing {
    async fn load(
        ctx: &AppContext,
        owner: UserId,
        blocks: &[ShowcaseBlock],
    ) -> Result<Self, AppError> {
        let mut grid_slugs = Vec::new();
        let mut list_slugs = Vec::new();
        let mut plan_ids = Vec::new();
        let mut plan_operators = Vec::new();
        for block in blocks {
            match block {
                ShowcaseBlock::Grid { slug } => grid_slugs.push(slug.clone()),
                ShowcaseBlock::TierList { slug } => list_slugs.push(slug.clone()),
                ShowcaseBlock::Plan { id, operator_id } => {
                    plan_ids.push(*id);
                    plan_operators.extend(operator_id.iter().cloned());
                }
                ShowcaseBlock::Favourites { .. } => {}
            }
        }
        let mut out = Self::default();
        if grid_slugs.is_empty() && list_slugs.is_empty() && plan_ids.is_empty() {
            return Ok(out);
        }
        let mut conn = ctx.db.conn().await?;
        if !grid_slugs.is_empty() {
            out.grids = grids::existing_slugs(&mut conn, &grid_slugs)
                .await?
                .into_iter()
                .collect();
        }
        out.tier_lists = tier_lists::active_slugs(&mut conn, &list_slugs)
            .await?
            .into_iter()
            .collect();
        if !plan_ids.is_empty() {
            for row in planner::shown_plans(&mut conn, owner, &plan_ids, &plan_operators).await? {
                out.plans.insert(row.id);
                out.plan_by_operator.insert(row.operator_id, row.id);
            }
        }
        Ok(out)
    }

    fn has(&self, block: &ShowcaseBlock) -> bool {
        match block {
            ShowcaseBlock::Grid { slug } => self.grids.contains(slug),
            ShowcaseBlock::TierList { slug } => self.tier_lists.contains(slug),
            ShowcaseBlock::Plan { id, operator_id } => {
                self.live_plan_id(*id, operator_id.as_ref()).is_some()
            }
            ShowcaseBlock::Favourites { .. } => true,
        }
    }

    fn live_plan_id(&self, id: Uuid, operator_id: Option<&String>) -> Option<Uuid> {
        if self.plans.contains(&id) {
            return Some(id);
        }
        operator_id
            .and_then(|op| self.plan_by_operator.get(op))
            .copied()
    }

    fn resolve(&self, block: &ShowcaseBlock) -> ShowcaseBlock {
        if let ShowcaseBlock::Plan { id, operator_id } = block
            && let Some(live) = self.live_plan_id(*id, operator_id.as_ref())
        {
            return ShowcaseBlock::Plan {
                id: live,
                operator_id: operator_id.clone(),
            };
        }
        block.clone()
    }
}

pub async fn validate(
    ctx: &AppContext,
    owner: UserId,
    showcase: &ProfileShowcase,
) -> Result<(), AppError> {
    let existing = Existing::load(ctx, owner, &showcase.blocks).await?;
    let loaded = entities::loaded_sources(ctx, &favourite_kinds(&showcase.blocks)).await;
    for (i, block) in showcase.blocks.iter().enumerate() {
        let n = i + 1;
        match block {
            ShowcaseBlock::Favourites {
                entity_kind, ids, ..
            } => {
                for id in ids {
                    entities::validate(sources(&loaded), *entity_kind, id)
                        .map_err(|e| prefixed(&format!("showcase block {n}"), e))?;
                }
            }
            _ if existing.has(block) => {}
            ShowcaseBlock::Grid { slug } => {
                return Err(AppError::BadRequest(format!(
                    "showcase block {n}: no grid `{slug}`"
                )));
            }
            ShowcaseBlock::TierList { slug } => {
                return Err(AppError::BadRequest(format!(
                    "showcase block {n}: no tier list `{slug}`"
                )));
            }
            ShowcaseBlock::Plan { id, .. } => {
                return Err(AppError::BadRequest(format!(
                    "showcase block {n}: plan `{id}` is not one of your plans shown on your profile"
                )));
            }
        }
    }
    Ok(())
}

fn prefixed(context: &str, e: AppError) -> AppError {
    match e {
        AppError::BadRequest(m) => AppError::BadRequest(format!("{context}: {m}")),
        other => other,
    }
}

pub async fn validate_background(
    ctx: &AppContext,
    background: &ProfileBackground,
) -> Result<(), AppError> {
    let id = &background.id;
    if background.kind == ProfileBackgroundKind::Skin && !id.contains('@') {
        return Err(AppError::BadRequest(format!(
            "background: `{id}` is not an outfit"
        )));
    }
    if let Some(kind) = background.kind.entity_kind() {
        entities::validate_loaded(ctx, kind, id)
            .await
            .map_err(|e| prefixed("background", e))?;
        let has_elite_2 = loaded_models(ctx).iter().any(|s| {
            s.model
                .operators
                .get(id)
                .is_some_and(|op| op.phases.len() >= 3)
        });
        if background.elite == Some(2) && !has_elite_2 {
            return Err(AppError::BadRequest(format!(
                "background: `{id}` has no elite 2 art"
            )));
        }
        return Ok(());
    }
    if background_known(ctx, background).await {
        return Ok(());
    }
    let noun = match background.kind {
        ProfileBackgroundKind::StoryCg => "story CG",
        ProfileBackgroundKind::StoryScene => "story scene",
        _ => "gallery picture",
    };
    Err(AppError::BadRequest(format!(
        "background: `{id}` is not a {noun}"
    )))
}

fn loaded_models(ctx: &AppContext) -> Vec<Arc<Snapshot>> {
    ctx.snapshots
        .loaded()
        .into_iter()
        .filter_map(|s| ctx.snapshots.get(s))
        .collect()
}

async fn background_known(ctx: &AppContext, background: &ProfileBackground) -> bool {
    let id = background.id.as_str();
    let kind = match background.kind {
        ProfileBackgroundKind::ArchivePic => {
            return loaded_models(ctx)
                .iter()
                .any(|s| myrtle_engine_story::gallery::known(&s.model, id));
        }
        ProfileBackgroundKind::StoryCg => EntityKind::StoryCg,
        ProfileBackgroundKind::StoryScene => EntityKind::StoryScene,
        ProfileBackgroundKind::Skin => EntityKind::Skin,
        ProfileBackgroundKind::Operator => EntityKind::Operator,
    };
    let loaded = entities::loaded_sources(ctx, &[kind]).await;
    entities::validate(sources(&loaded), kind, id).is_ok()
}

fn sources(
    loaded: &[(Arc<Snapshot>, StoryCatalogues)],
) -> impl Iterator<Item = (&GameModel, &StoryCatalogues)> {
    loaded.iter().map(|(s, story)| (&s.model, story))
}

fn favourite_kinds(blocks: &[ShowcaseBlock]) -> Vec<EntityKind> {
    blocks
        .iter()
        .filter_map(|block| match block {
            ShowcaseBlock::Favourites { entity_kind, .. } => Some(*entity_kind),
            _ => None,
        })
        .collect()
}

pub async fn save_profile(
    ctx: &AppContext,
    subject: &Subject,
    request: &UpdateSettingsRequest,
) -> Result<(), AppError> {
    let owner = subject.user_id().ok_or(AppError::Unauthorized)?;
    if let Some(Some(patch)) = &request.profile_layout {
        if let Some(showcase) = &patch.showcase {
            validate(ctx, owner, showcase).await?;
        }
        if let Some(Some(background)) = &patch.background {
            validate_background(ctx, background).await?;
        }
    }
    privacy::update_settings(ctx, subject, request).await
}

pub async fn prune_for_visitor(
    ctx: &AppContext,
    owner: UserId,
    layout: &mut ProfileLayout,
) -> Result<(), AppError> {
    if let Some(bg) = &layout.background
        && !background_known(ctx, bg).await
    {
        layout.background = None;
    }
    let Some(showcase) = layout.showcase.as_mut() else {
        return Ok(());
    };
    if showcase.blocks.is_empty() {
        return Ok(());
    }
    let existing = Existing::load(ctx, owner, &showcase.blocks).await?;
    let loaded = entities::loaded_sources(ctx, &favourite_kinds(&showcase.blocks)).await;
    showcase.blocks.retain_mut(|block| match block {
        ShowcaseBlock::Favourites {
            entity_kind, ids, ..
        } => {
            let kind = *entity_kind;
            ids.retain(|id| sources(&loaded).any(|(m, story)| entities::known(m, story, kind, id)));
            !ids.is_empty()
        }
        other => existing.has(other),
    });
    Ok(())
}

pub async fn view(
    ctx: &AppContext,
    owner: UserId,
    layout: Option<&ProfileLayout>,
    own: bool,
    server: Server,
) -> Result<ShowcaseView, AppError> {
    let projected = layout.map(|l| if own { l.clone() } else { l.visible_only() });
    let blocks = projected
        .as_ref()
        .map_or(&[][..], ProfileLayout::showcase_blocks);
    if blocks.is_empty() {
        return Ok(ShowcaseView { blocks: Vec::new() });
    }
    let existing = Existing::load(ctx, owner, blocks).await?;
    let resolver = Resolver::new(ctx, server, &favourite_kinds(blocks)).await;
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
                        let (entity, entity_server) = resolver.resolve(*entity_kind, id).unzip();
                        ShowcaseEntity {
                            id: id.clone(),
                            entity,
                            entity_server: entity_server.flatten(),
                        }
                    })
                    .collect();
                if !own {
                    entities.retain(|e| e.entity.is_some());
                }
                ShowcaseBlockView {
                    block: ShowcaseBlock::Favourites {
                        entity_kind: *entity_kind,
                        ids: entities.iter().map(|e| e.id.clone()).collect(),
                        title: title.clone(),
                    },
                    removed: entities.iter().all(|e| e.entity.is_none()),
                    entities,
                }
            }
            other => {
                let block = existing.resolve(other);
                ShowcaseBlockView {
                    removed: !existing.has(&block),
                    block,
                    entities: Vec::new(),
                }
            }
        };
        if own || !view.removed {
            out.push(view);
        }
    }
    Ok(ShowcaseView { blocks: out })
}

pub async fn user_showcase(
    ctx: &AppContext,
    subject: &Subject,
    uid: &str,
    server: Option<Server>,
) -> Result<ShowcaseView, AppError> {
    let (profile, own, layout) =
        privacy::resolve_profile_for_tabs(ctx, subject, uid, &[ProfileTabId::Showcase]).await?;
    let server = server.unwrap_or_else(|| ctx.snapshots.default_server());
    view(ctx, UserId(profile.id), layout.as_ref(), own, server).await
}
