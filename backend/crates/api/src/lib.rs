pub mod boot;
pub mod error;
pub mod extract;
pub mod health;
pub mod metrics;
pub mod middleware;
pub mod respond;
pub mod router;
pub mod routes;
pub mod server_prefix;

use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use myrtle_app::config::Config;
use myrtle_app::context::BootStep;
use myrtle_app::{AppContext, AppError};
use myrtle_db::{Db, PoolOptions};
use tokio::net::TcpListener;

pub use error::{ApiError, ApiResult};

const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);
const RELOAD_POLL: Duration = Duration::from_secs(15);

async fn migrate(config: &Config) -> Result<(), AppError> {
    let url = config.database.url.resolve().map_err(AppError::internal)?;
    let db = Db::connect(&PoolOptions {
        max_connections: 2,
        ..PoolOptions::new(url)
    })
    .await?;
    myrtle_db::migrate::run(&db).await?;
    Ok(())
}

pub async fn run(config: Option<&Path>) -> Result<(), AppError> {
    let config = Config::load(config).map_err(AppError::internal)?;
    boot::begin(4 + config.server.servers.len());
    boot::step("migrations");
    migrate(&config).await?;
    let ctx = AppContext::build_with(config, |step| match step {
        BootStep::Database => boot::step("database and cache"),
        BootStep::GameData(server) => boot::step(format!("game data {}", server.as_str())),
        BootStep::Upstream => boot::step("upstream versions"),
        BootStep::Accounts => boot::step("service accounts"),
    })
    .await?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    for switch in ctx.config.expired_switches(&today) {
        tracing::warn!(
            switch = switch.name,
            owner = switch.owner,
            remove_after = switch.remove_after,
            "kill switch is past its removal date"
        );
    }

    tokio::spawn(follow_game_data(ctx.clone()));
    tokio::spawn(metrics::serve(ctx.clone()));

    let server = &ctx.config.server;
    let listener = TcpListener::bind((server.bind_host.as_str(), server.port))
        .await
        .map_err(AppError::internal)?;
    let boot_secs = boot::done().as_secs();
    tracing::info!(
        host = server.bind_host,
        port = server.port,
        boot_secs,
        cpu_permits = ctx.cpu.permits(),
        "listening"
    );

    let (signalled_tx, signalled_rx) = tokio::sync::oneshot::channel::<()>();
    let serving = axum::serve(
        listener,
        router::build(ctx).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        shutdown_signal().await;
        let _ = signalled_tx.send(());
    });
    tokio::select! {
        served = serving => served.map_err(AppError::internal),
        () = async {
            let _ = signalled_rx.await;
            tokio::time::sleep(SHUTDOWN_GRACE).await;
        } => {
            tracing::warn!(grace_secs = SHUTDOWN_GRACE.as_secs(), "drain timed out, abandoning in-flight requests");
            Ok(())
        }
    }
}

async fn follow_game_data(ctx: AppContext) {
    let mut tick = tokio::time::interval(RELOAD_POLL);
    loop {
        tick.tick().await;
        let ctx = ctx.clone();
        let checked = tokio::spawn(async move { ctx.reload_if_changed().await }).await;
        match checked {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => tracing::warn!(error = %e, "game data generation check failed"),
            Err(e) => tracing::error!(error = %e, "game data reload panicked"),
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => tracing::info!("received Ctrl+C, shutting down"),
        () = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}
