//! A migrated, seeded database of its own for one test.

use std::time::Duration;

use myrtle_db::repos::users;
use myrtle_db::{Db, PoolOptions, migrate};
use myrtle_types::{Server, UserId};
use sqlx::{AssertSqlSafe, Connection, PgConnection};

const LOCAL: &str = "postgres://postgres:password@127.0.0.1:5432/postgres";

const CLEANUP_TIMEOUT: Duration = Duration::from_secs(10);

/// Dropped with its database, also when the test panics before `close`
pub struct TestDb {
    pub db: Db,
    guard: Guard,
}

/// Drops the database from the moment it exists, so a panic while connecting or migrating leaks nothing
struct Guard {
    name: String,
    admin_url: String,
}

async fn drop_database(admin_url: &str, name: &str) {
    let dropped = tokio::time::timeout(CLEANUP_TIMEOUT, async {
        let mut admin = PgConnection::connect(admin_url).await?;
        let drop = format!("DROP DATABASE IF EXISTS {name} WITH (FORCE)");
        sqlx::query(AssertSqlSafe(drop)).execute(&mut admin).await?;
        Ok::<_, sqlx::Error>(())
    })
    .await;
    match dropped {
        Ok(Ok(())) => {}
        Ok(Err(e)) => eprintln!("test database {name} was not dropped: {e}"),
        Err(_) => eprintln!("test database {name} was not dropped within {CLEANUP_TIMEOUT:?}"),
    }
}

fn admin_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| LOCAL.to_owned())
}

/// A fresh database on the server `DATABASE_URL` names. Panics when it is unreachable.
pub async fn database() -> TestDb {
    let test = unmigrated().await;
    migrate::run(&test.db)
        .await
        .expect("migrate the test database");
    test
}

/// [`database`] before any migration ran
pub async fn unmigrated() -> TestDb {
    let admin_url = admin_url();
    let name = format!("myrtle_test_{}", uuid::Uuid::new_v4().simple());
    let mut admin = PgConnection::connect(&admin_url)
        .await
        .expect("DATABASE_URL names a reachable Postgres");
    sqlx::query(AssertSqlSafe(format!("CREATE DATABASE {name}")))
        .execute(&mut admin)
        .await
        .expect("create the test database");
    let guard = Guard {
        name: name.clone(),
        admin_url,
    };

    let (server, _) = guard
        .admin_url
        .rsplit_once('/')
        .expect("DATABASE_URL has a path");
    let mut options = PoolOptions::new(format!("{server}/{name}"));
    options.max_connections = 5;
    options.min_connections = 0;
    let db = Db::connect(&options)
        .await
        .expect("connect to the test database");
    TestDb { db, guard }
}

impl TestDb {
    pub async fn close(self) {
        let Self { db, mut guard } = self;
        db.close().await;
        drop_database(&guard.admin_url, &std::mem::take(&mut guard.name)).await;
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.guard.name
    }

    /// A `users` row on `server` with uid `n`
    pub async fn user(&self, n: u32, server: Server) -> UserId {
        let mut conn = self.db.conn().await.expect("a connection");
        let row = users::find_or_create(&mut conn, &n.to_string(), server)
            .await
            .expect("create the user");
        UserId(row.id)
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if self.name.is_empty() {
            return;
        }
        let (admin_url, name) = (self.admin_url.clone(), std::mem::take(&mut self.name));
        let dropped = std::thread::spawn(move || {
            // the test's runtime may be the one unwinding, so this runs on its own
            match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime.block_on(drop_database(&admin_url, &name)),
                Err(e) => eprintln!("test database {name} was not dropped: {e}"),
            }
        });
        if dropped.join().is_err() {
            eprintln!("the test database cleanup thread panicked");
        }
    }
}
