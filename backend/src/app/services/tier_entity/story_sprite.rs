//! Story character sprites, keyed by folder with the set number cut
//! (`avg_npc_935`). They live in the asset extract, not the game tables:
//! see [`crate::core::gamedata::story_sprites`].

use super::{EntitySummary, Facets, KindSource, in_key_order, operator_href};
use crate::core::gamedata::assets::AssetIndex;
use crate::core::gamedata::story_sprites::StorySprite;
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::operator::Operator;
use crate::database::models::tier_list::EntityKind;

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |gd, assets, id| assets.story_sprites().get(id).map(|s| summary(gd, s)),
    known: |_, assets, id| assets.story_sprites().get(id).is_some(),
    catalogue,
};

/// Operators first, then the cast by name; a character the scripts never
/// name sorts after the named ones, by its id.
fn catalogue(gd: &GameData, assets: &AssetIndex) -> Vec<EntitySummary> {
    in_key_order(
        assets
            .story_sprites()
            .iter()
            .map(|s| {
                let is_npc = operator(gd, &s.id).is_none();
                let unnamed = is_npc && s.speaker.is_none();
                let summary = summary(gd, s);
                (((is_npc, unnamed), summary.name.clone(), &s.id), summary)
            })
            .collect(),
    )
}

/// The operator a story sprite draws, when its id names one:
/// `char_003_kalts`, `avg_103_angel`, `avgnew_112_siege`, `avg_char_501_durin`.
fn operator<'a>(gd: &'a GameData, id: &str) -> Option<(String, &'a Operator)> {
    let rest = ["avgnew_", "avg_char_", "avg_", "char_"]
        .iter()
        .find_map(|p| id.strip_prefix(p))?;
    if !rest.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let char_id = format!("char_{rest}");
    let op = gd.operators.get(&char_id)?;
    Some((char_id, op))
}

fn summary(gd: &GameData, s: &StorySprite) -> EntitySummary {
    let operator = operator(gd, &s.id);
    let facets = Facets::default()
        .one(
            "source",
            if operator.is_some() {
                "operator"
            } else {
                "npc"
            },
        )
        .one_opt(
            "char_id",
            operator.as_ref().map(|(char_id, _)| char_id.clone()),
        )
        .one_opt("face", s.face_center.map(|(x, y)| format!("{x:.3},{y:.3}")));
    // An operator by their roster name; the cast by the name the scripts
    // speak them under; anyone else by id.
    let name = operator
        .as_ref()
        .map(|(_, op)| op.name.clone())
        .or_else(|| s.speaker.clone())
        .unwrap_or_else(|| s.id.clone());
    EntitySummary {
        kind: EntityKind::StorySprite,
        id: s.id.clone(),
        name,
        // The head-and-shoulders thumbnail, not the ~0.7 MB whole-figure plate:
        // see `story_sprite_thumb`.
        icon: Some(format!(
            "/story-sprite-thumb/{}",
            urlencoding::encode(&s.id)
        )),
        href: operator.map(|(char_id, _)| operator_href(&char_id)),
        facets: facets.into(),
    }
}
