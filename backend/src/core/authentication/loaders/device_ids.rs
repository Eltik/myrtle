use std::sync::Arc;

use tokio::sync::RwLock;

use crate::core::authentication::config::{DeviceIds, GlobalConfig};

pub async fn load_device_ids(config: &Arc<RwLock<GlobalConfig>>) {
    let mut config = config.write().await;
    config.device_ids = DeviceIds::generate();
}
