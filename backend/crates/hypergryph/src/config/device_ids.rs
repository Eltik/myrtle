use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceIds {
    /// UUID v4
    pub device_id: String,
    /// `86` and 13 digits
    pub device_id2: String,
    /// UUID v4
    pub device_id3: String,
}

impl DeviceIds {
    #[must_use]
    pub fn generate() -> Self {
        Self {
            device_id: uuid_v4(),
            device_id2: format!("86{}", random_digits(13)),
            device_id3: uuid_v4(),
        }
    }

    #[must_use]
    pub const fn is_complete(&self) -> bool {
        !self.device_id.is_empty() && !self.device_id2.is_empty() && !self.device_id3.is_empty()
    }
}

fn uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    bytes[6] = (bytes[6] & 0x0F) | 0x40;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    hex::encode(bytes)
}

fn random_digits(count: usize) -> String {
    let mut buf = vec![0u8; count];
    rand::rng().fill_bytes(&mut buf);
    buf.iter().map(|b| char::from(b'0' + b % 10)).collect()
}
