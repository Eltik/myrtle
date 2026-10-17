//! Worker tests (contract v1.1): the power guard's decision on injected samples, the heartbeat's
//! `guard` object, and the `mid` profiles.

use crate::guard::{Change, Guard, GuardConfig, GuardView, Sample, Trip};

fn cfg() -> GuardConfig {
    GuardConfig { cooldown_secs: 300, ..GuardConfig::default() }
}

fn psi_cpu(v: f64) -> Sample {
    Sample { psi_cpu: Some(v), ..Sample::default() }
}

#[test]
fn guard_trips_after_three_consecutive_bad_samples_of_one_signal() {
    let mut g = Guard::new(cfg());
    assert_eq!(g.observe(&psi_cpu(70.0), 0), None);
    assert_eq!(g.observe(&psi_cpu(70.0), 5), None);
    // A good sample in between restarts the run.
    assert_eq!(g.observe(&psi_cpu(10.0), 10), None);
    assert_eq!(g.observe(&psi_cpu(70.0), 15), None);
    assert_eq!(g.observe(&psi_cpu(70.0), 20), None);
    match g.observe(&psi_cpu(70.0), 25) {
        Some(Change::Tripped(Trip { reason, since: 25, memory: false })) =>
            assert!(reason.starts_with("psi_cpu 70.0 > 60"), "{reason}"),
        other => panic!("expected a cpu trip, got {other:?}"),
    }
    // At the threshold is not above it; a missing signal is never bad.
    let mut g = Guard::new(cfg());
    for t in 0..10 {
        assert_eq!(g.observe(&psi_cpu(60.0), t), None);
        assert_eq!(g.observe(&Sample::default(), t), None);
    }
}

#[test]
fn guard_needs_the_same_signal_three_times_and_flags_memory_trips() {
    let mut g = Guard::new(cfg());
    // Three bad samples, but of alternating signals: no single signal held for 3.
    let mem = Sample { mem_avail_mb: Some(500.0), ..Sample::default() };
    assert_eq!(g.observe(&psi_cpu(90.0), 0), None);
    assert_eq!(g.observe(&mem, 5), None);
    assert_eq!(g.observe(&psi_cpu(90.0), 10), None);
    assert!(g.trip().is_none());
    // RSS and available memory are memory signals: a trip on them stops the llama-server.
    let mut g = Guard::new(cfg());
    let rss = Sample { rss_mb: Some(5000.0), load_per_cpu: Some(0.2), ..Sample::default() };
    g.observe(&rss, 0);
    g.observe(&rss, 5);
    assert!(matches!(g.observe(&rss, 10), Some(Change::Tripped(Trip { memory: true, .. }))));
    // A cpu trip that is later joined by a memory signal escalates once.
    let mut g = Guard::new(cfg());
    for t in 0..3 {
        g.observe(&Sample { load_per_cpu: Some(2.0), ..Sample::default() }, t);
    }
    assert!(matches!(g.trip(), Some(Trip { memory: false, .. })));
    let both = Sample { load_per_cpu: Some(2.0), psi_mem: Some(30.0), ..Sample::default() };
    assert_eq!(g.observe(&both, 3), None);
    assert_eq!(g.observe(&both, 4), None);
    match g.observe(&both, 5) {
        Some(Change::Tripped(t)) =>
            assert!(t.memory && t.since == 2 && t.reason.contains("psi_mem 30.0 > 20"), "{t:?}"),
        other => panic!("expected an escalation, got {other:?}"),
    }
    assert_eq!(g.observe(&both, 6), None, "escalates once");
}

