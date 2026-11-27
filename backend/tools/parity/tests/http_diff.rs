//! Two local servers on raw tokio sockets: deny.toml allows axum only under myrtle-api
#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use myrtle_parity::compare::DiffKind;
use myrtle_parity::corpus::{CorpusEntry, Source, write_corpus};
use myrtle_parity::http::{HttpOptions, Target, Tokens};
use myrtle_parity::report::Summary;
use myrtle_parity::rules::{RuleOverrides, Rules};
use myrtle_parity::run::{self, CompareRun, Recording, Side};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const TOKEN: &str = "test-token-7f3a9c";

#[derive(Clone)]
struct Reply {
    status: u16,
    content_type: &'static str,
    body: Vec<u8>,
    delay: Duration,
}

fn json(body: &str) -> Reply {
    Reply {
        status: 200,
        content_type: "application/json",
        body: body.as_bytes().to_vec(),
        delay: Duration::ZERO,
    }
}

fn reply(status: u16, content_type: &'static str, body: &[u8]) -> Reply {
    Reply {
        status,
        content_type,
        body: body.to_vec(),
        delay: Duration::ZERO,
    }
}

struct Server {
    addr: SocketAddr,
    auth_seen: Arc<Mutex<Vec<String>>>,
}

impl Server {
    fn base(&self) -> String {
        format!("http://{}", self.addr)
    }
}

async fn serve(routes: HashMap<&'static str, Reply>) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let routes = Arc::new(routes);
    let auth_seen = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&auth_seen);
    tokio::spawn(async move {
        loop {
            let Ok((sock, _)) = listener.accept().await else {
                return;
            };
            tokio::spawn(handle(sock, Arc::clone(&routes), Arc::clone(&seen)));
        }
    });
    Server { addr, auth_seen }
}

