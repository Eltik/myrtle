use std::{env, time::Instant};

use reqwest::Client;
use serde::Serialize;

use crate::api::CONFIG_TIMEOUT;

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

pub async fn status(client: &Client) -> Result<StatusResponse, env::VarError> {
    let local_backend = env::var("LOCAL_BACKEND")?;
    let local_frontend = env::var("LOCAL_FRONTEND")?;
    let public_backend = env::var("PUBLIC_BACKEND")?;
    let public_frontend = env::var("PUBLIC_FRONTEND")?;

    let (lb, lf, pb, pf) = tokio::join!(
        check_endpoint(client, local_backend),
        check_endpoint(client, local_frontend),
        check_endpoint(client, public_backend),
        check_endpoint(client, public_frontend),
    );

    Ok(StatusResponse {
        local_backend: lb,
        local_frontend: lf,
        public_backend: pb,
        public_frontend: pf,
    })
}
