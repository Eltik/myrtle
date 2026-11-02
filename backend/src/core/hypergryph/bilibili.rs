//! Login via the `BiliGame` publisher SDK (CN Bilibili client; the CN official
//! client uses `passport.rs`).
//!
//! Not `passport.bilibili.com` OAuth: a direct login against a `BiliGame`
//! merchant endpoint scoped to Arknights's merchant/game/server ids. The password
//! path (`issue/cipher/v3` + `login/v3`, RSA-encrypted password) follows the
//! protocol documented by thesadru/arkprts (GPL-3.0), confirmed against a real
//! response. Independent reimplementation, not a copy of arkprts.
//!
//! The SMS path ([`send_otp`] + [`login_otp`]) is `otp/send/v3` then
//! `login/otp/v3`, as in mikumifa/UmaShow `src/autouma/bilibiliSms.ts`, with
//! Arknights's ids and app key in place of Uma Musume's. Confirmed 2026-11-02:
//! it logs in an account that password login refuses with `200007` (risk
//! control), and a non-CN number needs its own `country_code` (`86005` is a
//! number that does not parse under the code sent). The earlier guesses,
//! `issue/sms_code/v3` and `login/sms/v3`, answer 404.

use std::time::Duration;

use base64::{Engine, engine::general_purpose::STANDARD};
use rand::Rng;
use reqwest::Client;
use rsa::Pkcs1v15Encrypt;
use rsa::pkcs8::DecodePublicKey;
use serde::{Deserialize, Serialize};

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

/// Device id in arkprts's nine-segment shape (longer than a UUID). One per
/// process, so retries within a run come from one device rather than reading
/// as a new device each time, which is what risk control flags.
fn bd_id() -> &'static str {
    static BD_ID: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    BD_ID.get_or_init(random_bd_id)
}

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
///
/// `code` arrives as a number or a string (`"200007"`); `need_captch` is `"1"`
/// on `200007` "trigger account secondary verify", the SDK's risk control.
/// `start_captcha/v3` does issue a Geetest v3 challenge, but a solved one sent
/// back with the login (as `ctoken`/`gt_user_id`, `challenge`, `validate`,
/// `seccode`, from a stable `bd_id`) still drew `200007` twice on 2026-10-15,
/// so the flag needs whatever the real client does, not a captcha.
#[derive(Deserialize)]
struct BilibiliError {
    #[serde(default, deserialize_with = "string_or_integer")]
    code: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    server_message: Option<String>,
    #[serde(default, deserialize_with = "string_or_integer")]
    need_captch: String,
}

impl BilibiliError {
    fn is_error(&self) -> bool {
        !self.code.is_empty() && self.code != "0"
    }

