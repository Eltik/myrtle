#![allow(clippy::unwrap_used)]

use clap::Parser;
use myrtle_cli::commands::{gamedata, hypergryph, jobs, session};
use myrtle_cli::{Cli, Command};
use myrtle_hypergryph::DeviceIds;
use myrtle_hypergryph::login::bilibili::{OTP_TTL_SECS, OtpChallenge};
use myrtle_jobs::{JobKind, JobState};
use serde::Deserialize;
use serde_json::json;

fn parse(args: &[&str]) -> Command {
    let argv = std::iter::once("myrtle").chain(args.iter().copied());
    Cli::try_parse_from(argv).unwrap().command
}

fn rejects(args: &[&str]) -> String {
    let argv = std::iter::once("myrtle").chain(args.iter().copied());
    Cli::try_parse_from(argv).unwrap_err().to_string()
}

fn session(args: &[&str]) -> session::Command {
    let mut argv = vec!["session"];
    argv.extend_from_slice(args);
    match parse(&argv) {
        Command::Session { command } => command,
        other => panic!("{other:?}"),
    }
}

fn challenge(phone: &str, issued_at: u64) -> OtpChallenge {
    serde_json::from_value(json!({
        "phone": phone,
        "country_code": "86",
        "captcha_key": "key",
        "device_id": "dev",
        "bd_id": "bd",
        "issued_at": issued_at,
    }))
    .unwrap()
}

