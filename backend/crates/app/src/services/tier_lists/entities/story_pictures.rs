use std::sync::Arc;

use myrtle_engine_story::dto::{
    GalleryGroup, GalleryPicture, StoryArtGallery, StoryArtGroup, StoryArtKind, StoryArtPicture,
    StoryGallery,
};
use myrtle_types::{EntityKind, Server};

use super::{EntitySummary, Facets};
use crate::AppContext;
use crate::services::story::{self, StoryArt};

#[derive(Clone)]
pub enum ArtRef {
    Shared(Arc<StoryArt>),
    Own(Arc<StoryArtGallery>),
}

impl ArtRef {
    fn gallery(&self, kind: StoryArtKind) -> &StoryArtGallery {
        match self {
            Self::Shared(art) => art.gallery(kind),
            Self::Own(gallery) => gallery,
        }
    }
}

#[derive(Clone, Default)]
pub struct StoryCatalogues {
    pub cg: Option<ArtRef>,
    pub scene: Option<ArtRef>,
    pub archive: Option<Arc<StoryGallery>>,
}

impl StoryCatalogues {
    pub const NONE: Self = Self {
        cg: None,
        scene: None,
        archive: None,
    };

    pub async fn gather(ctx: &AppContext, server: Server, kinds: &[EntityKind]) -> Self {
        let wants = |k: EntityKind| kinds.contains(&k);
        let (cg, scene, archive) = (
            wants(EntityKind::StoryCg),
            wants(EntityKind::StoryScene),
            wants(EntityKind::ArchivePic),
        );
        if !(cg || scene || archive) {
            return Self::NONE;
        }
        let server = if ctx.snapshots.get(server).is_some() {
            server
        } else {
            ctx.snapshots.default_server()
        };
        let art = if cg || scene {
            story::story_art_shared(ctx, server)
                .await
                .inspect_err(
                    |e| tracing::warn!(?server, error = %e, "story art catalogue unavailable"),
                )
                .ok()
                .map(ArtRef::Shared)
        } else {
            None
        };
        Self {
            cg: art.clone().filter(|_| cg),
            scene: art.filter(|_| scene),
            archive: if archive {
                story::gallery_shared(ctx, server)
                    .await
                    .inspect_err(
                        |e| tracing::warn!(?server, error = %e, "archive gallery unavailable"),
                    )
                    .ok()
            } else {
                None
            },
        }
    }

    fn art(&self, kind: StoryArtKind) -> Option<&StoryArtGallery> {
        match kind {
            StoryArtKind::Cg => self.cg.as_ref(),
            StoryArtKind::Scene => self.scene.as_ref(),
        }
        .map(|art| art.gallery(kind))
    }
}

pub(super) struct StorySource {
    pub resolve: fn(&StoryCatalogues, &str) -> Option<EntitySummary>,
    pub known: fn(&StoryCatalogues, &str) -> bool,
    pub catalogue: fn(&StoryCatalogues) -> Vec<EntitySummary>,
}

pub(super) const STORY_CG: StorySource = StorySource {
    resolve: |story, id| art_resolve(story, StoryArtKind::Cg, id),
    known: |story, id| find_art(story, StoryArtKind::Cg, id).is_some(),
    catalogue: |story| art_catalogue(story, StoryArtKind::Cg),
};

pub(super) const STORY_SCENE: StorySource = StorySource {
    resolve: |story, id| art_resolve(story, StoryArtKind::Scene, id),
    known: |story, id| find_art(story, StoryArtKind::Scene, id).is_some(),
    catalogue: |story| art_catalogue(story, StoryArtKind::Scene),
};

pub(super) const ARCHIVE_PIC: StorySource = StorySource {
    resolve: |story, id| find_archive(story, id).map(|(group, p)| archive_summary(group, p)),
    known: |story, id| find_archive(story, id).is_some(),
    catalogue: |story| {
        story.archive.as_ref().map_or_else(Vec::new, |gallery| {
            gallery
                .groups
                .iter()
                .flat_map(|g| g.pictures.iter().map(move |p| archive_summary(g, p)))
                .collect()
        })
    },
};

fn find_art<'a>(
    story: &'a StoryCatalogues,
    kind: StoryArtKind,
    id: &str,
) -> Option<(&'a StoryArtGroup, &'a StoryArtPicture)> {
    story
        .art(kind)?
        .groups
        .iter()
        .find_map(|g| g.pictures.iter().find(|p| p.id == id).map(|p| (g, p)))
}

fn find_archive<'a>(
    story: &'a StoryCatalogues,
    id: &str,
) -> Option<(&'a GalleryGroup, &'a GalleryPicture)> {
    story
        .archive
        .as_ref()?
        .groups
        .iter()
        .find_map(|g| g.pictures.iter().find(|p| p.id == id).map(|p| (g, p)))
}

fn art_resolve(story: &StoryCatalogues, kind: StoryArtKind, id: &str) -> Option<EntitySummary> {
    find_art(story, kind, id).map(|(group, p)| art_summary(kind, group, p))
}

fn art_catalogue(story: &StoryCatalogues, kind: StoryArtKind) -> Vec<EntitySummary> {
    story.art(kind).map_or_else(Vec::new, |gallery| {
        gallery
            .groups
            .iter()
            .flat_map(|g| g.pictures.iter().map(move |p| art_summary(kind, g, p)))
            .collect()
    })
}

fn art_summary(kind: StoryArtKind, group: &StoryArtGroup, p: &StoryArtPicture) -> EntitySummary {
    EntitySummary {
        kind: match kind {
            StoryArtKind::Cg => EntityKind::StoryCg,
            StoryArtKind::Scene => EntityKind::StoryScene,
        },
        id: p.id.clone(),
        name: group.name.clone(),
        icon: Some(format!(
            "/story/art-gallery/{}/{}/thumb",
            kind.as_str(),
            urlencoding::encode(&p.id)
        )),
        href: None,
        facets: Facets::default()
            .one("group", group.name.clone())
            .one("group_id", group.id.clone())
            .one("category", group.category.as_str())
            .into(),
    }
}

fn archive_summary(group: &GalleryGroup, p: &GalleryPicture) -> EntitySummary {
    EntitySummary {
        kind: EntityKind::ArchivePic,
        id: p.id.clone(),
        name: if p.title.is_empty() {
            p.id.clone()
        } else {
            p.title.clone()
        },
        icon: p
            .url
            .is_some()
            .then(|| format!("/story/gallery/{}/thumb", urlencoding::encode(&p.id))),
        href: None,
        facets: Facets::default()
            .one("group", group.name.clone())
            .one("group_id", group.id.clone())
            .one("picture_type", p.picture_type.clone())
            .into(),
    }
}
