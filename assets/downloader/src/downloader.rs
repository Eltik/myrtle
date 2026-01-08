use crate::utils::{back, clear, next, printc, printc_colors, scale};
use aes::Aes128;
use cbc::Decryptor;
use chrono;
use cipher::{BlockDecryptMut, KeyIvInit};
use crossterm::event::{self, Event, KeyCode};
use indicatif::{MultiProgress, ProgressBar};
use rand::rng;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use regex::Regex;
use reqwest::blocking::get;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, Cursor, Read};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, fs, path::PathBuf, time::Instant};
use unity_rs::generated::{AudioClip, Font, Mesh, TextAsset, Texture2D};
use unity_rs::{ClassIDType, Environment};
use zip::ZipArchive;

#[allow(dead_code)]
pub struct ArkAssets {
    pub server: Servers,
    pub asset_version: String,
    pub client_version: String,
    pub hot_update_list: HashMap<String, Value>,
    pub total_size: i64,
    pub ab_size: i64,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Servers {
    /// CN Official (Hypergryph)
    OFFICIAL = 0,
    /// CN Bilibili
    BILIBILI = 1,
    /// Global/EN (Yostar)
    EN = 2,
    /// Japan (Yostar)
    JP = 3,
    /// Korea (Yostar)
    KR = 4,
    /// Taiwan (Gryphline)
    TW = 5,
}

impl Servers {
    /// Returns the CDN base URL for asset downloads
    pub fn cdn_base_url(&self) -> &'static str {
        match self {
            Servers::OFFICIAL | Servers::BILIBILI => "https://ak.hycdn.cn/assetbundle",
            Servers::EN => "https://ark-us-static-online.yo-star.com/assetbundle",
            Servers::JP => "https://ark-jp-static-online.yo-star.com/assetbundle",
            // KR uses a CDN with a tenant ID suffix
            Servers::KR => "https://ark-kr-static-online-1300509597.yo-star.com/assetbundle",
            // TW uses Hypergryph's HG CDN
            Servers::TW => "https://ak-tw.hg-cdn.com/assetbundle",
        }
    }

    /// Returns the version API URL
    pub fn version_url(&self) -> String {
        match self {
            Servers::OFFICIAL => {
                "https://ak-conf.hypergryph.com/config/prod/official/Android/version".to_string()
            }
            Servers::BILIBILI => {
                "https://ak-conf.hypergryph.com/config/prod/b/Android/version".to_string()
            }
            Servers::EN => {
                "https://ark-us-static-online.yo-star.com/assetbundle/official/Android/version"
                    .to_string()
            }
            Servers::JP => {
                "https://ark-jp-static-online.yo-star.com/assetbundle/official/Android/version"
                    .to_string()
            }
            // KR uses a CDN with a tenant ID suffix
            Servers::KR => {
                "https://ark-kr-static-online-1300509597.yo-star.com/assetbundle/official/Android/version"
                    .to_string()
            }
            // TW uses Gryphline's config server for version
            Servers::TW => {
                "https://ak-conf-tw.gryphline.com/config/prod/official/Android/version".to_string()
            }
        }
    }

    /// Returns the server tag used in asset URLs
    pub fn asset_tag(&self) -> &'static str {
        match self {
            Servers::OFFICIAL => "official",
            Servers::BILIBILI => "bilibili",
            // All Yostar servers use "official" tag
            Servers::EN | Servers::JP | Servers::KR | Servers::TW => "official",
        }
    }

    /// Returns the cache file name suffix for version caching
    pub fn cache_name(&self) -> &'static str {
        match self {
            Servers::OFFICIAL => "official",
            Servers::BILIBILI => "bilibili",
            Servers::EN => "en",
            Servers::JP => "jp",
            Servers::KR => "kr",
            Servers::TW => "tw",
        }
    }

    /// Returns display name for the server
    pub fn display_name(&self) -> &'static str {
        match self {
            Servers::OFFICIAL => "CN Official",
            Servers::BILIBILI => "CN Bilibili",
            Servers::EN => "Global/EN",
            Servers::JP => "Japan",
            Servers::KR => "Korea",
            Servers::TW => "Taiwan",
        }
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct VersionResponse {
    resVersion: String,
    clientVersion: String,
}

#[derive(Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct VersionInfo {
    pub resVersion: String,
    pub clientVersion: String,
    pub lastChecked: String, // ISO 8601 timestamp
}

#[derive(Debug, PartialEq)]
pub enum UpdateStatus {
    NoUpdate,
    ResourceUpdate {
        old_version: String,
        new_version: String,
    },
    ClientUpdate {
        old_version: String,
        new_version: String,
    },
    BothUpdated {
        old_res_version: String,
        new_res_version: String,
        old_client_version: String,
        new_client_version: String,
    },
    FirstCheck, // No cached version exists
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct HotUpdateJSON {
    packInfos: Vec<PackInfo>,
    abInfos: Vec<AbInfo>,
}

#[derive(Deserialize)]
#[allow(non_snake_case, dead_code)]
struct PackInfo {
    name: String,
    totalSize: Option<i64>,
    // other fields ignored
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct AbInfo {
    name: String,
    totalSize: i64,
    abSize: i64,
    md5: String,
    pid: Option<String>,
}

pub struct HotFile {
    pub total_size: i64,
    pub ab_size: i64,
    pub md5: String,
}

pub struct HotGroup {
    pub total_size: i64,
    pub files: HashMap<String, HotFile>,
}

impl ArkAssets {
    pub const CHAT_MASK: [u8; 32] =
        hex_literal::hex!("554954704169383270484157776e7a7148524d4377506f6e4a4c49423357436c");