#[test]
fn gamedata_takes_an_optional_server() {
    match parse(&["gamedata", "compile", "--server", "jp"]) {
        Command::Gamedata {
            command: gamedata::Command::Compile { server },
        } => assert_eq!(server.as_deref(), Some("jp")),
        other => panic!("{other:?}"),
    }
    match parse(&["gamedata", "publish"]) {
        Command::Gamedata {
            command: gamedata::Command::Publish { server: None },
        } => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn jobs_list_parses_filters_and_defaults_the_limit() {
    match parse(&["jobs", "list"]) {
        Command::Jobs {
            command:
                jobs::Command::List {
                    state: None,
                    kind: None,
                    limit: 20,
                },
        } => {}
        other => panic!("{other:?}"),
    }
    match parse(&[
        "jobs",
        "list",
        "--state",
        "dead",
        "--kind",
        "regrade_all",
        "--limit",
        "5",
    ]) {
        Command::Jobs {
            command: jobs::Command::List { state, kind, limit },
        } => {
            assert_eq!(state, Some(JobState::Dead));
            assert_eq!(kind, Some(JobKind::RegradeAll));
            assert_eq!(limit, 5);
        }
        other => panic!("{other:?}"),
    }
    assert!(rejects(&["jobs", "list", "--state", "done"]).contains("queued, running"));
}

#[test]
fn jobs_enqueue_parses_kind_and_object_payload() {
    match parse(&[
        "jobs",
        "enqueue",
        "regrade_user",
        "--payload",
        r#"{"uid":"1"}"#,
    ]) {
        Command::Jobs {
            command: jobs::Command::Enqueue { kind, payload },
        } => {
            assert_eq!(kind, JobKind::RegradeUser);
            assert_eq!(payload, Some(json!({ "uid": "1" })));
        }
        other => panic!("{other:?}"),
    }
    match parse(&["jobs", "enqueue", "siren_sync"]) {
        Command::Jobs {
            command: jobs::Command::Enqueue { payload: None, .. },
        } => {}
        other => panic!("{other:?}"),
    }
    assert!(rejects(&["jobs", "enqueue", "siren_sync", "--payload", "[1]"]).contains("object"));
    assert!(rejects(&["jobs", "enqueue", "siren_sync", "--payload", "{"]).contains("JSON"));
}

#[test]
fn an_unknown_job_kind_lists_every_valid_name() {
    let error = rejects(&["jobs", "enqueue", "regrade"]);
    for kind in JobKind::ALL {
        assert!(error.contains(kind.as_str()), "{error}");
    }
    assert!(jobs::parse_kind("trending_recompute").is_ok());
}

#[test]
fn hypergryph_device_ids_parses() {
    assert!(matches!(
        parse(&["hypergryph", "device-ids"]),
        Command::Hypergryph {
            command: hypergryph::Command::DeviceIds
        }
    ));
}

#[test]
fn the_device_id_fragment_parses_back_into_three_values() {
    #[derive(Deserialize)]
    struct Fragment {
        hypergryph: DeviceIds,
    }
    let ids = DeviceIds::generate();
    let parsed: Fragment = toml::from_str(&hypergryph::fragment(&ids)).unwrap();
    assert_eq!(parsed.hypergryph, ids);
    assert!(parsed.hypergryph.is_complete());
    assert!(parsed.hypergryph.device_id2.starts_with("86"));
}

#[test]
fn session_subcommands_keep_their_flags() {
    match session(&["send-code", "--email", "a@b.c", "--server", "jp"]) {
        session::Command::SendCode { email, server } => {
            assert_eq!(email, "a@b.c");
            assert_eq!(server.as_deref(), Some("jp"));
        }
        other => panic!("{other:?}"),
    }
    match session(&["login", "--email", "a@b.c", "--code", "123456"]) {
        session::Command::Login {
            code, server: None, ..
        } => assert_eq!(code, "123456"),
        other => panic!("{other:?}"),
    }
    match session(&["bili", "--user", "u", "--pass", "p"]) {
        session::Command::Bili { user, pass } => assert_eq!((user, pass), ("u".into(), "p".into())),
        other => panic!("{other:?}"),
    }
    match session(&["import", "s.json", "--server", "bili"]) {
        session::Command::Import { path, server } => {
            assert_eq!(path.to_str(), Some("s.json"));
            assert_eq!(server.as_deref(), Some("bili"));
        }
        other => panic!("{other:?}"),
    }
    match session(&["import-dir", "dir", "--force"]) {
        session::Command::ImportDir { force: true, .. } => {}
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        session(&["status"]),
        session::Command::Status { server: None }
    ));
    assert!(rejects(&["session", "login", "--email", "a@b.c"]).contains("--code"));
    assert!(rejects(&["session", "bili", "--user", "u"]).contains("--pass"));
}

#[test]
fn session_sms_sends_then_logs_in() {
    match session(&["sms", "--phone", "138", "--country-code", "1"]) {
        session::Command::Sms {
            phone,
            country_code,
            otp: None,
        } => {
            assert_eq!(phone, "138");
            assert_eq!(country_code.as_deref(), Some("1"));
        }
        other => panic!("{other:?}"),
    }
    match session(&["sms", "--phone", "138", "--otp", "9999"]) {
        session::Command::Sms { otp, .. } => assert_eq!(otp.as_deref(), Some("9999")),
        other => panic!("{other:?}"),
    }
    let both = [
        "session",
        "sms",
        "--phone",
        "1",
        "--otp",
        "2",
        "--country-code",
        "1",
    ];
    assert!(rejects(&both).contains("cannot be used with"));
}

#[test]
fn only_game_server_logins_need_pinned_device_ids() {
    let needs = |args: &[&str]| session::needs_device_ids(&session(args));
    assert!(needs(&["login", "--email", "e", "--code", "c"]));
    assert!(needs(&["bili", "--user", "u", "--pass", "p"]));
    assert!(needs(&["sms", "--phone", "1", "--otp", "2"]));
    assert!(needs(&["import", "s.json"]));
    assert!(needs(&["status"]));
    assert!(!needs(&["send-code", "--email", "e"]));
    assert!(!needs(&["sms", "--phone", "1"]));
    assert!(!needs(&["import-dir", "d"]));

    let error = session::require_device_ids(&DeviceIds {
        device_id2: String::new(),
        ..DeviceIds::generate()
    })
    .unwrap_err();
    assert!(error.to_string().contains("myrtle hypergryph device-ids"));
    assert!(session::require_device_ids(&DeviceIds::generate()).is_ok());
}

#[test]
fn the_pending_otp_file_round_trips_owner_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(session::PENDING_OTP);
    let sent = challenge("13800000000", 1_000);
    session::write_pending(&path, &sent).unwrap();
    assert!(session::read_pending(&path).unwrap() == sent);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    std::fs::write(&path, "{").unwrap();
    let Err(corrupt) = session::read_pending(&path) else {
        panic!("a corrupt file read back");
    };
    assert!(format!("{corrupt:#}").contains("corrupt"));
    let Err(missing) = session::read_pending(&dir.path().join("absent.json")) else {
        panic!("a missing file read back");
    };
    assert!(missing.to_string().contains("without --otp"));
}

#[test]
fn rewriting_a_wide_open_pending_file_makes_it_owner_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(session::PENDING_OTP);
    std::fs::write(&path, "old").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        session::write_pending(&path, &challenge("1", 0)).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    assert!(session::read_pending(&path).unwrap() == challenge("1", 0));
}

#[test]
fn a_pending_code_must_match_the_phone_and_be_fresh() {
    let sent = challenge("13800000000", 1_000);
    assert!(session::check_pending(&sent, "13800000000", 1_000 + OTP_TTL_SECS).is_ok());

    let other = session::check_pending(&sent, "13900000000", 1_000).unwrap_err();
    assert!(other.to_string().contains("sent to 13800000000"));

    let late = session::check_pending(&sent, "13800000000", 1_001 + OTP_TTL_SECS).unwrap_err();
    assert!(late.to_string().contains("expired"));
}
