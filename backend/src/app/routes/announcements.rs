//! The in-game notice boards. Read from memory: the maintenance poller
//! refreshes them every few minutes, so these never call Yostar themselves.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
};
use serde::Deserialize;

use crate::app::routes::static_data::shared_json_response;
use crate::app::{error::ApiError, state::AppState};
use crate::core::announcements::{self, Announcement, AnnouncementList};
use crate::core::hypergryph::constants::Server;

#[derive(Debug, Deserialize)]
pub struct BoardQuery {
    server: Option<String>,
}

/// `?server=`, or the site's default server when it has a board, or the first
/// server that does.
fn board_server(state: &AppState, q: &BoardQuery) -> Result<Server, ApiError> {
    match q.server.as_deref() {
        Some(s) => {
            let server = Server::parse(s).ok_or(ApiError::NotFound)?;
            if announcements::servers().any(|b| b == server) {
                Ok(server)
            } else {
                Err(ApiError::NotFoundMessage(format!(
                    "no notice board is read for `{}`",
                    server.as_str()
                )))
            }
        }
        None => Ok(announcements::servers()
            .find(|s| *s == state.default_server)
            .or_else(|| announcements::servers().next())
            .unwrap_or(Server::EN)),
    }
}

/// The in-game notice board of one server, newest first: titles, categories, dates and banners, no bodies.
#[utoipa::path(
    get,
    path = "/announcements",
    operation_id = "announcements_list",
    tag = "meta",
    params(
        ("server" = Option<String>, Query, description = "A server with a board: `en`, `jp` or `kr` (the list is in `servers`). Defaults to the site's default server."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The board. Empty with `checked_at: null` before the first read. Served from memory with an `ETag` and `Cache-Control: public, max-age=300`.", body = AnnouncementList),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<BoardQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let server = board_server(&state, &q)?;
    let json = announcements::list(server).ok_or(ApiError::NotFound)?;
    Ok(shared_json_response(&json, &headers))
}

/// One notice with its body, sanitized to an allowlist of tags, every URL absolute, plus the official notice link.
#[utoipa::path(
    get,
    path = "/announcements/{id}",
    operation_id = "announcement_detail",
    tag = "meta",
    params(
        ("id" = String, Path, description = "The notice id from the board."),
        ("server" = Option<String>, Query, description = "The board the id is from. Defaults as on `/announcements`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The notice. Served from memory with an `ETag` and `Cache-Control: public, max-age=300`.", body = Announcement),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<BoardQuery>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let server = board_server(&state, &q)?;
    let json = announcements::detail(server, &id).ok_or(ApiError::NotFound)?;
    Ok(shared_json_response(&json, &headers))
}
