//! Squad Builder analysis on the real EN tables.
//!
//! Every assertion names the operator and the field it reads. Where the
//! reference tool's numbers disagree with the data, the test asserts what the
//! data says and the comment gives the field values.

mod common;

use std::fmt::Write as _;

use backend::core::gamedata::types::GameData;
use backend::core::gamedata::types::module::ModuleType;
use backend::core::squad::rules::SkillRules;
use backend::core::squad::types::Unlock;
use backend::core::squad::types::{
    CoverageOp, CoverageRow, CoverageRowKey, CutKind, EconomyNoteKind, EconomySeat,
    HandoffLeftOutReason, ReadinessFlagKind, SUPPORT_SEAT, SourceKind, SquadAnalysis,
    SquadAnalyzeRequest, SquadMemberBuild, SynergyCond, SynergyGroup, SynergyItem, SynergyReason,
    SynergyTargetKind, ThemeKind,
};
use backend::core::squad::{analyze, analyze_with};

const SCRATCH: &str = "/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/60e6ad18-68d6-441f-ad77-fd1a445fa52a/scratchpad/sb";

#[allow(clippy::too_many_arguments, clippy::unnecessary_wraps)] // one argument per build field; `members` holds `Option`s
fn m(
    id: &str,
    phase: i32,
    level: i32,
    potential: i32,
    skill_index: i32,
    skill_level: i32,
    mastery: i32,
    module: Option<&str>,
    module_level: i32,
) -> Option<SquadMemberBuild> {
    Some(SquadMemberBuild {
        char_id: id.to_string(),
        phase,
        level,
        potential,
        skill_index,
        skill_level,
        mastery,
        module_id: module.map(str::to_string),
        module_level,
    })
}

const fn req(members: Vec<Option<SquadMemberBuild>>) -> SquadAnalyzeRequest {
    SquadAnalyzeRequest {
        members,
        support: None,
        start_dp: 10,
        deploy_limit: None,
        dub: None,
    }
}

fn row(a: &SquadAnalysis, key: CoverageRowKey) -> &CoverageRow {
    a.coverage
        .iter()
        .flat_map(|g| &g.rows)
        .find(|r| r.key == key)
        .expect("every row is emitted")
}

fn cov<'a>(a: &'a SquadAnalysis, key: CoverageRowKey, id: &str) -> Option<&'a CoverageOp> {
    row(a, key).ops.iter().find(|o| o.char_id == id)
}

fn eco<'a>(a: &'a SquadAnalysis, id: &str) -> &'a EconomySeat {
    a.economy
        .seats
        .iter()
        .find(|s| s.char_id == id)
        .expect("seat in economy")
}

fn syn<'a>(a: &'a SquadAnalysis, id: &str, name: &str) -> Option<&'a SynergyItem> {
    a.synergy
        .items
        .iter()
        .find(|i| i.char_id == id && i.name == name)
}

fn first_advanced_module(data: &GameData, id: &str) -> String {
    data.operators[id]
        .modules
        .iter()
        .find(|m| m.module.module_type == ModuleType::Advanced)
        .expect("an advanced module")
        .module
        .uni_equip_id
        .clone()
}

const ARCH: &str = "char_332_archet";
const BPIPE: &str = "char_222_bpipe";
const EYJA: &str = "char_180_amgoat";
const CEYLON: &str = "char_348_ceylon";
const SAIL: &str = "char_479_sleach";
const IFRIT: &str = "char_134_ifrit";

fn reference(data: &GameData) -> SquadAnalyzeRequest {
    let bp_mod = first_advanced_module(data, BPIPE);
    SquadAnalyzeRequest {
        members: vec![
            m(ARCH, 2, 90, 0, 2, 7, 3, Some("uniequip_003_archet"), 3),
            m(BPIPE, 2, 60, 0, 2, 7, 3, Some(&bp_mod), 3),
            m(EYJA, 2, 60, 0, 1, 7, 3, None, 1),
            m(CEYLON, 2, 80, 5, 0, 7, 3, None, 1),
            m(SAIL, 2, 60, 0, 2, 7, 0, None, 1),
            m(IFRIT, 2, 60, 1, 1, 7, 0, None, 1),
        ],
        support: None,
        start_dp: 22,
        deploy_limit: Some(9),
        dub: None,
    }
}

