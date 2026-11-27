use myrtle_parity::corpus::{CorpusEntry, Source};
use myrtle_parity::http::{HttpOptions, Secret, Side, Target, Tokens, token_env_names};

#[test]
fn secret_debug_is_redacted() {
    let s = Secret::new("hunter2".to_owned());
    assert_eq!(format!("{s:?}"), "Secret(<redacted>)");
    let t = Tokens::from_lookup(["user"], None, |_| Some("hunter2".to_owned()));
    assert!(!format!("{t:?}").contains("hunter2"));
}

#[test]
fn env_names_per_side() {
    assert_eq!(token_env_names("user", None), ["PARITY_TOKEN_USER"]);
    assert_eq!(
        token_env_names("super-admin", Some(Side::B)),
        ["PARITY_B_TOKEN_SUPER_ADMIN", "PARITY_TOKEN_SUPER_ADMIN"]
    );
}

#[test]
fn side_specific_token_wins_and_empty_is_unset() {
    let env = |name: &str| match name {
        "PARITY_A_TOKEN_USER" => Some("a-tok".to_owned()),
        "PARITY_TOKEN_USER" => Some("shared".to_owned()),
        "PARITY_TOKEN_ADMIN" => Some("   ".to_owned()),
        _ => None,
    };
    let a = Tokens::from_lookup(["user", "admin"], Some(Side::A), env);
    let b = Tokens::from_lookup(["user", "admin"], Some(Side::B), env);
    assert_eq!(a.get("user").map(Secret::expose), Some("a-tok"));
    assert_eq!(b.get("user").map(Secret::expose), Some("shared"));
    assert!(a.get("admin").is_none());
}

#[test]
fn rejects_non_http_base() {
    assert!(
        Target::new(
            "localhost:3060",
            None,
            Tokens::default(),
            &HttpOptions::default()
        )
        .is_err()
    );
    let t = Target::new(
        "http://localhost:3060/",
        None,
        Tokens::default(),
        &HttpOptions::default(),
    )
    .unwrap();
    assert_eq!(t.label, "http://localhost:3060");
}

#[tokio::test]
async fn missing_token_is_reported_without_a_request() {
    let t = Target::new(
        "http://127.0.0.1:9",
        None,
        Tokens::default(),
        &HttpOptions::default(),
    )
    .unwrap();
    let e = CorpusEntry::new("/api/x", "/api/x", Some("admin"), Source::Openapi);
    let err = t.fetch(&e).await.unwrap_err();
    assert_eq!(
        err.to_string(),
        "no token for role `admin` (set PARITY_TOKEN_ADMIN)"
    );
}
