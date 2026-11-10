use super::*;

#[test]
fn trading_post_finds_penguin_logistics_synergy() {
    let gd = load_game_data();

    // Texas's +65% only triggers with Lappland; Lappland has ~0 standalone
    // efficiency. The two distractors are plain +30% trading operators. A naive
    // optimizer that credits Texas unconditionally would pick
    // Texas + Exusiai + a distractor and drop Lappland. The correct optimum is
    // Texas + Lappland + Exusiai.
    let roster = vec![
        profile(gd, TEXAS),
        profile(gd, EXUSIAI),
        profile(gd, LAPPLAND),
        profile(gd, "char_502_nblade"), // flat +30%
        profile(gd, "char_185_frncat"), // flat +30%
    ];

    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let asn =
        compute_optimal_assignment(&roster, &trading_post(3), gd.building(), &registry, &drains);

    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post in the assignment");

    let has = |id: &str| tp.operators.iter().any(|o| o == id);

    assert!(
        has(TEXAS),
        "Texas should be selected. Got: {:?}",
        tp.operators
    );
    assert!(
        has(LAPPLAND),
        "Lappland MUST be selected to enable Texas's +65% (this is the synergy fix). Got: {:?}",
        tp.operators
    );
    assert!(
        has(EXUSIAI),
        "Exusiai (+35% flat) should round out the team. Got: {:?}",
        tp.operators
    );
}

#[test]
fn texas_without_lappland_is_not_overcredited() {
    let gd = load_game_data();

    // Roster has Texas but NOT Lappland, plus two flat +30% operators. Since
    // Texas's +65% is conditional on Lappland (absent here), her Feud buff must
    // contribute 0, so the two +30% operators should both beat her.
    let roster = vec![
        profile(gd, TEXAS),
        profile(gd, "char_502_nblade"),
        profile(gd, "char_185_frncat"),
    ];

    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    // Single-slot trading post (level 1) forces a choice; Texas (0 without
    // Lappland) must lose to a +30% operator.
    let asn =
        compute_optimal_assignment(&roster, &trading_post(1), gd.building(), &registry, &drains);
    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post");

    assert!(
        !tp.operators.iter().any(|o| o == TEXAS),
        "Texas's conditional +65% must NOT be credited without Lappland. Got: {:?}",
        tp.operators
    );
}

#[test]
fn shamare_nullifies_teammate_output() {
    // Shamare zeroes every teammate's efficiency, so a Shamare room's output
    // depends only on how many bodies fill it, NOT on who those bodies are.
    let gd = load_game_data();
    const SHAMARE: &str = "char_254_vodfox";

    // Strong trading teammates …
    let with_strong = vec![
        profile(gd, SHAMARE),
        profile(gd, EXUSIAI),           // +35%
        profile(gd, "char_4045_heidi"), // +35%
    ];
    // … vs operators with no trading skill at all (pure bodies).
    let with_bodies = vec![
        profile(gd, SHAMARE),
        profile(gd, "char_003_kalts"),  // no trading buff
        profile(gd, "char_180_amgoat"), // no trading buff
    ];

    let strong = trading_efficiency(gd, &with_strong);
    let bodies = trading_efficiency(gd, &with_bodies);

    assert!(
        (strong - bodies).abs() < 1.0,
        "Shamare should zero teammates -> identical output regardless of who they are \
         (strong={strong:.1}, bodies={bodies:.1})"
    );
    // And it must NOT be the naive sum (Exusiai 35 + Heidi 35 + Shamare ~90).
    assert!(
        strong < 120.0,
        "teammates' efficiency must be zeroed, got {strong:.1}"
    );
}

#[test]
fn proviso_is_credited_as_a_strong_gold_trader() {
    // Proviso's Pure-Gold payoff raises LMD per ORDER (value), not order speed,
    // so it shows up in the LMD yield, not the efficiency %. By LMD she should
    // rival or beat a strong flat (speed) trader.
    let gd = load_game_data();
    let proviso = trading_lmd(gd, &[profile(gd, "char_4032_provs")]);
    let flat = trading_lmd(gd, &[profile(gd, "char_502_nblade")]); // +30% flat speed
    assert!(
        proviso >= flat,
        "Proviso's order value should rival a strong flat trader by LMD (proviso {proviso:.0} vs flat {flat:.0})"
    );
    // And her order VALUE must NOT inflate the displayed efficiency %.
    let proviso_eff = trading_efficiency(gd, &[profile(gd, "char_4032_provs")]);
    assert!(
        proviso_eff < 5.0,
        "Proviso's value must not show as order-speed efficiency, got +{proviso_eff:.1}%"
    );
}

#[test]
fn proviso_value_does_not_survive_shamare() {
    // Shamare shifts the post toward Precious-Metal orders, so Proviso's Pure-Gold
    // value no longer applies in her team. A Shamare + Proviso post reads only
    // Shamare's own ~10% value, NOT Proviso's +55%; she is wasted with Shamare and
    // belongs in a fast post instead.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let asn = compute_optimal_assignment(
        &[
            profile(gd, "char_254_vodfox"),
            profile(gd, "char_4032_provs"),
        ],
        &trading_post(3),
        gd.building(),
        &registry,
        &drains,
    );
    let tp = asn.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    assert!(
        tp.order_value < 20.0,
        "Proviso's +55% Pure-Gold value must be nullified by Shamare, got {:.0}",
        tp.order_value
    );
}

/// Jaye beside Texas and Lappland reads the game's figure: Feud +65 (with
/// Lappland), Lappland's Hidden Purpose β +4 order limit (with Texas); Basic
/// Needs cuts the limit by 1 per 10% the others provide (6), so the post
/// holds 10 + 4 - 6 = 8 orders and Jaye's two riders pay 4% each of them:
/// 65 + 32 = 97. Eight orders between collections is more than the post
/// fills, so the optimizer keeps him.
#[test]
fn jaye_order_limit_efficiency_is_counted() {
    let gd = load_game_data();
    let team = vec![
        profile(gd, TEXAS),
        profile(gd, LAPPLAND),
        profile(gd, "char_272_strong"), // Jaye
    ];
    let eff = trading_efficiency(gd, &team);
    assert!(
        (eff - 97.0).abs() < 0.01,
        "Texas+Lappland+Jaye should read +97% (65 + 4 x 8), got +{eff:.1}%"
    );
}

