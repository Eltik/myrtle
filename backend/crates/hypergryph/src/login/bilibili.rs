use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rand::Rng;
use rsa::pkcs8::DecodePublicKey;
use rsa::{BigUint, Pkcs1v15Encrypt, RsaPublicKey};
use serde::{Deserialize, Deserializer, Serialize};

use crate::constants::BILIBILI_BASE;
use crate::crypto::generate_bilibili_sign;
use crate::error::UpstreamError;
use crate::fetch::{read_text, redacted_body, shape_error};
use crate::upstream::Upstream;

const CIPHER: &str = "bilibili::issue/cipher/v3";
const LOGIN: &str = "bilibili::login/v3";
const OTP_SEND: &str = "bilibili::otp/send/v3";
const OTP_LOGIN: &str = "bilibili::login/otp/v3";

const MERCHANT_ID: &str = "328";
const GAME_ID: &str = "952";
const SERVER_ID: &str = "1178";
const VERSION: &str = "3";

/// `feign_sign` key for `login/otp/v3`
/// Exponent 65537
const FEIGN_SIGN_MODULUS: &str = concat!(
    "9bd251710f53895feb03bc6d679fe4068bde702113fbd58d2bda3b38ecd81399",
    "89cc28ab7055b45d729bdcfcc61e3903f539409471008125a296a53b6cb88785",
    "3e05d8948abb968a0aab12f9c73076aae1008a745545359a3e90f9bf445ee27e",
    "1be9266612be018d583642156b544f201d6905425459c1fe5fe93aeb91d3fe05",
);

/// How long an OTP code stays usable
pub const OTP_TTL_SECS: u64 = 5 * 60;

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn unix_millis() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
        .to_string()
}

