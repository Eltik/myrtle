use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::services::search::{
    MAX_HAS, Scope, SearchRequest, SearchSort, is_valid_operator_id, search_users,
};
use crate::app::{
    error::ApiError, extractors::pagination::Pagination, services::search::SearchPage,
    state::AppState,
};

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
    /// A sort token: `score`, `operators`, `joined`, `enemies`, `potentials`,
    /// `masteries`, `modules`, `skins`, `class:<PROFESSION>` or `sub:<archetype>`.
    pub sort: Option<String>,
    /// `asc` or `desc`; absent means the sort's own default.
    pub dir: Option<String>,
    /// Comma-separated operator ids the player must all own.
    pub has: Option<String>,
    /// An operator id that must sit in the player's support unit.
    pub support: Option<String>,
    /// `class:<PROFESSION>` or `sub:<archetype>`: the player owns every
    /// obtainable operator of it on their server.
    pub all: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

fn non_empty(value: Option<&String>) -> Option<&str> {
    value.map(|s| s.trim()).filter(|s| !s.is_empty())
}

impl SearchParams {
    /// The wire params as the service's own request, or the 400 that names
    /// what the caller got wrong. Public so a test can drive the exact path
    /// an HTTP request takes, `Query<SearchParams>` included.
    pub fn into_request(self) -> Result<SearchRequest, ApiError> {
        let sort = match non_empty(self.sort.as_ref()) {
            None => SearchSort::Score,
            Some(token) => SearchSort::parse(token)
                .ok_or_else(|| ApiError::BadRequest(format!("unknown sort `{token}`")))?,
        };
        let descending = match non_empty(self.dir.as_ref()) {
            None => None,
            Some("asc") => Some(false),
            Some("desc") => Some(true),
            Some(other) => {
                return Err(ApiError::BadRequest(format!(
                    "dir must be `asc` or `desc`, got `{other}`"
                )));
            }
        };
        let mut has: Vec<String> = non_empty(self.has.as_ref())
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
            return Err(ApiError::BadRequest(format!(
                "has names {} operators; at most {MAX_HAS}",
                has.len()
            )));
        }
        if let Some(bad) = has.iter().find(|id| !is_valid_operator_id(id)) {
            return Err(ApiError::BadRequest(format!(
                "`{bad}` is not an operator id"
            )));
        }
        let support = non_empty(self.support.as_ref()).map(str::to_owned);
        if let Some(id) = &support
            && !is_valid_operator_id(id)
        {
            return Err(ApiError::BadRequest(format!(
                "`{id}` is not an operator id"
            )));
        }
        let all = match non_empty(self.all.as_ref()) {
            None => None,
            Some(token) => Some(Scope::parse(token).ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "all must be `class:<PROFESSION>` or `sub:<archetype>`, got `{token}`"
                ))
            })?),
        };
        Ok(SearchRequest {
            q: non_empty(self.q.as_ref()).map(str::to_owned),
            sort,
            descending,
            has,
            support,
            all,
            limit: self.pagination.limit(),
            offset: self.pagination.offset(),
        })
    }
}

/// Search public players by nickname, ranked by score or by a roster metric,
/// cut by what they own.
#[utoipa::path(
    get,
    path = "/search",
    tag = "search",
    params(
        ("q" = Option<String>, Query, description = "Nickname substring. Absent or empty matches every public player."),
        ("sort" = Option<String>, Query, description = "`score` (default), `operators`, `joined`, `enemies`, `potentials`, `masteries` (skills at M3), `modules` (modules at level 3), `skins`, `class:<PROFESSION>` (owned operators of a class, e.g. `class:WARRIOR`) or `sub:<archetype>` (e.g. `sub:centurion`)."),
        ("dir" = Option<String>, Query, description = "`asc` or `desc`. Defaults to `desc` for every sort except `joined`, whose default is `asc` (oldest accounts first)."),
        ("has" = Option<String>, Query, description = "Comma-separated operator ids the player must all own, at most 20."),
        ("support" = Option<String>, Query, description = "An operator id that must be in the player's support unit."),
        ("all" = Option<String>, Query, description = "`class:<PROFESSION>` or `sub:<archetype>`: only players who own every obtainable operator of it on their server."),
        ("limit" = Option<u32>, Query, description = "Page size. Defaults to 20, capped at 100."),
        ("offset" = Option<u32>, Query, description = "Rows to skip. Defaults to 0.")
    ),
    responses(
        (status = 200, description = "One page of players, each carrying the active sort's value as `metric`.", body = SearchPage),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchPage>, ApiError> {
    let page = search_users(&state, params.into_request()?).await?;
    Ok(Json(page))
}