/// Jaye's cut is what bounds him: beside Exusiai (+35) and Lemuen (+20, +25
/// more with Exusiai) he cuts the limit by 8 and the post holds 2 orders, so
/// the displayed figure is 80 + 4 x 2 = 88, and a 2-order buffer sells four
/// orders a day, so the optimizer benches him and runs the pair.
#[test]
fn jaye_is_benched_when_his_cut_starves_the_order_buffer() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    const JAYE: &str = "char_272_strong";
    const LEMUEN: &str = "char_4193_lemuen";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let roster = vec![profile(gd, JAYE), profile(gd, EXUSIAI), profile(gd, LEMUEN)];
    let mut building = trading_post(3);
    for r in &mut building.rooms {
        if r.room_type == "TRADING" {
            r.current_operators = vec![JAYE.into(), EXUSIAI.into(), LEMUEN.into()];
        }
    }
    let seated =
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
    let tp = seated
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .unwrap();
    assert!(
        (tp.total_efficiency - 88.0).abs() < 0.01,
        "Jaye/Exusiai/Lemuen read +88% (80 + 4 x 2), got +{:.1}%",
        tp.total_efficiency
    );
    assert_eq!(tp.order_limit, Some(2), "10 - floor(80 / 10)");
    let opt = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let otp = opt.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    assert!(
        !otp.operators.iter().any(|o| o == JAYE),
        "a 2-order buffer benches Jaye, got {:?}",
        otp.operators
    );
    assert_eq!(
        otp.operators.len(),
        2,
        "the pair runs the post: {:?}",
        otp.operators
    );
}

#[test]
fn lemuen_conditional_bonus_with_exusiai_is_counted() {
    // Lemuen's "Amicus": base +20% order efficiency, PLUS +25% more when Exusiai
    // shares the post, phrased "if <Exusiai> is assigned to the same Trading
    // Post" (name BEFORE "same"), unlike Texas's "...same Post as <Lappland>".
    // The +25% was once dropped, leaving the team at ~+85% (reported as ~92%).
    // With it counted: Lemuen 45 + Exusiai 35 + Quartz 30 = 110.
    let gd = load_game_data();
    let team = vec![
        profile(gd, "char_4193_lemuen"),
        profile(gd, EXUSIAI),
        profile(gd, "char_4063_quartz"),
    ];
    let eff = trading_efficiency(gd, &team);
    assert!(
        eff > 105.0,
        "Lemuen+Exusiai+Quartz should read ~+110% (Lemuen's +25% Exusiai bonus counted), got +{eff:.1}%"
    );

    // The +25% is conditional: drop Exusiai and Lemuen keeps only her
    // base +20%, so the same two-trader pairing must read materially lower.
    let no_exu = trading_efficiency(
        gd,
        &[
            profile(gd, "char_4193_lemuen"),
            profile(gd, "char_4063_quartz"),
        ],
    );
    assert!(
        eff > no_exu + 20.0,
        "Exusiai should unlock Lemuen's +25% (with {eff:.1}% vs without {no_exu:.1}%)"
    );
}

#[test]
fn morgan_glasgow_count_carries_a_siege_rider() {
    // Morgan "Gang Compass": +20% per Glasgow Gang op in the post, AND +35% more
    // "when in the same Trading Post as Siege". The faction count is the dominant
    // term (MatchCountScaling), but the named Siege rider must ride along on top.

    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _drains) = build_registry(&gd.building().buffs, &name_to_char);
    let siege_id = name_to_char.get("siege").cloned();

    match registry.get("trade_ord_spd_par[000]") {
        Some(BuffResolutionStrategy::MatchCountScaling {
            token,
            per_match_pct,
            bonus_char_id,
            bonus_pct,
            ..
        }) => {
            assert_eq!(token, "glasgow", "counts Glasgow Gang operators");
            assert!(
                (*per_match_pct - 20.0).abs() < 0.01,
                "per Glasgow op +20%, got {per_match_pct}"
            );
            assert_eq!(
                bonus_char_id.as_deref(),
                siege_id.as_deref(),
                "rider gated on Siege"
            );
            assert!(
                (*bonus_pct - 35.0).abs() < 0.01,
                "Siege rider +35%, got {bonus_pct}"
            );
        }
        _ => panic!("Morgan's Gang Compass should be MatchCountScaling with a Siege rider"),
    }
}

#[test]
fn faction_conditional_credits_base_plus_glasgow_bonus() {
    // Vina Victoria "Resolution on Foreign Trade β": base +30% trading, plus +10%
    // more "if a Glasgow Gang Operator is assigned to the same Trading Post". The
    // faction analogue of a named-teammate conditional: base always on, bonus
    // gated on a faction tag rather than one operator.

    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _drains) = build_registry(&gd.building().buffs, &name_to_char);

    match registry.get("trade_ord_spd&par[001]") {
        Some(BuffResolutionStrategy::ConditionalOnFaction {
            faction_token,
            base_efficiency,
            efficiency,
        }) => {
            assert_eq!(
                faction_token, "glasgow",
                "bonus gated on Glasgow Gang presence"
            );
            assert!(
                (*base_efficiency - 30.0).abs() < 0.01,
                "base +30%, got {base_efficiency}"
            );
            assert!(
                (*efficiency - 10.0).abs() < 0.01,
                "Glasgow bonus +10%, got {efficiency}"
            );
        }
        _ => panic!("Resolution β should be a ConditionalOnFaction (base + Glasgow bonus)"),
    }
}

#[test]
fn morgan_reads_higher_with_siege_in_the_post() {
    // End-to-end: with Siege sharing the post, Morgan's +35% rider fires, so the
    // team must out-produce the same post with Siege swapped for another Glasgow
    // operator (which keeps the faction count but loses the rider).
    let gd = load_game_data();
    const MORGAN: &str = "char_154_morgan";
    const SIEGE: &str = "char_112_siege";
    const INDRA: &str = "char_155_tiger";
    const MATTERHORN: &str = "char_199_yak";

    let with_siege = trading_efficiency(
        gd,
        &[profile(gd, MORGAN), profile(gd, SIEGE), profile(gd, INDRA)],
    );
    let without_siege = trading_efficiency(
        gd,
        &[
            profile(gd, MORGAN),
            profile(gd, MATTERHORN),
            profile(gd, INDRA),
        ],
    );
    assert!(
        with_siege > without_siege,
        "Siege's presence should fire Morgan's +35% rider (with Siege {with_siege:.1}% vs without {without_siege:.1}%)"
    );
}

