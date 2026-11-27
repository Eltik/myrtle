mod audio {
    use myrtle_gamedata_raw::tables::audio::{MusicBanks, RawAudioData};

    const RAW: &str = r#"{
        "Musics": [
            {"Id": "music_act13side_0", "Name": "The Grand Knight Territory", "Bank": "sys.ON_ACTIVITY_LOADED.act13side"},
            {"Id": "music_3in1bg_act13side", "Name": " ", "Bank": "sys.ON_ACTIVITY_LOADED.act13side"},
            {"Id": "music_3in1bg_main1", "Name": " ", "Bank": "battle.ON_GAME_READY.escape"},
            {"Id": "music_bg_volcano_day", "Name": "Misty Memory", "Bank": "sys.ON_ACTIVITY_LOADED.act27side.day"}
        ],
        "BgmBanks": [
            {"Name": "sys.ON_ACTIVITY_LOADED.act13side", "Intro": "Audio/Sound_Beta_2/Music/act13side/m_sys_kazimierz2_intro", "Loop_": "Audio/Sound_Beta_2/Music/act13side/m_sys_kazimierz2_loop"},
            {"Name": "battle.ON_GAME_READY", "Loop_": "Audio/Sound_Beta_2/Music/beta1_180603/m_bat_indust_loop"},
            {"Name": "sys.ON_ACTIVITY_LOADED.act27side.day", "Loop_": "Audio/Sound_Beta_2/Music/act27side/m_sys_volcano_day_loop"}
        ],
        "BankAlias": [{"key": "battle.ON_GAME_READY.escape", "value": "battle.ON_GAME_READY"}]
    }"#;

    #[test]
    fn a_bank_resolves_through_its_alias() {
        let raw: RawAudioData = serde_json::from_str(RAW).unwrap();
        let banks = MusicBanks::from_raw(&raw);
        assert_eq!(banks.counts(), (4, 3, 1));
        let m = banks.music("music_3in1bg_main1").unwrap();
        assert_eq!(m.bank, "battle.ON_GAME_READY.escape");
        assert!(m.name.trim().is_empty());
        let bank = banks.bank(&m.bank).unwrap();
        assert_eq!(
            bank.loop_.as_deref(),
            Some("Audio/Sound_Beta_2/Music/beta1_180603/m_bat_indust_loop")
        );
        assert!(bank.intro.is_none());
        assert!(banks.bank("battle.ON_NOTHING").is_none());
    }

    #[test]
    fn the_named_row_wins_a_shared_bank() {
        let raw: RawAudioData = serde_json::from_str(RAW).unwrap();
        let banks = MusicBanks::from_raw(&raw);
        let m = banks
            .music_for_bank("sys.ON_ACTIVITY_LOADED.act13side")
            .unwrap();
        assert_eq!(m.id, "music_act13side_0");
        assert_eq!(m.name, "The Grand Knight Territory");
        let intro = banks.bank(&m.bank).unwrap().intro.as_deref();
        assert_eq!(
            intro,
            Some("Audio/Sound_Beta_2/Music/act13side/m_sys_kazimierz2_intro")
        );
    }

    #[test]
    fn a_suffixed_bank_is_reached_by_prefix() {
        let raw: RawAudioData = serde_json::from_str(RAW).unwrap();
        let banks = MusicBanks::from_raw(&raw);
        assert!(
            banks
                .music_for_bank("sys.ON_ACTIVITY_LOADED.act27side")
                .is_none()
        );
        let m = banks
            .music_by_bank_prefix("sys.ON_ACTIVITY_LOADED.act27side.")
            .unwrap();
        assert_eq!(m.id, "music_bg_volcano_day");
    }
}

mod autochess {
    use myrtle_gamedata_raw::tables::autochess::ActivityAutoChessFile;