#[test]
#[allow(clippy::too_many_lines)] // the reference squad's whole checklist in one place
fn reference_squad() {
    let data = common::load_game_data();
    let a = analyze(data, &reference(data));
    println!("{}", serde_json::to_string_pretty(&a).expect("json"));

    // ---- coverage
    let why = |key, id| cov(&a, key, id).map(|o| o.why.kind);
    // Bagpipe: sub `charger`, trait blackboard `cost` (2 with her module).
    assert_eq!(why(CoverageRowKey::DpGen, BPIPE), Some(SourceKind::Trait));
    // Saileach S3 "Glorious Banner": blackboard `cost` 10.
    let sail_dp = cov(&a, CoverageRowKey::DpGen, SAIL).expect("Saileach dp_gen");
    assert_eq!(sail_dp.why.kind, SourceKind::Skill);
    assert_eq!(sail_dp.why.name.as_deref(), Some("Glorious Banner"));
    assert_eq!(sail_dp.why.skill_index, Some(2));
    // Archetto: trait "Attacks aerial enemies first".
    assert_eq!(why(CoverageRowKey::AntiAir, ARCH), Some(SourceKind::Trait));
    // Archetto S3 "Thundering Arrows" hits several targets.
    assert_eq!(why(CoverageRowKey::Multi, ARCH), Some(SourceKind::Skill));
    // Eyjafjalla: trait "Deals Arts damage".
    assert_eq!(why(CoverageRowKey::Arts, EYJA), Some(SourceKind::Trait));
    // The reference tool says Debuff by tag; her S2 "Ignition" (the chosen
    // skill_index 1) cuts enemy `magic_resistance`, and Skill ranks before Tag.
    assert_eq!(why(CoverageRowKey::Debuff, EYJA), Some(SourceKind::Skill));
    // Ceylon: sub `healer` (Therapist), the curated single-heal list.
    assert_eq!(
        why(CoverageRowKey::HealSingle, CEYLON),
        Some(SourceKind::Trait)
    );
    let physical: Vec<&str> = row(&a, CoverageRowKey::Physical)
        .ops
        .iter()
        .map(|o| o.char_id.as_str())
        .collect();
    assert_eq!(physical, vec![ARCH, BPIPE, SAIL]);

    // ---- economy
    // Ceylon: key frame `cost` 22, potential ranks 0, 2 and 4 carry COST -1
    // each and potential 5 has ranks 0..5: one Potential cut of -3.
    let ceylon = eco(&a, CEYLON);
    assert_eq!((ceylon.base_cost, ceylon.cost), (22, 19));
    assert_eq!(ceylon.cuts.len(), 1);
    assert_eq!(ceylon.cuts[0].kind, CutKind::Potential);
    assert!((ceylon.cuts[0].amount + 3.0).abs() < 1e-9);
    // Ifrit: `cost` 34, potential 1 = rank 0 COST -1.
    assert_eq!((eco(&a, IFRIT).base_cost, eco(&a, IFRIT).cost), (34, 33));
    // Bagpipe: `cost` 13, `respawn_time` 70.
    assert_eq!(eco(&a, BPIPE).cost, 13);
    assert!((eco(&a, BPIPE).respawn - 70.0).abs() < 1e-9);
    let ladder: Vec<(&str, i32)> = a
        .economy
        .seats
        .iter()
        .map(|s| (s.char_id.as_str(), s.cost))
        .collect();
    assert_eq!(
        ladder,
        vec![
            (SAIL, 12),
            (BPIPE, 13),
            (ARCH, 14),
            (CEYLON, 19),
            (EYJA, 21),
            (IFRIT, 33)
        ]
    );
    assert_eq!(a.economy.total, 112);
    assert_eq!(a.economy.openers, 5);
    assert_eq!(a.economy.start_dp, 22);
    // Saileach S3 at rank 7, mastery 0 (the request) is `levels[6]`: sp_cost
    // 24, init_sp 6, so 18 s before squad SP. The reference tool's 20 / 7 / 13
    // are `levels[9]` (M3). Bagpipe at potential 0 has the "+6" Martial
    // Tradition candidate (the "+8" needs potential rank 4), so first_ready is
    // 24 - 6 - 6 = 12, not 5.
    let src = a
        .economy
        .sources
        .iter()
        .find(|s| s.char_id == SAIL)
        .expect("Saileach DP source");
    assert!((src.amount - 10.0).abs() < 1e-9);
    assert_eq!(src.duration, Some(10.0));
    assert_eq!(src.sp_cost, Some(24));
    assert_eq!(src.sp_init, Some(12));
    assert_eq!(src.first_ready, Some(12.0));
    assert!(
        a.economy
            .counted
            .iter()
            .any(|n| n.kind == EconomyNoteKind::SpBonus
                && n.char_id == BPIPE
                && n.amount == Some(6.0))
    );
    let bp_src = a
        .economy
        .sources
        .iter()
        .find(|s| s.char_id == BPIPE)
        .expect("Bagpipe DP source");
    assert!(bp_src.per_kill && bp_src.refund);

    // ---- synergy
    let mt = syn(&a, BPIPE, "Martial Tradition").expect("Martial Tradition");
    assert_eq!(
        (mt.cond, mt.group),
        (SynergyCond::Squad, SynergyGroup::Squad)
    );
    assert_eq!(mt.target.kind, SynergyTargetKind::Class);
    assert_eq!(mt.target.value.as_deref(), Some("PIONEER"));
    assert_eq!(mt.to, vec![4]);
    let pyro = syn(&a, EYJA, "Pyrobreath").expect("Pyrobreath");
    assert_eq!(
        (pyro.cond, pyro.group),
        (SynergyCond::Field, SynergyGroup::Field)
    );
    assert_eq!(pyro.to, vec![5]);
    let si = syn(&a, SAIL, "Spiritual Influence").expect("Spiritual Influence");
    assert_eq!(si.target.kind, SynergyTargetKind::Next);
    assert_eq!(
        (si.group, si.reason),
        (SynergyGroup::Check, Some(SynergyReason::Unseen))
    );
    let ub = syn(&a, SAIL, "Unwavering Banner").expect("Unwavering Banner");
    assert_eq!(
        (ub.cond, ub.group),
        (SynergyCond::Near, SynergyGroup::Place)
    );
    let lt = syn(&a, ARCH, "Landen Tactics").expect("Landen Tactics");
    assert_eq!(lt.target.kind, SynergyTargetKind::SpType);
    assert_eq!(lt.target.value.as_deref(), Some("INCREASE_WHEN_ATTACK"));
    assert_eq!(
        (lt.group, lt.reason),
        (SynergyGroup::Check, Some(SynergyReason::Nobody))
    );

    // ---- themes
    let th = &a.themes;
    assert_eq!(
        (
            th.stats.total_rarity,
            th.stats.top_rarity,
            th.stats.top_rarity_seats,
            th.stats.medics,
            th.stats.classes,
            th.stats.class_total
        ),
        (35, 6, 5, 1, 4, 8)
    );
    assert!(
        th.labels
            .iter()
            .any(|s| s.kind == ThemeKind::Gender && s.value == "Female" && s.seats.len() == 6)
    );
    let has_shared = |kind: ThemeKind, value: &str, seats: &[i32]| {
        th.shared
            .iter()
            .any(|s| s.kind == kind && s.value == value && s.seats == seats)
    };
    assert!(has_shared(ThemeKind::Faction, "Victoria", &[1, 4]));
    assert!(has_shared(ThemeKind::Race, "Liberi", &[0, 3]));
    assert!(has_shared(ThemeKind::Race, "Vouivre", &[1, 4]));

    // ---- handoff: Ifrit gets Pyrobreath's `atk`. Eyjafjalla at potential 0
    // has the 0.14 candidate; the reference's 0.16 is the potential-5 one.
    let h = a
        .handoff
        .iter()
        .find(|h| h.char_id == IFRIT)
        .expect("Ifrit");
    assert!((h.buffs.atk - 0.14).abs() < 1e-6);
    assert_eq!(h.counted.len(), 1);
    assert_eq!(h.counted[0].cond, SynergyCond::Field);
}