#[test]
fn shamare_money_printer_forms_despite_a_deep_distractor_pool() {
    // Shamare nullifies teammates' SPEED, so her only useful partners are
    // order-VALUE operators (Tequila, Bibeak) whose LMD-per-order survives. In a
    // deep roster (18 candidates > the 16-candidate cut) those low-speed value
    // operators would normally be dropped; the nullifier value-boost keeps them,
    // so Shamare + Tequila + Bibeak still forms instead of Shamare + dead bodies.
    let gd = load_game_data();
    const SHAMARE: &str = "char_254_vodfox";
    const TEQUILA: &str = "char_486_takila";
    const BIBEAK: &str = "char_252_bibeak";
    // 15 flat trading operators, distractors that crowd the candidate set.
    let distractors = [
        "char_502_nblade",
        "char_123_fang",
        "char_211_adnach",
        "char_240_wyvern",
        "char_110_deepcl",
        "char_141_nights",
        "char_150_snakek",
        "char_185_frncat",
        "char_289_gyuki",
        "char_101_sora",
        "char_103_angel",
        "char_302_glaze",
        "char_4045_heidi",
        "char_1033_swire2",
        "char_4163_rosesa",
    ];
    let mut roster = vec![
        profile(gd, SHAMARE),
        profile(gd, TEQUILA),
        profile(gd, BIBEAK),
    ];
    roster.extend(distractors.iter().map(|d| profile(gd, d)));

    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let asn =
        compute_optimal_assignment(&roster, &trading_post(3), gd.building(), &registry, &drains);
    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post");
    let has = |id: &str| tp.operators.iter().any(|o| o == id);

    assert!(
        has(SHAMARE),
        "Shamare should anchor the post. Got: {:?}",
        tp.operators
    );
    assert!(
        has(TEQUILA) && has(BIBEAK),
        "Shamare's order-value partners (Tequila + Bibeak) must survive the candidate cut. Got: {:?}",
        tp.operators
    );
}

#[test]
fn degenbrecher_is_not_used_in_a_trading_post_without_cap_synergy() {
    // Degenbrecher gives +25% speed but slashes the order limit by 6 ("minimum 1"),
    // and her payoff clause needs teammates who ADD order limit, which trading
    // posts don't field. So she cripples a normal post and must be left out in
    // favour of ordinary traders (the seat stays empty before she's forced in).
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);

    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("tp", "TRADING", 3)],
    };
    let roster = vec![
        profile(gd, "char_4116_blkkgt"), // Degenbrecher (order-limit wrecker)
        profile(gd, "char_103_angel"),   // Exusiai, trader
        profile(gd, "char_102_texas"),   // Texas, trader
    ];

    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("a trading post");
    assert!(
        !tp.operators.iter().any(|o| o == "char_4116_blkkgt"),
        "Degenbrecher should not be staffed in a normal trading post. Got: {:?}",
        tp.operators
    );
    assert!(
        tp.total_efficiency > 0.0,
        "the trading post should keep its positive efficiency, got {:.1}",
        tp.total_efficiency
    );
}

#[test]
fn an_idle_gold_factory_bounds_its_posts_instead_of_selling_from_stock() {
    // A base whose gold factory is set but UNSTAFFED makes no gold: its posts
    // sell nothing, they do not fall back to stock the way a base that runs
    // no gold factory at all does. The stationed reading used to drop idle
    // rooms before pricing, so two idle gold factories read as "runs no gold"
    // and the posts priced unbounded (32537844: a stationed value of 64,903
    // against a plan of 58,833 for the same rooms, 25,440 once priced right).
    use myrtle_engine_base::assignment::{compute_current_assignment, sustained_assignment_value};
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let roster = vec![
        profile(gd, TEXAS),
        profile(gd, LAPPLAND),
        profile(gd, "char_123_fang"),
    ];
    let stationed = |factory: Option<Vec<&str>>| {
        let mut post = room("tp", "TRADING", 3);
        post.current_operators = vec![TEXAS.into(), LAPPLAND.into()];
        let mut rooms = vec![post, room("d0", "DORMITORY", 5)];
        if let Some(crew) = factory {
            let mut mf = room("mf", "MANUFACTURE", 3);
            mf.current_formula = Some("F_GOLD".into());
            mf.current_operators = crew.iter().map(|s| (*s).to_string()).collect();
            rooms.push(mf);
        }
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        sustained_assignment_value(&asn, &roster, &building, gd.building(), &registry, &drains)
    };
    let from_stock = stationed(None);
    let idle = stationed(Some(vec![]));
    let fed = stationed(Some(vec!["char_123_fang"]));
    assert!(
        from_stock > 0.0,
        "a post with no gold factory sells from stock"
    );
    assert!(
        idle.abs() < 1e-9,
        "a post beside an idle gold factory sells nothing, got {idle:.1}"
    );
    assert!(
        fed > 0.0 && fed < from_stock,
        "a staffed gold factory bounds the post below stock: fed {fed:.1} vs stock {from_stock:.1}"
    );
}

#[test]
fn a_dominant_value_shape_does_not_hide_the_proviso_free_level_three_crew() {
    // Each post shortlists its best crews by uncoupled score. On a three-seat
    // post every top crew carried Proviso (+55% there), so the joint pick
    // never saw a Proviso-free level-3 crew and could not seat her at the
    // level-2 post where she pays on every order. The shortlist now also
    // keeps, for each operator of a post's best crew, the best crew without
    // them; the coupled value then seats Shamare/Tequila at the level-3 post
    // and Proviso at the level-2 one, the way the community runs it, and
    // the plan is worth at least what that player's own arrangement is
    // (user report 2026-10-08: their setup out-earned the plan by 7%).
    use myrtle_engine_base::assignment::{compute_current_assignment, sustained_assignment_value};
    use myrtle_engine_base::scoring::pools::{optimal_with_bundles, search_economy};
    const SHAMARE: &str = "char_254_vodfox";
    const TEQUILA: &str = "char_486_takila";
    const PROVISO: &str = "char_4032_provs";
    const QUARTZ: &str = "char_4063_quartz";
    const BIBEAK: &str = "char_252_bibeak";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let roster: Vec<OperatorBaseProfile> = [
        SHAMARE,
        TEQUILA,
        PROVISO,
        QUARTZ,
        "char_300_phenxi",
        TEXAS,
        LAPPLAND,
        EXUSIAI,
        "char_272_strong",
        "char_4193_lemuen",
        BIBEAK,
        "char_196_sunbr",
        "char_283_midn",
        "char_237_gravel",
        "char_141_nights",
        "char_240_wyvern",
        "char_235_jesica",
        "char_290_vigna",
        "char_193_frostl",
        "char_1502_crosly",
        "char_1032_excu2",
        "char_123_fang",
        "char_133_mm",
        "char_181_flower",
        "char_128_plosis",
        "char_002_amiya",
        "char_2027_wang",
        "char_107_liskam",
        "char_285_medic2",
        "char_1011_lava2",
        "char_159_peacok",
        "char_215_mantic",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    let factory = |slot: &str, formula: &str| {
        let mut r = room(slot, "MANUFACTURE", 3);
        r.current_formula = Some(formula.into());
        r
    };
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("cc", "CONTROL", 5),
            room("tp3", "TRADING", 3),
            room("tp2", "TRADING", 2),
            factory("mf0", "F_GOLD"),
            factory("mf1", "F_GOLD"),
            factory("mf2", "F_EXP"),
            room("pp0", "POWER", 3),
            room("d0", "DORMITORY", 5),
            room("d1", "DORMITORY", 5),
        ],
    };
    let economy = search_economy(&roster, &building, gd.building(), &registry);
    let plan = optimal_with_bundles(
        &roster,
        &building,
        gd.building(),
        &registry,
        &economy.registry,
        &drains,
        &economy.pins,
    )
    .optimal;
    let crew = |slot: &str| -> Vec<String> {
        plan.rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .map(|r| r.operators.clone())
            .unwrap_or_default()
    };
    let (tp3, tp2) = (crew("tp3"), crew("tp2"));
    assert!(
        tp2.iter().any(|o| o == PROVISO),
        "Proviso sells at the level-2 post: tp3 {tp3:?} tp2 {tp2:?}"
    );
    assert!(
        tp3.iter().any(|o| o == SHAMARE) && tp3.iter().any(|o| o == TEQUILA),
        "Shamare and Tequila sell at the level-3 post: tp3 {tp3:?}"
    );
    // The player's own arrangement on the plan's factories: the plan is worth
    // at least as much.
    let value = |b: &UserBuilding, a: &myrtle_engine_base::types::BaseAssignment| {
        sustained_assignment_value(a, &roster, b, gd.building(), &registry, &drains)
    };
    let mut theirs = building.clone();
    for r in &mut theirs.rooms {
        r.current_operators = match r.slot_id.as_str() {
            "tp3" => vec![SHAMARE.into(), TEQUILA.into(), BIBEAK.into()],
            "tp2" => vec![PROVISO.into(), QUARTZ.into()],
            slot => crew(slot)
                .into_iter()
                .filter(|o| ![SHAMARE, TEQUILA, BIBEAK, PROVISO, QUARTZ].contains(&o.as_str()))
                .collect(),
        };
    }
    let theirs_scored =
        compute_current_assignment(&roster, &theirs, gd.building(), &registry, &drains, None);
    let (ours, player) = (value(&building, &plan), value(&theirs, &theirs_scored));
    assert!(
        ours >= player * 0.999,
        "the plan is worth at least the community arrangement: ours {ours:.0} vs theirs {player:.0}"
    );
}

