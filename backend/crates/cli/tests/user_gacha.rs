#![allow(clippy::unwrap_used)]

use std::path::Path;

use clap::Parser;
use clap::error::ErrorKind;
use myrtle_cli::commands::{gacha, user};
use myrtle_cli::{Cli, Command};
use myrtle_db::repos::gacha as repo;
use myrtle_db::repos::users;
use myrtle_testkit::db::database;
use myrtle_types::{Server, UserId};
use serde_json::json;

const ID: &str = "8e3b6f1d-3c1e-4c1b-9a43-1a2b3c4d5ec1";

fn parse(args: &[&str]) -> Result<Command, clap::Error> {
    Cli::try_parse_from(std::iter::once("myrtle").chain(args.iter().copied())).map(|c| c.command)
}

fn user_export(args: &[&str]) -> user::ExportArgs {
    let all = [&["user", "export"], args].concat();
    match parse(&all).unwrap() {
        Command::User {
            command: user::Command::Export(args),
        } => args,
        other => panic!("{other:?}"),
    }
}

#[test]
fn user_export_takes_a_uid_or_a_user_id() {
    let args = user_export(&["--uid", "123456789", "--server", "cn", "-o", "a.json"]);
    assert_eq!(args.who.uid.as_deref(), Some("123456789"));
    assert_eq!(args.who.server.as_deref(), Some("cn"));
    assert_eq!(args.output.as_deref(), Some(Path::new("a.json")));
    assert!(!args.include_credentials);

    let args = user_export(&["--user-id", ID, "--include-credentials"]);
    assert_eq!(args.who.user_id, Some(UserId(ID.parse().unwrap())));
    assert_eq!(args.who.uid, None);
    assert!(args.include_credentials);
}

#[test]
fn uid_and_user_id_are_mutually_exclusive_and_one_is_required() {
    for command in [["user", "export"], ["gacha", "export"]] {
        let both = [&command[..], &["--uid", "1", "--user-id", ID]].concat();
        assert_eq!(
            parse(&both).unwrap_err().kind(),
            ErrorKind::ArgumentConflict
        );
        assert_eq!(
            parse(&command).unwrap_err().kind(),
            ErrorKind::MissingRequiredArgument
        );
        let server_only = [&command[..], &["--server", "en"]].concat();
        assert_eq!(
            parse(&server_only).unwrap_err().kind(),
            ErrorKind::MissingRequiredArgument
        );
    }
}

#[test]
fn user_id_must_be_a_uuid() {
    let err = parse(&["user", "export", "--user-id", "123"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::ValueValidation);
    assert!(user::parse_user_id(ID).is_ok());
}

#[test]
fn user_import_flags() {
    let Command::User {
        command: user::Command::Import(args),
    } = parse(&[
        "user",
        "import",
        "-f",
        "a.json",
        "--replace",
        "--dry-run",
        "--no-triggers",
    ])
    .unwrap()
    else {
        panic!("not an import");
    };
    assert_eq!(args.file, Path::new("a.json"));
    assert!(args.replace && args.dry_run && args.no_triggers);

    let Command::User {
        command: user::Command::Import(args),
    } = parse(&["user", "import", "--file", "a.json"]).unwrap()
    else {
        panic!("not an import");
    };
    assert!(!args.replace && !args.dry_run && !args.no_triggers);
    assert_eq!(
        parse(&["user", "import"]).unwrap_err().kind(),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn gacha_flags() {
    let Command::Gacha {
        command: gacha::Command::Export(args),
    } = parse(&["gacha", "export", "--uid", "5", "--output", "g.json"]).unwrap()
    else {
        panic!("not an export");
    };
    assert_eq!(args.who.uid.as_deref(), Some("5"));
    assert_eq!(args.output.as_deref(), Some(Path::new("g.json")));

    for (argv, dry_run) in [
        (&["gacha", "resync"][..], false),
        (&["gacha", "resync", "--dry-run"], true),
    ] {
        let Command::Gacha {
            command: gacha::Command::Resync(args),
        } = parse(argv).unwrap()
        else {
            panic!("not a resync");
        };
        assert_eq!(args.dry_run, dry_run);
    }
}

#[tokio::test]
async fn resolve_refuses_an_ambiguous_uid_and_honours_server() {
    let test = database().await;
    let en = test.user(42, Server::EN).await;
    let cn = test.user(42, Server::CN).await;
    let who = |uid: Option<&str>, user_id: Option<UserId>, server: Option<&str>| user::Who {
        uid: uid.map(str::to_owned),
        user_id,
        server: server.map(str::to_owned),
    };

    let err = who(Some("42"), None, None)
        .resolve(&test.db)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("--server"), "{err}");
    let account = who(Some("42"), None, Some("CN"))
        .resolve(&test.db)
        .await
        .unwrap();
    assert_eq!(UserId(account.id), cn);
    let account = who(None, Some(en), None).resolve(&test.db).await.unwrap();
    assert_eq!(account.server, "en");
    let err = who(None, Some(en), Some("cn"))
        .resolve(&test.db)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not cn"), "{err}");
    assert!(
        who(Some("42"), None, Some("xx"))
            .resolve(&test.db)
            .await
            .is_err()
    );
    test.close().await;
}

#[tokio::test]
async fn gacha_export_keeps_mains_shape_and_order() {
    let test = database().await;
    let user = test.user(43, Server::EN).await;
    let pulls = json!([
        {"char_id": "char_old", "pool_id": "NORM_1", "rarity": 4, "pull_timestamp": 100,
         "pool_name": "Banner", "gacha_type": "normal", "batch_index": 0},
        {"char_id": "char_new", "pool_id": "NORM_1", "rarity": 6, "pull_timestamp": 200,
         "pool_name": null, "gacha_type": "normal", "batch_index": 0},
    ]);
    let mut conn = test.db.conn().await.unwrap();
    let stored = repo::insert_batch(&mut conn, user, &pulls).await.unwrap();
    assert_eq!(stored, repo::BatchOutcome::Stored);
    let account = users::account(&mut conn, user).await.unwrap().unwrap();
    drop(conn);

    let document = gacha::export_document(&test.db, &account).await.unwrap();
    let value = serde_json::to_value(&document).unwrap();
    assert_eq!(
        value["user"],
        json!({"id": user.0, "uid": "43", "server": "en", "nickname": null})
    );
    assert_eq!(value["record_count"], 2);
    let records = value["records"].as_array().unwrap();
    assert_eq!(records[0]["char_id"], "char_new");
    assert_eq!(records[1]["pool_name"], "Banner");
    let mut keys: Vec<&str> = records[0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "batch_index",
            "char_id",
            "created_at",
            "gacha_type",
            "id",
            "pool_id",
            "pool_name",
            "pull_timestamp",
            "rarity",
            "user_id"
        ]
    );
    assert!(value["exported_at"].is_string());
    test.close().await;
}
