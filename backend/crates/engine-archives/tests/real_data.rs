//! The Archives over the real tables: parser coverage per profile field on
//! every extracted server (EN, CN, JP, KR), the network chart, and the counts
//! the page leads with. Each server skips when `assets/output/<server>` is
//! absent (CI has no extracted assets).
//!
//! `cargo test --test archives_real_data_test -- --nocapture` prints the
//! coverage table.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use myrtle_engine_archives::build::{card_faction, file_ids};
use myrtle_engine_archives::profile::{
    ArchiveGender, InfectionState, Lang, bracketed, parse_archive_profile,
};
use myrtle_engine_archives::{Archive, SceneIndex, build_archive};
use myrtle_gamedata::{
    compile::{CompileOptions, compile},
    model::GameModel,
};
use myrtle_gamedata_raw::tables::handbook_network::RawNetwork;
use myrtle_gamedata_raw::{DirSource, TableSource, load as load_raw};
use myrtle_testkit::real_data::excel_dir;
use myrtle_types::Server;

fn server_root(server: &str) -> PathBuf {
    excel_dir().join("../../..").join(server)
}

fn root() -> PathBuf {
    server_root("en")
}

fn server_of(server: &str) -> Server {
    match server {
        "en" => Server::EN,
        "cn" => Server::CN,
        "jp" => Server::JP,
        "kr" => Server::KR,
        other => panic!("no server {other}"),
    }
}

/// `None` when the server's tree is not on disk, which is CI for anything but EN
fn compile_server(server: &str) -> Option<GameModel> {
    let root = server_root(server);
    if !root.join("gamedata/excel").is_dir() {
        return None;
    }
    let raw = load_raw(&DirSource { root }, server_of(server)).ok()?;
    Some(
        compile(raw, &CompileOptions::new(server_of(server)))
            .ok()?
            .model,
    )
}

static EN: LazyLock<Option<GameModel>> = LazyLock::new(|| compile_server("en"));
static CN: LazyLock<Option<GameModel>> = LazyLock::new(|| compile_server("cn"));
static JP: LazyLock<Option<GameModel>> = LazyLock::new(|| compile_server("jp"));
static KR: LazyLock<Option<GameModel>> = LazyLock::new(|| compile_server("kr"));

/// Each server's model, compiled once per test binary
fn load(server: &str) -> Option<&'static GameModel> {
    match server {
        "en" => EN.as_ref(),
        "cn" => CN.as_ref(),
        "jp" => JP.as_ref(),
        "kr" => KR.as_ref(),
        other => panic!("no server {other}"),
    }
}

fn lang(server: &str) -> Lang {
    Lang::for_server(server_of(server)).expect("a read language")
}

/// The network tables as the app reads them, `None` when either is absent or unparsable
fn load_network(root: &std::path::Path) -> Option<RawNetwork> {
    let source = DirSource {
        root: root.to_path_buf(),
    };
    let force_map = source
        .read_gamedata("art/handbook_force_map_table.json.json")
        .ok()??;
    let positions = source
        .read_gamedata("art/handbookpos_table.json.json")
        .ok()??;
    RawNetwork::from_slices(&force_map, &positions).ok()
}

const FIELDS: [&str; 12] = [
    "code name",
    "gender",
    "race",
    "place of birth",
    "date of birth",
    "height",
    "combat experience",
    "infection",
    "blood density",
    "assimilation",
    "manufacturer",
    "exam rating",
];

/// One server's parser coverage: per field, how many files state it and how
/// many of those read into a value, the values left unread, the infection
/// and gender tallies.
#[derive(Default)]
struct Coverage {
    files: usize,
    stated: HashMap<&'static str, usize>,
    parsed: HashMap<&'static str, usize>,
    misses: HashMap<&'static str, Vec<String>>,
    infection: HashMap<InfectionState, usize>,
    unclear: Vec<(String, String)>,
    genders: HashMap<String, usize>,
}