    #[test]
    fn a_bond_reads_its_type_icon_and_factions() {
        let raw = r#"{"BasicInfo": [], "AutoChessData": {"BandDataDict": [], "BondInfoDict": [
            {"key": "yanShip", "value": {"BondId": "yanShip", "BondOrder": 2,
             "BondType": "SEASON", "Icon": "icon_yanShip", "IsHiddenCharList": false,
             "IsPower": true, "Name": "Yan", "PowerIdList": ["yan", "sui"]}}]}}"#;
        let file: ActivityAutoChessFile = serde_json::from_str(raw).expect("parses");
        let b = &file.auto_chess_data.bond_info_dict["yanShip"];
        assert_eq!(b.name, "Yan");
        assert_eq!(b.bond_type, "SEASON");
        assert_eq!(b.icon, "icon_yanShip");
        assert_eq!(b.power_id_list, ["yan", "sui"]);
    }
}

mod campaign {
    use myrtle_gamedata_raw::tables::campaign::{CampaignRotations, CampaignTableFile};

    /// Guard against the real table's shape drifting away from the parser
    /// (kill ladders live in `Campaigns[].BreakLadders[].KillCnt`). Skips
    /// when the gamedata assets aren't checked out.
    #[test]
    fn kill_max_parses_from_real_campaign_table() {
        let path = std::path::Path::new("../assets/output/en/gamedata/excel/campaign_table.json");
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        let table: CampaignTableFile = serde_json::from_str(&text).unwrap();
        let rotations = CampaignRotations::from_table(table);
        assert_eq!(rotations.kill_max("camp_01"), Some(400));
        assert_eq!(rotations.kill_max("camp_r_01"), Some(400));
        assert_eq!(rotations.kill_max("main_00-01"), None);
    }
}

mod event_shop {
    use myrtle_gamedata_raw::tables::event_shop::EventShopData;
    use serde_json::json;

    #[test]
    fn response_parses_goods_and_the_buyout_total() {
        let body = json!({"data": {
            "shopId": "shop_act44side", "shopName": "Vintage Bazaar",
            "price": {"id": "act44side_token_ticket", "count": 0, "type": "ACTIVITY_ITEM"},
            "startTime": 100, "endTime": 200,
            "allPriceDict": [{"startTime": 100, "maxPrice": 8854}],
            "shopGroup": {"g1": {"shopGood": {
                "act44side_2": {"goodId": "act44side_2", "goodType": "NORMAL", "item": {"id": "char_x@y#1", "count": 1, "type": "CHAR_SKIN"}, "price": 500, "availCount": 1},
                "act44side_35": {"goodId": "act44side_35", "goodType": "NORMAL", "item": {"id": "4001", "count": 100, "type": "GOLD"}, "price": 1, "availCount": -1},
                "act44side_1": {"goodId": "act44side_1", "goodType": "NORMAL", "item": {"id": "", "count": 0, "type": ""}, "price": 0, "availCount": 0}
            }}}
        }});
        let shop = EventShopData::from_response(&body).unwrap();
        assert_eq!(shop.max_price, 8854);
        assert_eq!(shop.goods.len(), 2);
        assert_eq!(shop.goods[0].item_id, "char_x@y#1");
        assert_eq!(shop.token_id, "act44side_token_ticket");
    }

    #[test]
    fn buyout_falls_back_to_limited_goods_when_the_dict_is_missing() {
        let body = json!({"data": {"shopId": "shop_a", "shopGroup": {"g": {"shopGood": {
            "a_1": {"goodId": "a_1", "item": {"id": "i", "count": 1, "type": "MATERIAL"}, "price": 10, "availCount": 3},
            "a_2": {"goodId": "a_2", "item": {"id": "j", "count": 1, "type": "MATERIAL"}, "price": 5, "availCount": -1}
        }}}}});
        assert_eq!(EventShopData::from_response(&body).unwrap().max_price, 30);
    }
}

mod handbook_team {
    use myrtle_gamedata_raw::tables::handbook_team::HandbookTeamTableFile;

    #[test]
    fn a_team_row_reads_its_level_and_name() {
        let raw = r#"{"Handbook_teams": [{"key": "rhodes", "value": {
            "Color": "000000", "IsLimited": false, "IsRaw": false, "OrderNum": 1,
            "PowerCode": "Rhodes Island", "PowerId": "rhodes", "PowerLevel": 0,
            "PowerName": "Rhodes Island"}}]}"#;
        let file: HandbookTeamTableFile = serde_json::from_str(raw).expect("parses");
        let t = &file.teams["rhodes"];
        assert_eq!(t.power_name, "Rhodes Island");
        assert_eq!(t.power_level, 0);
        assert_eq!(t.order_num, 1);
    }
}

