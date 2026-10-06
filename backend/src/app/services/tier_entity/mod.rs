//! What a tier list placement points at, resolved against one server's game data.
//!
//! A placement stores only `(kind, id)`. Everything a tile shows (name, icon,
//! link, the filters the editor's pool offers) is read here at serve time, so a
//! rename in the game data reaches every list without touching a row, and an id
//! the served data does not know comes back as `None` instead of vanishing.
//!
//! Each kind lives in its own module, which exposes one [`KindSource`]: how to
//! resolve an id, whether an id is known, and the pool's catalogue. Adding a
//! kind is a variant on [`EntityKind`], a module, and a line in [`source`].
//! Kind-specific detail travels in `facets`, so the wire shape does not grow a
//! field per kind.
//!
//! Icons are checked against the server's [`AssetIndex`]: an entity whose art
//! the extract does not hold has `icon: None`, and the client draws its own
//! placeholder rather than a broken image.

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
mod story_sprite;
mod stronghold_bond;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::app::error::ApiError;
use crate::core::gamedata::assets::AssetIndex;
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::operator::Operator;
use crate::database::models::tier_list::EntityKind;

/// One facet value: a single tag, or several (an enemy's several levels, an
/// event's several factions).
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FacetValue {
    One(String),
    Many(Vec<String>),
}

/// The display form of one entity: enough for a tile, a pool filter and a link.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySummary {
    pub kind: EntityKind,
    pub id: String,
    pub name: String,
    /// Image path under the API root (`/avatar/char_002_amiya`), server-neutral:
    /// the client prefixes its own base and server. `None` when the extract
    /// holds no art for the entity.
    pub icon: Option<String>,
    /// Site route for the entity's own page, when it has one.
    pub href: Option<String>,
    /// Kind-specific tags the tile and the pool filters read. Operators carry
    /// `rarity`, `profession`, `sub_profession_id`, `position`, and, when set,
    /// `nation_id`, `appellation`, and the server's own `profession_name`,
    /// `sub_profession_name` and `nation_name`. Subclasses carry
    /// `profession` and the server's `profession_name`; enemies
    /// `enemy_level` and `enemy_index`; events `display_type`, `type`,
    /// `start_time` (unix seconds) and, for a rerun, `rerun`; factions
    /// `power_level` (`nation`, `group` or `team`); Stronghold bonds
    /// `bond_type` (`season` or `regular`) and, for a season bond, `power_ids`.
    /// Skins carry `char_id`, `operator` (the wearer's name), `rarity`,
    /// `profession` and, when filed under one, `brand`; modules `char_id`,
    /// `operator`, `profession`, `module_type` (`X`, `Y`, `D`, `A`, `B`) and
    /// `type_code` (`SUM`); skills `char_id`, `operator`, `profession`, `slot`
    /// (`1` to `3`) and, when the skill has levels, `skill_type` and
    /// `sp_type`; Integrated Strategies entries `theme` (`rogue_N`,
    /// which is IS N+1) and `item_type` (`theme`, `relic`, `band`, ...);
    /// story sprites `source` (`operator` or `npc`), for an operator `char_id`,
    /// and, when the hub places the face, `face` (`x,y` fractions of the plate);
    /// main story episodes `episode` (`0` to `16` on EN), `act` (the
    /// `chapter_table` index) and `act_name`.
    pub facets: BTreeMap<String, FacetValue>,
}

/// One kind's three readings of a server's data.
struct KindSource {
    /// The id's summary, or `None` when the data has no such entity.
    resolve: fn(&GameData, &AssetIndex, &str) -> Option<EntitySummary>,
    /// Whether the data has the entity: `resolve` without building the summary.
    known: fn(&GameData, &AssetIndex, &str) -> bool,
    /// Every entity an editor may place, in pool order.
    catalogue: fn(&GameData, &AssetIndex) -> Vec<EntitySummary>,
}

const fn source(kind: EntityKind) -> &'static KindSource {
    match kind {
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
    }
}

/// `(kind, id)` in `gd`, or `None` when the served data has no such entity.
/// Icons are read off `assets`, the same server's index.
pub fn resolve(
    gd: &GameData,
    assets: &AssetIndex,
    kind: EntityKind,
    id: &str,
) -> Option<EntitySummary> {
    (source(kind).resolve)(gd, assets, id)
}

/// Whether one server knows `(kind, id)`: [`resolve`] without building the
/// summary. Every kind is read off `gd` except story sprites, which live in
/// the asset extract and are read off `assets`.
pub fn known(gd: &GameData, assets: &AssetIndex, kind: EntityKind, id: &str) -> bool {
    (source(kind).known)(gd, assets, id)
}

/// `Ok` when any of `servers` (each server's game data and asset index)
/// knows `(kind, id)`, else a `400` naming both.
///
/// # Errors
/// `ApiError::BadRequest` when no game data knows the entity.
pub fn validate<'a>(
    servers: impl IntoIterator<Item = (&'a GameData, &'a AssetIndex)>,
    kind: EntityKind,
    id: &str,
) -> Result<(), ApiError> {
    if servers
        .into_iter()
        .any(|(gd, assets)| known(gd, assets, kind, id))
    {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "unknown {} `{id}`: no loaded game data has it",
            kind.as_str()
        )))
    }
}