#[test]
fn guard_recovers_only_after_the_cooldown_with_every_signal_under_80_percent() {
    let mut g = Guard::new(cfg());
    for t in 0..3 {
        g.observe(&psi_cpu(80.0), t * 5);
    }
    assert!(g.trip().is_some());
    // 55 is under the threshold but not under 80% of it (48): not calm.
    assert_eq!(g.observe(&psi_cpu(55.0), 20), None);
    assert_eq!(g.observe(&psi_cpu(55.0), 400), None);
    // Calm from 405; one warm sample at 600 restarts the cool-down.
    assert_eq!(g.observe(&psi_cpu(40.0), 405), None);
    assert_eq!(g.observe(&psi_cpu(40.0), 600), None);
    assert_eq!(g.observe(&psi_cpu(50.0), 650), None);
    assert_eq!(g.observe(&psi_cpu(40.0), 700), None);
    assert_eq!(g.observe(&psi_cpu(40.0), 999), None);
    assert_eq!(g.observe(&psi_cpu(40.0), 1000), Some(Change::Recovered));
    assert!(g.trip().is_none());
    // After recovery the run starts from zero.
    assert_eq!(g.observe(&psi_cpu(80.0), 1005), None);
    // A floor signal is calm only above threshold / 0.8 (1,000 MiB for 800).
    let mut g = Guard::new(GuardConfig { cooldown_secs: 10, ..cfg() });
    let mem = |v: f64| Sample { mem_avail_mb: Some(v), ..Sample::default() };
    for t in 0..3 {
        g.observe(&mem(700.0), t);
    }
    assert_eq!(g.observe(&mem(900.0), 10), None);
    assert_eq!(g.observe(&mem(900.0), 30), None, "900 MiB is over the threshold but not calm");
    assert_eq!(g.observe(&mem(1100.0), 31), None);
    assert_eq!(g.observe(&mem(1100.0), 41), Some(Change::Recovered));
}

#[test]
fn guard_mac_signals_trip_on_critical_pressure_and_thermal_warnings() {
    let mut g = Guard::new(GuardConfig { cooldown_secs: 0, ..cfg() });
    let p = |level: u32, thermal: bool| Sample { mac_pressure: Some(level), thermal: Some(thermal),
                                                 mem_avail_mb: Some(9000.0), ..Sample::default() };
    // Level 2 (warning) is an everyday state on a Mac: it never trips.
    for t in 0..5 {
        assert_eq!(g.observe(&p(2, false), t), None);
    }
    g.observe(&p(4, false), 5);
    g.observe(&p(4, false), 6);
    assert!(matches!(g.observe(&p(4, false), 7), Some(Change::Tripped(Trip { memory: true, .. }))));
    assert_eq!(g.observe(&p(2, false), 8), Some(Change::Recovered), "level 2 is under 80% of 4; cool-down 0");
    let mut g = Guard::new(cfg());
    g.observe(&p(1, true), 0);
    g.observe(&p(1, true), 5);
    assert!(matches!(g.observe(&p(1, true), 10), Some(Change::Tripped(Trip { memory: false, .. }))));
}

#[test]
fn guard_config_reads_the_contract_env_and_the_kill_switch() {
    let env = |pairs: &'static [(&'static str, &'static str)]|
        move |k: &str| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| (*v).to_owned());
    assert_eq!(GuardConfig::from_lookup(env(&[("TREVOR_GUARD", "0")])), None);
    let d = GuardConfig::from_lookup(env(&[])).unwrap();
    assert_eq!((d.psi_cpu, d.psi_mem, d.mem_mb, d.rss_mb, d.load, d.steal, d.need, d.cooldown_secs, d.every.as_secs()),
               (60.0, 20.0, 800.0, 4500.0, 1.5, None, 3, 300, 5));
    let c = GuardConfig::from_lookup(env(&[("TREVOR_GUARD", "1"), ("TREVOR_GUARD_PSI_CPU", "40"),
                                           ("TREVOR_GUARD_MEM_MB", "2000"), ("TREVOR_GUARD_RSS_MB", "x"),
                                           ("TREVOR_GUARD_COOLDOWN_SECS", "20"), ("TREVOR_GUARD_STEAL", "25"),
                                           ("TREVOR_GUARD_SECS", "1")])).unwrap();
    assert_eq!((c.psi_cpu, c.mem_mb, c.rss_mb, c.cooldown_secs, c.steal, c.every.as_secs()),
               (40.0, 2000.0, 4500.0, 20, Some(25.0), 1));
}

