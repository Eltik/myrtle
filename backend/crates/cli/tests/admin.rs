#![allow(clippy::unwrap_used)]

use clap::Parser;
use myrtle_app::context::GameData;
use myrtle_cli::commands::grants::{self, Level, RoleArg};
use myrtle_cli::commands::regrade;
use myrtle_cli::commands::stats::{self, TASKS, game_data, select};
use myrtle_cli::commands::tier_list::{self, ListType, Toggle};
use myrtle_cli::{Cli, Command};

fn parse(args: &[&str]) -> Command {
    let argv = std::iter::once("myrtle").chain(args.iter().copied());
    Cli::try_parse_from(argv).unwrap().command
}

fn rejects(args: &[&str]) -> bool {
    let argv = std::iter::once("myrtle").chain(args.iter().copied());
    Cli::try_parse_from(argv).is_err()
}

fn grants(args: &[&str]) -> grants::Command {
    match parse(&[&["grants"], args].concat()) {
        Command::Grants { command } => command,
        other => panic!("{other:?}"),
    }
}

fn tier_list(args: &[&str]) -> tier_list::Command {
    match parse(&[&["tier-list"], args].concat()) {
        Command::TierList { command } => command,
        other => panic!("{other:?}"),
    }
}

fn stats(args: &[&str]) -> stats::Args {
    match parse(&[&["stats"], args].concat()) {
        Command::Stats(args) => args,
        other => panic!("{other:?}"),
    }
}

fn regrade(args: &[&str]) -> regrade::Args {
    match parse(&[&["regrade"], args].concat()) {
        Command::Regrade(args) => args,
        other => panic!("{other:?}"),
    }
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

fn task_names(tasks: &[&stats::Task]) -> Vec<&'static str> {
    tasks.iter().map(|t| t.name).collect()
}

#[test]
fn grants_users_takes_an_optional_search() {
    assert!(matches!(
        grants(&["users"]),
        grants::Command::Users { search: None }
    ));
    assert!(matches!(
        grants(&["users", "amiya"]),
        grants::Command::Users { search: Some(s) } if s == "amiya"
    ));
}

#[test]
fn grants_role_takes_a_known_role_and_a_server() {
    assert!(matches!(
        grants(&["role", "12345", "tier_list_admin", "--server", "jp"]),
        grants::Command::Role { user, role: RoleArg::TierListAdmin, server: Some(s) }
            if user == "12345" && s == "jp"
    ));
    assert!(matches!(
        grants(&["role", "12345", "user"]),
        grants::Command::Role {
            role: RoleArg::User,
            server: None,
            ..
        }
    ));
    assert!(rejects(&["grants", "role", "12345", "admin"]));
    assert!(rejects(&["grants", "role", "12345", "tier-list-admin"]));
    assert!(rejects(&["grants", "role", "12345"]));
}

#[test]
fn grants_add_list_and_remove_take_a_slug_user_and_level() {
    assert!(matches!(
        grants(&["list", "meta"]),
        grants::Command::List { slug } if slug == "meta"
    ));
    assert!(matches!(
        grants(&["add", "meta", "12345", "edit"]),
        grants::Command::Add { slug, user, permission: Level::Edit, server: None }
            if slug == "meta" && user == "12345"
    ));
    assert!(matches!(
        grants(&["remove", "meta", "12345", "all", "--server", "en"]),
        grants::Command::Remove { permission: Level::All, server: Some(s), .. } if s == "en"
    ));
    assert!(rejects(&["grants", "add", "meta", "12345", "owner"]));
    assert!(rejects(&["grants", "add", "meta", "12345"]));
}

#[test]
fn tier_list_subcommands_parse() {
    assert!(matches!(tier_list(&["list"]), tier_list::Command::List));
    assert!(matches!(
        tier_list(&["visibility", "meta", "off"]),
        tier_list::Command::Visibility { slug, listed: Toggle::Off } if slug == "meta"
    ));
    assert!(rejects(&["tier-list", "visibility", "meta", "maybe"]));
}

#[test]
fn set_type_is_a_preview_without_yes() {
    assert!(matches!(
        tier_list(&["set-type", "meta", "official"]),
        tier_list::Command::SetType {
            list_type: ListType::Official,
            yes: false,
            ..
        }
    ));
    for yes in ["--yes", "-y"] {
        assert!(matches!(
            tier_list(&["set-type", "meta", "community", yes]),
            tier_list::Command::SetType {
                list_type: ListType::Community,
                yes: true,
                ..
            }
        ));
    }
    assert!(rejects(&["tier-list", "set-type", "meta", "featured"]));
    assert!(rejects(&["tier-list", "set-type", "meta", "--official"]));
}

