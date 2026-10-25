//! The profile header's three picture sources as entity kinds: story CGs and
//! scene plates (`story_art`, keyed by the lowercase asset key the scripts
//! name) and Archives gallery pictures (`gallery`, keyed by the
//! `ActArchiveResData.Pics` key). The ids are the ones a profile background
//! stores, so a picture is the same `(kind, id)` in either place.
//!
//! Unlike every other kind these are not read off [`GameData`]: the
//! catalogues are built per server off the story index and the art tree, on
//! the CPU pool, and cached ([`story_art::story_art_shared`],
//! [`gallery::gallery_shared`]). A request takes a [`StoryCatalogues`]
//! snapshot of them once, asynchronously, with [`StoryCatalogues::gather`],
//! and the synchronous reads in [`super`] look ids up in it. The snapshot
//! holds only the catalogues the request's kinds read, so a list with no
//! picture kind never waits on (or triggers) a story index build.
//!
//! [`GameData`]: crate::core::gamedata::types::GameData

use std::sync::Arc;

use super::{EntitySummary, Facets};
use crate::app::services::story::{
    GalleryGroup, GalleryPicture, StoryArtGallery, StoryArtGroup, StoryArtKind, StoryArtPicture,
    StoryGallery, gallery, story_art,
};
use crate::app::state::AppState;
use crate::core::hypergryph::constants::Server;
use crate::database::models::tier_list::EntityKind;

/// One server's story picture catalogues, as one request reads them. A
/// catalogue the request's kinds do not read, or one that failed to build,
/// is `None` and lists nothing: an id in it neither resolves nor validates,
/// and the request goes on.
#[derive(Debug, Clone, Default)]
pub struct StoryCatalogues {
    pub cg: Option<Arc<StoryArtGallery>>,
    pub scene: Option<Arc<StoryArtGallery>>,
    pub archive: Option<Arc<StoryGallery>>,
}

impl StoryCatalogues {
    /// No catalogue at all: what a request whose kinds are all game-data
    /// kinds passes.
    pub const NONE: Self = Self {
        cg: None,
        scene: None,
        archive: None,
    };

    /// The catalogues `kinds` read, off `server` (the default server's when
    /// `server` is not loaded, as [`AppState::game_data`] falls back). Each
    /// comes from its per-server cache, built on the CPU pool on a miss, so
    /// nothing heavy runs on the calling task. A build that fails (the pool
    /// sheds, the server unloads) is logged and leaves that catalogue empty.
    pub async fn gather(state: &AppState, server: Server, kinds: &[EntityKind]) -> Self {
        let (mut cg, mut scene, mut archive) = (false, false, false);
        for kind in kinds {
            match kind {
                EntityKind::StoryCg => cg = true,
                EntityKind::StoryScene => scene = true,
                EntityKind::ArchivePic => archive = true,
                _ => {}
            }
        }
        if !(cg || scene || archive) {
            return Self::NONE;
        }
        let server = if state.try_server_data(server).is_some() {
            server
        } else {
            state.default_server
        };
        let art = |kind: StoryArtKind| async move {
            story_art::story_art_shared(state, server, kind)
                .await
                .inspect_err(|e| {
                    tracing::warn!(
                        ?server,
                        kind = kind.as_str(),
                        error = %e,
                        "story art catalogue unavailable"
                    );
                })
                .ok()
        };
        Self {
            cg: if cg {
                art(StoryArtKind::Cg).await
            } else {
                None
            },
            scene: if scene {
                art(StoryArtKind::Scene).await
            } else {
                None
            },
            archive: if archive {
                gallery::gallery_shared(state, server)
                    .await
                    .inspect_err(|e| {
                        tracing::warn!(?server, error = %e, "archive gallery unavailable");
                    })
                    .ok()
            } else {
                None
            },
        }
    }

