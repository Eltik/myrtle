use std::time::Instant;

use reqwest::Client;
use serde::Serialize;

use crate::api::CONFIG_TIMEOUT;
use crate::config::EndpointsConfig;

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub local_backend: EndpointStatus,
    pub local_frontend: EndpointStatus,
    pub public_backend: EndpointStatus,
    pub public_frontend: EndpointStatus,
}

#[derive(Debug, Serialize)]
pub struct EndpointStatus {
    pub url: String,
    pub reachable: bool,
    pub status_code: Option<u16>,
    pub response_time_ms: Option<u128>,
}

async fn check_endpoint(client: &Client, url: String) -> EndpointStatus {
    let start = Instant::now();
    let result = client
        .get(&url)
        .timeout(CONFIG_TIMEOUT)
        .send()
        .await;

    match result {
        Ok(response) => EndpointStatus {
            url,
            reachable: true,
            status_code: Some(response.status().as_u16()),
            response_time_ms: Some(start.elapsed().as_millis()),
        },
        Err(_) => EndpointStatus {
            url,
            reachable: false,
            status_code: None,
            response_time_ms: None,
        },
    }
}

pub async fn status(client: &Client, endpoints: &EndpointsConfig) -> StatusResponse {
    let (lb, lf, pb, pf) = tokio::join!(
        check_endpoint(client, endpoints.local_backend.clone()),
        check_endpoint(client, endpoints.local_frontend.clone()),
        check_endpoint(client, endpoints.public_backend.clone()),
        check_endpoint(client, endpoints.public_frontend.clone()),
    );

    StatusResponse {
        local_backend: lb,
        local_frontend: lf,
        public_backend: pb,
        public_frontend: pf,
    }
}