mod level {
    use myrtle_gamedata_raw::tables::level::is_level_file;

    #[test]
    fn level_files_skip_the_stats_and_replacer_directories() {
        assert!(is_level_file("obt/main/level_main_01-07.json"));
        assert!(is_level_file("levels_meta.json.json"));
        assert!(!is_level_file("enemydata/enemy_database.json"));
        assert!(!is_level_file(
            "obt/roguelike/ro4/levelreplacers/level_x.json"
        ));
        assert!(!is_level_file("obt/main/level_main_01-07.txt"));
        assert!(!is_level_file("obt/main/.json"));
    }
}

mod material {
    use myrtle_gamedata_raw::tables::material::ItemType;

    /// Every item type the EN and CN item tables shipped on 2026-11-03 whose name the plain
    /// `rename_all` spelling would miss.
    #[test]
    fn game_item_types_round_trip() {
        for name in [
            "TKT_GACHA_10",
            "TKT_RECRUIT",
            "LIMITED_TKT_GACHA_10",
            "LINKAGE_TKT_GACHA",
            "LINKAGE_TKT_GACHA_10",
            "CLASSIC_TKT_GACHA_10",
            "EXCLUSIVE_TKT_GACHA_10",
            "VOUCHER_LEVELMAX_4",
            "VOUCHER_LEVELMAX_5",
            "VOUCHER_LEVELMAX_6",
            "VOUCHER_ELITE_II_4",
            "VOUCHER_SKILL_SPECIALLEVELMAX_4",
            "VOUCHER_SKILL_SPECIALLEVELMAX_5",
            "VOUCHER_SKILL_SPECIALLEVELMAX_6",
            "VOUCHER_SKIN_V2",
            "RANDOM_VOUCHER_SKIN",
            "CLASSIC_FES_PICK_TIER_5",
            "CLASSIC_FES_PICK_TIER_6",
            "CRS_SHOP_COIN_V2",
            "GIFTPACKAGE_TKT",
            "EMOTICON_SET",
            "SO_CHAR_EXP",
            "PLOT_ITEM",
        ] {
            let json = format!("\"{name}\"");
            let parsed: ItemType = serde_json::from_str(&json).unwrap();
            assert_ne!(parsed, ItemType::Unknown, "{name} fell to Unknown");
            assert_eq!(
                serde_json::to_string(&parsed).unwrap(),
                json,
                "{name} does not round-trip"
            );
        }
    }
}

mod medal {
    use std::collections::HashMap;

    use myrtle_gamedata_raw::tables::medal::{
        EventWindow, ExpireTime, MedalData, MedalDefinition, Obtainability,
    };
    use myrtle_gamedata_raw::tables::operator::Operator;

    const NOW: i64 = 1_700_000_000;

    fn mk_medal(id: &str, template: &str, unlock_param: &[&str]) -> MedalDefinition {
        MedalDefinition {
            medal_id: id.to_string(),
            medal_name: id.to_string(),
            medal_type: "activityMedal".to_string(),
            slot_id: 0,
            pre_medal_id_list: Vec::new(),
            rarity: "T1".to_string(),
            template: template.to_string(),
            unlock_param: unlock_param.iter().map(|s| (*s).to_string()).collect(),
            get_method: String::new(),
            description: String::new(),
            display_time: 0,
            expire_times: Vec::new(),
            medal_reward_group: serde_json::Value::Null,
            is_hidden: false,
            origin_medal: String::new(),
        }
    }

    fn data_with(medals: Vec<MedalDefinition>) -> MedalData {
        let mut d = MedalData::default();
        for m in medals {
            d.medals.insert(m.medal_id.clone(), m);
        }
        d
    }

