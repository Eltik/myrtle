use axum::Json;

use crate::core::maintenance::{self, MaintenanceStatus};

/// The next EN game maintenance, from Yostar's own notices.
///
/// Read from memory: a background poller refreshes it every few minutes (every
/// minute while a window is running), so this never calls Yostar itself.
/// `window` is `null` when nothing is announced, or before the first poll.
#[utoipa::path(
    get,
    path = "/maintenance",
    tag = "meta",
    responses(
        (status = 200, description = "The nearest upcoming, running or overrunning window, if any.", body = MaintenanceStatus),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn maintenance() -> Json<MaintenanceStatus> {
    Json(maintenance::current())
}
