use std::collections::HashMap;

use super::*;

fn entry(id: &str, group: &str, sort: i32, scripted: bool) -> StoryEntry {
    StoryEntry {
        id: id.into(),
        name: id.into(),
        code: None,
        sort,
        avg_tag: None,
        group_id: group.into(),
        has_script: scripted,
        word_count: 0,
        required_stages: vec![],
    }
}

fn group(id: &str, start: i64, stories: Vec<StoryEntry>) -> StoryGroup {
    StoryGroup {
        id: id.into(),
        name: id.into(),
        category: serde_json::Value::Null,
        entry_type: String::new(),
        act_type: String::new(),
        start_time: start,
        chapter_number: None,
        stories,
    }
}

fn index(groups: Vec<StoryGroup>, records: Vec<OperatorRecordGroup>) -> StoryIndex {
    StoryIndex {
        groups,
        records,
        totals: StoryTotals {
            stories: 0,
            with_script: 0,
            words: 0,
        },
    }
}

#[test]
fn plan_is_stable_under_shuffled_input() {
    let a = index(
        vec![
            group("g2", 20, vec![entry("b", "g2", 2, true), entry("a", "g2", 1, true)]),
            group("g1", 10, vec![entry("c", "g1", 1, true)]),
        ],
        vec![],
    );
    let b = index(
        vec![
            group("g1", 10, vec![entry("c", "g1", 1, true)]),
            group("g2", 20, vec![entry("a", "g2", 1, true), entry("b", "g2", 2, true)]),
        ],
        vec![],
    );
    let ids = |i: &StoryIndex| plan(i).0.into_iter().map(|j| j.entry.id).collect::<Vec<_>>();
    assert_eq!(ids(&a), vec!["c", "a", "b"]);
    assert_eq!(ids(&a), ids(&b));
}

#[test]
fn records_dedupe_against_groups_and_unscripted_are_set_aside() {
    let i = index(
        vec![group(
            "record",
            0,
            vec![entry("r1", "record", 1, true), entry("x", "record", 2, false)],
        )],
        vec![OperatorRecordGroup {
            char_id: "char_002_amiya".into(),
            name: "Amiya".into(),
            stories: vec![entry("r1", "record", 1, true), entry("r2", "set", 1, true)],
        }],
    );
    let (jobs, unscripted) = plan(&i);
    let ids: Vec<_> = jobs.iter().map(|j| j.entry.id.as_str()).collect();
    assert_eq!(ids, vec!["r1", "r2"]);
    assert_eq!(jobs[0].source, Source::Group, "first listing wins");
    assert!(matches!(jobs[1].source, Source::Record { .. }));
    assert_eq!(unscripted.len(), 1);
}

fn tmpfile(name: &str, contents: &[u8]) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("trevor-ingest-{}-{name}", std::process::id()));
    std::fs::write(&p, contents).expect("write");
    p
}

#[test]
fn recover_cuts_a_torn_line_and_drops_the_last_story() {
    let body = concat!(
        "{\"storyId\":\"s1\",\"x\":1}\n",
        "{\"storyId\":\"s1\",\"x\":2}\n",
        "{\"storyId\":\"s2\",\"x\":1}\n",
        "{\"storyId\":\"s2\",\"x\":2}\n",
        "{\"storyId\":\"s3\",\"x\":1}\n",
        "{\"storyId\":\"s3\",\"x\""
    );
    let p = tmpfile("torn", body.as_bytes());
    let (done, rec) = recover(&p).expect("recover");
    assert_eq!(rec.torn_bytes_discarded, "{\"storyId\":\"s3\",\"x\"".len() as u64);
    assert_eq!(rec.dropped_last_story.as_deref(), Some("s3"));
    assert_eq!(rec.lines_dropped, 1);
    let mut d: Vec<_> = done.into_iter().collect();
    d.sort();
    assert_eq!(d, vec!["s1", "s2"]);
    let after = std::fs::read_to_string(&p).expect("read");
    assert!(after.ends_with("{\"storyId\":\"s2\",\"x\":2}\n"));
    assert!(!after.contains("s3"));
}

#[test]
fn recover_on_a_missing_file_is_empty() {
    let p = std::env::temp_dir().join("trevor-ingest-definitely-absent.jsonl");
    let _ = std::fs::remove_file(&p);
    let (done, rec) = recover(&p).expect("recover");
    assert!(done.is_empty());
    assert!(rec.dropped_last_story.is_none());
}