    fn mk_operator(team_id: Option<&str>, display_number: &str, name: &str) -> Operator {
        Operator {
            name: name.to_string(),
            team_id: team_id.map(str::to_string),
            display_number: display_number.to_string(),
            ..Default::default()
        }
    }

    fn data_with_tower_windows(windows: Vec<(i64, i64)>) -> MedalData {
        let mut d = data_with(vec![mk_medal("medal_t", "PassTower", &["tower_n_10"])]);
        d.tower_windows.insert("medal_t".to_string(), windows);
        d
    }

    fn data_with_event_window(window: EventWindow) -> MedalData {
        let mut d = data_with(vec![mk_medal("medal_ev", "", &[])]);
        d.event_windows.insert("medal_ev".to_string(), window);
        d
    }

    #[test]
    fn no_expire_times_is_permanent() {
        let d = data_with(vec![mk_medal("medal_perm", "", &[])]);
        assert_eq!(d.obtainability("medal_perm", NOW), Obtainability::Permanent);
    }

    /// Login-anniversary medals (`JoinGameDays`, "log in for 6 years") carry an
    /// `INIT` `ExpireTimes` window with a real End, open since account creation.
    /// INIT is neither TEMP nor PERM, so it must stay Permanent.
    #[test]
    fn init_expire_time_is_permanent() {
        let mut medal = mk_medal("medal_player_joingame_07", "JoinGameDays", &["2190"]);
        medal.medal_type = "playerMedal".to_string();
        medal.expire_times = vec![ExpireTime {
            start: -1,
            end: NOW + 1_000_000,
            expire_type: "INIT".to_string(),
        }];
        let d = data_with(vec![medal]);
        assert_eq!(
            d.obtainability("medal_player_joingame_07", NOW),
            Obtainability::Permanent
        );
    }

    #[test]
    fn tower_in_season_is_active_event() {
        let d = data_with_tower_windows(vec![(NOW - 100, NOW + 100)]);
        assert_eq!(
            d.obtainability("medal_t", NOW),
            Obtainability::Event {
                proxy_close_ts: NOW + 100
            }
        );
    }

    #[test]
    fn tower_between_seasons_is_closed_event() {
        let d = data_with_tower_windows(vec![(NOW - 300, NOW - 100)]);
        assert_eq!(
            d.obtainability("medal_t", NOW),
            Obtainability::Event {
                proxy_close_ts: NOW - 100
            }
        );
    }

    #[test]
    fn tower_future_season_is_not_yet() {
        let d = data_with_tower_windows(vec![(NOW + 100, NOW + 300)]);
        assert_eq!(d.obtainability("medal_t", NOW), Obtainability::NotYet);
    }

    #[test]
    fn retired_tower_with_no_windows_is_unobtainable() {
        // Pre-schedule tower never carried into SeasonInfos -> empty windows.
        let d = data_with_tower_windows(Vec::new());
        assert_eq!(d.obtainability("medal_t", NOW), Obtainability::Unobtainable);
    }

    #[test]
    fn one_time_mode_active_is_event() {
        let d = data_with_event_window(EventWindow {
            start: NOW - 100,
            close: NOW + 100,
            one_time: true,
        });
        assert_eq!(
            d.obtainability("medal_ev", NOW),
            Obtainability::Event {
                proxy_close_ts: NOW + 100
            }
        );
    }

    #[test]
    fn one_time_mode_future_is_not_yet() {
        let d = data_with_event_window(EventWindow {
            start: NOW + 100,
            close: NOW + 300,
            one_time: true,
        });
        assert_eq!(d.obtainability("medal_ev", NOW), Obtainability::NotYet);
    }

    #[test]
    fn one_time_mode_finished_is_unobtainable() {
        let d = data_with_event_window(EventWindow {
            start: NOW - 300,
            close: NOW - 100,
            one_time: true,
        });
        assert_eq!(
            d.obtainability("medal_ev", NOW),
            Obtainability::Unobtainable
        );
    }

    // Event repair: the activity schedule overrides a mislabeled table.