fn signed_form(mut pairs: Vec<(&'static str, String)>) -> Vec<(&'static str, String)> {
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

fn string_or_integer<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
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

/// `need_captch` is `"1"` on `200007`
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

    fn into_rejection(self, context: &'static str) -> UpstreamError {
        let detail = match self.server_message.as_deref() {
            Some(m) if !m.is_empty() => format!(" ({m})"),
            _ => String::new(),
        };
        UpstreamError::LoginRejected {
            context,
            code: self.code,
            message: format!("{}{detail}", self.message),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct BilibiliLoginResult {
    #[serde(deserialize_with = "string_or_integer")]
    pub uid: String,
    pub access_key: String,
}

/// Login must come from device that sent OTP code
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtpChallenge {
    /// National number w/ no country code
    pub phone: String,
    pub country_code: String,
    captcha_key: String,
    device_id: String,
    bd_id: String,
    pub issued_at: u64,
}

impl OtpChallenge {
    #[must_use]
    pub const fn is_expired(&self, now: u64) -> bool {
        now.saturating_sub(self.issued_at) > OTP_TTL_SECS
    }
}

/// Nine dash-joined lowercase hex segments of 8-4-4-4-12-8-4-4-4 digits
pub(crate) fn random_bd_id() -> String {
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

fn random_device_id() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode_upper(bytes)
}

async fn post_form(
    upstream: &Upstream,
    path: &str,
    body: &[(&str, String)],
    context: &'static str,
) -> Result<(reqwest::StatusCode, String), UpstreamError> {
    let base = upstream.config().hosts.bilibili.clone();
    let response = upstream
        .client()
        .post(format!("{base}/{path}"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(encode_form(body))
        .timeout(upstream.request_timeout())
        .send()
        .await?;
    let status = response.status();
    let text = read_text(response, context).await?;
    Ok((status, text))
}

#[derive(Deserialize)]
struct CipherResponse {
    cipher_key: String,
    hash: String,
}

async fn load_cipher(upstream: &Upstream) -> Result<CipherResponse, UpstreamError> {
    let body = signed_form(vec![
        ("merchant_id", MERCHANT_ID.to_owned()),
        ("game_id", GAME_ID.to_owned()),
        ("server_id", SERVER_ID.to_owned()),
        ("version", VERSION.to_owned()),
        ("timestamp", unix_seconds().to_string()),
        ("cipher_type", "bili_login_rsa".to_owned()),
    ]);
    let (status, text) = post_form(upstream, "issue/cipher/v3", &body, CIPHER).await?;
    if !status.is_success() {
        return Err(UpstreamError::parse(
            CIPHER,
            format!(
                "upstream non-success (status={status}): {}",
                redacted_body(&text)
            ),
        ));
    }
    serde_json::from_str(&text).map_err(|e| {
        tracing::warn!(body = %redacted_body(&text), error = %shape_error(&e), "bilibili: cipher response does not parse");
        UpstreamError::parse(CIPHER, "invalid upstream response")
    })
}

/// Cipher key is `SubjectPublicKeyInfo` PEM ("BEGIN PUBLIC KEY")
fn sign_password(password: &str, cipher_key: &str, hash: &str) -> Result<String, UpstreamError> {
    let public_key = RsaPublicKey::from_public_key_pem(cipher_key)
        .map_err(|e| UpstreamError::parse(LOGIN, format!("invalid cipher key: {e}")))?;
    let plaintext = format!("{hash}{password}");
    let encrypted = public_key
        .encrypt(
            &mut rsa::rand_core::OsRng,
            Pkcs1v15Encrypt,
            plaintext.as_bytes(),
        )
        .map_err(|e| UpstreamError::parse(LOGIN, format!("rsa encrypt failed: {e}")))?;
    Ok(STANDARD.encode(encrypted))
}

fn parse_login(text: &str, context: &'static str) -> Result<BilibiliLoginResult, UpstreamError> {
    let data: BilibiliLoginResult = serde_json::from_str(text).map_err(|e| {
        UpstreamError::parse(
            context,
            format!(
                "invalid upstream response ({}): {}",
                shape_error(&e),
                redacted_body(text)
            ),
        )
    })?;
    if data.uid.is_empty() || data.access_key.is_empty() {
        return Err(UpstreamError::parse(
            context,
            "upstream returned an empty uid or access_key",
        ));
    }
    Ok(data)
}

/// Note: `200007` (risk control) is not cleared by solved captcha
pub async fn login(
    upstream: &Upstream,
    username: &str,
    password: &str,
) -> Result<BilibiliLoginResult, UpstreamError> {
    let cipher = load_cipher(upstream).await?;
    let pwd = sign_password(password, &cipher.cipher_key, &cipher.hash)?;

    let body = signed_form(vec![
        ("merchant_id", MERCHANT_ID.to_owned()),
        ("game_id", GAME_ID.to_owned()),
        ("server_id", SERVER_ID.to_owned()),
        ("version", VERSION.to_owned()),
        ("timestamp", unix_seconds().to_string()),
        ("bd_id", upstream.bilibili_bd_id().to_owned()),
        ("user_id", username.to_owned()),
        ("pwd", pwd),
    ]);
    let (status, text) = post_form(upstream, "login/v3", &body, LOGIN).await?;
    if !status.is_success() {
        return Err(UpstreamError::parse(
            LOGIN,
            format!(
                "upstream non-success (status={status}): {}",
                redacted_body(&text)
            ),
        ));
    }

    if let Ok(err) = serde_json::from_str::<BilibiliError>(&text)
        && err.is_error()
    {
        if err.needs_captcha() {
            return Err(UpstreamError::RiskControl {
                code: err.code,
                message: err.message,
            });
        }
        return Err(err.into_rejection(LOGIN));
    }
    parse_login(&text, LOGIN)
}

fn otp_common_fields(
    device_id: &str,
    bd_id: &str,
    country_code: &str,
) -> Vec<(&'static str, String)> {
    let host = BILIBILI_BASE.trim_end_matches("/api/external");
    vec![
        ("cur_buvid", device_id.to_owned()),
        ("old_buvid", device_id.to_owned()),
        ("udid", device_id.to_owned()),
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

/// RSA/ECB/PKCS1 of form `sign`
fn feign_sign(sign: &str) -> Result<String, UpstreamError> {
    let modulus = BigUint::parse_bytes(FEIGN_SIGN_MODULUS.as_bytes(), 16)
        .ok_or_else(|| UpstreamError::parse(OTP_LOGIN, "bad modulus"))?;
    let key = RsaPublicKey::new(modulus, BigUint::from(65_537u32))
        .map_err(|e| UpstreamError::parse(OTP_LOGIN, format!("key: {e}")))?;
    let encrypted = key
        .encrypt(&mut rsa::rand_core::OsRng, Pkcs1v15Encrypt, sign.as_bytes())
        .map_err(|e| UpstreamError::parse(OTP_LOGIN, format!("encrypt: {e}")))?;
    Ok(STANDARD.encode(encrypted))
}

async fn post_otp(
    upstream: &Upstream,
    path: &str,
    body: &[(&str, String)],
    context: &'static str,
) -> Result<String, UpstreamError> {
    let (status, text) = post_form(upstream, path, body, context).await?;
    if !status.is_success() {
        return Err(UpstreamError::parse(
            context,
            format!(
                "upstream non-success (status={status}): {}",
                redacted_body(&text)
            ),
        ));
    }
    if let Ok(err) = serde_json::from_str::<BilibiliError>(&text)
        && err.is_error()
    {
        return Err(err.into_rejection(context));
    }
    Ok(text)
}

pub async fn send_otp(
    upstream: &Upstream,
    phone: &str,
    country_code: &str,
) -> Result<OtpChallenge, UpstreamError> {
    #[derive(Deserialize)]
    struct Sent {
        #[serde(default)]
        captcha_key: String,
    }

    let device_id = random_device_id();
    let bd_id = random_bd_id();
    let mut fields = otp_common_fields(&device_id, &bd_id, country_code);
    fields.extend([
        ("otp_type", "login".to_owned()),
        ("otp_channel_no", phone.to_owned()),
        ("otp_channel_category", "tel".to_owned()),
        ("timestamp", unix_millis()),
    ]);
    let text = post_otp(upstream, "otp/send/v3", &signed_form(fields), OTP_SEND).await?;

    let sent: Sent = serde_json::from_str(&text).map_err(|e| {
        UpstreamError::parse(
            OTP_SEND,
            format!(
                "invalid upstream response ({}): {}",
                shape_error(&e),
                redacted_body(&text)
            ),
        )
    })?;
    if sent.captcha_key.is_empty() {
        return Err(UpstreamError::parse(
            OTP_SEND,
            "upstream returned no captcha_key",
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

pub async fn login_otp(
    upstream: &Upstream,
    challenge: &OtpChallenge,
    otp: &str,
) -> Result<BilibiliLoginResult, UpstreamError> {
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

    let text = post_otp(upstream, "login/otp/v3", &body, OTP_LOGIN).await?;
    parse_login(&text, OTP_LOGIN)
}
