//! What a placement's `(kind, id)` points at in one server's game data, read at serve time.

mod background;
mod class;
mod enemy;
mod event;
mod faction;
mod integrated_strategies;
mod main_story;
mod module;
mod operator;
mod skill;
mod skin;
mod story_pictures;
mod story_sprite;
mod stronghold_bond;

use std::collections::BTreeMap;
use std::sync::Arc;

use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::operator::{Operator, OperatorRarity};
use myrtle_types::{EntityKind, Server};
use serde::{Deserialize, Serialize};

use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

pub use story_pictures::{ArtRef, StoryCatalogues};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FacetValue {
    One(String),
    Many(Vec<String>),
}

/// Enough of one entity for a tile, a pool filter and a link
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySummary {
    pub kind: EntityKind,
    pub id: String,
    pub name: String,
    /// Path under the API root; `None` when the extract has no art
    pub icon: Option<String>,
    pub href: Option<String>,
    pub facets: BTreeMap<String, FacetValue>,
}

struct KindSource {
    resolve: fn(&GameModel, &str) -> Option<EntitySummary>,
    known: fn(&GameModel, &str) -> bool,
    /// What an editor may place, in pool order
    catalogue: fn(&GameModel) -> Vec<EntitySummary>,
}

/// Picture kinds read the request's [`StoryCatalogues`], every other kind the game model
enum Source {
    Data(&'static KindSource),
    Story(&'static story_pictures::StorySource),
}

const fn source(kind: EntityKind) -> Source {
    Source::Data(match kind {
        EntityKind::Operator => &operator::SOURCE,
        EntityKind::Class => &class::CLASS,
        EntityKind::Subclass => &class::SUBCLASS,
        EntityKind::Enemy => &enemy::SOURCE,
        EntityKind::Event => &event::SOURCE,
        EntityKind::Faction => &faction::SOURCE,
        EntityKind::StrongholdBond => &stronghold_bond::SOURCE,
        EntityKind::Skin => &skin::SOURCE,
        EntityKind::Module => &module::SOURCE,
        EntityKind::Skill => &skill::SOURCE,
        EntityKind::IntegratedStrategies => &integrated_strategies::SOURCE,
        EntityKind::StorySprite => &story_sprite::SOURCE,
        EntityKind::MainStory => &main_story::SOURCE,
        EntityKind::Background => &background::SOURCE,
        EntityKind::StoryCg => return Source::Story(&story_pictures::STORY_CG),
        EntityKind::StoryScene => return Source::Story(&story_pictures::STORY_SCENE),
        EntityKind::ArchivePic => return Source::Story(&story_pictures::ARCHIVE_PIC),
    })
}

#[must_use]
pub fn resolve(
    model: &GameModel,
    story: &StoryCatalogues,
    kind: EntityKind,
    id: &str,
) -> Option<EntitySummary> {
    match source(kind) {
        Source::Data(s) => (s.resolve)(model, id),
        Source::Story(s) => (s.resolve)(story, id),
    }
}

#[must_use]
pub fn known(model: &GameModel, story: &StoryCatalogues, kind: EntityKind, id: &str) -> bool {
    match source(kind) {
        Source::Data(s) => (s.known)(model, id),
        Source::Story(s) => (s.known)(story, id),
    }
}

/// Narrower than [`resolve`]: an unobtainable operator resolves but is not offered
#[must_use]
pub fn catalogue(
    model: &GameModel,
    story: &StoryCatalogues,
    kind: EntityKind,
) -> Vec<EntitySummary> {
    match source(kind) {
        Source::Data(s) => (s.catalogue)(model),
        Source::Story(s) => (s.catalogue)(story),
    }
}

/// `Ok` when any of `servers` knows `(kind, id)`
pub fn validate<'a>(
    servers: impl IntoIterator<Item = (&'a GameModel, &'a StoryCatalogues)>,
    kind: EntityKind,
    id: &str,
) -> Result<(), AppError> {
    if servers
        .into_iter()
        .any(|(model, story)| known(model, story, kind, id))
    {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "unknown {} `{id}`: no loaded game data has it",
            kind.as_str()
        )))
    }
}