#[test]
fn proviso_and_tequila_in_a_level_two_post_read_proviso_alone() {
    // Order value follows the post's order mix. A level-2 post draws only 2- and
    // 3-gold orders: every one is "defaulted", so Proviso's +2 gold fires on all
    // of them (+83%), while Tequila's "+500 LMD above 3 gold" has no 4-gold order
    // to land on and adds exactly 0. The post reads Proviso alone, not a sum.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    // Level-2 post (2 seats) forces both value operators in together.
    let asn = compute_optimal_assignment(
        &[
            profile(gd, "char_4032_provs"),
            profile(gd, "char_486_takila"),
        ],
        &trading_post(2),
        gd.building(),
        &registry,
        &drains,
    );
    let tp = asn.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    let expected = (2200.0 / 1200.0 - 1.0) * 100.0;
    assert!(
        (tp.order_value - expected).abs() < 1e-6,
        "a level-2 post reads Proviso's +83% with nothing from Tequila, got {:.1}",
        tp.order_value
    );
}

#[test]
fn proviso_pairs_with_speed_not_another_value_operator() {
    // Tequila only adds value on the 4-gold fifth of a level-3 post's orders that
    // Proviso leaves alone (~+7 points of value), which loses to any +30% speed
    // partner. The optimizer must staff Proviso with the two fastest speed traders
    // and leave Tequila out.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let roster = vec![
        profile(gd, "char_4032_provs"), // Proviso (value)
        profile(gd, "char_486_takila"), // Tequila (value)
        profile(gd, "char_103_angel"),  // Exusiai +35 speed
        profile(gd, "char_502_nblade"), // Yato +30 speed
        profile(gd, "char_185_frncat"), // Mousse +30 speed
    ];
    let asn =
        compute_optimal_assignment(&roster, &trading_post(3), gd.building(), &registry, &drains);
    let tp = asn.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    assert!(
        tp.operators.iter().any(|o| o == "char_4032_provs"),
        "Proviso should staff the post. Got: {:?}",
        tp.operators
    );
    assert!(
        !tp.operators.iter().any(|o| o == "char_486_takila"),
        "Tequila's sliver of value loses to a speed partner and must be left out. Got: {:?}",
        tp.operators
    );
}

#[test]
fn proviso_is_not_staffed_in_a_shamare_post() {
    // Proviso's Pure-Gold value is nullified by Shamare, so she belongs in her OWN
    // post. With two posts and no Texas, the optimizer must NOT put Proviso and
    // Shamare in the same post (the bad "Lappland + Proviso + Shamare" team); Proviso
    // gets her own post, and Shamare's post is staffed with surviving-value/filler ops.
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("t0", "TRADING", 3),
            room("t1", "TRADING", 3),
            room("d", "DORMITORY", 3),
        ],
    };
    let roster = vec![
        profile(gd, "char_140_whitew"), // Lappland (needs Texas, absent)
        profile(gd, "char_4032_provs"), // Proviso
        profile(gd, "char_254_vodfox"), // Shamare
        profile(gd, "char_486_takila"), // Tequila
        profile(gd, "char_003_kalts"),  // body
        profile(gd, "char_180_amgoat"), // body
    ];
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let same_post = asn.rooms.iter().any(|r| {
        r.room_type == "TRADING"
            && r.operators.iter().any(|o| o == "char_4032_provs")
            && r.operators.iter().any(|o| o == "char_254_vodfox")
    });
    assert!(
        !same_post,
        "Proviso must not share a post with Shamare. Rooms: {:?}",
        asn.rooms
            .iter()
            .filter(|r| r.room_type == "TRADING")
            .map(|r| (&r.slot_id, &r.operators))
            .collect::<Vec<_>>()
    );
}

/// Counts include the holder (user-verified 2026-09-10 on uid 44947595):
/// Mizuki's "+5% for every Standardization Skill in the same Factory" counts
/// his own Standardization β (3 -> +15 beside Vanilla and Jessica); Morgan's
/// Gang Compass counts herself as a Glasgow Gang member (2 x 20 + 35 beside
/// Siege = 75; Siege has no trading skill of her own).
#[test]
fn room_counts_include_the_holder_mizuki_and_morgan() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const MIZUKI: &str = "char_437_mizuki";
    const VANILLA: &str = "char_240_wyvern";
    const JESSICA: &str = "char_235_jesica";
    const MORGAN: &str = "char_154_morgan";
    const SIEGE: &str = "char_112_siege";
    let mut rooms = vec![room("mf", "MANUFACTURE", 3), room("tp", "TRADING", 3)];
    rooms[0].current_operators = vec![VANILLA.into(), JESSICA.into(), MIZUKI.into()];
    rooms[0].current_formula = Some("F_EXP".into());
    rooms[1].current_operators = vec![MORGAN.into(), SIEGE.into()];
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let roster: Vec<_> = [MIZUKI, VANILLA, JESSICA, MORGAN, SIEGE]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn =
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
    let of = |slot: &str| {
        asn.rooms
            .iter()
            .find(|r| r.slot_id == slot)
            .map_or(0.0, |r| r.total_efficiency)
    };
    assert!(
        (of("mf") - 90.0).abs() < 1e-9,
        "25 x 3 + 5 x 3 Standardization skills: {}",
        of("mf")
    );
    assert!(
        (of("tp") - 75.0).abs() < 1e-9,
        "Morgan 20 x 2 Glasgow + 35 with Siege: {}",
        of("tp")
    );
}

