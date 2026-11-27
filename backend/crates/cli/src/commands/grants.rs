use anyhow::{Context, bail};
use clap::{Subcommand, ValueEnum};
use myrtle_app::context::GameData;
use myrtle_app::policy::{Permission, Subject};
use myrtle_app::services::accounts::account;
use myrtle_app::services::accounts::jwt::role_name;
use myrtle_app::services::tier_lists::{self, GrantRequest};
use myrtle_db::repos::users::{self, AccountRow};
use myrtle_types::{Role, Server, UserId};

use crate::Env;
use crate::env::find_account;

const RECENT: i64 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum RoleArg {
    User,
    Translator,
    TierListEditor,
    TierListAdmin,
    SuperAdmin,
}

impl From<RoleArg> for Role {
    fn from(r: RoleArg) -> Self {
        match r {
            RoleArg::User => Self::User,
            RoleArg::Translator => Self::Translator,
            RoleArg::TierListEditor => Self::TierListEditor,
            RoleArg::TierListAdmin => Self::TierListAdmin,
            RoleArg::SuperAdmin => Self::SuperAdmin,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Level {
    View,
    Edit,
    Publish,
    Admin,
    All,
}

impl Level {
    const fn permission(self) -> Option<Permission> {
        match self {
            Self::View => Some(Permission::View),
            Self::Edit => Some(Permission::Edit),
            Self::Publish => Some(Permission::Publish),
            Self::Admin => Some(Permission::Admin),
            Self::All => None,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Search users by uid or nickname
    Users { search: Option<String> },
    /// Set an user's site role
    Role {
        /// UUID or game uid
        user: String,
        role: RoleArg,
        /// Server of the uid
        #[arg(long)]
        server: Option<String>,
    },
    /// Show who holds permissions on a tier list
    List { slug: String },
    /// Grant a tier-list permission
    Add {
        slug: String,
        /// UUID or game uid
        user: String,
        permission: Level,
        /// Server of the uid
        #[arg(long)]
        server: Option<String>,
    },
    /// Revoke a tier-list permission or all of an user's
    Remove {
        slug: String,
        /// UUID or game uid
        user: String,
        permission: Level,
        /// Server of the uid
        #[arg(long)]
        server: Option<String>,
    },
}

pub async fn run(env: &Env, command: Command) -> anyhow::Result<()> {
    match command {
        Command::Users { search } => list_users(env, search.as_deref()).await,
        Command::Role { user, role, server } => {
            set_role(env, &user, role_name(role.into()), server).await
        }
        Command::List { slug } => list_grants(env, &slug).await,
        Command::Add {
            slug,
            user,
            permission,
            server,
        } => {
            let Some(permission) = permission.permission() else {
                bail!("grant one permission: view, edit, publish or admin");
            };
            let ctx = env.ctx(GameData::None).await?;
            let admin = env.admin(&ctx).await?;
            let target = find_account(&ctx.db, &user, server_of(env, server)?).await?;
            let req = GrantRequest {
                user_id: target.id,
                permission: permission.as_str().to_owned(),
            };
            tier_lists::grant(&ctx, &admin, &slug, &req).await?;
            println!(
                "granted {} on {slug} to {}",
                permission.as_str(),
                label(&target)
            );
            Ok(())
        }
        Command::Remove {
            slug,
            user,
            permission,
            server,
        } => revoke(env, &slug, &user, permission, server).await,
    }
}

fn server_of(env: &Env, server: Option<String>) -> anyhow::Result<Option<Server>> {
    server.map(|s| env.server(Some(&s))).transpose()
}

async fn list_users(env: &Env, search: Option<&str>) -> anyhow::Result<()> {
    let db = env.db().await?;
    let mut conn = db.conn().await?;
    let rows = match search.map(str::trim).filter(|s| !s.is_empty()) {
        Some(needle) => users::search_accounts(&mut conn, needle).await?,
        None => users::recent(&mut conn, RECENT).await?,
    };
    if rows.is_empty() {
        println!("no users");
        return Ok(());
    }
    println!(
        "{:<36}  {:<12}  {:<6}  {:<20}  role",
        "id", "uid", "server", "nickname"
    );
    for row in &rows {
        println!(
            "{:<36}  {:<12}  {:<6}  {:<20}  {}",
            row.id,
            row.uid,
            row.server,
            row.nickname.as_deref().unwrap_or("-"),
            row.role
        );
    }
    Ok(())
}

async fn set_role(env: &Env, user: &str, role: &str, server: Option<String>) -> anyhow::Result<()> {
    let ctx = env.ctx(GameData::None).await?;
    let target = find_account(&ctx.db, user, server_of(env, server)?).await?;
    if target.role == role {
        println!("{} is already {role}", label(&target));
        return Ok(());
    }
    account::set_user_role(&ctx, &Subject::System, UserId(target.id), role).await?;
    println!("{}: {} -> {role}", label(&target), target.role);
    Ok(())
}

async fn list_grants(env: &Env, slug: &str) -> anyhow::Result<()> {
    let ctx = env.ctx(GameData::None).await?;
    let grants = tier_lists::permissions(&ctx, &Subject::System, slug)
        .await
        .with_context(|| format!("tier list {slug}"))?;
    if grants.is_empty() {
        println!("no permissions on {slug}");
        return Ok(());
    }
    let mut conn = ctx.db.conn().await?;
    println!(
        "{:<10}  {:<12}  {:<6}  {:<20}  granted",
        "permission", "uid", "server", "nickname"
    );
    for grant in &grants {
        let account = users::account(&mut conn, UserId(grant.user_id)).await?;
        let (uid, server, nickname) = account.as_ref().map_or(("?", "?", "-"), |a| {
            (
                a.uid.as_str(),
                a.server.as_str(),
                a.nickname.as_deref().unwrap_or("-"),
            )
        });
        println!(
            "{:<10}  {uid:<12}  {server:<6}  {nickname:<20}  {}",
            grant.permission,
            grant.granted_at.format("%Y-%m-%d %H:%M")
        );
    }
    Ok(())
}

async fn revoke(
    env: &Env,
    slug: &str,
    user: &str,
    level: Level,
    server: Option<String>,
) -> anyhow::Result<()> {
    let ctx = env.ctx(GameData::None).await?;
    let admin = env.admin(&ctx).await?;
    let target = find_account(&ctx.db, user, server_of(env, server)?).await?;
    let held: Vec<String> = tier_lists::permissions(&ctx, &admin, slug)
        .await
        .with_context(|| format!("tier list {slug}"))?
        .into_iter()
        .filter(|p| p.user_id == target.id)
        .map(|p| p.permission)
        .collect();
    let revoking: Vec<String> = match level.permission() {
        Some(wanted) => held.into_iter().filter(|p| p == wanted.as_str()).collect(),
        None => held,
    };
    if revoking.is_empty() {
        bail!("{} holds no such permission on {slug}", label(&target));
    }
    for held in &revoking {
        tier_lists::revoke(&ctx, &admin, slug, UserId(target.id), held).await?;
        println!("revoked {held} on {slug} from {}", label(&target));
    }
    Ok(())
}

fn label(account: &AccountRow) -> String {
    match &account.nickname {
        Some(nickname) => format!("{} ({nickname}, {})", account.uid, account.server),
        None => format!("{} ({})", account.uid, account.server),
    }
}