fn coverage(server: &str) -> Option<Coverage> {
    let gd = load(server)?;
    let ids = file_ids(gd);
    let mut c = Coverage {
        files: ids.len(),
        ..Coverage::default()
    };
    for id in &ids {
        let item = &gd.handbook.handbook_dict[id];
        let Some(p) = parse_archive_profile(lang(server), &item.story_text_audio) else {
            continue;
        };
        let mut tally = |field: &'static str, raw: &Option<String>, ok: bool| {
            if let Some(raw) = raw {
                *c.stated.entry(field).or_default() += 1;
                if ok {
                    *c.parsed.entry(field).or_default() += 1;
                } else {
                    c.misses.entry(field).or_default().push(raw.clone());
                }
            }
        };
        tally("code name", &p.code_name, true);
        tally("gender", &p.gender, true);
        tally("race", &p.race, true);
        tally("place of birth", &p.birthplace, true);
        tally("date of birth", &p.birthday, p.birth_month.is_some());
        tally("height", &p.height, p.height_cm.is_some());
        tally(
            "combat experience",
            &p.combat_experience,
            p.combat_years.is_some(),
        );
        tally(
            "infection",
            &p.infection,
            matches!(
                p.infection_state,
                InfectionState::Infected | InfectionState::Clear
            ),
        );
        tally(
            "blood density",
            &p.blood_density,
            p.blood_density_ul.is_some(),
        );
        tally(
            "assimilation",
            &p.assimilation,
            p.assimilation_pct.is_some(),
        );
        tally("manufacturer", &p.manufacturer, true);
        for r in &p.exam {
            tally("exam rating", &Some(r.raw.clone()), r.step.is_some());
        }
        *c.infection.entry(p.infection_state).or_default() += 1;
        if p.infection_state == InfectionState::Unclear {
            c.unclear
                .push((id.clone(), p.infection.clone().unwrap_or_default()));
        }
        if let Some(g) = p.gender.as_deref() {
            let key = ArchiveGender::read(lang(server).vocab(), g)
                .map_or_else(|| format!("unread {g}"), |k| format!("{k:?}"));
            *c.genders.entry(key).or_default() += 1;
        }
    }
    Some(c)
}

fn print_coverage(server: &str, c: &Coverage) {
    println!("== {server}: files {}", c.files);
    println!("{:<20} {:>7} {:>7}", "field", "stated", "parsed");
    for f in FIELDS {
        println!(
            "{:<20} {:>7} {:>7}",
            f,
            c.stated.get(f).copied().unwrap_or(0),
            c.parsed.get(f).copied().unwrap_or(0)
        );
    }
    for f in FIELDS {
        if let Some(m) = c.misses.get(f) {
            println!("unread {f}: {m:?}");
        }
    }
    let state = |s| c.infection.get(&s).copied().unwrap_or(0);
    println!(
        "infection: infected {} clear {} unclear {} unstated {}",
        state(InfectionState::Infected),
        state(InfectionState::Clear),
        state(InfectionState::Unclear),
        state(InfectionState::Unstated)
    );
    let mut genders: Vec<_> = c.genders.iter().collect();
    genders.sort();
    println!("gender: {genders:?}");
}

/// Floors per server from the 2026-10-11 tables (EN 2026-09-05 extract, CN,
/// JP and KR 2026-10-06); a drop means the parser regressed or the wording
/// moved. Fields left out of a row have no floor on that server.
fn floors(server: &str) -> &'static [(&'static str, usize)] {
    match server {
        "en" => &[
            ("gender", 405),
            ("height", 405),
            ("combat experience", 355),
            ("date of birth", 385),
            ("infection", 395),
            ("blood density", 375),
            ("assimilation", 390),
            ("exam rating", 2390),
        ],
        "cn" => &[
            ("code name", 410),
            ("gender", 420),
            ("race", 420),
            ("place of birth", 420),
            ("height", 415),
            ("combat experience", 365),
            ("date of birth", 400),
            ("infection", 415),
            ("blood density", 385),
            ("assimilation", 410),
            ("manufacturer", 8),
            ("exam rating", 2480),
        ],
        "jp" => &[
            ("code name", 405),
            ("gender", 405),
            ("race", 405),
            ("place of birth", 405),
            ("height", 405),
            ("combat experience", 355),
            ("date of birth", 385),
            ("infection", 395),
            ("blood density", 375),
            ("assimilation", 390),
            ("manufacturer", 7),
            ("exam rating", 2390),
        ],
        "kr" => &[
            ("code name", 400),
            ("gender", 405),
            ("race", 405),
            ("place of birth", 405),
            ("height", 405),
            ("combat experience", 355),
            ("date of birth", 385),
            ("infection", 395),
            ("blood density", 375),
            ("assimilation", 390),
            ("manufacturer", 7),
            ("exam rating", 2390),
        ],
        _ => &[],
    }
}

