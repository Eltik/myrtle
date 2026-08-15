import { InfoContent } from "frontend";

// The Information tab: profile grid, the promotion/level/trust/potential/module
// controls, the derived combat-stat table, tags, range, module details and
// talents. Every number is computed live from the payload's key frames, so the
// fixtures carry the real `attributesKeyFrames`, `favorKeyFrames`,
// `potentialRanks` and module phases from `/api/operators/<id>`.
//
// Two data-coupled pieces resolve to their empty branch in a preview: the attack
// range (`/static/ranges`) and the community Operator Notes block.
const MLYNAR = {
    "id": "char_4064_mlynar",
    "name": "Młynar",
    "description": "Normally does not attack and has 0 Block; When skill is inactive, ATK gradually increases up to <@ba.kw>+{atk:0%}</> over <@ba.kw>{max_stack_cnt}</> seconds. ATK is reset when the skill ends",
    "rarity": "TIER_6",
    "profession": "WARRIOR",
    "subProfessionId": "librator",
    "position": "MELEE",
    "tagList": [
        "DPS",
        "Nuker"
    ],
    "nationId": "kazimierz",
    "groupId": null,
    "teamId": null,
    "artists": [
        "竜崎いち"
    ],
    "portrait": "/portraits/char_4064_mlynar_2.png",
    "skin": "/textures/chararts/char_4064_mlynar/char_4064_mlynar_2.png",
    "server": "en",
    "isNotObtainable": false,
    "trait": {
        "candidates": [
            {
                "unlockCondition": {
                    "phase": "PHASE_0",
                    "level": 1
                },
                "requiredPotentialRank": 0,
                "blackboard": [
                    {
                        "key": "atk",
                        "value": 2,
                        "valueStr": null
                    },
                    {
                        "key": "mid_stack_cnt",
                        "value": 20,
                        "valueStr": null
                    },
                    {
                        "key": "max_stack_cnt",
                        "value": 40,
                        "valueStr": null
                    }
                ],
                "overrideDescription": null
            }
        ]
    },
    "phases": [
        {
            "rangeId": "1-2",
            "maxLevel": 50,
            "attributesKeyFrames": [
                {
                    "level": 1,
                    "data": {
                        "maxHp": 1945,
                        "atk": 161,
                        "def": 239,
                        "magicResistance": 15,
                        "cost": 10,
                        "blockCnt": 2,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.2000000476837158,
                        "respawnTime": 70
                    }
                },
                {
                    "level": 50,
                    "data": {
                        "maxHp": 2560,
                        "atk": 231,
                        "def": 332,
                        "magicResistance": 15,
                        "cost": 10,
                        "blockCnt": 2,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.2000000476837158,
                        "respawnTime": 70
                    }
                }
            ]
        },
        {
            "rangeId": "1-2",
            "maxLevel": 80,
            "attributesKeyFrames": [
                {
                    "level": 1,
                    "data": {
                        "maxHp": 2560,
                        "atk": 231,
                        "def": 332,
                        "magicResistance": 15,
                        "cost": 12,
                        "blockCnt": 2,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.2000000476837158,
                        "respawnTime": 70
                    }
                },
                {
                    "level": 80,
                    "data": {
                        "maxHp": 3241,
                        "atk": 301,
                        "def": 426,
                        "magicResistance": 15,
                        "cost": 12,
                        "blockCnt": 2,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.2000000476837158,
                        "respawnTime": 70
                    }
                }
            ]
        },
        {
            "rangeId": "1-2",
            "maxLevel": 90,
            "attributesKeyFrames": [
                {
                    "level": 1,
                    "data": {
                        "maxHp": 3241,
                        "atk": 301,
                        "def": 426,
                        "magicResistance": 15,
                        "cost": 12,
                        "blockCnt": 3,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.2000000476837158,
                        "respawnTime": 70
                    }
                },
                {
                    "level": 90,
                    "data": {
                        "maxHp": 3906,
                        "atk": 355,
                        "def": 502,
                        "magicResistance": 15,
                        "cost": 12,
                        "blockCnt": 3,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.2000000476837158,
                        "respawnTime": 70
                    }
                }
            ]
        }
    ],
    "favorKeyFrames": [
        {
            "level": 0,
            "data": {
                "maxHp": 0,
                "atk": 0,
                "def": 0,
                "magicResistance": 0,
                "cost": 0,
                "blockCnt": 0,
                "moveSpeed": 0,
                "attackSpeed": 0,
                "baseAttackTime": 0,
                "respawnTime": 0
            }
        },
        {
            "level": 50,
            "data": {
                "maxHp": 360,
                "atk": 30,
                "def": 0,
                "magicResistance": 0,
                "cost": 0,
                "blockCnt": 0,
                "moveSpeed": 0,
                "attackSpeed": 0,
                "baseAttackTime": 0,
                "respawnTime": 0
            }
        }
    ],
    "potentialRanks": [
        {
            "type": "BUFF",
            "description": "DP Cost -1",
            "buff": {
                "attributes": {
                    "attributeModifiers": [
                        {
                            "attributeType": "COST",
                            "formulaItem": "ADDITION",
                            "value": -1
                        }
                    ]
                }
            }
        },
        {
            "type": "CUSTOM",
            "description": "Improves First Talent",
            "buff": null
        },
        {
            "type": "BUFF",
            "description": "ATK +25",
            "buff": {
                "attributes": {
                    "attributeModifiers": [
                        {
                            "attributeType": "ATK",
                            "formulaItem": "ADDITION",
                            "value": 25
                        }
                    ]
                }
            }
        },
        {
            "type": "CUSTOM",
            "description": "Improves Second Talent",
            "buff": null
        },
        {
            "type": "BUFF",
            "description": "DP Cost -1",
            "buff": {
                "attributes": {
                    "attributeModifiers": [
                        {
                            "attributeType": "COST",
                            "formulaItem": "ADDITION",
                            "value": -1
                        }
                    ]
                }
            }
        }
    ],
    "talents": [
        {
            "candidates": [
                {
                    "unlockCondition": {
                        "phase": "PHASE_1",
                        "level": 1
                    },
                    "requiredPotentialRank": 0,
                    "name": "Wanderer",
                    "description": "ATK increased to 105% when attacking. If there are 3 or more enemies nearby, ATK increased to 110%, and take 10% less damage",
                    "rangeId": null,
                    "blackboard": [
                        {
                            "key": "atk_scale_base",
                            "value": 1.0499999523162842,
                            "valueStr": null
                        },
                        {
                            "key": "cnt",
                            "value": 3,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale_up",
                            "value": 1.100000023841858,
                            "valueStr": null
                        },
                        {
                            "key": "damage_resistance",
                            "value": 0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "unlockCondition": {
                        "phase": "PHASE_1",
                        "level": 1
                    },
                    "requiredPotentialRank": 2,
                    "name": "Wanderer",
                    "description": "ATK increased to 108%<@ba.talpu>(+3%)</>when attacking. If there are 3 or more enemies nearby, ATK increased to 113%<@ba.talpu>(+3%)</>, and take 10% less damage",
                    "rangeId": null,
                    "blackboard": [
                        {
                            "key": "atk_scale_base",
                            "value": 1.0800000429153442,
                            "valueStr": null
                        },
                        {
                            "key": "cnt",
                            "value": 3,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale_up",
                            "value": 1.1299999952316284,
                            "valueStr": null
                        },
                        {
                            "key": "damage_resistance",
                            "value": 0.10000000149011612,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "unlockCondition": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "requiredPotentialRank": 0,
                    "name": "Wanderer",
                    "description": "ATK increased to 110% when attacking. If there are 3 or more enemies nearby, ATK increased to 115%, and take 15% less damage",
                    "rangeId": null,
                    "blackboard": [
                        {
                            "key": "atk_scale_base",
                            "value": 1.100000023841858,
                            "valueStr": null
                        },
                        {
                            "key": "cnt",
                            "value": 3,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale_up",
                            "value": 1.149999976158142,
                            "valueStr": null
                        },
                        {
                            "key": "damage_resistance",
                            "value": 0.15000000596046448,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "unlockCondition": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "requiredPotentialRank": 2,
                    "name": "Wanderer",
                    "description": "ATK increased to 113%<@ba.talpu>(+3%)</> when attacking. If there are 3 or more enemies nearby, ATK increased to 118%<@ba.talpu>(+3%)</>, and take 15% less damage",
                    "rangeId": null,
                    "blackboard": [
                        {
                            "key": "atk_scale_base",
                            "value": 1.1299999952316284,
                            "valueStr": null
                        },
                        {
                            "key": "cnt",
                            "value": 3,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale_up",
                            "value": 1.1799999475479126,
                            "valueStr": null
                        },
                        {
                            "key": "damage_resistance",
                            "value": 0.15000000596046448,
                            "valueStr": null
                        }
                    ]
                }
            ]
        },
        {
            "candidates": [
                {
                    "unlockCondition": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "requiredPotentialRank": 0,
                    "name": "Unmoved",
                    "description": "More likely to be attacked while deployed. When any Kazimierz Operator is attacked, reflect 15% of Młynar's ATK as True Damage",
                    "rangeId": null,
                    "blackboard": [
                        {
                            "key": "taunt_level",
                            "value": 1,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.15000000596046448,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "unlockCondition": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "requiredPotentialRank": 4,
                    "name": "Unmoved",
                    "description": "More likely to be attacked while deployed. When any Kazimierz Operator is attacked, reflect 18%<@ba.talpu>(+3%)</> of Młynar's ATK as True Damage",
                    "rangeId": null,
                    "blackboard": [
                        {
                            "key": "taunt_level",
                            "value": 1,
                            "valueStr": null
                        },
                        {
                            "key": "atk_scale",
                            "value": 0.18000000715255737,
                            "valueStr": null
                        }
                    ]
                }
            ]
        }
    ],
    "modules": [
        {
            "uniEquipId": "uniequip_002_mlynar",
            "uniEquipName": "'Man in Scabbard'",
            "uniEquipIcon": "uniequip_002_mlynar",
            "uniEquipDesc": "The family motto engraved on the sword is near unreadable thanks to the many times I've put it to the grindstone... though it does not need sharpening at all, for it has not seen blood for far too long. I merely became accustomed to the practice. Every day—after replying to those summons, answering those letters, helping father wash his body, and putting the children to sleep—I would head down to the garage to sharpen it.\nHow old were we when father gave us this sword to play with? He called you 'Little Nearl' while I was 'Littler Nearl', as if the first names he gave us were only for government identification. All he cared for was if we would be qualified to be 'Nearls'. After all, simply knowing you were a knight of the Nearl family would have shown you your path in life, in his time.\nBut now? Remind me, where did the path we put so much trust in back then lead us?\n...Alright, we've all changed. I became frustrated, and you became.... heh, silent.\nMy interviewer today pulled me into a small meeting room afterward, and anxiously explained to me that those questions he asked were only part of the procedure. He seemed to believe that a longsword glowing with golden light would be falling upon his neck if he had offended me—after all, the Nearls in the papers are always depicted as arrogant, bloodthirsty, and indiscriminate 'envoys of justice'. But after a moment's hesitation, he ambivalently asked me if he could tell the Nearls' story to his child. For a man who works six-point-five days a week, whose brain has no room for anything other than work, he is left with only those third-rate radio dramas—the ones he listened to during commutes—to lull his child to sleep. That is, until he stumbled upon the 'knight stories' while conducting my background check.\n...Pathetic? Perhaps. To tell his child knight tales he doesn't even believe in, only because those are the only decent stories he knows. But how can I deride him? How am I any different? After all, what drives me to sharpen this sword every day? I know well it cannot cut anything.\n...Heh, I was afraid you wouldn't even recognize me now.\nMaria is nearly old enough to understand what is going on, to know what being a 'Nearl' signifies. She should not learn it through the lamentations of someone as disappointing as me.\nI will carry the sword on me. When the two children have grown up... I should wander once more. But for now, I want only to forget that I belong to the Nearl family.\nHow did I never manage to notice what ordinary people usually occupy their eyes with? I have to fill this empty shell of mine with <i>something</i>.\n...Though, it doesn't matter what I am filled with.\n\nPerhaps Młynar was merely thinking in silence, or perhaps he mumbled these words to himself. Thanks to the excellent soundproof car windows manufactured by Mieszko, no one heard a word he spoke in that underground garage. He silently returns the sword into its scabbard, and takes out a stack of newspapers that had previously acted as a seat cushion. He reads them without thinking, with no concern for their date, or where he had previously started or stopped reading. He fills himself with their words, like inflating a balloon man.",
            "typeName1": "LIB",
            "typeName2": "X",
            "type": "ADVANCED",
            "image": "/textures/spritepack/ui_equip_big_img_hub_19/uniequip_002_mlynar.png",
            "itemCost": {
                "1": [
                    {
                        "id": "4001",
                        "count": 80000,
                        "type": "GOLD",
                        "iconId": "GOLD",
                        "image": "/textures/arts/items/item_icons_no_tiny_hub/GOLD.png"
                    },
                    {
                        "id": "mod_unlock_token",
                        "count": 4,
                        "type": "MATERIAL",
                        "iconId": "mod_unlock_token",
                        "image": "/textures/arts/ui_item_icons_0/mod_unlock_token.png"
                    },
                    {
                        "id": "30155",
                        "count": 2,
                        "type": "MATERIAL",
                        "iconId": "MTL_SL_SHJ",
                        "image": "/textures/arts/ui_item_icons_5/MTL_SL_SHJ.png"
                    }
                ],
                "2": [
                    {
                        "id": "4001",
                        "count": 100000,
                        "type": "GOLD",
                        "iconId": "GOLD",
                        "image": "/textures/arts/items/item_icons_no_tiny_hub/GOLD.png"
                    },
                    {
                        "id": "mod_unlock_token",
                        "count": 4,
                        "type": "MATERIAL",
                        "iconId": "mod_unlock_token",
                        "image": "/textures/arts/ui_item_icons_0/mod_unlock_token.png"
                    },
                    {
                        "id": "mod_update_token_1",
                        "count": 60,
                        "type": "MATERIAL",
                        "iconId": "mod_update_token_1",
                        "image": "/textures/arts/ui_item_icons_5/mod_update_token_1.png"
                    },
                    {
                        "id": "30145",
                        "count": 3,
                        "type": "MATERIAL",
                        "iconId": "MTL_SL_OEU",
                        "image": "/textures/arts/ui_item_icons_5/MTL_SL_OEU.png"
                    }
                ],
                "3": [
                    {
                        "id": "4001",
                        "count": 120000,
                        "type": "GOLD",
                        "iconId": "GOLD",
                        "image": "/textures/arts/items/item_icons_no_tiny_hub/GOLD.png"
                    },
                    {
                        "id": "mod_unlock_token",
                        "count": 4,
                        "type": "MATERIAL",
                        "iconId": "mod_unlock_token",
                        "image": "/textures/arts/ui_item_icons_0/mod_unlock_token.png"
                    },
                    {
                        "id": "mod_update_token_2",
                        "count": 20,
                        "type": "MATERIAL",
                        "iconId": "mod_update_token_2",
                        "image": "/textures/arts/ui_item_icons_5/mod_update_token_2.png"
                    },
                    {
                        "id": "30165",
                        "count": 4,
                        "type": "MATERIAL",
                        "iconId": "MTL_SL_DYT",
                        "image": "/textures/arts/ui_item_icons_5/MTL_SL_DYT.png"
                    }
                ]
            },
            "data": {
                "phases": [
                    {
                        "equipLevel": 1,
                        "attributeBlackboard": [
                            {
                                "key": "max_hp",
                                "value": 225,
                                "valueStr": null
                            },
                            {
                                "key": "atk",
                                "value": 18,
                                "valueStr": null
                            },
                            {
                                "key": "attack_speed",
                                "value": 5,
                                "valueStr": null
                            }
                        ],
                        "parts": [
                            {
                                "target": "DISPLAY",
                                "overrideTraitDataBundle": {
                                    "candidates": [
                                        {
                                            "overrideDescription": "Normally does not attack and has 0 Block; when skill is inactive, ATK gradually increases up to <@ba.kw>+200%</> over <@ba.kw>40</> seconds; ATK is reset when the skill ends",
                                            "additionalDescription": "Directly gains <@ba.kw>+100%</> charge after deployment",
                                            "blackboard": [
                                                {
                                                    "key": "init_atk",
                                                    "value": 1,
                                                    "valueStr": null
                                                }
                                            ]
                                        }
                                    ]
                                },
                                "addOrOverrideTalentDataBundle": {
                                    "candidates": []
                                }
                            },
                            {
                                "target": "TALENT",
                                "overrideTraitDataBundle": {
                                    "candidates": []
                                },
                                "addOrOverrideTalentDataBundle": {
                                    "candidates": [
                                        {
                                            "name": "",
                                            "upgradeDescription": "",
                                            "description": null,
                                            "talentIndex": -1,
                                            "requiredPotentialRank": 0,
                                            "blackboard": [
                                                {
                                                    "key": "init_atk",
                                                    "value": 1,
                                                    "valueStr": null
                                                }
                                            ]
                                        }
                                    ]
                                }
                            }
                        ]
                    },
                    {
                        "equipLevel": 2,
                        "attributeBlackboard": [
                            {
                                "key": "max_hp",
                                "value": 270,
                                "valueStr": null
                            },
                            {
                                "key": "atk",
                                "value": 27,
                                "valueStr": null
                            },
                            {
                                "key": "attack_speed",
                                "value": 6,
                                "valueStr": null
                            }
                        ],
                        "parts": [
                            {
                                "target": "DISPLAY",
                                "overrideTraitDataBundle": {
                                    "candidates": [
                                        {
                                            "overrideDescription": "Normally does not attack and has 0 Block; when skill is inactive, ATK gradually increases up to <@ba.kw>+200%</> over <@ba.kw>40</> seconds; ATK is reset when the skill ends",
                                            "additionalDescription": "Directly gains <@ba.kw>+100%</> charge after deployment",
                                            "blackboard": [
                                                {
                                                    "key": "init_atk",
                                                    "value": 1,
                                                    "valueStr": null
                                                }
                                            ]
                                        }
                                    ]
                                },
                                "addOrOverrideTalentDataBundle": {
                                    "candidates": []
                                }
                            },
                            {
                                "target": "TALENT_DATA_ONLY",
                                "overrideTraitDataBundle": {
                                    "candidates": []
                                },
                                "addOrOverrideTalentDataBundle": {
                                    "candidates": [
                                        {
                                            "name": "Wanderer",
                                            "upgradeDescription": "ATK increased to 115% when attacking. If there are 3 or more enemies nearby, ATK increased to 120%, and takes 20% less damage",
                                            "description": null,
                                            "talentIndex": 0,
                                            "requiredPotentialRank": 0,
                                            "blackboard": [
                                                {
                                                    "key": "atk_scale_base",
                                                    "value": 1.149999976158142,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "cnt",
                                                    "value": 3,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "atk_scale_up",
                                                    "value": 1.2000000476837158,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "damage_resistance",
                                                    "value": 0.20000000298023224,
                                                    "valueStr": null
                                                }
                                            ]
                                        },
                                        {
                                            "name": "Wanderer",
                                            "upgradeDescription": "ATK increased to 118% <@ba.talpu>(+3%)</> when attacking. If there are 3 or more enemies nearby, ATK increased to 123% <@ba.talpu>(+3%)</>, and takes 20% less damage",
                                            "description": null,
                                            "talentIndex": 0,
                                            "requiredPotentialRank": 2,
                                            "blackboard": [
                                                {
                                                    "key": "atk_scale_base",
                                                    "value": 1.1799999475479126,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "cnt",
                                                    "value": 3,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "atk_scale_up",
                                                    "value": 1.2300000190734863,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "damage_resistance",
                                                    "value": 0.20000000298023224,
                                                    "valueStr": null
                                                }
                                            ]
                                        }
                                    ]
                                }
                            },
                            {
                                "target": "TALENT",
                                "overrideTraitDataBundle": {
                                    "candidates": []
                                },
                                "addOrOverrideTalentDataBundle": {
                                    "candidates": [
                                        {
                                            "name": "",
                                            "upgradeDescription": "",
                                            "description": null,
                                            "talentIndex": -1,
                                            "requiredPotentialRank": 0,
                                            "blackboard": [
                                                {
                                                    "key": "init_atk",
                                                    "value": 1,
                                                    "valueStr": null
                                                }
                                            ]
                                        }
                                    ]
                                }
                            }
                        ]
                    },
                    {
                        "equipLevel": 3,
                        "attributeBlackboard": [
                            {
                                "key": "max_hp",
                                "value": 360,
                                "valueStr": null
                            },
                            {
                                "key": "atk",
                                "value": 35,
                                "valueStr": null
                            },
                            {
                                "key": "attack_speed",
                                "value": 7,
                                "valueStr": null
                            }
                        ],
                        "parts": [
                            {
                                "target": "DISPLAY",
                                "overrideTraitDataBundle": {
                                    "candidates": [
                                        {
                                            "overrideDescription": "Normally does not attack and has 0 Block; when skill is inactive, ATK gradually increases up to <@ba.kw>+200%</> over <@ba.kw>40</> seconds; ATK is reset when the skill ends",
                                            "additionalDescription": "Directly gains <@ba.kw>+100%</> charge after deployment",
                                            "blackboard": [
                                                {
                                                    "key": "init_atk",
                                                    "value": 1,
                                                    "valueStr": null
                                                }
                                            ]
                                        }
                                    ]
                                },
                                "addOrOverrideTalentDataBundle": {
                                    "candidates": []
                                }
                            },
                            {
                                "target": "TALENT_DATA_ONLY",
                                "overrideTraitDataBundle": {
                                    "candidates": []
                                },
                                "addOrOverrideTalentDataBundle": {
                                    "candidates": [
                                        {
                                            "name": "Wanderer",
                                            "upgradeDescription": "ATK increased to 120% when attacking. If there are 3 or more enemies nearby, ATK increased to 125%, and takes 25% less damage",
                                            "description": null,
                                            "talentIndex": 0,
                                            "requiredPotentialRank": 0,
                                            "blackboard": [
                                                {
                                                    "key": "atk_scale_base",
                                                    "value": 1.2000000476837158,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "cnt",
                                                    "value": 3,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "atk_scale_up",
                                                    "value": 1.25,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "damage_resistance",
                                                    "value": 0.25,
                                                    "valueStr": null
                                                }
                                            ]
                                        },
                                        {
                                            "name": "Wanderer",
                                            "upgradeDescription": "ATK increased to 123% <@ba.talpu>(+3%)</> when attacking. If there are 3 or more enemies nearby, ATK increased to 128% <@ba.talpu>(+3%)</>, and takes 25% less damage",
                                            "description": null,
                                            "talentIndex": 0,
                                            "requiredPotentialRank": 2,
                                            "blackboard": [
                                                {
                                                    "key": "atk_scale_base",
                                                    "value": 1.2300000190734863,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "cnt",
                                                    "value": 3,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "atk_scale_up",
                                                    "value": 1.2799999713897705,
                                                    "valueStr": null
                                                },
                                                {
                                                    "key": "damage_resistance",
                                                    "value": 0.25,
                                                    "valueStr": null
                                                }
                                            ]
                                        }
                                    ]
                                }
                            },
                            {
                                "target": "TALENT",
                                "overrideTraitDataBundle": {
                                    "candidates": []
                                },
                                "addOrOverrideTalentDataBundle": {
                                    "candidates": [
                                        {
                                            "name": "",
                                            "upgradeDescription": "",
                                            "description": null,
                                            "talentIndex": -1,
                                            "requiredPotentialRank": 0,
                                            "blackboard": [
                                                {
                                                    "key": "init_atk",
                                                    "value": 1,
                                                    "valueStr": null
                                                }
                                            ]
                                        }
                                    ]
                                }
                            }
                        ]
                    }
                ]
            }
        },
        {
            "uniEquipId": "uniequip_001_mlynar",
            "uniEquipName": "Młynar's Badge",
            "uniEquipIcon": "uniequip_001_mlynar",
            "uniEquipDesc": "Operator Młynar has demonstrated excellent timing when dealing with perilous combat situations.\nThe Field Operations Department has thus passed the following resolution:\nThis operator shall be appointed a Guard Operator during field operations to exercise Liberator responsibilities.\nIn witness whereof,\nThis badge is hereby conferred upon the above named.",
            "typeName1": "ORIGINAL",
            "typeName2": null,
            "type": "INITIAL",
            "image": null,
            "itemCost": {},
            "data": {
                "phases": []
            }
        }
    ],
    "baseSkills": [
        {
            "buffId": "control_mp_cost[008]",
            "buffName": "Self-Absorbed",
            "description": "When this Operator is assigned to the Control Center, increases Morale of all Operators in the Control Center by <@cc.vup>+0.05</> per hour.",
            "roomType": "CONTROL",
            "targets": [],
            "skillIcon": "bskill_ctrl_cost",
            "unlockElite": 0,
            "unlockLevel": 1
        },
        {
            "buffId": "control_mp_lonely[000]",
            "buffName": "Business Is Business",
            "description": "When this Operator is assigned to the Control Center, working Operators in <$cc.c.room1><@cc.kw>certain facilities</></> will recover <@cc.vup>+0.1</> Morale per hour, and <$cc.c.skill><@cc.kw>some skills</></> in the Control Center will provide additional Morale recovery for Operators working in <$cc.c.room2><@cc.kw>other facilities</></>",
            "roomType": "CONTROL",
            "targets": [],
            "skillIcon": "bskill_ctrl_lonely",
            "unlockElite": 2,
            "unlockLevel": 1
        }
    ],
    "profile": {
        "basicInfo": {
            "codeName": "Młynar",
            "gender": "Male",
            "combatExperience": "17 Years",
            "placeOfBirth": "Kazimierz",
            "dateOfBirth": "Dec 3",
            "race": "Kuranta",
            "height": "191cm",
            "infectionStatus": ""
        }
    },
    "drones": [],
    "skills": []
};

const TEXAS = {
    "id": "char_102_texas",
    "name": "Texas",
    "description": "Blocks 2 enemies",
    "rarity": "TIER_5",
    "profession": "PIONEER",
    "subProfessionId": "pioneer",
    "position": "MELEE",
    "tagList": [
        "DP-Recovery",
        "Crowd-Control"
    ],
    "nationId": "lungmen",
    "groupId": "penguin",
    "teamId": null,
    "artists": [
        "幻象黑兔"
    ],
    "portrait": "/portraits/char_102_texas_2.png",
    "skin": "/textures/chararts/char_102_texas/char_102_texas_2.png",
    "server": "en",
    "isNotObtainable": false,
    "trait": null,
    "phases": [
        {
            "rangeId": "1-1",
            "maxLevel": 50,
            "attributesKeyFrames": [
                {
                    "level": 1,
                    "data": {
                        "maxHp": 727,
                        "atk": 203,
                        "def": 139,
                        "magicResistance": 0,
                        "cost": 11,
                        "blockCnt": 2,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.0499999523162842,
                        "respawnTime": 70
                    }
                },
                {
                    "level": 50,
                    "data": {
                        "maxHp": 996,
                        "atk": 299,
                        "def": 208,
                        "magicResistance": 0,
                        "cost": 11,
                        "blockCnt": 2,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.0499999523162842,
                        "respawnTime": 70
                    }
                }
            ]
        },
        {
            "rangeId": "1-1",
            "maxLevel": 70,
            "attributesKeyFrames": [
                {
                    "level": 1,
                    "data": {
                        "maxHp": 996,
                        "atk": 299,
                        "def": 208,
                        "magicResistance": 0,
                        "cost": 13,
                        "blockCnt": 2,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.0499999523162842,
                        "respawnTime": 70
                    }
                },
                {
                    "level": 70,
                    "data": {
                        "maxHp": 1365,
                        "atk": 410,
                        "def": 274,
                        "magicResistance": 0,
                        "cost": 13,
                        "blockCnt": 2,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.0499999523162842,
                        "respawnTime": 70
                    }
                }
            ]
        },
        {
            "rangeId": "1-1",
            "maxLevel": 80,
            "attributesKeyFrames": [
                {
                    "level": 1,
                    "data": {
                        "maxHp": 1365,
                        "atk": 410,
                        "def": 274,
                        "magicResistance": 0,
                        "cost": 13,
                        "blockCnt": 2,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.0499999523162842,
                        "respawnTime": 70
                    }
                },
                {
                    "level": 80,
                    "data": {
                        "maxHp": 1950,
                        "atk": 500,
                        "def": 343,
                        "magicResistance": 0,
                        "cost": 13,
                        "blockCnt": 2,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 1.0499999523162842,
                        "respawnTime": 70
                    }
                }
            ]
        }
    ],
    "favorKeyFrames": [
        {
            "level": 0,
            "data": {
                "maxHp": 0,
                "atk": 0,
                "def": 0,
                "magicResistance": 0,
                "cost": 0,
                "blockCnt": 0,
                "moveSpeed": 0,
                "attackSpeed": 0,
                "baseAttackTime": 0,
                "respawnTime": 0
            }
        },
        {
            "level": 50,
            "data": {
                "maxHp": 0,
                "atk": 70,
                "def": 0,
                "magicResistance": 0,
                "cost": 0,
                "blockCnt": 0,
                "moveSpeed": 0,
                "attackSpeed": 0,
                "baseAttackTime": 0,
                "respawnTime": 0
            }
        }
    ],
    "potentialRanks": [
        {
            "type": "BUFF",
            "description": "DP Cost -1",
            "buff": {
                "attributes": {
                    "attributeModifiers": [
                        {
                            "attributeType": "COST",
                            "formulaItem": "ADDITION",
                            "value": -1
                        }
                    ]
                }
            }
        },
        {
            "type": "BUFF",
            "description": "Redeployment Time -4 sec",
            "buff": {
                "attributes": {
                    "attributeModifiers": [
                        {
                            "attributeType": "RESPAWN_TIME",
                            "formulaItem": "ADDITION",
                            "value": -4
                        }
                    ]
                }
            }
        },
        {
            "type": "BUFF",
            "description": "ATK +24",
            "buff": {
                "attributes": {
                    "attributeModifiers": [
                        {
                            "attributeType": "ATK",
                            "formulaItem": "ADDITION",
                            "value": 24
                        }
                    ]
                }
            }
        },
        {
            "type": "BUFF",
            "description": "Redeployment Time -6 sec",
            "buff": {
                "attributes": {
                    "attributeModifiers": [
                        {
                            "attributeType": "RESPAWN_TIME",
                            "formulaItem": "ADDITION",
                            "value": -6
                        }
                    ]
                }
            }
        },
        {
            "type": "BUFF",
            "description": "DP Cost -1",
            "buff": {
                "attributes": {
                    "attributeModifiers": [
                        {
                            "attributeType": "COST",
                            "formulaItem": "ADDITION",
                            "value": -1
                        }
                    ]
                }
            }
        }
    ],
    "talents": [
        {
            "candidates": [
                {
                    "unlockCondition": {
                        "phase": "PHASE_1",
                        "level": 1
                    },
                    "requiredPotentialRank": 0,
                    "name": "Tactical Delivery",
                    "description": "Gains 1 extra DP when Texas is in the squad",
                    "rangeId": null,
                    "blackboard": [
                        {
                            "key": "cost",
                            "value": 1,
                            "valueStr": null
                        }
                    ]
                },
                {
                    "unlockCondition": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "requiredPotentialRank": 0,
                    "name": "Tactical Delivery",
                    "description": "Gains 2 extra DP when Texas is in the squad",
                    "rangeId": null,
                    "blackboard": [
                        {
                            "key": "cost",
                            "value": 2,
                            "valueStr": null
                        }
                    ]
                }
            ]
        }
    ],
    "modules": [],
    "baseSkills": [
        {
            "buffId": "trade_ord_spd&cost_P[000]",
            "buffName": "Feud",
            "description": "When this Operator is assigned to the same Trading Post as <@cc.kw>Lappland</>, Morale consumed each hour <@cc.vdown>+0.3</>, and order acquisition efficiency <@cc.vup>+65%</>",
            "roomType": "TRADING",
            "targets": [],
            "skillIcon": "bskill_tra_texas1",
            "unlockElite": 0,
            "unlockLevel": 1
        },
        {
            "buffId": "trade_ord_limit&cost_P[010]",
            "buffName": "Tacit Understanding",
            "description": "When this Operator is assigned to the same Trading Post as <@cc.kw>Exusiai</>, Morale consumed each hour <@cc.vup>-0.3</>",
            "roomType": "TRADING",
            "targets": [],
            "skillIcon": "bskill_tra_texas2",
            "unlockElite": 2,
            "unlockLevel": 1
        }
    ],
    "profile": {
        "basicInfo": {
            "codeName": "Texas",
            "gender": "Female",
            "combatExperience": "3 years",
            "placeOfBirth": "Columbia",
            "dateOfBirth": "Jun. 1",
            "race": "Lupo",
            "height": "161cm",
            "infectionStatus": ""
        }
    },
    "drones": [],
    "skills": []
};

export const MaxedSixStar = () => <InfoContent operator={MLYNAR} />;

export const FiveStarVanguard = () => <InfoContent operator={TEXAS} />;
