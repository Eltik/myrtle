//! A scripted HTTP server on localhost, an in-memory `SessionStore`, and an `Upstream` aimed at both.

#![allow(dead_code, clippy::unwrap_used)]

use std::collections::HashMap;
use std::fmt::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Once};
use std::time::Duration;

use async_trait::async_trait;
use myrtle_hypergryph::{
    AuthSession, DeviceIds, Domain, DurableCredential, NetworkConfig, SessionKey, SessionStore,
    SessionStoreError, Upstream, VersionInfo,
};
use myrtle_types::Server;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug, Clone)]
pub struct Recorded {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Recorded {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn header_names(&self) -> Vec<String> {
        self.headers
            .iter()
            .map(|(k, _)| k.to_ascii_lowercase())
            .collect()
    }

    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap()
    }

    pub fn form(&self) -> Vec<(String, String)> {
        self.body
            .split('&')
            .map(|pair| {
                let (k, v) = pair.split_once('=').unwrap();
                (
                    urlencoding::decode(k).unwrap().into_owned(),
                    urlencoding::decode(v).unwrap().into_owned(),
                )
            })
            .collect()
    }

    pub fn seqnum(&self) -> u32 {
        self.header("seqnum").unwrap().parse().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Reply {
    pub status: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
    pub delay: Duration,
    /// Headers go out at once, the body only after this
    pub body_delay: Duration,
}

impl Reply {
    pub fn json(body: &Value) -> Self {
        Self::status(200, body)
    }

    pub fn status(status: u16, body: &Value) -> Self {
        Self {
            status,
            body: body.to_string(),
            headers: Vec::new(),
            delay: Duration::ZERO,
            body_delay: Duration::ZERO,
        }
    }

    pub fn text(status: u16, body: &str) -> Self {
        Self {
            status,
            body: body.to_owned(),
            headers: Vec::new(),
            delay: Duration::ZERO,
            body_delay: Duration::ZERO,
        }
    }

    pub fn with_header(mut self, k: &str, v: &str) -> Self {
        self.headers.push((k.to_owned(), v.to_owned()));
        self
    }

    pub const fn after(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub const fn body_after(mut self, delay: Duration) -> Self {
        self.body_delay = delay;
        self
    }
}

pub struct FakeServer {
    pub base: String,
    seen: Arc<Mutex<Vec<Recorded>>>,
}

impl FakeServer {
    pub fn requests(&self) -> Vec<Recorded> {
        self.seen.lock().unwrap().clone()
    }

    pub fn paths(&self) -> Vec<String> {
        self.requests().into_iter().map(|r| r.path).collect()
    }

    pub fn to(&self, path: &str) -> Vec<Recorded> {
        self.requests()
            .into_iter()
            .filter(|r| r.path == path)
            .collect()
    }
}

/// Answers every request with `handler`'s reply and records the request first
pub async fn serve<F>(handler: F) -> FakeServer
where
    F: Fn(&Recorded) -> Reply + Send + Sync + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let handler = Arc::new(handler);
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let log = Arc::clone(&log);
            let handler = Arc::clone(&handler);
            tokio::spawn(async move {
                let Some(req) = read_request(&mut sock).await else {
                    return;
                };
                log.lock().unwrap().push(req.clone());
                let reply = handler(&req);
                if !reply.delay.is_zero() {
                    tokio::time::sleep(reply.delay).await;
                }
                let mut out = format!(
                    "HTTP/1.1 {} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
                    reply.status,
                    reply.body.len()
                );
                for (k, v) in &reply.headers {
                    let _ = write!(out, "{k}: {v}\r\n");
                }
                out.push_str("\r\n");
                if !reply.body_delay.is_zero() {
                    let _ = sock.write_all(out.as_bytes()).await;
                    let _ = sock.flush().await;
                    tokio::time::sleep(reply.body_delay).await;
                    out.clear();
                }
                out.push_str(&reply.body);
                let _ = sock.write_all(out.as_bytes()).await;
                let _ = sock.shutdown().await;
            });
        }
    });
    FakeServer {
        base: format!("http://{addr}"),
        seen,
    }
}

