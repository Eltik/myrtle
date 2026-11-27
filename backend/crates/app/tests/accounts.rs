#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::AppError;
use myrtle_app::services::accounts::account::verify;
use myrtle_app::services::accounts::credentials::{CredentialError, CredentialKey};
use myrtle_app::services::accounts::jwt::{
    Claims, TOKEN_DAYS, create_token, parse_role, role_name, verify_token,
};
use myrtle_types::{Role, Server, UserId};
use uuid::Uuid;

const SECRET: &str = "0123456789abcdef0123456789abcdef";

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn key() -> CredentialKey {
    CredentialKey::parse(&"ab".repeat(32)).unwrap()
}

#[test]
fn a_token_carries_its_claims_back() {
    let user = UserId(Uuid::from_u128(42));
    let issued = now();
    let token = create_token(
        SECRET,
        user,
        "12345678",
        Server::Bilibili,
        Role::TierListAdmin,
        issued,
    )
    .unwrap();
    let claims = verify_token(SECRET, &token).unwrap();
    assert_eq!(claims.user_id().unwrap(), user);
    assert_eq!(claims.uid, "12345678");
    assert_eq!(claims.server(), Some(Server::Bilibili));
    assert_eq!(parse_role(&claims.role), Role::TierListAdmin);
    assert_eq!(claims.exp - claims.iat, TOKEN_DAYS * 86_400);
}

#[test]
fn a_token_fails_under_another_secret_or_after_expiry() {
    let user = UserId(Uuid::from_u128(1));
    let token = create_token(SECRET, user, "1", Server::EN, Role::User, now()).unwrap();
    assert!(matches!(
        verify_token("another-secret-of-the-same-length!", &token),
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        verify_token(SECRET, "not.a.token"),
        Err(AppError::Unauthorized)
    ));

    let stale = now() - (TOKEN_DAYS + 1) * 86_400;
    let expired = create_token(SECRET, user, "1", Server::EN, Role::User, stale).unwrap();
    assert!(matches!(
        verify_token(SECRET, &expired),
        Err(AppError::Unauthorized)
    ));
}

#[test]
fn role_names_parse_back_and_unknown_reads_as_user() {
    for role in [
        Role::User,
        Role::Translator,
        Role::TierListEditor,
        Role::TierListAdmin,
        Role::SuperAdmin,
    ] {
        assert_eq!(parse_role(role_name(role)), role);
        assert_eq!(
            serde_json::to_value(role).unwrap(),
            role_name(role),
            "the wire spelling"
        );
    }
    assert_eq!(parse_role("root"), Role::User);
}

#[test]
fn a_sealed_credential_opens_for_its_owner_only() {
    let k = key();
    let owner = Uuid::from_u128(1);
    let (nonce, sealed) = k.seal(owner, b"durable-token").unwrap();
    assert_eq!(k.open(owner, &nonce, &sealed).unwrap(), b"durable-token");
    assert_eq!(
        k.open(Uuid::from_u128(2), &nonce, &sealed),
        Err(CredentialError::Open)
    );
    assert_eq!(
        k.open(owner, &nonce[..8], &sealed),
        Err(CredentialError::Open)
    );

    let (other_nonce, _) = k.seal(owner, b"durable-token").unwrap();
    assert_ne!(nonce, other_nonce, "a nonce is never reused");
}

#[test]
fn a_key_is_32_bytes_as_hex_or_base64() {
    assert!(CredentialKey::parse(&"ab".repeat(32)).is_ok());
    assert!(CredentialKey::parse("q6urq6urq6urq6urq6urq6urq6urq6urq6urq6urq6s=\n").is_ok());
    for bad in ["deadbeef", "", &"ab".repeat(33), "not hex or base64!"] {
        assert_eq!(
            CredentialKey::parse(bad).err(),
            Some(CredentialError::BadKey),
            "{bad}"
        );
    }
    assert_eq!(format!("{:?}", key()), "CredentialKey(<redacted>)");
}

#[tokio::test]
async fn a_subject_with_no_account_row_reports_its_claim_as_a_known_role() {
    let (ctx, db) = common::context().await;
    let claims = |role: &str| Claims {
        sub: "service".to_owned(),
        uid: "0".to_owned(),
        server: "en".to_owned(),
        role: role.to_owned(),
        exp: now() + 60,
        iat: now(),
    };
    let unknown = verify(&ctx, &claims("Mystery")).await.unwrap();
    assert_eq!(
        (unknown.role.as_str(), unknown.can_access_admin_panel),
        ("user", false)
    );
    let known = verify(&ctx, &claims("super_admin")).await.unwrap();
    assert_eq!(
        (known.role.as_str(), known.can_access_admin_panel),
        ("super_admin", true)
    );
    db.close().await;
}
