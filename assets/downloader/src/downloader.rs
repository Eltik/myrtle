use crate::utils::{back, clear, next, printc, printc_colors, scale};
use aes::Aes128;
use cbc::Decryptor;
use chrono;
use cipher::{BlockDecryptMut, KeyIvInit};
use crossterm::event::{self, Event, KeyCode};
use indicatif::{MultiProgress, ProgressBar};
use rand::seq::SliceRandom;
use rand::thread_rng;
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
    OFFICIAL = 0,
    BILIBILI = 1,
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
    ResourceUpdate { old_version: String, new_version: String },
    ClientUpdate { old_version: String, new_version: String },
    BothUpdated {
        old_res_version: String,
        new_res_version: String,
        old_client_version: String,
        new_client_version: String
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
            &vec![1, 32],
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
            &vec![1, 32],
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

    pub fn text_asset_decrypt(stream: &[u8], has_rsa: bool) -> Vec<u8> {
        let aes_key = &Self::CHAT_MASK[..16];
        let data = if has_rsa { &stream[128..] } else { stream };

        let buf = &data[..16];
        let mask = &Self::CHAT_MASK[16..];

        let mut iv = [0u8; 16];
        for i in 0..16 {
            iv[i] = buf[i] ^ mask[i];
        }

        let decryptor = Decryptor::<Aes128>::new(aes_key.into(), iv.as_slice().into());

        let mut payload = data[16..].to_vec();
        decryptor
            .decrypt_padded_mut::<cipher::block_padding::NoPadding>(&mut payload)
            .expect("AES CBC decrypt failed");

        let pad = *payload.last().unwrap() as usize;
        let end = payload.len() - pad;
        payload.truncate(end);

        payload
    }

    pub fn get_version(server: Servers) -> Result<(String, String), Box<dyn std::error::Error>> {
        let server_tag = match server {
            Servers::OFFICIAL => "official",
            Servers::BILIBILI => "b",
        };

        let url = format!(
            "https://ak-conf.hypergryph.com/config/prod/{}/Android/version",
            server_tag
        );

        let js: VersionResponse = get(&url)?.json()?;
        Ok((js.resVersion, js.clientVersion))
    }

    /// Get the path to the version cache file for a given server and savedir
    fn get_version_cache_path(server: Servers, savedir: &str) -> PathBuf {
        let server_name = match server {
            Servers::OFFICIAL => "official",
            Servers::BILIBILI => "bilibili",
        };
        PathBuf::from(savedir).join(format!("version_cache_{}.json", server_name))
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
                    Self::save_version_cache(server, savedir, &new_res_version, &new_client_version)?;
                }

                Ok(status)
            }
        }
    }

    pub fn get_hot_update_list(
        server: &Servers,
        asset_version: &str,
    ) -> Result<(HashMap<String, Value>, i64, i64), Box<dyn std::error::Error>> {
        let server_tag = if *server == Servers::OFFICIAL {
            "official"
        } else {
            "bilibili"
        };

        let url = format!(
            "https://ak.hycdn.cn/assetbundle/{}/Android/assets/{}/hot_update_list.json",
            server_tag, asset_version
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
            printc(&formatted, &vec![1, 34]);
        }

        printc(
            "Download Options [Download All=A, Select Download=C, Cancel=Other]: ",
            &vec![1, 36],
        );

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let inp = input.trim();

        back(1);

        if inp == "A" || inp == "a" {
            back(1);
            clear();
            printc("Start Downloading All", &vec![1, 36]);

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
                &vec![1, 36],
            );
            printc(
                "[Up/Down Arrow=Select, Left Arrow=Remove, Right Arrow=Add, ESC Key=Confirm]",
                &vec![1, 33],
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
                                printc(&options[pos].1, &vec![1, 33, 100]);
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
                                    &vec![1, 36],
                                );
                                back(options.len() - pos);
                            }
                        }
                        KeyCode::Left => {
                            if let Some(index) = chosen.iter().position(|x| x == &options[pos].0) {
                                chosen.remove(index);

                                // Update color back to blue [1, 34] and display
                                clear();
                                printc(&options[pos].1, &vec![1, 34, 100]);
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
                                    &vec![1, 36],
                                );
                                back(options.len() - pos);
                            }
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }

            printc("Start Download", &vec![1, 36]);
            self.download_fromlist(&chosen, savedir, 6)?;
            Ok(())
        } else {
            printc("Download Canceled", &vec![1, 33]);
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
        let thread_handles = Arc::new(Mutex::new(Vec::new()));

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
        let _unpack_count = Arc::new(Mutex::new(0usize));
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
                        printc(&format!("[{}] Will be Updated", filename), &vec![1, 33]);

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

        files_vec.shuffle(&mut thread_rng());

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

                let server_tag = if server == Servers::OFFICIAL {
                    "official"
                } else {
                    "bilibili"
                };

                let url = format!(
                    "https://ak.hycdn.cn/assetbundle/{}/Android/assets/{}/{}",
                    server_tag, asset_version, url_path
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
                        &vec![1, 34],
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

                    // Line 268-365: Spawn unzip and unpack threads
                    {
                        // Clone all the data we need for the thread
                        let stream_clone = stream.clone();
                        let filename_clone = filename.to_string();
                        let md5_clone = md5.to_string();
                        let savedir_clone = savedir.to_string();
                        let lock_clone = Arc::clone(&lock);
                        let write_lock_clone = Arc::clone(&_write_lock); // Note: use _write_lock
                        let zipbar_clone = zipbar.clone();
                        let per_path_clone = per_path.clone();
                        let unpackbar_clone = unpackbar.clone();
                        let unpack_count_clone = Arc::clone(&_unpack_count);
                        let handles_clone = Arc::clone(&thread_handles);

                        // Spawn unzip thread and collect handle
                        let handle = std::thread::spawn(move || {
                            // Unzip logic
                            let unzip_result = (|| -> Result<(), Box<dyn std::error::Error>> {
                                // Create cursor and open zip archive
                                let cursor = Cursor::new(&stream_clone);
                                let mut archive = ZipArchive::new(cursor)?;

                                // Extract all files
                                for i in 0..archive.len() {
                                    let mut file = archive.by_index(i)?;
                                    let outpath = PathBuf::from(&savedir_clone).join(file.name());

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

                                // Spawn unpack thread for each extracted file (Python line 364)
                                let savedir_unpack = savedir_clone.clone();
                                let filename_unpack = filename_clone.clone();
                                let lock_unpack = Arc::clone(&lock_clone);
                                let unpackbar_unpack = unpackbar_clone.clone();
                                let unpack_count_unpack = Arc::clone(&unpack_count_clone);

                                std::thread::spawn(move || {
                                    let file_path = PathBuf::from(&savedir_unpack).join(&filename_unpack);

                                    // Unpack logic (Python lines 286-362)
                                    let unpack_result = (|| -> Result<(), Box<dyn std::error::Error>> {
                                        // Line 289: Load Unity bundle
                                        let mut env = Environment::new();
                                        if env.load_file(
                                            unity_rs::helpers::import_helper::FileSource::Path(
                                                file_path.to_str().unwrap().to_string()
                                            ),
                                            None,
                                            false,
                                        ).is_none() {
                                            return Err("Failed to load file".into());
                                        }

                                        // Line 295: Create unpack directory
                                        let unpack_dir = file_path.with_file_name(
                                            format!("[unpack]{}", file_path.file_stem().unwrap().to_str().unwrap())
                                        );
                                        fs::create_dir_all(&unpack_dir)?;

                                        // Line 290-307: Extract images (Texture2D and Sprite)
                                        // Collect images in memory for alpha channel merging
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

                                                        // Save to temp directory
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
                                                // Python line 293: Extract Sprite images with alpha support
                                                let mut obj_mut = obj.clone();
                                                if let Ok(data) = obj_mut.read(false) {
                                                    if let Ok(mut sprite) = serde_json::from_value::<unity_rs::classes::generated::Sprite>(data) {
                                                        sprite.object_reader = Some(Box::new(obj.clone()));
                                                        let name_val = sprite.m_Name.as_deref().unwrap_or("unnamed");
                                                        let key = format!("(Sprite){}", name_val);
                                                        let temp_path = temp_dir.join(format!("{}.png", key));

                                                        // Get assets_file reference from object_reader (similar to AudioClip)
                                                        if let Some(ref obj_reader) = sprite.object_reader {
                                                            if let Some(concrete_reader) = obj_reader
                                                                .as_any()
                                                                .downcast_ref::<unity_rs::files::ObjectReader<()>>() {
                                                                if let Some(ref weak_assets) = concrete_reader.assets_file {
                                                                    if let Some(rc_assets) = weak_assets.upgrade() {
                                                                        // Use sprite_helper to save sprite with alpha support
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

                                        // Second pass: merge alpha channels and save final images (Python lines 296-307)
                                        let image_keys: Vec<String> = images.keys().cloned().collect();
                                        for name in &image_keys {
                                            if !name.ends_with("[alpha]") {
                                                let alpha_key = format!("{}[alpha]", name);

                                                if images.contains_key(&alpha_key) {
                                                    // Load both images
                                                    let rgb_img = image::open(&images[name])?.to_rgba8();
                                                    let alpha_img = image::open(&images[&alpha_key])?.to_luma8();

                                                    // Merge RGB with separate alpha channel
                                                    let (width, height) = rgb_img.dimensions();
                                                    let mut merged_img = image::ImageBuffer::new(width, height);

                                                    for y in 0..height {
                                                        for x in 0..width {
                                                            let rgb_pixel = rgb_img.get_pixel(x, y);
                                                            let (a_width, a_height) = alpha_img.dimensions();

                                                            // Resize alpha if dimensions don't match (Python line 301-302)
                                                            let alpha_pixel = if a_width == width && a_height == height {
                                                                alpha_img.get_pixel(x, y)[0]
                                                            } else {
                                                                // If alpha size mismatch, use full opacity
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

                                                    // Save merged image (Python line 307)
                                                    let final_path = unpack_dir.join(format!("{}.png", name));
                                                    image::DynamicImage::ImageRgba8(merged_img).save(&final_path)?;
                                                } else {
                                                    // No alpha channel, just copy the image (Python line 305)
                                                    let final_path = unpack_dir.join(format!("{}.png", name));
                                                    fs::copy(&images[name], &final_path)?;
                                                }
                                            }
                                        }

                                        // Clean up temp directory
                                        let _ = fs::remove_dir_all(&temp_dir);

                                        // Line 308-357: Process other asset types
                                        for obj in env.objects() {
                                            let mut obj_mut = obj.clone();

                                            match obj.obj_type {
                                                // Line 311-337: TextAsset
                                                ClassIDType::TextAsset => {
                                                    if let Ok(data) = obj_mut.read(false) {
                                                        if let Ok(text_asset) = serde_json::from_value::<TextAsset>(data) {
                                                            let script_str = text_asset.m_Script.as_deref().unwrap_or("");
                                                            let script_bytes = script_str.as_bytes();

                                                            // Decrypt if needed (lines 313-320)
                                                            let decrypted = {
                                                                let fpath_str = file_path.to_str().unwrap();
                                                                if (fpath_str.contains("gamedata\\levels") || fpath_str.contains("gamedata/levels"))
                                                                    && !fpath_str.contains("enemydata") {
                                                                    // Skip first 128 bytes
                                                                    if script_bytes.len() > 128 {
                                                                        script_bytes[128..].to_vec()
                                                                    } else {
                                                                        script_bytes.to_vec()
                                                                    }
                                                                } else {
                                                                    // Use text_asset_decrypt
                                                                    ArkAssets::text_asset_decrypt(script_bytes, !fpath_str.contains("gamedata/levels"))
                                                                }
                                                            };

                                                            let name_val = text_asset.m_Name.as_deref().unwrap_or("unnamed");

                                                            // Try JSON (lines 321-325)
                                                            if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&decrypted) {
                                                                let json_str = serde_json::to_string_pretty(&json_val)?;
                                                                let save_path = unpack_dir.join(format!("{}.json", name_val));
                                                                fs::write(&save_path, json_str)?;
                                                            } else {
                                                                // Try BSON (lines 327-333)
                                                                match bson::from_slice::<Vec<bson::Document>>(&decrypted) {
                                                                    Ok(docs) => {
                                                                        for (i, doc) in docs.iter().enumerate() {
                                                                            let json_val = serde_json::to_value(doc)?;
                                                                            let json_str = serde_json::to_string_pretty(&json_val)?;
                                                                            let suffix = if i >= 1 { format!("_{}", i) } else { String::new() };
                                                                            let save_path = unpack_dir.join(format!("{}{}.json", name_val, suffix));
                                                                            fs::write(&save_path, json_str)?;
                                                                        }
                                                                    }
                                                                    Err(_) => {
                                                                        // Save as raw (lines 335-337)
                                                                        let re_ext = Regex::new(r"\.(lua|atlas|skel)$")?;
                                                                        let extension = if re_ext.is_match(name_val) {
                                                                            ""
                                                                        } else {
                                                                            ".txt"
                                                                        };
                                                                        let save_path = unpack_dir.join(format!("{}{}", name_val, extension));
                                                                        fs::write(&save_path, decrypted)?;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                // Line 338-343: AudioClip
                                                ClassIDType::AudioClip => {
                                                    if let Ok(data) = obj_mut.read(false) {
                                                        if let Ok(mut audio) = serde_json::from_value::<AudioClip>(data.clone()) {
                                                            // Set object_reader for resource access
                                                            audio.object_reader = Some(Box::new(obj.clone()));

                                                            // Python lines 338-343: Extract all named samples
                                                            match unity_rs::export::audio_clip_converter::extract_audioclip_samples(&mut audio, true) {
                                                                Ok(samples) => {
                                                                    // Python: for aname, adata in data.samples.items()
                                                                    for (aname, adata) in samples {
                                                                        // Python lines 339-342
                                                                        let save_path = unpack_dir.join(&aname);
                                                                        fs::write(&save_path, adata)?;
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    eprintln!("Warning: Failed to extract audio samples: {}", e);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                // Line 344-348: Mesh
                                                ClassIDType::Mesh => {
                                                    if let Ok(data) = obj_mut.read(false) {
                                                        if let Ok(mesh) = serde_json::from_value::<Mesh>(data) {
                                                            let name_val = mesh.m_Name.as_deref().unwrap_or("unnamed");
                                                            let save_path = unpack_dir.join(format!("{}.obj", name_val));

                                                            // Use mesh exporter
                                                            if let Ok(obj_str) = unity_rs::export::mesh_exporter::export_mesh_obj(&mesh, None) {
                                                                fs::write(&save_path, obj_str)?;
                                                            }
                                                        }
                                                    }
                                                }

                                                // Line 349-357: Font
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
                                                                fs::write(&save_path, font_bytes)?;
                                                            }
                                                        }
                                                    }
                                                }

                                                _ => {}
                                            }
                                        }

                                        Ok(())
                                    })();

                                    // Line 360-362: Finally block - update unpackbar
                                    {
                                        let mut count = unpack_count_unpack.lock().unwrap();
                                        *count += 1;
                                    }
                                    {
                                        let _guard = lock_unpack.lock().unwrap();
                                        unpackbar_unpack.inc(1);
                                    }

                                    // Line 358-359: Silent error handling (pass)
                                    if let Err(_e) = unpack_result {
                                        // Silent - matches Python's "pass"
                                    }
                                });

                                // Update persistent_res_list.json
                                {
                                    let _write_guard = write_lock_clone.lock().unwrap();

                                    let mut per: HashMap<String, String> = match fs::read_to_string(&per_path_clone) {
                                        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                                        Err(_) => HashMap::new(),
                                    };

                                    per.insert(filename_clone.clone(), md5_clone.clone());

                                    let json_str = serde_json::to_string(&per)?;
                                    fs::write(&per_path_clone, json_str)?;
                                }

                                // Update zipbar
                                {
                                    let _lock_guard = lock_clone.lock().unwrap();
                                    zipbar_clone.inc(1);
                                }

                                Ok(())
                            })();

                            // Handle errors
                            if let Err(e) = unzip_result {
                                let _lock_guard = lock_clone.lock().unwrap();
                                printc(&format!("Unzip error: {}", e), &vec![31]);
                            }
                        });

                        // Store the thread handle for later joining
                        handles_clone.lock().unwrap().push(handle);
                    }

                    // Line 373: Update main progress bar
                    {
                        let _lock_guard = lock.lock().unwrap();
                        pbar.inc(1);
                    }
                }
                Err(e) => {
                    let _lock_guard = lock.lock().unwrap();
                    printc(&format!("Error: {}", e), &vec![31]);
                    pbar.inc(1);
                }
            }
        });

        // Python lines 378-382: Wait for all unpack threads to complete
        // First, wait for the counter to reach the expected value
        loop {
            let unpack_done = {
                let current_count = _unpack_count.lock().unwrap();
                *current_count >= count
            };

            if unpack_done {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Then, join all spawned threads to ensure they've fully completed
        let handles: Vec<_> = {
            let mut handles_guard = thread_handles.lock().unwrap();
            handles_guard.drain(..).collect()
        };

        for handle in handles {
            let _ = handle.join();
        }

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

        let server_tag = if self.server == Servers::OFFICIAL {
            "official"
        } else {
            "bilibili"
        };

        let url = format!(
            "https://ak.hycdn.cn/assetbundle/{}/Android/assets/{}/{}",
            server_tag, self.asset_version, url_path
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
                &vec![1, 34],
            );
        }

        Ok(res)
    }
}
