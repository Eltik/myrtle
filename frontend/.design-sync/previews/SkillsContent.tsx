import { SkillsContent } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The Skills tab: skill picker, a level slider that runs Lv.1 -> M3, and the
// mastery comparison table behind the "Compare Skill Levels" switch. Fixture is
// Młynar's three skills with all ten levels, verbatim from
// `/api/operators/char_4064_mlynar`.
//
// The skill-range block only draws when `/static/ranges` resolves, which no
// preview can reach; none of Młynar's skills change range anyway.
const MLYNAR_SKILLS = [
    {
        "skillId": "skchr_mlynar_1",
        "levelUpCostCond": [],
        "static": {
            "skillId": "skchr_mlynar_1",
            "iconId": null,
            "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_1.png",
            "levels": [
                {
                    "name": "Unvoiced Anger",
                    "rangeId": null,
                    "description": "Attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage, DEF <@ba.vup>+{def:0%}</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 40,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.2999999523162842,
                            "valueStr": null
                        },
                        {
                            "key": "def",
                            "value": 0.20000000298023224,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unvoiced Anger",
                    "rangeId": null,
                    "description": "Attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage, DEF <@ba.vup>+{def:0%}</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 40,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.350000023841858,
                            "valueStr": null
                        },
                        {
                            "key": "def",
                            "value": 0.20000000298023224,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unvoiced Anger",
                    "rangeId": null,
                    "description": "Attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage, DEF <@ba.vup>+{def:0%}</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 40,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.399999976158142,
                            "valueStr": null
                        },
                        {
                            "key": "def",
                            "value": 0.20000000298023224,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unvoiced Anger",
                    "rangeId": null,
                    "description": "Attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage, DEF <@ba.vup>+{def:0%}</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 35,
                        "initSp": 15,
                        "increment": 1
                    },
                    "duration": 25,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.4500000476837158,
                            "valueStr": null
                        },
                        {
                            "key": "def",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unvoiced Anger",
                    "rangeId": null,
                    "description": "Attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage, DEF <@ba.vup>+{def:0%}</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 35,
                        "initSp": 15,
                        "increment": 1
                    },
                    "duration": 25,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.5,
                            "valueStr": null
                        },
                        {
                            "key": "def",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unvoiced Anger",
                    "rangeId": null,
                    "description": "Attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage, DEF <@ba.vup>+{def:0%}</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 35,
                        "initSp": 15,
                        "increment": 1
                    },
                    "duration": 25,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.600000023841858,
                            "valueStr": null
                        },
                        {
                            "key": "def",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unvoiced Anger",
                    "rangeId": null,
                    "description": "Attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage, DEF <@ba.vup>+{def:0%}</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 30,
                        "initSp": 15,
                        "increment": 1
                    },
                    "duration": 30,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.7000000476837158,
                            "valueStr": null
                        },
                        {
                            "key": "def",
                            "value": 0.4000000059604645,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unvoiced Anger",
                    "rangeId": null,
                    "description": "Attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage, DEF <@ba.vup>+{def:0%}</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 30,
                        "initSp": 15,
                        "increment": 1
                    },
                    "duration": 30,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.7999999523162842,
                            "valueStr": null
                        },
                        {
                            "key": "def",
                            "value": 0.44999998807907104,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unvoiced Anger",
                    "rangeId": null,
                    "description": "Attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage, DEF <@ba.vup>+{def:0%}</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 30,
                        "initSp": 15,
                        "increment": 1
                    },
                    "duration": 30,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.899999976158142,
                            "valueStr": null
                        },
                        {
                            "key": "def",
                            "value": 0.5,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unvoiced Anger",
                    "rangeId": null,
                    "description": "Attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage, DEF <@ba.vup>+{def:0%}</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 30,
                        "initSp": 15,
                        "increment": 1
                    },
                    "duration": 30,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 2,
                            "valueStr": null
                        },
                        {
                            "key": "def",
                            "value": 0.6000000238418579,
                            "valueStr": null
                        }
                    ]
                }
            ]
        }
    },
    {
        "skillId": "skchr_mlynar_2",
        "levelUpCostCond": [],
        "static": {
            "skillId": "skchr_mlynar_2",
            "iconId": null,
            "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_2.png",
            "levels": [
                {
                    "name": "Unresolved Sorrow",
                    "rangeId": "2-3",
                    "description": "Attack interval <@ba.vdown>increased</>, attack range <@ba.vup>increased</>, attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK <@ba.vup>twice</>.\n<@ba.rem>If an enemy has been defeated while skill is active, Trait effect is not reset when skill expires. This can be manually deactivated. (Skill can be halted at any time while active)</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 40,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1,
                            "valueStr": null
                        },
                        {
                            "key": "base_attack_time",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unresolved Sorrow",
                    "rangeId": "2-3",
                    "description": "Attack interval <@ba.vdown>increased</>, attack range <@ba.vup>increased</>, attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK <@ba.vup>twice</>.\n<@ba.rem>If an enemy has been defeated while skill is active, Trait effect is not reset when skill expires. This can be manually deactivated. (Skill can be halted at any time while active)</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 40,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.100000023841858,
                            "valueStr": null
                        },
                        {
                            "key": "base_attack_time",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unresolved Sorrow",
                    "rangeId": "2-3",
                    "description": "Attack interval <@ba.vdown>increased</>, attack range <@ba.vup>increased</>, attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK <@ba.vup>twice</>.\n<@ba.rem>If an enemy has been defeated while skill is active, Trait effect is not reset when skill expires. This can be manually deactivated. (Skill can be halted at any time while active)</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 40,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.2000000476837158,
                            "valueStr": null
                        },
                        {
                            "key": "base_attack_time",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unresolved Sorrow",
                    "rangeId": "2-3",
                    "description": "Attack interval <@ba.vdown>increased</>, attack range <@ba.vup>increased</>, attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK <@ba.vup>twice</>.\n<@ba.rem>If an enemy has been defeated while skill is active, Trait effect is not reset when skill expires. This can be manually deactivated. (Skill can be halted at any time while active)</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 35,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.2999999523162842,
                            "valueStr": null
                        },
                        {
                            "key": "base_attack_time",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unresolved Sorrow",
                    "rangeId": "2-3",
                    "description": "Attack interval <@ba.vdown>increased</>, attack range <@ba.vup>increased</>, attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK <@ba.vup>twice</>.\n<@ba.rem>If an enemy has been defeated while skill is active, Trait effect is not reset when skill expires. This can be manually deactivated. (Skill can be halted at any time while active)</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 35,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.399999976158142,
                            "valueStr": null
                        },
                        {
                            "key": "base_attack_time",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unresolved Sorrow",
                    "rangeId": "2-3",
                    "description": "Attack interval <@ba.vdown>increased</>, attack range <@ba.vup>increased</>, attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK <@ba.vup>twice</>.\n<@ba.rem>If an enemy has been defeated while skill is active, Trait effect is not reset when skill expires. This can be manually deactivated. (Skill can be halted at any time while active)</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 35,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.5,
                            "valueStr": null
                        },
                        {
                            "key": "base_attack_time",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unresolved Sorrow",
                    "rangeId": "2-3",
                    "description": "Attack interval <@ba.vdown>increased</>, attack range <@ba.vup>increased</>, attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK <@ba.vup>twice</>.\n<@ba.rem>If an enemy has been defeated while skill is active, Trait effect is not reset when skill expires. This can be manually deactivated. (Skill can be halted at any time while active)</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 30,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.600000023841858,
                            "valueStr": null
                        },
                        {
                            "key": "base_attack_time",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unresolved Sorrow",
                    "rangeId": "2-3",
                    "description": "Attack interval <@ba.vdown>increased</>, attack range <@ba.vup>increased</>, attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK <@ba.vup>twice</>.\n<@ba.rem>If an enemy has been defeated while skill is active, Trait effect is not reset when skill expires. This can be manually deactivated. (Skill can be halted at any time while active)</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 30,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.7000000476837158,
                            "valueStr": null
                        },
                        {
                            "key": "base_attack_time",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unresolved Sorrow",
                    "rangeId": "2-3",
                    "description": "Attack interval <@ba.vdown>increased</>, attack range <@ba.vup>increased</>, attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK <@ba.vup>twice</>.\n<@ba.rem>If an enemy has been defeated while skill is active, Trait effect is not reset when skill expires. This can be manually deactivated. (Skill can be halted at any time while active)</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 30,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.7999999523162842,
                            "valueStr": null
                        },
                        {
                            "key": "base_attack_time",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unresolved Sorrow",
                    "rangeId": "2-3",
                    "description": "Attack interval <@ba.vdown>increased</>, attack range <@ba.vup>increased</>, attacks deal <@ba.vup>{attack@atk_scale:0%}</> ATK <@ba.vup>twice</>.\n<@ba.rem>If an enemy has been defeated while skill is active, Trait effect is not reset when skill expires. This can be manually deactivated. (Skill can be halted at any time while active)</>",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 25,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 20,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.899999976158142,
                            "valueStr": null
                        },
                        {
                            "key": "base_attack_time",
                            "value": 0.30000001192092896,
                            "valueStr": null
                        }
                    ]
                }
            ]
        }
    },
    {
        "skillId": "skchr_mlynar_3",
        "levelUpCostCond": [],
        "static": {
            "skillId": "skchr_mlynar_3",
            "iconId": null,
            "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_3.png",
            "levels": [
                {
                    "name": "Unbrilliant Glory",
                    "rangeId": "3-18",
                    "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 55,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 25,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.100000023841858,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.07999999821186066,
                            "valueStr": null
                        },
                        {
                            "key": "attack@max_target",
                            "value": 5,
                            "valueStr": null
                        },
                        {
                            "key": "trait_up",
                            "value": 2,
                            "valueStr": null
                        },
                        {
                            "key": "per_kill_reduce",
                            "value": -0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unbrilliant Glory",
                    "rangeId": "3-18",
                    "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 55,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 25,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.149999976158142,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.07999999821186066,
                            "valueStr": null
                        },
                        {
                            "key": "attack@max_target",
                            "value": 5,
                            "valueStr": null
                        },
                        {
                            "key": "trait_up",
                            "value": 2,
                            "valueStr": null
                        },
                        {
                            "key": "per_kill_reduce",
                            "value": -0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unbrilliant Glory",
                    "rangeId": "3-18",
                    "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 55,
                        "initSp": 10,
                        "increment": 1
                    },
                    "duration": 25,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.2000000476837158,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.07999999821186066,
                            "valueStr": null
                        },
                        {
                            "key": "attack@max_target",
                            "value": 5,
                            "valueStr": null
                        },
                        {
                            "key": "trait_up",
                            "value": 2,
                            "valueStr": null
                        },
                        {
                            "key": "per_kill_reduce",
                            "value": -0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unbrilliant Glory",
                    "rangeId": "3-18",
                    "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 50,
                        "initSp": 15,
                        "increment": 1
                    },
                    "duration": 26,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.25,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.10000000149011612,
                            "valueStr": null
                        },
                        {
                            "key": "attack@max_target",
                            "value": 5,
                            "valueStr": null
                        },
                        {
                            "key": "trait_up",
                            "value": 2,
                            "valueStr": null
                        },
                        {
                            "key": "per_kill_reduce",
                            "value": -0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unbrilliant Glory",
                    "rangeId": "3-18",
                    "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 50,
                        "initSp": 15,
                        "increment": 1
                    },
                    "duration": 26,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.2999999523162842,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.10000000149011612,
                            "valueStr": null
                        },
                        {
                            "key": "attack@max_target",
                            "value": 5,
                            "valueStr": null
                        },
                        {
                            "key": "trait_up",
                            "value": 2,
                            "valueStr": null
                        },
                        {
                            "key": "per_kill_reduce",
                            "value": -0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unbrilliant Glory",
                    "rangeId": "3-18",
                    "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 50,
                        "initSp": 15,
                        "increment": 1
                    },
                    "duration": 26,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.399999976158142,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.10000000149011612,
                            "valueStr": null
                        },
                        {
                            "key": "attack@max_target",
                            "value": 5,
                            "valueStr": null
                        },
                        {
                            "key": "trait_up",
                            "value": 2,
                            "valueStr": null
                        },
                        {
                            "key": "per_kill_reduce",
                            "value": -0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unbrilliant Glory",
                    "rangeId": "3-18",
                    "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 45,
                        "initSp": 20,
                        "increment": 1
                    },
                    "duration": 26,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.5,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.10999999940395357,
                            "valueStr": null
                        },
                        {
                            "key": "attack@max_target",
                            "value": 5,
                            "valueStr": null
                        },
                        {
                            "key": "trait_up",
                            "value": 2,
                            "valueStr": null
                        },
                        {
                            "key": "per_kill_reduce",
                            "value": -0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unbrilliant Glory",
                    "rangeId": "3-18",
                    "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 45,
                        "initSp": 20,
                        "increment": 1
                    },
                    "duration": 27,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.600000023841858,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.10999999940395357,
                            "valueStr": null
                        },
                        {
                            "key": "attack@max_target",
                            "value": 5,
                            "valueStr": null
                        },
                        {
                            "key": "trait_up",
                            "value": 2,
                            "valueStr": null
                        },
                        {
                            "key": "per_kill_reduce",
                            "value": -0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unbrilliant Glory",
                    "rangeId": "3-18",
                    "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 45,
                        "initSp": 20,
                        "increment": 1
                    },
                    "duration": 27,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.7000000476837158,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.10999999940395357,
                            "valueStr": null
                        },
                        {
                            "key": "attack@max_target",
                            "value": 5,
                            "valueStr": null
                        },
                        {
                            "key": "trait_up",
                            "value": 2,
                            "valueStr": null
                        },
                        {
                            "key": "per_kill_reduce",
                            "value": -0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "name": "Unbrilliant Glory",
                    "rangeId": "3-18",
                    "description": "Attack range <@ba.vup>increased</>, Trait effect increased by <@ba.vup>{trait_up}</>x (trait multiplier <@ba.vup>{per_kill_reduce:0%}</> for each enemy defeated), attacks hit <@ba.vup>{attack@max_target}</> targets for <@ba.vup>{attack@atk_scale:0%}</> ATK as Physical damage. When any enemy within range is attacked by a Kazimierz Operator, deal an extra <@ba.vup>{atk_scale:0%}</> of Młynar ATK as <@ba.vup>True</> damage.",
                    "skillType": "MANUAL",
                    "durationType": "NONE",
                    "spData": {
                        "spType": "INCREASE_WITH_TIME",
                        "maxChargeTime": 1,
                        "spCost": 42,
                        "initSp": 20,
                        "increment": 1
                    },
                    "duration": 28,
                    "blackboard": [
                        {
                            "key": "attack@atk_scale",
                            "value": 1.7999999523162842,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.119999997317791,
                            "valueStr": null
                        },
                        {
                            "key": "attack@max_target",
                            "value": 5,
                            "valueStr": null
                        },
                        {
                            "key": "trait_up",
                            "value": 2,
                            "valueStr": null
                        },
                        {
                            "key": "per_kill_reduce",
                            "value": -0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                }
            ]
        }
    }
];

const operator = (skills: unknown) => ({ id: "char_4064_mlynar", name: "Młynar", server: "en", skills, phases: [{ rangeId: "1-2" }, { rangeId: "1-2" }, { rangeId: "1-2" }] });

// Comparison mode is an internal `useState` behind a Switch, so the story flips
// it on mount. Base UI wires the control after the first paint - wait two frames.
const EnableComparison = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => (ref.current?.querySelector('[role="switch"]') as HTMLElement | null)?.click());
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, []);
    return <div ref={ref}>{children}</div>;
};

export const MasteryThreeDetail = () => <SkillsContent operator={operator(MLYNAR_SKILLS)} />;

export const LevelComparison = () => (
    <EnableComparison>
        <SkillsContent operator={operator(MLYNAR_SKILLS)} />
    </EnableComparison>
);

export const NoSkills = () => <SkillsContent operator={{ id: "char_285_medic2", name: "Lancet-2", server: "en", skills: [], phases: [{ rangeId: "3-1" }] }} />;