    pub fn new(server: Servers) -> Result<Self, Box<dyn std::error::Error>> {
        let (asset_version, client_version) = Self::get_version(server)?;

        // Python line 79: Print version info
        printc(
            &format!(
                "Game Versions: {} Material Versions: {}",
                client_version, asset_version
            ),
            &[1, 32],
        );

        let (hot_update_list, total_size, ab_size) =
            Self::get_hot_update_list(&server, &asset_version)?;

        // Python line 81: Print size info
        printc(
            &format!(
                "Total Resource Size: {} Unzipped Size: {}",
                scale(total_size, 1024.0, 2),
                scale(ab_size, 1024.0, 2)
            ),
            &[1, 32],
        );

        Ok(ArkAssets {
            server,
            asset_version,
            client_version,
            hot_update_list,
            total_size,
            ab_size,
        })
    }

    /// Decrypt text asset data using AES-128-CBC.
    /// Returns the original data if decryption fails (matching Python's behavior
    /// where the outer try/except catches and ignores decryption errors).
    pub fn text_asset_decrypt(stream: &[u8], has_rsa: bool) -> Vec<u8> {
        // Try to decrypt, return original data on any failure
        Self::try_text_asset_decrypt(stream, has_rsa).unwrap_or_else(|| stream.to_vec())
    }

    /// Internal decryption that can fail
    fn try_text_asset_decrypt(stream: &[u8], has_rsa: bool) -> Option<Vec<u8>> {
        let aes_key = &Self::CHAT_MASK[..16];

        // Skip RSA header if present
        let data = if has_rsa {
            if stream.len() <= 128 {
                return None; // Not enough data for RSA header
            }
            &stream[128..]
        } else {
            stream
        };

        // Need at least 16 bytes for IV + some data
        if data.len() < 32 {
            return None;
        }

        // Data after IV must be multiple of 16 for AES
        let payload_len = data.len() - 16;
        if payload_len % 16 != 0 {
            return None;
        }

        let buf = &data[..16];
        let mask = &Self::CHAT_MASK[16..];

        // Derive IV by XORing with mask
        let mut iv = [0u8; 16];
        for i in 0..16 {
            iv[i] = buf[i] ^ mask[i];
        }

        let decryptor = Decryptor::<Aes128>::new(aes_key.into(), iv.as_slice().into());

        let mut payload = data[16..].to_vec();

        // Decrypt - NoPadding shouldn't fail, but handle it anyway
        if decryptor
            .decrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut payload)
            .is_err()
        {
            return None;
        }

        // Manual PKCS7 unpadding (matching Python's: s[0:(len(s) - s[-1])])
        if let Some(&pad_byte) = payload.last() {
            let pad = pad_byte as usize;
            // Validate padding - must be 1-16 and not exceed data length
            if pad > 0 && pad <= 16 && pad <= payload.len() {
                payload.truncate(payload.len() - pad);
            }
            // If padding is invalid, return data as-is (Python does this too)
        }

        Some(payload)
    }

    pub fn get_version(server: Servers) -> Result<(String, String), Box<dyn std::error::Error>> {
        let url = server.version_url();
        let js: VersionResponse = get(&url)?.json()?;
        Ok((js.resVersion, js.clientVersion))
    }

    /// Get the path to the version cache file for a given server and savedir
    fn get_version_cache_path(server: Servers, savedir: &str) -> PathBuf {
        PathBuf::from(savedir).join(format!("version_cache_{}.json", server.cache_name()))
    }

    /// Load cached version info from disk
    pub fn load_version_cache(server: Servers, savedir: &str) -> Option<VersionInfo> {
        let cache_path = Self::get_version_cache_path(server, savedir);

        if !cache_path.exists() {
            return None;
        }

        match fs::read_to_string(&cache_path) {
            Ok(content) => serde_json::from_str(&content).ok(),
            Err(_) => None,
        }
    }

    /// Save current version info to cache
    pub fn save_version_cache(
        server: Servers,
        savedir: &str,
        res_version: &str,
        client_version: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cache_path = Self::get_version_cache_path(server, savedir);

        // Ensure directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let version_info = VersionInfo {
            resVersion: res_version.to_string(),
            clientVersion: client_version.to_string(),
            lastChecked: chrono::Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string_pretty(&version_info)?;
        fs::write(&cache_path, json)?;

        Ok(())
    }