/// The Shamare / Bibeak / Tequila post beats the speed teams once the ranker
/// prices Bibeak's Tailoring beside Tequila: a sliver alone, +17 next to his
/// rider, so she survives the candidate cut and the trio forms (user report
/// 2026-09-13: Texas / Lappland / Kichisei and Exusiai / Exusiai / Lemuen
/// were recommended instead).
#[test]
fn shamare_bibeak_tequila_form_the_post_on_a_deep_roster() {
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const SHAMARE: &str = "char_254_vodfox";
    const BIBEAK: &str = "char_252_bibeak";
    const TEQUILA: &str = "char_486_takila";
    let ids = [
        SHAMARE,
        BIBEAK,
        TEQUILA,
        "char_102_texas",
        "char_1028_texas2",
        "char_4203_kichi",
        "char_1029_yato2",
        EXUSIAI,
        "char_1041_angel2",
        "char_4193_lemuen",
        "char_214_kafka",
        "char_427_vigil",
        "char_308_swire",
        "char_1033_swire2",
        "char_172_svrash",
    ];
    let roster: Vec<_> = ids.iter().map(|id| profile(gd, id)).collect();
    let mut rooms = vec![room("tp", "TRADING", 3), room("mf", "MANUFACTURE", 3)];
    rooms.extend((0..2).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("the post");
    for id in [SHAMARE, BIBEAK, TEQUILA] {
        assert!(
            tp.operators.iter().any(|o| o == id),
            "{id} in the post: {:?}",
            tp.operators
        );
    }
}

/// The feedback sheet's daily-LMD cases (Annex 3): a level-3 post's base
/// rate is its order mix (2.9 gold per 203.4 min), every slotted operator
/// adds an innate +1%, and the Control Center's +7% rides on top.
/// Shamare E2 / Bibeak E2 / Tequila E2 + Amiya = 25,479; Proviso E2 /
/// Archetto / Vigil = 30,265.
#[test]
fn daily_lmd_matches_the_feedback_sheets_squads() {
    use myrtle_engine_base::scoring::yield_model::room_yield;
    // Shamare 90 + innate 3 + CC 7 = 100 displayed; value 24.1 (Tailoring β + Tequila β).
    let y = room_yield("TRADING", None, 3, 97.0, 24.1025, 3, None);
    assert!(
        (y.lmd_per_day - 25479.0).abs() < 60.0,
        "Shamare squad: {}",
        y.lmd_per_day
    );
    // Proviso 55.2 value; Archetto 40 + Vigil 40 + CC 7 = 87 displayed.
    let y = room_yield("TRADING", None, 3, 87.0, 55.2, 3, None);
    assert!(
        (y.lmd_per_day - 30265.0).abs() < 300.0,
        "Proviso squad: {}",
        y.lmd_per_day
    );
}

/// The Kjerag/Gnosis trading-post arithmetic: six level-3 posts measured
/// in-game (community sheet, Annex 1). `SilverAsh` +20% / +4; Degenbrecher
/// +25% / -6 plus +25% per 5 limit points provided by others (max 100); Swire
/// the Elegant Wit +20% plus +4% per limit point provided by others; Jaye -1
/// limit per 10% the others provide and +4% per order of the FINAL limit
/// (Street Economics per empty slot, Basic Needs per filled order); Gnosis in
/// the Control Center gives Kjerag traders -15% and +6. The readers resolve in
/// stages: Degenbrecher on the fixed limits, Jaye's cut on the efficiency so
/// far, Swire on the limit after the cut. A mutual fixed point would read 104
/// and 113 for the two Jaye+Gnosis posts, not 121 and 129. The "(minimum 1)"
/// floors the ROOM limit, never a single skill.
#[test]
fn kjerag_posts_read_the_games_six_measured_figures() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    const SWIRE2: &str = "char_1033_swire2";
    const SA: &str = "char_172_svrash";
    const DEGEN: &str = "char_4116_blkkgt";
    const JAYE: &str = "char_272_strong";
    const GNOSIS: &str = "char_206_gnosis";
    // (crew, Gnosis in the CC, efficiency %, final order limit)
    let cases: Vec<(&str, Vec<&str>, bool, f64, i32)> = vec![
        ("Swire/SA/Degen", vec![SWIRE2, SA, DEGEN], false, 65.0, 8),
        (
            "Swire/SA/Degen +Gnosis",
            vec![SWIRE2, SA, DEGEN],
            true,
            125.0,
            20,
        ),
        ("Jaye/Swire/SA", vec![JAYE, SWIRE2, SA], false, 80.0, 10),
        (
            "Jaye/Swire/SA +Gnosis",
            vec![JAYE, SWIRE2, SA],
            true,
            129.0,
            18,
        ),
        ("Jaye/SA/Degen", vec![JAYE, SA, DEGEN], false, 61.0, 4),
        (
            "Jaye/SA/Degen +Gnosis",
            vec![JAYE, SA, DEGEN],
            true,
            121.0,
            14,
        ),
    ];
    for (label, crew, gnosis, expect, limit) in cases {
        let mut ids = crew.clone();
        if gnosis {
            ids.push(GNOSIS);
        }
        let roster: Vec<_> = ids.iter().map(|id| profile(gd, id)).collect();
        let mut rooms = vec![room("tp", "TRADING", 3), room("cc", "CONTROL", 5)];
        rooms[0].current_operators = crew.iter().map(|s| (*s).to_string()).collect();
        if gnosis {
            rooms[1].current_operators = vec![GNOSIS.into()];
        }
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        let tp = asn.rooms.iter().find(|r| r.slot_id == "tp").unwrap();
        assert!(
            (tp.total_efficiency - expect).abs() < 0.01,
            "{label}: ours {:.2} expected {expect}",
            tp.total_efficiency
        );
        assert_eq!(tp.order_limit, Some(limit), "{label}: order limit");
    }
    // The parses behind the figures.
    assert!(
        matches!(
            registry.get("control_tra_limit&spd[000]"),
            Some(BuffResolutionStrategy::ConditionalGlobalEffect {
                bonus_pct,
                order_limit: 6,
                per_operator: true,
                ..
            }) if (*bonus_pct + 15.0).abs() < 1e-9
        ),
        "Gnosis: {:?}",
        registry.get("control_tra_limit&spd[000]")
    );
    assert!(
        matches!(
            registry.get("trade_ord_limit_count[000]"),
            Some(BuffResolutionStrategy::LimitCutPerPeerEfficiency {
                pct_per_cut,
                cut: -1,
                per_order_pct,
            }) if (*pct_per_cut - 10.0).abs() < 1e-9 && (*per_order_pct - 4.0).abs() < 1e-9
        ),
        "Jaye E1: {:?}",
        registry.get("trade_ord_limit_count[000]")
    );
}

