use crate::assignment::CandidateTeam;
use crate::rotation::shift::SHIFT_COUNT;
use crate::rotation::team_select::{Tiling, packing_shortlist, teams_for_rooms, tile_group};

fn team(ops: &[&str], score: f64) -> CandidateTeam {
    CandidateTeam {
        ops: ops.iter().map(|o| (*o).to_string()).collect(),
        speed: score,
        value: score,
        gold: 0.0,
        order_limit: None,
        score,
    }
}

#[test]
fn the_shortlist_keeps_a_disjoint_team_past_the_score_head() {
    let teams = vec![
        team(&["a", "b"], 10.0),
        team(&["a", "c"], 9.0),
        team(&["b", "c"], 8.0),
        team(&["d", "e"], 7.0),
        team(&["a", "d"], 6.0),
    ];
    let kept: Vec<Vec<String>> = packing_shortlist(teams, 2, 1)
        .into_iter()
        .map(|t| t.ops)
        .collect();
    assert_eq!(kept, vec![vec!["a", "b"], vec!["a", "c"], vec!["d", "e"]]);
}

#[test]
fn the_shortlist_is_the_whole_list_when_it_fits_the_head() {
    let teams = vec![team(&["a"], 2.0), team(&["b"], 1.0)];
    assert_eq!(packing_shortlist(teams, 5, 3).len(), 2);
}

#[test]
fn tiling_invariants_hold_for_small_groups() {
    for n_rooms in 1..=4 {
        let cells = tile_group(n_rooms, Tiling::Blocks);
        let teams = teams_for_rooms(n_rooms, Tiling::Blocks);
        let mut count = vec![0usize; teams];
        for room in &cells {
            for &t in room {
                assert!(t < teams, "cell team index in range (n={n_rooms})");
                count[t] += 1;
            }
        }
        for (t, &c) in count.iter().enumerate() {
            let expect = if (n_rooms * SHIFT_COUNT) % 2 == 1 && t == teams - 1 {
                1
            } else {
                2
            };
            assert_eq!(c, expect, "team {t} cell count (n={n_rooms})");
        }
        for t in 0..teams {
            let positions: Vec<usize> = (0..n_rooms * SHIFT_COUNT)
                .filter(|p| cells[p / SHIFT_COUNT][p % SHIFT_COUNT] == t)
                .collect();
            if positions.len() == 2 {
                assert_eq!(positions[1] - positions[0], 1, "team {t} block consecutive");
            }
        }
    }
}

#[test]
fn two_room_tiling_matches_the_login_rhythm() {
    let cells = tile_group(2, Tiling::Blocks);
    assert_eq!(cells, vec![[0, 0, 1], [1, 2, 2]]);
    for s in 1..SHIFT_COUNT {
        let changes = cells.iter().filter(|room| room[s] != room[s - 1]).count();
        assert_eq!(changes, 1, "one room swaps its team at boundary {s}");
    }
}