#[test]
fn stats_flags_parse() {
    let bare = stats(&[]);
    assert!(!bare.list && !bare.all && !bare.include_heavy);
    assert_eq!(bare.concurrency, 1);
    assert!(bare.names.is_empty());

    let all = stats(&["-a", "--include-heavy", "-j", "3"]);
    assert!(all.all && all.include_heavy);
    assert_eq!(all.concurrency, 3);
    assert_eq!(
        stats(&["trending", "regrade"]).names,
        names(&["trending", "regrade"])
    );
    assert!(stats(&["-l"]).list);
    assert!(rejects(&["stats", "--include-heavy", "trending"]));
    assert!(rejects(&["stats", "-j", "0", "--all"]));
}

#[test]
fn all_runs_the_cheap_tasks_and_heavy_only_on_request() {
    assert_eq!(
        task_names(&select(true, false, &[]).unwrap()),
        [
            "operator-ownership",
            "medal-ownership",
            "trending",
            "leaderboard-snapshot"
        ]
    );
    assert_eq!(select(true, true, &[]).unwrap().len(), TASKS.len());
}

#[test]
fn names_run_in_the_order_given_and_must_exist() {
    assert_eq!(
        task_names(&select(false, false, &names(&["regrade", "trending"])).unwrap()),
        ["regrade", "trending"]
    );
    assert!(select(true, false, &names(&["trending"])).is_err());
    assert!(select(false, false, &[]).is_err());
    let unknown = select(false, false, &names(&["trending", "trendy"])).unwrap_err();
    assert!(unknown.to_string().contains("trendy"));
}

#[test]
fn every_task_name_is_unique() {
    for (i, task) in TASKS.iter().enumerate() {
        assert!(
            TASKS[i + 1..].iter().all(|t| t.name != task.name),
            "{}",
            task.name
        );
    }
}

#[test]
fn tasks_load_the_least_game_data_that_serves_them() {
    let pick = |list: &[&str]| game_data(&select(false, false, &names(list)).unwrap());
    assert_eq!(
        game_data(&select(true, false, &[]).unwrap()),
        GameData::None
    );
    assert_eq!(pick(&["trending", "regrade"]), GameData::Default);
    assert_eq!(pick(&["regrade", "event-shops"]), GameData::All);
    assert_eq!(pick(&["gacha-details"]), GameData::All);
    assert_eq!(game_data(&[]), GameData::None);
}

#[test]
fn regrade_defaults_come_from_the_config() {
    let args = regrade(&[]);
    assert_eq!((args.concurrency, args.page_size), (None, None));
    assert_eq!(args.progress_every, 50);
    assert!(!args.dry_run && !args.fail_fast && !args.only_sandbox && !args.enqueue);
    assert!(args.uid.is_none() && args.server.is_none());
}

#[test]
fn regrade_flags_parse() {
    let args = regrade(&[
        "-j",
        "4",
        "--page-size",
        "100",
        "--uid",
        "12345",
        "--server",
        "en",
        "--fail-fast",
        "--only-sandbox",
        "--dry-run",
        "--progress-every",
        "0",
    ]);
    assert_eq!((args.concurrency, args.page_size), (Some(4), Some(100)));
    assert_eq!(args.uid.as_deref(), Some("12345"));
    assert_eq!(args.server.as_deref(), Some("en"));
    assert!(args.fail_fast && args.only_sandbox && args.dry_run);
    assert_eq!(args.progress_every, 0);

    let queued = regrade(&["--enqueue", "--uid", "12345", "--server", "en"]);
    assert!(queued.enqueue);
}

#[test]
fn regrade_rejects_flags_that_do_nothing() {
    assert!(rejects(&["regrade", "-j", "0"]));
    assert!(rejects(&["regrade", "--page-size", "0"]));
    for flag in ["--dry-run", "--fail-fast", "--only-sandbox"] {
        assert!(rejects(&["regrade", "--enqueue", flag]), "{flag}");
    }
    assert!(rejects(&["regrade", "--enqueue", "-j", "2"]));
    assert!(rejects(&["regrade", "--enqueue", "--progress-every", "5"]));
}