#[test]
fn profile_parser_coverage() {
    let mut any = false;
    for server in ["en", "cn", "jp", "kr"] {
        let Some(c) = coverage(server) else {
            eprintln!("skipping {server}: assets/output/{server} missing");
            continue;
        };
        any = true;
        print_coverage(server, &c);
        assert!(c.files >= 410, "{server} files {}", c.files);
        for (f, n) in floors(server) {
            let got = c.parsed.get(f).copied().unwrap_or(0);
            assert!(got >= *n, "{server} {f}: {got} < {n}");
        }
    }
    if !any {
        eprintln!("skipping: no extracted server");
    }
}

#[test]
fn every_infection_sentence_is_read_or_named() {
    for server in ["en", "cn", "jp", "kr"] {
        let Some(c) = coverage(server) else {
            eprintln!("skipping {server}: assets/output/{server} missing");
            continue;
        };
        println!("{server} unclear infection lines: {:#?}", c.unclear);
        // Each of these is a sentence no keyword reads safely (Undisclosed, a
        // corrupted line). More than a handful means a wording slipped through.
        assert!(c.unclear.len() <= 8, "{server} {}", c.unclear.len());
    }
}

#[test]
fn network_and_counts() {
    let Some(en) = load("en") else {
        eprintln!("skipping: assets/output/en missing");
        return;
    };
    let teams = &en.factions;
    let network = load_network(&root()).expect("network tables");
    let archive = build_archive(
        en,
        teams,
        Some(&network),
        &SceneIndex::new(),
        Some(Lang::En),
    );

    let net = &archive.network;
    assert!(net.imported);
    assert_eq!(net.forces.len(), 41);
    assert_eq!(net.forces.iter().map(|f| f.hexes.len()).sum::<usize>(), 96);
    assert_eq!(net.groups.len(), 21);
    let rhodes = net.forces.iter().find(|f| f.id == "rhodes").unwrap();
    assert_eq!(rhodes.name, "Rhodes Island");
    assert_eq!(rhodes.hexes.len(), 7);
    assert_eq!(rhodes.member_count, 59);
    // Every person on a chart resolves to a name the data gives.
    let unnamed: Vec<_> = net.people.iter().filter(|p| p.name == p.id).collect();
    assert!(unnamed.is_empty(), "{unnamed:?}");
    let kazdel = net.groups.iter().find(|g| g.id == "kazdel").unwrap();
    assert_eq!(kazdel.name, "Babel");

    let amiya = &archive.operators["char_002_amiya"];
    assert_eq!(amiya.number.as_ref().unwrap().prefix, "R");
    assert!(amiya.linked.contains(&"char_003_kalts".to_owned()));
    assert!(!amiya.linked.contains(&"npc_003_kalts".to_owned()));
    println!("amiya linked: {:?}", amiya.linked);
    assert!(
        amiya
            .charts
            .first()
            .is_some_and(|c| c.home && c.group_id == "rhodes")
    );

    let d = &archive.demographics;
    println!(
        "files {} races {} birthplaces {} prefixes {} height {}..{} mean {} over {}",
        d.total,
        d.race.len(),
        d.birthplace.len(),
        d.prefixes.len(),
        d.height.min,
        d.height.max,
        d.height.mean,
        d.height.counted
    );
    println!("gender {:?}", d.gender);
    println!("infection {:?}", d.infection);
    println!("manufactured {:?}", d.manufactured);
    println!(
        "hidden: {} secret links, {} nations",
        archive.hidden.secret.len(),
        archive.hidden.nations.len()
    );
    assert_eq!(d.total as usize, archive.dossiers.dossiers.len());
    // Gender is read off `[Gender]`; a renamed label once left every EN file
    // unread and the chart showed one "not recorded" bar for all of them.
    let keyed = |k| {
        d.gender
            .iter()
            .find(|g| g.key == Some(k))
            .map_or(0, |g| g.count)
    };
    let (female, male) = (keyed(ArchiveGender::Female), keyed(ArchiveGender::Male));
    println!("gender keyed: female {female} male {male} of {}", d.total);
    // 2026-10-11 EN: female 302, male 100, 3 unread words, 7 unstated, of 412.
    assert!(female >= 250 && male >= 80, "female {female} male {male}");
    assert!(
        female + male >= 380,
        "female {female} male {male} of {}",
        d.total
    );
    // Each dossier row carries what the Dossiers filters read.
    let rows = &archive.dossiers.dossiers;
    let gendered = rows.iter().filter(|r| r.gender_key.is_some()).count();
    let nations = rows.iter().filter(|r| r.nation.is_some()).count();
    let affiliated = rows.iter().filter(|r| !r.affiliations.is_empty()).count();
    println!(
        "dossiers {}: gender read {gendered}, nation {nations}, group/team {affiliated}",
        rows.len()
    );
    assert_eq!(gendered, (female + male) as usize);
    assert!(
        nations * 10 >= rows.len() * 8,
        "nation {nations} of {}",
        rows.len()
    );
    assert_eq!(d.birth_months.iter().sum::<u32>(), d.dated);
    // Each exam column accounts for every file.
    for col in &d.exam {
        assert_eq!(col.steps.iter().sum::<u32>() + col.unrecorded, d.total);
    }
    // The R prefix belongs to Rhodes Island in the roster's own records.
    let r = d.prefixes.iter().find(|p| p.prefix == "R").unwrap();
    assert_eq!(r.faction.as_ref().unwrap().id, "rhodes");
    assert!(!archive.hidden.secret.is_empty());
}

