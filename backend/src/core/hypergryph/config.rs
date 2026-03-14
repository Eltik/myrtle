use std::sync::{Arc, OnceLock};

use tokio::sync::RwLock;

use serde::{Deserialize, Serialize};

use crate::{
    core::hypergryph::constants::{Domain, Server},
    utils::random::{fill_random, random_digits},
};

static GLOBAL_CONFIG: OnceLock<Arc<RwLock<GlobalConfig>>> = OnceLock::new();

pub fn config() -> &'static Arc<RwLock<GlobalConfig>> {
    GLOBAL_CONFIG.get().expect("GlobalConfig not initialized")
}

pub fn init_config(cfg: GlobalConfig) {
    GLOBAL_CONFIG.set(Arc::new(RwLock::new(cfg))).ok();
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
            device_id: Self::uuid_v4(),
            device_id2: format!("86{}", random_digits(13)),
            device_id3: Self::uuid_v4(),
        }
    }

    pub fn as_array(&self) -> [&str; 3] {
        [&self.device_id, &self.device_id2, &self.device_id3]
    }

    fn uuid_v4() -> String {
        let mut bytes = [0u8; 16];
        fill_random(&mut bytes);

        bytes[6] = (bytes[6] & 0x0F) | 0x40;
        bytes[8] = (bytes[8] & 0x3F) | 0x80;

        let b = bytes;
        format!(
            "{:08x}{:04x}{:04x}{:04x}{:012x}",
            u32::from_be_bytes(b[0..4].try_into().unwrap()),
            u16::from_be_bytes(b[4..6].try_into().unwrap()),
            u16::from_be_bytes(b[6..8].try_into().unwrap()),
            u16::from_be_bytes(b[8..10].try_into().unwrap()),
            u64::from_be_bytes([0, 0, b[10], b[11], b[12], b[13], b[14], b[15]]),
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub res_version: String,
    pub client_version: String,
}

type ServerMap<T> = [T; 6];
type DomainMap<T> = [T; 12];

#[derive(Debug, Clone, Default)]
pub struct GlobalConfig {
    pub device_ids: DeviceIds,
    pub domains: ServerMap<DomainMap<Option<String>>>,
    pub versions: ServerMap<VersionInfo>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a domain URL for a specific server + domain pair
    pub fn domain(&self, server: Server, domain: Domain) -> Option<&str> {
        self.domains[server.index()][domain.index()].as_deref()
    }

    /// Set a domain URL for a specific server + domain pair
    pub fn set_domain(&mut self, server: Server, domain: Domain, url: String) {
        self.domains[server.index()][domain.index()] = Some(url);
    }

    /// Get version info for a server
    pub fn version(&self, server: Server) -> &VersionInfo {
        &self.versions[server.index()]
    }

    /// Set version info for a server
    pub fn set_version(&mut self, server: Server, info: VersionInfo) {
        self.versions[server.index()] = info;
    }

    pub fn reset_network(&mut self) {
        self.domains = Default::default();
    }

    pub fn reset_versions(&mut self) {
        self.versions = Default::default();
    }
}