fn lines(story: &str, n: usize, tag: &str) -> Vec<u8> {
    (0..n)
        .map(|i| format!("{{\"storyId\":\"{story}\",\"n\":{i},\"t\":\"{tag}\"}}\n"))
        .collect::<String>()
        .into_bytes()
}

#[test]
fn refresh_classifies_every_story_and_keeps_plan_order() {
    let old_bytes = [
        lines("a", 2, "x"),
        lines("b", 1, "x"),
        lines("gone", 3, "x"),
        lines("c", 2, "x"),
        lines("d", 1, "x"),
    ]
    .concat();
    let old = OldCorpus::parse(&old_bytes).expect("parse");
    assert_eq!((old.lines, old.torn_bytes), (9, 0));

    let mut r = Refresh::new(&old);
    let mut out = Vec::new();
    // Plan order puts the new story first; the file follows the plan.
    let (new, a, b, empty) = (lines("new", 2, "x"), lines("a", 2, "x"), lines("b", 2, "y"), Vec::new());
    out.extend_from_slice(r.story("new", Some(&new)));
    out.extend_from_slice(r.story("a", Some(&a)));
    out.extend_from_slice(r.story("b", Some(&b)));
    out.extend_from_slice(r.story("c", None));
    out.extend_from_slice(r.story("video", Some(&empty)));
    out.extend_from_slice(r.story("d", Some(&empty)));
    let rep = r.finish();

    assert_eq!(rep.added, ["new"]);
    assert_eq!(rep.changed, ["b", "d"], "d now chunks to nothing");
    assert_eq!(rep.unchanged, ["a", "video"]);
    assert_eq!(rep.failed, ["c"]);
    assert_eq!(rep.removed, ["gone"]);
    assert_eq!(
        rep.counts,
        ChangeCounts {
            added: 1,
            changed: 2,
            removed: 1,
            unchanged: 2,
            failed: 1,
            old_chunks: 9,
            new_chunks: 8,
            chunks_added_or_changed: 4,
        }
    );
    // A failed fetch keeps the story's old lines.
    let want = [new, a, b, lines("c", 2, "x")].concat();
    assert_eq!(out, want);
}

#[test]
fn refresh_of_unchanged_data_reproduces_the_file_and_ignores_a_torn_tail() {
    let body = [lines("a", 2, "x"), lines("b", 3, "x")].concat();
    let mut torn = body.clone();
    torn.extend_from_slice(b"{\"storyId\":\"b\",\"n\"");
    let old = OldCorpus::parse(&torn).expect("parse");
    assert_eq!(old.torn_bytes, 18);
    let mut r = Refresh::new(&old);
    let mut out = Vec::new();
    out.extend_from_slice(r.story("a", Some(&lines("a", 2, "x"))));
    out.extend_from_slice(r.story("b", Some(&lines("b", 3, "x"))));
    let rep = r.finish();
    assert_eq!(out, body);
    assert_eq!(rep.unchanged, ["a", "b"]);
    assert_eq!(rep.counts.chunks_added_or_changed, 0);
}

// ---- end to end against a fake backend -------------------------------

type Routes = Arc<std::sync::Mutex<HashMap<String, String>>>;

