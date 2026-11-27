//! Yostar, Bilibili and passport logins end to end against a scripted server.

mod common;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use common::{FakeServer, Recorded, Reply, serve, upstream};
use myrtle_hypergryph::crypto::generate_bilibili_sign;
use myrtle_hypergryph::login::passport::PassportCredential;
use myrtle_hypergryph::login::{bilibili, passport, yostar};
use myrtle_hypergryph::{Domain, NetworkConfig, UpstreamError, session};
use myrtle_types::Server;
use rsa::pkcs8::DecodePrivateKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use serde_json::json;

const TEST_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCqdKKgj8nhry16ZBHZ87ea2mce
iFJBuHEPuEjBuKD6xDkOdVS/ntYIjbMPaQEGaTZGPafs2bJvqIpLIoufwm2L9FMA
/TKqPB2wRtabALaZFhRaXfk/jfoSrsbN9D/OKVqx8Kf+efiBPh32LxDhKjaq+fQU
+26hH+ZgJQbX5G81MwIDAQAB
-----END PUBLIC KEY-----
";

const TEST_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIICeAIBADANBgkqhkiG9w0BAQEFAASCAmIwggJeAgEAAoGBAKp0oqCPyeGvLXpk
Ednzt5raZx6IUkG4cQ+4SMG4oPrEOQ51VL+e1giNsw9pAQZpNkY9p+zZsm+oiksi
i5/CbYv0UwD9Mqo8HbBG1psAtpkWFFpd+T+N+hKuxs30P84pWrHwp/55+IE+HfYv
EOEqNqr59BT7bqEf5mAlBtfkbzUzAgMBAAECgYAge5OWN3CIYoKvJSzFtQ0TJHWs
Otw6C/T3wu2ShSWgRA5/c9qPnaw5+w7Za/A3EYu3PP6oQyAe6SZZhR7Idhu71RwM
d+Ahtj/dL0B1kE3lki2Wo744vdR7fNCzpAMgYV3lvq7ekEJ957jLfL0eQmcKQGaO
bmpzkkNg8+M/iefrMQJBAOGlhMgu8Q/CVlhbsMYmyboiAn7HxWDCbzRcVlCQafFb
N+vx5nMiqYGPyADtKs1DY+SrVuJiKWUF3+m5dP5fgvkCQQDBYohSrvOv9oM2VUOL
Jo4cxLfl+yCEZBJjEJL4Gc8q2l92J8Va/8dAcifA3183GtNSWgveM4RH184o5BRD
2diLAkEAo0vpoAV4pvL5EdCpSX1gl7dtHtrr2G5dlAMO5Bbe1LAy9CmHoOMA0KaB
E0qPoySV8jKdGg/37QgnVznDlSMB4QJBAJd42ziQarWoS+xR6r7iqyujdjvAcR7h
13ka8VM9vARf2agNzMrTT7mN8M9k5wqdi2qUt3kLNxCi24sx4yGmT7MCQQC+ALjr
Tyb8NjJy2K0hmr/3r6e0hHW+oYbezoo6XHpksmMN4GSpSemzJOQybiDSab/PkeeY
FnnAuTYpnWyDuDDM
-----END PRIVATE KEY-----
";

fn mint_routes(req: &Recorded) -> Option<Reply> {
    match req.path.as_str() {
        "/u8/user/v1/getToken" => Some(Reply::json(
            &json!({ "result": 0, "uid": "u8-uid", "token": "u8-tok" }),
        )),
        "/gs/account/login" => Some(Reply::json(&json!({ "result": 0, "secret": "secret-1" }))),
        _ => None,
    }
}