    #[test]
    fn event_future_window_repairs_to_not_yet() {
        // Table mislabels it Permanent (empty expire times), but the activity
        // hasn't started -> NotYet, not a false permanent.
        let d = data_with_event_window(EventWindow {
            start: NOW + 100,
            close: NOW + 300,
            one_time: false,
        });
        assert_eq!(d.obtainability("medal_ev", NOW), Obtainability::NotYet);
    }

    #[test]
    fn event_closed_window_is_decayed_event() {
        let d = data_with_event_window(EventWindow {
            start: NOW - 300,
            close: NOW - 100,
            one_time: false,
        });
        assert_eq!(
            d.obtainability("medal_ev", NOW),
            Obtainability::Event {
                proxy_close_ts: NOW - 100
            }
        );
    }

    #[test]
    fn link_operator_locks_catches_collab_only() {
        let d_medals = vec![
            mk_medal("medal_collab", "GotChars", &["char_9001_collab"]),
            mk_medal("medal_regular", "GotChars", &["char_0001_regular"]),
        ];
        let mut d = data_with(d_medals);

        let mut ops = HashMap::new();
        ops.insert(
            "char_9001_collab".to_string(),
            mk_operator(Some("rainbow"), "RB01", "Collab Op"),
        );
        ops.insert(
            "char_0001_regular".to_string(),
            mk_operator(None, "R001", "Regular Op"),
        );

        d.link_operator_locks(&ops);

        assert!(d.operator_lock("medal_collab").is_some());
        assert!(d.operator_lock("medal_regular").is_none());
    }

    /// Luo Xiaohei (`char_4067_lolxh`) is a team-less collab whose `R` display
    /// prefix collides with ordinary Rhodes ops, so `is_collab` catches it via
    /// its curated char id. Its `CharStoryUnlock` medal must be operator-locked.
    #[test]
    fn link_operator_locks_catches_teamless_lolxh_collab() {
        let mut d = data_with(vec![mk_medal(
            "medal_hidden_story_lolxh_1",
            "CharStoryUnlock",
            &["char_4067_lolxh", "story_lolxh_set_1"],
        )]);

        let mut ops = HashMap::new();
        // TeamId null, DisplayNumber "R161", as in the real gamedata.
        let mut lolxh = mk_operator(None, "R161", "Luo Xiaohei");
        lolxh.id = Some("char_4067_lolxh".to_string());
        ops.insert("char_4067_lolxh".to_string(), lolxh);

        d.link_operator_locks(&ops);

        assert!(d.operator_lock("medal_hidden_story_lolxh_1").is_some());
    }
}

mod mission {
    use std::collections::HashMap;

    use myrtle_gamedata_raw::tables::mission::{
        DailyPeriod, DailyPeriodGroup, LMD_ITEM_ID, MissionData, PeriodicalReward, RewardItem,
    };

    fn chest(group: &str, lmd: i64, begin: i64, end: i64) -> PeriodicalReward {
        PeriodicalReward {
            group_id: group.into(),
            rewards: vec![
                RewardItem {
                    id: LMD_ITEM_ID.into(),
                    count: lmd,
                },
                RewardItem {
                    id: "2001".into(),
                    count: 4,
                },
            ],
            begin_time: begin,
            end_time: end,
        }
    }

    #[test]
    fn daily_lmd_weights_weekday_and_weekend_groups_by_their_days() {
        let data = MissionData {
            daily_periods: vec![DailyPeriod {
                start_time: 100,
                end_time: 200,
                period_list: vec![
                    DailyPeriodGroup {
                        reward_group_id: "wk".into(),
                        period: vec![1, 2, 3, 4, 5],
                    },
                    DailyPeriodGroup {
                        reward_group_id: "we".into(),
                        period: vec![6, 7],
                    },
                ],
            }],
            daily_rewards: HashMap::from([
                ("a".to_string(), chest("wk", 700, 0, 0)),
                ("b".to_string(), chest("wk", 700, 0, 0)),
                ("c".to_string(), chest("we", 1400, 0, 0)),
            ]),
            weekly_rewards: HashMap::new(),
        };
        // Weekdays 1400 x 5/7, weekends 1400 x 2/7: 1400 a day.
        assert!((data.daily_lmd_per_day(150).unwrap() - 1400.0).abs() < 1e-9);
        assert!(
            data.daily_lmd_per_day(300).is_none(),
            "outside every period"
        );
    }

