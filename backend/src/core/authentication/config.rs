use super::constants::{Domain, Server};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

fn generate_random_digits(length: usize) -> String {
    (0..length)
        .map(|_| rand::rng().random_range(0..10).to_string())
        .collect()
}

#[derive(Debug, Clone, Default)]
pub struct DeviceIds {
    pub device_id: String,  // UUID (no dashes)
    pub device_id2: String, // "86" + 13 random digits
    pub device_id3: String, // UUID (no dashes)
}

impl DeviceIds {
    pub fn generate() -> Self {
        Self {
            device_id: Uuid::new_v4().to_string().replace("-", ""),
            device_id2: format!("86{}", generate_random_digits(13)),
            device_id3: Uuid::new_v4().to_string().replace("-", ""),
        }
    }

    pub fn as_array(&self) -> [&str; 3] {
        [&self.device_id, &self.device_id2, &self.device_id3]
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub res_version: String,
    pub client_version: String,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalConfig {
    pub device_ids: DeviceIds,
    pub domains: HashMap<Server, HashMap<Domain, String>>,
    pub versions: HashMap<Server, VersionInfo>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset_network(&mut self) {
        self.domains.clear();
        for server in Server::all() {
            self.domains.insert(*server, HashMap::new());
        }
    }

    pub fn reset_versions(&mut self) {
        self.versions.clear();
        for server in Server::all() {
            self.versions.insert(*server, VersionInfo::default());
        }
    }
}