fn yostar_server(portal_cookies: bool) -> impl Fn(&Recorded) -> Reply + Send + Sync {
    move |req: &Recorded| {
        if let Some(reply) = mint_routes(req) {
            return reply;
        }
        match req.path.as_str() {
            "/sdk/yostar/send-code" => Reply::json(&json!({ "Code": 200 })),
            "/sdk/yostar/get-auth" if req.json()["Code"] == "000000" => {
                Reply::json(&json!({ "Code": 100_403, "Msg": "wrong code", "Data": null }))
            }
            "/sdk/yostar/get-auth" => {
                Reply::json(&json!({ "Code": 200, "Msg": "OK", "Data": { "Token": "auth-tok" } }))
            }
            "/sdk/user/login" => Reply::json(&json!({
                "Data": { "UserInfo": { "ID": "yostar-uid", "Token": "yostar-tok" } }
            })),
            "/portal/api/user/login" if portal_cookies => Reply::json(&json!({}))
                .with_header("Set-Cookie", "YSSID=abc; Path=/; HttpOnly")
                .with_header("Set-Cookie", "YSSID.sig=def; Path=/"),
            _ => Reply::json(&json!({})),
        }
    }
}

#[tokio::test]
async fn send_code_posts_the_signed_account_body() {
    let server = serve(yostar_server(true)).await;
    let up = upstream(&server.base);
    let answer = yostar::send_code(&up, "doc@rhodes.island", Server::EN)
        .await
        .unwrap();
    assert_eq!(answer, json!({ "Code": 200 }));
    let req = &server.requests()[0];
    assert_eq!(req.path, "/sdk/yostar/send-code");
    assert_eq!(
        req.json(),
        json!({ "Account": "doc@rhodes.island", "Randstr": "", "Ticket": "" })
    );
    common::assert_signed(req, "", "");
}

#[tokio::test]
async fn yostar_login_runs_auth_portal_token_u8_and_login() {
    let server = serve(yostar_server(true)).await;
    let up = upstream(&server.base);
    let result = session::login(&up, "doc@rhodes.island", "123456", Server::JP)
        .await
        .unwrap();

    assert_eq!(
        server.paths(),
        [
            "/sdk/yostar/get-auth",
            "/portal/api/user/login",
            "/sdk/user/login",
            "/u8/user/v1/getToken",
            "/gs/account/login",
        ]
    );
    assert_eq!(result.yostar_email, "doc@rhodes.island");
    let portal = result.portal_session.unwrap();
    assert_eq!(
        (portal.yssid.as_str(), portal.yssid_sig.as_str()),
        ("abc", "def")
    );
    let s = result.session;
    assert_eq!((&*s.uid, &*s.secret, s.seqnum), ("u8-uid", "secret-1", 1));
    assert_eq!(
        (&*s.yostar_uid, &*s.yostar_token),
        ("yostar-uid", "yostar-tok")
    );

    let reqs = server.requests();
    assert_eq!(
        reqs[0].json(),
        json!({ "Account": "doc@rhodes.island", "Code": "123456" })
    );
    let portal_req = &reqs[1];
    assert_eq!(
        portal_req.json(),
        json!({ "channel": "yostar", "token": "auth-tok", "openId": "doc@rhodes.island", "account": "doc@rhodes.island", "checkAccount": false })
    );
    assert_eq!(
        portal_req.header("origin"),
        Some("https://account.yo-star.com")
    );
    assert_eq!(
        portal_req.header("referer"),
        Some("https://account.yo-star.com/login")
    );
    assert_eq!(portal_req.header("lang"), Some("en"));
    assert_eq!(
        reqs[2].json(),
        json!({
            "CheckAccount": 0,
            "Geetest": { "CaptchaId": null, "CaptchaOutput": null, "GenTime": null, "LotNumber": null, "PassToken": null },
            "OpenID": "doc@rhodes.island", "Secret": "", "Token": "auth-tok", "Type": "yostar", "UserName": "doc@rhodes.island",
        })
    );
    let auth: serde_json::Value =
        serde_json::from_str(reqs[2].header("authorization").unwrap()).unwrap();
    assert_eq!(auth["Head"]["PID"], "JP-AK");
    assert_eq!(
        reqs[3].json()["extension"],
        r#"{"token":"yostar-tok","type":1,"uid":"yostar-uid"}"#
    );
}

