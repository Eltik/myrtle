use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};

use serde::{Deserialize, Serialize};

use crate::core::hypergryph::constants::{Domain, Server};

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
            device_id2: format!("86{}", Self::random_digits(13)),
            device_id3: Self::new_v4(),
        }
    }

    pub fn as_array(&self) -> [&str; 3] {
        [&self.device_id, &self.device_id2, &self.device_id3]
    }

    fn uuid_v4() -> String {
        let mut bytes = [0u8; 16];
        Self::fill_random(&mut bytes);

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

    fn random_digits(count: usize) -> String {
        let mut buf = [0u8; 1];
        (0..count)
            .map(|_| {
                Self::fill_random(&mut buf);
                char::from_digit((buf[0] % 10) as u32, 10).unwrap()
            })
            .collect()
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