async fn handle(
    mut sock: TcpStream,
    routes: Arc<HashMap<&'static str, Reply>>,
    seen: Arc<Mutex<Vec<String>>>,
) {
    let mut buf = Vec::new();
    let mut chunk = [0_u8; 2048];
    while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
        match sock.read(&mut chunk).await {
            Ok(0) | Err(_) => return,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    }
    let head = String::from_utf8_lossy(&buf).into_owned();
    let mut lines = head.split("\r\n");
    let target = lines
        .next()
        .and_then(|l| l.split(' ').nth(1))
        .unwrap_or("/")
        .to_owned();
    for line in lines {
        if let Some((k, v)) = line.split_once(':')
            && k.eq_ignore_ascii_case("authorization")
        {
            seen.lock().unwrap().push(v.trim().to_owned());
        }
    }
    let r = routes
        .get(target.as_str())
        .cloned()
        .unwrap_or_else(|| reply(404, "application/json", br#"{"error":"not found"}"#));
    if !r.delay.is_zero() {
        tokio::time::sleep(r.delay).await;
    }
    let head = format!(
        "HTTP/1.1 {} X\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        r.status,
        r.content_type,
        r.body.len()
    );
    let _ = sock.write_all(head.as_bytes()).await;
    let _ = sock.write_all(&r.body).await;
    let _ = sock.shutdown().await;
}

/// The legacy side
fn routes_a() -> HashMap<&'static str, Reply> {
    HashMap::from([
        (
            "/api/same",
            json(r#"{"name":"Amiya","atk":612.4,"tags":["caster","starter"]}"#),
        ),
        (
            "/api/stats",
            json(r#"{"stats":{"atk":612.4,"def":100,"hp":[1,2,3]}}"#),
        ),
        ("/api/keys", json(r#"{"a":1,"gone":true}"#)),
        (
            "/api/drops",
            json(r#"{"drops":[{"id":1,"p":0.25},{"id":2,"p":0.75}]}"#),
        ),
        ("/api/img", reply(200, "image/png", b"\x89PNG legacy")),
        ("/api/img-same", reply(200, "image/webp", b"RIFF same")),
        ("/api/status", json(r#"{"ok":true}"#)),
        ("/api/ctype", json("[1]")),
        ("/api/me", json(r#"{"me":"doctor"}"#)),
    ])
}

/// The rewrite side, slightly different
fn routes_b() -> HashMap<&'static str, Reply> {
    HashMap::from([
        (
            "/api/same", // keys reordered, float one ulp off
            json(r#"{"tags":["caster","starter"],"atk":612.4000000000001,"name":"Amiya"}"#),
        ),
        (
            "/api/stats",
            json(r#"{"stats":{"def":100,"atk":613.0,"hp":[1,2,4]}}"#),
        ),
        ("/api/keys", json(r#"{"a":1,"added":null}"#)),
        (
            "/api/drops", // equal only with /drops unordered
            json(r#"{"drops":[{"p":0.75,"id":2},{"id":1,"p":0.25}]}"#),
        ),
        ("/api/img", reply(200, "image/png", b"\x89PNG rewrite")),
        ("/api/img-same", reply(200, "image/webp", b"RIFF same")),
        (
            "/api/status",
            reply(404, "application/json", br#"{"error":"not found"}"#),
        ),
        (
            "/api/ctype",
            reply(200, "text/plain; charset=utf-8", b"[1]"),
        ),
        ("/api/me", json(r#"{"me":"doctor"}"#)),
    ])
}

fn corpus() -> Vec<CorpusEntry> {
    let e = |path: &str| CorpusEntry::new(path, path, None, Source::Openapi);
    vec![
        e("/api/same"),
        e("/api/stats"),
        e("/api/keys"),
        e("/api/drops"),
        e("/api/img"),
        e("/api/img-same"),
        e("/api/status"),
        e("/api/ctype"),
        CorpusEntry::new("/api/me", "/api/me", Some("user"), Source::Openapi),
    ]
}

const fn http_opts() -> HttpOptions {
    HttpOptions {
        concurrency: 3,
        timeout: Duration::from_secs(10),
        rps: 0.0,
        no_proxy: true,
    }
}

fn tokens() -> Tokens {
    Tokens::from_lookup(["user"], None, |name| {
        (name == "PARITY_TOKEN_USER").then(|| TOKEN.to_owned())
    })
}

fn rules(unordered: &[&str]) -> Rules {
    Rules::build(
        None,
        &RuleOverrides {
            unordered: unordered.iter().map(|s| (*s).to_owned()).collect(),
            max_diffs: 10,
            ..RuleOverrides::default()
        },
    )
    .unwrap()
}

const fn run_opts() -> CompareRun {
    CompareRun {
        concurrency: 3,
        max_problems: 50,
        progress: false,
    }
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("parity-it-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// (path, [(pointer, kind)]) per problem, in corpus order
fn problem_table(s: &Summary) -> Vec<(String, Vec<(String, DiffKind)>)> {
    s.problems
        .iter()
        .map(|p| {
            (
                p.path.clone(),
                p.diffs
                    .iter()
                    .map(|d| (d.pointer.clone(), d.kind))
                    .collect(),
            )
        })
        .collect()
}

fn expected_problems() -> Vec<(String, Vec<(String, DiffKind)>)> {
    let p = |path: &str, diffs: &[(&str, DiffKind)]| {
        (
            path.to_owned(),
            diffs
                .iter()
                .map(|(ptr, k)| ((*ptr).to_owned(), *k))
                .collect(),
        )
    };
    vec![
        p(
            "/api/stats",
            &[
                ("/stats/atk", DiffKind::Number),
                ("/stats/hp/2", DiffKind::Number),
            ],
        ),
        p(
            "/api/keys",
            &[
                ("/gone", DiffKind::MissingInB),
                ("/added", DiffKind::MissingInA),
            ],
        ),
        p("/api/img", &[("", DiffKind::BodyHash)]),
        p("/api/status", &[("", DiffKind::Status)]),
        p(
            "/api/ctype",
            &[("", DiffKind::ContentType), ("", DiffKind::BodyHash)],
        ),
    ]
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn diff_two_live_servers_reports_pointers() {
    let a = serve(routes_a()).await;
    let b = serve(routes_b()).await;
    let ta = Target::new(&a.base(), None, tokens(), &http_opts()).unwrap();
    let tb = Target::new(&b.base(), None, tokens(), &http_opts()).unwrap();
    let corpus = corpus();

    let s = run::compare(
        &Side::Live(&ta),
        &Side::Live(&tb),
        &corpus,
        &rules(&["/drops"]),
        &run_opts(),
    )
    .await;
    assert_eq!(
        (
            s.total,
            s.equal,
            s.different,
            s.errors,
            s.skipped,
            s.rate_limited
        ),
        (9, 4, 5, 0, 0, 0),
        "{}",
        s.render_text()
    );
    assert_eq!(problem_table(&s), expected_problems());
    assert!(!s.passed());
    assert_eq!(s.routes["/api/stats"].different, 1);
    assert_eq!(s.routes["/api/same"].equal, 1);

    let stats = &s.problems[0].diffs[0];
    assert_eq!(
        (stats.a.as_deref(), stats.b.as_deref()),
        (Some("612.4"), Some("613.0"))
    );

    let strict = run::compare(
        &Side::Live(&ta),
        &Side::Live(&tb),
        &corpus,
        &rules(&[]),
        &run_opts(),
    )
    .await;
    assert_eq!(strict.different, 6);
    assert!(
        strict
            .problems
            .iter()
            .any(|p| p.path == "/api/drops" && p.diffs.iter().any(|d| d.pointer == "/drops/0/id"))
    );

    for server in [&a, &b] {
        let seen = server.auth_seen.lock().unwrap().clone();
        assert!(
            seen.iter().all(|v| v == &format!("Bearer {TOKEN}")),
            "{seen:?}"
        );
        assert_eq!(seen.len(), 2);
    }
    let text = s.render_text();
    let json = serde_json::to_string(&s).unwrap();
    assert!(!text.contains(TOKEN) && !json.contains(TOKEN));
    assert!(
        text.contains("/stats/atk: number outside tolerance  A=612.4  B=613.0"),
        "{text}"
    );
    assert!(text.contains("result: FAIL"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn record_then_replay_and_offline_compare() {
    let a = serve(routes_a()).await;
    let b = serve(routes_b()).await;
    let ta = Target::new(&a.base(), None, tokens(), &http_opts()).unwrap();
    let tb = Target::new(&b.base(), None, tokens(), &http_opts()).unwrap();
    let corpus = corpus();
    let dir = temp_dir("record");
    let rec_a = dir.join("a");
    let rec_b = dir.join("b");

    let stats = run::record(&ta, &corpus, &rec_a, 3, false, false, false)
        .await
        .unwrap();
    assert_eq!(
        (stats.total, stats.ok, stats.errors, stats.skipped),
        (9, 9, 0, 0)
    );
    let bodies = std::fs::read_dir(rec_a.join("bodies")).unwrap().count();
    assert_eq!(bodies, 7);
    assert!(
        rec_a
            .join("bodies")
            .join(format!("{}.json", corpus[0].id))
            .exists()
    );
    assert!(
        !rec_a
            .join("bodies")
            .join(format!("{}.bin", corpus[4].id))
            .exists()
    );
    let recorded = std::fs::read_to_string(rec_a.join("responses.jsonl")).unwrap();
    assert!(!recorded.contains(TOKEN));
    assert!(recorded.contains(r#""role":"user""#));
    assert!(
        run::record(&ta, &corpus, &rec_a, 3, false, false, false)
            .await
            .is_err()
    );

    let recording_a = Recording::open(&rec_a).unwrap();
    let same = run::compare(
        &Side::Recorded(&recording_a),
        &Side::Live(&ta),
        &corpus,
        &rules(&[]),
        &run_opts(),
    )
    .await;
    assert!(same.passed(), "{}", same.render_text());
    assert_eq!(same.equal, 9);

    let replay = run::compare(
        &Side::Recorded(&recording_a),
        &Side::Live(&tb),
        &corpus,
        &rules(&["/drops"]),
        &run_opts(),
    )
    .await;
    assert_eq!(problem_table(&replay), expected_problems());

    run::record(&tb, &corpus, &rec_b, 3, true, false, false)
        .await
        .unwrap();
    assert_eq!(std::fs::read_dir(rec_b.join("bodies")).unwrap().count(), 9);
    let recording_b = Recording::open(&rec_b).unwrap();
    let offline = run::compare(
        &Side::Recorded(&recording_a),
        &Side::Recorded(&recording_b),
        &corpus,
        &rules(&["/drops"]),
        &run_opts(),
    )
    .await;
    assert_eq!(problem_table(&offline), expected_problems());

    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn timeouts_and_missing_tokens() {
    let mut slow_routes = routes_a();
    slow_routes.insert(
        "/api/slow",
        Reply {
            delay: Duration::from_secs(5),
            ..json("{}")
        },
    );
    let a = serve(slow_routes.clone()).await;
    let b = serve(slow_routes).await;
    let opts = HttpOptions {
        timeout: Duration::from_millis(300),
        ..http_opts()
    };
    let ta = Target::new(&a.base(), None, Tokens::default(), &opts).unwrap();
    let tb = Target::new(&b.base(), None, Tokens::default(), &opts).unwrap();
    let corpus = vec![
        CorpusEntry::new("/api/slow", "/api/slow", None, Source::Openapi),
        CorpusEntry::new("/api/me", "/api/me", Some("user"), Source::Openapi),
        CorpusEntry::new("/api/same", "/api/same", None, Source::Openapi),
    ];
    let s = run::compare(
        &Side::Live(&ta),
        &Side::Live(&tb),
        &corpus,
        &rules(&[]),
        &run_opts(),
    )
    .await;
    assert_eq!(
        (s.equal, s.errors, s.skipped),
        (1, 1, 1),
        "{}",
        s.render_text()
    );
    let msg = s.problems[0].message.as_deref().unwrap();
    assert!(msg.starts_with("A: timeout"), "{msg}");
    assert!(
        s.skip_reasons
            .keys()
            .any(|r| r.contains("PARITY_TOKEN_USER"))
    );
    assert!(a.auth_seen.lock().unwrap().is_empty());
}

fn parity_bin() -> std::process::Command {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_parity"));
    cmd.env_remove("PARITY_TOKEN_USER")
        .env_remove("PARITY_A_TOKEN_USER")
        .env_remove("PARITY_B_TOKEN_USER");
    cmd
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn binary_exit_codes_and_report() {
    let a = serve(routes_a()).await;
    let b = serve(routes_b()).await;
    let dir = temp_dir("bin");
    let corpus_file = dir.join("corpus.jsonl");
    write_corpus(&corpus_file, &corpus()).unwrap();
    let summary_file = dir.join("summary.json");

    let (base_a, base_b) = (a.base(), b.base());
    let (cf, sf) = (corpus_file.clone(), summary_file.clone());
    let out = tokio::task::spawn_blocking(move || {
        parity_bin()
            .args([
                "diff",
                "--a",
                &base_a,
                "--b",
                &base_b,
                "--no-proxy",
                "--quiet",
                "--unordered",
                "/drops",
            ])
            .arg("--corpus")
            .arg(&cf)
            .arg("--summary")
            .arg(&sf)
            .env("PARITY_TOKEN_USER", TOKEN)
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(
        stdout.contains("requests 9  equal 4  different 5"),
        "{stdout}"
    );
    assert!(
        stdout.contains("/stats/atk: number outside tolerance"),
        "{stdout}"
    );
    assert!(stdout.contains("status differs  A=200  B=404"), "{stdout}");
    assert!(!stdout.contains(TOKEN) && !stderr.contains(TOKEN));
    let summary: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&summary_file).unwrap()).unwrap();
    assert_eq!(summary["different"], 5);
    assert_eq!(summary["problems"][1]["diffs"][0]["pointer"], "/gone");
    assert_eq!(summary["problems"][1]["diffs"][0]["kind"], "missing_in_b");
    assert!(!summary.to_string().contains(TOKEN));

    let (base_a, cf) = (a.base(), corpus_file.clone());
    let out = tokio::task::spawn_blocking(move || {
        parity_bin()
            .args([
                "diff",
                "--a",
                &base_a,
                "--b",
                &base_a,
                "--no-proxy",
                "--quiet",
            ])
            .arg("--corpus")
            .arg(&cf)
            .env("PARITY_TOKEN_USER", TOKEN)
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );

    let (base_a, cf, missing) = (a.base(), corpus_file.clone(), dir.join("nope"));
    let out = tokio::task::spawn_blocking(move || {
        parity_bin()
            .args(["replay", "--base", &base_a, "--no-proxy"])
            .arg("--corpus")
            .arg(&cf)
            .arg("--against")
            .arg(&missing)
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("not a recording"));

    std::fs::remove_dir_all(&dir).unwrap();
}
