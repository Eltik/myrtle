//! The `OpenAPI` document for the public API, and the shared pieces annotated
//! handlers reference.
//!
//! The router is a [`utoipa_axum::router::OpenApiRouter`]: a handler in `routes!()`
//! takes its path and method from its own `#[utoipa::path]`, so registering a route
//! and documenting it are one declaration. A plain `.route()` still serves but never
//! reaches the spec (that let the 166-handler annotation pass go incrementally).
//! None are left: `tests/openapi_snapshot_test.rs` asserts the count is zero and
//! pins the document, so any drift lands in a PR as a diff.
//!
//! # Adding an endpoint to the spec
//!
//! 1. Put `#[utoipa::path(...)]` on the handler, with the path written as it
//!    appears inside [`crate::app::routes::router`] (the `/api` prefix is
//!    added by the `nest` in [`crate::app::server`]; do not repeat it here).
//! 2. Derive `ToSchema` on the request and response types.
//! 3. Swap its `.route("/x", get(handler))` for `.routes(routes!(handler))`.
//! 4. Re-run the snapshot: `UPDATE_OPENAPI=1 cargo test --test openapi_snapshot_test`.

use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi, ToResponse};

use crate::app::error::ErrorBody;

/// Named error responses, so a 401's description is written once for the whole
/// API. Each is a newtype over [`ErrorBody`].
pub mod responses {
    // Each response is a newtype whose field exists only so `ToResponse` can
    // point at `ErrorBody`'s schema; nothing constructs these types at runtime.
    #![allow(dead_code)]

    use super::{ErrorBody, ToResponse};

    #[derive(ToResponse)]
    #[response(description = "The request body or query string was malformed.")]
    pub struct BadRequest(ErrorBody);

    #[derive(ToResponse)]
    #[response(
        description = "No credentials were supplied, or the bearer token is expired or invalid."
    )]
    pub struct Unauthorized(ErrorBody);

    #[derive(ToResponse)]
    #[response(
        description = "Authenticated, but not permitted to read or modify this resource. \
                       Reading another player's data returns this when their profile is private."
    )]
    pub struct Forbidden(ErrorBody);

    #[derive(ToResponse)]
    #[response(description = "No such resource.")]
    pub struct NotFound(ErrorBody);

    #[derive(ToResponse)]
    #[response(
        description = "The write collides with something already stored, such as a \
                       group name that is already taken."
    )]
    pub struct Conflict(ErrorBody);

    #[derive(ToResponse)]
    #[response(
        description = "Rate limit exceeded. The limit is per-IP and applies across the whole API."
    )]
    pub struct RateLimited(ErrorBody);

    #[derive(ToResponse)]
    #[response(
        description = "The body was well-formed but failed validation; `error.details` \
                       lists the offending fields."
    )]
    pub struct ValidationFailed(ErrorBody);

    #[derive(ToResponse)]
    #[response(description = "Unexpected server-side failure.")]
    pub struct InternalError(ErrorBody);

    #[derive(ToResponse)]
    #[response(
        description = "The server is shedding load (CPU admission control or database \
                       backpressure). The request is safe to retry."
    )]
    pub struct ServiceUnavailable(ErrorBody);
}

/// The two auth schemes [`crate::app::extractors::auth::AuthUser`] reads: a player
/// JWT in `Authorization: Bearer`, and the service key the frontend's SSR uses. The
/// key is documented because the extractor reads it, not because third parties can
/// get one.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi
            .components
            .as_mut()
            .expect("components registered by the OpenApi derive");

        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some(
                        "A site token from one of the `/api/login*` endpoints. Valid for seven days.",
                    ))
                    .build(),
            ),
        );

        components.add_security_scheme(
            "service_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                "x-service-key",
                "Internal shared secret for server-side rendering. Not issued to third parties.",
            ))),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "myrtle.moe API",
        description = "\
The public HTTP API behind [myrtle.moe](https://myrtle.moe), an Arknights companion app.

Every endpoint lives under `/api`. Responses are JSON unless noted; errors share a single \
envelope whose `error.code` is the stable field to branch on.

This document is generated from the server's own route definitions, so it describes what \
the running server actually serves: a route and its OpenAPI operation are the same \
declaration, and a test refuses any route that lacks one. If an endpoint is missing here, \
it is missing from the server too.",
        license(name = "Documentation for the myrtle.moe API"),
        contact(name = "myrtle.moe", url = "https://myrtle.moe")
    ),
    servers(
        (url = "https://api.myrtle.moe", description = "Production"),
        (url = "http://localhost:3060", description = "Local development")
    ),
    modifiers(&SecurityAddon),
    components(
        schemas(
            ErrorBody,
            // `PlanRequirementItem::recipe` has `#[schema(no_recursion)]` to cut the
            // item/recipe cycle, so utoipa never collects `PlanRecipe` while the field
            // still `$ref`s it. Without this line the `$ref` dangles and validators
            // reject the document; `tests/openapi_snapshot_test.rs` caught it.
            crate::database::models::planner::PlanRecipe,
            // A query-parameter enum is referenced by `$ref` from the handler's
            // `params(...)` but utoipa collects schemas only from bodies, so it
            // must be named here or the `$ref` dangles.
            crate::app::services::max_level::LevelTarget,
        ),
        responses(
            responses::BadRequest,
            responses::Unauthorized,
            responses::Forbidden,
            responses::NotFound,
            responses::Conflict,
            responses::RateLimited,
            responses::ValidationFailed,
            responses::InternalError,
            responses::ServiceUnavailable,
        )
    ),
    tags(
        (name = "auth", description = "Logging in to a Hypergryph, Bilibili or CN account, and managing the resulting session."),
        (name = "leaderboard", description = "Global and per-server score rankings, movement over time, a single player's standing, and per-item inventory rankings."),
        (name = "dps", description = "Damage- and healing-per-second simulation. Pure functions of the request body and the loaded game data."),
        (name = "gamedata", description = "Game data as shipped by the game: operators, stages, enemies, skins and the raw tables behind them. Cacheable and unauthenticated."),
        (name = "assets", description = "Images, audio and animation files. Long-lived cache headers, conditional requests and byte ranges."),
        (name = "player", description = "One player's synced account data. Reading another player's data requires their profile to be public."),
        (name = "gacha", description = "Pull history and statistics, both the caller's own and the community aggregate."),
        (name = "base", description = "RIIC base planning: catalog, layout scoring, assignment search and shift rotation."),
        (name = "planner", description = "Per-operator upgrade plans and the groups they are filed under."),
        (name = "release", description = "What CN has that this server does not yet, and when it is projected to arrive."),
        (name = "notes", description = "Editorial operator notes and their audit trail."),
        (name = "search", description = "Cross-entity search."),
        (name = "trevor", description = "Trevor, the story question answerer: questions answered from cache or queued for a GPU worker, per-story panels from the published corpus version, feedback, and the worker protocol behind the service key."),
        (name = "meta", description = "Health and site-wide totals."),
        (name = "admin", description = "Administrative endpoints. A non-admin token is answered with 403, not 404."),
        (name = "tier-lists", description = "Community tier lists: their tiers, the operators placed in them, published versions, per-list edit grants and engagement counters."),
        (name = "grids", description = "Community grids: titled boards of labelled cells, each optionally holding one tier-list entity, and forks that reuse a grid as a template."),
        (name = "i18n", description = "The public translation catalog a browser renders from. The content hash is in the path, so a catalog response is immutable and cacheable indefinitely."),
        (name = "i18n-admin", description = "The translation workspace: locales, message editing with an audit trail, game-data text overrides and per-locale grants.")
    )
)]
pub struct ApiDoc;