async fn read_request(sock: &mut tokio::net::TcpStream) -> Option<Recorded> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let head_end = loop {
        let n = sock.read(&mut chunk).await.unwrap_or(0);
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    let mut lines = head.split("\r\n");
    let mut start = lines.next()?.split(' ');
    let method = start.next()?.to_owned();
    let path = start.next()?.to_owned();
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            Some((k.trim().to_owned(), v.trim().to_owned()))
        })
        .collect();
    let len = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse::<usize>().ok())
        .unwrap_or(0);
    while buf.len() < head_end + len {
        let n = sock.read(&mut chunk).await.unwrap_or(0);
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    Some(Recorded {
        method,
        path,
        headers,
        body: String::from_utf8_lossy(&buf[head_end..]).to_string(),
    })
}

pub const DEVICE_IDS: [&str; 3] = ["dev-one", "86000000000001", "dev-three"];

pub fn device_ids() -> DeviceIds {
    DeviceIds {
        device_id: DEVICE_IDS[0].into(),
        device_id2: DEVICE_IDS[1].into(),
        device_id3: DEVICE_IDS[2].into(),
    }
}

pub fn version() -> VersionInfo {
    VersionInfo {
        res_version: "res-1".into(),
        client_version: "client-1".into(),
    }
}

/// Every route and host on `base`, versions already loaded
pub fn network_config(base: &str) -> NetworkConfig {
    let mut cfg = NetworkConfig::new(device_ids());
    for server in Server::ALL {
        let code = server.as_str();
        cfg.set_domain(server, Domain::GS, format!("{base}/gs"));
        cfg.set_domain(server, Domain::U8, format!("{base}/u8"));
        cfg.set_domain(
            server,
            Domain::HV,
            format!("{base}/hv/{code}/{{0}}/version"),
        );
        cfg.set_version(server, version());
        cfg.hosts.network_routes[server.index() as usize] = format!("{base}/conf/{code}");
        if cfg.hosts.yostar_sdk[server.index() as usize].is_some() {
            cfg.hosts.yostar_sdk[server.index() as usize] = Some(format!("{base}/sdk"));
        }
    }
    cfg.hosts.account_portal = format!("{base}/portal");
    cfg.hosts.passport = format!("{base}/passport");
    cfg.hosts.bilibili = format!("{base}/bili");
    cfg
}

pub fn client() -> reqwest::Client {
    static PROVIDER: Once = Once::new();
    PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
    reqwest::Client::builder().no_proxy().build().unwrap()
}

pub fn upstream(base: &str) -> Upstream {
    Upstream::new(client(), network_config(base))
}

pub fn durable(uid: &str, token: &str) -> DurableCredential {
    DurableCredential {
        yostar_uid: uid.into(),
        yostar_token: token.into(),
    }
}

pub fn session_with(credential: &DurableCredential) -> AuthSession {
    AuthSession::from_durable("", credential)
}

#[derive(Default)]
pub struct MemoryStore {
    sessions: Mutex<HashMap<SessionKey, AuthSession>>,
    durable: Mutex<HashMap<SessionKey, DurableCredential>>,
    saves: Mutex<Vec<AuthSession>>,
    pub fail_load: AtomicBool,
    pub fail_save: AtomicBool,
    pub fail_load_durable: AtomicBool,
}

fn fails(flag: &AtomicBool) -> Result<(), SessionStoreError> {
    if flag.load(Ordering::SeqCst) {
        Err(SessionStoreError::Unavailable("injected".into()))
    } else {
        Ok(())
    }
}

impl MemoryStore {
    pub fn put_session(&self, key: &SessionKey, session: AuthSession) {
        self.sessions.lock().unwrap().insert(key.clone(), session);
    }

    pub fn put_durable(&self, key: &SessionKey, credential: DurableCredential) {
        self.durable.lock().unwrap().insert(key.clone(), credential);
    }

    pub fn session(&self, key: &SessionKey) -> Option<AuthSession> {
        self.sessions.lock().unwrap().get(key).cloned()
    }

    pub fn saves(&self) -> Vec<AuthSession> {
        self.saves.lock().unwrap().clone()
    }
}

#[async_trait]
impl SessionStore for MemoryStore {
    async fn load(&self, key: &SessionKey) -> Result<Option<AuthSession>, SessionStoreError> {
        fails(&self.fail_load)?;
        Ok(self.session(key))
    }

