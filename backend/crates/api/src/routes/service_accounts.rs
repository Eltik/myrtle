use axum::Json;
use axum::extract::State;
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_hypergryph::AccountState;
use serde::Serialize;

use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new().route("/service-accounts", get(status))
}

#[derive(Serialize)]
struct ServiceAccountsStatus {
    accounts: Vec<ServiceAccountStatus>,
}

#[derive(Serialize)]
struct ServiceAccountStatus {
    server: &'static str,
    state: &'static str,
    refused_since: Option<i64>,
    last_ok_at: Option<i64>,
}

async fn status(State(ctx): State<AppContext>) -> Json<ServiceAccountsStatus> {
    let accounts = ctx
        .accounts
        .health()
        .into_iter()
        .map(|h| ServiceAccountStatus {
            server: h.server.as_str(),
            state: match h.state {
                AccountState::Ok => "ok",
                AccountState::Refused => "refused",
                AccountState::Unknown => "unknown",
            },
            refused_since: h.refused_since,
            last_ok_at: h.last_ok_at,
        })
        .collect();
    Json(ServiceAccountsStatus { accounts })
}