#[test]
fn zima_cuts_vanguards_not_herself() {
    let data = common::load_game_data();
    let a = analyze(
        data,
        &req(vec![
            m("char_115_headbr", 2, 70, 0, 0, 7, 0, None, 1),
            m("char_151_myrtle", 2, 70, 0, 0, 7, 0, None, 1),
        ]),
    );
    // Zima "Spearheader": `cost` -1 to "All [Vanguard] Operators" in the squad.
    let myrtle = eco(&a, "char_151_myrtle");
    assert!(myrtle.cuts.iter().any(|c| c.kind == CutKind::Squad
        && c.from_seat == Some(0)
        && (c.amount + 1.0).abs() < 1e-9));
    assert_eq!(myrtle.cost, myrtle.base_cost - 1);
    let zima = eco(&a, "char_115_headbr");
    assert!(
        zima.cuts.is_empty(),
        "Zima does not cut herself: {:?}",
        zima.cuts
    );
}

#[test]
fn silverash_leader_cuts_others_redeploy() {
    let data = common::load_game_data();
    let a = analyze(
        data,
        &req(vec![
            m("char_172_svrash", 2, 90, 5, 2, 7, 3, None, 1),
            m("char_151_myrtle", 2, 70, 0, 0, 7, 0, None, 1),
        ]),
    );
    // "Leader" at potential 5: `respawn_time` -0.12, "all allied units".
    let myrtle = eco(&a, "char_151_myrtle");
    let cut = myrtle
        .respawn_cuts
        .iter()
        .find(|c| c.kind == CutKind::Squad)
        .expect("Leader respawn cut");
    assert!((cut.ratio.expect("ratio") + 0.12).abs() < 1e-6);
    // 70 s base times (1 - 0.12).
    assert!((myrtle.respawn - 61.6).abs() < 1e-3);
    let sa = eco(&a, "char_172_svrash");
    assert!(sa.respawn_cuts.iter().all(|c| c.kind != CutKind::Squad));
}

#[test]
fn jaye_drains_and_does_not_generate() {
    let data = common::load_game_data();
    let a = analyze(
        data,
        &req(vec![m("char_272_strong", 2, 70, 0, 0, 7, 0, None, 1)]),
    );
    // Merchant trait: `cost` -3 every `interval` 3 s = -1 DP/s.
    let drain = a
        .economy
        .notes
        .iter()
        .find(|n| n.kind == EconomyNoteKind::Drain)
        .expect("Drain note");
    assert!((drain.amount.expect("amount") + 1.0).abs() < 1e-9);
    assert!(cov(&a, CoverageRowKey::DpGen, "char_272_strong").is_none());
}

#[test]
fn cuora_blocks_three() {
    let data = common::load_game_data();
    // Cuora's key frame `block_cnt` is already 3 at every promotion (E1
    // included), so the reference's "base 2 + 1" does not hold. Her S2 "Shell
    // Defense" `block_cnt` +1 still earns the row, and Skill ranks before Stats.
    let op = &data.operators["char_150_snakek"];
    assert_eq!(op.phases[1].attributes_key_frames[0].data.block_cnt, 3);
    let a = analyze(
        data,
        &req(vec![m("char_150_snakek", 1, 50, 0, 1, 7, 0, None, 1)]),
    );
    let o = cov(&a, CoverageRowKey::Block3, "char_150_snakek").expect("block3");
    assert_eq!(o.why.kind, SourceKind::Skill);
    let s1 = analyze(
        data,
        &req(vec![m("char_150_snakek", 1, 50, 0, 0, 7, 0, None, 1)]),
    );
    assert_eq!(
        cov(&s1, CoverageRowKey::Block3, "char_150_snakek").map(|o| o.why.kind),
        Some(SourceKind::Stats)
    );
}

#[test]
fn fang_lightweight_from_elite_one() {
    let data = common::load_game_data();
    // "Lightweight" `cost` -1 unlocks at PHASE_1 level 1.
    let e1 = analyze(
        data,
        &req(vec![m("char_123_fang", 1, 1, 0, 0, 7, 0, None, 1)]),
    );
    let f = eco(&e1, "char_123_fang");
    assert!(f.cuts.iter().any(|c| c.kind == CutKind::Talent
        && c.name.as_deref() == Some("Lightweight")
        && (c.amount + 1.0).abs() < 1e-9));
    assert_eq!(f.cost, f.base_cost - 1);
    let e0 = analyze(
        data,
        &req(vec![m("char_123_fang", 0, 30, 0, 0, 4, 0, None, 1)]),
    );
    assert!(eco(&e0, "char_123_fang").cuts.is_empty());
}

#[test]
fn texas_adds_starting_dp() {
    let data = common::load_game_data();
    let a = analyze(
        data,
        &req(vec![m("char_102_texas", 2, 70, 0, 0, 7, 0, None, 1)]),
    );
    // "Tactical Delivery" `cost` 2, "when Texas is in the squad".
    assert_eq!(a.economy.start_bonus.len(), 1);
    assert_eq!(a.economy.start_bonus[0].amount, 2);
    assert_eq!(a.economy.start_dp, 12);
}

#[test]
fn mulberry_elemental_heals() {
    let data = common::load_game_data();
    let a = analyze(
        data,
        &req(vec![m("char_473_mberry", 2, 70, 0, 0, 7, 0, None, 1)]),
    );
    // Trait blackboard `ep_heal_ratio` 0.5 ranks before S1's `ep_heal_ratio`.
    assert_eq!(
        cov(&a, CoverageRowKey::HealElemental, "char_473_mberry").map(|o| o.why.kind),
        Some(SourceKind::Trait)
    );
    // S2 `ep_damage_resistance` protects allies, it is not Elemental damage.
    let s2 = analyze(
        data,
        &req(vec![m("char_473_mberry", 2, 70, 0, 1, 7, 0, None, 1)]),
    );
    assert!(cov(&s2, CoverageRowKey::TrueElemental, "char_473_mberry").is_none());
    // Mulberry S1 "Healing Haze" carries `ep_heal_ratio`: the skill source.
    let op = &data.operators["char_473_mberry"];
    let s1 = &op.skills[0].static_data.as_ref().expect("S1").levels[6];
    assert!(s1.blackboard.iter().any(|b| b.key.starts_with("ep_heal")));
}

