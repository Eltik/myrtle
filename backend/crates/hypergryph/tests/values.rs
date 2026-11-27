//! Signing, parsing, serde shapes, error classification and redaction, no network.

mod config {
    mod device_ids {
        use myrtle_hypergryph::DeviceIds;

        #[test]
        fn generated_ids_have_the_client_shapes() {
            let ids = DeviceIds::generate();
            assert!(ids.is_complete());
            for uuid in [&ids.device_id, &ids.device_id3] {
                assert_eq!(uuid.len(), 32);
                assert!(
                    uuid.bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                );
                assert_eq!(&uuid[12..13], "4", "version nibble");
                assert!(
                    matches!(&uuid[16..17], "8" | "9" | "a" | "b"),
                    "variant bits"
                );
            }
            assert_eq!(ids.device_id2.len(), 15);
            assert!(ids.device_id2.starts_with("86"));
            assert!(ids.device_id2.bytes().all(|b| b.is_ascii_digit()));
            assert_ne!(ids.device_id, ids.device_id3);
        }

        #[test]
        fn a_missing_id_is_incomplete() {
            let mut ids = DeviceIds::generate();
            ids.device_id2.clear();
            assert!(!ids.is_complete());
            assert!(!DeviceIds::default().is_complete());
        }

        #[test]
        fn serde_names_are_the_persisted_ones() {
            let ids: DeviceIds =
                serde_json::from_str(r#"{"device_id":"a","device_id2":"b","device_id3":"c"}"#)
                    .unwrap();
            assert_eq!(
                (
                    ids.device_id.as_str(),
                    ids.device_id2.as_str(),
                    ids.device_id3.as_str()
                ),
                ("a", "b", "c")
            );
        }
    }

    mod network {
        use myrtle_hypergryph::config::network::parse_network;
        use myrtle_hypergryph::{DeviceIds, Domain, NetworkConfig, VersionInfo};
        use myrtle_types::Server;

        #[test]
        fn domains_and_versions_are_per_server() {
            let mut cfg = NetworkConfig::new(DeviceIds::default());
            cfg.set_domain(Server::EN, Domain::GS, "https://gs.en");
            cfg.set_domain(Server::Bilibili, Domain::HV, "https://hv.b/{0}/version");
            cfg.set_version(
                Server::JP,
                VersionInfo {
                    res_version: "r".into(),
                    client_version: "c".into(),
                },
            );

            assert_eq!(cfg.domain(Server::EN, Domain::GS), Some("https://gs.en"));
            assert_eq!(cfg.domain(Server::JP, Domain::GS), None);
            assert_eq!(cfg.domain(Server::EN, Domain::U8), None);
            assert_eq!(
                cfg.version_url(Server::Bilibili).as_deref(),
                Some("https://hv.b/Android/version")
            );
            assert_eq!(cfg.version(Server::JP).client_version, "c");
            assert!(cfg.version(Server::EN).res_version.is_empty());
        }

        #[test]
        fn each_server_reads_its_own_host_slot() {
            let cfg = NetworkConfig::new(DeviceIds::default());
            assert!(cfg.network_route(Server::KR).contains(".kr/"));
            assert!(
                cfg.network_route(Server::Bilibili)
                    .ends_with("/b/network_config")
            );
            assert!(cfg.network_route(Server::TW).contains("-tw."));
            assert!(
                cfg.yostar_sdk(Server::EN)
                    .is_some_and(|h| h.contains("en-sdk"))
            );
            assert_eq!(cfg.yostar_sdk(Server::CN), None);
        }

