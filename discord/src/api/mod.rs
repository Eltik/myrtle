use std::time::Duration;

pub mod gamedata;
pub mod operator_detail;
pub mod stats;
pub mod status;

pub const CONFIG_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout on the shared HTTP client, for a request that sets none of its own.
///
/// Every request in the crate sets one today (2 s to 30 s, and a per-request timeout wins over
/// this); 30 s is the longest of them, `gamedata::LIST_TIMEOUT`, so no current request is cut
/// shorter than it already is, and a future one that forgets cannot hang forever.
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);
