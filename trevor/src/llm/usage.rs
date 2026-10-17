//! Token and call counts over every call of a client and its clones.

use std::sync::atomic::AtomicU64;

use serde::Serialize;

/// Running totals over every call of this client (and its clones).
#[derive(Debug, Default)]
pub(super) struct Usage {
    pub(super) calls: AtomicU64,
    pub(super) tokens_in: AtomicU64,
    pub(super) tokens_out: AtomicU64,
    pub(super) json_calls: AtomicU64,
    pub(super) json_retries: AtomicU64,
    pub(super) json_failures: AtomicU64,
}

/// A copy of the totals; subtract two to get one job's share.
#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub calls: u64,
    /// Every prompt token, cached or not.
    pub tokens_in: u64,
    pub tokens_out: u64,
    /// Grammar-bound calls answered as plain JSON prompts (OpenAI-compatible backend only).
    pub json_calls: u64,
    /// Of those, the ones whose first reply did not validate.
    pub json_retries: u64,
    /// And the ones whose second reply did not validate either.
    pub json_failures: u64,
}

impl std::ops::Sub for UsageSnapshot {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self { calls: self.calls - o.calls, tokens_in: self.tokens_in - o.tokens_in,
               tokens_out: self.tokens_out - o.tokens_out,
               json_calls: self.json_calls - o.json_calls, json_retries: self.json_retries - o.json_retries,
               json_failures: self.json_failures - o.json_failures }
    }
}
