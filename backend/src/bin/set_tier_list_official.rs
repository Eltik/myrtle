//! Mark a tier list as official, or strip the official designation from it.
//!
//! A tier list's `list_type` column is what decides whether it is presented as
//! official (`"official"`) or as a user-created community list (`"community"`).
//! This tool flips that flag for a single list, looked up by slug.
//!
//! Usage:
//!   cargo run --release --bin set-tier-list-official -- <slug> --official
//!   cargo run --release --bin set-tier-list-official -- <slug> --community
//!   cargo run --release --bin set-tier-list-official -- <slug> --official --yes   # skip confirm
//!
//! `--community` is the "remove official" action — it returns the list to a
//! community list. Reads `DATABASE_URL` from the environment (or `.env`).

use anyhow::{Context, Result, bail};
use backend::database::queries;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::{
    io::{self, Write},
    time::Duration,
};

const OFFICIAL: &str = "official";
const COMMUNITY: &str = "community";

struct Args {
    slug: String,
    target: &'static str,
    yes: bool,
}

fn parse_args() -> Result<Args> {
    let mut slug: Option<String> = None;
    let mut target: Option<&'static str> = None;
    let mut yes = false;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--official" => target = Some(OFFICIAL),
            "--community" | "--unofficial" | "--remove" => target = Some(COMMUNITY),
            "-y" | "--yes" => yes = true,
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other if other.starts_with('-') => bail!("unknown flag: {other}"),
            other => {
                if slug.replace(other.to_string()).is_some() {
                    bail!("unexpected extra argument: {other}");
                }
            }
        }
    }

    let slug = slug.context("missing <slug> argument")?;
    let target = target.context("specify --official or --community")?;
    Ok(Args { slug, target, yes })
}

fn print_usage() {
    eprintln!(
        "Usage: set-tier-list-official <slug> (--official | --community) [--yes]\n\
         \n\
         Marks a tier list official or removes the official designation.\n\
         \n\
         Actions:\n  \
           --official           Present the list as an official tier list\n  \
           --community          Return the list to a community list (remove official)\n\
         \n\
         Options:\n  \
           -y, --yes            Skip the confirmation prompt\n  \
           -h, --help           Show this help"
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {e}\n");
            print_usage();
            std::process::exit(2);
        }
    };

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    let Some(list) = queries::tier_lists::find_by_slug(&pool, &args.slug)
        .await
        .context("look up tier list by slug")?
    else {
        eprintln!("No active tier list found with slug '{}'.", args.slug);
        std::process::exit(1);
    };

    println!("Tier list: {} ({})", list.name, list.slug);
    println!("  current list_type: {}", list.list_type);
    println!("  target  list_type: {}", args.target);

    if list.list_type == args.target {
        println!("\nAlready set to '{}'. Nothing to do.", args.target);
        return Ok(());
    }

    if !args.yes {
        print!("\nApply this change? (y/n): ");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        if !buf.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let updated = queries::tier_lists::set_list_type(&pool, list.id, args.target)
        .await
        .context("update list_type")?
        .context("tier list disappeared mid-update")?;

    let verb = if args.target == OFFICIAL {
        "now official"
    } else {
        "no longer official"
    };
    println!("\nDone. '{}' is {} (list_type = {}).", updated.name, verb, updated.list_type);
    println!(
        "Note: the running server caches tier-list detail by slug. Restart it or \
         flush the cache for this change to show immediately."
    );
    Ok(())
}
