//! The rules of the OAuth 2.0 provider that need no database: what a secret
//! looks like and how it is checked, which scopes exist, which redirect URIs
//! an application may register, and how a PKCE verifier is matched.
//!
//! "Sign in with myrtle.moe" is the authorization-code grant of RFC 6749 with
//! PKCE (RFC 7636). The services in `app::services::oauth` do the storage;
//! everything they decide on is decided here, so it can be tested without one.
//!
//! # Secrets
//!
//! Client secrets, authorization codes, access tokens and refresh tokens are
//! each 32 bytes from the OS generator, written as unpadded base64url behind a
//! prefix that says what the value is (`myrtle_at_...` is an access token).
//! The prefix carries no authority. It is there so a leaked value can be
//! recognised for what it is, by a person or by a secret scanner.
//!
//! Only [`digest`] of a secret is ever stored. An OAuth access token is
//! deliberately NOT a site JWT: it is opaque, lives in its own table, and the
//! only extractor that accepts it is the one in front of `/oauth/userinfo`. A
//! third party holding one can read an identity and nothing else; it cannot
//! reach the routes a player's own session reaches (settings, account
//! deletion, a roster sync).

use std::fmt;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::Url;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::utils::random::fill_random;

/// How long an authorization code may wait to be exchanged. RFC 6749 4.1.2
/// recommends ten minutes at most; the exchange is one server-to-server call
/// made as the browser lands, so five is generous.
pub const CODE_TTL_SECS: i64 = 5 * 60;

/// Lifetime of an access token. A week, so a site that only wants to know who
/// signed in never has to implement refresh.
pub const ACCESS_TTL_SECS: i64 = 7 * 24 * 60 * 60;

/// Lifetime of a refresh token, restarted by every refresh. An application
/// nobody has used for this long has to send the player through consent again.
pub const REFRESH_TTL_SECS: i64 = 90 * 24 * 60 * 60;

/// Applications one account may register.
pub const MAX_APPS_PER_USER: i64 = 10;

/// Redirect URIs one application may register.
pub const MAX_REDIRECT_URIS: usize = 10;

/// Live sign-ins kept per player per application. One row is one device or
/// browser signed in to the third-party site; past this the oldest goes.
pub const MAX_SESSIONS_PER_GRANT: i64 = 20;

pub const MAX_NAME_LEN: usize = 64;
pub const MAX_DESCRIPTION_LEN: usize = 280;
const MAX_REDIRECT_URI_LEN: usize = 512;

/// The longest `state` echoed back. It is the client's own value and is never
/// stored, so the bound only keeps a redirect URL a sane length.
pub const MAX_STATE_LEN: usize = 1024;

/// What a generated secret is, which decides its prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretKind {
    ClientSecret,
    Code,
    AccessToken,
    RefreshToken,
}

impl SecretKind {
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::ClientSecret => "myrtle_cs_",
            Self::Code => "myrtle_ac_",
            Self::AccessToken => "myrtle_at_",
            Self::RefreshToken => "myrtle_rt_",
        }
    }
}

/// A new secret of `kind`: its prefix, then 32 random bytes as base64url.
pub fn mint(kind: SecretKind) -> String {
    let mut bytes = [0u8; 32];
    fill_random(&mut bytes);
    format!("{}{}", kind.prefix(), URL_SAFE_NO_PAD.encode(bytes))
}

/// What is stored in place of a secret, and what a presented one is hashed to
/// before it is looked up.
pub fn digest(secret: &str) -> Vec<u8> {
    Sha256::digest(secret.as_bytes()).to_vec()
}

/// Whether `presented` is the secret `stored` is the digest of, in constant
/// time. Used where the row is found by something else (a client by its id),
/// so the comparison is the check; a token found BY its digest needs none.
pub fn matches_digest(presented: &str, stored: &[u8]) -> bool {
    digest(presented).ct_eq(stored).into()
}

/// One permission an application can ask a player for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scope {
    /// Who the player is: account id, nickname and tag, game uid and server,
    /// and avatar. Everything `/oauth/userinfo` returns.
    Identify,
}

impl Scope {
    pub const ALL: [Self; 1] = [Self::Identify];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Identify => "identify",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.as_str() == raw)
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The scopes a `scope` parameter names: space-separated, sorted, each once.
/// An absent or blank parameter is the default set (`identify`), as RFC 6749
/// 3.3 allows. `Err` carries the first name that is not a scope.
pub fn parse_scopes(raw: Option<&str>) -> Result<Vec<Scope>, String> {
    let mut scopes = Vec::new();
    for name in raw.unwrap_or_default().split_ascii_whitespace() {
        let scope = Scope::parse(name).ok_or_else(|| name.to_owned())?;
        if !scopes.contains(&scope) {
            scopes.push(scope);
        }
    }
    if scopes.is_empty() {
        scopes.push(Scope::Identify);
    }
    scopes.sort_unstable();
    Ok(scopes)
}

