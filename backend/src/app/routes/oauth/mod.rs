//! The OAuth 2.0 provider, "Sign in with myrtle.moe".
//!
//! Three groups of routes, by who calls them:
//!
//! - [`apps`]: a developer registers and manages applications, with their own
//!   site token. A super admin can take any application down.
//! - [`consent`]: the consent page (`/oauth/authorize` on the web client)
//!   checks an authorization request and records the player's answer, and a
//!   player reviews what they have authorized. Site token.
//! - [`protocol`]: the token, revocation and userinfo endpoints a third
//!   party's server calls, authenticated by client secret or access token and
//!   answering errors in the RFC 6749 format.
//!
//! The developer-facing walkthrough is `backend/OAUTH.md`.

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app::state::AppState;

pub mod apps;
pub mod consent;
pub mod protocol;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(apps::list, apps::create))
        .routes(routes!(apps::update, apps::delete))
        .routes(routes!(apps::rotate_secret))
        .routes(routes!(apps::admin_delete))
        .routes(routes!(consent::check, consent::decide))
        .routes(routes!(consent::list_authorizations))
        .routes(routes!(consent::revoke_authorization))
        .routes(routes!(protocol::token))
        .routes(routes!(protocol::revoke))
        .routes(routes!(protocol::userinfo))
}