    async fn save(&self, key: &SessionKey, session: &AuthSession) -> Result<(), SessionStoreError> {
        fails(&self.fail_save)?;
        self.saves.lock().unwrap().push(session.clone());
        self.put_session(key, session.clone());
        Ok(())
    }

    async fn load_durable(
        &self,
        key: &SessionKey,
    ) -> Result<Option<DurableCredential>, SessionStoreError> {
        fails(&self.fail_load_durable)?;
        Ok(self.durable.lock().unwrap().get(key).cloned())
    }

    async fn save_durable(
        &self,
        key: &SessionKey,
        credential: &DurableCredential,
    ) -> Result<(), SessionStoreError> {
        self.put_durable(key, credential.clone());
        Ok(())
    }

    async fn forget(&self, key: &SessionKey) -> Result<bool, SessionStoreError> {
        let a = self.sessions.lock().unwrap().remove(key).is_some();
        let b = self.durable.lock().unwrap().remove(key).is_some();
        Ok(a || b)
    }
}

/// Yostar tokens `getToken` accepts; anything else is refused with `result: 1`
#[derive(Clone, Default)]
pub struct ValidTokens(Arc<Mutex<Vec<String>>>);

impl ValidTokens {
    pub fn new(tokens: &[&str]) -> Self {
        Self(Arc::new(Mutex::new(
            tokens.iter().map(|t| (*t).to_owned()).collect(),
        )))
    }

    pub fn allow(&self, token: &str) {
        self.0.lock().unwrap().push(token.to_owned());
    }

    pub fn revoke(&self, token: &str) {
        self.0.lock().unwrap().retain(|t| t != token);
    }

    fn accepts(&self, token: &str) -> bool {
        self.0.lock().unwrap().iter().any(|t| t == token)
    }
}

/// The durable token a `getToken` request carries in its `extension`
pub fn extension_token(req: &Recorded) -> String {
    let extension: Value = serde_json::from_str(req.json()["extension"].as_str().unwrap()).unwrap();
    extension
        .get("token")
        .or_else(|| extension.get("access_token"))
        .and_then(Value::as_str)
        .unwrap()
        .to_owned()
}

/// `getToken` gated on `valid`, `account/login` minting `secret-N`, other gs routes from `gs(request, nth)`
pub fn game_server<F>(
    valid: ValidTokens,
    gs: F,
) -> impl Fn(&Recorded) -> Reply + Send + Sync + 'static
where
    F: Fn(&Recorded, usize) -> Reply + Send + Sync + 'static,
{
    let logins = std::sync::atomic::AtomicUsize::new(0);
    let gs_calls = std::sync::atomic::AtomicUsize::new(0);
    move |req: &Recorded| match req.path.as_str() {
        "/u8/user/v1/getToken" => {
            let token = extension_token(req);
            if valid.accepts(&token) {
                Reply::json(
                    &serde_json::json!({ "result": 0, "uid": "u8-uid", "token": format!("u8-{token}") }),
                )
            } else {
                Reply::json(&serde_json::json!({ "result": 1 }))
            }
        }
        "/gs/account/login" => {
            let n = logins.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            Reply::json(
                &serde_json::json!({ "result": 0, "uid": "u8-uid", "secret": format!("secret-{n}") }),
            )
        }
        _ => gs(
            req,
            gs_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        ),
    }
}

/// The MD5 signature Yostar expects over the auth header's head, the body and the client secret
pub fn assert_signed(req: &Recorded, uid: &str, token: &str) {
    use md5::{Digest, Md5};
    let auth: Value = serde_json::from_str(req.header("authorization").unwrap()).unwrap();
    let head = &auth["Head"];
    assert_eq!(head["UID"], uid, "{}", req.path);
    assert_eq!(head["Token"], token, "{}", req.path);
    assert_eq!(head["DeviceID"], DEVICE_IDS[0]);
    let head_json = req
        .header("authorization")
        .unwrap()
        .strip_prefix(r#"{"Head":"#)
        .unwrap()
        .rsplit_once(r#","Sign":"#)
        .unwrap()
        .0
        .to_owned();
    let mut md5 = Md5::new();
    md5.update(format!(
        "{head_json}{}886c085e4a8d30a703367b120dd8353948405ec2",
        req.body
    ));
    assert_eq!(
        auth["Sign"],
        format!("{:X}", md5.finalize()),
        "{}",
        req.path
    );
}
