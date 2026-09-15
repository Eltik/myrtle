//! Translation layer routes.
//!
//! Split the way `tier_lists` is: public reads in `catalog`, gated writes in
//! `admin`, per-locale grants in `permissions`, composed here and merged into
//! the top-level router.

pub mod admin;
pub mod catalog;
pub mod permissions;

use axum::{
    Router,
    routing::{get, post, put},
};

use crate::app::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Public: the catalog a browser renders from.
        .route("/i18n/manifest", get(catalog::manifest))
        .route("/i18n/{locale}/{namespace}/{hash}", get(catalog::catalog))
        // Admin: the translation workspace.
        .route(
            "/admin/i18n/locales",
            get(admin::list_locales).put(admin::upsert_locale),
        )
        .route("/admin/i18n/writable-locales", get(admin::writable_locales))
        .route("/admin/i18n/messages", get(admin::list_messages))
        .route("/admin/i18n/message", put(admin::update_message))
        .route("/admin/i18n/message/clear", post(admin::clear_message))
        .route("/admin/i18n/namespaces", get(admin::namespaces))
        .route("/admin/i18n/progress", get(admin::progress))
        .route("/admin/i18n/audit", get(admin::audit_log))
        .route("/admin/i18n/audit/entry", get(admin::entry_audit_log))
        .route("/admin/i18n/sync", post(admin::sync))
        .route(
            "/admin/i18n/overrides",
            get(admin::list_overrides).put(admin::put_override),
        )
        .route("/admin/i18n/overrides/delete", post(admin::delete_override))
        .route(
            "/admin/i18n/permissions",
            get(permissions::list).post(permissions::grant),
        )
        .route("/admin/i18n/permissions/revoke", post(permissions::revoke))
}
