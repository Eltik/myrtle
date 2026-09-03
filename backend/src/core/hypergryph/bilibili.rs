//! Login via the Bilibili channel's `BiliGame` publisher SDK, used by the CN
//! Bilibili client (as opposed to the CN official/Hypergryph client, see
//! `passport.rs`).
//!
//! This is not `passport.bilibili.com` OAuth: it is a direct login against a
//! `BiliGame` merchant endpoint scoped to Arknights's own merchant/game/server
//! ids on Bilibili's platform. The username/password path (`issue/cipher/v3`
//! and `login/v3`, RSA-encrypted password) is a reverse-engineered protocol
//! fact documented by thesadru/arkprts (GPL-3.0) and confirmed against a real
//! biligame response; this module is an independent reimplementation from
//! that documented protocol, not a copy of arkprts's source.
//!
//! [`send_sms_code`] and [`login_sms`] are NOT from arkprts (it has no SMS
//! path at all) and are NOT confirmed against a real response. The actual
//! Bilibili Arknights client does offer SMS-code login (this was checked;
//! see the module's accompanying report), but no endpoint for it has
//! surfaced anywhere in research. These two functions are a structural guess
//! that mirrors the confirmed password endpoints as closely as possible:
//! same merchant/game/server ids, same signing, an `issue/sms_code/v3` sibling
//! to `issue/cipher/v3`, and a `login/sms/v3` sibling to `login/v3` with a
//! plaintext `sms_code` field standing in for the RSA-encrypted `pwd` (a
//! short-lived SMS code has no obvious reason to need RSA encryption the way
//! a reusable password does). Treat a failure here as "wrong path or field
//! name," not "bad code": if it 404s or comes back with an unexpected shape,
//! the fix is to see the actual request errors and recognize which
//! assumption to correct.

use std::time::Duration;

use base64::{Engine, engine::general_purpose::STANDARD};
use rand::Rng;
use reqwest::Client;
use rsa::Pkcs1v15Encrypt;
use rsa::pkcs8::DecodePublicKey;
use serde::Deserialize;

use crate::core::hypergryph::crypto::generate_bilibili_sign;
use crate::core::hypergryph::fetch::FetchError;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const BASE_URL: &str = "https://line1-sdk-center-login-sh.biligame.net/api/external";
const MERCHANT_ID: &str = "328";
const GAME_ID: &str = "952";
const SERVER_ID: &str = "1178";
const VERSION: &str = "3";

fn unix_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

fn signed_form(mut pairs: Vec<(&str, String)>) -> Vec<(&str, String)> {
    let view: Vec<(&str, &str)> = pairs.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let sign = generate_bilibili_sign(&view);
    pairs.push(("sign", sign));
    pairs
}