#[test]
fn bracketed_values_continue_onto_the_next_line() {
    let pairs = bracketed("[Height] 160cm\n[Infection Status]\nConfirmed Infected.\n\nFree text.");
    assert_eq!(
        pairs[1],
        ("Infection Status".into(), "Confirmed Infected.".into())
    );
}

/// The card faction is read as the `MainPower` id with the highest
/// `PowerLevel`. The rule it replaced (top-level team, else group, else
/// nation) is kept here to price the change on every server.
#[test]
fn card_faction_reads_main_power_by_level() {
    for server in ["en", "cn", "jp", "kr"] {
        let Some(gd) = load(server) else {
            eprintln!("skipping {server}: assets/output/{server} missing");
            continue;
        };
        let teams = &gd.factions;
        let real = |id: Option<&str>| {
            id.filter(|i| !i.is_empty() && *i != "none")
                .map(str::to_owned)
        };
        let mut differ = Vec::new();
        let ids = file_ids(gd);
        for id in &ids {
            let op = &gd.operators[id];
            let old = real(op.team_id.as_deref())
                .or_else(|| real(op.group_id.as_deref()))
                .or_else(|| real(Some(op.nation_id.as_str())));
            let new = card_faction(teams, op);
            if old != new {
                differ.push((id.clone(), old, new));
            }
        }
        println!(
            "{server} card faction: {} of {} files differ from team>group>nation: {differ:?}",
            differ.len(),
            ids.len()
        );
        assert!(differ.is_empty(), "{server} {differ:?}");
    }
}

fn build_server(server: &str) -> Option<Archive> {
    let gd = load(server)?;
    let network = load_network(&server_root(server));
    Some(build_archive(
        gd,
        &gd.factions,
        network.as_ref(),
        &SceneIndex::new(),
        Some(lang(server)),
    ))
}

/// Whether `s` holds a letter of the server's own script: Han for CN, Han or
/// kana for JP, Hangul for KR.
fn in_script(server: &str, s: &str) -> bool {
    let han = |c: char| ('\u{4e00}'..='\u{9fff}').contains(&c);
    let kana = |c: char| ('\u{3040}'..='\u{30ff}').contains(&c);
    let hangul = |c: char| ('\u{ac00}'..='\u{d7a3}').contains(&c);
    s.chars().any(|c| match server {
        "cn" => han(c),
        "jp" => han(c) || kana(c),
        "kr" => hangul(c),
        _ => c.is_alphabetic(),
    })
}

