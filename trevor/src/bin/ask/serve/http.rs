//! `--serve`: the local HTTP API (`GET /health`, `POST /v1/ask`), one request per connection.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::answer::Tables;
use crate::chrono::Chrono;
use crate::cli::Args;

use super::{AskRequest, Need, Service};
use super::worker::pull;

/// `--serve` or `--worker`: load once, then serve until SIGINT or SIGTERM (the llama-server this process started, if
/// any, stops with it).
pub(crate) async fn run(a: Args, corpus_default: bool, tables: Tables, chrono: Chrono) -> Result<()> {
    let worker = a.worker.clone();
    let port = a.port;
    let need = if worker.is_some() { Need::Local } else { Need::Model };
    let mut svc = Service::load(a, corpus_default, tables, chrono, need).await?;
    match worker {
        Some(base) => pull(&mut svc, &base).await,
        None => listen(&mut svc, port).await,
    }
}

async fn listen(svc: &mut Service, port: u16) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await.with_context(|| format!("binding 127.0.0.1:{port}"))?;
    eprintln!("serve: http://127.0.0.1:{port} ({}, model {})", svc.corpus_version, svc.llm.as_ref().map_or("none", |l| l.model.as_str()));
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    loop {
        let mut s = tokio::select! {
            r = listener.accept() => r?.0,
            _ = tokio::signal::ctrl_c() => break,
            _ = term.recv() => break,
        };
        if let Err(e) = handle(svc, &mut s).await {
            eprintln!("serve: {e:#}");
        }
    }
    eprintln!("serve: stopping");
    Ok(())
}

/// One request: read it, answer it, close.
async fn handle(svc: &mut Service, s: &mut TcpStream) -> Result<()> {
    let (method, path, body) = tokio::time::timeout(Duration::from_secs(10), read_request(s)).await.context("request timed out")??;
    let (code, json) = match (method.as_str(), path.as_str()) {
        ("GET", "/health") => (200, svc.health()),
        ("POST", "/v1/ask") => match serde_json::from_slice::<AskRequest>(&body) {
            Err(e) => (400, serde_json::json!({"error": e.to_string()})),
            Ok(req) => match svc.ask(req, None).await {
                Ok(j) => (200, serde_json::to_value(j.result)?),
                Err((code, e)) => (code, serde_json::json!({"error": e})),
            },
        },
        _ => (404, serde_json::json!({"error": "not found"})),
    };
    let body = serde_json::to_vec(&json)?;
    let reason = match code { 200 => "OK", 400 => "Bad Request", 404 => "Not Found", 503 => "Service Unavailable", _ => "Internal Server Error" };
    let head = format!("HTTP/1.1 {code} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    s.write_all(head.as_bytes()).await?;
    s.write_all(&body).await?;
    s.shutdown().await?;
    Ok(())
}

/// The request line's method and path and the body (by `Content-Length`, at most 64 KiB).
pub(crate) async fn read_request(s: &mut (impl AsyncReadExt + Unpin)) -> Result<(String, String, Vec<u8>)> {
    const MAX: usize = 64 * 1024;
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    let end = loop {
        let n = s.read(&mut tmp).await?;
        if n == 0 {
            bail!("connection closed before the headers ended");
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        if buf.len() > MAX {
            bail!("headers too large");
        }
    };
    let head = String::from_utf8_lossy(&buf[..end]).into_owned();
    let mut lines = head.lines();
    let mut first = lines.next().unwrap_or_default().split_whitespace();
    let (method, path) = (first.next().unwrap_or_default().to_owned(), first.next().unwrap_or_default().to_owned());
    let len = lines.filter_map(|l| l.split_once(':')).find(|(k, _)| k.trim().eq_ignore_ascii_case("content-length"))
        .map_or(Ok(0), |(_, v)| v.trim().parse::<usize>()).context("bad Content-Length")?;
    if len > MAX {
        bail!("body too large");
    }
    let mut body = buf[end + 4..].to_vec();
    while body.len() < len {
        let n = s.read(&mut tmp).await?;
        if n == 0 {
            bail!("connection closed before the body ended");
        }
        body.extend_from_slice(&tmp[..n]);
    }
    body.truncate(len);
    Ok((method, path, body))
}
