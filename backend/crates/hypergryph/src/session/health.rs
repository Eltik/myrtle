use crate::session::account::AccountState;

/// Outage start and last success, in unix seconds the caller supplies
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HealthTracker {
    refused_since: Option<i64>,
    last_ok_at: Option<i64>,
}

impl HealthTracker {
    #[must_use]
    pub const fn state(&self) -> AccountState {
        if self.refused_since.is_some() {
            AccountState::Refused
        } else if self.last_ok_at.is_some() {
            AccountState::Ok
        } else {
            AccountState::Unknown
        }
    }

    #[must_use]
    pub const fn refused_since(&self) -> Option<i64> {
        self.refused_since
    }

    #[must_use]
    pub const fn last_ok_at(&self) -> Option<i64> {
        self.last_ok_at
    }

    /// Keeps the start of an outage already under way
    pub const fn refused(&mut self, now: i64) {
        if self.refused_since.is_none() {
            self.refused_since = Some(now);
        }
    }

    pub const fn ok(&mut self, now: i64) {
        self.refused_since = None;
        self.last_ok_at = Some(now);
    }
}
