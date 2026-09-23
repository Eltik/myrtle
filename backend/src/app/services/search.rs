use std::hash::{DefaultHasher, Hash, Hasher};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    core::{gamedata::types::operator::OperatorProfession, hypergryph::constants::Server},
    database::{
        models::user::SearchEntry,
        queries::user_search::{OwnsAll, Rank, UserSearch},
    },
};

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Serialize, Deserialize)]
pub struct SearchPage {
    pub entries: Vec<SearchEntry>,
    #[ts(type = "number")]
    pub total: i64,
}

/// A set of operators named by the caller: `class:WARRIOR` or
/// `sub:centurion`. Resolved against game data per server, since a class on
/// CN holds operators EN has not released.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    Class(OperatorProfession),
    Archetype(String),
}

impl Scope {
    /// The classes a player can own; tokens, traps and the unknown bucket are
    /// not operators.
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

    pub fn parse(token: &str) -> Option<Self> {
        if let Some(raw) = token.strip_prefix("class:") {
            return Self::PLAYABLE
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

    /// The obtainable operators of this scope on one server's data, sorted
    /// so the same scope always binds the same array.
    fn members(&self, state: &AppState, server: Server) -> Option<Vec<String>> {
        let data = state.try_server_data(server)?;
        let game_data = data.game_data.load_full();
        let mut ids: Vec<String> = game_data
            .operators
            .iter()
            .filter(|(_, op)| !op.is_not_obtainable && Self::PLAYABLE.contains(&op.profession))
            .filter(|(_, op)| match self {
                Self::Class(p) => op.profession == *p,
                Self::Archetype(sub) => op.sub_profession_id == *sub,
            })
            .map(|(id, _)| id.clone())
            .collect();
        ids.sort();
        Some(ids)
    }

    /// The scope over every loaded server: the union of members, and how
    /// many each server requires for "owns all". A user can only own what
    /// their own server released, so counting their roster against the union
    /// and comparing to their server's count is exact.
    fn resolve(&self, state: &AppState) -> ResolvedScope {
        let mut union: Vec<String> = Vec::new();
        let mut required: Vec<(String, i64)> = Vec::new();
        for &server in Server::all() {
            let Some(members) = self.members(state, server) else {
                continue;
            };
            required.push((server.as_str().to_owned(), members.len() as i64));
            union.extend(members);
        }
        union.sort();
        union.dedup();
        ResolvedScope { union, required }
    }
}

struct ResolvedScope {
    union: Vec<String>,
    required: Vec<(String, i64)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    /// Highest first everywhere except the join date, where the oldest
    /// accounts are the interesting end.
    pub const fn default_descending(&self) -> bool {
        !matches!(self, Self::Joined)
    }
}

/// Everything the route parsed. `descending` is `None` when the caller left
/// the direction to the sort's default.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// The most operators `has` may name. Every id becomes a bind in two
/// queries and a part of the cache key, so an unbounded list is a cheap way
/// to make one request expensive; the page's picker never needs more.
pub const MAX_HAS: usize = 20;

/// An operator id as the game writes them: `char_1035_wisdel`.
pub fn is_valid_operator_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 50 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

pub async fn search_users(
    state: &AppState,
    request: SearchRequest,
) -> Result<SearchPage, ApiError> {
    let mut hasher = DefaultHasher::new();
    request.hash(&mut hasher);
    let key = CacheKey::Search {
        query_hash: hasher.finish(),
    };

    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }

    let sort_scope = match &request.sort {
        SearchSort::Owned(scope) => Some(scope.resolve(state)),
        _ => None,
    };
    let all_scope = request.all.as_ref().map(|scope| scope.resolve(state));
    let rank = match &request.sort {
        SearchSort::Score => Rank::Score,
        SearchSort::Operators => Rank::Operators,
        SearchSort::Joined => Rank::Joined,
        SearchSort::Enemies => Rank::Enemies,
        SearchSort::Potentials => Rank::Potentials,
        SearchSort::Masteries => Rank::Masteries,
        SearchSort::Modules => Rank::Modules,
        SearchSort::Skins => Rank::Skins,
        SearchSort::Owned(_) => Rank::OwnedOf(
            sort_scope
                .as_ref()
                .map_or(&[][..], |scope| scope.union.as_slice()),
        ),
    };
    let search = UserSearch {
        q: request.q.as_deref(),
        rank,
        descending: request
            .descending
            .unwrap_or_else(|| request.sort.default_descending()),
        has: &request.has,
        support: request.support.as_deref(),
        owns_all: all_scope.as_ref().map(|scope| OwnsAll {
            ids: &scope.union,
            required: &scope.required,
        }),
    };

    let (entries, total) = tokio::try_join!(
        search.fetch_page(
            &state.db,
            i64::from(request.limit),
            i64::from(request.offset)
        ),
        search.count(&state.db),
    )?;

    let page = SearchPage { entries, total };
    state.cache.set(&key, &page).await;
    Ok(page)
}
