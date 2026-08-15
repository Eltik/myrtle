import { LevelUpContent } from "frontend";

// The Level-Up Cost tab: promotion, level, skill-rank, mastery and module
// material ladders plus a grand total. Item names come from `/static/materials`,
// which no preview can fetch, so the tiles show the icon + count the way they do
// before that request lands; every icon and count is real.
//
// Fixtures are `/api/operators/char_4064_mlynar` and `/api/operators/char_285_medic2`.
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
            ],
            "evolveCost": [],
            "levelUpCost": [
                {
                    "id": "4001",
                    "count": 26719,
                    "type": "GOLD",
                    "iconId": "GOLD",
                    "image": "/textures/arts/items/item_icons_no_tiny_hub/GOLD.png"
                },
                {
                    "id": "5001",
                    "count": 24400,
                    "type": "EXP_PLAYER",
                    "iconId": "EXP_PLAYER",
                    "image": "/textures/arts/ui_item_icons_4/EXP_PLAYER.png"
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
            ],
            "evolveCost": [
                {
                    "id": "4001",
                    "count": 30000,
                    "type": "GOLD",
                    "iconId": "GOLD",
                    "image": "/textures/arts/items/item_icons_no_tiny_hub/GOLD.png"
                },
                {
                    "id": "3221",
                    "count": 5,
                    "type": "MATERIAL",
                    "iconId": "MTL_ASC_GRD1",
                    "image": "/textures/arts/ui_item_icons_5/MTL_ASC_GRD1.png"
                },
                {
                    "id": "30032",
                    "count": 8,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_RUSH2",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_RUSH2.png"
                },
                {
                    "id": "30012",
                    "count": 8,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_G2",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_G2.png"
                }
            ],
            "levelUpCost": [
                {
                    "id": "4001",
                    "count": 353122,
                    "type": "GOLD",
                    "iconId": "GOLD",
                    "image": "/textures/arts/items/item_icons_no_tiny_hub/GOLD.png"
                },
                {
                    "id": "5001",
                    "count": 337000,
                    "type": "EXP_PLAYER",
                    "iconId": "EXP_PLAYER",
                    "image": "/textures/arts/ui_item_icons_4/EXP_PLAYER.png"
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
            ],
            "evolveCost": [
                {
                    "id": "4001",
                    "count": 180000,
                    "type": "GOLD",
                    "iconId": "GOLD",
                    "image": "/textures/arts/items/item_icons_no_tiny_hub/GOLD.png"
                },
                {
                    "id": "3223",
                    "count": 4,
                    "type": "MATERIAL",
                    "iconId": "MTL_ASC_GRD3",
                    "image": "/textures/arts/ui_item_icons_5/MTL_ASC_GRD3.png"
                },
                {
                    "id": "30135",
                    "count": 4,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_DS",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_DS.png"
                },
                {
                    "id": "31024",
                    "count": 5,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_IAM4",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_IAM4.png"
                }
            ],
            "levelUpCost": [
                {
                    "id": "4001",
                    "count": 744955,
                    "type": "GOLD",
                    "iconId": "GOLD",
                    "image": "/textures/arts/items/item_icons_no_tiny_hub/GOLD.png"
                },
                {
                    "id": "5001",
                    "count": 750000,
                    "type": "EXP_PLAYER",
                    "iconId": "EXP_PLAYER",
                    "image": "/textures/arts/ui_item_icons_4/EXP_PLAYER.png"
                }
            ]
        }
    ],
    "allSkillLevelUp": [
        {
            "unlockCond": {
                "phase": "PHASE_0",
                "level": 1
            },
            "lvlUpCost": [
                {
                    "id": "3301",
                    "count": 5,
                    "type": "MATERIAL",
                    "iconId": "MTL_SKILL1",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SKILL1.png"
                }
            ]
        },
        {
            "unlockCond": {
                "phase": "PHASE_0",
                "level": 1
            },
            "lvlUpCost": [
                {
                    "id": "3301",
                    "count": 5,
                    "type": "MATERIAL",
                    "iconId": "MTL_SKILL1",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SKILL1.png"
                },
                {
                    "id": "30041",
                    "count": 5,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_IRON1",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_IRON1.png"
                },
                {
                    "id": "30021",
                    "count": 4,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_STRG1",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_STRG1.png"
                }
            ]
        },
        {
            "unlockCond": {
                "phase": "PHASE_0",
                "level": 1
            },
            "lvlUpCost": [
                {
                    "id": "3302",
                    "count": 8,
                    "type": "MATERIAL",
                    "iconId": "MTL_SKILL2",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SKILL2.png"
                },
                {
                    "id": "30052",
                    "count": 4,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_KETONE2",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_KETONE2.png"
                }
            ]
        },
        {
            "unlockCond": {
                "phase": "PHASE_1",
                "level": 1
            },
            "lvlUpCost": [
                {
                    "id": "3302",
                    "count": 8,
                    "type": "MATERIAL",
                    "iconId": "MTL_SKILL2",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SKILL2.png"
                },
                {
                    "id": "30062",
                    "count": 3,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_BOSS2",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_BOSS2.png"
                },
                {
                    "id": "30032",
                    "count": 3,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_RUSH2",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_RUSH2.png"
                }
            ]
        },
        {
            "unlockCond": {
                "phase": "PHASE_1",
                "level": 1
            },
            "lvlUpCost": [
                {
                    "id": "3302",
                    "count": 8,
                    "type": "MATERIAL",
                    "iconId": "MTL_SKILL2",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SKILL2.png"
                },
                {
                    "id": "31033",
                    "count": 6,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_OC3",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_OC3.png"
                }
            ]
        },
        {
            "unlockCond": {
                "phase": "PHASE_1",
                "level": 1
            },
            "lvlUpCost": [
                {
                    "id": "3303",
                    "count": 8,
                    "type": "MATERIAL",
                    "iconId": "MTL_SKILL3",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SKILL3.png"
                },
                {
                    "id": "31043",
                    "count": 3,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_SS",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_SS.png"
                },
                {
                    "id": "30103",
                    "count": 4,
                    "type": "MATERIAL",
                    "iconId": "MTL_SL_RMA7012",
                    "image": "/textures/arts/ui_item_icons_5/MTL_SL_RMA7012.png"
                }
            ]
        }
    ],
    "skills": [
        {
            "skillId": "skchr_mlynar_1",
            "levelUpCostCond": [
                {
                    "unlockCond": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "levelUpCost": [
                        {
                            "id": "3303",
                            "count": 8,
                            "type": "MATERIAL",
                            "iconId": "MTL_SKILL3",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SKILL3.png"
                        },
                        {
                            "id": "30104",
                            "count": 3,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_RMA7024",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_RMA7024.png"
                        },
                        {
                            "id": "30083",
                            "count": 9,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_MANGANESE1",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_MANGANESE1.png"
                        }
                    ]
                },
                {
                    "unlockCond": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "levelUpCost": [
                        {
                            "id": "3303",
                            "count": 12,
                            "type": "MATERIAL",
                            "iconId": "MTL_SKILL3",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SKILL3.png"
                        },
                        {
                            "id": "30064",
                            "count": 3,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_BOSS4",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_BOSS4.png"
                        },
                        {
                            "id": "30044",
                            "count": 6,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_IRON4",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_IRON4.png"
                        }
                    ]
                },
                {
                    "unlockCond": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "levelUpCost": [
                        {
                            "id": "3303",
                            "count": 15,
                            "type": "MATERIAL",
                            "iconId": "MTL_SKILL3",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SKILL3.png"
                        },
                        {
                            "id": "30115",
                            "count": 6,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_PP",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_PP.png"
                        },
                        {
                            "id": "30074",
                            "count": 7,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_ALCOHOL2",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_ALCOHOL2.png"
                        }
                    ]
                }
            ],
            "static": {
                "skillId": "skchr_mlynar_1",
                "iconId": null,
                "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_1.png",
                "levels": [
                    {
                        "name": "Unvoiced Anger"
                    }
                ]
            }
        },
        {
            "skillId": "skchr_mlynar_2",
            "levelUpCostCond": [
                {
                    "unlockCond": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "levelUpCost": [
                        {
                            "id": "3303",
                            "count": 8,
                            "type": "MATERIAL",
                            "iconId": "MTL_SKILL3",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SKILL3.png"
                        },
                        {
                            "id": "31034",
                            "count": 4,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_OC4",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_OC4.png"
                        },
                        {
                            "id": "31013",
                            "count": 3,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_PGEL3",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_PGEL3.png"
                        }
                    ]
                },
                {
                    "unlockCond": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "levelUpCost": [
                        {
                            "id": "3303",
                            "count": 12,
                            "type": "MATERIAL",
                            "iconId": "MTL_SKILL3",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SKILL3.png"
                        },
                        {
                            "id": "30074",
                            "count": 4,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_ALCOHOL2",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_ALCOHOL2.png"
                        },
                        {
                            "id": "30054",
                            "count": 8,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_KETONE4",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_KETONE4.png"
                        }
                    ]
                },
                {
                    "unlockCond": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "levelUpCost": [
                        {
                            "id": "3303",
                            "count": 15,
                            "type": "MATERIAL",
                            "iconId": "MTL_SKILL3",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SKILL3.png"
                        },
                        {
                            "id": "30135",
                            "count": 6,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_DS",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_DS.png"
                        },
                        {
                            "id": "30104",
                            "count": 5,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_RMA7024",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_RMA7024.png"
                        }
                    ]
                }
            ],
            "static": {
                "skillId": "skchr_mlynar_2",
                "iconId": null,
                "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_2.png",
                "levels": [
                    {
                        "name": "Unresolved Sorrow"
                    }
                ]
            }
        },
        {
            "skillId": "skchr_mlynar_3",
            "levelUpCostCond": [
                {
                    "unlockCond": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "levelUpCost": [
                        {
                            "id": "3303",
                            "count": 8,
                            "type": "MATERIAL",
                            "iconId": "MTL_SKILL3",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SKILL3.png"
                        },
                        {
                            "id": "31054",
                            "count": 4,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_PLCF",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_PLCF.png"
                        },
                        {
                            "id": "30063",
                            "count": 5,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_BOSS3",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_BOSS3.png"
                        }
                    ]
                },
                {
                    "unlockCond": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "levelUpCost": [
                        {
                            "id": "3303",
                            "count": 12,
                            "type": "MATERIAL",
                            "iconId": "MTL_SKILL3",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SKILL3.png"
                        },
                        {
                            "id": "30084",
                            "count": 4,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_MANGANESE2",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_MANGANESE2.png"
                        },
                        {
                            "id": "31044",
                            "count": 8,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_RS",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_RS.png"
                        }
                    ]
                },
                {
                    "unlockCond": {
                        "phase": "PHASE_2",
                        "level": 1
                    },
                    "levelUpCost": [
                        {
                            "id": "3303",
                            "count": 15,
                            "type": "MATERIAL",
                            "iconId": "MTL_SKILL3",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SKILL3.png"
                        },
                        {
                            "id": "30145",
                            "count": 6,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_OEU",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_OEU.png"
                        },
                        {
                            "id": "31034",
                            "count": 4,
                            "type": "MATERIAL",
                            "iconId": "MTL_SL_OC4",
                            "image": "/textures/arts/ui_item_icons_5/MTL_SL_OC4.png"
                        }
                    ]
                }
            ],
            "static": {
                "skillId": "skchr_mlynar_3",
                "iconId": null,
                "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_3.png",
                "levels": [
                    {
                        "name": "Unbrilliant Glory"
                    }
                ]
            }
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
            "data": null
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
            "data": null
        }
    ]
};

