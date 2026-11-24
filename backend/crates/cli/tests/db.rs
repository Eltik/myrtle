#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use clap::Parser;
use myrtle_cli::commands::db::Command as Db;
use myrtle_cli::{Cli, Command};

fn db(args: &[&str]) -> Result<Db, clap::Error> {
    let cli = Cli::try_parse_from(["myrtle", "db"].iter().chain(args))?;
    match cli.command {
        Command::Db { command } => Ok(command),
        other => panic!("parsed as {other:?}"),
    }
}

#[test]
fn export_takes_every_table_by_default() {
    let Db::Export {
        out,
        tables,
        with_parents,
    } = db(&["export", "--out", "/tmp/dump"]).unwrap()
    else {
        panic!("not export");
    };
    assert_eq!(out, PathBuf::from("/tmp/dump"));
    assert_eq!(tables, None);
    assert!(!with_parents);
}

#[test]
fn export_splits_a_table_list_on_commas() {
    let Db::Export {
        out,
        tables,
        with_parents,
    } = db(&[
        "export",
        "-o",
        "d",
        "--tables",
        "users,gacha_records",
        "--with-parents",
    ])
    .unwrap()
    else {
        panic!("not export");
    };
    assert_eq!(out, PathBuf::from("d"));
    assert_eq!(
        tables.unwrap(),
        ["users".to_owned(), "gacha_records".to_owned()]
    );
    assert!(with_parents);
    let Db::Export { tables, .. } = db(&["export", "-o", "d", "-t", "users"]).unwrap() else {
        panic!("not export");
    };
    assert_eq!(tables.unwrap(), ["users".to_owned()]);
}

#[test]
fn export_needs_an_out_dir_and_with_parents_needs_tables() {
    assert!(db(&["export"]).is_err());
    assert!(db(&["export", "--out", "d", "--with-parents"]).is_err());
    assert!(db(&["export", "--out", "d", "--all"]).is_err());
}

#[test]
fn import_defaults_to_a_thousand_rows_per_batch() {
    let Db::Import {
        input,
        truncate,
        cascade,
        migrate,
        batch_size,
    } = db(&["import", "--in", "d"]).unwrap()
    else {
        panic!("not import");
    };
    assert_eq!(input, PathBuf::from("d"));
    assert!(!truncate && !cascade && !migrate);
    assert_eq!(batch_size.get(), 1000);
}

#[test]
fn import_takes_its_flags() {
    let Db::Import {
        input,
        truncate,
        cascade,
        migrate,
        batch_size,
    } = db(&[
        "import",
        "-i",
        "d",
        "--truncate",
        "--cascade",
        "--migrate",
        "--batch-size",
        "50",
    ])
    .unwrap()
    else {
        panic!("not import");
    };
    assert_eq!(input, PathBuf::from("d"));
    assert!(truncate && cascade && migrate);
    assert_eq!(batch_size.get(), 50);
}

#[test]
fn cascade_needs_truncate() {
    assert!(db(&["import", "--in", "d", "--cascade"]).is_err());
}

#[test]
fn import_refuses_a_zero_or_missing_batch_size_and_a_missing_dir() {
    assert!(db(&["import", "--in", "d", "--batch-size", "0"]).is_err());
    assert!(db(&["import", "--in", "d", "--batch-size", "-1"]).is_err());
    assert!(db(&["import", "--in", "d", "--batch-size"]).is_err());
    assert!(db(&["import"]).is_err());
}

#[test]
fn translate_legacy_needs_both_dirs() {
    let Db::TranslateLegacy { input, out } =
        db(&["translate-legacy", "--in", "backup", "--out", "bundle"]).unwrap()
    else {
        panic!("not translate-legacy");
    };
    assert_eq!(
        (input, out),
        (PathBuf::from("backup"), PathBuf::from("bundle"))
    );
    assert!(db(&["translate-legacy", "--in", "backup"]).is_err());
    assert!(db(&["translate-legacy", "--out", "bundle"]).is_err());
}