/// application/x-www-form-urlencoded encoding of `pairs`, in order.
fn encode_form(pairs: &[(&str, String)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

#[derive(Deserialize)]
struct CipherResponse {
    cipher_key: String,
    hash: String,
}

async fn load_cipher(client: &Client) -> Result<CipherResponse, FetchError> {
    let body = signed_form(vec![
        ("merchant_id", MERCHANT_ID.to_owned()),
        ("game_id", GAME_ID.to_owned()),
        ("server_id", SERVER_ID.to_owned()),
        ("version", VERSION.to_owned()),
        ("timestamp", unix_timestamp()),
        ("cipher_type", "bili_login_rsa".to_owned()),
    ]);

    let response = client
        .post(format!("{BASE_URL}/issue/cipher/v3"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(encode_form(&body))
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| FetchError::ParseError(format!("bilibili::load_cipher: read body: {e}")))?;

    if !status.is_success() {
        return Err(FetchError::ParseError(format!(
            "bilibili::load_cipher: upstream non-success (status={status}): {text}"
        )));
    }

    serde_json::from_str(&text).map_err(|e| {
        tracing::warn!(body = %text, error = %e, "bilibili::load_cipher: failed to parse response");
        FetchError::ParseError("bilibili::load_cipher: invalid upstream response".into())
    })
}

/// Random device id in the odd nine-segment shape arkprts's client generates
/// (longer than a UUID's five segments). Purely client-side; nothing in the
/// research gathered suggests the server validates its shape, only that it be
/// present and reasonably unique per login attempt.
fn random_bd_id() -> String {
    const SEGMENT_LENGTHS: [usize; 9] = [8, 4, 4, 4, 12, 8, 4, 4, 4];
    let mut rng = rand::rng();
    SEGMENT_LENGTHS
        .iter()
        .map(|&n| {
            let mut bytes = vec![0u8; n.div_ceil(2)];
            rng.fill_bytes(&mut bytes);
            hex::encode(bytes)[..n].to_owned()
        })
        .collect::<Vec<_>>()
        .join("-")
}

fn sign_password(password: &str, cipher_key: &str, hash: &str) -> Result<String, FetchError> {
    // biligame returns the cipher key as a standard OpenSSL `rsa -pubout` PEM
    // (SubjectPublicKeyInfo, "-----BEGIN PUBLIC KEY-----"), not a raw PKCS1
    // "-----BEGIN RSA PUBLIC KEY-----" block, so this needs pkcs8 decoding, not
    // pkcs1 (confirmed against a real cipher_key response: pkcs1 parsing
    // rejected it with "expecting \"RSA PUBLIC KEY\"").
    let public_key = RsaPublicKey::from_public_key_pem(cipher_key)
        .map_err(|e| FetchError::ParseError(format!("bilibili: invalid cipher key: {e}")))?;

    let plaintext = format!("{hash}{password}");
    let encrypted = public_key
        .encrypt(
            &mut rsa::rand_core::OsRng,
            Pkcs1v15Encrypt,
            plaintext.as_bytes(),
        )
        .map_err(|e| FetchError::ParseError(format!("bilibili: rsa encrypt failed: {e}")))?;

    Ok(STANDARD.encode(encrypted))
}

use rsa::RsaPublicKey;

#[derive(Deserialize)]
pub struct BilibiliLoginResult {
    pub uid: String,
    pub access_key: String,
}

/// Logs into the `BiliGame` publisher SDK, yielding a channel `uid` +
/// `access_key` pair. Feed these into
/// [`crate::core::hypergryph::session::login_bilibili`]'s u8 exchange with
/// `channel_id = "2"`, the same shared pipeline Yostar's `uid`/`token` go
/// through.
pub async fn login(
    client: &Client,
    username: &str,
    password: &str,
) -> Result<BilibiliLoginResult, FetchError> {
    let cipher = load_cipher(client).await?;
    let pwd = sign_password(password, &cipher.cipher_key, &cipher.hash)?;

    let body = signed_form(vec![
        ("merchant_id", MERCHANT_ID.to_owned()),
        ("game_id", GAME_ID.to_owned()),
        ("server_id", SERVER_ID.to_owned()),
        ("version", VERSION.to_owned()),
        ("timestamp", unix_timestamp()),
        ("bd_id", random_bd_id()),
        ("user_id", username.to_owned()),
        ("pwd", pwd),
    ]);

    let response = client
        .post(format!("{BASE_URL}/login/v3"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(encode_form(&body))
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| FetchError::ParseError(format!("bilibili::login: read body: {e}")))?;

    if !status.is_success() {
        return Err(FetchError::ParseError(format!(
            "bilibili::login: upstream non-success (status={status}): {text}"
        )));
    }

    let data: BilibiliLoginResult = serde_json::from_str(&text).map_err(|e| {
        tracing::warn!(body = %text, error = %e, "bilibili::login: failed to parse response");
        FetchError::ParseError(
            "bilibili::login: invalid upstream response, check credentials".into(),
        )
    })?;

    if data.uid.is_empty() || data.access_key.is_empty() {
        return Err(FetchError::ParseError(
            "bilibili::login: upstream returned an empty uid or access_key".into(),
        ));
    }

    Ok(data)
}

/// Requests an SMS code for `phone` on the `BiliGame` channel, ahead of
/// [`login_sms`]. UNVERIFIED: see module docs, the endpoint path itself is a
/// guess.
pub async fn send_sms_code(client: &Client, phone: &str) -> Result<(), FetchError> {
    let body = signed_form(vec![
        ("merchant_id", MERCHANT_ID.to_owned()),
        ("game_id", GAME_ID.to_owned()),
        ("server_id", SERVER_ID.to_owned()),
        ("version", VERSION.to_owned()),
        ("timestamp", unix_timestamp()),
        ("user_id", phone.to_owned()),
    ]);

    let response = client
        .post(format!("{BASE_URL}/issue/sms_code/v3"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(encode_form(&body))
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| FetchError::ParseError(format!("bilibili::send_sms_code: read body: {e}")))?;

    if !status.is_success() {
        return Err(FetchError::ParseError(format!(
            "bilibili::send_sms_code: upstream non-success (status={status}): {text}"
        )));
    }

    Ok(())
}

/// Logs into the `BiliGame` publisher SDK with a phone number and the SMS code
/// requested via [`send_sms_code`], yielding the same `{uid, access_key}`
/// shape [`login`] does. UNVERIFIED: see module docs.
pub async fn login_sms(
    client: &Client,
    phone: &str,
    sms_code: &str,
) -> Result<BilibiliLoginResult, FetchError> {
    let body = signed_form(vec![
        ("merchant_id", MERCHANT_ID.to_owned()),
        ("game_id", GAME_ID.to_owned()),
        ("server_id", SERVER_ID.to_owned()),
        ("version", VERSION.to_owned()),
        ("timestamp", unix_timestamp()),
        ("bd_id", random_bd_id()),
        ("user_id", phone.to_owned()),
        ("sms_code", sms_code.to_owned()),
    ]);

    let response = client
        .post(format!("{BASE_URL}/login/sms/v3"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(encode_form(&body))
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| FetchError::ParseError(format!("bilibili::login_sms: read body: {e}")))?;

    if !status.is_success() {
        return Err(FetchError::ParseError(format!(
            "bilibili::login_sms: upstream non-success (status={status}): {text}"
        )));
    }

    let data: BilibiliLoginResult = serde_json::from_str(&text).map_err(|e| {
        tracing::warn!(body = %text, error = %e, "bilibili::login_sms: failed to parse response");
        FetchError::ParseError(
            "bilibili::login_sms: invalid upstream response, check the code or that this guessed endpoint is even right".into(),
        )
    })?;

    if data.uid.is_empty() || data.access_key.is_empty() {
        return Err(FetchError::ParseError(
            "bilibili::login_sms: upstream returned an empty uid or access_key".into(),
        ));
    }

    Ok(data)
}