        #[test]
        fn network_reads_the_func_ver_routes() {
            let content = serde_json::json!({
                "funcVer": "V2",
                "configs": {
                    "V1": { "network": { "gs": "https://old" } },
                    "V2": { "network": { "gs": "https://gs", "HV": "https://hv/{0}", "u8": "https://u8", "pkgad": "https://ad", "PKGIOS": "https://ios", "sl": 3 } },
                },
            });
            let body = serde_json::json!({ "content": content.to_string() }).to_string();
            let mut routes = parse_network(body.as_bytes()).unwrap();
            routes.sort_by_key(|(d, _)| d.index());
            assert_eq!(
                routes,
                [
                    (Domain::GS, "https://gs".to_owned()),
                    (Domain::U8, "https://u8".to_owned()),
                    (Domain::HV, "https://hv/{0}".to_owned()),
                    (Domain::PkgAd, "https://ad".to_owned()),
                    (Domain::PkgIos, "https://ios".to_owned()),
                ]
            );
        }

        #[test]
        fn a_bom_or_invalid_utf8_body_is_a_decode_error() {
            let content = serde_json::json!({ "funcVer": "V1", "configs": { "V1": { "network": { "gs": "https://gs" } } } });
            let body = serde_json::json!({ "content": content.to_string() }).to_string();
            assert_eq!(parse_network(body.as_bytes()).unwrap().len(), 1);
            let mut bom = b"\xEF\xBB\xBF".to_vec();
            bom.extend_from_slice(body.as_bytes());
            assert!(parse_network(&bom).is_err());
            let mut bytes = body.replacen("https", "#", 1).into_bytes();
            let at = bytes.iter().position(|b| *b == b'#').unwrap();
            bytes[at] = 0xFF;
            assert!(parse_network(&bytes).is_err());
        }

        #[test]
        fn version_info_is_camel_case() {
            let v: VersionInfo =
                serde_json::from_str(r#"{"resVersion":"24-10","clientVersion":"2.4.01"}"#).unwrap();
            assert_eq!(
                (v.res_version.as_str(), v.client_version.as_str()),
                ("24-10", "2.4.01")
            );
        }
    }
}

mod constants {
    use myrtle_hypergryph::constants::yostar_domain;
    use myrtle_hypergryph::{Domain, Pid};
    use myrtle_types::Server;

    #[test]
    fn domain_index_matches_all_order() {
        for (i, d) in Domain::ALL.into_iter().enumerate() {
            assert_eq!(d.index(), i);
        }
    }

    #[test]
    fn network_config_keys_parse_case_insensitively() {
        assert_eq!(Domain::parse("gs"), Some(Domain::GS));
        assert_eq!(Domain::parse("HV"), Some(Domain::HV));
        assert_eq!(Domain::parse("prean"), Some(Domain::PREAN));
        assert_eq!(Domain::parse("nope"), None);
        assert_eq!(Domain::parse("pkgAd"), Some(Domain::PkgAd));
        assert_eq!(Domain::parse("pkgios"), Some(Domain::PkgIos));
        for d in Domain::ALL {
            let json = serde_json::to_value(d).unwrap();
            assert_eq!(json, d.key(), "key matches the serde name");
        }
    }

    #[test]
    fn yostar_publishes_en_jp_kr_only() {
        let with_sdk: Vec<_> = Server::ALL
            .into_iter()
            .filter(|s| yostar_domain(*s).is_some())
            .collect();
        assert_eq!(with_sdk, [Server::EN, Server::JP, Server::KR]);
        assert_eq!(yostar_domain(Server::KR), yostar_domain(Server::JP));
    }

    #[test]
    fn pid_follows_the_server() {
        assert_eq!(Pid::from(Server::JP), Pid::JpAk);
        assert_eq!(Pid::from(Server::KR), Pid::KrArknights);
        assert_eq!(Pid::from(Server::EN), Pid::UsArknights);
        assert_eq!(Pid::from(Server::CN), Pid::UsArknights);
    }
}

mod crypto {
    use myrtle_hypergryph::crypto::{auth_header_at, generate_bilibili_sign, generate_u8_sign};
    use myrtle_types::Server;