#[test]
fn shaw_shifts_flint_does_not() {
    let data = common::load_game_data();
    // Shaw S1 `force` 2.0; Flint S1 `force` 0.0.
    let a = analyze(
        data,
        &req(vec![m("char_277_sqrrel", 2, 70, 0, 0, 7, 0, None, 1)]),
    );
    // Shaw is a `pusher` (curated Shift branch): Trait ranks before Skill, so
    // the chip says Trait although S1's `force` qualifies too.
    assert_eq!(
        cov(&a, CoverageRowKey::Shift, "char_277_sqrrel").map(|o| o.why.kind),
        Some(SourceKind::Trait)
    );
    let op = &data.operators["char_277_sqrrel"];
    let s1 = &op.skills[0].static_data.as_ref().expect("S1").levels[6];
    assert!(
        s1.blackboard
            .iter()
            .any(|b| b.key == "force" && b.value > 0.0)
    );
    let f = analyze(
        data,
        &req(vec![m("char_415_flint", 2, 70, 0, 0, 7, 0, None, 1)]),
    );
    assert!(cov(&f, CoverageRowKey::Shift, "char_415_flint").is_none());
}

fn flags(a: &SquadAnalysis, seat: i32) -> Vec<ReadinessFlagKind> {
    a.readiness
        .rows
        .iter()
        .find(|r| r.seat == seat)
        .expect("row")
        .flags
        .iter()
        .map(|f| f.kind)
        .collect()
}

#[test]
fn readiness_flags() {
    let data = common::load_game_data();
    let five_e2: Vec<_> = [ARCH, BPIPE, EYJA, CEYLON, SAIL]
        .iter()
        .map(|id| m(id, 2, 80, 0, 0, 7, 0, None, 1))
        .collect();
    let mut members = five_e2.clone();
    members.push(m(IFRIT, 2, 60, 0, 0, 7, 0, None, 1));
    let a = analyze(data, &req(members));
    let row = a
        .readiness
        .rows
        .iter()
        .find(|r| r.seat == 5)
        .expect("Ifrit");
    let lvl = row
        .flags
        .iter()
        .find(|f| f.kind == ReadinessFlagKind::Level)
        .expect("Level flag");
    assert_eq!((lvl.value, lvl.reference), (Some(60), Some(80)));

    let mut members = five_e2.clone();
    members.push(m(IFRIT, 1, 70, 0, 0, 7, 0, None, 1));
    let a = analyze(data, &req(members));
    assert!(flags(&a, 5).contains(&ReadinessFlagKind::Promotion));

    // Mastery 3 at phase 1.
    let a = analyze(data, &req(vec![m(EYJA, 1, 70, 0, 0, 7, 3, None, 1)]));
    assert!(flags(&a, 0).contains(&ReadinessFlagKind::SkillRankLocked));

    // Module at E1 (Archetto's modules unlock at PHASE_2 level 60).
    let a = analyze(
        data,
        &req(vec![m(
            ARCH,
            1,
            70,
            0,
            0,
            7,
            0,
            Some("uniequip_003_archet"),
            1,
        )]),
    );
    assert!(flags(&a, 0).contains(&ReadinessFlagKind::ModuleLocked));

    // Archetto S3 at phase 1: her raw `Skills[2].UnlockCond` is PHASE_2.
    let a = analyze(data, &req(vec![m(ARCH, 1, 70, 0, 2, 7, 0, None, 1)]));
    assert!(flags(&a, 0).contains(&ReadinessFlagKind::SkillLocked));
    let r = &a.readiness.rows[0];
    assert_eq!(r.skill_unlock.as_ref().map(|u| u.phase), Some(2));
    // Archetto S2 unlocks at PHASE_1 (raw `UnlockCond`), not E0.
    let a = analyze(data, &req(vec![m(ARCH, 0, 30, 0, 1, 4, 0, None, 1)]));
    assert_eq!(
        a.readiness.rows[0].skill_unlock.as_ref().map(|u| u.phase),
        Some(1)
    );
    assert!(flags(&a, 0).contains(&ReadinessFlagKind::SkillLocked));
}

#[test]
fn support_is_left_out_of_stats_and_never_gives() {
    let data = common::load_game_data();
    let mut r = req(vec![m("char_151_myrtle", 2, 70, 0, 0, 7, 0, None, 1)]);
    r.support = m("char_115_headbr", 2, 90, 0, 0, 7, 0, None, 1);
    let a = analyze(data, &r);
    assert!(flags(&a, SUPPORT_SEAT).contains(&ReadinessFlagKind::SupportLeftOut));
    assert_eq!(a.readiness.stats.seats, 1);
    assert_eq!(a.themes.stats.seats, 1);
    // Zima as support gives no squad cost cut.
    assert!(eco(&a, "char_151_myrtle").cuts.is_empty());
    assert!(a.synergy.items.iter().all(|i| i.seat != SUPPORT_SEAT));
    // ...but she is in coverage with `support: true`.
    assert!(
        cov(&a, CoverageRowKey::DpGen, "char_115_headbr")
            .is_some_and(|o| o.support && o.seat == SUPPORT_SEAT)
    );
}

#[test]
fn empty_and_unknown_requests() {
    let data = common::load_game_data();
    for r in [
        req(vec![]),
        req(vec![
            None,
            m("char_does_not_exist", 2, 90, 0, 0, 7, 3, Some("x"), 3),
        ]),
    ] {
        let a = analyze(data, &r);
        assert_eq!(a.coverage.len(), 5);
        let rows: usize = a.coverage.iter().map(|g| g.rows.len()).sum();
        assert_eq!(rows, 17);
        assert!(
            a.coverage
                .iter()
                .flat_map(|g| &g.rows)
                .all(|r| r.ops.is_empty())
        );
        assert_eq!((a.economy.total, a.economy.openers), (0, 0));
        assert!(a.economy.seats.is_empty());
        assert!(a.handoff.is_empty());
    }
}