    const fn art(&self, kind: StoryArtKind) -> Option<&Arc<StoryArtGallery>> {
        match kind {
            StoryArtKind::Cg => self.cg.as_ref(),
            StoryArtKind::Scene => self.scene.as_ref(),
        }
    }
}

/// One picture kind's three readings of a [`StoryCatalogues`], the twin of
/// [`super::KindSource`].
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
    // The archive's own order: groups by component key, pictures as shown.
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

/// The group listing `id` and its picture. Exact: ids are lowercase on the
/// wire, as [`story_art::story_art_known`] matches them.
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

/// Library order, each picture under the first group that draws it.
fn art_catalogue(story: &StoryCatalogues, kind: StoryArtKind) -> Vec<EntitySummary> {
    story.art(kind).map_or_else(Vec::new, |gallery| {
        gallery
            .groups
            .iter()
            .flat_map(|g| g.pictures.iter().map(move |p| art_summary(kind, g, p)))
            .collect()
    })
}

const fn entity_kind(kind: StoryArtKind) -> EntityKind {
    match kind {
        StoryArtKind::Cg => EntityKind::StoryCg,
        StoryArtKind::Scene => EntityKind::StoryScene,
    }
}

fn art_summary(kind: StoryArtKind, group: &StoryArtGroup, p: &StoryArtPicture) -> EntitySummary {
    EntitySummary {
        kind: entity_kind(kind),
        id: p.id.clone(),
        // A CG or scene has no title of its own; the profile picker prints
        // its story's name too.
        name: group.name.clone(),
        // The catalogue lists only pictures that resolved to a file, so
        // every one has its 320 px thumb.
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
        // The catalogue keeps a picture whose file is missing, with no url;
        // it has no thumb to serve.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::services::story::StoryCategory;
    use crate::app::services::tier_entity::{FacetValue, catalogue, known, resolve, validate};
    use crate::core::gamedata::assets::AssetIndex;
    use crate::core::gamedata::types::GameData;

    fn art(kind: StoryArtKind, ids: &[&str]) -> Arc<StoryArtGallery> {
        Arc::new(StoryArtGallery {
            kind,
            groups: vec![StoryArtGroup {
                id: "act13side".to_owned(),
                name: "Near Light".to_owned(),
                category: StoryCategory::Side,
                pictures: ids
                    .iter()
                    .map(|id| StoryArtPicture {
                        id: (*id).to_owned(),
                        width: 1600,
                        height: 900,
                    })
                    .collect(),
            }],
        })
    }

    fn archive() -> Arc<StoryGallery> {
        let pic = |id: &str, title: &str, url: Option<&str>| GalleryPicture {
            id: id.to_owned(),
            title: title.to_owned(),
            description: None,
            picture_type: "ROGUE_IMAGE".to_owned(),
            url: url.map(str::to_owned),
        };
        Arc::new(StoryGallery {
            groups: vec![GalleryGroup {
                id: "rogue_1".to_owned(),
                name: "Phantom & Crimson Solitaire".to_owned(),
                pictures: vec![
                    pic(
                        "pic_rogue_1_1",
                        "Warm Fire",
                        Some("/textures/avg/imgs/pic_rogue_1_1.png"),
                    ),
                    pic("pic_rogue_1_KV1", "", None),
                ],
            }],
        })
    }

    fn full() -> StoryCatalogues {
        StoryCatalogues {
            cg: Some(art(StoryArtKind::Cg, &["avg_1_1", "avg_1_2"])),
            scene: Some(art(StoryArtKind::Scene, &["bg_corridor"])),
            archive: Some(archive()),
        }
    }

    fn one<'a>(s: &'a EntitySummary, key: &str) -> Option<&'a str> {
        match s.facets.get(key)? {
            FacetValue::One(v) => Some(v),
            FacetValue::Many(_) => None,
        }
    }

    #[test]
    fn a_story_cg_is_its_story_with_the_thumb_route() {
        let (gd, assets, story) = (GameData::new(), AssetIndex::default(), full());
        let s = resolve(&gd, &assets, &story, EntityKind::StoryCg, "avg_1_2").expect("listed");
        assert_eq!(s.kind, EntityKind::StoryCg);
        assert_eq!(s.name, "Near Light");
        assert_eq!(
            s.icon.as_deref(),
            Some("/story/art-gallery/cg/avg_1_2/thumb")
        );
        assert_eq!(one(&s, "group"), Some("Near Light"));
        assert_eq!(one(&s, "group_id"), Some("act13side"));
        assert_eq!(one(&s, "category"), Some("side"));
        let ids: Vec<String> = catalogue(&gd, &assets, &story, EntityKind::StoryCg)
            .into_iter()
            .map(|s| s.id)
            .collect();
        assert_eq!(ids, ["avg_1_1", "avg_1_2"]);
    }

    #[test]
    fn cg_and_scene_ids_are_separate_spaces() {
        let (gd, assets, story) = (GameData::new(), AssetIndex::default(), full());
        assert!(known(
            &gd,
            &assets,
            &story,
            EntityKind::StoryScene,
            "bg_corridor"
        ));
        assert!(!known(
            &gd,
            &assets,
            &story,
            EntityKind::StoryCg,
            "bg_corridor"
        ));
        assert!(!known(
            &gd,
            &assets,
            &story,
            EntityKind::StoryScene,
            "avg_1_1"
        ));
        let s =
            resolve(&gd, &assets, &story, EntityKind::StoryScene, "bg_corridor").expect("listed");
        assert_eq!(
            s.icon.as_deref(),
            Some("/story/art-gallery/scene/bg_corridor/thumb")
        );
        // Exact: the wire ids are lowercase.
        assert!(!known(&gd, &assets, &story, EntityKind::StoryCg, "AVG_1_1"));
    }

    #[test]
    fn an_archive_picture_carries_its_group_and_type() {
        let (gd, assets, story) = (GameData::new(), AssetIndex::default(), full());
        let s = resolve(
            &gd,
            &assets,
            &story,
            EntityKind::ArchivePic,
            "pic_rogue_1_1",
        )
        .expect("listed");
        assert_eq!(s.name, "Warm Fire");
        assert_eq!(
            s.icon.as_deref(),
            Some("/story/gallery/pic_rogue_1_1/thumb")
        );
        assert_eq!(one(&s, "group"), Some("Phantom & Crimson Solitaire"));
        assert_eq!(one(&s, "group_id"), Some("rogue_1"));
        assert_eq!(one(&s, "picture_type"), Some("ROGUE_IMAGE"));
        // No file: still listed, named by its id when untitled, no icon.
        let bare = resolve(
            &gd,
            &assets,
            &story,
            EntityKind::ArchivePic,
            "pic_rogue_1_KV1",
        )
        .expect("listed");
        assert_eq!(bare.name, "pic_rogue_1_KV1");
        assert_eq!(bare.icon, None);
    }

    #[test]
    fn a_missing_catalogue_lists_nothing_and_does_not_validate() {
        let (gd, assets) = (GameData::new(), AssetIndex::default());
        let none = StoryCatalogues::NONE;
        for kind in [
            EntityKind::StoryCg,
            EntityKind::StoryScene,
            EntityKind::ArchivePic,
        ] {
            assert!(catalogue(&gd, &assets, &none, kind).is_empty());
        }
        assert!(resolve(&gd, &assets, &none, EntityKind::StoryCg, "avg_1_1").is_none());
        assert!(validate([(&gd, &assets, &none)], EntityKind::StoryCg, "avg_1_1").is_err());
        // Another server whose snapshot has it is enough.
        let story = full();
        assert!(
            validate(
                [(&gd, &assets, &none), (&gd, &assets, &story)],
                EntityKind::StoryCg,
                "avg_1_1"
            )
            .is_ok()
        );
    }
}
