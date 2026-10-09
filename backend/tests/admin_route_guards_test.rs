//! Pins who may call each admin and staff route, so a dropped or loosened role
//! check fails a test instead of shipping.
//!
//! The route list is derived, not picked by hand: every operation under
//! `/api/admin` in the served `OpenAPI` document, plus every operation whose
//! handler calls one of the role guards (found by reading `src/app/routes`, the
//! way `openapi_snapshot_test` counts routes). [`GUARDED`] must name exactly that
//! set, so a new admin route cannot land without a row here.
//!
//! Each row is then checked three ways:
//! - through the real router from `api_parts`, with no token, a token signed with
//!   the wrong secret, and a token for every `GlobalRole`: callers below the gate
//!   get 401 or 403, callers at or above it are let through;
//! - against its `OpenAPI` annotation: a required credential, a documented 401
//!   and 403, and the role its prose names, where every place the prose and the
//!   code disagree is spelled out in the row ([`Doc::Drift`]);
//! - against its handler source: the guard it calls is the one its gate names.
//!
//! No database. The pool points at a closed port, so a request the role check
//! lets through fails at its first query with a 5xx. That leaves one thing the
//! router alone cannot show: for a gate a database row can open (a translation
//! grant), "admitted by role" and "sent to the grant table" both end in that
//! 5xx. `translation_guards_admit_by_role_only_where_expected` calls the two
//! guards directly to tell them apart. Whether a grant row then admits or
//! refuses needs the local Postgres and is not covered here.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use backend::app::cache::store::CacheStore;
use backend::app::error::ApiError;
use backend::app::extractors::auth::AuthUser;
use backend::app::server::api_parts;
use backend::app::services::i18n;
use backend::app::state::{AppConfig, AppState};
use backend::core::auth::credentials::CredentialKey;
use backend::core::auth::jwt::create_token;
use backend::core::auth::permissions::{GlobalRole, Permission};
use backend::core::hypergryph::constants::Server;
use backend::core::service_account::ServiceAccounts;
use futures_util::future::join_all;
use tower::ServiceExt;
use utoipa::openapi::path::{Operation, PathItem};

const JWT_SECRET: &str = "admin-route-guards-test-secret";
/// Port 1 refuses connections, so every query fails fast and nothing can reach
/// a real database.
const CLOSED_DB: &str = "postgres://guards@127.0.0.1:1/none";
/// The caller's account id in every minted token.
const SUBJECT: &str = "00000000-0000-4000-8000-000000000001";

/// Who a route admits, as the code decides it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Gate {
    /// `super_admin` only.
    SuperAdmin,
    /// `tier_list_admin` and `super_admin`.
    TierListAdmin,
    /// The tier-list staff roles: `tier_list_editor`, `tier_list_admin` and
    /// `super_admin`. Not `translator`.
    Staff,
    /// Any role but `user`. A `user` gets in only with a translation grant row.
    PanelOrAnyGrant,
    /// `super_admin`. Anyone else, whatever the role, needs an `edit` grant on
    /// the body's locale.
    LocaleEdit,
    /// `super_admin`. Anyone else needs an `admin` grant on the body's locale.
    LocaleAdmin,
    /// Any signed-in account; the handler scopes the answer to the caller.
    SignedIn,
    /// No credential at all.
    Public,
}

/// What a route's `OpenAPI` prose says about who may call it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Claim {
    SuperAdmin,
    TierListAdmin,
    Staff,
    PanelOrGrant,
    SignedIn,
    ListOwner,
}

/// How a row's prose relates to its gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Doc {
    /// The prose names the gate's role.
    Matches,
    /// The prose says nothing about who may call it.
    Silent,
    /// The prose names a different rule than the code enforces. Pinned so the
    /// disagreement stays visible until one side is changed.
    Drift(Claim),
}

struct Guarded {
    method: &'static str,
    /// As the `OpenAPI` document spells it.
    path: &'static str,
    gate: Gate,
    doc: Doc,
    /// A concrete URI: path parameters filled, required query present.
    uri: &'static str,
    /// A body the handler's `Json` extractor accepts, so that a refusal comes
    /// from the guard and not from the parser.
    body: Option<&'static str>,
}

