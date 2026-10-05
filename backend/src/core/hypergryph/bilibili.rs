//! Login via the `BiliGame` publisher SDK (CN Bilibili client; the CN official
//! client uses `passport.rs`).
//!
//! Not `passport.bilibili.com` OAuth: a direct login against a `BiliGame`
//! merchant endpoint scoped to Arknights's merchant/game/server ids. The password
//! path (`issue/cipher/v3` + `login/v3`, RSA-encrypted password) follows the
//! protocol documented by thesadru/arkprts (GPL-3.0), confirmed against a real
//! response. Independent reimplementation, not a copy of arkprts.
//!
//! [`send_sms_code`] and [`login_sms`] are a GUESS: arkprts has no SMS path and
//! neither is confirmed. Probed 2026-09-15, `issue/sms_code/v3` answers 404, so
//! SMS login doesn't work until the real endpoint turns up. The guess mirrors the
//! password endpoints (same ids and signing, `issue/sms_code/v3`, `login/sms/v3`,
//! plaintext `sms_code` in place of the RSA `pwd`). A 404 or odd shape means a
//! wrong path or field name: read the actual error and fix that assumption.

use std::time::Duration;

use base64::{Engine, engine::general_purpose::STANDARD};
use rand::Rng;
use reqwest::Client;
use rsa::Pkcs1v15Encrypt;
use rsa::pkcs8::DecodePublicKey;
use serde::Deserialize;

use crate::core::hypergryph::crypto::generate_bilibili_sign;
use crate::core::hypergryph::fetch::FetchError;
use crate::utils::redact::redacted_body;

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
        tracing::warn!(body = %redacted_body(&text), error = %e, "bilibili::load_cipher: failed to parse response");
        FetchError::ParseError("bilibili::load_cipher: invalid upstream response".into())
    })
}

/// Device id in arkprts's nine-segment shape (longer than a UUID). Client-side
/// only; nothing suggests the server checks the shape, just presence and
/// per-attempt uniqueness.
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
    // The cipher key is an OpenSSL `rsa -pubout` PEM (SubjectPublicKeyInfo,
    // "BEGIN PUBLIC KEY"), not PKCS1 "BEGIN RSA PUBLIC KEY", so pkcs8 decoding.
    // Confirmed on a real response: pkcs1 rejected it with "expecting \"RSA PUBLIC KEY\"".
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

/// `{"code": 500002, "message": "PWD_INVALID", ...}` for a wrong password or an
/// unknown account (indistinguishable).
#[derive(Deserialize)]
struct BilibiliError {
    #[serde(default)]
    code: i64,
    #[serde(default)]
    message: String,
    #[serde(default)]
    server_message: Option<String>,
}

#[derive(Deserialize)]
pub struct BilibiliLoginResult {
    /// The SDK sends this as a JSON number; the u8 exchange wants a string.
    #[serde(deserialize_with = "string_or_integer")]
    pub uid: String,
    pub access_key: String,
}

fn string_or_integer<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Text(String),
        Integer(i64),
    }
    Ok(match Raw::deserialize(d)? {
        Raw::Text(s) => s,
        Raw::Integer(n) => n.to_string(),
    })
}

/// Yields a channel `uid` + `access_key` for
/// [`crate::core::hypergryph::session::login_bilibili`]'s u8 exchange with
/// `channel_id = "2"`, the same pipeline Yostar's `uid`/`token` use.
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

    if let Ok(err) = serde_json::from_str::<BilibiliError>(&text)
        && err.code != 0
    {
        return Err(FetchError::ParseError(format!(
            "bilibili::login: upstream refused (code {}): {}{}",
            err.code,
            err.message,
            match err.server_message.as_deref() {
                Some(m) if !m.is_empty() => format!(" ({m})"),
                _ => String::new(),
            }
        )));
    }
    let data: BilibiliLoginResult = serde_json::from_str(&text).map_err(|e| {
        tracing::warn!(body = %redacted_body(&text), error = %e, "bilibili::login: failed to parse response");
        FetchError::ParseError(format!(
            "bilibili::login: invalid upstream response ({e}): {}",
            redacted_body(&text)
        ))
    })?;

    if data.uid.is_empty() || data.access_key.is_empty() {
        return Err(FetchError::ParseError(
            "bilibili::login: upstream returned an empty uid or access_key".into(),
        ));
    }

    Ok(data)
}

/// SMS code for `phone`, ahead of [`login_sms`]. UNVERIFIED: the path is a guess
/// (see module docs).
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

/// Phone + SMS code login, same `{uid, access_key}` shape as [`login`].
/// UNVERIFIED: see module docs.
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
        tracing::warn!(body = %redacted_body(&text), error = %e, "bilibili::login_sms: failed to parse response");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_result_accepts_a_numeric_or_string_uid() {
        let n: BilibiliLoginResult =
            serde_json::from_str(r#"{"code":0,"uid":123456,"access_key":"k"}"#).unwrap();
        assert_eq!((n.uid.as_str(), n.access_key.as_str()), ("123456", "k"));
        let s: BilibiliLoginResult =
            serde_json::from_str(r#"{"uid":"123456","access_key":"k"}"#).unwrap();
        assert_eq!(s.uid, "123456");
    }
}
