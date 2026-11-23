use std::collections::HashSet;

use myrtle_gamedata_raw::tables::gacha::GachaTag;
use myrtle_gamedata_raw::tables::operator::{
    Operator, OperatorPosition, OperatorProfession, PotentialRank,
};
use myrtle_types::Server;
use serde::Serialize;

use super::operators::rarity_to_stars;
use super::static_data::{cached, to_json};
use crate::cache::CachedJson;
use crate::{AppContext, AppError};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentData<'a> {
    pub tags: &'a [GachaTag],
    pub operators: Vec<RecruitmentOperator>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentOperator {
    pub id: String,
    pub name: String,
    pub rarity: u8,
    pub profession: OperatorProfession,
    pub position: OperatorPosition,
    pub tag_list: Vec<String>,
    pub potential_ranks: Vec<RecruitmentPotentialRank>,
    pub talent_potential_ranks: Vec<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentPotentialRank {
    pub description: String,
    pub modifier: Option<RecruitmentModifier>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentModifier {
    pub attribute_type: String,
    pub value: f64,
}

#[must_use]
pub fn strip_tags(line: &str) -> String {
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

#[must_use]
pub fn recruitable_names(recruit_detail: &str) -> HashSet<String> {
    let mut names = HashSet::new();
    for line in recruit_detail.split('\n') {
        let trimmed = line.trim();
        if ["<@rc.title>", "<@rc.subtitle>", "<@rc.em>", "-----"]
            .iter()
            .any(|p| line.starts_with(p))
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

fn recruitment_operator(id: &str, op: &Operator) -> RecruitmentOperator {
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

pub async fn recruitment(ctx: &AppContext, server: Server) -> Result<CachedJson, AppError> {
    cached(ctx, server, "recruitment", |gd| {
        let names = recruitable_names(&gd.gacha.recruit_detail);
        let mut operators: Vec<RecruitmentOperator> = gd
            .operators
            .values()
            .filter(|op| names.contains(&op.name))
            .filter_map(|op| Some(recruitment_operator(op.id.as_deref()?, op)))
            .collect();
        operators.sort_unstable_by(|a, b| a.id.cmp(&b.id)); // map order would move the ETag
        to_json(&RecruitmentData {
            tags: &gd.gacha.gacha_tags,
            operators,
        })
    })
    .await
}