    #[test]
    fn u8_sign_sorts_and_percent_encodes() {
        let extension = r#"{"type":1,"uid":"42","token":"t k&="}"#;
        let sign = generate_u8_sign(&[
            ("appId", "1"),
            ("platform", "1"),
            ("extension", extension),
            ("deviceId", "dev"),
        ]);
        assert_eq!(sign, "d1559454bc80061fb2af93e56854a27ae6e22368");
    }

    #[test]
    fn u8_sign_ignores_input_order() {
        let a = generate_u8_sign(&[("b", "2"), ("a", "1")]);
        let b = generate_u8_sign(&[("a", "1"), ("b", "2")]);
        assert_eq!(a, b);
    }

    #[test]
    fn bilibili_sign_concatenates_values_by_key() {
        let sign = generate_bilibili_sign(&[
            ("merchant_id", "328"),
            ("game_id", "952"),
            ("timestamp", "1700000000"),
            ("user_id", "a@b"),
        ]);
        assert_eq!(sign, "c4e1798f919bfc168d7a228cca4a07a4");
    }

    #[test]
    fn auth_header_signs_head_body_and_secret() {
        let header = auth_header_at(
            r#"{"a":1}"#,
            Server::EN,
            Some("uid1"),
            Some("tok1"),
            "dev",
            1_700_000_000,
        );
        let expected_head = r#"{"PID":"US-ARKNIGHTS","Channel":"googleplay","Platform":"android","Version":"4.10.0","GVersionNo":"2000112","GBuildNo":"","Lang":"en","DeviceID":"dev","DeviceModel":"F9","UID":"uid1","Token":"tok1","Time":1700000000}"#;
        assert_eq!(
            header,
            format!(r#"{{"Head":{expected_head},"Sign":"9F177D5CA00BC7B0EB8EC1083CC372A1"}}"#)
        );
    }

    #[test]
    fn auth_header_lang_and_pid_follow_the_server() {
        let head = |server| {
            let v: serde_json::Value =
                serde_json::from_str(&auth_header_at("", server, None, None, "d", 0)).unwrap();
            (
                v["Head"]["Lang"].as_str().unwrap().to_owned(),
                v["Head"]["PID"].as_str().unwrap().to_owned(),
                v["Head"]["UID"].as_str().unwrap().to_owned(),
            )
        };
        assert_eq!(
            head(Server::JP),
            ("jp".into(), "JP-AK".into(), String::new())
        );
        assert_eq!(
            head(Server::KR),
            ("ko".into(), "KR-ARKNIGHTS".into(), String::new())
        );
        assert_eq!(
            head(Server::CN),
            ("ko".into(), "US-ARKNIGHTS".into(), String::new())
        );
    }
}

mod error {
    use myrtle_hypergryph::{UpstreamEnvelope, UpstreamError, envelope_code};

    fn envelope(field: &str, text: &str) -> UpstreamEnvelope {
        let mut e = UpstreamEnvelope {
            status_code: 503,
            ..UpstreamEnvelope::default()
        };
        match field {
            "error" => e.error = text.to_owned(),
            "message" => e.message = text.to_owned(),
            "msg" => e.msg = text.to_owned(),
            _ => e.info = text.to_owned(),
        }
        e
    }

    #[test]
    fn maintenance_wording_in_any_field_classifies() {
        for field in ["error", "message", "msg", "info"] {
            for text in [
                "Server under Maintenance",
                "服务器维护中",
                "ただいまメンテナンス中です",
            ] {
                let err = UpstreamError::from_envelope(envelope(field, text));
                assert!(
                    matches!(err, UpstreamError::Maintenance(_)),
                    "{field}: {text}"
                );
            }
        }
    }

    #[test]
    fn other_envelopes_stay_envelopes() {
        let err = UpstreamError::from_envelope(envelope("msg", "invalid shop id"));
        assert!(matches!(err, UpstreamError::Envelope(_)));
        assert!(matches!(
            UpstreamError::from_envelope(UpstreamEnvelope::default()),
            UpstreamError::Envelope(_)
        ));
    }

