use crate::core::hypergryph::config::{DeviceIds, config};

pub async fn load_device_ids() {
    let mut cfg = config().write().await;
    cfg.device_ids = DeviceIds::generate();
}