const LANCET = {
    "id": "char_285_medic2",
    "name": "Lancet-2",
    "description": "Restores the HP of allies and ignores the <@ba.kw>Deployment Limit</>, but has a long Redeployment Time",
    "rarity": "TIER_1",
    "profession": "MEDIC",
    "subProfessionId": "physician",
    "position": "RANGED",
    "tagList": [
        "Robot",
        "Healing"
    ],
    "nationId": "rhodes",
    "groupId": null,
    "teamId": null,
    "artists": [
        "过失帝国",
        "TOKI"
    ],
    "portrait": "/portraits/char_285_medic2_1.png",
    "skin": "/textures/chararts/char_285_medic2/char_285_medic2_1.png",
    "server": "en",
    "isNotObtainable": false,
    "phases": [
        {
            "rangeId": "3-1",
            "maxLevel": 30,
            "attributesKeyFrames": [
                {
                    "level": 1,
                    "data": {
                        "maxHp": 261,
                        "atk": 42,
                        "def": 16,
                        "magicResistance": 0,
                        "cost": 3,
                        "blockCnt": 1,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 2.8499999046325684,
                        "respawnTime": 200
                    }
                },
                {
                    "level": 30,
                    "data": {
                        "maxHp": 435,
                        "atk": 70,
                        "def": 27,
                        "magicResistance": 0,
                        "cost": 3,
                        "blockCnt": 1,
                        "moveSpeed": 1,
                        "attackSpeed": 100,
                        "baseAttackTime": 2.8499999046325684,
                        "respawnTime": 200
                    }
                }
            ],
            "evolveCost": [],
            "levelUpCost": [
                {
                    "id": "4001",
                    "count": 6043,
                    "type": "GOLD",
                    "iconId": "GOLD",
                    "image": "/textures/arts/items/item_icons_no_tiny_hub/GOLD.png"
                },
                {
                    "id": "5001",
                    "count": 9800,
                    "type": "EXP_PLAYER",
                    "iconId": "EXP_PLAYER",
                    "image": "/textures/arts/ui_item_icons_4/EXP_PLAYER.png"
                }
            ]
        }
    ],
    "allSkillLevelUp": [],
    "skills": [],
    "modules": []
};

export const FullUpgradePath = () => <LevelUpContent operator={MLYNAR} />;

export const RobotOperator = () => <LevelUpContent operator={LANCET} />;
