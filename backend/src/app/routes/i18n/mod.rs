//! Translation layer routes.
//!
//! Split the way `tier_lists` is: public reads in `catalog`, gated writes in
//! `admin`, per-locale grants in `permissions`, composed here and merged into
//! the top-level router.

pub mod admin;
pub mod catalog;
pub mod permissions;

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        // Public: the catalog a browser renders from.
        .routes(routes!(catalog::manifest))
        .routes(routes!(catalog::catalog))
        // Admin: the translation workspace.
        .routes(routes!(admin::list_locales, admin::upsert_locale))
        .routes(routes!(admin::writable_locales))
        .routes(routes!(admin::list_messages))
        .routes(routes!(admin::update_message))
        .routes(routes!(admin::clear_message))
        .routes(routes!(admin::namespaces))
        .routes(routes!(admin::progress))
        .routes(routes!(admin::audit_log))
        .routes(routes!(admin::entry_audit_log))
        .routes(routes!(admin::sync))
        .routes(routes!(admin::list_overrides, admin::put_override))
        .routes(routes!(admin::delete_override))
        .routes(routes!(permissions::list, permissions::grant))
        .routes(routes!(permissions::revoke))
}