    /// Check for version updates by comparing server version with cached version
    ///
    /// Returns UpdateStatus indicating what (if anything) has changed
    pub fn check_for_updates(
        server: Servers,
        savedir: &str,
    ) -> Result<UpdateStatus, Box<dyn std::error::Error>> {
        // Fetch current version from server
        let (new_res_version, new_client_version) = Self::get_version(server)?;

        // Load cached version (if exists)
        let cached = Self::load_version_cache(server, savedir);

        match cached {
            None => {
                // No cached version - this is the first check
                // Save current version for future comparisons
                Self::save_version_cache(server, savedir, &new_res_version, &new_client_version)?;
                Ok(UpdateStatus::FirstCheck)
            }
            Some(old_version) => {
                let res_changed = old_version.resVersion != new_res_version;
                let client_changed = old_version.clientVersion != new_client_version;

                let status = match (res_changed, client_changed) {
                    (false, false) => UpdateStatus::NoUpdate,
                    (true, false) => UpdateStatus::ResourceUpdate {
                        old_version: old_version.resVersion.clone(),
                        new_version: new_res_version.clone(),
                    },
                    (false, true) => UpdateStatus::ClientUpdate {
                        old_version: old_version.clientVersion.clone(),
                        new_version: new_client_version.clone(),
                    },
                    (true, true) => UpdateStatus::BothUpdated {
                        old_res_version: old_version.resVersion.clone(),
                        new_res_version: new_res_version.clone(),
                        old_client_version: old_version.clientVersion.clone(),
                        new_client_version: new_client_version.clone(),
                    },
                };

                // Update cache if anything changed
                if res_changed || client_changed {
                    Self::save_version_cache(
                        server,
                        savedir,
                        &new_res_version,
                        &new_client_version,
                    )?;
                }

                Ok(status)
            }
        }
    }

    pub fn get_hot_update_list(
        server: &Servers,
        asset_version: &str,
    ) -> Result<(HashMap<String, Value>, i64, i64), Box<dyn std::error::Error>> {
        let url = format!(
            "{}/{}/Android/assets/{}/hot_update_list.json",
            server.cdn_base_url(),
            server.asset_tag(),
            asset_version
        );

        let js: HotUpdateJSON = get(&url)?.json()?;

        let mut out: HashMap<String, Value> = HashMap::new();
        out.insert(
            "other".to_string(),
            json!({
                "totalSize": 0,
                "files": {}
            }),
        );

        let mut total_size: i64 = 0;
        let mut ab_size: i64 = 0;

        // ========= packInfos =========
        for item in js.packInfos {
            let key = item.name.replace("_", "/");

            out.insert(
                key.clone(),
                json!({
                    "totalSize": 0,
                    "files": {}
                }),
            );
        }

        // ========= abInfos =========
        for item in js.abInfos {
            total_size += item.totalSize;
            ab_size += item.abSize;

            if let Some(pid) = &item.pid {
                let pid_key = pid.replace("_", "/");

                if out.contains_key(&pid_key) {
                    // out[pid]["totalSize"] += size
                    let entry = out.get_mut(&pid_key).unwrap();
                    let obj = entry.as_object_mut().unwrap();

                    let ts = obj.get("totalSize").unwrap().as_i64().unwrap();
                    obj.insert("totalSize".to_string(), json!(ts + item.totalSize));

                    // Add file info
                    let files = obj.get_mut("files").unwrap().as_object_mut().unwrap();
                    files.insert(
                        item.name.clone(),
                        json!({
                            "totalSize": item.totalSize,
                            "abSize": item.abSize,
                            "md5": item.md5
                        }),
                    );
                } else {
                    let other = out.get_mut("other").unwrap();
                    let obj = other.as_object_mut().unwrap();
                    let ts = obj.get_mut("totalSize").unwrap().as_i64().unwrap();
                    *obj.get_mut("totalSize").unwrap() = json!(ts + item.totalSize);
                    let files = obj.get_mut("files").unwrap().as_object_mut().unwrap();
                    files.insert(
                        item.name.clone(),
                        json!({
                            "totalSize": item.totalSize,
                            "abSize": item.abSize,
                            "md5": item.md5
                        }),
                    );
                }
            } else {
                let other = out.get_mut("other").unwrap();
                let obj = other.as_object_mut().unwrap();
                let ts = obj.get_mut("totalSize").unwrap().as_i64().unwrap();
                *obj.get_mut("totalSize").unwrap() = json!(ts + item.totalSize);
                let files = obj.get_mut("files").unwrap().as_object_mut().unwrap();
                files.insert(
                    item.name.clone(),
                    json!({
                        "totalSize": item.totalSize,
                        "abSize": item.abSize,
                        "md5": item.md5
                    }),
                );
            }
        }

        Ok((out, total_size, ab_size))
    }

