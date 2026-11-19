//! Public player search, ranked by score or a roster metric and cut by ownership.

use std::sync::Arc;
use std::time::Duration;

use myrtle_db::repos::user_search::{self, Direction, OwnsAll, Rank, Search, SearchRow};
use myrtle_gamedata_raw::tables::operator::OperatorProfession;
use myrtle_types::{PageRequest, Server};
use serde::{Deserialize, Serialize};

use super::trimmed;
use crate::cache::key_hash;
use crate::services::accounts::account::UserProfile;
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

const TTL: Duration = Duration::from_mins(2);

/// Each id is a bind in two queries and part of the cache key
pub const MAX_HAS: usize = 20;

/// Tokens, traps and the unknown bucket are not operators a player owns
const PLAYABLE: [OperatorProfession; 8] = [
    OperatorProfession::Guard,
    OperatorProfession::Sniper,
    OperatorProfession::Defender,
    OperatorProfession::Medic,
    OperatorProfession::Supporter,
    OperatorProfession::Caster,
    OperatorProfession::Specialist,
    OperatorProfession::Vanguard,
];

/// One row: the profile and the active sort's value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntry {
    /// A visitor's view whose layout carries the visible tabs only
    #[serde(flatten)]
    pub profile: UserProfile,
    pub metric: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPage {
    pub entries: Vec<SearchEntry>,
    pub total: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
    pub sort: Option<String>,
    /// `asc` or `desc`; absent is the sort's own default
    pub dir: Option<String>,
    /// Comma-separated operator ids the player must all own
    pub has: Option<String>,
    pub support: Option<String>,
    /// `class:<PROFESSION>` or `sub:<archetype>`
    pub all: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// A class or archetype, resolved per server: CN holds operators EN has not released
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Class(OperatorProfession),
    Archetype(String),
}

