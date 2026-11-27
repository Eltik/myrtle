use myrtle_gamedata::model::enemy_stages::{EnemyStageIndex, EnemyStageRef, EnemyStageTable};

fn stage(stage_id: &str, zone_name: Option<&str>, count: u32) -> EnemyStageRef {
    EnemyStageRef {
        stage_id: stage_id.to_owned(),
        code: stage_id.to_uppercase(),
        zone_id: "main_1".to_owned(),
        zone_name: zone_name.map(str::to_owned),
        category: "stages".to_owned(),
        group: "story".to_owned(),
        stage_name: None,
        is_hard: false,
        count,
        battle: None,
    }
}

#[test]
fn table_writes_each_stage_once_and_rebuilds_the_index() {
    let index: EnemyStageIndex = [
        (
            "enemy_b".to_owned(),
            vec![stage("s1", Some("Zone"), 3), stage("s2", None, 0)],
        ),
        ("enemy_a".to_owned(), vec![stage("s1", Some("Zone"), 7)]),
        // Same stage id, different zone name: kept as its own stage.
        ("enemy_c".to_owned(), vec![stage("s1", None, 1)]),
    ]
    .into_iter()
    .collect();

    let table = EnemyStageTable::from_index(&index);
    assert_eq!(table.stages.len(), 3);
    assert_eq!(table.refs["enemy_a"], [(0, 7)]);
    assert_eq!(table.refs["enemy_b"], [(0, 3), (1, 0)]);
    assert_eq!(table.refs["enemy_c"], [(2, 1)]);

    for (enemy, refs) in &index {
        let rebuilt: Vec<_> = table.refs[enemy]
            .iter()
            .map(|&(s, count)| {
                let info = &table.stages[s as usize];
                EnemyStageRef {
                    stage_id: info.stage_id.clone(),
                    code: info.code.clone(),
                    zone_id: info.zone_id.clone(),
                    zone_name: info.zone_name.clone(),
                    category: info.category.clone(),
                    group: info.group.clone(),
                    stage_name: info.stage_name.clone(),
                    is_hard: info.is_hard,
                    count,
                    battle: None,
                }
            })
            .collect();
        assert_eq!(
            serde_json::to_value(&rebuilt).unwrap(),
            serde_json::to_value(refs).unwrap()
        );
    }
}