    #[test]
    fn weekly_lmd_is_the_live_chests_over_seven_days() {
        let data = MissionData {
            daily_periods: Vec::new(),
            daily_rewards: HashMap::new(),
            weekly_rewards: HashMap::from([
                ("old".to_string(), chest("g1", 7000, 0, 99)),
                ("live".to_string(), chest("g3", 7000, 100, 200)),
                ("live2".to_string(), chest("g3", 14000, 100, 200)),
            ]),
        };
        assert!((data.weekly_lmd_per_day(150).unwrap() - 3000.0).abs() < 1e-9);
        assert!((data.weekly_lmd_per_day(50).unwrap() - 1000.0).abs() < 1e-9);
        assert!(data.weekly_lmd_per_day(300).is_none());
    }
}

mod obtain {
    use myrtle_gamedata_raw::tables::obtain::{ObtainChannel, SkinChannel};

    #[test]
    fn operator_wording_reads_by_prefix() {
        assert_eq!(
            ObtainChannel::classify("Recruitment & Headhunting, Pinboard Missions"),
            Some(ObtainChannel::Headhunting)
        );
        assert_eq!(
            ObtainChannel::classify("集成战略获得"),
            Some(ObtainChannel::IntegratedStrategies)
        );
        assert_eq!(
            ObtainChannel::classify("Main Theme Story"),
            Some(ObtainChannel::Other)
        );
        // KR wording is not read here; the reference pass resolves it.
        assert_eq!(
            ObtainChannel::classify("인재 채용"),
            Some(ObtainChannel::Other)
        );
        assert_eq!(ObtainChannel::classify(""), None);
    }

    #[test]
    fn skin_tags_read_exactly() {
        assert_eq!(SkinChannel::classify(None), SkinChannel::Store);
        assert_eq!(SkinChannel::classify(Some("联动获得")), SkinChannel::Collab);
        assert_eq!(
            SkinChannel::classify(Some("Official Artworks")),
            SkinChannel::CodeExchange
        );
        assert_eq!(SkinChannel::classify(Some("콜라보")), SkinChannel::Other);
    }
}

mod sandbox_universe {
    use myrtle_gamedata_raw::tables::sandbox_universe::{SandboxTemplate, SandboxUniverse};
    use serde_json::{Value, json};

    /// First rows of CN's `sandbox_perm_table`, both topics.
    fn cn_slice() -> Value {
        serde_json::from_str(include_str!("fixtures/sandbox_perm_cn_slice.json"))
            .expect("fixture parses")
    }

    #[test]
    fn every_basic_info_topic_is_read_under_its_own_template() {
        let u = SandboxUniverse::build(&cn_slice());
        let ids: Vec<(&str, &SandboxTemplate, i64)> = u
            .topics
            .iter()
            .map(|t| (t.topic_id.as_str(), &t.template, t.sort_id))
            .collect();
        assert_eq!(
            ids,
            vec![
                ("sandbox_1", &SandboxTemplate::V2, 1),
                ("sandbox_2", &SandboxTemplate::V3, 2),
            ]
        );

        let v3 = &u.topics[1].counts;
        assert_eq!(v3.max_achievements, 3, "AchievementData rows");
        assert_eq!(v3.max_nodes, 4, "MainMapData.Nodes without HOME");
        assert_eq!(v3.max_zones, 2, "zones of those nodes, not the HOME zone");
        assert_eq!(v3.node_zones["sandbox_2_node_04"], "z_2_2");
        assert_eq!(v3.max_tech_nodes, 4, "DevelopmentData");
        assert_eq!(v3.max_quests, 2, "ArchiveQuestData");
        assert_eq!(v3.max_stages, 2, "StageData");
        assert_eq!(v3.max_recipes, 2, "CookbookData");
        assert_eq!(v3.max_music, 2, "ArchiveMusicUnlockData");
        assert_eq!(v3.max_base_level, 3, "BasementUpdateDatas");
        assert_eq!(v3.max_blueprints, 0, "run-scoped recipes are not graded");
        assert_eq!(v3.max_band_levels, 6, "2 bands x 3 levels");
        assert_eq!(v3.max_rifts, 0, "V3 has no fixed rifts");
        assert_eq!(u.topics[1].name, "重启锚点", "BasicInfo.TopicName");
        assert!(v3.topics.is_empty());
    }

