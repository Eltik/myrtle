//! The recruitment calculator's input: the gacha tags plus the recruitable
//! operators, reduced to the fields the calculator reads. Replaces the frontend
//! server fn pulling `/static/gacha` and the whole `/static/operators` table
//! (23,946,518 B on production) on every SSR call.

use std::collections::HashSet;

use serde::Serialize;
use ts_rs::TS;

use crate::app::cache::keys::CacheKey;
use crate::app::cache::{CachedJson, cached_json};
use crate::app::error::ApiError;
use crate::app::services::operators::rarity_to_stars;
use crate::app::state::AppState;
use crate::core::gamedata::types::gacha::GachaTag;
use crate::core::gamedata::types::operator::{
    Operator, OperatorPosition, OperatorProfession, PotentialRank,
};
use crate::core::hypergryph::constants::Server;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentData {
    pub tags: Vec<GachaTag>,
    /// Every operator `recruitDetail` names, sorted by id.
    pub operators: Vec<RecruitmentOperator>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentOperator {
    pub id: String,
    pub name: String,
    /// 1-6, converted from the `TIER_N` enum.
    pub rarity: u8,
    pub profession: OperatorProfession,
    pub position: OperatorPosition,
    /// The game's own affix tags, unfiltered: the frontend adds the position,
    /// class and rarity tags and hides the ones the tool does not show.
    pub tag_list: Vec<String>,
    pub potential_ranks: Vec<RecruitmentPotentialRank>,
    /// Per talent, every candidate's `requiredPotentialRank`; the frontend
    /// names the talent a CUSTOM rank unlocks from these.
    pub talent_potential_ranks: Vec<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentPotentialRank {
    pub description: String,
    /// The rank's first attribute modifier, the only one the label reads.
    pub modifier: Option<RecruitmentModifier>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentModifier {
    pub attribute_type: String,
    pub value: f64,
}

/// Drop every `<...>` tag. An unclosed `<` is kept, as the regex
/// `/<[^>]*>/g` this ports from keeps it.
fn strip_tags(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(open) = rest.find('<') {
        let Some(close) = rest[open..].find('>') else {
            break;
        };
        out.push_str(&rest[..open]);
        rest = &rest[open + close + 1..];
    }
    out.push_str(rest);
    out
}

/// Operator names listed in `recruitDetail`, the in-game recruitment pool text.
/// Ported from the frontend's `parseRecruitableNames`: headings, separators and
/// star-only lines are skipped, the rest are `/`-separated names with stars.
pub(crate) fn recruitable_names(recruit_detail: &str) -> HashSet<String> {
    let mut names = HashSet::new();
    for line in recruit_detail.split('\n') {
        let trimmed = line.trim();
        if line.starts_with("<@rc.title>")
            || line.starts_with("<@rc.subtitle>")
            || line.starts_with("<@rc.em>")
            || line.starts_with("-----")
            || trimmed.is_empty()
            || trimmed.chars().all(|c| c == '★')
        {
            continue;
        }
        for part in strip_tags(line).split('/') {
            let name = part.replace('★', "");
            let name = name.trim();
            if !name.is_empty() {
                names.insert(name.to_owned());
            }
        }
    }
    names
}

fn potential_rank(rank: &PotentialRank) -> RecruitmentPotentialRank {
    let modifier = rank
        .buff
        .as_ref()
        .and_then(|b| b.attributes.attribute_modifiers.as_ref())
        .and_then(|mods| mods.first())
        .map(|m| RecruitmentModifier {
            attribute_type: m.attribute_type.clone(),
            value: m.value,
        });
    RecruitmentPotentialRank {
        description: rank.description.clone(),
        modifier,
    }
}

fn to_recruitment_operator(id: &str, op: &Operator) -> RecruitmentOperator {
    RecruitmentOperator {
        id: id.to_owned(),
        name: op.name.clone(),
        rarity: rarity_to_stars(&op.rarity),
        profession: op.profession.clone(),
        position: op.position.clone(),
        tag_list: op.tag_list.clone(),
        potential_ranks: op.potential_ranks.iter().map(potential_rank).collect(),
        talent_potential_ranks: op
            .talents
            .iter()
            .map(|t| {
                t.candidates
                    .iter()
                    .map(|c| c.required_potential_rank)
                    .collect()
            })
            .collect(),
    }
}

pub async fn get_recruitment(state: &AppState, server: Server) -> Result<CachedJson, ApiError> {
    let sd = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let key = CacheKey::StaticData {
        resource: "recruitment",
        server: server.as_str(),
        fields_hash: 0,
        page: 0,
    };
    cached_json(state, &key, move || async move {
        let gd = sd.game_data.load_full();
        let names = recruitable_names(&gd.gacha.recruit_detail);

        let mut operators: Vec<RecruitmentOperator> = gd
            .operators
            .values()
            .filter(|op| names.contains(&op.name))
            // The server fn this replaces skipped a record with no `id`; so does this.
            .filter_map(|op| Some(to_recruitment_operator(op.id.as_deref()?, op)))
            .collect();
        // HashMap order would change the body, and so the ETag, per process.
        operators.sort_unstable_by(|a, b| a.id.cmp(&b.id));

        serde_json::to_string(&RecruitmentData {
            tags: gd.gacha.gacha_tags.clone(),
            operators,
        })
        .map_err(|e| ApiError::Internal(e.into()))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::{recruitable_names, strip_tags};

    #[test]
    fn strip_tags_keeps_an_unclosed_bracket() {
        assert_eq!(
            strip_tags("<@rc.eml>Lancet-2</> / Castle-3"),
            "Lancet-2 / Castle-3"
        );
        assert_eq!(strip_tags("a < b"), "a < b");
    }

    #[test]
    fn recruitable_names_skip_headings_and_star_lines() {
        let detail = "<@rc.title>Recruitment rules</>\n\
                      <@rc.subtitle>※Recruitable operators※</>\n\
                      <@rc.em>Note</>\n\
                      ------------------\n\
                      \n\
                      ★\n\
                      ★★★\n\
                      <@rc.eml>Lancet-2</> / Castle-3 / THRM-EX\n\
                      Yato / Noir Corne / Rangers\n\
                      ★★ Steward / Kroos ★";
        let mut names: Vec<_> = recruitable_names(detail).into_iter().collect();
        names.sort_unstable();
        assert_eq!(
            names,
            [
                "Castle-3",
                "Kroos",
                "Lancet-2",
                "Noir Corne",
                "Rangers",
                "Steward",
                "THRM-EX",
                "Yato"
            ]
        );
    }
}
