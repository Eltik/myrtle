use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{PgPool, Postgres};

use crate::DbError;

pub type Conn = sqlx::PgConnection;

#[derive(Debug, Clone)]
pub struct PoolOptions {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub statement_timeout: Option<Duration>,
    pub lock_timeout: Option<Duration>,
    pub idle_in_transaction_timeout: Option<Duration>,
}

impl PoolOptions {
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_connections: 40,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_mins(10),
            max_lifetime: Duration::from_mins(30),
            statement_timeout: None,
            lock_timeout: None,
            idle_in_transaction_timeout: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn connect(opts: &PoolOptions) -> Result<Self, DbError> {
        let timeouts = [
            ("statement_timeout", opts.statement_timeout),
            ("lock_timeout", opts.lock_timeout),
            (
                "idle_in_transaction_session_timeout",
                opts.idle_in_transaction_timeout,
            ),
        ]
        .into_iter()
        .filter_map(|(name, t)| Some((name, format!("{}ms", t?.as_millis().max(1)))));
        let connect = PgConnectOptions::from_str(&opts.url)?.options(timeouts);

        let pool = PgPoolOptions::new()
            .max_connections(opts.max_connections)
            .min_connections(opts.min_connections)
            .acquire_timeout(opts.acquire_timeout)
            .idle_timeout(opts.idle_timeout)
            .max_lifetime(opts.max_lifetime)
            .test_before_acquire(false) // a round trip per checkout otherwise
            .connect_with(connect)
            .await?;
        Ok(Self { pool })
    }

    #[must_use]
    pub const fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn conn(&self) -> Result<PoolConn, DbError> {
        Ok(PoolConn(self.pool.acquire().await?))
    }

    pub async fn begin(&self) -> Result<Tx, DbError> {
        Ok(Tx(self.pool.begin().await?))
    }

    pub async fn ping(&self) -> Result<(), DbError> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[derive(Debug)]
pub struct PoolConn(sqlx::pool::PoolConnection<Postgres>);

impl PoolConn {
    pub async fn close(self) {
        self.0.close().await.ok();
    }
}

impl Deref for PoolConn {
    type Target = Conn;

    fn deref(&self) -> &Conn {
        &self.0
    }
}

impl DerefMut for PoolConn {
    fn deref_mut(&mut self) -> &mut Conn {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct Tx(sqlx::Transaction<'static, Postgres>);

impl Tx {
    pub async fn commit(self) -> Result<(), DbError> {
        Ok(self.0.commit().await?)
    }

    pub async fn rollback(self) -> Result<(), DbError> {
        Ok(self.0.rollback().await?)
    }
}

impl Deref for Tx {
    type Target = Conn;

    fn deref(&self) -> &Conn {
        &self.0
    }
}

impl DerefMut for Tx {
    fn deref_mut(&mut self) -> &mut Conn {
        &mut self.0
    }
}