    #[test]
    fn envelope_code_matches_envelopes_of_any_wording() {
        let shop = UpstreamError::from_envelope(UpstreamEnvelope {
            code: envelope_code::INVALID_SHOP_ID,
            ..UpstreamEnvelope::default()
        });
        assert!(shop.is_envelope_code(envelope_code::INVALID_SHOP_ID));
        assert!(!shop.is_envelope_code(envelope_code::STAGE_NO_REPLAY));
        assert!(!UpstreamError::NotLoggedIn.is_envelope_code(0));

        let mut down = envelope("msg", "maintenance");
        down.code = envelope_code::INVALID_SHOP_ID;
        let down = UpstreamError::from_envelope(down);
        assert!(matches!(down, UpstreamError::Maintenance(_)));
        assert!(down.is_envelope_code(envelope_code::INVALID_SHOP_ID));
        assert!(!down.is_envelope_code(envelope_code::STAGE_NO_REPLAY));
    }

    #[test]
    fn display_falls_back_to_message() {
        let e: UpstreamEnvelope = serde_json::from_str(
            r#"{"statusCode":503,"error":"Service Unavailable","message":"Server under Maintenance"}"#,
        )
        .unwrap();
        assert_eq!(
            UpstreamError::from_envelope(e).to_string(),
            "game maintenance: upstream 503 (code=0) Service Unavailable: Server under Maintenance "
        );
        let both = UpstreamEnvelope {
            msg: "a".into(),
            message: "b".into(),
            ..UpstreamEnvelope::default()
        };
        assert_eq!(both.to_string(), "upstream 0 (code=0) : a b ");
    }