fn with_myrtle(data: &GameData, id: &str) -> SquadAnalysis {
    analyze(
        data,
        &req(vec![
            m(id, 2, 70, 0, 0, 7, 0, None, 1),
            m("char_151_myrtle", 2, 70, 0, 0, 7, 0, None, 1),
        ]),
    )
}

#[test]
fn absence_conditions_are_not_squad_effects() {
    let data = common::load_game_data();
    // Ceobe "Lone Journey": "When there are no other friendly units in the
    // surrounding 4 tiles, ATK +8%": a condition on Ceobe, nobody receives.
    let a = with_myrtle(data, "char_2013_cerber");
    assert!(syn(&a, "char_2013_cerber", "Lone Journey").is_none());
    // Kirara "Lonesome Protector": "If there are no allied Operators in the
    // surrounding 8 tiles, restores 4% HP": self-only too.
    let a = with_myrtle(data, "char_478_kirara");
    assert!(syn(&a, "char_478_kirara", "Lonesome Protector").is_none());
    // Thorns the Lodestar "El Alcance de la Visión": "all allies have ASPD +5"
    // is a real grant and stays.
    let a = with_myrtle(data, "char_1039_thorn2");
    let t = syn(&a, "char_1039_thorn2", "El Alcance de la Visión").expect("Thorns alter grant");
    assert_eq!(t.target.kind, SynergyTargetKind::Everyone);
    assert!(t.to_all);
}

#[test]
fn random_ally_is_unseen() {
    let data = common::load_game_data();
    // Exusiai "Angel's Blessings": "The same buff is given to a random ally
    // when Exusiai is deployed".
    let a = with_myrtle(data, "char_103_angel");
    let ex = syn(&a, "char_103_angel", "Angel's Blessings").expect("Angel's Blessings");
    assert_eq!(ex.target.kind, SynergyTargetKind::Everyone);
    assert_eq!(ex.cond, SynergyCond::Field);
    assert_eq!(
        (ex.group, ex.reason),
        (SynergyGroup::Check, Some(SynergyReason::Unseen))
    );
    assert!(!ex.to_all && ex.to.is_empty());
    // The hand-off lists it for Myrtle as Conditional and counts nothing.
    let h = a
        .handoff
        .iter()
        .find(|h| h.char_id == "char_151_myrtle")
        .expect("Myrtle");
    assert!(h.counted.is_empty());
    assert!(h.left_out.iter().any(|l| l.name == "Angel's Blessings"
        && l.reason == HandoffLeftOutReason::Conditional));
    // Warfarin "Blood Sample Recycle": "a random ally within range": Near.
    let w = with_myrtle(data, "char_171_bldsk");
    let wf = syn(&w, "char_171_bldsk", "Blood Sample Recycle").expect("Blood Sample Recycle");
    assert_eq!(wf.cond, SynergyCond::Near);
    assert_eq!(wf.reason, Some(SynergyReason::Unseen));
    // Haruka "Fleeting Foam": "an allied unit within range gains a bubble".
    let hr = with_myrtle(data, "char_4202_haruka");
    let hf = syn(&hr, "char_4202_haruka", "Fleeting Foam").expect("Fleeting Foam");
    assert_eq!(hf.reason, Some(SynergyReason::Unseen));
}

#[test]
fn invisible_needs_enemies_in_the_clause() {
    let data = common::load_game_data();
    let one =
        |id: &str, skill: i32| analyze(data, &req(vec![m(id, 2, 70, 0, skill, 7, 0, None, 1)]));
    // Manticore "Hidden Killer - Elite" removes her own Invisibility.
    let a = one("char_215_mantic", 0);
    assert!(cov(&a, CoverageRowKey::Invisible, "char_215_mantic").is_none());
    // These name enemies in the same sentence and stay: SilverAsh "Eagle
    // Eyes", Ines "Shadow Sentry", Totter, Tsukinogi S1, Mantra S3, Iana S2,
    // and Elysium S2, whose sentence "Locked enemies have ... and lose
    // Invisibility" is cut apart at `<@ba.vup>` unless tags go first.
    for (id, skill) in [
        ("char_172_svrash", 0),
        ("char_4087_ines", 0),
        ("char_4062_totter", 0),
        ("char_343_tknogi", 0),
        ("char_4204_mantra", 2),
        ("char_4124_iana", 1),
        ("char_401_elysm", 1),
    ] {
        let a = one(id, skill);
        assert!(
            cov(&a, CoverageRowKey::Invisible, id).is_some(),
            "{id} counters Invisibility"
        );
    }
}

/// Every operator as a one-seat squad at full build with each skill: no panics,
/// and a dump of every detected squad talent and every row's coverage.
#[test]
fn census_every_operator() {
    let data = common::load_game_data();
    let mut ids: Vec<&String> = data
        .operators
        .keys()
        .filter(|k| k.starts_with("char_"))
        .collect();
    ids.sort();
    let mut synergy = String::new();
    let mut rows: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut seen_rows: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    let mut synergy_items = 0usize;
    for id in ids {
        let op = &data.operators[id];
        let phase = i32::try_from(op.phases.len().saturating_sub(1)).unwrap_or(0);
        let level = op.phases.last().map_or(1, |p| p.max_level);
        let module = op
            .modules
            .iter()
            .find(|m| m.module.module_type == ModuleType::Advanced)
            .map(|m| m.module.uni_equip_id.as_str());
        let skills = op.skills.len().max(1);
        for si in 0..skills {
            let a = analyze(
                data,
                &req(vec![m(
                    id,
                    phase,
                    level,
                    5,
                    i32::try_from(si).unwrap_or(0),
                    7,
                    3,
                    module,
                    3,
                )]),
            );
            if si == 0 {
                for i in &a.synergy.items {
                    synergy_items += 1;
                    let _ = writeln!(
                        synergy,
                        "{} | {} | {:?} {:?} {} | {:?} | {:?}",
                        op.name,
                        i.name,
                        i.target.kind,
                        i.target.value,
                        i.target.label,
                        i.cond,
                        i.group
                    );
                }
            }
            for g in &a.coverage {
                for r in &g.rows {
                    for o in &r.ops {
                        let key = format!("{:?}", r.key);
                        if seen_rows.entry(key.clone()).or_default().insert(id.clone()) {
                            rows.entry(key).or_default().push(format!(
                                "{} ({:?}{})",
                                op.name,
                                o.why.kind,
                                o.why
                                    .name
                                    .as_ref()
                                    .map(|n| format!(" {n}"))
                                    .unwrap_or_default()
                            ));
                        }
                    }
                }
            }
        }
    }
    let mut cov = String::new();
    for (k, v) in &rows {
        let _ = writeln!(
            cov,
            "{k}: {} | {}",
            v.len(),
            v.iter().take(15).cloned().collect::<Vec<_>>().join(", ")
        );
    }
    let _ = writeln!(synergy, "TOTAL {synergy_items}");
    std::fs::write(format!("{SCRATCH}/synergy_census.txt"), synergy).ok();
    std::fs::write(format!("{SCRATCH}/coverage_census.txt"), cov).ok();
    println!("synergy items: {synergy_items}");
}