#[test]
fn heartbeat_carries_the_guard_only_when_it_is_on() {
    use crate::serve::worker::heartbeat_body;
    let rungs = serde_json::json!({"cf": false, "cpu": false});
    let off = heartbeat_body("w", rungs.clone(), "en-1", None, None);
    assert_eq!(off, serde_json::json!({"workerId": "w", "rungs": rungs, "corpusVersion": "en-1", "busyJobId": null}),
               "TREVOR_GUARD=0: v1 body");
    let ok = heartbeat_body("w", rungs.clone(), "en-1", None, Some(&GuardView::default()));
    assert_eq!(ok["guard"], serde_json::json!({"state": "ok", "reason": null, "since": null}));
    let t = GuardView { trip: Some(Trip { reason: "psi_cpu 70.0 > 60 for 3 samples".into(), since: 1_760_000_000,
                                          memory: false }) };
    let tripped = heartbeat_body("w", rungs, "en-1", Some("j1".into()), Some(&t));
    assert_eq!(tripped["guard"], serde_json::json!({"state": "tripped", "reason": "psi_cpu 70.0 > 60 for 3 samples",
                                                    "since": 1_760_000_000u64}));
    assert_eq!(tripped["busyJobId"], "j1");
}

/// `--profile mid` is exactly the eval arm e-e2b-mid36 (`--profile full` plus queue_mid.sh's flags), and `mid6k` the arm
/// f-e2b-mid6k: every field of the parsed and profiled arguments is equal but the profile name.
#[test]
fn profile_mid_is_exactly_the_eval_arm_flags() {
    use crate::cli::{Args, Profile, apply_profile};
    let parse = |v: &[&str]| {
        let mut a = <Args as clap::Parser>::parse_from(std::iter::once("ask").chain(v.iter().copied()));
        let p = a.profile;
        apply_profile(&mut a, p);
        a
    };
    let arm = ["--router", "keywords", "--k", "8", "--n-predict", "250", "--no-typed-fallback", "--no-answer-extend",
               "--no-answer-check", "--no-operator-list-check"];
    for (profile, budget) in [("mid", "3600"), ("mid6k", "6000")] {
        let mut flags = vec!["q", "--profile", "full", "--max-context-tokens", budget];
        flags.extend(arm);
        let mut by_flags = parse(&flags);
        let by_profile = parse(&["q", "--profile", profile]);
        assert_eq!(by_profile.profile.name(), profile);
        by_flags.profile = by_profile.profile;
        assert_eq!(format!("{by_flags:?}"), format!("{by_profile:?}"), "{profile}");
        assert!(!crate::cli::form_on(&by_profile), "one call: no form call under the keyword router");
    }
    assert_eq!(Profile::NAMES.iter().map(|n| Profile::parse(n).map(Profile::name)).collect::<Vec<_>>(),
               Profile::NAMES.iter().map(|n| Some(*n)).collect::<Vec<_>>());
    assert_eq!(Profile::parse("medium"), None);
}

#[test]
fn the_cpu_rung_defaults_to_mid_and_slim_is_the_kill_switch() {
    use crate::serve::worker::default_profile;
    assert_eq!(default_profile("cf", None), "full");
    assert_eq!(default_profile("cf", Some("slim")), "full");
    assert_eq!(default_profile("cpu", None), "mid");
    assert_eq!(default_profile("cpu", Some("slim")), "slim");
    assert_eq!(default_profile("cpu", Some("mid6k")), "mid6k");
    assert_eq!(default_profile("cpu", Some("bogus")), "mid");
    assert_eq!(default_profile("passages", None), "mid");
}