/// Every name the map, the charts, the hidden factions and By the Numbers
/// show on a CN, JP or KR server comes from that server's own tables: no
/// faction, chart or person falls back to its id, and every faction name is
/// the server's own `handbook_team_table.PowerName`. The share written in
/// the server's script is printed and floored; the names outside it are
/// printed so a reader can see they are the game's own Latin spellings.
#[test]
fn every_label_is_in_the_server_language() {
    for server in ["cn", "jp", "kr"] {
        let Some(a) = build_server(server) else {
            eprintln!("skipping {server}: assets/output/{server} missing");
            continue;
        };
        let teams = &load(server).expect("built above").factions;
        let net = &a.network;
        assert!(net.imported, "{server} network tables");
        assert!(!net.forces.is_empty());
        let mut names: Vec<(&str, &str)> = Vec::new();
        for f in &net.forces {
            assert!(
                !f.name.is_empty() && f.name != f.id,
                "{server} force {} unnamed",
                f.id
            );
            if let Some(t) = teams.get(&f.id) {
                assert_eq!(f.name, t.power_name, "{server} force {}", f.id);
            }
            names.push((f.id.as_str(), f.name.as_str()));
        }
        for g in &net.groups {
            assert!(
                !g.name.is_empty() && g.name != g.id,
                "{server} chart {} unnamed",
                g.id
            );
            names.push((g.id.as_str(), g.name.as_str()));
            for b in &g.blocks {
                assert!(
                    !b.name.is_empty() && b.name != b.force_id,
                    "{server} block {}",
                    b.force_id
                );
            }
        }
        let unnamed: Vec<_> = net
            .people
            .iter()
            .filter(|p| p.name.is_empty() || p.name == p.id)
            .map(|p| &p.id)
            .collect();
        assert!(unnamed.is_empty(), "{server} people {unnamed:?}");
        for s in &a.hidden.secret {
            for r in [&s.shown, &s.hidden] {
                assert!(
                    !r.name.is_empty() && r.name != r.id,
                    "{server} faction {}",
                    r.id
                );
                names.push((r.id.as_str(), r.name.as_str()));
            }
        }
        names.sort_unstable();
        names.dedup();
        let outside: Vec<_> = names
            .iter()
            .filter(|(_, n)| !in_script(server, n))
            .collect();
        println!(
            "{server}: {} of {} faction and chart names in the server's script; outside it: {outside:?}",
            names.len() - outside.len(),
            names.len()
        );
        assert!(outside.len() * 10 <= names.len(), "{server} {outside:?}");

        let d = &a.demographics;
        for step in &d.scale {
            let word = step.word.as_deref().unwrap_or_default();
            assert!(
                in_script(server, word),
                "{server} scale {:?} word {word:?}",
                step.key
            );
        }
        for g in d.gender.iter().filter(|g| g.key.is_some()) {
            assert!(in_script(server, &g.value), "{server} gender {:?}", g.value);
        }
        println!(
            "{server} scale {:?} gender {:?}",
            d.scale
                .iter()
                .map(|s| (s.key, s.word.clone()))
                .collect::<Vec<_>>(),
            d.gender
                .iter()
                .map(|g| (g.key, g.value.clone(), g.count))
                .collect::<Vec<_>>()
        );
    }
}

fn hidden_ids(data: &GameModel, id: &str) -> Vec<String> {
    data.operators[id]
        .hidden_factions
        .iter()
        .map(|f| f.id.clone())
        .collect()
}

/// Hidden factions (`SubPower` ids outside `MainPower`) on the real EN tree,
/// the input of the hidden-factions view.
#[test]
fn hidden_factions_are_sub_powers_the_card_does_not_show() {
    let Some(en) = load("en") else {
        eprintln!("skipping: assets/output/en missing");
        return;
    };

    let (chars, others): (Vec<&String>, Vec<&String>) = en
        .operators
        .iter()
        .filter(|(_, op)| !op.hidden_factions.is_empty())
        .map(|(id, _)| id)
        .partition(|id| id.starts_with("char_"));
    eprintln!(
        "en operators with a hidden faction: {} char_, others {others:?}",
        chars.len()
    );
    // 80 on the EN tree of 2026-10-11: 78 `character_table` operators plus
    // Amiya's two `char_patch_table` forms, which inherit her `rim`. The
    // three Laterano cosplayer traps carry a `SubPower` too and are not
    // operators. New operators only add to it.
    assert!(chars.len() >= 80, "{}", chars.len());

    assert_eq!(hidden_ids(en, "char_113_cqbw"), ["rhodes"]);
    assert_eq!(hidden_ids(en, "char_1028_texas2"), ["siracusa"]);
    // Reed's entry is `tara` + `victoria`; Victoria is already her nation.
    assert_eq!(hidden_ids(en, "char_261_sddrag"), ["tara"]);
    // Mostima's entry names a group with its nation, both new to her.
    assert_eq!(hidden_ids(en, "char_213_mostma"), ["penguin", "lungmen"]);
    assert_eq!(hidden_ids(en, "char_017_huang"), ["yan", "victoria"]);
    // No `SubPower`: nothing hidden, whatever the card shows.
    assert!(hidden_ids(en, "char_1012_skadi2").is_empty());
    assert!(hidden_ids(en, "char_003_kalts").is_empty());
    assert!(hidden_ids(en, "char_456_ash").is_empty());

    let w = &en.operators["char_113_cqbw"].hidden_factions[0];
    assert_eq!(w.name.as_deref(), Some("Rhodes Island"));
}
