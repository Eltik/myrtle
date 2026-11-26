#![allow(clippy::unwrap_used)]

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use myrtle_app::services::oauth::error::{OAuthErrorCode, ProtocolError};
use myrtle_app::services::oauth::protocol::{
    MAX_REDIRECT_URI_LEN, Scope, SecretKind, api_base, avatar_stem, bearer_token,
    client_credentials, digest, is_valid_challenge, join_scopes, matches_digest, mint,
    parse_scopes, redirect_host, redirect_with, stored_scopes, validate_redirect_uri, verify_pkce,
};

#[test]
fn a_minted_secret_carries_its_prefix_and_256_bits() {
    for kind in [
        SecretKind::ClientSecret,
        SecretKind::Code,
        SecretKind::AccessToken,
        SecretKind::RefreshToken,
    ] {
        let secret = mint(kind);
        let body = secret.strip_prefix(kind.prefix()).unwrap();
        assert_eq!(URL_SAFE_NO_PAD.decode(body).unwrap().len(), 32);
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
    assert_eq!(
        parse_scopes(Some("Identify")),
        Err("Identify".to_owned()),
        "scope names are case-sensitive (RFC 6749 3.3)"
    );
}

#[test]
fn a_scope_set_survives_storage() {
    let scopes = parse_scopes(Some("identify")).unwrap();
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

/// RFC 7636 appendix B
#[test]
fn pkce_matches_the_rfc_example() {
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    assert!(is_valid_challenge(challenge));
    assert!(verify_pkce(verifier, challenge));
    assert!(!verify_pkce(verifier, &challenge.replace('E', "F")));
    assert!(
        !verify_pkce(challenge, challenge),
        "plain is not a method: the challenge is never its own verifier"
    );
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

/// The same cases as `toAvatarStem` in the web client's `lib/utils.ts`
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

#[test]
fn the_api_base_is_the_configured_one_else_the_request_host() {
    assert_eq!(api_base("", None, None), "https://api.myrtle.moe");
    assert_eq!(
        api_base("", Some("localhost:3060"), None),
        "http://localhost:3060"
    );
    assert_eq!(
        api_base("", Some("localhost:3060"), Some("https")),
        "https://localhost:3060"
    );
    assert_eq!(
        api_base(
            " https://api.example.com/ ",
            Some("localhost:3060"),
            Some("https")
        ),
        "https://api.example.com"
    );
}

fn basic(raw: &str) -> String {
    format!("Basic {}", STANDARD.encode(raw))
}

fn refused(result: Result<(String, String), ProtocolError>) -> OAuthErrorCode {
    result.unwrap_err().code
}

#[test]
fn credentials_come_from_basic_or_from_the_body() {
    let pair = ("app".to_owned(), "secret".to_owned());
    assert_eq!(
        client_credentials(Some(&basic("app:secret")), None, None).ok(),
        Some(pair.clone())
    );
    assert_eq!(
        client_credentials(None, Some("app"), Some("secret")).ok(),
        Some(pair.clone())
    );
    assert_eq!(
        client_credentials(Some(&basic("app:secret")), Some("app"), Some("other")).ok(),
        Some(pair),
        "Basic wins, and a matching client_id in the body is harmless"
    );
}

#[test]
fn basic_credentials_are_form_decoded_and_split_at_the_first_colon() {
    assert_eq!(
        client_credentials(Some(&basic("a%3Ab:s%20e:c")), None, None).ok(),
        Some(("a:b".to_owned(), "s e:c".to_owned()))
    );
    assert_eq!(
        client_credentials(
            Some(&format!("bAsIc {}", STANDARD.encode("a:b"))),
            None,
            None
        )
        .ok(),
        Some(("a".to_owned(), "b".to_owned())),
        "the scheme is case-insensitive"
    );
}

#[test]
fn missing_or_mangled_credentials_are_invalid_client() {
    for (header, id) in [
        (None, None),
        (None, Some("app")),
        (Some(basic("no-colon")), None),
        (Some("Basic !!!not-base64!!!".to_owned()), None),
    ] {
        assert_eq!(
            refused(client_credentials(header.as_deref(), id, None)),
            OAuthErrorCode::InvalidClient,
            "{header:?} {id:?}"
        );
    }
}

#[test]
fn two_different_client_ids_in_one_request_are_refused() {
    assert_eq!(
        refused(client_credentials(
            Some(&basic("app:secret")),
            Some("other"),
            None
        )),
        OAuthErrorCode::InvalidRequest
    );
}

#[test]
fn a_bearer_token_needs_the_bearer_scheme_and_a_value() {
    assert_eq!(bearer_token(Some("Bearer abc")), Some("abc"));
    assert_eq!(bearer_token(Some("bearer  abc ")), Some("abc"));
    assert_eq!(bearer_token(Some("Bearer ")), None);
    assert_eq!(bearer_token(Some("Basic abc")), None);
    assert_eq!(bearer_token(Some("abc")), None);
    assert_eq!(bearer_token(None), None);
}
