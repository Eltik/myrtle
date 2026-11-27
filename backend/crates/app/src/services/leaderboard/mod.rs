pub mod board;
pub mod items;
pub mod jobs;
pub mod lookback;
pub mod search;
pub mod stats;

use myrtle_types::Server;

use crate::AppError;

fn trimmed(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|s| !s.is_empty())
}

fn server_filter(server: Option<&str>) -> Result<Option<Server>, AppError> {
    trimmed(server)
        .map(|s| {
            Server::parse(s).ok_or_else(|| AppError::bad_request(format!("unknown server `{s}`")))
        })
        .transpose()
}
