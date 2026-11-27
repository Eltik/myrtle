use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("not found")]
    NotFound,
    #[error("unique violation on {constraint}")]
    Conflict { constraint: String },
    #[error("foreign key violation on {constraint}")]
    ForeignKey { constraint: String },
    #[error("value too long")]
    TooLong,
    #[error("pool exhausted")]
    PoolTimedOut,
    #[error("schema has not applied {expected}")]
    LegacyBehind { expected: &'static str },
    #[error("schema is behind migration {version}")]
    SchemaBehind { version: i64 },
    #[error(transparent)]
    Migrate(#[from] MigrateError),
    #[error(transparent)]
    Other(sqlx::Error),
}

impl From<sqlx::Error> for DbError {
    fn from(e: sqlx::Error) -> Self {
        let constraint =
            |db: &dyn sqlx::error::DatabaseError| db.constraint().unwrap_or_default().to_owned();
        match &e {
            sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::PoolTimedOut => Self::PoolTimedOut,
            sqlx::Error::Database(db) => match db.code().as_deref() {
                Some("23505") => Self::Conflict {
                    constraint: constraint(db.as_ref()),
                },
                Some("23503") => Self::ForeignKey {
                    constraint: constraint(db.as_ref()),
                },
                Some("22001") => Self::TooLong,
                _ => Self::Other(e),
            },
            _ => Self::Other(e),
        }
    }
}