    pub fn download(&self, savedir: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut options: Vec<(String, String, Vec<i32>)> = Vec::new();

        for (item, value) in &self.hot_update_list {
            let size = value["totalSize"].as_i64().unwrap();
            let per = size as f64 / self.total_size as f64;

            let terminal_width = crossterm::terminal::size().unwrap().0 as usize;
            let bar_count = ((per * (terminal_width - 46) as f64) as usize).max(0);
            let bar = "█".repeat(bar_count);

            let formatted = format!(
                "{:<35}{:<7}{}",
                format!("{:<15} 包大小: {}", item, scale(size, 1024.0, 2)),
                format!("{:.2}%", per * 100.0),
                bar
            );

            options.push((item.clone(), formatted.clone(), vec![1, 34]));
            printc(&formatted, &[1, 34]);
        }

        printc(
            "Download Options [Download All=A, Select Download=C, Cancel=Other]: ",
            &[1, 36],
        );

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let inp = input.trim();

        back(1);

        if inp == "A" || inp == "a" {
            back(1);
            clear();
            printc("Start Downloading All", &[1, 36]);

            let all_keys: Vec<String> = self.hot_update_list.keys().cloned().collect();
            self.download_fromlist(&all_keys, savedir, 6)?;
            Ok(())
        } else if inp == "C" || inp == "c" {
            let mut chosen: Vec<String> = Vec::new();
            let mut pos: usize = 0;

            // Python line 147: Initial status display
            back(1);
            clear();

            let total_files: usize = 0;
            let total_size: i64 = 0;
            printc(
                &format!(
                    "Total Number: {} Estimated Size: {} Selected: {:?}",
                    total_files,
                    scale(total_size, 1024.0, 2),
                    chosen
                ),
                &[1, 36],
            );
            printc(
                "[Up/Down Arrow=Select, Left Arrow=Remove, Right Arrow=Add, ESC Key=Confirm]",
                &[1, 33],
            );

            loop {
                if let Event::Key(key_event) = event::read().unwrap() {
                    match key_event.code {
                        KeyCode::Up => {
                            if pos > 0 {
                                clear();
                                printc(&options[pos].1, &options[pos].2);
                                back(1);
                                pos -= 1;
                                let mut highlight_color = options[pos].2.clone();
                                highlight_color.push(100);
                                printc(&options[pos].1, &highlight_color);
                            }
                        }
                        KeyCode::Down => {
                            if pos + 1 < options.len() {
                                clear();
                                printc(&options[pos].1, &options[pos].2);
                                next(1);
                                pos += 1;
                                let mut highlight_color = options[pos].2.clone();
                                highlight_color.push(100);
                                printc(&options[pos].1, &highlight_color);
                            }
                        }
                        KeyCode::Right => {
                            if !chosen.contains(&options[pos].0) {
                                chosen.push(options[pos].0.clone());

                                // Update color to yellow [1, 33] and display
                                clear();
                                printc(&options[pos].1, &[1, 33, 100]);
                                options[pos].2 = vec![1, 33];

                                // Python line 174: Update status display
                                next(options.len() - pos);
                                clear();

                                let total_files: usize = chosen
                                    .iter()
                                    .map(|key| {
                                        self.hot_update_list[key]["files"]
                                            .as_object()
                                            .map(|obj| obj.len())
                                            .unwrap_or(0)
                                    })
                                    .sum();

                                let total_size: i64 = chosen
                                    .iter()
                                    .map(|key| {
                                        self.hot_update_list[key]["totalSize"].as_i64().unwrap_or(0)
                                    })
                                    .sum();

                                printc(
                                    &format!(
                                        "Total Number: {} Estimated Size: {} Selected: {:?}",
                                        total_files,
                                        scale(total_size, 1024.0, 2),
                                        chosen
                                    ),
                                    &[1, 36],
                                );
                                back(options.len() - pos);
                            }
                        }
                        KeyCode::Left => {
                            if let Some(index) = chosen.iter().position(|x| x == &options[pos].0) {
                                chosen.remove(index);

                                // Update color back to blue [1, 34] and display
                                clear();
                                printc(&options[pos].1, &[1, 34, 100]);
                                options[pos].2 = vec![1, 34];

                                // Python line 185: Update status display
                                next(options.len() - pos);
                                clear();

                                let total_files: usize = chosen
                                    .iter()
                                    .map(|key| {
                                        self.hot_update_list[key]["files"]
                                            .as_object()
                                            .map(|obj| obj.len())
                                            .unwrap_or(0)
                                    })
                                    .sum();

                                let total_size: i64 = chosen
                                    .iter()
                                    .map(|key| {
                                        self.hot_update_list[key]["totalSize"].as_i64().unwrap_or(0)
                                    })
                                    .sum();

                                printc(
                                    &format!(
                                        "Total Number: {} Estimated Size: {} Selected: {:?}",
                                        total_files,
                                        scale(total_size, 1024.0, 2),
                                        chosen
                                    ),
                                    &[1, 36],
                                );
                                back(options.len() - pos);
                            }
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }

            printc("Start Download", &[1, 36]);
            self.download_fromlist(&chosen, savedir, 6)?;
            Ok(())
        } else {
            printc("Download Canceled", &[1, 33]);
            Ok(())
        }
    }

    pub fn download_fromlist(
        &self,
        keys: &[String],
        savedir: &str,
        _threading_count: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let count: usize = keys
            .iter()
            .map(|key| {
                self.hot_update_list[key]["files"]
                    .as_object()
                    .unwrap()
                    .len()
            })
            .sum();

        let lock = Arc::new(Mutex::new(()));
        let _read_lock = Arc::new(Mutex::new(()));
        let _size_lock = Arc::new(Mutex::new(()));
        let _write_lock = Arc::new(Mutex::new(()));

        let per_path = PathBuf::from(savedir).join("persistent_res_list.json");
        let per: HashMap<String, String> = if !per_path.exists() {
            fs::create_dir_all(savedir)?;
            fs::write(&per_path, "{}")?;
            HashMap::new()
        } else {
            match fs::read_to_string(&per_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| {
                    fs::write(&per_path, "{}").ok();
                    HashMap::new()
                }),
                Err(_) => {
                    fs::write(&per_path, "{}").ok();
                    HashMap::new()
                }
            }
        };

        let mut files = HashMap::new();
        let mut count = count; // make mutable

        for key in keys {
            let file_list = self.hot_update_list[key]["files"].as_object().unwrap();

            for (filename, file_info) in file_list {
                let file_md5 = file_info["md5"].as_str().unwrap();

                if let Some(existing_md5) = per.get(filename) {
                    if existing_md5 == file_md5 {
                        printc_colors(
                            &[
                                &format!("{:<80}", format!("[{}]", filename)),
                                "Already Exists Latest Version",
                            ],
                            &[vec![36], vec![1, 32]],
                        );
                        count -= 1;
                        continue;
                    } else {
                        printc(&format!("[{}] Will be Updated", filename), &[1, 33]);

                        // Python lines 237-240: Delete old file and unpack directory
                        let file_path = PathBuf::from(savedir).join(filename);

                        // Delete the file if it exists
                        if file_path.is_file() {
                            let _ = fs::remove_file(&file_path);
                        }

                        // Delete the [unpack] directory if it exists
                        if let Some(stem) = file_path.file_stem() {
                            let unpack_dir = file_path
                                .with_file_name(format!("[unpack]{}", stem.to_str().unwrap()));
                            if unpack_dir.is_dir() {
                                let _ = fs::remove_dir_all(&unpack_dir);
                            }
                        }

                        files.insert(filename.clone(), file_info.clone());
                    }
                } else {
                    files.insert(filename.clone(), file_info.clone());
                }
            }
        }

        let files = Arc::new(Mutex::new(files));

        let multi_progress = MultiProgress::new();
        let pbar = multi_progress.add(ProgressBar::new(count as u64));
        let zipbar = multi_progress.add(ProgressBar::new(count as u64));
        let unpackbar = multi_progress.add(ProgressBar::new(count as u64));

        pbar.set_message("Total Progress 0B");
        zipbar.set_message("Unzip Progress");
        unpackbar.set_message("Unpack Progress");

        let res_size = Arc::new(Mutex::new(0u64));
        let ts = Instant::now();

        let mut files_vec: Vec<(String, Value)> = {
            let files_guard = files.lock().unwrap();
            files_guard
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        };

        files_vec.shuffle(&mut rng());

        // Extract data from self that we need in the parallel closure
        let server = self.server;
        let asset_version = self.asset_version.clone();

        files_vec.par_iter().for_each(|(filename, file_info)| {
            let md5 = file_info["md5"].as_str().unwrap();

            // Line 260: Download the asset
            // We need to inline the download logic here since we can't call self.download_asset()
            let download_result = (|| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
                // Build URL with regex substitution
                let re = Regex::new(r"\.[^.]*$")?;
                let dat_path = re.replace(filename, ".dat");
                let url_path = dat_path.replace("/", "_").replace("#", "__");

                let url = format!(
                    "{}/{}/Android/assets/{}/{}",
                    server.cdn_base_url(),
                    server.asset_tag(),
                    asset_version,
                    url_path
                );

                // Python line 393: Create progress bar for this download
                let download_bar = multi_progress.add(ProgressBar::new(0));
                download_bar.set_style(
                    indicatif::ProgressStyle::default_bar()
                        .template("[{msg}] {bar:40.yellow/yellow} {bytes}/{total_bytes} {bytes_per_sec}")
                        .unwrap()
                );
                download_bar.set_message(format!("[{}]: Reading headers", filename));

                // Create client and download
                let client = reqwest::blocking::Client::new();
                let mut response = client
                    .get(&url)
                    .header("User-Agent", "BestHTTP")
                    .send()?;

                let length = response.content_length().ok_or("No content length")? as usize;

                // Python line 401: Update bar with total length and downloading message
                download_bar.set_length(length as u64);
                download_bar.set_message(format!("[{}]: Downloading", filename));

                // Calculate chunk size
                let mut chunk_size = length / 24;
                if chunk_size == 0 {
                    chunk_size = 1;
                } else if chunk_size >= 10485760 {
                    chunk_size = 10485760;
                }

                // Download in chunks with progress updates (Python lines 411-419)
                let mut res = Vec::with_capacity(length);
                let st = Instant::now();
                let mut buffer = vec![0u8; chunk_size];

                loop {
                    let bytes_read = response.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    res.extend_from_slice(&buffer[..bytes_read]);

                    // Python line 416: Update progress bar
                    download_bar.set_position(res.len() as u64);
                }

                // Finish and remove the download progress bar
                download_bar.finish_and_clear();

                let elapsed = st.elapsed().as_secs_f64();

                // Print completion message
                {
                    let _guard = lock.lock().unwrap();
                    printc_colors(
                        &[
                            &format!("{:<80}", format!("[{}]", filename)),
                            &format!(
                                "Downloaded   Time: {:.3}s  Average Speed:{}/s",
                                elapsed,
                                scale((length as f64 / elapsed) as i64, 1024.0, 2)
                            ),
                        ],
                        &[vec![36], vec![1, 32]],
                    );

                    let per = (length as f64 / 10485760.0).min(1.0);
                    let terminal_width = crossterm::terminal::size().unwrap_or((80, 24)).0 as usize;
                    let bar_width = ((per * (terminal_width.saturating_sub(33)) as f64) as usize)
                        .min(terminal_width.saturating_sub(33));
                    printc(
                        &format!(
                            "{:<16} {}",
                            format!("dat size: {}", scale(length as i64, 1024.0, 2)),
                            "█".repeat(bar_width)
                        ),
                        &[1, 34],
                    );
                }

                Ok(res)
            })();

            match download_result {
                Ok(stream) => {
                    // Line 261-266: Update res_size
                    {
                        let mut size_guard = res_size.lock().unwrap();
                        *size_guard += stream.len() as u64;

                        let elapsed = ts.elapsed().as_secs_f64();
                        let avg_speed = (*size_guard as f64) / elapsed;

                        let _lock_guard = lock.lock().unwrap();
                        let msg = format!(
                            "Total Progress {} Average Speed {}/s",
                            scale(*size_guard as i64, 1024.0, 2),
                            scale(avg_speed as i64, 1024.0, 2)
                        );
                        pbar.set_message(msg);
                    }

                    // Line 268-365: Unzip and unpack inline (no separate threads - controlled by rayon)
                    // This ensures parallelism is bounded by the rayon thread pool
                    let unzip_unpack_result = (|| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                        // === UNZIP ===
                        // Create cursor and open zip archive
                        let cursor = Cursor::new(&stream);
                        let mut archive = ZipArchive::new(cursor)?;

                        // Extract all files
                        for i in 0..archive.len() {
                            let mut file = archive.by_index(i)?;
                            let outpath = PathBuf::from(savedir).join(file.name());

                            if file.is_dir() {
                                fs::create_dir_all(&outpath)?;
                            } else {
                                if let Some(parent) = outpath.parent() {
                                    fs::create_dir_all(parent)?;
                                }
                                let mut outfile = fs::File::create(&outpath)?;
                                std::io::copy(&mut file, &mut outfile)?;
                            }
                        }

                        // Update zipbar
                        zipbar.inc(1);

                        // === UNPACK ===
                        let file_path = PathBuf::from(savedir).join(filename);

                        // Load Unity bundle
                        let mut env = Environment::new();
                        if env.load_file(
                            unity_rs::helpers::import_helper::FileSource::Path(
                                file_path.to_str().unwrap().to_string()
                            ),
                            None,
                            false,
                        ).is_none() {
                            // Not a Unity bundle, skip unpacking
                            unpackbar.inc(1);
                            return Ok(());
                        }

                        // Create unpack directory
                        let unpack_dir = file_path.with_file_name(
                            format!("[unpack]{}", file_path.file_stem().unwrap().to_str().unwrap())
                        );
                        fs::create_dir_all(&unpack_dir)?;

                        // Extract images (Texture2D and Sprite)
                        let mut images: HashMap<String, PathBuf> = HashMap::new();
                        let temp_dir = unpack_dir.join("temp");
                        fs::create_dir_all(&temp_dir)?;

                        // First pass: save all textures to temp directory
                        for obj in env.objects() {
                            if obj.obj_type == ClassIDType::Texture2D {
                                let mut obj_mut = obj.clone();
                                if let Ok(data) = obj_mut.read(false) {
                                    if let Ok(mut texture) = serde_json::from_value::<Texture2D>(data) {
                                        texture.object_reader = Some(Box::new(obj.clone()));
                                        let name_val = texture.m_Name.as_deref().unwrap_or("unnamed");
                                        let key = format!("(Texture2D){}", name_val);
                                        let temp_path = temp_dir.join(format!("{}.png", key));

                                        if unity_rs::export::texture_2d_converter::save_texture_as_png(
                                            &texture,
                                            &temp_path,
                                            true
                                        ).is_ok() {
                                            images.insert(key, temp_path);
                                        }
                                    }
                                }
                            } else if obj.obj_type == ClassIDType::Sprite {
                                let mut obj_mut = obj.clone();
                                if let Ok(data) = obj_mut.read(false) {
                                    if let Ok(mut sprite) = serde_json::from_value::<unity_rs::classes::generated::Sprite>(data) {
                                        sprite.object_reader = Some(Box::new(obj.clone()));
                                        let name_val = sprite.m_Name.as_deref().unwrap_or("unnamed");
                                        let key = format!("(Sprite){}", name_val);
                                        let temp_path = temp_dir.join(format!("{}.png", key));

                                        if let Some(ref obj_reader) = sprite.object_reader {
                                            if let Some(concrete_reader) = obj_reader
                                                .as_any()
                                                .downcast_ref::<unity_rs::files::ObjectReader<()>>() {
                                                if let Some(ref weak_assets) = concrete_reader.assets_file {
                                                    if let Some(rc_assets) = weak_assets.upgrade() {
                                                        if unity_rs::export::sprite_helper::save_sprite(
                                                            &sprite,
                                                            &rc_assets,
                                                            temp_path.to_str().unwrap()
                                                        ).is_ok() {
                                                            images.insert(key, temp_path);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Second pass: merge alpha channels and save final images
                        let image_keys: Vec<String> = images.keys().cloned().collect();
                        for name in &image_keys {
                            if !name.ends_with("[alpha]") {
                                let alpha_key = format!("{}[alpha]", name);

                                if images.contains_key(&alpha_key) {
                                    if let (Ok(rgb_img), Ok(alpha_img)) = (
                                        image::open(&images[name]).map(|i| i.to_rgba8()),
                                        image::open(&images[&alpha_key]).map(|i| i.to_luma8())
                                    ) {
                                        let (width, height) = rgb_img.dimensions();
                                        let mut merged_img = image::ImageBuffer::new(width, height);

                                        for y in 0..height {
                                            for x in 0..width {
                                                let rgb_pixel = rgb_img.get_pixel(x, y);
                                                let (a_width, a_height) = alpha_img.dimensions();

                                                let alpha_pixel = if a_width == width && a_height == height {
                                                    alpha_img.get_pixel(x, y)[0]
                                                } else {
                                                    255
                                                };

                                                merged_img.put_pixel(x, y, image::Rgba([
                                                    rgb_pixel[0],
                                                    rgb_pixel[1],
                                                    rgb_pixel[2],
                                                    alpha_pixel
                                                ]));
                                            }
                                        }

                                        let final_path = unpack_dir.join(format!("{}.png", name));
                                        let _ = image::DynamicImage::ImageRgba8(merged_img).save(&final_path);
                                    }
                                } else {
                                    let final_path = unpack_dir.join(format!("{}.png", name));
                                    let _ = fs::copy(&images[name], &final_path);
                                }
                            }
                        }

                        // Clean up temp directory
                        let _ = fs::remove_dir_all(&temp_dir);

                        // Process other asset types
                        for obj in env.objects() {
                            let mut obj_mut = obj.clone();

                            match obj.obj_type {
                                ClassIDType::TextAsset => {
                                    if let Ok(data) = obj_mut.read(false) {
                                        if let Ok(text_asset) = serde_json::from_value::<TextAsset>(data) {
                                            let script_str = text_asset.m_Script.as_deref().unwrap_or("");
                                            let script_bytes = script_str.as_bytes();

                                            let decrypted = {
                                                let fpath_str = file_path.to_str().unwrap();
                                                if (fpath_str.contains("gamedata\\levels") || fpath_str.contains("gamedata/levels"))
                                                    && !fpath_str.contains("enemydata") {
                                                    if script_bytes.len() > 128 {
                                                        script_bytes[128..].to_vec()
                                                    } else {
                                                        script_bytes.to_vec()
                                                    }
                                                } else {
                                                    ArkAssets::text_asset_decrypt(script_bytes, !fpath_str.contains("gamedata/levels"))
                                                }
                                            };

                                            let name_val = text_asset.m_Name.as_deref().unwrap_or("unnamed");

                                            if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&decrypted) {
                                                if let Ok(json_str) = serde_json::to_string_pretty(&json_val) {
                                                    let save_path = unpack_dir.join(format!("{}.json", name_val));
                                                    let _ = fs::write(&save_path, json_str);
                                                }
                                            } else {
                                                // Try to parse as BSON documents
                                                let mut cursor = Cursor::new(&decrypted);
                                                let mut docs: Vec<bson::Document> = Vec::new();
                                                while let Ok(doc) = bson::Document::from_reader(&mut cursor) {
                                                    docs.push(doc);
                                                }
                                                if !docs.is_empty() {
                                                    for (i, doc) in docs.iter().enumerate() {
                                                        if let Ok(json_val) = serde_json::to_value(doc) {
                                                            if let Ok(json_str) = serde_json::to_string_pretty(&json_val) {
                                                                let suffix = if i >= 1 { format!("_{}", i) } else { String::new() };
                                                                let save_path = unpack_dir.join(format!("{}{}.json", name_val, suffix));
                                                                let _ = fs::write(&save_path, json_str);
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    let re_ext = Regex::new(r"\.(lua|atlas|skel)$").unwrap();
                                                    let extension = if re_ext.is_match(name_val) { "" } else { ".txt" };
                                                    let save_path = unpack_dir.join(format!("{}{}", name_val, extension));
                                                    let _ = fs::write(&save_path, decrypted);
                                                }
                                            }
                                        }
                                    }
                                }

                                ClassIDType::AudioClip => {
                                    if let Ok(data) = obj_mut.read(false) {
                                        if let Ok(mut audio) = serde_json::from_value::<AudioClip>(data.clone()) {
                                            audio.object_reader = Some(Box::new(obj.clone()));
                                            if let Ok(samples) = unity_rs::export::audio_clip_converter::extract_audioclip_samples(&mut audio, true) {
                                                for (aname, adata) in samples {
                                                    let save_path = unpack_dir.join(&aname);
                                                    let _ = fs::write(&save_path, adata);
                                                }
                                            }
                                        }
                                    }
                                }

                                ClassIDType::Mesh => {
                                    if let Ok(data) = obj_mut.read(false) {
                                        if let Ok(mesh) = serde_json::from_value::<Mesh>(data) {
                                            let name_val = mesh.m_Name.as_deref().unwrap_or("unnamed");
                                            let save_path = unpack_dir.join(format!("{}.obj", name_val));
                                            if let Ok(obj_str) = unity_rs::export::mesh_exporter::export_mesh_obj(&mesh, None) {
                                                let _ = fs::write(&save_path, obj_str);
                                            }
                                        }
                                    }
                                }

                                ClassIDType::Font => {
                                    if let Ok(data) = obj_mut.read(false) {
                                        if let Ok(font) = serde_json::from_value::<Font>(data) {
                                            if let Some(ref font_data) = font.m_FontData {
                                                let font_bytes: Vec<u8> = font_data.iter().map(|&b| b as u8).collect();
                                                let extension = if font_bytes.len() >= 4 && &font_bytes[0..4] == b"OTTO" {
                                                    ".otf"
                                                } else {
                                                    ".ttf"
                                                };
                                                let name_val = font.m_Name.as_deref().unwrap_or("unnamed");
                                                let save_path = unpack_dir.join(format!("{}{}", name_val, extension));
                                                let _ = fs::write(&save_path, font_bytes);
                                            }
                                        }
                                    }
                                }

                                _ => {}
                            }
                        }

                        // Update unpackbar
                        unpackbar.inc(1);

                        // Update persistent_res_list.json
                        {
                            let _write_guard = _write_lock.lock().unwrap();

                            let mut per: HashMap<String, String> = match fs::read_to_string(&per_path) {
                                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                                Err(_) => HashMap::new(),
                            };

                            per.insert(filename.clone(), md5.to_string());

                            if let Ok(json_str) = serde_json::to_string(&per) {
                                let _ = fs::write(&per_path, json_str);
                            }
                        }

                        Ok(())
                    })();

                    // Silent error handling (matches Python's pass)
                    if let Err(e) = unzip_unpack_result {
                        let _lock_guard = lock.lock().unwrap();
                        printc(&format!("Error processing {}: {}", filename, e), &[31]);
                    }

                    // Update main progress bar
                    pbar.inc(1);
                }
                Err(e) => {
                    let _lock_guard = lock.lock().unwrap();
                    printc(&format!("Error: {}", e), &[31]);
                    pbar.inc(1);
                }
            }
        });

        // All work is done inline now - no thread joining needed

        // Close progress bars (Python lines 380-382)
        pbar.finish();
        zipbar.finish();
        unpackbar.finish();

        Ok(())
    }

    pub fn download_asset(
        &self,
        path: &str,
        lock: Option<&Arc<Mutex<()>>>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Line 386-389: Build URL with regex substitution
        let re = Regex::new(r"\.[^.]*$")?;
        let dat_path = re.replace(path, ".dat");
        let url_path = dat_path.replace("/", "_").replace("#", "__");

        let url = format!(
            "{}/{}/Android/assets/{}/{}",
            self.server.cdn_base_url(),
            self.server.asset_tag(),
            self.asset_version,
            url_path
        );

        // Line 390: Set headers and create client
        let client = reqwest::blocking::Client::new();

        // Line 396: Make streaming GET request
        let mut response = client.get(&url).header("User-Agent", "BestHTTP").send()?;

        // Line 397: Get content length
        let length = response.content_length().ok_or("No content length")? as usize;

        // Lines 404-410: Calculate chunk size
        let mut chunk_size = length / 24;
        if chunk_size == 0 {
            chunk_size = 1;
        } else if chunk_size >= 10485760 {
            chunk_size = 10485760;
        }

        // Lines 404, 411-419: Download in chunks
        let mut res = Vec::with_capacity(length);
        let st = Instant::now();

        let mut buffer = vec![0u8; chunk_size];

        loop {
            let bytes_read = response.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            res.extend_from_slice(&buffer[..bytes_read]);
        }

        let elapsed = st.elapsed().as_secs_f64();

        // Lines 423-429: Print completion message
        if let Some(lock) = lock {
            let _guard = lock.lock().unwrap();
            printc_colors(
                &[
                    &format!("{:<80}", format!("[{}]", path)),
                    &format!(
                        "Downloaded   Time: {:.3}s  Average Speed:{}/s",
                        elapsed,
                        scale((length as f64 / elapsed) as i64, 1024.0, 2)
                    ),
                ],
                &[vec![36], vec![1, 32]],
            );

            let per = (length as f64 / 10485760.0).min(1.0);
            let terminal_width = crossterm::terminal::size().unwrap_or((80, 24)).0 as usize;
            let bar_width = ((per * (terminal_width.saturating_sub(33)) as f64) as usize)
                .min(terminal_width.saturating_sub(33));
            printc(
                &format!(
                    "{:<16} {}",
                    format!("dat size: {}", scale(length as i64, 1024.0, 2)),
                    "█".repeat(bar_width)
                ),
                &[1, 34],
            );
        }

        Ok(res)
    }
}
