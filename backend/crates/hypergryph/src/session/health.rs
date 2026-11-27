use crate::session::account::AccountState;

/// Outage start and last success, in unix seconds the caller supplies
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HealthTracker {
    refused_since: Option<i64>,
    last_ok_at: Option<i64>,
}

#[must_use]
pub const fn state_of(refused_since: Option<i64>, last_ok_at: Option<i64>) -> AccountState {
    if refused_since.is_some() {
        AccountState::Refused
    } else if last_ok_at.is_some() {
        AccountState::Ok
    } else {
        AccountState::Unknown
    }
}

impl HealthTracker {
    #[must_use]
    pub const fn state(&self) -> AccountState {
        state_of(self.refused_since, self.last_ok_at)
    }

    #[must_use]
    pub const fn refused_since(&self) -> Option<i64> {
        self.refused_since
    }

    #[must_use]
    pub const fn last_ok_at(&self) -> Option<i64> {
        self.last_ok_at
    }

    /// Keeps the start of an outage already under way; true when this one starts it
    pub const fn refused(&mut self, now: i64) -> bool {
        if self.refused_since.is_some() {
            return false;
        }
        self.refused_since = Some(now);
        true
    }

    pub const fn ok(&mut self, now: i64) {
        self.refused_since = None;
        self.last_ok_at = Some(now);
    }
}