const fn row(
    method: &'static str,
    path: &'static str,
    gate: Gate,
    doc: Doc,
    uri: &'static str,
    body: Option<&'static str>,
) -> Guarded {
    Guarded {
        method,
        path,
        gate,
        doc,
        uri,
        body,
    }
}

const GRANT_BODY: &str =
    r#"{"locale":"ja","user_id":"00000000-0000-4000-8000-000000000002","permission":"edit"}"#;

/// Every admin and staff route. `every_admin_and_staff_route_has_a_row` derives
/// the set from the document and the handlers and compares.
const GUARDED: &[Guarded] = &[
    // Super-admin.
    row(
        "PUT",
        "/api/admin/users/{user_id}/role",
        Gate::SuperAdmin,
        Doc::Matches,
        "/api/admin/users/00000000-0000-4000-8000-000000000002/role",
        Some(r#"{"role":"user"}"#),
    ),
    row(
        "PUT",
        "/api/admin/i18n/locales",
        Gate::SuperAdmin,
        Doc::Matches,
        "/api/admin/i18n/locales",
        Some(
            r#"{"code":"xx","english_name":"Probe","native_name":"Probe","gamedata_server":"en","enabled":false}"#,
        ),
    ),
    row(
        "POST",
        "/api/admin/i18n/sync",
        Gate::SuperAdmin,
        Doc::Matches,
        "/api/admin/i18n/sync",
        Some(r#"{"entries":[]}"#),
    ),
    row(
        "GET",
        "/api/release/overrides",
        Gate::SuperAdmin,
        Doc::Silent,
        "/api/release/overrides",
        None,
    ),
    row(
        "PUT",
        "/api/release/overrides",
        Gate::SuperAdmin,
        Doc::Silent,
        "/api/release/overrides",
        Some(r#"{"kind":"event","cnId":"probe"}"#),
    ),
    row(
        "DELETE",
        "/api/release/overrides/{kind}/{cn_id}",
        Gate::SuperAdmin,
        Doc::Silent,
        "/api/release/overrides/event/probe",
        None,
    ),
    // Tier list admin and up.
    row(
        "GET",
        "/api/admin/users",
        Gate::TierListAdmin,
        Doc::Matches,
        "/api/admin/users",
        None,
    ),
    row(
        "GET",
        "/api/admin/stats",
        Gate::TierListAdmin,
        Doc::Silent,
        "/api/admin/stats",
        None,
    ),
    row(
        "GET",
        "/api/admin/tier-lists/permissions",
        Gate::TierListAdmin,
        Doc::Matches,
        "/api/admin/tier-lists/permissions",
        None,
    ),
    // The prose says "Owner only"; the handler checks the global role and never
    // looks at the owner, so a list's owner who is a plain player gets 403.
    row(
        "GET",
        "/api/tier-lists/{slug}/permissions",
        Gate::TierListAdmin,
        Doc::Drift(Claim::ListOwner),
        "/api/tier-lists/probe/permissions",
        None,
    ),
    row(
        "POST",
        "/api/tier-lists/{slug}/permissions",
        Gate::TierListAdmin,
        Doc::Drift(Claim::ListOwner),
        "/api/tier-lists/probe/permissions",
        Some(r#"{"user_id":"00000000-0000-4000-8000-000000000002","permission":"view"}"#),
    ),
    row(
        "DELETE",
        "/api/tier-lists/{slug}/permissions/{user_id}/{permission}",
        Gate::TierListAdmin,
        Doc::Drift(Claim::ListOwner),
        "/api/tier-lists/probe/permissions/00000000-0000-4000-8000-000000000002/view",
        None,
    ),
    row(
        "POST",
        "/api/tier-list-flairs",
        Gate::TierListAdmin,
        Doc::Silent,
        "/api/tier-list-flairs",
        Some(r#"{"code":"probe","label":"Probe"}"#),
    ),
    // Tier-list staff.
    row(
        "GET",
        "/api/admin/operator-notes/audit",
        Gate::Staff,
        Doc::Matches,
        "/api/admin/operator-notes/audit",
        None,
    ),
    row(
        "PUT",
        "/api/operator-notes/{operator_id}",
        Gate::Staff,
        Doc::Silent,
        "/api/operator-notes/char_002_amiya",
        Some("{}"),
    ),
    // Any panel role, or any translation grant.
    row(
        "GET",
        "/api/admin/i18n/locales",
        Gate::PanelOrAnyGrant,
        Doc::Matches,
        "/api/admin/i18n/locales",
        None,
    ),
    row(
        "GET",
        "/api/admin/i18n/writable-locales",
        Gate::PanelOrAnyGrant,
        Doc::Silent,
        "/api/admin/i18n/writable-locales",
        None,
    ),
    row(
        "GET",
        "/api/admin/i18n/messages",
        Gate::PanelOrAnyGrant,
        Doc::Matches,
        "/api/admin/i18n/messages?locale=ja",
        None,
    ),
    row(
        "GET",
        "/api/admin/i18n/namespaces",
        Gate::PanelOrAnyGrant,
        Doc::Silent,
        "/api/admin/i18n/namespaces",
        None,
    ),
    row(
        "GET",
        "/api/admin/i18n/progress",
        Gate::PanelOrAnyGrant,
        Doc::Silent,
        "/api/admin/i18n/progress",
        None,
    ),
    row(
        "GET",
        "/api/admin/i18n/audit",
        Gate::PanelOrAnyGrant,
        Doc::Silent,
        "/api/admin/i18n/audit",
        None,
    ),
    row(
        "GET",
        "/api/admin/i18n/audit/entry",
        Gate::PanelOrAnyGrant,
        Doc::Silent,
        "/api/admin/i18n/audit/entry?locale=ja&key=probe",
        None,
    ),
    row(
        "GET",
        "/api/admin/i18n/overrides",
        Gate::PanelOrAnyGrant,
        Doc::Silent,
        "/api/admin/i18n/overrides?locale=ja",
        None,
    ),
    row(
        "GET",
        "/api/admin/i18n/permissions",
        Gate::PanelOrAnyGrant,
        Doc::Matches,
        "/api/admin/i18n/permissions",
        None,
    ),
    // A grant on the locale, or super-admin. The prose says "or a role that can
    // open the admin panel", but `assert_can_write` lets only a super-admin past
    // without a grant row: a tier list admin with no grant gets 403.
    row(
        "PUT",
        "/api/admin/i18n/message",
        Gate::LocaleEdit,
        Doc::Drift(Claim::PanelOrGrant),
        "/api/admin/i18n/message",
        Some(r#"{"locale":"ja","key":"probe","value":"probe"}"#),
    ),
    row(
        "POST",
        "/api/admin/i18n/message/clear",
        Gate::LocaleEdit,
        Doc::Drift(Claim::PanelOrGrant),
        "/api/admin/i18n/message/clear",
        Some(r#"{"locale":"ja","key":"probe"}"#),
    ),
    row(
        "PUT",
        "/api/admin/i18n/overrides",
        Gate::LocaleEdit,
        Doc::Drift(Claim::PanelOrGrant),
        "/api/admin/i18n/overrides",
        Some(r#"{"locale":"ja","kind":"k","entity_id":"e","field":"f","value":"v"}"#),
    ),
    row(
        "POST",
        "/api/admin/i18n/overrides/delete",
        Gate::LocaleEdit,
        Doc::Drift(Claim::PanelOrGrant),
        "/api/admin/i18n/overrides/delete",
        Some(r#"{"locale":"ja","kind":"k","entity_id":"e","field":"f"}"#),
    ),
    row(
        "POST",
        "/api/admin/i18n/permissions",
        Gate::LocaleAdmin,
        Doc::Silent,
        "/api/admin/i18n/permissions",
        Some(GRANT_BODY),
    ),
    row(
        "POST",
        "/api/admin/i18n/permissions/revoke",
        Gate::LocaleAdmin,
        Doc::Silent,
        "/api/admin/i18n/permissions/revoke",
        Some(GRANT_BODY),
    ),
    // Under `/admin` but self-scoped.
    row(
        "GET",
        "/api/admin/i18n/permissions/me",
        Gate::SignedIn,
        Doc::Matches,
        "/api/admin/i18n/permissions/me",
        None,
    ),
    // Public on purpose or not, it is public: it answers without a token and,
    // since f77bc0bd, names every editor by nickname. Pinned so that changing
    // either is a visible decision (2026-10-09 audit, A10).
    row(
        "GET",
        "/api/operator-notes/{operator_id}/audit",
        Gate::Public,
        Doc::Silent,
        "/api/operator-notes/char_002_amiya/audit",
        None,
    ),
];

/// Role gates inside a service rather than the handler, which the source scan
/// cannot see. Listed by hand; `POST /tier-lists` is staff-only for an official
/// list and open to any account for a community one.
const SERVICE_GATES: &[Guarded] = &[
    row(
        "POST",
        "/api/tier-lists",
        Gate::Staff,
        Doc::Silent,
        "/api/tier-lists",
        Some(r#"{"name":"Guard probe","list_type":"official"}"#),
    ),
    row(
        "POST",
        "/api/tier-lists",
        Gate::SignedIn,
        Doc::Silent,
        "/api/tier-lists",
        Some(r#"{"name":"Guard probe","list_type":"community"}"#),
    ),
];

/// The phrase a correct annotation would carry for each gate. `None` where no
/// phrase the classifier knows expresses the gate.
const fn claim_for(gate: Gate) -> Option<Claim> {
    match gate {
        Gate::SuperAdmin => Some(Claim::SuperAdmin),
        Gate::TierListAdmin => Some(Claim::TierListAdmin),
        Gate::Staff => Some(Claim::Staff),
        Gate::PanelOrAnyGrant => Some(Claim::PanelOrGrant),
        Gate::SignedIn => Some(Claim::SignedIn),
        Gate::LocaleEdit | Gate::LocaleAdmin | Gate::Public => None,
    }
}

/// The calls that implement each gate in a handler body.
const fn guard_calls(gate: Gate) -> &'static [&'static str] {
    match gate {
        Gate::SuperAdmin => &["is_super_admin()", "require_super_admin("],
        Gate::TierListAdmin => &["is_tier_list_admin()"],
        Gate::Staff => &["is_any_admin_role()", "audit_actor_scope("],
        Gate::PanelOrAnyGrant => &["assert_can_read(", "service::can_access_admin_panel("],
        Gate::LocaleEdit => &["assert_can_write("],
        Gate::LocaleAdmin => &["assert_can_manage("],
        Gate::SignedIn | Gate::Public => &[],
    }
}

const ALL_GATES: &[Gate] = &[
    Gate::SuperAdmin,
    Gate::TierListAdmin,
    Gate::Staff,
    Gate::PanelOrAnyGrant,
    Gate::LocaleEdit,
    Gate::LocaleAdmin,
    Gate::SignedIn,
    Gate::Public,
];

fn every_guard_call() -> impl Iterator<Item = &'static str> {
    ALL_GATES
        .iter()
        .flat_map(|g| guard_calls(*g).iter().copied())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
    /// 401.
    Unauthorized,
    /// 403, decided on the role alone.
    Refused,
    /// Past the guard: anything but 401 or 403.
    Admitted,
    /// Only a grant row can decide, so the guard queries the database, which
    /// here is unreachable: a 5xx.
    AsksGrantTable,
}

/// Spelled out per role rather than through `GlobalRole`'s own predicates, so
/// the table states the rule independently instead of copying it.
const fn expected(gate: Gate, role: GlobalRole) -> Outcome {
    use GlobalRole as R;
    match (gate, role) {
        (Gate::Public | Gate::SignedIn, _)
        | (Gate::SuperAdmin | Gate::LocaleEdit | Gate::LocaleAdmin, R::SuperAdmin)
        | (Gate::TierListAdmin, R::TierListAdmin | R::SuperAdmin)
        | (Gate::Staff, R::TierListEditor | R::TierListAdmin | R::SuperAdmin)
        | (
            Gate::PanelOrAnyGrant,
            R::Translator | R::TierListEditor | R::TierListAdmin | R::SuperAdmin,
        ) => Outcome::Admitted,
        (Gate::PanelOrAnyGrant, R::User) | (Gate::LocaleEdit | Gate::LocaleAdmin, _) => {
            Outcome::AsksGrantTable
        }
        _ => Outcome::Refused,
    }
}

fn satisfies(outcome: Outcome, status: StatusCode) -> bool {
    match outcome {
        Outcome::Unauthorized => status == StatusCode::UNAUTHORIZED,
        Outcome::Refused => status == StatusCode::FORBIDDEN,
        Outcome::Admitted => status != StatusCode::UNAUTHORIZED && status != StatusCode::FORBIDDEN,
        Outcome::AsksGrantTable => status.is_server_error(),
    }
}

/// An `AppState` with no game data and a pool that cannot connect. The short
/// acquire timeout bounds how long a query keeps retrying the refused port.
fn state() -> AppState {
    let db = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(300))
        .connect_lazy(CLOSED_DB)
        .expect("lazy pool");
    let config = AppConfig {
        jwt_secret: JWT_SECRET.into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "admin-route-guards-service-key".into(),
        assets_base_dir: String::new(),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
    };
    AppState::new(
        db,
        CacheStore::new_memory(),
        HashMap::new(),
        Server::EN,
        config,
        reqwest::Client::new(),
        ServiceAccounts::default(),
    )
}

fn token(secret: &str, role: GlobalRole) -> String {
    create_token(secret, SUBJECT, "1", "en", &role.to_string(), 1).expect("token")
}

fn request(row: &Guarded, bearer: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(row.method).uri(row.uri);
    if let Some(t) = bearer {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    if row.body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    builder
        .body(row.body.map_or_else(Body::empty, Body::from))
        .expect("request builds")
}

/// `PathItem` holds one `Option<Operation>` per method, not a map.
fn operations(item: &PathItem) -> Vec<(&'static str, &Operation)> {
    [
        ("GET", &item.get),
        ("PUT", &item.put),
        ("POST", &item.post),
        ("DELETE", &item.delete),
        ("OPTIONS", &item.options),
        ("HEAD", &item.head),
        ("PATCH", &item.patch),
        ("TRACE", &item.trace),
    ]
    .into_iter()
    .filter_map(|(method, op)| op.as_ref().map(|op| (method, op)))
    .collect()
}

/// Every operation in the served document, keyed by method and path.
fn spec() -> BTreeMap<(String, String), Operation> {
    let (_router, api) = api_parts();
    let mut out = BTreeMap::new();
    for (path, item) in &api.paths.paths {
        for (method, op) in operations(item) {
            out.insert((method.to_owned(), path.clone()), op.clone());
        }
    }
    out
}

/// Every security alternative names a scheme: no `{}` making it optional.
fn requires_credential(op: &Operation) -> bool {
    op.security.as_ref().is_some_and(|reqs| {
        !reqs.is_empty()
            && reqs.iter().all(|req| {
                serde_json::to_value(req)
                    .ok()
                    .and_then(|v| v.as_object().map(|o| !o.is_empty()))
                    .unwrap_or(false)
            })
    })
}

/// Reads the who-may-call rule out of an operation's prose. Most specific
/// phrase first: the role-change route also names `Translator`.
fn documented_claim(op: &Operation) -> Option<Claim> {
    let text = format!(
        "{} {}",
        op.summary.as_deref().unwrap_or_default(),
        op.description.as_deref().unwrap_or_default()
    )
    .to_lowercase()
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ");
    let has = |phrase: &str| text.contains(phrase);
    if has("owner only") {
        Some(Claim::ListOwner)
    } else if has("super-admin only") || has("super-admin action") {
        Some(Claim::SuperAdmin)
    } else if has("tier list admin or super-admin") {
        Some(Claim::TierListAdmin)
    } else if has("tier list editor") {
        Some(Claim::Staff)
    } else if has("role that can open the admin panel") || has("may open the admin panel") {
        Some(Claim::PanelOrGrant)
    } else if has("any signed-in") {
        Some(Claim::SignedIn)
    } else {
        None
    }
}

fn is_fn_start(line: &str) -> bool {
    [
        "pub async fn ",
        "async fn ",
        "pub fn ",
        "pub(crate) async fn ",
    ]
    .iter()
    .any(|p| line.starts_with(p))
}

/// Every annotated handler's source, from its `fn` line to its closing brace,
/// keyed by method and the path as the document spells it. Handlers generated
/// by a macro carry no literal path and are skipped;
/// `every_admin_and_staff_route_has_a_row` checks that none of those needs a
/// credential.
fn handler_sources() -> BTreeMap<(String, String), String> {
    let mut out = BTreeMap::new();
    let mut queue = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/app/routes")];
    while let Some(dir) = queue.pop() {
        for entry in fs::read_dir(&dir).expect("routes dir is readable") {
            let path = entry.expect("readable dir entry").path();
            if path.is_dir() {
                queue.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let src = fs::read_to_string(&path).expect("route module is readable");
                collect_handlers(&src, &mut out);
            }
        }
    }
    out
}

fn collect_handlers(src: &str, out: &mut BTreeMap<(String, String), String>) {
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() != "#[utoipa::path(" {
            i += 1;
            continue;
        }
        let method = lines
            .get(i + 1)
            .map(|l| l.trim().trim_end_matches(',').to_uppercase());
        let mut j = i + 1;
        let mut path = None;
        while j < lines.len() && !lines[j].trim_start().starts_with(")]") {
            if path.is_none() {
                path = lines[j]
                    .trim()
                    .strip_prefix("path = \"")
                    .and_then(|rest| rest.split('"').next());
            }
            j += 1;
        }
        while j < lines.len() && !is_fn_start(lines[j]) {
            j += 1;
        }
        let start = j;
        while j < lines.len() && lines[j] != "}" {
            j += 1;
        }
        if let (Some(method), Some(path)) = (method, path) {
            let end = j.min(lines.len().saturating_sub(1));
            out.insert(
                (method, format!("/api{path}")),
                lines[start..=end].join("\n"),
            );
        }
        i = j + 1;
    }
}

fn key(row: &Guarded) -> (String, String) {
    (row.method.to_owned(), row.path.to_owned())
}

#[test]
fn every_admin_and_staff_route_has_a_row() {
    let spec = spec();
    let handlers = handler_sources();

    // The scan must see every operation that needs a credential, or a guarded
    // route could hide from the derivation below.
    let unscanned: Vec<_> = spec
        .iter()
        .filter(|(k, op)| !handlers.contains_key(*k) && requires_credential(op))
        .map(|(k, _)| k.clone())
        .collect();
    assert!(
        unscanned.is_empty(),
        "credentialed operations whose handler the source scan cannot find: {unscanned:#?}"
    );

    let derived: BTreeSet<(String, String)> = spec
        .keys()
        .filter(|(_, path)| path.starts_with("/api/admin/"))
        .chain(handlers.iter().filter_map(|(k, src)| {
            every_guard_call()
                .any(|call| src.contains(call))
                .then_some(k)
        }))
        .cloned()
        .collect();
    let listed: BTreeSet<(String, String)> = GUARDED
        .iter()
        .filter(|r| r.gate != Gate::Public)
        .map(key)
        .collect();

    let missing: Vec<_> = derived.difference(&listed).collect();
    let extra: Vec<_> = listed.difference(&derived).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "GUARDED is out of step with the router.\n\
         Guarded or under /api/admin but no row: {missing:#?}\n\
         A row but neither: {extra:#?}"
    );
    assert_eq!(listed.len(), GUARDED.len() - 1, "a route is listed twice");
}

#[test]
fn each_handler_calls_the_guard_its_gate_names() {
    let handlers = handler_sources();
    let mut wrong = Vec::new();

    for row in GUARDED {
        let src = handlers
            .get(&key(row))
            .unwrap_or_else(|| panic!("{} {}: no handler found", row.method, row.path));
        let own = guard_calls(row.gate);
        let calls_own = own.is_empty() || own.iter().any(|c| src.contains(c));
        let foreign: Vec<_> = every_guard_call()
            .filter(|c| !own.contains(c) && src.contains(c))
            .collect();
        if !calls_own || !foreign.is_empty() {
            wrong.push(format!(
                "{} {}: gate {:?} expects one of {own:?}, foreign guards {foreign:?}",
                row.method, row.path, row.gate
            ));
        }
        if row.gate == Gate::LocaleEdit && !src.contains("Permission::Edit") {
            wrong.push(format!(
                "{} {}: not checked at Permission::Edit",
                row.method, row.path
            ));
        }
        if row.gate == Gate::Public && src.contains("AuthUser") {
            wrong.push(format!("{} {}: reads a credential", row.method, row.path));
        }
    }

    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn each_annotation_declares_the_credential_and_the_rule() {
    let spec = spec();
    let mut wrong = Vec::new();

    for row in GUARDED {
        let op = spec
            .get(&key(row))
            .unwrap_or_else(|| panic!("{} {} is not in the document", row.method, row.path));
        let label = format!("{} {}", row.method, row.path);
        let documents = |code: &str| op.responses.responses.contains_key(code);

        if row.gate == Gate::Public {
            if op.security.as_ref().is_some_and(|s| !s.is_empty()) || documents("401") {
                wrong.push(format!(
                    "{label}: public, but the document asks for a credential"
                ));
            }
        } else {
            if !requires_credential(op) || !documents("401") {
                wrong.push(format!("{label}: needs a required credential and a 401"));
            }
            if row.gate != Gate::SignedIn && !documents("403") {
                wrong.push(format!("{label}: refuses roles but documents no 403"));
            }
        }

        let said = documented_claim(op);
        let ok = match row.doc {
            Doc::Matches => said.is_some() && said == claim_for(row.gate),
            Doc::Silent => said.is_none(),
            Doc::Drift(claim) => said == Some(claim) && said != claim_for(row.gate),
        };
        if !ok {
            wrong.push(format!(
                "{label}: the row says {:?} for gate {:?}, the prose reads as {said:?}",
                row.doc, row.gate
            ));
        }
    }

    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[tokio::test]
async fn routes_admit_exactly_the_roles_their_gate_names() {
    let app: Router = api_parts().0.with_state(state());
    let forged = token("not-the-server-secret", GlobalRole::SuperAdmin);

    let mut probes = Vec::new();
    for row in GUARDED.iter().chain(SERVICE_GATES) {
        let anonymous = if row.gate == Gate::Public {
            Outcome::Admitted
        } else {
            Outcome::Unauthorized
        };
        probes.push(("no token".to_owned(), row, None, anonymous));
        probes.push((
            "forged token".to_owned(),
            row,
            Some(forged.clone()),
            anonymous,
        ));
        for role in GlobalRole::all() {
            probes.push((
                role.to_string(),
                row,
                Some(token(JWT_SECRET, *role)),
                expected(row.gate, *role),
            ));
        }
    }

    let results = join_all(probes.into_iter().map(|(who, row, bearer, want)| {
        let app = app.clone();
        async move {
            let response = app
                .oneshot(request(row, bearer.as_deref()))
                .await
                .expect("the router is infallible");
            (who, row, want, response.status())
        }
    }))
    .await;

    let total = results.len();
    let wrong: Vec<String> = results
        .into_iter()
        .filter(|(_, _, want, status)| !satisfies(*want, *status))
        .map(|(who, row, want, status)| {
            format!(
                "{} {} as {who}: expected {want:?}, got {status}",
                row.method, row.uri
            )
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "{} of {total} probes: {wrong:#?}",
        wrong.len()
    );
}

/// The router cannot tell "admitted by role" from "asked the grant table" with
/// no database, since both end in a failed query. The guards can: a role
/// that suffices returns before any query.
#[tokio::test]
async fn translation_guards_admit_by_role_only_where_expected() {
    let state = state();

    for role in GlobalRole::all() {
        let auth = AuthUser {
            user_id: SUBJECT.to_owned(),
            uid: "1".to_owned(),
            server: "en".to_owned(),
            role: *role,
        };

        let write = i18n::assert_can_write(&state, &auth, "ja", Permission::Edit).await;
        let write_ok = if *role == GlobalRole::SuperAdmin {
            write.is_ok()
        } else {
            matches!(
                write,
                Err(ApiError::ServiceUnavailable | ApiError::Internal(_))
            )
        };
        assert!(write_ok, "assert_can_write as {role}: {write:?}");

        let read = i18n::can_access_admin_panel(&state, &auth).await;
        let read_ok = if *role == GlobalRole::User {
            matches!(
                read,
                Err(ApiError::ServiceUnavailable | ApiError::Internal(_))
            )
        } else {
            matches!(read, Ok(true))
        };
        assert!(read_ok, "can_access_admin_panel as {role}: {read:?}");
    }
}
