use myrtle_engine_story::sprites::sprite_operator;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata::story_sprites::StorySprite;
use myrtle_types::EntityKind;

use super::{EntitySummary, Facets, KindSource, in_key_order, operator_href};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| m.assets.story_sprites().get(id).map(|s| summary(m, s)),
    known: |m, id| m.assets.story_sprites().get(id).is_some(),
    catalogue,
};

fn catalogue(m: &GameModel) -> Vec<EntitySummary> {
    in_key_order(
        m.assets
            .story_sprites()
            .iter()
            .map(|s| {
                let is_npc = sprite_operator(m, &s.id).is_none();
                let unnamed = is_npc && s.speaker.is_none();
                let summary = summary(m, s);
                (((is_npc, unnamed), summary.name.clone(), &s.id), summary)
            })
            .collect(),
    )
}

fn summary(m: &GameModel, s: &StorySprite) -> EntitySummary {
    let operator = sprite_operator(m, &s.id);
    let name = operator
        .as_ref()
        .map(|(_, op)| op.name.clone())
        .or_else(|| s.speaker.clone())
        .unwrap_or_else(|| s.id.clone());
    let lower = name.to_lowercase();
    let aliases: Vec<String> = s
        .aliases
        .iter()
        .filter(|a| a.to_lowercase() != lower)
        .cloned()
        .collect();
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
        .one_opt("face", s.face_center.map(|(x, y)| format!("{x:.3},{y:.3}")))
        .many("aliases", &aliases);
    EntitySummary {
        kind: EntityKind::StorySprite,
        id: s.id.clone(),
        name,
        icon: Some(format!(
            "/story-sprite-thumb/{}",
            urlencoding::encode(&s.id)
        )),
        href: operator.map(|(char_id, _)| operator_href(&char_id)),
        facets: facets.into(),
    }
}