/// The stored and wire form of a scope set.
pub fn join_scopes(scopes: &[Scope]) -> String {
    scopes
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A stored scope string back as a set. A name this build no longer knows is
/// dropped: a token keeps what it still validly holds.
pub fn stored_scopes(stored: &str) -> Vec<Scope> {
    stored
        .split_ascii_whitespace()
        .filter_map(Scope::parse)
        .collect()
}

/// Whether an application may register `uri` as a place to send players back
/// to. `Err` is the reason, worded for the developer.
///
/// - Absolute `https`, or `http` only for a loopback host (local development).
///   Anything else would hand the code to whoever can read the network.
/// - No fragment (RFC 6749 3.1.2) and no `user:password@`.
/// - Used exactly as written: the authorization request must repeat one
///   registered URI character for character, so nothing here normalizes.
pub fn validate_redirect_uri(uri: &str) -> Result<(), String> {
    if uri.len() > MAX_REDIRECT_URI_LEN {
        return Err(format!("is longer than {MAX_REDIRECT_URI_LEN} characters"));
    }
    if uri.trim() != uri || uri.chars().any(char::is_control) {
        return Err("must not contain whitespace or control characters".to_owned());
    }
    let url = Url::parse(uri).map_err(|_| "is not an absolute URL".to_owned())?;
    let Some(host) = url.host_str() else {
        return Err("has no host".to_owned());
    };
    match url.scheme() {
        "https" => {}
        "http" if is_loopback_host(host) => {}
        "http" => {
            return Err(
                "must use https (http is allowed only for localhost, 127.0.0.1 and [::1])"
                    .to_owned(),
            );
        }
        _ => return Err("must be an https URL".to_owned()),
    }
    if url.fragment().is_some() {
        return Err("must not contain a fragment (#...)".to_owned());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("must not contain credentials".to_owned());
    }
    Ok(())
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "[::1]")
}

/// `redirect_uri` with `params` added to its query, for sending the browser
/// back to the application. A query the registered URI already has is kept.
/// `None` only for a URI that does not parse, which registration refuses.
pub fn redirect_with(redirect_uri: &str, params: &[(&str, &str)]) -> Option<String> {
    let mut url = Url::parse(redirect_uri).ok()?;
    {
        let mut query = url.query_pairs_mut();
        for (key, value) in params {
            query.append_pair(key, value);
        }
    }
    Some(url.into())
}

/// The host a consent screen names as where the player is being sent.
pub fn redirect_host(redirect_uri: &str) -> Option<String> {
    let url = Url::parse(redirect_uri).ok()?;
    let host = url.host_str()?;
    Some(match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_owned(),
    })
}

/// Whether `challenge` is a well-formed S256 code challenge: the unpadded
/// base64url of a SHA-256, so exactly 43 characters of that alphabet.
pub fn is_valid_challenge(challenge: &str) -> bool {
    challenge.len() == 43 && challenge.bytes().all(is_base64url)
}

/// Whether `verifier` is a well-formed code verifier (RFC 7636 4.1): 43 to 128
/// characters from the unreserved set.
pub fn is_valid_verifier(verifier: &str) -> bool {
    (43..=128).contains(&verifier.len())
        && verifier
            .bytes()
            .all(|b| is_base64url(b) || b == b'.' || b == b'~')
}

