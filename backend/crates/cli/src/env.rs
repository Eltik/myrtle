use std::path::Path;

use anyhow::{Context, anyhow, bail};
use myrtle_app::AppContext;
use myrtle_app::config::Config;
use myrtle_app::context::{BootStep, GameData};
use myrtle_app::policy::Subject;
use myrtle_app::services::accounts::jwt::parse_role;
use myrtle_db::repos::users::{self, AccountRow};
use myrtle_db::{Db, PoolOptions};
use myrtle_types::{Server, UserId};

pub struct Env {
    pub config: Config,
}

impl Env {
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        Ok(Self {
            config: Config::load(path)?,
        })
    }

    pub async fn db(&self) -> anyhow::Result<Db> {
        let url = self.config.database.url.resolve()?;
        Ok(Db::connect(&PoolOptions {
            max_connections: 2,
            ..PoolOptions::new(url)
        })
        .await?)
    }

    pub async fn ctx(&self, game_data: GameData) -> anyhow::Result<AppContext> {
        let ctx = AppContext::open(self.config.clone(), game_data, |step| {
            if let BootStep::GameData(server) = step {
                tracing::info!(server = server.as_str(), "loading game data");
            }
        })
        .await?;
        Ok(ctx)
    }

    pub fn server(&self, arg: Option<&str>) -> anyhow::Result<Server> {
        let configured = Some(self.config.cli.default_server.as_str()).filter(|s| !s.is_empty());
        match arg.or(configured) {
            Some(code) => Server::parse(&code.to_lowercase())
                .ok_or_else(|| anyhow!("unknown server {code:?}")),
            None => {
                (self.config.server.servers.first().copied()).context("server.servers is empty")
            }
        }
    }

    pub async fn admin(&self, ctx: &AppContext) -> anyhow::Result<Subject> {
        let admin = &self.config.cli.admin_uid;
        if admin.is_empty() {
            bail!("set [cli] admin_uid in the config: this command records who acted");
        }
        let account = find_account(&ctx.db, admin, None).await?;
        Ok(Subject::User {
            id: UserId(account.id),
            uid: account.uid.as_str().into(),
            server: Server::from_index(account.server_id),
            role: parse_role(&account.role),
            grants: [].into(),
        })
    }
}

pub async fn find_account(
    db: &Db,
    user: &str,
    server: Option<Server>,
) -> anyhow::Result<AccountRow> {
    let mut conn = db.conn().await?;
    if let Ok(id) = user.parse()
        && let Some(account) = users::account(&mut conn, UserId(id)).await?
    {
        return Ok(account);
    }
    let mut accounts = users::accounts_by_uid(&mut conn, user).await?;
    if let Some(server) = server {
        accounts.retain(|a| a.server_id == server.index()); // stored codes predate the lowercase seed
    }
    match accounts.len() {
        0 => bail!("no user {user:?}"),
        1 => Ok(accounts.remove(0)),
        _ => {
            let servers: Vec<&str> = accounts.iter().map(|a| a.server.as_str()).collect();
            bail!("uid {user} exists on {}: pass --server", servers.join(", "))
        }
    }
}
