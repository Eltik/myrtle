use anyhow::{Context, bail};
use clap::{Subcommand, ValueEnum};
use myrtle_app::context::GameData;
use myrtle_app::services::tier_lists;
use myrtle_db::repos::tier_lists as repo;
use myrtle_types::TierListType;

use crate::Env;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Toggle {
    On,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ListType {
    Official,
    Community,
}

impl From<ListType> for TierListType {
    fn from(t: ListType) -> Self {
        match t {
            ListType::Official => Self::Official,
            ListType::Community => Self::Community,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Every tier list with its type, visibility, and size
    List,
    /// Show or hide a tier list publicly
    Visibility { slug: String, listed: Toggle },
    /// Mark a tier list official or community
    SetType {
        slug: String,
        list_type: ListType,
        #[arg(long, short)]
        yes: bool,
    },
}

pub async fn run(env: &Env, command: Command) -> anyhow::Result<()> {
    match command {
        Command::List => list(env).await,
        Command::Visibility { slug, listed } => {
            let ctx = env.ctx(GameData::None).await?;
            let admin = env.admin(&ctx).await?;
            let state =
                tier_lists::set_visibility(&ctx, &admin, &slug, listed == Toggle::On).await?;
            println!(
                "{slug} is {}",
                if state.is_listed {
                    "listed"
                } else {
                    "unlisted"
                }
            );
            Ok(())
        }
        Command::SetType {
            slug,
            list_type,
            yes,
        } => set_type(env, &slug, list_type.into(), yes).await,
    }
}

async fn list(env: &Env) -> anyhow::Result<()> {
    let db = env.db().await?;
    let rows = repo::list_details(&mut *db.conn().await?).await?;
    if rows.is_empty() {
        println!("no tier lists");
        return Ok(());
    }
    let width = |text: fn(&repo::ListDetailRow) -> &str| {
        rows.iter()
            .map(|row| text(row).chars().count())
            .max()
            .map_or(4, |w| w.max(4))
    };
    let (name, slug) = (width(|row| &row.name), width(|row| &row.slug));
    println!(
        "{:<name$}  {:<slug$}  {:<9}  {:<6}  {:<6}  {:>5}  {:>5}",
        "name", "slug", "type", "active", "listed", "tiers", "ops"
    );
    for row in &rows {
        println!(
            "{:<name$}  {:<slug$}  {:<9}  {:<6}  {:<6}  {:>5}  {:>5}",
            row.name,
            row.slug,
            row.list_type,
            yes_no(row.is_active),
            yes_no(row.is_listed),
            row.tier_count,
            row.placement_count
        );
    }
    Ok(())
}

async fn set_type(env: &Env, slug: &str, to: TierListType, yes: bool) -> anyhow::Result<()> {
    let ctx = env.ctx(GameData::None).await?;
    let list = repo::find_by_slug(&mut *ctx.db.conn().await?, slug)
        .await?
        .with_context(|| format!("no active tier list {slug:?}"))?;
    println!(
        "{} ({}): {} -> {}",
        list.name,
        list.slug,
        list.list_type,
        to.as_str()
    );
    if list.list_type == to.as_str() {
        println!("nothing to do");
        return Ok(());
    }
    if !yes {
        bail!("pass --yes to apply");
    }
    let admin = env.admin(&ctx).await?;
    tier_lists::set_list_type(&ctx, &admin, slug, to).await?;
    println!("done");
    Ok(())
}

const fn yes_no(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}