const fn is_base64url(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

/// Whether `verifier` is the secret behind `challenge` under S256, the only
/// method accepted: `plain` would let anyone who saw the authorization request
/// redeem the code.
pub fn verify_pkce(verifier: &str, challenge: &str) -> bool {
    if !is_valid_verifier(verifier) {
        return false;
    }
    let computed = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    computed.as_bytes().ct_eq(challenge.as_bytes()).into()
}

/// The path segment the avatar route serves `avatar_id` under, as the web
/// client's `toAvatarStem` writes it: a skin id's `@` becomes `_`, a base art
/// `#1` is dropped and `#2` becomes `_2`, then the whole is percent-encoded.
pub fn avatar_stem(avatar_id: &str) -> String {
    let stem = if avatar_id.contains('@') {
        avatar_id.replace('@', "_")
    } else if let Some(base) = avatar_id.strip_suffix("#1") {
        base.to_owned()
    } else {
        match avatar_id.rsplit_once('#') {
            Some((base, suffix)) if is_art_suffix(suffix) => format!("{base}_{suffix}"),
            _ => avatar_id.to_owned(),
        }
    };
    urlencoding::encode(&stem).into_owned()
}

/// `2`, `1+`: digits with an optional trailing plus.
fn is_art_suffix(suffix: &str) -> bool {
    let digits = suffix.strip_suffix('+').unwrap_or(suffix);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_minted_secret_carries_its_prefix_and_256_bits() {
        for kind in [
            SecretKind::ClientSecret,
            SecretKind::Code,
            SecretKind::AccessToken,
            SecretKind::RefreshToken,
        ] {
            let secret = mint(kind);
            let body = secret.strip_prefix(kind.prefix()).expect("prefix");
            assert_eq!(URL_SAFE_NO_PAD.decode(body).expect("base64url").len(), 32);
            assert_ne!(secret, mint(kind), "two mints of {kind:?} collide");
        }
    }

    #[test]
    fn only_the_secret_itself_matches_its_digest() {
        let secret = mint(SecretKind::ClientSecret);
        let stored = digest(&secret);
        assert!(matches_digest(&secret, &stored));
        assert!(!matches_digest(&format!("{secret}x"), &stored));
        assert!(!matches_digest("", &stored));
        assert!(!matches_digest(&secret, &[]));
    }

    #[test]
    fn scopes_default_to_identify_and_refuse_unknown_names() {
        assert_eq!(parse_scopes(None), Ok(vec![Scope::Identify]));
        assert_eq!(parse_scopes(Some("  ")), Ok(vec![Scope::Identify]));
        assert_eq!(
            parse_scopes(Some("identify identify")),
            Ok(vec![Scope::Identify])
        );
        assert_eq!(
            parse_scopes(Some("identify roster")),
            Err("roster".to_owned())
        );
        // Scope names are case-sensitive (RFC 6749 3.3).
        assert_eq!(parse_scopes(Some("Identify")), Err("Identify".to_owned()));
    }

    #[test]
    fn a_scope_set_survives_storage() {
        let scopes = parse_scopes(Some("identify")).expect("valid");
        assert_eq!(stored_scopes(&join_scopes(&scopes)), scopes);
        assert_eq!(stored_scopes("identify retired"), vec![Scope::Identify]);
    }

    #[test]
    fn redirect_uris_are_https_or_loopback_http() {
        for ok in [
            "https://miowawa.de/auth/myrtle/callback",
            "https://example.com/cb?tenant=1",
            "https://example.com:8443/cb",
            "http://localhost:4000/callback",
            "http://127.0.0.1/callback",
            "http://[::1]:4000/callback",
        ] {
            assert_eq!(validate_redirect_uri(ok), Ok(()), "{ok}");
        }
        for bad in [
            "http://example.com/callback",
            "http://localhost.example.com/callback",
            "https://example.com/cb#fragment",
            "https://user:pw@example.com/cb",
            "https://user@example.com/cb",
            "javascript:alert(1)",
            "myapp://callback",
            "/relative/callback",
            "example.com/callback",
            " https://example.com/cb",
            "https://example.com/c\nb",
            "",
        ] {
            assert!(validate_redirect_uri(bad).is_err(), "{bad:?} was accepted");
        }
        let long = format!("https://example.com/{}", "a".repeat(MAX_REDIRECT_URI_LEN));
        assert!(validate_redirect_uri(&long).is_err());
    }

    #[test]
    fn redirect_params_join_an_existing_query_and_are_encoded() {
        assert_eq!(
            redirect_with(
                "https://example.com/cb",
                &[("code", "a b"), ("state", "x&y")]
            )
            .as_deref(),
            Some("https://example.com/cb?code=a+b&state=x%26y")
        );
        assert_eq!(
            redirect_with("https://example.com/cb?tenant=1", &[("code", "c")]).as_deref(),
            Some("https://example.com/cb?tenant=1&code=c")
        );
        assert_eq!(redirect_with("not a url", &[("code", "c")]), None);
    }

    #[test]
    fn the_redirect_host_keeps_a_non_default_port() {
        assert_eq!(
            redirect_host("https://miowawa.de/cb").as_deref(),
            Some("miowawa.de")
        );
        assert_eq!(
            redirect_host("http://localhost:4000/cb").as_deref(),
            Some("localhost:4000")
        );
    }

    /// The worked example of RFC 7636 appendix B.
    #[test]
    fn pkce_matches_the_rfc_example() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        assert!(is_valid_challenge(challenge));
        assert!(verify_pkce(verifier, challenge));
        assert!(!verify_pkce(verifier, &challenge.replace('E', "F")));
        // `plain` is not a method here: the challenge is never its own verifier.
        assert!(!verify_pkce(challenge, challenge));
        assert!(!verify_pkce("short", challenge));
    }

    #[test]
    fn a_challenge_is_43_base64url_characters() {
        assert!(!is_valid_challenge(""));
        assert!(!is_valid_challenge(&"a".repeat(42)));
        assert!(!is_valid_challenge(&"a".repeat(44)));
        assert!(!is_valid_challenge(&format!("{}=", "a".repeat(42))));
        assert!(is_valid_challenge(&"a".repeat(43)));
    }

    /// The same cases as `toAvatarStem` in the web client's `lib/utils.ts`.
    #[test]
    fn the_avatar_stem_matches_the_web_client() {
        assert_eq!(avatar_stem("char_002_amiya"), "char_002_amiya");
        assert_eq!(avatar_stem("char_002_amiya#1"), "char_002_amiya");
        assert_eq!(avatar_stem("char_002_amiya#2"), "char_002_amiya_2");
        assert_eq!(avatar_stem("char_002_amiya#1+"), "char_002_amiya_1%2B");
        assert_eq!(
            avatar_stem("char_002_amiya@winter#1"),
            "char_002_amiya_winter%231"
        );
    }
}