/// One line per analysis panel fact, for a before/after diff of a rule change.
fn census_lines(a: &SquadAnalysis) -> Vec<String> {
    let mut out = Vec::new();
    for g in &a.coverage {
        for r in &g.rows {
            for o in &r.ops {
                out.push(format!(
                    "cov {:?} {:?} {}",
                    r.key,
                    o.why.kind,
                    o.why.name.clone().unwrap_or_default()
                ));
            }
        }
    }
    for i in &a.synergy.items {
        out.push(format!(
            "syn {} | {:?} {:?} | {:?} {:?} {:?} | to {:?} all {} need {:?} hi {}",
            i.name,
            i.target.kind,
            i.target.value,
            i.cond,
            i.group,
            i.reason,
            i.to,
            i.to_all,
            i.need.as_ref().map(|n| (n.count, n.have)),
            i.highest_only
        ));
    }
    for s in &a.economy.seats {
        out.push(format!(
            "eco cost {} respawn {:.3} cuts {:?} rcuts {:?}",
            s.cost,
            s.respawn,
            s.cuts
                .iter()
                .map(|c| (c.kind, c.amount, c.ratio))
                .collect::<Vec<_>>(),
            s.respawn_cuts
                .iter()
                .map(|c| (c.kind, c.amount, c.ratio))
                .collect::<Vec<_>>()
        ));
    }
    for d in &a.economy.sources {
        out.push(format!(
            "src {:?} {} kill {} refund {} first {:?}",
            d.source.kind, d.amount, d.per_kill, d.refund, d.first_ready
        ));
    }
    for n in a.economy.notes.iter().chain(&a.economy.counted) {
        out.push(format!("note {:?} {} {:?}", n.kind, n.name, n.amount));
    }
    for b in &a.economy.start_bonus {
        out.push(format!("bonus {} {}", b.name, b.amount));
    }
    for r in &a.readiness.rows {
        out.push(format!(
            "ready sl {} m {} ml {} unlock {:?} flags {:?}",
            r.skill_level,
            r.mastery,
            r.module_level,
            r.skill_unlock.as_ref().map(|u| (u.phase, u.level)),
            r.flags
                .iter()
                .map(|f| (f.kind, f.value, f.reference))
                .collect::<Vec<_>>()
        ));
    }
    out
}

/// Every operator, every skill, at a full build and at an Elite 0 level 1
/// build asking for rank 7 M3: every fact the panels report, one line each,
/// written to `full_census.txt` so a rule change can be diffed operator by
/// operator.
#[test]
fn full_census_dump() {
    let en = common::load_game_data();
    census_to(en, en, "full_census.txt");
    // The CN tables read through EN's text, for an operator-by-operator
    // comparison with the EN dump. CI has no CN tree.
    if let Some(cn) = common::load_server_with_en_art("cn") {
        census_to(&cn, en, "full_census_cn.txt");
    }
}

fn census_to(data: &GameData, english: &GameData, file: &str) {
    let analyze = |d: &GameData, r: &SquadAnalyzeRequest| analyze_with(d, Some(english), r);
    let mut ids: Vec<&String> = data
        .operators
        .keys()
        .filter(|k| k.starts_with("char_"))
        .collect();
    ids.sort();
    let mut text = String::new();
    for id in ids {
        let op = &data.operators[id];
        let phase = i32::try_from(op.phases.len().saturating_sub(1)).unwrap_or(0);
        let level = op.phases.last().map_or(1, |p| p.max_level);
        let module = op
            .modules
            .iter()
            .find(|m| m.module.module_type == ModuleType::Advanced)
            .map(|m| m.module.uni_equip_id.as_str());
        for si in 0..op.skills.len().max(1) {
            let si = i32::try_from(si).unwrap_or(0);
            for (tag, b) in [
                ("full", m(id, phase, level, 5, si, 7, 3, module, 3)),
                ("low", m(id, 0, 1, 0, si, 7, 3, module, 3)),
            ] {
                let a = analyze(data, &req(vec![b]));
                for l in census_lines(&a) {
                    let _ = writeln!(text, "{id} s{si} {tag} | {l}");
                }
            }
            // With receivers of four classes and SP types, so squad effects
            // show who they reach.
            if si == 0 {
                let a = analyze(
                    data,
                    &req(vec![
                        m(id, phase, level, 5, 0, 7, 3, module, 3),
                        m("char_151_myrtle", 2, 70, 0, 0, 7, 0, None, 1),
                        m("char_103_angel", 2, 80, 0, 2, 7, 3, None, 1),
                        m("char_002_amiya", 2, 80, 0, 1, 7, 3, None, 1),
                        m("char_202_demkni", 2, 90, 0, 1, 7, 3, None, 1),
                    ]),
                );
                for l in census_lines(&a) {
                    let _ = writeln!(text, "{id} s{si} squad | {l}");
                }
            }
        }
    }
    std::fs::write(format!("{SCRATCH}/{file}"), text).ok();
}

const fn u(phase: i32, level: i32) -> Unlock {
    Unlock { phase, level }
}

