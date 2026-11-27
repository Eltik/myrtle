use myrtle_db::DbError;
use myrtle_hypergryph::UpstreamError;
use myrtle_jobs::QueueError;
use myrtle_types::FieldError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("validation failed")]
    Validation(Vec<FieldError>),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("{}", .0.as_deref().unwrap_or("not found"))]
    NotFound(Option<String>),
    #[error("{0}")]
    Conflict(String),
    #[error("rate limited")]
    RateLimited,
    #[error("no game session - login again")]
    GameLoginRequired,
    #[error("the game is down for maintenance")]
    GameMaintenance { ends_at: Option<i64> },
    #[error("service unavailable")]
    Unavailable,
    #[error(transparent)]
    Upstream(#[from] UpstreamError),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn internal(e: impl std::fmt::Display) -> Self {
        Self::Internal(e.to_string())
    }

    #[must_use]
    pub fn shared_copy(&self) -> Self {
        match self {
            Self::BadRequest(m) => Self::BadRequest(m.clone()),
            Self::Validation(errors) => Self::Validation(errors.clone()),
            Self::Unauthorized => Self::Unauthorized,
            Self::Forbidden => Self::Forbidden,
            Self::NotFound(m) => Self::NotFound(m.clone()),
            Self::Conflict(m) => Self::Conflict(m.clone()),
            Self::RateLimited => Self::RateLimited,
            Self::GameLoginRequired => Self::GameLoginRequired,
            Self::GameMaintenance { ends_at } => Self::GameMaintenance { ends_at: *ends_at },
            Self::Unavailable => Self::Unavailable,
            Self::Upstream(UpstreamError::NotLoggedIn) => Self::GameLoginRequired,
            Self::Upstream(UpstreamError::Maintenance(_)) => {
                Self::GameMaintenance { ends_at: None }
            }
            Self::Upstream(e) => Self::Internal(e.to_string()),
            Self::Internal(m) => Self::Internal(m.clone()),
        }
    }
}

impl From<DbError> for AppError {
    fn from(e: DbError) -> Self {
        match e {
            DbError::NotFound => Self::NotFound(None),
            DbError::Conflict { .. } => Self::Conflict("resource already exists".into()),
            DbError::ForeignKey { .. } => {
                Self::Conflict("a referenced resource does not exist".into())
            }
            DbError::TooLong => Self::BadRequest("value too long".into()),
            DbError::PoolTimedOut => {
                tracing::warn!("database pool exhausted; shedding request");
                Self::Unavailable
            }
            other => Self::Internal(other.to_string()),
        }
    }
}

impl From<QueueError> for AppError {
    fn from(e: QueueError) -> Self {
        match e {
            QueueError::Db(e) => e.into(),
            lost @ QueueError::LeaseLost(_) => Self::Internal(lost.to_string()),
        }
    }
}