/// A mixed-level base prices each post at ITS level: the level-3 post keeps its
/// 10-order limit and rarity-3 order value while the level-2 post beside it
/// reads 8 and rarity 2. Every post used to be priced at the base-wide
/// minimum, so a level-2 post made the level-3 one read as level 2 too.
#[test]
fn mixed_level_posts_price_each_post_at_its_own_level() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    const PROVISO: &str = "char_4032_provs";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let roster = vec![profile(gd, PROVISO)];
    let seated_in = |slot: &str| {
        let mut building = UserBuilding {
            drone_capacity: None,
            rooms: vec![room("tp3", "TRADING", 3), room("tp2", "TRADING", 2)],
        };
        for r in &mut building.rooms {
            if r.slot_id == slot {
                r.current_operators = vec![PROVISO.to_string()];
            }
        }
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None)
    };
    let alone_at = |level: i32| {
        let mut building = UserBuilding {
            drone_capacity: None,
            rooms: vec![room("tp", "TRADING", level)],
        };
        building.rooms[0].current_operators = vec![PROVISO.to_string()];
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .into_iter()
            .find(|r| r.room_type == "TRADING")
            .unwrap()
    };
    let mixed = seated_in("tp3");
    let tp3 = mixed.rooms.iter().find(|r| r.slot_id == "tp3").unwrap();
    let tp2 = mixed.rooms.iter().find(|r| r.slot_id == "tp2").unwrap();
    assert_eq!(tp3.order_limit, Some(10), "a level-3 post holds 10 orders");
    assert_eq!(tp2.order_limit, Some(8), "a level-2 post holds 8 orders");
    let solo3 = alone_at(3);
    assert!(
        (tp3.order_value - solo3.order_value).abs() < 1e-9,
        "Proviso in the level-3 post is priced at rarity 3 beside a level-2 post: {} vs {}",
        tp3.order_value,
        solo3.order_value
    );
    let mixed2 = seated_in("tp2");
    let tp2 = mixed2.rooms.iter().find(|r| r.slot_id == "tp2").unwrap();
    let solo2 = alone_at(2);
    assert!(
        (tp2.order_value - solo2.order_value).abs() < 1e-9,
        "Proviso in the level-2 post is priced at rarity 2: {} vs {}",
        tp2.order_value,
        solo2.order_value
    );
    assert!(
        (solo3.order_value - solo2.order_value).abs() > 1e-6,
        "the two rarities must differ for the check to mean anything"
    );
}

/// A trading post sells what its order buffer holds between collections. The
/// buffer is the level's base limit (6/8/10 for L1-L3, gamedata
/// `TradingData.Phases`) plus the crew's deltas, floored at 1 for the ROOM: so
/// Degenbrecher's -6 leaves an L3 post 4 orders and an L1 post its floor of 1.
/// The displayed efficiency (+25%) is the game's, untouched by the limit; the
/// cost lands in the yield, and a limit-bound post's output no longer depends
/// on its speed.
#[test]
fn trading_order_cap_scales_with_post_level() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    use myrtle_engine_base::scoring::yield_model::room_yield;
    const DEGENBRECHER: &str = "char_4116_blkkgt";
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let roster = vec![profile(gd, DEGENBRECHER)];
    let post_at = |level: i32| {
        let mut building = UserBuilding {
            drone_capacity: None,
            rooms: vec![room("tp0", "TRADING", level)],
        };
        building.rooms[0].current_operators = vec![DEGENBRECHER.to_string()];
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .into_iter()
            .find(|r| r.room_type == "TRADING")
            .expect("a trading post")
    };
    let l3 = post_at(3);
    let l1 = post_at(1);
    assert!(
        (l3.total_efficiency - 25.0).abs() < 1e-6,
        "the displayed figure is the game's +25%, got {}",
        l3.total_efficiency
    );
    assert_eq!(l3.order_limit, Some(4), "L3: 10 - 6");
    assert_eq!(
        l1.order_limit,
        Some(1),
        "L1: 6 - 6 floors at the room minimum of 1"
    );
    let lmd = |level: i32, speed: f64, limit: Option<i32>| {
        room_yield("TRADING", None, level, speed, 0.0, 1, limit).lmd_per_day
    };
    let l3_capped = lmd(3, l3.total_efficiency, l3.order_limit);
    let l1_capped = lmd(1, l1.total_efficiency, l1.order_limit);
    assert!(
        l3_capped <= lmd(3, l3.total_efficiency, None),
        "a 4-order buffer never sells more than the free rate"
    );
    assert!(
        lmd(3, 200.0, Some(4)) < lmd(3, 200.0, None),
        "at +200% four orders between collections throttle an L3 post"
    );
    assert!(
        l1_capped < l3_capped,
        "the 1-order floor is stricter still: {l1_capped} vs {l3_capped}"
    );
    assert!(
        (lmd(1, 1000.0, Some(1)) - l1_capped).abs() < 1e-6,
        "a limit-bound post sells its buffer whatever its speed"
    );
}

/// Order VALUE follows the post's level. Proviso's "+2 gold on orders below
/// 4" fires on every order a level-1 post draws (2-gold only: +100%), on
/// every order a level-2 post draws (2- and 3-gold: +83%), but leaves the
/// 4-gold orders only a level-3 post draws untouched (+55%), which is why
/// she shines in a 2/5/2 base's level-2 posts. Tequila's "+500 LMD above 3
/// gold" needs the 4-gold orders, so it is worth 0 below level 3 and
/// composes with Proviso at level 3 on the orders she leaves alone.
#[test]
fn order_value_follows_the_posts_level() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    const PROVISO: &str = "char_4032_provs";
    const TEQUILA: &str = "char_486_takila";
    let order_value = |crew: &[&str], level: i32| -> f64 {
        let mut building = UserBuilding {
            drone_capacity: None,
            rooms: vec![room("tp", "TRADING", level)],
        };
        building.rooms[0].current_operators = crew.iter().map(|s| (*s).to_string()).collect();
        let roster: Vec<OperatorBaseProfile> = crew.iter().map(|id| profile(gd, id)).collect();
        let asn =
            compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
        asn.rooms
            .iter()
            .find(|r| r.slot_id == "tp")
            .expect("post")
            .order_value
    };
    let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
    assert!(
        close(order_value(&[PROVISO], 1), 100.0),
        "L1: {}",
        order_value(&[PROVISO], 1)
    );
    assert!(
        close(order_value(&[PROVISO], 2), (2200.0 / 1200.0 - 1.0) * 100.0),
        "L2: {}",
        order_value(&[PROVISO], 2)
    );
    assert!(
        close(order_value(&[PROVISO], 3), (2250.0 / 1450.0 - 1.0) * 100.0),
        "L3: {}",
        order_value(&[PROVISO], 3)
    );
    assert!(
        close(order_value(&[TEQUILA], 2), 0.0),
        "Tequila needs 4-gold orders"
    );
    assert!(
        order_value(&[PROVISO, TEQUILA], 3) > order_value(&[PROVISO], 3),
        "Tequila composes with Proviso on the 4-gold orders"
    );
}

