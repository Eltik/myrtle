use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use subtle::ConstantTimeEq;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::auth::jwt::verify_token;
use crate::core::auth::permissions::GlobalRole;
use crate::database::queries;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub uid: String,
    pub server: String,
    /// The role claim as minted at login, up to 7 days stale. Gates read
    /// [`RoleCheckedUser::role`] instead; only `/auth/verify` reads this, as
    /// the fallback when the account row is gone.
    pub token_role: GlobalRole,
}

impl AuthUser {
    /// The subject as a `Uuid`; `Unauthorized` for a non-UUID subject such as the
    /// internal "service" principal.
    pub fn user_uuid(&self) -> Result<uuid::Uuid, ApiError> {
        self.user_id.parse().map_err(|_| ApiError::Unauthorized)
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    // `async fn` is the idiomatic spelling of this trait method even though nothing awaits.
    #[allow(clippy::unused_async_trait_impl)]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Service key first (internal SSR calls), compared in constant time.
        // Both sides must be non-empty: `ct_eq` on two empty slices is equal, so an
        // empty key would match an empty header. `AppConfig::require_secret` also
        // refuses an empty key at boot.
        if let Some(key) = parts.headers.get("x-service-key")
            && !key.is_empty()
            && !state.config.service_key.is_empty()
            && key
                .as_bytes()
                .ct_eq(state.config.service_key.as_bytes())
                .into()
        {
            return Ok(Self {
                user_id: "service".to_owned(),
                uid: "service".to_owned(),
                server: "internal".to_owned(),
                token_role: GlobalRole::SuperAdmin,
            });
        }

        let token = extract_bearer(&parts.headers)?;
        let claims = verify_token(&state.config.jwt_secret, token)?;

        let token_role = parse_role(&claims.role);

        Ok(Self {
            user_id: claims.sub,
            uid: claims.uid,
            server: claims.server,
            token_role,
        })
    }
}

pub struct MaybeAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for MaybeAuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(Self(Some(user))),
            Err(_) => Ok(Self(None)),
        }
    }
}

/// An [`AuthUser`] whose role has been checked against the database. Every
/// handler that decides anything on the caller's role takes this, never a bare
/// `AuthUser`.
///
/// The token's role claim freezes at login for 7 days, and `set_user_role`
/// changes only the row, so a gate reading the claim keeps a demoted admin's
/// access until the token expires. [`Self::role`] is the LOWER of the claim and
/// the stored role: a demotion takes effect on the next request, a promotion
/// still waits for the next login as it always has, and a token never acts
/// above the role it was minted with. An account that no longer exists is 401.
///
/// Cost: one primary-key read per request, and only for a claim above `user`,
/// since the lower of `user` and anything is `user`; anonymous callers and
/// plain accounts pay nothing. `DISABLE_ROLE_RECHECK=1` skips the read and
/// restores the claim-only behaviour exactly.
#[derive(Debug, Clone)]
pub struct RoleCheckedUser {
    user: AuthUser,
    /// The role every gate acts on.
    pub role: GlobalRole,
}

impl RoleCheckedUser {
    /// A caller whose role a unit test asserts was already checked.
    #[cfg(test)]
    pub(crate) const fn checked_for_test(user: AuthUser, role: GlobalRole) -> Self {
        Self { user, role }
    }
}

impl Deref for RoleCheckedUser {
    type Target = AuthUser;

    fn deref(&self) -> &AuthUser {
        &self.user
    }
}

impl FromRequestParts<AppState> for RoleCheckedUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        let role = checked_role(state, &user).await?;
        Ok(Self { user, role })
    }
}

/// [`MaybeAuthUser`] for a public route whose answer depends on the role.
/// Anything that would refuse the role check, a deleted account or an
/// unreachable database included, reads as anonymous: the least privilege.
pub struct MaybeRoleCheckedUser(pub Option<RoleCheckedUser>);

impl FromRequestParts<AppState> for MaybeRoleCheckedUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Ok(user) = AuthUser::from_request_parts(parts, state).await else {
            return Ok(Self(None));
        };
        let checked = checked_role(state, &user)
            .await
            .ok()
            .map(|role| RoleCheckedUser { user, role });
        Ok(Self(checked))
    }
}

async fn checked_role(state: &AppState, user: &AuthUser) -> Result<GlobalRole, ApiError> {
    if !state.config.role_recheck || user.token_role == GlobalRole::User {
        return Ok(user.token_role);
    }
    // Only the service principal has a non-UUID subject (login always signs
    // the account id), and it has no row: its key is its authority.
    let Ok(id) = user.user_uuid() else {
        return Ok(user.token_role);
    };
    let stored = queries::users::find_role(&state.db, id).await?;
    resolve_role(user.token_role, stored.as_deref())
}

/// The role a gate acts on, from the token's claim and the stored role
/// (`None`: the account is gone, which is 401 like a bad token). An unknown
/// stored spelling reads as `user`, as it does in a token.
fn resolve_role(token: GlobalRole, stored: Option<&str>) -> Result<GlobalRole, ApiError> {
    stored
        .map(|s| token.lower_of(parse_role(s)))
        .ok_or(ApiError::Unauthorized)
}

fn extract_bearer(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthorized)
}

/// Unknown or absent role degrades to `User`: a token minted before the role
/// existed still authenticates, minus the privilege. `GlobalRole::from_str` owns
/// the spellings so this can't drift from `Display`.
fn parse_role(role: &str) -> GlobalRole {
    GlobalRole::from_str(role).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lowest first, written out here rather than read from `GlobalRole`, so
    /// the expectations do not copy the code under test.
    const CHAIN: [GlobalRole; 5] = [
        GlobalRole::User,
        GlobalRole::Translator,
        GlobalRole::TierListEditor,
        GlobalRole::TierListAdmin,
        GlobalRole::SuperAdmin,
    ];

    #[test]
    fn a_gate_acts_on_the_lower_of_the_token_and_the_stored_role() {
        for (ti, &token) in CHAIN.iter().enumerate() {
            for (si, &stored) in CHAIN.iter().enumerate() {
                let got = resolve_role(token, Some(&stored.to_string())).expect("row exists");
                assert_eq!(got, CHAIN[ti.min(si)], "token {token}, stored {stored}");
            }
        }
    }

    #[test]
    fn a_missing_account_is_unauthorized_whatever_the_token_says() {
        for token in CHAIN {
            assert!(
                matches!(resolve_role(token, None), Err(ApiError::Unauthorized)),
                "token {token}"
            );
        }
    }

    #[test]
    fn an_unknown_stored_role_reads_as_user() {
        for token in CHAIN {
            assert_eq!(
                resolve_role(token, Some("moderator")).expect("row exists"),
                GlobalRole::User
            );
        }
    }
}