/// Every entity of `kind` an editor may place, ordered the way the pool lists
/// them. Narrower than [`resolve`]: an operator the game marks unobtainable,
/// an enemy the handbook hides, or an activity that is a login bonus still
/// resolves on a list that already holds it, but is not offered.
pub fn catalogue(gd: &GameData, assets: &AssetIndex, kind: EntityKind) -> Vec<EntitySummary> {
    (source(kind).catalogue)(gd, assets)
}

/// A summary's facets, built a tag at a time.
#[derive(Default)]
struct Facets(BTreeMap<String, FacetValue>);

impl Facets {
    fn one(mut self, key: &str, value: impl Into<String>) -> Self {
        self.0.insert(key.to_owned(), FacetValue::One(value.into()));
        self
    }

    /// [`Self::one`] when there is a value.
    fn one_opt(self, key: &str, value: Option<impl Into<String>>) -> Self {
        match value {
            Some(v) => self.one(key, v),
            None => self,
        }
    }

    /// Several values under one tag, when there are any.
    fn many(mut self, key: &str, values: &[String]) -> Self {
        if !values.is_empty() {
            self.0
                .insert(key.to_owned(), FacetValue::Many(values.to_vec()));
        }
        self
    }

    /// The tags every operator-owned entity (skin, module, skill) carries:
    /// `char_id`, and the owner's `operator` name and `profession` when the
    /// operator is in the data.
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

/// `value` unless it is empty.
fn non_empty(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}

/// The wire spelling of a game enum (`WARRIOR`, `MELEE`), read through serde so
/// it can never drift from what the operator index sends.
fn wire_name<T: Serialize>(value: &T) -> Option<String> {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(s)) => Some(s),
        _ => None,
    }
}

/// An asset index path as an API path under `/assets`. The pack directories
/// spell `[uc]`, which a URL path must not carry raw.
fn asset_url(path: &str) -> String {
    format!(
        "/assets{}",
        path.replace('[', "%5B")
            .replace(']', "%5D")
            .replace('#', "%23")
    )
}

/// The site route of an operator's page.
fn operator_href(char_id: &str) -> String {
    format!("/operators/{char_id}")
}

/// `keyed` in ascending key order, keys dropped. The sort is stable, so
/// entities with equal keys keep their input order.
fn in_key_order<K: Ord>(mut keyed: Vec<(K, EntitySummary)>) -> Vec<EntitySummary> {
    keyed.sort_by(|(a, _), (b, _)| a.cmp(b));
    keyed.into_iter().map(|(_, s)| s).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gd_with(id: &str, name: &str, obtainable: bool) -> GameData {
        let mut gd = GameData::new();
        gd.operators.insert(
            id.to_owned(),
            Operator {
                name: name.to_owned(),
                is_not_obtainable: !obtainable,
                ..Default::default()
            },
        );
        gd
    }

    #[test]
    fn an_unknown_id_does_not_resolve() {
        let gd = gd_with("char_002_amiya", "Amiya", true);
        let assets = AssetIndex::default();
        assert!(resolve(&gd, &assets, EntityKind::Operator, "char_002_amiya").is_some());
        assert!(resolve(&gd, &assets, EntityKind::Operator, "char_999_nobody").is_none());
    }

    #[test]
    fn placing_an_unknown_id_is_a_bad_request() {
        let en = gd_with("char_002_amiya", "Amiya", true);
        let cn = gd_with("char_4999_cnonly", "Someone", true);
        let none = AssetIndex::default();
        let servers = [(&en, &none), (&cn, &none)];
        assert!(validate(servers, EntityKind::Operator, "char_002_amiya").is_ok());
        // Known on CN only: still accepted, a CN-locale editor offers it.
        assert!(validate(servers, EntityKind::Operator, "char_4999_cnonly").is_ok());
        let err = validate(servers, EntityKind::Operator, "char_999_nobody")
            .expect_err("unknown id is rejected");
        match err {
            ApiError::BadRequest(msg) => assert!(msg.contains("char_999_nobody"), "{msg}"),
            other => panic!("expected BadRequest, got {other:?}"),
        }
    }

    #[test]
    fn an_unobtainable_operator_resolves_but_is_not_offered() {
        let gd = gd_with("char_1001_amiya2", "Amiya", false);
        let assets = AssetIndex::default();
        assert!(resolve(&gd, &assets, EntityKind::Operator, "char_1001_amiya2").is_some());
        assert!(catalogue(&gd, &assets, EntityKind::Operator).is_empty());
    }

    #[test]
    fn an_operator_carries_its_avatar_link_and_facets() {
        let gd = gd_with("char_002_amiya", "Amiya", true);
        let s = resolve(
            &gd,
            &AssetIndex::default(),
            EntityKind::Operator,
            "char_002_amiya",
        )
        .expect("resolves");
        assert_eq!(s.icon.as_deref(), Some("/avatar/char_002_amiya"));
        assert_eq!(s.href.as_deref(), Some("/operators/char_002_amiya"));
        assert!(s.facets.contains_key("rarity"));
        assert!(s.facets.contains_key("profession"));
    }
}
