//! `downloader -s cn client-download`: fetch the official CN Windows client's
//! IL2CPP pair (`GameAssembly.dll` + `global-metadata.dat`) for
//! `unpacker schema-gen`.
//!
//! The launcher endpoint answers unauthenticated with the current client
//! version and the package base URL (`.pkg.file_path`). The two files sit at
//! fixed paths under that base; on 2026-10-15 (client 2.7.81) they were
//! 218,523,112 and 45,805,368 bytes. Only CN publishes a desktop client; the
//! Yostar servers ship Google Play only, so they are refused here.
//!
//! Output: the two files plus `client.json` (version, base URL, size and
//! SHA-256 of each file, download time in RFC 3339 UTC).

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::server::Server;

/// Launcher endpoint for the CN Windows client (`appcode` is the official
/// Arknights launcher's, channel 1 = official, not Bilibili).
pub const LAUNCHER_URL: &str = "https://launcher.hypergryph.com/api/game/get_latest?appcode=GzD1CpaWgmSq1wew&channel=1&sub_channel=1&platform=Windows";

/// The two files `schema-gen` reads, as (local name, path under `file_path`).
pub const CLIENT_FILES: [(&str, &str); 2] = [
    ("GameAssembly.dll", "GameAssembly.dll"),
    (
        "global-metadata.dat",
        "Arknights_Data/il2cpp_data/Metadata/global-metadata.dat",
    ),
];

#[derive(Debug, Deserialize)]
struct LauncherResponse {
    client_version: String,
    pkg: LauncherPkg,
}

#[derive(Debug, Deserialize)]
struct LauncherPkg {
    file_path: String,
}

/// One downloaded file in `client.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientFile {
    pub size: u64,
    pub sha256: String,
}

/// `<dir>/client.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientManifest {
    pub server: String,
    pub platform: String,
    pub client_version: String,
    pub file_path: String,
    pub files: BTreeMap<String, ClientFile>,
    pub downloaded_at: String,
}

/// Run `client-download` into `dir`. Returns the written manifest.
///
/// # Errors
///
/// Refuses every server but CN official and every platform but `windows`;
/// fails on any HTTP or file error, or when a body is shorter than its
/// `Content-Length`.
pub async fn run(server: Server, platform: &str, dir: &Path) -> Result<ClientManifest> {
    if server != Server::Official {
        bail!(
            "client-download supports only `-s cn` (the official CN client); `{server}` has no \
             desktop client to download (the Yostar servers ship Google Play builds only)"
        );
    }
    if !platform.eq_ignore_ascii_case("windows") {
        bail!("client-download supports only `--platform windows`, got `{platform}`");
    }
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .context("building the HTTP client")?;
    let launcher: LauncherResponse = client
        .get(LAUNCHER_URL)
        .send()
        .await
        .context("requesting the launcher endpoint")?
        .error_for_status()?
        .json()
        .await
        .context("parsing the launcher response (.client_version, .pkg.file_path)")?;
    println!("Client: {}", launcher.client_version);

    tokio::fs::create_dir_all(dir)
        .await
        .with_context(|| format!("creating {}", dir.display()))?;
    let base = launcher.pkg.file_path.trim_end_matches('/');
    let mut files = BTreeMap::new();
    for (name, rel) in CLIENT_FILES {
        let url = format!("{base}/{rel}");
        let dest = dir.join(name);
        let info = fetch_file(&client, &url, &dest)
            .await
            .with_context(|| format!("downloading {url}"))?;
        files.insert(name.to_string(), info);
    }

    let manifest = ClientManifest {
        server: "cn".to_string(),
        platform: "windows".to_string(),
        client_version: launcher.client_version,
        file_path: launcher.pkg.file_path,
        files,
        downloaded_at: rfc3339_utc_now(),
    };
    let json = serde_json::to_string_pretty(&manifest)? + "\n";
    tokio::fs::write(dir.join("client.json"), json).await?;
    Ok(manifest)
}

/// Download `url` to `dest` unless a file of the same size is already there.
/// The body streams into `<dest>.part`, renamed into place once complete.
async fn fetch_file(client: &reqwest::Client, url: &str, dest: &Path) -> Result<ClientFile> {
    let mut resp = client.get(url).send().await?.error_for_status()?;
    let expected = resp.content_length();
    let name = dest
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    if let (Some(len), Ok(meta)) = (expected, std::fs::metadata(dest))
        && meta.len() == len
    {
        drop(resp);
        println!("{name}: {len} bytes already present, skipped");
        let dest = dest.to_path_buf();
        let sha256 = tokio::task::spawn_blocking(move || sha256_file(&dest)).await??;
        return Ok(ClientFile { size: len, sha256 });
    }

    let part = PathBuf::from(format!("{}.part", dest.display()));
    let mut file = tokio::fs::File::create(&part)
        .await
        .with_context(|| format!("creating {}", part.display()))?;
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    while let Some(chunk) = resp.chunk().await? {
        hasher.update(&chunk);
        size += chunk.len() as u64;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    drop(file);
    if let Some(len) = expected
        && len != size
    {
        bail!("{name}: body was {size} bytes, Content-Length said {len}");
    }
    tokio::fs::rename(&part, dest).await?;
    println!("{name}: downloaded {size} bytes");
    Ok(ClientFile {
        size,
        sha256: hex(&hasher.finalize()),
    })
}

/// SHA-256 of a file on disk, lowercase hex.
///
/// # Errors
///
/// Fails when the file cannot be read.
pub fn sha256_file(path: &Path) -> Result<String> {
    let mut f = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

fn rfc3339_utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    rfc3339_utc(secs)
}

/// Format Unix seconds as `YYYY-MM-DDTHH:MM:SSZ` (Howard Hinnant's
/// civil-from-days algorithm, valid for every date after 1970).
#[must_use]
pub fn rfc3339_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem / 60) % 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_rfc3339() {
        assert_eq!(rfc3339_utc(0), "1970-01-01T00:00:00Z");
        // 2026-10-15 12:34:56 UTC
        assert_eq!(rfc3339_utc(1_792_067_696), "2026-10-15T12:34:56Z");
        // leap day
        assert_eq!(rfc3339_utc(951_782_400), "2000-02-29T00:00:00Z");
    }

    #[test]
    fn hexes() {
        assert_eq!(hex(&[0x00, 0xab, 0x0f]), "00ab0f");
    }
}
