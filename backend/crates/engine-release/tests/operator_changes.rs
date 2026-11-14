#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::collections::HashMap;

use myrtle_engine_release::operator_changes::*;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::module::{
    AddModuleCandidates, AddOrOverrideTalentDataBundle, Module, ModuleCandidates, ModuleData,
    ModulePart, ModulePhase, OverrideTraitDataBundle,
};
use myrtle_gamedata_raw::tables::module::{ModuleBlackboard, ModuleType};
use myrtle_gamedata_raw::tables::operator::AttributeData;
use myrtle_gamedata_raw::tables::operator::{
    AttributeKeyFrame, Blackboard, Operator, OperatorModule,
};
use myrtle_gamedata_raw::tables::operator::{
    EnrichedSkill, Phase, SkillStatic, Talent, TalentCandidate,
};
use myrtle_gamedata_raw::tables::skill::SkillSpData;
use myrtle_gamedata_raw::tables::skill::{SkillBlackboard, SkillLevel};
use serde_json::json;

fn module(id: &str, kind: ModuleType, atk: f64) -> OperatorModule {
    OperatorModule {
        module: Module {
            uni_equip_id: id.to_string(),
            module_type: kind,
            ..Module::default()
        },
        data: ModuleData {
            phases: vec![ModulePhase {
                equip_level: 1,
                attribute_blackboard: vec![ModuleBlackboard {
                    key: "atk".into(),
                    value: atk,
                }],
                ..ModulePhase::default()
            }],
        },
    }
}

fn skill(id: &str, values: &[(&str, f64)], name: &str) -> EnrichedSkill {
    EnrichedSkill {
        skill_id: id.to_string(),
        static_data: Some(SkillStatic {
            skill_id: id.to_string(),
            levels: vec![SkillLevel {
                name: name.to_string(),
                description: format!("{name} description"),
                sp_data: SkillSpData {
                    sp_cost: 30,
                    ..SkillSpData::default()
                },
                blackboard: values
                    .iter()
                    .map(|(k, v)| SkillBlackboard {
                        key: (*k).to_string(),
                        value: *v,
                        value_str: None,
                    })
                    .collect(),
                ..SkillLevel::default()
            }],
            ..SkillStatic::default()
        }),
        ..EnrichedSkill::default()
    }
}

fn base() -> Operator {
    Operator {
        name: "Shu".into(),
        modules: vec![
            module("uniequip_001_shu", ModuleType::Initial, 0.0),
            module("uniequip_002_shu", ModuleType::Advanced, 60.0),
        ],
        skills: vec![skill("skchr_shu_1", &[("atk", 0.3), ("heal", 0.2)], "Rain")],
        phases: vec![Phase {
            max_level: 50,
            ..Phase::default()
        }],
        talents: vec![Talent {
            candidates: vec![TalentCandidate {
                name: Some("Harvest".into()),
                blackboard: vec![Blackboard {
                    key: "hp_ratio".into(),
                    value: 0.1,
                    value_str: None,
                }],
                ..TalentCandidate::default()
            }],
        }],
        ..Operator::default()
    }
}

#[test]
fn a_kind_on_most_shared_operators_is_reported_as_data_wide() {
    let on = |id: &str, changes: Vec<OperatorChange>| OperatorCnChanges {
        id: id.into(),
        changes,
    };
    let changes = vec![
        on(
            "a",
            vec![OperatorChange::StatsChanged, OperatorChange::TraitChanged],
        ),
        on("b", vec![OperatorChange::StatsChanged]),
        on(
            "c",
            vec![OperatorChange::StatsChanged, OperatorChange::RangeChanged],
        ),
        on("d", vec![OperatorChange::TraitChanged]),
    ];
    assert_eq!(data_wide_kinds(&changes, 4), vec![("stats_changed", 3)]);
    assert!(data_wide_kinds(&changes, 6).is_empty());
    assert_eq!(
        data_wide_kinds(&changes, 3),
        vec![("stats_changed", 3), ("trait_changed", 2)]
    );
    assert!(data_wide_kinds(&changes, 0).is_empty());
}