    fn needs_captcha(&self) -> bool {
        self.need_captch == "1"
    }
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
        ("bd_id", bd_id().to_owned()),
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
        && err.is_error()
    {
        if err.needs_captcha() {
            return Err(FetchError::ParseError(format!(
                "bilibili::login: account under risk control (code {}): {}. A \
                 solved captcha does not clear it (tried 2026-10-15); wait days \
                 and retry, use another account, or `set-session --import` an \
                 access_key from the real client",
                err.code, err.message
            )));
        }
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

/// The SDK's public key for `feign_sign` on `login/otp/v3`: SDK-wide, not per
/// game. Exponent 65537.
const FEIGN_SIGN_MODULUS: &str = concat!(
    "9bd251710f53895feb03bc6d679fe4068bde702113fbd58d2bda3b38ecd81399",
    "89cc28ab7055b45d729bdcfcc61e3903f539409471008125a296a53b6cb88785",
    "3e05d8948abb968a0aab12f9c73076aae1008a745545359a3e90f9bf445ee27e",
    "1be9266612be018d583642156b544f201d6905425459c1fe5fe93aeb91d3fe05",
);

/// How long a sent code stays usable. The SDK does not say; this matches the
/// client's own resend window.
pub const OTP_TTL_SECS: u64 = 5 * 60;

/// A sent SMS code, waiting for the number's owner to type it in.
///
/// `login/otp/v3` must come from the same device as `otp/send/v3` and name its
/// `captcha_key`, so the pair is carried between the two calls (in memory for
/// the site, in a file for `set-session`).
#[derive(Clone, Serialize, Deserialize)]
pub struct OtpChallenge {
    /// The national number, without the country code.
    pub phone: String,
    pub country_code: String,
    captcha_key: String,
    device_id: String,
    bd_id: String,
    /// Unix seconds.
    pub issued_at: u64,
}

impl OtpChallenge {
    #[must_use]
    pub fn is_expired(&self, now: u64) -> bool {
        now.saturating_sub(self.issued_at) > OTP_TTL_SECS
    }
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn unix_millis() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
        .to_string()
}

/// The device and client fields the SDK sends with every OTP call.
fn otp_common_fields(
    challenge_device: &str,
    bd_id: &str,
    country_code: &str,
) -> Vec<(&'static str, String)> {
    let host = BASE_URL.trim_end_matches("/api/external");
    vec![
        ("cur_buvid", challenge_device.to_owned()),
        ("old_buvid", challenge_device.to_owned()),
        ("udid", challenge_device.to_owned()),
        ("bd_id", bd_id.to_owned()),
        ("app_id", GAME_ID.to_owned()),
        ("game_id", GAME_ID.to_owned()),
        ("server_id", SERVER_ID.to_owned()),
        ("merchant_id", MERCHANT_ID.to_owned()),
        ("sdk_ver", "6.28.0".to_owned()),
        ("version_code", "11150".to_owned()),
        ("channel_id", "1".to_owned()),
        ("sdk_type", "1".to_owned()),
        ("platform", "3".to_owned()),
        ("platform_type", "3".to_owned()),
        ("version", VERSION.to_owned()),
        ("domain_switch_count", "0".to_owned()),
        ("country_code", country_code.to_owned()),
        ("domain", host.trim_start_matches("https://").to_owned()),
        ("original_domain", host.to_owned()),
        ("sdk_log_type", "1".to_owned()),
        ("current_env", "0".to_owned()),
    ]
}

fn random_device_id() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode_upper(bytes)
}

/// RSA/ECB/PKCS1 of the form `sign`, base64, as `login/otp/v3` wants it.
fn feign_sign(sign: &str) -> Result<String, FetchError> {
    let modulus = rsa::BigUint::parse_bytes(FEIGN_SIGN_MODULUS.as_bytes(), 16)
        .ok_or_else(|| FetchError::ParseError("bilibili: bad feign_sign modulus".into()))?;
    let key = RsaPublicKey::new(modulus, rsa::BigUint::from(65_537u32))
        .map_err(|e| FetchError::ParseError(format!("bilibili: feign_sign key: {e}")))?;
    let encrypted = key
        .encrypt(&mut rsa::rand_core::OsRng, Pkcs1v15Encrypt, sign.as_bytes())
        .map_err(|e| FetchError::ParseError(format!("bilibili: feign_sign encrypt: {e}")))?;
    Ok(STANDARD.encode(encrypted))
}

async fn post_otp(
    client: &Client,
    path: &str,
    body: &[(&str, String)],
    what: &str,
) -> Result<String, FetchError> {
    let response = client
        .post(format!("{BASE_URL}/{path}"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(encode_form(body))
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| FetchError::ParseError(format!("bilibili::{what}: read body: {e}")))?;

    if !status.is_success() {
        return Err(FetchError::ParseError(format!(
            "bilibili::{what}: upstream non-success (status={status}): {}",
            redacted_body(&text)
        )));
    }
    if let Ok(err) = serde_json::from_str::<BilibiliError>(&text)
        && err.is_error()
    {
        return Err(FetchError::ParseError(format!(
            "bilibili::{what}: upstream refused (code {}): {}{}",
            err.code,
            err.message,
            match err.server_message.as_deref() {
                Some(m) if !m.is_empty() => format!(" ({m})"),
                _ => String::new(),
            }
        )));
    }
    Ok(text)
}

/// Text a login code to `phone` (national number) under `country_code` (`86`
/// for mainland China, `1` for the US).
pub async fn send_otp(
    client: &Client,
    phone: &str,
    country_code: &str,
) -> Result<OtpChallenge, FetchError> {
    let device_id = random_device_id();
    let bd_id = random_bd_id();
    let mut fields = otp_common_fields(&device_id, &bd_id, country_code);
    fields.extend([
        ("otp_type", "login".to_owned()),
        ("otp_channel_no", phone.to_owned()),
        ("otp_channel_category", "tel".to_owned()),
        ("timestamp", unix_millis()),
    ]);
    let text = post_otp(client, "otp/send/v3", &signed_form(fields), "send_otp").await?;

    #[derive(Deserialize)]
    struct Sent {
        #[serde(default)]
        captcha_key: String,
    }
    let sent: Sent = serde_json::from_str(&text).map_err(|e| {
        FetchError::ParseError(format!(
            "bilibili::send_otp: invalid upstream response ({e}): {}",
            redacted_body(&text)
        ))
    })?;
    if sent.captcha_key.is_empty() {
        return Err(FetchError::ParseError(
            "bilibili::send_otp: upstream returned no captcha_key".into(),
        ));
    }

    Ok(OtpChallenge {
        phone: phone.to_owned(),
        country_code: country_code.to_owned(),
        captcha_key: sent.captcha_key,
        device_id,
        bd_id,
        issued_at: unix_seconds(),
    })
}

/// Trade the texted code for a channel `uid` + `access_key`, the same shape
/// [`login`] yields.
pub async fn login_otp(
    client: &Client,
    challenge: &OtpChallenge,
    otp: &str,
) -> Result<BilibiliLoginResult, FetchError> {
    let mut fields = otp_common_fields(
        &challenge.device_id,
        &challenge.bd_id,
        &challenge.country_code,
    );
    fields.extend([
        ("captcha_key", challenge.captcha_key.clone()),
        ("mobile", challenge.phone.clone()),
        ("otp", otp.to_owned()),
        ("timestamp", unix_millis()),
    ]);
    let mut body = signed_form(fields);
    let sign = body.last().map(|(_, v)| v.clone()).unwrap_or_default();
    body.push(("feign_sign", feign_sign(&sign)?));

    let text = post_otp(client, "login/otp/v3", &body, "login_otp").await?;
    let data: BilibiliLoginResult = serde_json::from_str(&text).map_err(|e| {
        FetchError::ParseError(format!(
            "bilibili::login_otp: invalid upstream response ({e}): {}",
            redacted_body(&text)
        ))
    })?;
    if data.uid.is_empty() || data.access_key.is_empty() {
        return Err(FetchError::ParseError(
            "bilibili::login_otp: upstream returned an empty uid or access_key".into(),
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

    #[test]
    fn risk_control_reads_as_a_captcha_demand() {
        let e: BilibiliError = serde_json::from_str(
            r#"{"code":"200007","need_captch":"1","message":"trigger account secondary verify"}"#,
        )
        .unwrap();
        assert!(e.is_error() && e.needs_captcha());

        let wrong_pwd: BilibiliError =
            serde_json::from_str(r#"{"code":500002,"message":"PWD_INVALID"}"#).unwrap();
        assert!(wrong_pwd.is_error() && !wrong_pwd.needs_captcha());

        let ok: BilibiliError = serde_json::from_str(r#"{"code":0,"uid":1}"#).unwrap();
        assert!(!ok.is_error());
    }

    #[test]
    fn feign_sign_encrypts_to_one_rsa_block() {
        let out = STANDARD
            .decode(feign_sign("0123456789abcdef0123456789abcdef").unwrap())
            .unwrap();
        assert_eq!(out.len(), 128);
    }

    #[test]
    fn otp_fields_carry_the_arknights_ids_and_country_code() {
        let fields = otp_common_fields("DEV", "bd", "1");
        let get = |k: &str| {
            fields
                .iter()
                .find(|(key, _)| *key == k)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(get("game_id"), Some(GAME_ID));
        assert_eq!(get("merchant_id"), Some(MERCHANT_ID));
        assert_eq!(get("country_code"), Some("1"));
        assert_eq!(
            get("domain"),
            Some("line1-sdk-center-login-sh.biligame.net")
        );
        assert_eq!(
            get("original_domain"),
            Some("https://line1-sdk-center-login-sh.biligame.net")
        );
    }

    #[test]
    fn a_challenge_expires_after_its_window() {
        let c = OtpChallenge {
            phone: "2065550100".into(),
            country_code: "1".into(),
            captcha_key: "k".into(),
            device_id: "d".into(),
            bd_id: "b".into(),
            issued_at: 1_000,
        };
        assert!(!c.is_expired(1_000 + OTP_TTL_SECS));
        assert!(c.is_expired(1_001 + OTP_TTL_SECS));
    }
}
