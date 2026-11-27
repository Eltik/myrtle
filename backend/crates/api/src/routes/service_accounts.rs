use axum::Json;
use axum::extract::State;
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::accounts::service_account::service_health;
use myrtle_hypergryph::AccountState;
use serde::Serialize;

use crate::error::ApiResult;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new().route("/service-accounts", get(status))
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
struct ServiceAccountsStatus {
    accounts: Vec<ServiceAccountStatus>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
struct ServiceAccountStatus {
    server: &'static str,
    #[ts(type = "\"ok\" | \"refused\" | \"unknown\"")]
    state: &'static str,
    #[ts(type = "number | null")]
    refused_since: Option<i64>,
    #[ts(type = "number | null")]
    last_ok_at: Option<i64>,
}

async fn status(State(ctx): State<AppContext>) -> ApiResult<Json<ServiceAccountsStatus>> {
    let accounts = service_health(&ctx)
        .await?
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
    Ok(Json(ServiceAccountsStatus { accounts }))
}