#[test]
fn identical_operators_and_translated_text_report_nothing() {
    let en = base();
    let mut cn = base();
    cn.name = "黍".into();
    cn.description = "不同语言".into();
    cn.skills[0].static_data.as_mut().unwrap().levels[0].name = "雨".into();
    cn.talents[0].candidates[0].name = Some("丰收".into());
    assert_eq!(operator_changes(&en, &cn), vec![]);
}

#[test]
fn a_reordered_blackboard_or_widened_float_is_not_a_change() {
    let en = base();
    let mut cn = base();
    cn.skills[0] = skill(
        "skchr_shu_1",
        &[("heal", 0.200_000_002_980_232_24), ("atk", 0.3)],
        "雨",
    );
    assert_eq!(operator_changes(&en, &cn), vec![]);
}

#[test]
fn a_new_advanced_module_is_reported_and_the_default_one_is_not() {
    let mut en = base();
    en.modules.clear();
    let mut cn = base();
    cn.modules
        .push(module("uniequip_003_shu", ModuleType::Advanced, 80.0));
    assert_eq!(
        operator_changes(&en, &cn),
        vec![
            OperatorChange::ModuleAdded {
                module_id: "uniequip_002_shu".into(),
                translations: vec![],
            },
            OperatorChange::ModuleAdded {
                module_id: "uniequip_003_shu".into(),
                translations: vec![],
            },
        ]
    );
}

#[test]
fn numeric_changes_are_reported_per_category() {
    let en = base();
    let mut cn = base();
    cn.modules[1] = module("uniequip_002_shu", ModuleType::Advanced, 65.0);
    cn.skills[0] = skill("skchr_shu_1", &[("atk", 0.35), ("heal", 0.2)], "雨");
    cn.skills.push(skill("skchr_shu_4", &[], "新"));
    cn.talents[0].candidates[0].blackboard[0].value = 0.12;
    cn.phases[0].max_level = 55;
    assert_eq!(
        operator_changes(&en, &cn),
        vec![
            OperatorChange::ModuleChanged {
                module_id: "uniequip_002_shu".into()
            },
            OperatorChange::SkillValuesChanged {
                skill_id: "skchr_shu_1".into()
            },
            OperatorChange::SkillAdded {
                skill_id: "skchr_shu_4".into()
            },
            OperatorChange::TalentsChanged,
            OperatorChange::StatsChanged,
        ]
    );
}

#[test]
fn an_immunity_flag_is_not_a_stat_or_trust_change() {
    let frame = AttributeKeyFrame {
        level: 1,
        data: AttributeData {
            atk: 500,
            ..AttributeData::default()
        },
    };
    let mut en = base();
    en.phases[0].attributes_key_frames = vec![frame.clone()];
    en.favor_key_frames = vec![frame];
    let mut cn = en.clone();
    cn.phases[0].attributes_key_frames[0].data.stun_immune = true;
    cn.favor_key_frames[0].data.stun_immune = true;
    assert_eq!(operator_changes(&en, &cn), vec![]);

    cn.phases[0].attributes_key_frames[0].data.atk = 520;
    assert_eq!(
        operator_changes(&en, &cn),
        vec![OperatorChange::StatsChanged]
    );
}

#[test]
fn every_kind_names_its_wire_tag() {
    for change in [
        OperatorChange::StatsChanged,
        OperatorChange::TrustBonusChanged,
        OperatorChange::SkillAdded {
            skill_id: "s".into(),
        },
    ] {
        let wire = serde_json::to_value(&change).unwrap();
        assert_eq!(wire["kind"], change.kind());
    }
}

#[test]
fn the_wire_shape_is_kind_tagged_with_camel_case_fields() {
    let value = serde_json::to_value(OperatorChange::ModuleAdded {
        module_id: "uniequip_003_shu".into(),
        translations: vec![CnTranslation {
            cn: "受到的伤害减少15%".into(),
            en: "Reduces damage taken by 15%".into(),
        }],
    })
    .unwrap();
    assert_eq!(
        value,
        json!({
            "kind": "module_added",
            "moduleId": "uniequip_003_shu",
            "translations": [{ "cn": "受到的伤害减少15%", "en": "Reduces damage taken by 15%" }],
        })
    );
    assert_eq!(
        serde_json::to_value(OperatorChange::StatsChanged).unwrap(),
        json!({ "kind": "stats_changed" })
    );
}