    #[test]
    fn flat_totals_stay_the_first_v2_topic() {
        let raw = cn_slice();
        let u = SandboxUniverse::build(&raw);
        // The same table without its V3 half: what the grader read before.
        let mut v2_only = raw;
        v2_only["Detail"]
            .as_object_mut()
            .expect("Detail")
            .remove("SANDBOX_V3");
        v2_only["BasicInfo"]
            .as_array_mut()
            .expect("BasicInfo")
            .truncate(1);
        let before = SandboxUniverse::build(&v2_only);

        let flat = |x: &SandboxUniverse| SandboxUniverse {
            topics: Vec::new(),
            ..x.clone()
        };
        assert_eq!(flat(&u), flat(&before), "V3 does not move a V2 total");
        assert_eq!(flat(&u), u.topics[0].counts, "flat fields are sandbox_1's");
        assert_eq!(u.max_nodes, 3, "the `_main` map only, not the hunt variant");
        assert_eq!(u.max_zones, 2, "tutorial zone excluded");
    }

    #[test]
    fn a_template_with_no_reader_is_listed_with_zero_totals() {
        let raw = json!({
            "BasicInfo": [{ "key": "sandbox_9", "value": {
                "TopicId": "sandbox_9", "TopicTemplate": "SANDBOX_V4", "SortId": 9,
            } }],
            "Detail": { "SANDBOX_V4": [{ "key": "sandbox_9", "value": { "AchievementData": [{}] } }] },
        });
        let u = SandboxUniverse::build(&raw);
        assert_eq!(u.topics.len(), 1);
        assert_eq!(
            u.topics[0].template,
            SandboxTemplate::Other("SANDBOX_V4".into())
        );
        assert_eq!(u.topics[0].counts, SandboxUniverse::default());
    }

    #[test]
    fn no_basic_info_falls_back_to_the_detail_entries() {
        let raw = json!({ "Detail": {
            "SANDBOX_V3": [{ "key": "sandbox_2", "value": { "AchievementData": [{}, {}] } }],
        } });
        let u = SandboxUniverse::build(&raw);
        assert_eq!(u.topics.len(), 1);
        assert_eq!(u.topics[0].topic_id, "sandbox_2");
        assert_eq!(u.topics[0].counts.max_achievements, 2);
    }

    #[test]
    fn max_zones_excludes_the_tutorial_zone() {
        let raw = json!({ "Detail": { "SANDBOX_V2": [ { "value": {
            "ZoneData": [
                { "key": "z_1_0" }, { "key": "z_1_1" }, { "key": "z_1_tutorial" }, { "key": "z_1_2" },
            ],
        } } ] } });
        let u = SandboxUniverse::build(&raw);
        assert_eq!(
            u.max_zones, 3,
            "the tutorial zone is excluded from the zone total"
        );
    }

    #[test]
    fn max_blueprints_does_not_double_count_buildings() {
        // `BuildingItemData` ids all reappear in `CraftItemData`; the total is
        // the `CraftItemData` count, never the sum of the two.
        let raw = json!({ "Detail": { "SANDBOX_V2": [ { "value": {
            "BuildingItemData": [
                { "key": "sandbox_1_building_1" }, { "key": "sandbox_1_building_2" },
            ],
            "CraftItemData": [
                { "key": "sandbox_1_building_1" }, { "key": "sandbox_1_building_2" },
                { "key": "sandbox_1_tactical_1" },
            ],
        } } ] } });
        let u = SandboxUniverse::build(&raw);
        assert_eq!(
            u.max_blueprints, 3,
            "blueprints total is the craftable set, not craft + building"
        );
    }
}
