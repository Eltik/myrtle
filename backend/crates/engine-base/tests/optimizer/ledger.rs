use super::*;

/// The deep-dive skill ledger: marginals by ablation, with coupled skills
/// (Feud + Jaye's limit cut), pair riders, CC globals and capacity lines all
/// classified.
#[test]
fn skill_ledger_reports_marginals_and_dispositions() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let mut building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)],
    };
    building.rooms[0].current_operators = vec!["char_4132_ascln".into()];
    building.rooms[1].current_operators = vec![
        "char_140_whitew".into(),
        "char_102_texas".into(),
        "char_272_strong".into(),
    ];
    let roster: Vec<OperatorBaseProfile> = [
        "char_4132_ascln",
        "char_140_whitew",
        "char_102_texas",
        "char_272_strong",
    ]
    .iter()
    .map(|id| profile(gd, id))
    .collect();
    let asn =
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
    let tp = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .expect("trading room scored");
    let line = |op: &str, buff: &str| {
        tp.ledger
            .iter()
            .find(|l| l.operator_id == op && l.buff_id == buff)
            .unwrap_or_else(|| panic!("no ledger line for {op} {buff}"))
    };
    use myrtle_engine_base::scoring::skill_ledger::LineDisposition;
    // Texas' Feud fires with Lappland present. Its marginal is BELOW its face
    // +65: its efficiency is what Jaye's Basic Needs cuts the limit by (6 of
    // the post's 10 + 4), and Jaye pays 4% per order of the final limit, so
    // the ledger reports what the room actually loses (65 - 6 x 4 = 41).
    let feud = line("char_102_texas", "trade_ord_spd&cost_P[000]");
    assert_eq!(feud.disposition, LineDisposition::Contributes);
    assert!(
        (feud.speed_pct - 41.0).abs() < 1e-6,
        "Feud marginal {} = 65 - 24",
        feud.speed_pct
    );
    // Jaye's Street Economics pays per empty slot of the final limit (8).
    let jaye = line("char_272_strong", "trade_ord_limit_diff[000]");
    assert_eq!(jaye.disposition, LineDisposition::Contributes);
    assert!(
        (jaye.speed_pct - 16.0).abs() < 1e-6,
        "Street Economics {} = 2 x 8",
        jaye.speed_pct
    );
    // Lappland's Texas-gated +4 order limit is what Jaye's riders read: it
    // moves efficiency through him (4 x 4 = 16), not only capacity.
    let lapp = line("char_140_whitew", "trade_ord_limit&cost_P[001]");
    assert_eq!(lapp.disposition, LineDisposition::Contributes);
    assert!(
        (lapp.speed_pct - 16.0).abs() < 1e-6,
        "Hidden Purpose marginal {} = 4 x 4",
        lapp.speed_pct
    );
    // Ascalon's CC-wide +7% shows on the post as a Control-Center line.
    let cc_line = line("char_4132_ascln", "control_tra_spd[030]");
    assert!(cc_line.from_control_center);
    assert_eq!(cc_line.disposition, LineDisposition::Contributes);
    assert!((cc_line.speed_pct - 7.0).abs() < 1e-6);
}

