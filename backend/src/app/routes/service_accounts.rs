use axum::Json;
use axum::extract::State;

use crate::app::state::AppState;
use crate::core::service_account::ServiceAccountsStatus;

/// Whether each service account still logs in.
///
/// Read from memory, never calls the game. An account reads `refused` from the
/// first refused token refresh until one succeeds again, and `unknown` until a
/// job first uses it after a restart.
#[utoipa::path(
    get,
    path = "/service-accounts",
    tag = "meta",
    responses(
        (status = 200, description = "One entry per account with stored credentials.", body = ServiceAccountsStatus),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn service_accounts(State(state): State<AppState>) -> Json<ServiceAccountsStatus> {
    Json(state.service_accounts.status())
}