/// A gold-starved base sells what its factories make, so a post is worth
/// what it earns PER BAR: beside one gold factory the Shamare/Tequila/Bibeak
/// squad (speed plus Tequila's LMD rider) out-earns Proviso's squad, whose
/// bonus bars come from stock the base does not have. Beside three gold
/// factories the supply is there and Proviso's squad wins. The search score
/// (speed x value) alone always picked Proviso.
#[test]
fn a_gold_starved_post_prefers_the_squad_paid_per_bar() {
    const SHAMARE: &str = "char_254_vodfox";
    const TEQUILA: &str = "char_486_takila";
    const BIBEAK: &str = "char_252_bibeak";
    const PROVISO: &str = "char_4032_provs";
    const ARCHETTO: &str = "char_332_archet";
    const VIGIL: &str = "char_427_vigil";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let mut roster: Vec<_> = [SHAMARE, TEQUILA, BIBEAK, PROVISO, ARCHETTO, VIGIL]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    // Plain factory hands, three per factory.
    for id in [
        "char_123_fang",
        "char_133_mm",
        "char_502_nblade",
        "char_36_forget",
        "char_120_hibisc",
        "char_121_lava",
        "char_124_kroos",
        "char_211_adnach",
        "char_212_ansel",
    ] {
        if gd.building().chars.contains_key(id) {
            roster.push(profile(gd, id));
        }
    }
    let post_crew = |gold_factories: usize| -> Vec<String> {
        let mut rooms = vec![room("tp", "TRADING", 3)];
        rooms.extend((0..gold_factories).map(|i| {
            let mut r = room(&format!("mf{i}"), "MANUFACTURE", 3);
            r.current_formula = Some("F_GOLD".into());
            r
        }));
        let building = UserBuilding {
            drone_capacity: None,
            rooms,
        };
        let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
        asn.rooms
            .into_iter()
            .find(|r| r.room_type == "TRADING")
            .map(|r| r.operators)
            .unwrap_or_default()
    };
    let starved = post_crew(1);
    assert!(
        starved.iter().any(|o| o == SHAMARE) && starved.iter().any(|o| o == TEQUILA),
        "one gold factory: the per-bar squad, got {starved:?}"
    );
    // Rich: no gold factory at all: the post sells from stock, so every
    // bonus bar Proviso draws is supplied, and her squad earns more.
    let rich = {
        let building = UserBuilding {
            drone_capacity: None,
            rooms: vec![room("tp", "TRADING", 3)],
        };
        let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
        asn.rooms
            .into_iter()
            .find(|r| r.room_type == "TRADING")
            .map(|r| r.operators)
            .unwrap_or_default()
    };
    assert!(
        rich.iter().any(|o| o == PROVISO),
        "gold-rich: Proviso's bonus bars have supply, got {rich:?}"
    );
}

