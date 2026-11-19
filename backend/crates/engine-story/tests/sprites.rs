#![allow(clippy::unwrap_used, clippy::expect_used)]

use myrtle_engine_story::dto::StorySpriteName;
use myrtle_engine_story::sprites::{keep_aliases, key_of};

#[test]
fn a_script_name_reads_as_the_key_the_resolver_reads() {
    assert_eq!(key_of("avg_npc_043_1"), "#1$1");
    assert_eq!(key_of("avg_npc_043_1#2"), "#2$1");
    assert_eq!(key_of("avg_225_haak_1#03$2"), "#3$2");
    assert_eq!(key_of("avg_1_a_1@smile"), "@smile");
}

#[test]
fn an_alias_needs_three_lines_and_two_percent() {
    let n = |name: &str, count: f64| StorySpriteName {
        name: name.to_owned(),
        count,
    };
    let names = |v: &[StorySpriteName]| v.iter().map(|x| x.name.clone()).collect::<Vec<_>>();
    let (kept, stray) = keep_aliases(
        vec![n("Kal'tsit", 3102.0), n("Wei Yenwu", 3.0), n("Mon3tr", 0.5)],
        3108.0,
    );
    assert_eq!(names(&kept), ["Kal'tsit"]);
    assert_eq!(names(&stray), ["Wei Yenwu", "Mon3tr"]);
    let (kept, _) = keep_aliases(
        vec![
            n("Jie", 252.0),
            n("Minister of Works", 14.0),
            n("Chun", 3.0),
        ],
        269.0,
    );
    assert_eq!(names(&kept), ["Jie", "Minister of Works"]);
    let (kept, stray) = keep_aliases(vec![n("Guard", 2.0), n("Soldier", 2.0)], 4.0);
    assert_eq!(names(&kept), ["Guard"]);
    assert_eq!(names(&stray), ["Soldier"]);
}