/// Non-stacking family attribution: with two "+7% all Trading Posts" globals
/// in the Control Center, exactly ONE line claims the +7 and the duplicate
/// reads "covered", and the CC row's lines sum to the crew's global total.
/// (Pure ablation marginals are tie-blind: removing either copy changes
/// nothing, so nobody would claim the value.)
#[test]
fn ledger_attributes_nonstacking_families_to_one_winner() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    use myrtle_engine_base::scoring::skill_ledger::LineDisposition;
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let mut building = UserBuilding {
        drone_capacity: None,
        rooms: vec![room("cc", "CONTROL", 5), room("tp", "TRADING", 3)],
    };
    building.rooms[0].current_operators = vec!["char_4132_ascln".into(), "char_308_swire".into()];
    building.rooms[1].current_operators = vec!["char_103_angel".into()];
    let roster: Vec<OperatorBaseProfile> = ["char_4132_ascln", "char_308_swire", "char_103_angel"]
        .iter()
        .map(|id| profile(gd, id))
        .collect();
    let asn =
        compute_current_assignment(&roster, &building, gd.building(), &registry, &drains, None);
    let cc = asn
        .rooms
        .iter()
        .find(|r| r.room_type == "CONTROL")
        .expect("cc scored");
    let trading_lines: Vec<_> = cc
        .ledger
        .iter()
        .filter(|l| l.speed_pct > 0.0 || l.disposition == LineDisposition::Covered)
        .collect();
    let winners = trading_lines
        .iter()
        .filter(|l| l.disposition == LineDisposition::Contributes)
        .count();
    let covered = trading_lines
        .iter()
        .filter(|l| l.disposition == LineDisposition::Covered)
        .count();
    assert_eq!(winners, 1, "exactly one +7 claims the family");
    assert_eq!(covered, 1, "the duplicate reads covered");
    let line_sum: f64 = cc.ledger.iter().map(|l| l.speed_pct).sum();
    assert!(
        (line_sum - cc.total_efficiency).abs() < 1e-6,
        "CC lines ({line_sum}) sum to the row total ({})",
        cc.total_efficiency
    );
    // And the trading post sees exactly one +7 [CC] line, one covered.
    let tp = asn.rooms.iter().find(|r| r.room_type == "TRADING").unwrap();
    let cc_lines: Vec<_> = tp.ledger.iter().filter(|l| l.from_control_center).collect();
    assert_eq!(
        cc_lines
            .iter()
            .filter(|l| l.disposition == LineDisposition::Contributes)
            .count(),
        1
    );
    assert_eq!(
        cc_lines
            .iter()
            .filter(|l| l.disposition == LineDisposition::Covered)
            .count(),
        1
    );
}

/// Dump-driven diagnostic: the planner's plain-optimal path AND the rotation
/// on a captured base, printing Control-Center / trading crews and where
/// Delphine-class conditionals land.
/// Run: `BASE_REPRO_DIR=<dir> BASE_REPRO_UID=<uid> cargo test -- planner_probe --ignored --nocapture`
#[test]
#[ignore = "needs a captured user dump (BASE_REPRO_DIR)"]
fn ledger_probe() {
    use myrtle_engine_base::assignment::compute_current_assignment;
    let gd = load_game_data();
    let dir = std::env::var("BASE_REPRO_DIR").expect("BASE_REPRO_DIR");
    let uid = std::env::var("BASE_REPRO_UID").expect("BASE_REPRO_UID");
    let read = |what: &str| -> serde_json::Value {
        let path = format!("{dir}/{what}_{uid}.json");
        serde_json::from_str(
            &std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}")),
        )
        .expect("valid json")
    };
    let building = UserBuilding::from_json(&read("building"));
    let roster: Vec<RosterEntry> = serde_json::from_value(read("roster")).expect("roster rows");
    let profiles: Vec<OperatorBaseProfile> = roster
        .iter()
        .filter_map(|entry| {
            let bc = gd.building().chars.get(&entry.operator_id)?;
            let static_op = gd.operators().get(&entry.operator_id);
            let faction_tags = static_op
                .map(|op| faction_tags_with_glossary(gd, &entry.operator_id, op))
                .unwrap_or_default();
            let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
            Some(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                rarity,
                gd.building(),
                false,
            ))
        })
        .collect();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let asn = compute_current_assignment(
        &profiles,
        &building,
        gd.building(),
        &registry,
        &drains,
        None,
    );
    for r in &asn.rooms {
        if let Some(f) = &r.fill {
            println!(
                "FILL {} {} {:?} cap {} fill {:.1}h",
                r.slot_id,
                r.room_type,
                r.formula_type.as_deref().unwrap_or("-"),
                f.capacity,
                f.fill_hours
            );
        }
        if r.room_type != "TRADING" && r.room_type != "CONTROL" {
            continue;
        }
        println!(
            "== {} {} eff {:.1} value {:.1}",
            r.slot_id, r.room_type, r.total_efficiency, r.order_value
        );
        for l in &r.ledger {
            println!(
                "   {:22} {:32} speed {:+7.2} value {:+6.2} {:?}{}",
                l.operator_id,
                l.buff_id,
                l.speed_pct,
                l.value_pct,
                l.disposition,
                if l.from_control_center { "  [CC]" } else { "" }
            );
        }
    }
}

#[test]
#[ignore]
fn ledger_probe_registry_peek() {
    let gd = load_game_data();
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, _) = build_registry(&gd.building().buffs, &name_to_char);
    for id in ["trade_ord_limit&cost_P[001]", "trade_ord_limit&cost_P[000]"] {
        println!("{id} => {:?}", registry.get(id));
    }
}