#[tokio::test]
async fn a_portal_without_cookies_does_not_fail_the_login() {
    let server = serve(yostar_server(false)).await;
    let up = upstream(&server.base);
    let result = session::login(&up, "a@b", "1", Server::EN).await.unwrap();
    assert!(result.portal_session.is_none());
    let err = yostar::account_portal_login(&up, "a@b", "t")
        .await
        .err()
        .unwrap();
    assert!(err.to_string().contains("missing YSSID cookies"), "{err}");
}

#[tokio::test]
async fn a_wrong_email_code_is_rejected_and_cn_has_no_yostar() {
    let server = serve(yostar_server(true)).await;
    let up = upstream(&server.base);
    let err = session::login(&up, "a@b", "000000", Server::EN)
        .await
        .err()
        .unwrap();
    assert_eq!(refused(&err), (100_403, "wrong code"));
    let err = yostar::send_code(&up, "a@b", Server::CN).await.unwrap_err();
    assert!(
        matches!(
            err,
            UpstreamError::Unsupported {
                server: Server::CN,
                ..
            }
        ),
        "{err:?}"
    );
    assert_eq!(server.paths(), ["/sdk/yostar/get-auth"]);
}

fn refused(err: &UpstreamError) -> (i64, &str) {
    match err {
        UpstreamError::Envelope(e) => (e.code, e.msg.as_str()),
        other => panic!("expected a refusal envelope, got {other:?}"),
    }
}

async fn sdk_answering(reply: serde_json::Value) -> FakeServer {
    serve(move |_: &Recorded| Reply::json(&reply)).await
}

async fn send_code_answered(reply: serde_json::Value) -> Result<serde_json::Value, UpstreamError> {
    let server = sdk_answering(reply).await;
    yostar::send_code(&upstream(&server.base), "a@b", Server::EN).await
}

async fn submit_auth_answered(reply: serde_json::Value) -> Result<String, UpstreamError> {
    let server = sdk_answering(reply).await;
    yostar::submit_auth(&upstream(&server.base), "a@b", "123456", Server::EN)
        .await
        .map(|auth| auth.token)
}

#[tokio::test]
async fn send_code_200_is_the_only_success() {
    let sent = json!({ "Code": 200, "Data": {}, "Msg": "OK" });
    assert_eq!(send_code_answered(sent.clone()).await.unwrap(), sent);
}

#[tokio::test]
async fn a_send_code_refusal_carries_yostars_code_and_message() {
    let err = send_code_answered(
        json!({ "Code": 100_555, "Data": {}, "Msg": "Requests are too frequent" }),
    )
    .await
    .unwrap_err();
    assert_eq!(refused(&err), (100_555, "Requests are too frequent"));
}

#[tokio::test]
async fn a_send_code_refusal_without_a_message_still_says_something() {
    for reply in [
        json!({ "Code": 100_999 }),
        json!({ "Code": 100_999, "Msg": "" }),
    ] {
        let err = send_code_answered(reply).await.unwrap_err();
        assert_eq!(refused(&err), (100_999, "Yostar refused the request"));
    }
}

#[tokio::test]
async fn a_send_code_reply_without_a_code_is_not_a_confirmed_send() {
    for reply in [
        json!({}),
        json!({ "Msg": "OK" }),
        json!({ "Code": "200" }),
        json!([]),
    ] {
        let err = send_code_answered(reply.clone()).await.unwrap_err();
        assert!(
            matches!(&err, UpstreamError::Parse { context, .. } if context == "yostar::yostar/send-code"),
            "{reply}: {err:?}"
        );
    }
}

#[tokio::test]
async fn the_email_limit_is_explained_in_english() {
    let err = send_code_answered(
        json!({ "Code": 100_302, "Data": {}, "Msg": "邮件发送频率上限,错误代码:1" }),
    )
    .await
    .unwrap_err();
    assert_eq!(
        refused(&err),
        (
            100_302,
            "Yostar has emailed this address too many codes recently. Wait a while before requesting another"
        )
    );
}

#[tokio::test]
async fn a_rejected_login_code_says_so() {
    let err = submit_auth_answered(
        json!({ "Code": 100_303, "Data": {}, "Msg": "获取授权信息失败,错误代码:%!d(string=3)" }),
    )
    .await
    .unwrap_err();
    assert_eq!(
        refused(&err),
        (
            100_303,
            "The code is wrong, has expired, or was already used"
        )
    );
}