fn texted(id: &str, trait_add: &str, talent: (&str, &str)) -> OperatorModule {
    let mut m = module(id, ModuleType::Advanced, 60.0);
    m.data.phases[0].parts = vec![
        ModulePart {
            override_trait_data_bundle: OverrideTraitDataBundle {
                candidates: Some(vec![ModuleCandidates {
                    additional_description: trait_add.into(),
                    ..ModuleCandidates::default()
                }]),
            },
            ..ModulePart::default()
        },
        ModulePart {
            add_or_override_talent_data_bundle: AddOrOverrideTalentDataBundle {
                candidates: Some(vec![AddModuleCandidates {
                    name: talent.0.into(),
                    upgrade_description: talent.1.into(),
                    ..AddModuleCandidates::default()
                }]),
            },
            ..ModulePart::default()
        },
    ];
    m
}

fn pair(cn: &str, en: &str) -> CnTranslation {
    CnTranslation {
        cn: cn.into(),
        en: en.into(),
    }
}

#[test]
fn shared_texts_keep_exact_pairs_and_drop_ambiguous_ones() {
    let shared = SharedTexts::from_pairs([
        ("受到的伤害减少15%", "Reduces damage taken by 15%"),
        ("受到的伤害减少15%", "Reduces damage taken by 15%"),
        ("攻击力+{atk}", "ATK +{atk}"),
        ("攻击力+{atk}", "Attack +{atk}"),
        ("", "Empty"),
        ("+15%", "+15%"),
    ]);
    assert_eq!(shared.ambiguous, 1);
    assert_eq!(
        shared.texts,
        HashMap::from([(
            "受到的伤害减少15%".to_string(),
            "Reduces damage taken by 15%".to_string()
        )])
    );
}

#[test]
fn shared_texts_pair_the_same_part_position_only() {
    let mut en_op = base();
    en_op.modules = vec![texted(
        "uniequip_002_yato",
        "Reduces damage taken by 15%",
        ("Edge", "ATK +{atk}"),
    )];
    let mut cn_op = base();
    cn_op.modules = vec![texted(
        "uniequip_002_yato",
        "受到的伤害减少15%",
        ("锋刃", "攻击力+{atk}"),
    )];
    cn_op
        .modules
        .push(texted("uniequip_003_yato", "仅CN", ("新", "新的")));
    let mut pairs = Vec::new();
    module_text_pairs(&en_op, &cn_op, &mut pairs);
    assert_eq!(
        pairs,
        vec![
            ("受到的伤害减少15%", "Reduces damage taken by 15%"),
            ("锋刃", "Edge"),
            ("攻击力+{atk}", "ATK +{atk}"),
        ]
    );
}

#[test]
fn an_added_module_carries_the_translations_its_text_has() {
    let mut en = GameModel::default();
    let mut cn = GameModel::default();
    let mut yato_en = base();
    yato_en.modules = vec![texted(
        "uniequip_002_yato",
        "Reduces damage taken by 15%",
        ("Edge", "ATK +{atk}"),
    )];
    let mut yato_cn = base();
    yato_cn.modules = vec![texted(
        "uniequip_002_yato",
        "受到的伤害减少15%",
        ("锋刃", "攻击力+{atk}"),
    )];
    let mut shu_en = base();
    shu_en.modules.truncate(1);
    let mut shu_cn = base();
    shu_cn.modules = vec![
        shu_cn.modules[0].clone(),
        texted(
            "uniequip_002_shu",
            "受到的伤害减少15%",
            ("丰", "攻击力+{atk}"),
        ),
    ];
    en.operators.insert("char_yato".into(), yato_en);
    en.operators.insert("char_shu".into(), shu_en);
    cn.operators.insert("char_yato".into(), yato_cn);
    cn.operators.insert("char_shu".into(), shu_cn);
    let changes = cn_changes(&en, &cn);
    let shu = changes.iter().find(|o| o.id == "char_shu").unwrap();
    assert_eq!(
        shu.changes,
        vec![OperatorChange::ModuleAdded {
            module_id: "uniequip_002_shu".into(),
            translations: vec![
                pair("受到的伤害减少15%", "Reduces damage taken by 15%"),
                pair("攻击力+{atk}", "ATK +{atk}"),
            ],
        }]
    );
}