impl Scope {
    #[must_use]
    pub fn parse(token: &str) -> Option<Self> {
        if let Some(raw) = token.strip_prefix("class:") {
            return PLAYABLE
                .into_iter()
                .find(|p| p.to_raw_str() == raw)
                .map(Self::Class);
        }
        let sub = token.strip_prefix("sub:")?;
        (!sub.is_empty()
            && sub.len() <= 50
            && sub
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'))
        .then(|| Self::Archetype(sub.to_owned()))
    }

    /// Members over every loaded server and each server's "owns all" count; exact, as a user owns only their server's releases
    fn resolve(&self, loaded: &[(Server, Arc<Snapshot>)]) -> (Vec<String>, Vec<(String, i64)>) {
        let mut union = Vec::new();
        let mut required = Vec::new();
        for (server, snapshot) in loaded {
            let before = union.len();
            union.extend(
                snapshot
                    .model
                    .operators
                    .iter()
                    .filter(|(_, op)| !op.is_not_obtainable && PLAYABLE.contains(&op.profession))
                    .filter(|(_, op)| match self {
                        Self::Class(p) => op.profession == *p,
                        Self::Archetype(sub) => op.sub_profession_id == *sub,
                    })
                    .map(|(id, _)| id.clone()),
            );
            required.push((server.as_str().to_owned(), (union.len() - before) as i64));
        }
        union.sort();
        union.dedup();
        (union, required)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchSort {
    Score,
    Operators,
    Joined,
    Enemies,
    Potentials,
    Masteries,
    Modules,
    Skins,
    Owned(Scope),
}

impl SearchSort {
    #[must_use]
    pub fn parse(token: &str) -> Option<Self> {
        Some(match token {
            "score" => Self::Score,
            "operators" => Self::Operators,
            "joined" => Self::Joined,
            "enemies" => Self::Enemies,
            "potentials" => Self::Potentials,
            "masteries" => Self::Masteries,
            "modules" => Self::Modules,
            "skins" => Self::Skins,
            _ => Self::Owned(Scope::parse(token)?),
        })
    }
}

/// The validated request; its `Debug` text is the cache key's input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRequest {
    pub q: Option<String>,
    pub sort: SearchSort,
    pub descending: Option<bool>,
    pub has: Vec<String>,
    pub support: Option<String>,
    pub all: Option<Scope>,
    pub limit: u32,
    pub offset: u32,
}

impl SearchRequest {
    pub fn parse(params: &SearchParams) -> Result<Self, AppError> {
        let sort = match trimmed(params.sort.as_deref()) {
            None => SearchSort::Score,
            Some(token) => SearchSort::parse(token)
                .ok_or_else(|| AppError::bad_request(format!("unknown sort `{token}`")))?,
        };
        let descending = match trimmed(params.dir.as_deref()) {
            None => None,
            Some("asc") => Some(false),
            Some("desc") => Some(true),
            Some(other) => {
                return Err(AppError::bad_request(format!(
                    "dir must be `asc` or `desc`, got `{other}`"
                )));
            }
        };
        let mut has: Vec<String> = trimmed(params.has.as_deref())
            .map(|list| {
                list.split(',')
                    .map(str::trim)
                    .filter(|id| !id.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        has.sort();
        has.dedup();
        if has.len() > MAX_HAS {
            return Err(AppError::bad_request(format!(
                "has names {} operators; at most {MAX_HAS}",
                has.len()
            )));
        }
        let support = trimmed(params.support.as_deref()).map(str::to_owned);
        if let Some(bad) = has
            .iter()
            .chain(&support)
            .find(|id| !is_valid_operator_id(id))
        {
            return Err(AppError::bad_request(format!(
                "`{bad}` is not an operator id"
            )));
        }
        let all = match trimmed(params.all.as_deref()) {
            None => None,
            Some(token) => Some(Scope::parse(token).ok_or_else(|| {
                AppError::bad_request(format!(
                    "all must be `class:<PROFESSION>` or `sub:<archetype>`, got `{token}`"
                ))
            })?),
        };
        let paging = PageRequest {
            limit: params.limit,
            offset: params.offset,
        };
        Ok(Self {
            q: trimmed(params.q.as_deref()).map(str::to_owned),
            sort,
            descending,
            has,
            support,
            all,
            limit: paging.limit(),
            offset: paging.offset(),
        })
    }
}

/// An operator id as the game writes them: `char_1035_wisdel`
#[must_use]
pub fn is_valid_operator_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 50 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

pub async fn search(ctx: &AppContext, params: &SearchParams) -> Result<SearchPage, AppError> {
    let request = SearchRequest::parse(params)?;
    let scoped = matches!(request.sort, SearchSort::Owned(_)) || request.all.is_some();
    let loaded: Vec<(Server, Arc<Snapshot>)> = Server::ALL
        .into_iter()
        .filter(|_| scoped)
        .filter_map(|server| Some((server, ctx.snapshots.get(server)?)))
        .collect();
    let key = loaded.iter().fold(
        format!("search:{}", key_hash(format!("{request:?}").as_bytes())),
        |key, (_, snapshot)| snapshot.key(&key),
    );
    let mut page = match ctx.cache.get_json(&key).await {
        Some(cached) => cached,
        None => build(ctx, &request, &loaded, &key).await?,
    };
    page.entries.iter_mut().for_each(hide_private); // a projected layout would re-normalize on a cache read, so the cache holds the owner's rows
    Ok(page)
}

async fn build(
    ctx: &AppContext,
    request: &SearchRequest,
    loaded: &[(Server, Arc<Snapshot>)],
    key: &str,
) -> Result<SearchPage, AppError> {
    let sort_scope = match &request.sort {
        SearchSort::Owned(scope) => Some(scope.resolve(loaded)),
        _ => None,
    };
    let all_scope = request.all.as_ref().map(|scope| scope.resolve(loaded));
    let rank = match &request.sort {
        SearchSort::Score => Rank::Score,
        SearchSort::Operators => Rank::Operators,
        SearchSort::Joined => Rank::Joined,
        SearchSort::Enemies => Rank::Enemies,
        SearchSort::Potentials => Rank::Potentials,
        SearchSort::Masteries => Rank::Masteries,
        SearchSort::Modules => Rank::Modules,
        SearchSort::Skins => Rank::Skins,
        SearchSort::Owned(_) => Rank::OwnedOf(sort_scope.as_ref().map_or(&[], |s| &s.0)),
    };
    let descending = request
        .descending
        .unwrap_or(!matches!(request.sort, SearchSort::Joined)); // the oldest accounts are the interesting end
    let query = Search {
        q: request.q.as_deref(),
        rank,
        direction: if descending {
            Direction::Desc
        } else {
            Direction::Asc
        },
        has: &request.has,
        support: request.support.as_deref(),
        owns_all: all_scope
            .as_ref()
            .map(|(ids, required)| OwnsAll { ids, required }),
    };
    let mut conn = ctx.db.conn().await?;
    let rows = user_search::page(
        &mut conn,
        &query,
        i64::from(request.limit),
        i64::from(request.offset),
    )
    .await?;
    let total = user_search::count(&mut conn, &query).await?;
    let page = SearchPage {
        entries: rows
            .into_iter()
            .map(|SearchRow { profile, metric }| SearchEntry {
                profile: profile.into(),
                metric,
            })
            .collect(),
        total,
    };
    ctx.cache.set_json(key, &page, TTL).await;
    Ok(page)
}

/// A row draws no header and no blocks, so the layout keeps only the visible tabs
fn hide_private(entry: &mut SearchEntry) {
    entry.profile.hide_private();
    if let Some(layout) = entry.profile.profile_layout.as_mut() {
        layout.showcase = None;
        layout.background = None;
        layout.header = None;
    }
}