#[tokio::test]
async fn an_accepted_login_code_yields_its_token() {
    let token =
        submit_auth_answered(json!({ "Code": 200, "Data": { "Token": "abc123" }, "Msg": "OK" }))
            .await
            .unwrap();
    assert_eq!(token, "abc123");
}

#[tokio::test]
async fn other_login_refusals_keep_yostars_own_message() {
    let err =
        submit_auth_answered(json!({ "Code": 100_123, "Data": null, "Msg": "Account is banned" }))
            .await
            .unwrap_err();
    assert_eq!(refused(&err), (100_123, "Account is banned"));
}

#[tokio::test]
async fn a_200_without_a_token_is_not_a_login() {
    for body in [
        json!({ "Code": 200, "Data": {}, "Msg": "OK" }),
        json!({ "Code": 200, "Msg": "OK" }),
        json!({ "Code": 200, "Data": { "Token": "" } }),
    ] {
        let err = submit_auth_answered(body.clone()).await.unwrap_err();
        assert!(
            matches!(&err, UpstreamError::Parse { context, .. } if context == "yostar::yostar/get-auth"),
            "{body}: {err:?}"
        );
    }
}

#[tokio::test]
async fn a_token_request_refusal_is_read_before_the_success_fields() {
    let server =
        sdk_answering(json!({ "Code": 100_123, "Data": {}, "Msg": "Account is banned" })).await;
    let err = yostar::request_token(&upstream(&server.base), "a@b", "auth-tok", Server::EN)
        .await
        .err()
        .unwrap();
    assert_eq!(refused(&err), (100_123, "Account is banned"));
}

#[tokio::test]
async fn a_login_is_refused_before_the_code_is_spent_while_the_domains_are_missing() {
    let server = serve(yostar_server(true)).await;
    let up = upstream(&server.base);
    let mut cfg = NetworkConfig::new(common::device_ids());
    cfg.hosts.yostar_sdk = up.config().hosts.yostar_sdk.clone();
    cfg.hosts
        .account_portal
        .clone_from(&up.config().hosts.account_portal);
    up.update(|c| *c = cfg.clone());

    let err = session::login(&up, "a@b", "123456", Server::EN)
        .await
        .err()
        .unwrap();
    assert!(
        matches!(
            err,
            UpstreamError::DomainNotFound {
                server: Server::EN,
                domain: Domain::U8
            }
        ),
        "{err:?}"
    );
    assert!(server.requests().is_empty());

    cfg.set_domain(Server::EN, Domain::U8, format!("{}/u8", server.base));
    cfg.set_domain(Server::EN, Domain::GS, format!("{}/gs", server.base));
    up.update(|c| *c = cfg.clone());
    session::login(&up, "a@b", "123456", Server::EN)
        .await
        .unwrap();
    let err = session::login(&up, "a@b", "123456", Server::JP)
        .await
        .err()
        .unwrap();
    assert!(
        matches!(
            err,
            UpstreamError::DomainNotFound {
                server: Server::JP,
                ..
            }
        ),
        "{err:?}"
    );
}

fn form_value<'a>(form: &'a [(String, String)], key: &str) -> &'a str {
    form.iter()
        .find_map(|(k, v)| (k == key).then_some(v.as_str()))
        .expect(key)
}

fn assert_bilibili_signed(form: &[(String, String)], extra_after_sign: usize) {
    let fields = &form[..form.len() - 1 - extra_after_sign];
    let view: Vec<(&str, &str)> = fields
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    assert_eq!(form[fields.len()].0, "sign");
    assert_eq!(form[fields.len()].1, generate_bilibili_sign(&view));
}