/// A one-thread HTTP/1.1 server answering GETs from `routes`, 404
/// otherwise. Enough for `Fetcher`; it never sees a real backend.
fn fake_backend(routes: Routes) -> String {
    use std::io::{BufRead as _, BufReader};
    let l = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = l.local_addr().expect("addr");
    std::thread::spawn(move || {
        for conn in l.incoming() {
            let Ok(mut conn) = conn else { continue };
            let mut reader = BufReader::new(conn.try_clone().expect("clone"));
            let mut first = String::new();
            if reader.read_line(&mut first).is_err() {
                continue;
            }
            let mut h = String::new();
            while reader.read_line(&mut h).is_ok_and(|n| n > 2) {
                h.clear();
            }
            let path = first.split_whitespace().nth(1).unwrap_or("").to_owned();
            let body = routes.lock().expect("lock").get(&path).cloned();
            let (status, body) = body.map_or(("404 Not Found", String::new()), |b| ("200 OK", b));
            let _ = write!(
                conn,
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\
                 connection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    format!("http://{addr}")
}

/// Serve an index of one group with `stories` (id, lines of dialogue).
fn serve(routes: &Routes, stories: &[(&str, &[&str])]) {
    let mut m = routes.lock().expect("lock");
    m.clear();
    let entries: Vec<serde_json::Value> = stories
        .iter()
        .enumerate()
        .map(|(i, (id, _))| {
            serde_json::json!({"id": id, "name": id, "sort": i, "groupId": "g",
                "hasScript": true, "wordCount": 0})
        })
        .collect();
    let index = serde_json::json!({
        "groups": [{"id": "g", "name": "g", "startTime": 1, "stories": entries}],
        "totals": {"stories": stories.len(), "withScript": stories.len(), "words": 0},
    });
    m.insert("/api/story/index".into(), index.to_string());
    for (id, said) in stories {
        let commands: Vec<serde_json::Value> = said
            .iter()
            .enumerate()
            .map(|(i, t)| {
                serde_json::json!({"kind": "name", "args": {"name": "Amiya"},
                "text": t, "line": i + 1})
            })
            .collect();
        let words: usize = said.iter().map(|t| t.split_whitespace().count()).sum();
        let script = serde_json::json!({"id": id, "commands": commands, "wordCount": words});
        m.insert(format!("/api/story/{id}"), script.to_string());
    }
}

fn test_cfg(base: &str, out: &Path, refresh: bool) -> IngestConfig {
    IngestConfig {
        base: base.into(),
        out: out.to_path_buf(),
        concurrency: 2,
        limit: None,
        service_key: None,
        chunk: ChunkConfig::default(),
        tokenizer_sha: "t".into(),
        refresh,
        max_drop_pct: Some(20),
    }
}

fn words() -> Arc<dyn Fn(&str) -> u32 + Send + Sync> {
    Arc::new(|s: &str| u32::try_from(s.split_whitespace().count()).unwrap_or(0))
}

fn scratch(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("trevor-refresh-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    p
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn refresh_matches_a_fresh_build_and_leaves_resume_alone() {
    let routes: Routes = Arc::default();
    let base = fake_backend(Arc::clone(&routes));
    let v1: &[(&str, &[&str])] = &[
        ("s1", &["Doctor, are you awake?", "We land at dawn."]),
        ("s2", &["Kal'tsit is waiting."]),
        ("s3", &["This story will be cut."]),
    ];
    serve(&routes, v1);
    let dir = scratch("main");
    let m = run(&test_cfg(&base, &dir, false), words()).await.expect("build");
    assert!(m.clean() && m.refresh.is_none());
    let built = std::fs::read(dir.join("chunks.jsonl")).expect("read");

    // Unchanged data: the file is untouched and every story unchanged.
    let m = run(&test_cfg(&base, &dir, true), words()).await.expect("refresh");
    let c = m.refresh.expect("counts");
    assert_eq!((c.unchanged, c.added, c.changed, c.removed), (3, 0, 0, 0));
    assert_eq!(std::fs::read(dir.join("chunks.jsonl")).expect("read"), built);
    assert!(!dir.join("chunks.prev.jsonl").exists());

    // An asset update: s2 edited, s3 gone, s4 new. Resume misses the
    // edit (the reason refresh exists) and keeps the file as it was.
    let v2: &[(&str, &[&str])] = &[
        ("s1", &["Doctor, are you awake?", "We land at dawn."]),
        ("s2", &["Kal'tsit is still waiting."]),
        ("s4", &["A new story."]),
    ];
    serve(&routes, v2);
    let resumed = scratch("resume");
    std::fs::create_dir_all(&resumed).expect("mkdir");
    std::fs::write(resumed.join("chunks.jsonl"), &built).expect("w");
    run(&test_cfg(&base, &resumed, false), words()).await.expect("resume");
    let r = std::fs::read_to_string(resumed.join("chunks.jsonl")).expect("read");
    assert!(r.contains("Kal'tsit is waiting.") && !r.contains("still waiting"));

    let m = run(&test_cfg(&base, &dir, true), words()).await.expect("refresh");
    let c = m.refresh.expect("counts");
    assert_eq!((c.unchanged, c.added, c.changed, c.removed), (1, 1, 1, 1));
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("changes.json")).expect("changes")).expect("json");
    assert_eq!(report["changed"], serde_json::json!(["s2"]));
    assert_eq!(report["added"], serde_json::json!(["s4"]));
    assert_eq!(report["removed"], serde_json::json!(["s3"]));
    assert_eq!(std::fs::read(dir.join("chunks.prev.jsonl")).expect("prev"), built);

    // The refreshed file is byte for byte a from-scratch build of v2.
    let fresh = scratch("fresh");
    run(&test_cfg(&base, &fresh, false), words()).await.expect("fresh");
    let refreshed = std::fs::read(dir.join("chunks.jsonl")).expect("read");
    assert_eq!(refreshed, std::fs::read(fresh.join("chunks.jsonl")).expect("read"));
    assert_eq!(report["newChunksSha"], serde_json::json!(sha_hex(&refreshed)));

    // Refresh with --limit would drop the rest of the corpus: refused.
    let mut limited = test_cfg(&base, &dir, true);
    limited.limit = Some(1);
    assert!(run(&limited, words()).await.is_err());
    for d in [dir, resumed, fresh] {
        let _ = std::fs::remove_dir_all(d);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_shrunken_or_empty_index_writes_nothing_unless_allowed() {
    let routes: Routes = Arc::default();
    let base = fake_backend(Arc::clone(&routes));
    let v1: &[(&str, &[&str])] = &[
        ("s1", &["One."]),
        ("s2", &["Two."]),
        ("s3", &["Three."]),
        ("s4", &["Four."]),
        ("s5", &["Five."]),
    ];
    serve(&routes, v1);
    let dir = scratch("shrink");
    run(&test_cfg(&base, &dir, false), words()).await.expect("build");
    let files = |d: &Path| {
        ["spoiler.jsonl", "chunks.jsonl", "unresolved.jsonl", "manifest.p0.json"]
            .map(|f| std::fs::read(d.join(f)).expect("read"))
    };
    let before = files(&dir);

    // Two of five stories gone (40%, over the 20% default): refused in both
    // modes, every file left byte for byte as it was.
    serve(&routes, &v1[..3]);
    for refresh in [true, false] {
        let err = run(&test_cfg(&base, &dir, refresh), words()).await.expect_err("shrink");
        assert!(err.to_string().contains("2 fewer than the previous plan's 5"), "{err}");
        assert_eq!(files(&dir), before);
    }
    // An empty index is refused even with no previous plan to compare.
    serve(&routes, &[]);
    assert!(run(&test_cfg(&base, &dir, true), words()).await.is_err());
    let empty = scratch("shrink-empty");
    assert!(run(&test_cfg(&base, &empty, false), words()).await.is_err());
    assert!(!empty.join("spoiler.jsonl").exists());
    assert_eq!(files(&dir), before);

    // One of five (exactly 20%) is within the limit.
    serve(&routes, &v1[..4]);
    let m = run(&test_cfg(&base, &dir, true), words()).await.expect("within limit");
    assert_eq!(m.refresh.expect("counts").removed, 1);

    // The kill switch: the old behaviour, the shrunken plan replaces the old.
    serve(&routes, &v1[..2]);
    let mut allow = test_cfg(&base, &dir, true);
    allow.max_drop_pct = None;
    let m = run(&allow, words()).await.expect("allowed");
    assert_eq!(m.refresh.expect("counts").removed, 2);
    let spoiler = std::fs::read_to_string(dir.join("spoiler.jsonl")).expect("read");
    assert_eq!(spoiler.lines().count(), 2);
    for d in [dir, empty] {
        let _ = std::fs::remove_dir_all(d);
    }
}

fn cmd(kind: &str, args: &[(&str, &str)], text: Option<&str>, line: u32) -> StoryCommand {
    StoryCommand {
        kind: kind.into(),
        args: args.iter().map(|(k, v)| ((*k).into(), (*v).into())).collect(),
        text: text.map(Into::into),
        line,
    }
}

#[test]
fn reconstruction_catches_loss_and_accepts_multiword_speakers() {
    let cmds = vec![
        cmd(
            "name",
            &[("name", "Rhodes Island Operator")],
            Some("Doctor: stand by."),
            1,
        ),
        cmd("name", &[("name", "Amiya")], Some("Understood."), 2),
    ];
    let words = |s: &str| u32::try_from(s.split_whitespace().count()).unwrap_or(0);
    let chunks = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
    assert!(reconstructs(&cmds, &chunks));

    let mut lossy = chunks.clone();
    lossy[0].text = lossy[0].text.replace("stand by", "stand");
    assert!(!reconstructs(&cmds, &lossy));
}