/// Skill unlocks, ranks and masteries come from `character_table`, the one
/// source the analysis, the save checks and the page read.
#[test]
fn skill_rules_from_the_table() {
    let data = common::load_game_data();
    // Archetto: `Skills[i].UnlockCond` PHASE_0/1/2; `AllSkillLvlup` ranks 2-4
    // at PHASE_0 and 5-7 at PHASE_1; three masteries per skill at PHASE_2.
    let r = SkillRules::of(&data.operators[ARCH]);
    assert_eq!(r.max_rank(), 7);
    assert_eq!(
        r.skills
            .iter()
            .map(|s| s.unlock.clone())
            .collect::<Vec<_>>(),
        vec![u(0, 1), u(1, 1), u(2, 1)]
    );
    assert_eq!(r.rank_unlock(4), Some(u(0, 1)));
    assert_eq!(r.rank_unlock(5), Some(u(1, 1)));
    assert_eq!(r.rank_unlock(1), None);
    assert_eq!(
        (0..3).map(|i| r.max_mastery(i)).collect::<Vec<_>>(),
        vec![3, 3, 3]
    );
    assert_eq!(r.mastery_unlock(2, 3), Some(u(2, 1)));
    // Reserve Operator - Guard (4-star): S3 unlocks at PHASE_1, and S1 has an
    // empty `LevelUpCostCond`, so no mastery, while S2 and S3 have three.
    let g = SkillRules::of(&data.operators["char_601_cguard"]);
    assert_eq!(g.skill_unlock(2), Some(u(1, 1)));
    assert_eq!(
        (0..3).map(|i| g.max_mastery(i)).collect::<Vec<_>>(),
        vec![0, 3, 3]
    );
    // Kroos (3-star): ranks to 7, no mastery. 12F (2-star): no rank-ups.
    let k = SkillRules::of(&data.operators["char_124_kroos"]);
    assert_eq!((k.max_rank(), k.max_mastery(0)), (7, 0));
    assert_eq!(
        SkillRules::of(&data.operators["char_009_12fce"]).max_rank(),
        1
    );

    // The analysis clamps to them: Kroos asked for M3 gets M0 and no
    // mastery flag; the old hand rule asked her for Elite 2, which a 3-star
    // cannot reach.
    let a = analyze(
        data,
        &req(vec![m("char_124_kroos", 1, 55, 0, 0, 7, 3, None, 1)]),
    );
    let row = &a.readiness.rows[0];
    assert_eq!((row.skill_level, row.mastery), (7, 0));
    assert!(row.flags.is_empty(), "{:?}", row.flags);
    // Reserve Guard S1 asked for M3 at E1 gets M0; S2 keeps M3 and is flagged
    // for its PHASE_2 mastery unlock.
    let a = analyze(
        data,
        &req(vec![m("char_601_cguard", 1, 55, 0, 0, 7, 3, None, 1)]),
    );
    assert_eq!(a.readiness.rows[0].mastery, 0);
    let a = analyze(
        data,
        &req(vec![m("char_601_cguard", 1, 55, 0, 1, 7, 3, None, 1)]),
    );
    let f = &a.readiness.rows[0].flags;
    assert!(
        f.iter()
            .any(|f| f.kind == ReadinessFlagKind::SkillRankLocked
                && (f.value, f.reference) == (Some(2), Some(1)))
    );
}

/// The stage threat thresholds are the game's grade bands, and the challenge
/// stage link is the table's `HardStagedId`.
#[test]
fn load_time_tables() {
    let data = common::load_game_data();
    let t = &data.squad.threats;
    // `LevelInfoList`: RES grade A from 50, DEF grade A from 1000, move speed
    // grade S from 1.5: the numbers the page had typed in.
    assert_eq!(t.high_res, Some(50.0));
    assert_eq!(t.high_def, Some(1000.0));
    assert_eq!(t.fast_move_speed, Some(1.5));
    assert_eq!(
        data.squad
            .normal_of
            .get("main_00-01#f#")
            .map(String::as_str),
        Some("main_00-01")
    );
    // Every challenge id the table links ends in `#f#` and maps to its strip.
    assert!(data.squad.normal_of.len() > 700);
    assert!(
        data.squad
            .normal_of
            .iter()
            .all(|(c, n)| c.strip_suffix("#f#") == Some(n.as_str()))
    );
    assert!(data.squad.text.len() > 10_000);
}

/// A faction talent names a `handbook_team_table` power id and reaches by id.
#[test]
fn faction_targets_are_power_ids() {
    let data = common::load_game_data();
    let a = analyze(
        data,
        &req(vec![
            m("char_1023_ghost2", 2, 90, 0, 0, 7, 3, None, 1),
            m("char_263_skadi", 2, 90, 0, 0, 7, 3, None, 1),
        ]),
    );
    let item = syn(&a, "char_1023_ghost2", "Ægir's Depths").expect("Ægir's Depths");
    assert_eq!(item.target.kind, SynergyTargetKind::Faction);
    assert_eq!(item.target.value.as_deref(), Some("abyssal"));
    assert_eq!(item.to, vec![1], "Skadi's `GroupId` is abyssal");
}

