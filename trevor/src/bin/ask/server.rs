//! The llama-server connection: connect, or start one under the model lock and stop it on exit.

use std::path::PathBuf;

use anyhow::{Context, Result};
use trevor::goldgen::llm::Llm;

use crate::cli::Args;

/// A llama-server this process started; killed when dropped, so it never outlives `ask`.
pub(crate) struct Spawned(std::process::Child, Option<PathBuf>);

impl Drop for Spawned {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
        // The lock this process wrote for its server, only while it still names this process.
        if let Some(l) = &self.1 {
            let me = std::process::id().to_string();
            if std::fs::read_to_string(l).is_ok_and(|t| t.split_whitespace().next() == Some(me.as_str())) {
                let _ = std::fs::remove_file(l);
            }
        }
    }
}

/// The model lock's holder, when it is another job: (pid, label). A lock whose pid is this process or one of its
/// ancestors is ours (the wrapper script that took the lock and started `ask`).
pub(crate) fn foreign_lock(path: &std::path::Path) -> Option<(u32, String)> {
    let text = std::fs::read_to_string(path).ok()?;
    let (pid, label) = text.trim().split_once(' ').unwrap_or((text.trim(), ""));
    let pid: u32 = pid.parse().ok()?;
    let mut p = std::process::id();
    for _ in 0..64 {
        if p == pid {
            return None;
        }
        let out = std::process::Command::new("ps").args(["-o", "ppid=", "-p", &p.to_string()]).output().ok()?;
        match String::from_utf8_lossy(&out.stdout).trim().parse::<u32>() {
            Ok(pp) if pp > 1 && pp != p => p = pp,
            _ => break,
        }
    }
    Some((pid, label.to_owned()))
}

/// Wait while another job holds the model lock (or fail with `--no-wait`).
pub(crate) async fn wait_for_lock(a: &Args) -> Result<()> {
    if a.ignore_lock {
        return Ok(());
    }
    let mut told = false;
    while let Some((pid, label)) = foreign_lock(&a.lock) {
        if a.no_wait {
            anyhow::bail!("the model is busy with {label} (pid {pid}, lock {}); --no-wait was given", a.lock.display());
        }
        if !told {
            eprintln!("the model is busy with {label} (pid {pid}); waiting for {} to go", a.lock.display());
            told = true;
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
    Ok(())
}

/// Connect to the server at `a.server`, or start one on its port with the answer model and wait
/// until it is healthy (the model loads in about 15 s). Waits first while another job holds the model lock.
pub(crate) async fn connect_or_spawn(a: &Args) -> Result<(Llm, Option<Spawned>)> {
    let (llm, guard) = connect_or_spawn_plain(a).await?;
    let llm = if a.no_cache_prompt { llm.without_prompt_cache() } else { llm };
    let llm = match &a.dump_prompt { Some(p) => llm.with_dump(p)?, None => llm };
    Ok((llm, guard))
}

pub(crate) async fn connect_or_spawn_plain(a: &Args) -> Result<(Llm, Option<Spawned>)> {
    wait_for_lock(a).await?;
    if let Ok(llm) = Llm::connect(&a.server).await {
        return Ok((llm, None));
    }
    if a.no_spawn {
        return Err(anyhow::anyhow!("no llama-server at {} (and --no-spawn was given)", a.server));
    }
    let port = a.server.rsplit(':').next().unwrap_or("8081").trim_end_matches('/').to_owned();
    eprintln!("no llama-server at {}; starting {} (about 15 s)...", a.server, a.model.display());
    let child = std::process::Command::new("llama-server")
        .args(["-m"]).arg(&a.model)
        .args(["--host", "127.0.0.1", "--port", &port, "-np", "1", "--kv-unified-per-slot", "16384", "-fa", "on",
               "-cram", "512", "--no-webui", "--reasoning", "off", "-ngl", "all"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("starting llama-server (brew install llama.cpp)")?;
    let guard = Spawned(child, (!a.ignore_lock).then(|| a.lock.clone()));
    if let Some(l) = &guard.1 {
        let _ = std::fs::write(l, format!("{} ask\n", std::process::id()));
    }
    for _ in 0..180 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if let Ok(llm) = Llm::connect(&a.server).await {
            return Ok((llm, Some(guard)));
        }
    }
    Err(anyhow::anyhow!("llama-server did not become ready in 180 s"))
}