/// Every loaded server once, with the picture catalogues `kinds` read
pub async fn loaded_sources(
    ctx: &AppContext,
    kinds: &[EntityKind],
) -> Vec<(Arc<Snapshot>, StoryCatalogues)> {
    let mut snapshots: Vec<(Server, Arc<Snapshot>)> = Vec::new();
    for server in ctx.snapshots.loaded() {
        if let Some(s) = ctx.snapshots.get(server)
            && !snapshots.iter().any(|(_, seen)| Arc::ptr_eq(seen, &s))
        {
            snapshots.push((server, s));
        }
    }
    let mut out = Vec::with_capacity(snapshots.len());
    for (server, snapshot) in snapshots {
        out.push((snapshot, StoryCatalogues::gather(ctx, server, kinds).await));
    }
    out
}

/// [`validate`] over [`loaded_sources`]: a CN-locale editor places what EN lacks
pub async fn validate_loaded(ctx: &AppContext, kind: EntityKind, id: &str) -> Result<(), AppError> {
    let loaded = loaded_sources(ctx, &[kind]).await;
    validate(loaded.iter().map(|(s, story)| (&s.model, story)), kind, id)
}

#[derive(Default)]
struct Facets(BTreeMap<String, FacetValue>);

impl Facets {
    fn one(mut self, key: &str, value: impl Into<String>) -> Self {
        self.0.insert(key.to_owned(), FacetValue::One(value.into()));
        self
    }

    fn one_opt(self, key: &str, value: Option<impl Into<String>>) -> Self {
        match value {
            Some(v) => self.one(key, v),
            None => self,
        }
    }

    fn many(mut self, key: &str, values: &[String]) -> Self {
        if !values.is_empty() {
            self.0
                .insert(key.to_owned(), FacetValue::Many(values.to_vec()));
        }
        self
    }

    /// `char_id`, plus the owner's `operator` and `profession` when it is in the data
    fn owner(self, char_id: &str, op: Option<&Operator>) -> Self {
        let facets = self.one("char_id", char_id);
        match op {
            Some(op) => facets
                .one("operator", op.name.clone())
                .one_opt("profession", wire_name(&op.profession)),
            None => facets,
        }
    }
}

impl From<Facets> for BTreeMap<String, FacetValue> {
    fn from(facets: Facets) -> Self {
        facets.0
    }
}

fn non_empty(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}

/// A game enum's serde spelling (`WARRIOR`, `MELEE`)
fn wire_name<T: Serialize>(value: &T) -> Option<String> {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(s)) => Some(s),
        _ => None,
    }
}

/// The pack directories spell `[uc]`, which a URL path must not carry raw
fn asset_url(path: &str) -> String {
    format!(
        "/assets{}",
        path.replace('[', "%5B")
            .replace(']', "%5D")
            .replace('#', "%23")
    )
}

fn operator_href(char_id: &str) -> String {
    format!("/operators/{char_id}")
}

const fn stars(rarity: &OperatorRarity) -> u8 {
    match rarity {
        OperatorRarity::SixStar => 6,
        OperatorRarity::FiveStar => 5,
        OperatorRarity::FourStar => 4,
        OperatorRarity::ThreeStar => 3,
        OperatorRarity::TwoStar => 2,
        OperatorRarity::OneStar => 1,
    }
}

/// Stable, so equal keys keep their input order
fn in_key_order<K: Ord>(mut keyed: Vec<(K, EntitySummary)>) -> Vec<EntitySummary> {
    keyed.sort_by(|(a, _), (b, _)| a.cmp(b));
    keyed.into_iter().map(|(_, s)| s).collect()
}