    #[test]
    fn envelope_body_without_fields_parses() {
        let e: UpstreamEnvelope = serde_json::from_str(r#"{"result":4}"#).unwrap();
        assert_eq!(e, UpstreamEnvelope::default());
        let e: UpstreamEnvelope =
            serde_json::from_str(r#"{"statusCode":401,"code":5516,"msg":"no replay"}"#).unwrap();
        assert_eq!(
            (e.status_code, e.code, e.msg.as_str()),
            (401, 5516, "no replay")
        );
    }
}

mod fetch {
    use myrtle_hypergryph::fetch::redacted_body;

    #[test]
    fn credential_shaped_keys_are_redacted_and_plain_ones_kept() {
        for key in [
            "token",
            "refresh_token",
            "accessToken",
            "access_key",
            "captcha_key",
            "TransCode",
            "password",
            "pwd",
            "Secret",
            "yostar_token",
            "sign",
            "cookie",
            "ticket",
            "credential",
        ] {
            let body = format!(r#"{{"{key}":"VALUE"}}"#);
            assert_eq!(
                redacted_body(&body),
                format!(r#"{{"{key}":"<redacted>"}}"#),
                "{key}"
            );
        }
        for key in [
            "code",
            "statusCode",
            "msg",
            "message",
            "uid",
            "result",
            "error",
            "info",
        ] {
            let body = format!(r#"{{"{key}":"VALUE"}}"#);
            assert_eq!(redacted_body(&body), body, "{key}");
        }
    }

    #[test]
    fn redaction_blanks_credentials_and_bounds_length() {
        let body = r#"{"uid":"1","token":"abc","access_key":"k","Secret":"s","code":0}"#;
        assert_eq!(
            redacted_body(body),
            r#"{"uid":"1","token":"<redacted>","access_key":"<redacted>","Secret":"<redacted>","code":0}"#
        );
        let long = "é".repeat(400);
        let out = redacted_body(&long);
        assert!(out.starts_with(&"é".repeat(300)));
        assert!(out.ends_with("(+200 bytes)"), "{out}");
    }
}

mod login {
    mod bilibili {
        use myrtle_hypergryph::login::bilibili::{BilibiliLoginResult, OTP_TTL_SECS, OtpChallenge};

        #[test]
        fn login_result_accepts_a_numeric_or_string_uid() {
            let n: BilibiliLoginResult =
                serde_json::from_str(r#"{"code":0,"uid":123456,"access_key":"k"}"#).unwrap();
            assert_eq!((n.uid.as_str(), n.access_key.as_str()), ("123456", "k"));
            let s: BilibiliLoginResult =
                serde_json::from_str(r#"{"uid":"123456","access_key":"k"}"#).unwrap();
            assert_eq!(s.uid, "123456");
            assert!(
                serde_json::from_str::<BilibiliLoginResult>(r#"{"uid":1.5,"access_key":"k"}"#)
                    .is_err()
            );
        }

        #[test]
        fn a_challenge_expires_after_its_window() {
            let c: OtpChallenge = serde_json::from_value(serde_json::json!({
                "phone": "2065550100",
                "country_code": "1",
                "captcha_key": "k",
                "device_id": "d",
                "bd_id": "b",
                "issued_at": 1_000,
            }))
            .unwrap();
            assert!(!c.is_expired(1_000 + OTP_TTL_SECS));
            assert!(c.is_expired(1_001 + OTP_TTL_SECS));
            assert!(!c.is_expired(0), "a clock behind issue time is not expired");
        }
    }

    mod yostar {
        use myrtle_hypergryph::login::yostar::AccountPortalSession;

        #[test]
        fn debug_hides_the_portal_cookies() {
            let portal = AccountPortalSession {
                yssid: "COOKIE-ONE".into(),
                yssid_sig: "COOKIE-TWO".into(),
            };
            let shown = format!("{portal:?}");
            assert!(
                !shown.contains("COOKIE-ONE") && !shown.contains("COOKIE-TWO"),
                "{shown}"
            );
        }
    }
}

mod replay {
    use std::time::Duration;

    use myrtle_hypergryph::replay::{
        BattleReplay, Direction, Op, ReplayHarvestOptions, decode_battle_replay,
        saved_replay_targets,
    };
    use serde_json::{Value, json};

    /// A deflate zip of one `default_entry`
    const REPLAY_ZIP_B64: &str = "UEsDBBQAAAAIAKqxblfS/TKfrQEAAN4CAAANAAAAZGVmYXVsdF9lbnRyeW1SwW7bMAz9F+7qtJaTIoBvW7BDsQwNumKHFYXB2YzDRZYcSfYWBP73UXK7tdh8sh7f43skeIEaux65NXdGn7+S82wNlHkGgTvyQYpQglrnzx9k8MMOzqCG8gIdBWwwYPwXrmnQNTuN5wfRQqmK/OomgxY7uic/6CBQBh5HmutQ5MVyodRCrR6KolTLssi/iYGjDtmwaTfWR03+CtrynnaWjeDLDI6sNTUfDXVMfhPBlZA79v4tKqCmkfRtI65338P1Z+l2naAqNq5ytcjXYi1DtJRYb+ARNTef/mM2ieQ0oAgeL1Af0N0aH6K+kMKRTWoV8SrPiwo7PuM7JQ1D16cwZtA6MfWczR/rg5t5VQEvFdPQr3l38bkdZffr54mgvJHh+gN6Sqa9DWQCo75Hc5RiBnscrfuAIWja/aGxf+89+7CRaFDuUXvKYDBMp4H7lOTlIcHVHAj+MraztZqeJIZtfRr/1cGoq0KcvVwVh8FRPA+Rnoa0WomeNvXvakDWafsUsGFHdUi3uIxT+djD2Z/pWVsxX03R3cnR2e4LkXRbic4NhrYcz+bxaZp+A1BLAQIUAxQAAAAIAKqxblfS/TKfrQEAAN4CAAANAAAAAAAAAAAAAACAAQAAAABkZWZhdWx0X2VudHJ5UEsFBgAAAAABAAEAOwAAANgBAAAAAA==";

    #[test]
    fn decodes_a_deflated_replay() {
        let replay = decode_battle_replay(REPLAY_ZIP_B64).unwrap();
        assert!(!replay.campaign_only_version);
        assert_eq!(replay.timestamp.timestamp(), 1_700_000_000);
        let meta = &replay.journal.metadata;
        assert!(meta.game_result);
        assert_eq!(meta.stage_id, "main_01-07");
        assert_eq!(meta.save_time.timestamp(), 1_700_000_000);
        assert_eq!(replay.journal.random_seed, 42);

        let amiya = &replay.journal.squad[0];
        assert_eq!((amiya.char_inst_id, amiya.tmpl_id.as_deref()), (2, None));
        assert!(!amiya.is_assist_char);

        let log = &replay.journal.logs[0];
        assert_eq!((log.op, log.direction), (Op::Skill, Direction::Left));
        assert_eq!((log.pos.row, log.pos.col), (3, 4));
        assert_eq!(log.signature.char_id, "char_002_amiya");
        assert_eq!(log.op.to_string(), "skill");
        assert_eq!(log.direction.to_string(), "left");
    }

    #[test]
    fn decode_failures_name_their_stage() {
        let msg = |b64: &str| decode_battle_replay(b64).unwrap_err().to_string();
        assert!(msg("***").contains("base64 decode"));
        assert!(msg("aGVsbG8=").contains("open zip"));
    }

    #[test]
    fn loose_fields_accept_both_spellings() {
        let probe = |b: Value, n: Value| {
            serde_json::from_str::<BattleReplay>(
                &json!({
                    "campaignOnlyVersion": b,
                    "timestamp": n,
                    "journal": {
                        "metadata": {
                            "standardPlayTime": 0.0, "gameResult": true, "saveTime": "2023-11-14T22:13:20Z",
                            "remainingCost": 0, "remainingLifePoint": 0, "killedEnemiesCnt": 0, "missedEnemiesCnt": 0,
                            "levelId": "", "stageId": "", "validKilledEnemiesCnt": 0,
                        },
                        "squad": [], "logs": [], "randomSeed": 0, "runeList": null,
                    },
                })
                .to_string(),
            )
        };
        let p = probe(json!(1), json!("12")).unwrap();
        assert!(p.campaign_only_version);
        assert_eq!(p.timestamp.timestamp(), 12);
        let p = probe(json!(false), json!(-3)).unwrap();
        assert!(!p.campaign_only_version);
        assert_eq!(p.timestamp.timestamp(), -3);
        assert!(probe(json!("yes"), json!(1)).is_err());
        assert!(probe(json!(true), json!("x")).is_err());
    }

    #[test]
    fn unknown_op_codes_are_rejected() {
        assert!(serde_json::from_str::<Op>("3").is_err());
        assert!(serde_json::from_str::<Direction>("4").is_err());
        assert_eq!(serde_json::from_str::<Op>("1").unwrap(), Op::Retreat);
    }

    #[test]
    fn targets_come_from_both_flag_trees() {
        let sync = json!({ "user": {
            "dungeon": { "stages": {
                "main_01-07": { "hasBattleReplay": 1 },
                "main_01-08": { "hasBattleReplay": 0 },
                "main_01-09": { "hasBattleReplay": true },
                "main_01-10": {},
            }},
            "campaignsV2": { "instances": { "camp_r_01": { "hasBattleReplay": 1 } } },
        }});
        let mut targets = saved_replay_targets(&sync);
        targets.sort();
        assert_eq!(
            targets,
            [
                ("campaignV2".to_owned(), "camp_r_01".to_owned()),
                ("quest".to_owned(), "main_01-07".to_owned()),
                ("quest".to_owned(), "main_01-09".to_owned()),
            ]
        );
        assert!(saved_replay_targets(&json!({})).is_empty());
    }

    #[test]
    fn jitter_stays_in_range() {
        let (lo, hi) = (Duration::from_millis(80), Duration::from_millis(180));
        let delay = |min_interval, max_interval| {
            ReplayHarvestOptions {
                min_interval,
                max_interval,
                ..ReplayHarvestOptions::default()
            }
            .next_delay()
        };
        for _ in 0..50 {
            let d = delay(lo, hi);
            assert!(d >= lo && d <= hi);
        }
        assert_eq!(delay(hi, lo), hi);
        assert_eq!(delay(Duration::ZERO, Duration::ZERO), Duration::ZERO);
    }
}

mod session {
    use myrtle_hypergryph::AuthSession;

    #[test]
    fn debug_hides_every_secret() {
        let session = AuthSession {
            uid: "u8-uid".into(),
            secret: "SECRET-VALUE".into(),
            seqnum: 3,
            token: "U8-TOKEN".into(),
            yostar_uid: "yostar-uid".into(),
            yostar_token: "DURABLE-TOKEN".into(),
        };
        let shown = format!("{session:?}");
        for secret in ["SECRET-VALUE", "U8-TOKEN", "DURABLE-TOKEN"] {
            assert!(!shown.contains(secret), "{shown}");
        }
        assert!(
            shown.contains("u8-uid") && shown.contains("seqnum: 3"),
            "{shown}"
        );
    }

    mod account {
        use myrtle_hypergryph::session::account::refuses_credential;
        use myrtle_hypergryph::{Domain, SessionStoreError, UpstreamEnvelope, UpstreamError};
        use myrtle_types::Server;

        #[test]
        fn every_variant_is_classified() {
            fn slot(e: &UpstreamError) -> usize {
                match e {
                    UpstreamError::DomainNotFound { .. } => 0,
                    UpstreamError::Transport(_) => 1,
                    UpstreamError::Timeout(_) => 2,
                    UpstreamError::HttpStatus { .. } => 3,
                    UpstreamError::ApiCode { .. } => 4,
                    UpstreamError::NotLoggedIn => 5,
                    UpstreamError::Parse { .. } => 6,
                    UpstreamError::Envelope(_) => 7,
                    UpstreamError::Maintenance(_) => 8,
                    UpstreamError::LoginRejected { .. } => 9,
                    UpstreamError::RiskControl { .. } => 10,
                    UpstreamError::Unsupported { .. } => 11,
                    UpstreamError::ServiceAccountRefused { .. } => 12,
                    UpstreamError::Store(_) => 13,
                }
            }

            let reqwest_error = || {
                reqwest::Response::from(http::Response::builder().status(500).body("").unwrap())
                    .error_for_status()
                    .unwrap_err()
            };
            let table = [
                (
                    UpstreamError::DomainNotFound {
                        server: Server::EN,
                        domain: Domain::U8,
                    },
                    false,
                ),
                (UpstreamError::Transport(reqwest_error()), false),
                (UpstreamError::Timeout(reqwest_error()), false),
                (
                    UpstreamError::HttpStatus {
                        url: "u".into(),
                        status: 500,
                    },
                    false,
                ),
                (
                    UpstreamError::ApiCode {
                        url: "u".into(),
                        code: 1,
                        message: String::new(),
                    },
                    false,
                ),
                (UpstreamError::NotLoggedIn, false),
                (
                    UpstreamError::Parse {
                        context: "get_u8_token".into(),
                        message: "x".into(),
                    },
                    true,
                ),
                (
                    UpstreamError::from_envelope(UpstreamEnvelope::default()),
                    true,
                ),
                (
                    UpstreamError::from_envelope(UpstreamEnvelope {
                        msg: "maintenance".into(),
                        ..UpstreamEnvelope::default()
                    }),
                    true,
                ),
                (
                    UpstreamError::LoginRejected {
                        context: "user/v1/getToken",
                        code: "1".into(),
                        message: String::new(),
                    },
                    true,
                ),
                (
                    UpstreamError::RiskControl {
                        code: "200007".into(),
                        message: String::new(),
                    },
                    true,
                ),
                (
                    UpstreamError::Unsupported {
                        server: Server::TW,
                        what: "x",
                    },
                    true,
                ),
                (
                    UpstreamError::ServiceAccountRefused {
                        server: Server::EN,
                        first_of_outage: true,
                        reason: String::new(),
                    },
                    false,
                ),
                (
                    UpstreamError::Store(SessionStoreError::Unavailable("x".into())),
                    false,
                ),
            ];
            for (i, (err, refuses)) in table.iter().enumerate() {
                assert_eq!(slot(err), i, "table order");
                assert_eq!(refuses_credential(err), *refuses, "{err:?}");
            }
            assert_eq!(table.len(), 14, "one row per variant");
        }
    }

    mod health {
        use myrtle_hypergryph::{AccountState, HealthTracker};

        #[test]
        fn health_moves_through_unknown_refused_ok() {
            let mut health = HealthTracker::default();
            assert_eq!(health.state(), AccountState::Unknown);

            assert!(health.refused(100), "the outage starts");
            assert!(!health.refused(200));
            assert_eq!(health.state(), AccountState::Refused);
            assert_eq!(
                health.refused_since(),
                Some(100),
                "an outage keeps its start"
            );

            health.ok(300);
            assert_eq!(health.state(), AccountState::Ok);
            assert_eq!(health.refused_since(), None);
            assert_eq!(health.last_ok_at(), Some(300));

            assert!(health.refused(400));
            assert_eq!(
                health.refused_since(),
                Some(400),
                "a new outage starts fresh"
            );
            assert_eq!(health.state(), AccountState::Refused);

            health.ok(500);
            assert_eq!(health.last_ok_at(), Some(500), "each success is recorded");
        }
    }

    mod store {
        use std::error::Error as _;

        use myrtle_hypergryph::{DurableCredential, SessionStoreError, UpstreamError};

        #[test]
        fn debug_hides_the_durable_token() {
            let credential = DurableCredential {
                yostar_uid: "yostar-uid".into(),
                yostar_token: "DURABLE-TOKEN".into(),
            };
            let shown = format!("{credential:?}");
            assert!(!shown.contains("DURABLE-TOKEN"), "{shown}");
            assert!(shown.contains("yostar-uid"), "{shown}");
        }

        #[test]
        fn store_errors_log_their_kind_only() {
            for err in [
                SessionStoreError::Corrupt("token=abc".into()),
                SessionStoreError::Unavailable("token=abc".into()),
            ] {
                assert_eq!(err.detail(), "token=abc");
                assert!(!err.to_string().contains("abc"), "{err}");
                assert_eq!(
                    format!("{err:?}"),
                    format!("SessionStoreError::{}", err.kind())
                );
                let kind = err.kind();
                let upstream = UpstreamError::from(err);
                assert!(!upstream.to_string().contains("abc"), "{upstream}");
                assert!(!format!("{upstream:?}").contains("abc"), "{upstream:?}");
                let source = upstream.source().unwrap().to_string();
                assert!(!source.contains("abc") && !source.is_empty(), "{source}");
                assert!(upstream.to_string().contains(kind));
            }
        }
    }
}

mod siren {
    use chrono::NaiveDate;
    use myrtle_hypergryph::siren::{cdn_day, signed_url_expiry};

    #[test]
    fn expiry_is_the_hex_segment() {
        let url = "https://res01.hycdn.cn/2034bfa9e33b9483d844dcb611b9f0b9/6ACB78D8/siren/audio/20240709/a7f6.wav";
        assert_eq!(signed_url_expiry(url), Some(0x6ACB_78D8));
        assert_eq!(signed_url_expiry("https://example.com/x.wav"), None);
    }

    #[test]
    fn cdn_day_reads_the_eight_digit_segment() {
        let url = "https://web.hycdn.cn/siren/pic/20261010/ab46323f8b0d4580e34b61c837851cc7.jpg";
        assert_eq!(cdn_day(url), NaiveDate::from_ymd_opt(2026, 10, 10));
        assert_eq!(cdn_day("https://web.hycdn.cn/siren/pic/x.jpg"), None);
    }
}