fn bilibili_server(login: serde_json::Value) -> impl Fn(&Recorded) -> Reply + Send + Sync {
    move |req: &Recorded| {
        if let Some(reply) = mint_routes(req) {
            return reply;
        }
        match req.path.as_str() {
            "/bili/issue/cipher/v3" => {
                Reply::json(&json!({ "cipher_key": TEST_PUBLIC_KEY, "hash": "salt-" }))
            }
            "/bili/login/v3" | "/bili/login/otp/v3" => Reply::json(&login),
            "/bili/otp/send/v3" => Reply::json(&json!({ "code": 0, "captcha_key": "ck" })),
            _ => Reply::status(404, &json!({})),
        }
    }
}

#[tokio::test]
async fn bilibili_password_login_encrypts_hash_and_password() {
    let server = serve(bilibili_server(
        json!({ "code": 0, "uid": 98_765, "access_key": "ak" }),
    ))
    .await;
    let up = upstream(&server.base);
    let s = session::login_bilibili(&up, "doc", "hunter2")
        .await
        .unwrap();
    assert_eq!((&*s.yostar_uid, &*s.yostar_token), ("98765", "ak"));
    assert_eq!(
        server.paths(),
        [
            "/bili/issue/cipher/v3",
            "/bili/login/v3",
            "/u8/user/v1/getToken",
            "/gs/account/login"
        ]
    );

    let reqs = server.requests();
    for req in &reqs[..2] {
        assert_eq!(
            req.header("content-type"),
            Some("application/x-www-form-urlencoded")
        );
    }
    let cipher = reqs[0].form();
    let keys: Vec<&str> = cipher.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(
        keys,
        [
            "merchant_id",
            "game_id",
            "server_id",
            "version",
            "timestamp",
            "cipher_type",
            "sign"
        ]
    );
    assert_eq!(form_value(&cipher, "cipher_type"), "bili_login_rsa");
    assert_bilibili_signed(&cipher, 0);

    let login = reqs[1].form();
    let keys: Vec<&str> = login.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(
        keys,
        [
            "merchant_id",
            "game_id",
            "server_id",
            "version",
            "timestamp",
            "bd_id",
            "user_id",
            "pwd",
            "sign"
        ]
    );
    assert_eq!(form_value(&login, "merchant_id"), "328");
    assert_eq!(form_value(&login, "game_id"), "952");
    assert_eq!(form_value(&login, "server_id"), "1178");
    assert_eq!(form_value(&login, "user_id"), "doc");
    assert_eq!(form_value(&login, "bd_id"), up.bilibili_bd_id());
    assert_bilibili_signed(&login, 0);
    let key = RsaPrivateKey::from_pkcs8_pem(TEST_PRIVATE_KEY).unwrap();
    let pwd = key
        .decrypt(
            Pkcs1v15Encrypt,
            &STANDARD.decode(form_value(&login, "pwd")).unwrap(),
        )
        .unwrap();
    assert_eq!(pwd, b"salt-hunter2");

    let u8 = reqs[2].json();
    assert_eq!(u8["channelId"], "2");
    assert_eq!(u8["extension"], r#"{"access_token":"ak","uid":"98765"}"#);
    assert_eq!(reqs[3].json()["networkVersion"], "5");
}

#[tokio::test]
async fn bilibili_risk_control_and_refusals_are_typed() {
    let server = serve(bilibili_server(json!({
        "code": "200007", "need_captch": "1", "message": "trigger account secondary verify"
    })))
    .await;
    let err = bilibili::login(&upstream(&server.base), "doc", "pw")
        .await
        .err()
        .unwrap();
    assert!(
        matches!(&err, UpstreamError::RiskControl { code, .. } if code == "200007"),
        "{err:?}"
    );

    let server = serve(bilibili_server(json!({
        "code": 500_002, "message": "PWD_INVALID", "server_message": ""
    })))
    .await;
    let err = bilibili::login(&upstream(&server.base), "doc", "pw")
        .await
        .err()
        .unwrap();
    assert!(
        matches!(&err, UpstreamError::LoginRejected { context: "bilibili::login/v3", code, message } if code == "500002" && message == "PWD_INVALID"),
        "{err:?}"
    );
}

#[tokio::test]
async fn bilibili_sms_login_carries_the_challenge_device() {
    let server = serve(bilibili_server(
        json!({ "uid": "4242", "access_key": "ak2" }),
    ))
    .await;
    let up = upstream(&server.base);

    let challenge = session::send_bilibili_sms(&up, "2065550100", "1")
        .await
        .unwrap();
    assert_eq!(
        (challenge.phone.as_str(), challenge.country_code.as_str()),
        ("2065550100", "1")
    );
    assert!(!challenge.is_expired(challenge.issued_at));
    let s = session::login_bilibili_sms(&up, &challenge, "654321")
        .await
        .unwrap();
    assert_eq!(&*s.yostar_uid, "4242");

    let sent = server.to("/bili/otp/send/v3")[0].form();
    assert_bilibili_signed(&sent, 0);
    assert_eq!(form_value(&sent, "otp_channel_no"), "2065550100");
    assert_eq!(form_value(&sent, "otp_type"), "login");
    assert_eq!(form_value(&sent, "otp_channel_category"), "tel");
    assert_eq!(form_value(&sent, "country_code"), "1");
    assert_eq!(
        form_value(&sent, "domain"),
        "line1-sdk-center-login-sh.biligame.net"
    );
    assert_eq!(form_value(&sent, "timestamp").len(), 13, "milliseconds");

    let login = server.to("/bili/login/otp/v3")[0].form();
    assert_eq!(login.last().unwrap().0, "feign_sign");
    assert_bilibili_signed(&login, 1);
    assert_eq!(
        STANDARD
            .decode(form_value(&login, "feign_sign"))
            .unwrap()
            .len(),
        128
    );
    assert_eq!(form_value(&login, "captcha_key"), "ck");
    assert_eq!(form_value(&login, "mobile"), "2065550100");
    assert_eq!(form_value(&login, "otp"), "654321");
    for field in ["cur_buvid", "udid", "bd_id"] {
        assert_eq!(
            form_value(&login, field),
            form_value(&sent, field),
            "{field}"
        );
    }
}

#[tokio::test]
async fn bilibili_rejection_carries_the_server_message() {
    let server = serve(bilibili_server(json!({
        "code": 500_002, "message": "PWD_INVALID", "server_message": "wrong"
    })))
    .await;
    let err = bilibili::login(&upstream(&server.base), "doc", "pw")
        .await
        .err()
        .unwrap();
    assert_eq!(
        err.to_string(),
        "bilibili::login/v3 rejected (code 500002): PWD_INVALID (wrong)"
    );
}

#[tokio::test]
async fn bilibili_otp_fields_carry_the_arknights_ids_and_country_code() {
    let server = serve(bilibili_server(
        json!({ "uid": "4242", "access_key": "ak2" }),
    ))
    .await;
    let up = upstream(&server.base);
    let challenge = bilibili::send_otp(&up, "2065550100", "1").await.unwrap();
    bilibili::login_otp(&up, &challenge, "654321")
        .await
        .unwrap();
    let device_id = serde_json::to_value(&challenge).unwrap()["device_id"]
        .as_str()
        .unwrap()
        .to_owned();

    for path in ["/bili/otp/send/v3", "/bili/login/otp/v3"] {
        let form = server.to(path)[0].form();
        assert_eq!(form_value(&form, "game_id"), "952", "{path}");
        assert_eq!(form_value(&form, "merchant_id"), "328", "{path}");
        assert_eq!(form_value(&form, "country_code"), "1", "{path}");
        assert_eq!(form_value(&form, "cur_buvid"), device_id, "{path}");
        assert_eq!(
            form_value(&form, "domain"),
            "line1-sdk-center-login-sh.biligame.net",
            "{path}"
        );
        assert_eq!(
            form_value(&form, "original_domain"),
            "https://line1-sdk-center-login-sh.biligame.net",
            "{path}"
        );
    }
}

#[tokio::test]
async fn bilibili_signed_form_appends_the_sign_of_the_fields() {
    let server = serve(bilibili_server(
        json!({ "code": 0, "uid": 1, "access_key": "ak" }),
    ))
    .await;
    bilibili::login(&upstream(&server.base), "a b&c", "pw")
        .await
        .unwrap();
    let req = &server.to("/bili/login/v3")[0];
    assert!(
        req.body
            .starts_with("merchant_id=328&game_id=952&server_id=1178&version=3&timestamp="),
        "{}",
        req.body
    );
    assert!(req.body.contains("&user_id=a%20b%26c&pwd="), "{}", req.body);
    let form = req.form();
    assert_eq!(form.last().unwrap().0, "sign");
    assert_bilibili_signed(&form, 0);
}

#[tokio::test]
async fn bilibili_device_ids_have_the_sdk_shapes() {
    let server = serve(bilibili_server(json!({}))).await;
    let up = upstream(&server.base);
    bilibili::send_otp(&up, "2065550100", "1").await.unwrap();
    let sent = server.to("/bili/otp/send/v3")[0].form();

    for bd in [form_value(&sent, "bd_id"), up.bilibili_bd_id()] {
        let lens: Vec<usize> = bd.split('-').map(str::len).collect();
        assert_eq!(lens, [8, 4, 4, 4, 12, 8, 4, 4, 4], "{bd}");
    }
    let dev = form_value(&sent, "cur_buvid");
    assert_eq!(dev.len(), 32);
    assert!(
        dev.bytes()
            .all(|b| b.is_ascii_digit() || b.is_ascii_uppercase())
    );
}

fn passport_server(grant_status: i32) -> impl Fn(&Recorded) -> Reply + Send + Sync {
    move |req: &Recorded| {
        if let Some(reply) = mint_routes(req) {
            return reply;
        }
        match req.path.as_str() {
            "/passport/general/v1/send_phone_code" => {
                Reply::json(&json!({ "status": 0, "msg": "OK" }))
            }
            "/passport/user/auth/v2/token_by_phone_code" => {
                Reply::json(&json!({ "status": 0, "data": { "token": "pp-tok" } }))
            }
            "/passport/user/oauth2/v2/grant" => Reply::json(&json!({
                "status": grant_status, "msg": "denied", "data": { "code": "grant-code", "uid": "cn-uid" }
            })),
            _ => Reply::status(500, &json!({ "token": "LEAKED-TOKEN" })),
        }
    }
}

#[tokio::test]
async fn cn_login_goes_through_passport_and_channel_one() {
    let server = serve(passport_server(0)).await;
    let up = upstream(&server.base);
    passport::send_phone_code(&up, "13800000000").await.unwrap();
    let s = session::login_cn(
        &up,
        PassportCredential::SmsCode {
            phone: "13800000000",
            code: "1234",
        },
    )
    .await
    .unwrap();
    assert_eq!((&*s.yostar_uid, &*s.yostar_token), ("cn-uid", "grant-code"));

    let reqs = server.requests();
    assert_eq!(reqs[0].json(), json!({ "phone": "13800000000", "type": 2 }));
    assert_eq!(
        reqs[1].json(),
        json!({ "phone": "13800000000", "code": "1234" })
    );
    assert_eq!(
        reqs[2].json(),
        json!({ "token": "pp-tok", "appCode": "4ca99fa6b56cc2ba", "type": 0 })
    );
    assert_eq!(reqs[3].json()["channelId"], "1");
}

#[tokio::test]
async fn a_passport_refusal_is_rejected_and_a_server_error_is_parse() {
    let server = serve(passport_server(101)).await;
    let up = upstream(&server.base);
    let err = passport::oauth2_grant(&up, "t").await.err().unwrap();
    assert!(
        matches!(&err, UpstreamError::LoginRejected { context: "passport::user/oauth2/v2/grant", code, .. } if code == "101"),
        "{err:?}"
    );
    let err = passport::login(
        &up,
        PassportCredential::Password {
            phone: "1",
            password: "2",
        },
    )
    .await
    .unwrap_err();
    assert!(
        matches!(&err, UpstreamError::Parse { message, .. } if message.contains("status=500") && message.contains("<redacted>") && !message.contains("LEAKED-TOKEN")),
        "{err:?}"
    );
}