/// Two posts share one gold supply, so their crews are chosen together.
/// Picked one post at a time, the first post took Proviso against the
/// whole supply and the second was left selling bars nobody makes
/// (31010962: Proviso/Exusiai/Lemuen + Hoederer/Quartz realized 39.5k
/// LMD/day where Proviso/Shamare/Tequila + Exusiai/Lemuen makes 42.8k).
/// The joint pick must realize at least what every sequential order of the
/// same candidates would, and on a starved base it seats the per-bar squad.
#[test]
fn two_posts_on_one_gold_supply_are_crewed_together() {
    const SHAMARE: &str = "char_254_vodfox";
    const TEQUILA: &str = "char_486_takila";
    const BIBEAK: &str = "char_252_bibeak";
    const PROVISO: &str = "char_4032_provs";
    let gd = load_game_data();
    let (registry, drains) =
        build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let mut roster: Vec<_> = [
        SHAMARE,
        TEQUILA,
        BIBEAK,
        PROVISO,
        "char_103_angel",
        "char_4193_lemuen",
        "char_4088_hodrer",
        "char_4063_quartz",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .map(|id| profile(gd, id))
    .collect();
    assert!(
        roster.len() >= 7,
        "the traders are in the game data: {}",
        roster.len()
    );
    // Six plain factory hands for the six factory seats, so no trader is
    // padded into a factory before the posts are picked.
    let hands: Vec<_> = [
        "char_123_fang",
        "char_133_mm",
        "char_502_nblade",
        "char_124_kroos",
        "char_211_adnach",
        "char_212_ansel",
        "char_210_stward",
        "char_120_hibisc",
        "char_121_lava",
        "char_36_forget",
    ]
    .iter()
    .filter(|id| gd.building().chars.contains_key(**id))
    .take(6)
    .map(|id| profile(gd, id))
    .collect();
    assert_eq!(hands.len(), 6, "six plain hands exist in the game data");
    roster.extend(hands);
    let mut rooms = vec![room("tp3", "TRADING", 3), room("tp2", "TRADING", 2)];
    for i in 0..2 {
        let mut r = room(&format!("mf{i}"), "MANUFACTURE", 3);
        r.current_formula = Some("F_GOLD".into());
        rooms.push(r);
    }
    let building = UserBuilding {
        drone_capacity: None,
        rooms,
    };
    let asn = compute_optimal_assignment(&roster, &building, gd.building(), &registry, &drains);
    let posts: Vec<&_> = asn
        .rooms
        .iter()
        .filter(|r| r.room_type == "TRADING")
        .collect();
    assert_eq!(posts.len(), 2, "both posts staffed");
    let seated = |id: &str| posts.iter().any(|p| p.operators.iter().any(|o| o == id));
    assert!(
        seated(SHAMARE) && seated(TEQUILA),
        "the per-bar squad sells on the starved base, got {:?}",
        posts
            .iter()
            .map(|p| (&p.slot_id, &p.operators))
            .collect::<Vec<_>>()
    );
    let l3 = posts
        .iter()
        .find(|p| p.slot_id == "tp3")
        .expect("the level-3 post");
    let l2 = posts
        .iter()
        .find(|p| p.slot_id == "tp2")
        .expect("the level-2 post");
    assert!(
        l3.operators.iter().any(|o| o == SHAMARE) && l3.operators.iter().any(|o| o == TEQUILA),
        "the squad takes the level-3 post, got {:?}",
        l3.operators
    );
    assert!(
        l2.operators.len() == 2 && l2.operators.iter().all(|o| o != SHAMARE && o != TEQUILA),
        "the level-2 post is fully crewed from the rest, got {:?}",
        l2.operators
    );
    // Proviso's "+2 gold on orders below 4" pays on every order a level-2
    // post draws and only on some of a level-3 post's: when the coupled
    // value cannot tell the two apart (both posts outsell the factories),
    // she takes the lower post.
    assert!(
        l2.operators.iter().any(|o| o == PROVISO),
        "Proviso sells at the level-2 post, got L3 {:?} / L2 {:?}",
        l3.operators,
        l2.operators
    );
}

/// The community sheet's Proviso table: her order manipulation as a
/// multiplier on a level-3 post's LMD rate, per post level and elite, and
/// its worked example (E2 Proviso + Exusiai at a level-2 post, Amiya in the
/// Control Center: (100 + 35 + 7 + 2) x 1.811073 - 103 - 7 = 150.79). The
/// order-mix model reproduces every figure to six decimals from the game's
/// order sizes and each level's draw mix, no table of multipliers.
#[test]
#[allow(clippy::suboptimal_flops)] // the sheet's formula, written as the sheet writes it
fn proviso_multipliers_match_the_community_sheet_at_every_level_and_elite() {
    use myrtle_engine_base::scoring::order_mix::bars_per_day;
    const PROVISO: &str = "char_4032_provs";
    let gd = load_game_data();
    let (registry, _) = build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let l2_over_l3 = bars_per_day(2) / bars_per_day(3);
    assert!(
        (l2_over_l3 - 0.987_858).abs() < 5e-7,
        "a bare level-2 post sells {l2_over_l3:.6} of a level-3 post's LMD (sheet 0.987858)"
    );
    for (level, elite, sheet) in [
        (3, 0, 1.275_862),
        (3, 2, 1.551_724),
        (2, 0, 1.399_466),
        (2, 2, 1.811_073),
    ] {
        let t = ledger_totals(gd, &registry, &[profile_at(gd, PROVISO, elite)], level);
        let ratio = if level == 2 { l2_over_l3 } else { 1.0 };
        let ours = (1.0 + t.order_value_pct / 100.0) * ratio;
        assert!(
            (ours - sheet).abs() < 5e-7,
            "Proviso E{elite} at a level-{level} post: {ours:.6} vs sheet {sheet:.6}"
        );
        assert!(
            t.speed_pct.abs() < 1e-9,
            "Proviso adds no order speed of her own, got {}",
            t.speed_pct
        );
    }
    let t = ledger_totals(
        gd,
        &registry,
        &[
            profile_at(gd, PROVISO, 2),
            profile_at(gd, "char_103_angel", 2),
        ],
        2,
    );
    let equivalent =
        (100.0 + t.speed_pct + 7.0 + 2.0) * (1.0 + t.order_value_pct / 100.0) * l2_over_l3
            - 103.0
            - 7.0;
    assert!(
        (equivalent - 150.79).abs() < 0.01,
        "the sheet's worked example reads {equivalent:.2} (speed {}, value {})",
        t.speed_pct,
        t.order_value_pct
    );
}

/// The community sheet's Shamare-squad table: equivalent productivity of a
/// level-3 post over a three-seat bare post (103), for every Tequila and
/// Bibeak tier. Tailoring steps add across the crew (Shamare's α plus E0
/// Bibeak's α is more than either), and one measured mix per step count
/// reproduces all eight rows within 0.1.
#[test]
#[allow(clippy::suboptimal_flops)] // the sheet's formula, written as the sheet writes it
fn shamare_squad_equivalents_match_the_community_sheet() {
    const SHAMARE: &str = "char_254_vodfox";
    const TEQUILA: &str = "char_486_takila";
    const BIBEAK: &str = "char_252_bibeak";
    let gd = load_game_data();
    let (registry, _) = build_registry(&gd.building().buffs, &build_name_to_char(gd.operators()));
    let other = |n: usize| profile_at(gd, ["char_123_fang", "char_133_mm"][n], 2);
    let rows: Vec<(&str, Vec<OperatorBaseProfile>, f64)> = vec![
        (
            "Shamare + other",
            vec![profile_at(gd, SHAMARE, 2), other(0), other(1)],
            91.6,
        ),
        (
            "E2 Bibeak + other",
            vec![
                profile_at(gd, SHAMARE, 2),
                profile_at(gd, BIBEAK, 2),
                other(0),
            ],
            92.8,
        ),
        (
            "E0 Tequila + other",
            vec![
                profile_at(gd, SHAMARE, 2),
                profile_at(gd, TEQUILA, 0),
                other(0),
            ],
            108.07,
        ),
        (
            "E0 Tequila + E0 Bibeak",
            vec![
                profile_at(gd, SHAMARE, 2),
                profile_at(gd, TEQUILA, 0),
                profile_at(gd, BIBEAK, 0),
            ],
            110.08,
        ),
        (
            "E0 Tequila + E2 Bibeak",
            vec![
                profile_at(gd, SHAMARE, 2),
                profile_at(gd, TEQUILA, 0),
                profile_at(gd, BIBEAK, 2),
            ],
            115.52,
        ),
        (
            "E2 Tequila + other",
            vec![
                profile_at(gd, SHAMARE, 2),
                profile_at(gd, TEQUILA, 2),
                other(0),
            ],
            124.39,
        ),
        (
            "E2 Tequila + E0 Bibeak",
            vec![
                profile_at(gd, SHAMARE, 2),
                profile_at(gd, TEQUILA, 2),
                profile_at(gd, BIBEAK, 0),
            ],
            128.12,
        ),
        (
            "E2 Tequila + E2 Bibeak",
            vec![
                profile_at(gd, SHAMARE, 2),
                profile_at(gd, TEQUILA, 2),
                profile_at(gd, BIBEAK, 2),
            ],
            138.21,
        ),
    ];
    for (name, crew, sheet) in rows {
        let t = ledger_totals(gd, &registry, &crew, 3);
        let equivalent = (100.0 + t.speed_pct + 3.0) * (1.0 + t.order_value_pct / 100.0) - 103.0;
        assert!(
            (equivalent - sheet).abs() < 0.1,
            "{name}: {equivalent:.2} vs sheet {sheet:.2} (speed {}, value {:.2})",
            t.speed_pct,
            t.order_value_pct
        );
    }
}