/// The CN tables, read through EN's text readings by id: the same reading
/// for a shared operator, CN's own words in the labels, and a CN-only
/// operator listed as unread rather than skipped. Skipped where the CN tree
/// is not on disk (CI carries EN only).
#[test]
fn cn_server_reads_through_en_text() {
    let Some(cn) = common::load_server_with_en_art("cn") else {
        eprintln!("no CN tree on disk, skipped");
        return;
    };
    let en = common::load_game_data();
    let zima = |data| {
        analyze_with(
            data,
            Some(en),
            &req(vec![
                m("char_115_headbr", 2, 70, 0, 0, 7, 0, None, 1),
                m("char_151_myrtle", 2, 70, 0, 0, 7, 0, None, 1),
                m(BPIPE, 2, 60, 0, 2, 7, 3, None, 1),
            ]),
        )
    };
    let (a_en, a_cn) = (zima(en), zima(&cn));
    assert!(a_cn.unread_seats.is_empty());
    // Zima's squad cut reaches Myrtle on CN exactly as on EN.
    for a in [&a_en, &a_cn] {
        let myrtle = eco(a, "char_151_myrtle");
        assert!(
            myrtle
                .cuts
                .iter()
                .any(|c| c.kind == CutKind::Squad && c.from_seat == Some(0))
        );
    }
    let shape = |a: &SquadAnalysis| {
        a.synergy
            .items
            .iter()
            .map(|i| {
                (
                    i.seat,
                    i.target.kind,
                    i.target.value.clone(),
                    i.cond,
                    i.group,
                    i.to.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(shape(&a_en), shape(&a_cn));
    let covered = |a: &SquadAnalysis| {
        a.coverage
            .iter()
            .flat_map(|g| &g.rows)
            .flat_map(|r| r.ops.iter().map(move |o| (r.key, o.seat, o.why.kind)))
            .collect::<Vec<_>>()
    };
    assert_eq!(covered(&a_en), covered(&a_cn));
    // Bagpipe's "Martial Tradition" names Vanguards: CN's own class name.
    let mt = a_cn
        .synergy
        .items
        .iter()
        .find(|i| i.char_id == BPIPE && i.target.kind == SynergyTargetKind::Class)
        .expect("Bagpipe class talent");
    assert_eq!(mt.target.label, cn.profession_names["PIONEER"]);
    assert_ne!(mt.target.label, "Vanguard");

    // 乌啾 (char_4224_turdus) is on CN only. Her talents come back Unread,
    // and the rules that read blackboards still see her: a Chain Healer's
    // `attack@chain.max_target` trait heals several.
    assert!(!en.operators.contains_key("char_4224_turdus"));
    let a = analyze_with(
        &cn,
        Some(en),
        &req(vec![m("char_4224_turdus", 2, 80, 0, 0, 7, 3, None, 1)]),
    );
    assert_eq!(a.unread_seats, vec![0]);
    assert!(!a.synergy.items.is_empty());
    assert!(
        a.synergy
            .items
            .iter()
            .all(|i| i.reason == Some(SynergyReason::Unread))
    );
    assert_eq!(
        cov(&a, CoverageRowKey::HealArea, "char_4224_turdus").map(|o| o.why.kind),
        Some(SourceKind::Trait)
    );
    // Recruitment tags resolve by id on CN: her 治疗 is not a fallback row,
    // but Myrtle's 费用回复 is `GachaTags` 18, DP-Recovery.
    let myrtle = &cn.operators["char_151_myrtle"];
    assert!(cn.squad.tags.ids_of(myrtle).contains(&18));
}

/// Timing, run by hand: `cargo test --test squad_analysis_test perf -- --ignored --nocapture`.
/// What one analysis costs now, what the load-time index costs once, and
/// what the per-request faction table the analysis used to rebuild costs.
#[test]
#[ignore = "timing only"]
fn perf_analyze_timing() {
    use std::time::Instant;
    let data = common::load_game_data();
    let r = reference(data);
    let n = 200u32;
    let t = Instant::now();
    for _ in 0..n {
        std::hint::black_box(analyze(data, &r));
    }
    let per = t.elapsed() / n;
    let t = Instant::now();
    let idx = backend::core::squad::index::SquadIndex::build(data);
    let build = t.elapsed();
    // The old per-request `Factions::from_data`: every operator's faction
    // names, normalized, into a fresh set.
    let t = Instant::now();
    for _ in 0..n {
        let set: std::collections::HashSet<String> = data
            .operators
            .values()
            .flat_map(|op| {
                [&op.nation_name, &op.group_name, &op.team_name]
                    .into_iter()
                    .flatten()
                    .cloned()
                    .chain(op.hidden_factions.iter().filter_map(|f| f.name.clone()))
                    .collect::<Vec<_>>()
            })
            .map(|s| {
                s.chars()
                    .filter(char::is_ascii_alphanumeric)
                    .map(|c| c.to_ascii_lowercase())
                    .collect::<String>()
            })
            .collect();
        std::hint::black_box(set);
    }
    let factions = t.elapsed() / n;
    println!(
        "analyze (6 seats): {per:?}/request; old Factions::from_data: {factions:?}/request; load-time index: {build:?} once, {} text readings",
        idx.text.len()
    );
}

/// The ratio-or-flat read of a talent's `atk` / `respawn_time` comes from how
/// its text prints the value (`TalentText::scale_of`), not its magnitude.
/// Census of every EN talent candidate with a non-zero value, and the four
/// candidates where the old |v| < 1 guess was wrong.
#[test]
fn talent_scale_reads_the_printed_number() {
    use backend::core::squad::index::{StatScale, TextKey};
    let data = common::load_game_data();
    let mut ids: Vec<&String> = data.operators.keys().collect();
    ids.sort();
    let (mut seen, mut read, mut against) = (0, 0, Vec::new());
    for id in ids {
        let op = &data.operators[id];
        for (ti, t) in op.talents.iter().enumerate() {
            for (ci, c) in t.candidates.iter().enumerate() {
                let key = TextKey::Talent {
                    char_id: id.clone(),
                    talent: ti,
                    cand: ci,
                };
                let Some(tt) = data.squad.text.get(&key).and_then(|f| f.talent.as_ref()) else {
                    continue;
                };
                for (stat, complement) in [("atk", false), ("respawn_time", true)] {
                    let Some(v) = c
                        .blackboard
                        .iter()
                        .find(|b| b.key.eq_ignore_ascii_case(stat))
                        .map(|b| b.value)
                        .filter(|v| *v != 0.0)
                    else {
                        continue;
                    };
                    seen += 1;
                    let Some(scale) = tt.scale_of(v, complement) else {
                        continue;
                    };
                    read += 1;
                    let guess = if v.abs() < 1.0 {
                        StatScale::Ratio
                    } else {
                        StatScale::Flat
                    };
                    if scale != guess {
                        against.push(format!("{id} {stat} {v}"));
                    }
                }
            }
        }
    }
    eprintln!("talent scale: {read} of {seen} read, against the magnitude guess: {against:?}");
    assert_eq!((seen, read), (365, 359));
    assert_eq!(against.len(), 4, "{against:?}");
    assert!(
        against
            .iter()
            .all(|a| a.starts_with("char_290_vigna") || a.starts_with("char_411_tomimi"))
    );
}
