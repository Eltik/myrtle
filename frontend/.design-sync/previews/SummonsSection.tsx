import { SummonsSection } from "frontend";

// The collapsible summon panel on the Information tab (InfoContent.tsx renders it
// from `operator.drones`). Stats are interpolated from the parent's promotion and
// level, so the two stories also sweep that: E2 Lv.90 vs E0 Lv.30.
//
// Fixture is Ling's three Soul tokens from `/api/operators/char_2023_ling`. The
// attack-range grid needs `/static/ranges`, which no preview can fetch, so it is
// absent here - everything else is the real render.
const LING_TOKENS = [
    {
        "id": "token_10020_ling_soul1",
        "name": "'Tranquility'",
        "description": "Blocks 1 enemy",
        "position": "MELEE",
        "phases": [
            {
                "rangeId": "1-1",
                "maxLevel": 50,
                "attributesKeyFrames": [
                    {
                        "level": 1,
                        "data": {
                            "maxHp": 1665,
                            "atk": 366,
                            "def": 233,
                            "magicResistance": 0,
                            "cost": 12,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.25,
                            "respawnTime": 10
                        }
                    },
                    {
                        "level": 50,
                        "data": {
                            "maxHp": 1936,
                            "atk": 425,
                            "def": 271,
                            "magicResistance": 0,
                            "cost": 12,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.25,
                            "respawnTime": 10
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
                            "maxHp": 1936,
                            "atk": 425,
                            "def": 271,
                            "magicResistance": 0,
                            "cost": 12,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.25,
                            "respawnTime": 10
                        }
                    },
                    {
                        "level": 80,
                        "data": {
                            "maxHp": 2225,
                            "atk": 489,
                            "def": 312,
                            "magicResistance": 0,
                            "cost": 12,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.25,
                            "respawnTime": 10
                        }
                    }
                ]
            },
            {
                "rangeId": "1-1",
                "maxLevel": 90,
                "attributesKeyFrames": [
                    {
                        "level": 1,
                        "data": {
                            "maxHp": 2225,
                            "atk": 489,
                            "def": 312,
                            "magicResistance": 0,
                            "cost": 12,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.25,
                            "respawnTime": 10
                        }
                    },
                    {
                        "level": 90,
                        "data": {
                            "maxHp": 2500,
                            "atk": 549,
                            "def": 351,
                            "magicResistance": 0,
                            "cost": 12,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.25,
                            "respawnTime": 10
                        }
                    }
                ]
            }
        ],
        "talents": [
            {
                "candidates": [
                    {
                        "unlockCondition": {
                            "phase": "PHASE_0",
                            "level": 1
                        },
                        "requiredPotentialRank": 0,
                        "prefabKey": "1",
                        "name": null,
                        "description": "+3 Tokens",
                        "rangeId": null,
                        "blackboard": [
                            {
                                "key": "max_deck_stack_cnt",
                                "value": 3,
                                "valueStr": null
                            },
                            {
                                "key": "max_deploy_count",
                                "value": 2,
                                "valueStr": null
                            }
                        ]
                    },
                    {
                        "unlockCondition": {
                            "phase": "PHASE_1",
                            "level": 1
                        },
                        "requiredPotentialRank": 0,
                        "prefabKey": "1",
                        "name": null,
                        "description": "+4 Tokens",
                        "rangeId": null,
                        "blackboard": [
                            {
                                "key": "max_deck_stack_cnt",
                                "value": 4,
                                "valueStr": null
                            },
                            {
                                "key": "max_deploy_count",
                                "value": 2,
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
                        "prefabKey": "1",
                        "name": null,
                        "description": "+5 Tokens",
                        "rangeId": null,
                        "blackboard": [
                            {
                                "key": "max_deck_stack_cnt",
                                "value": 5,
                                "valueStr": null
                            },
                            {
                                "key": "max_deploy_count",
                                "value": 2,
                                "valueStr": null
                            }
                        ]
                    }
                ]
            }
        ]
    },
    {
        "id": "token_10020_ling_soul2",
        "name": "'Peripateticism'",
        "description": "Deals <@ba.kw>Arts damage</>",
        "position": "RANGED",
        "phases": [
            {
                "rangeId": "3-1",
                "maxLevel": 50,
                "attributesKeyFrames": [
                    {
                        "level": 1,
                        "data": {
                            "maxHp": 810,
                            "atk": 270,
                            "def": 83,
                            "magicResistance": 15,
                            "cost": 9,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.600000023841858,
                            "respawnTime": 10
                        }
                    },
                    {
                        "level": 50,
                        "data": {
                            "maxHp": 942,
                            "atk": 314,
                            "def": 96,
                            "magicResistance": 15,
                            "cost": 9,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.600000023841858,
                            "respawnTime": 10
                        }
                    }
                ]
            },
            {
                "rangeId": "3-1",
                "maxLevel": 80,
                "attributesKeyFrames": [
                    {
                        "level": 1,
                        "data": {
                            "maxHp": 942,
                            "atk": 314,
                            "def": 96,
                            "magicResistance": 20,
                            "cost": 9,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.600000023841858,
                            "respawnTime": 10
                        }
                    },
                    {
                        "level": 80,
                        "data": {
                            "maxHp": 1083,
                            "atk": 361,
                            "def": 110,
                            "magicResistance": 20,
                            "cost": 9,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.600000023841858,
                            "respawnTime": 10
                        }
                    }
                ]
            },
            {
                "rangeId": "3-1",
                "maxLevel": 90,
                "attributesKeyFrames": [
                    {
                        "level": 1,
                        "data": {
                            "maxHp": 1083,
                            "atk": 361,
                            "def": 110,
                            "magicResistance": 20,
                            "cost": 9,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.600000023841858,
                            "respawnTime": 10
                        }
                    },
                    {
                        "level": 90,
                        "data": {
                            "maxHp": 1217,
                            "atk": 406,
                            "def": 124,
                            "magicResistance": 20,
                            "cost": 9,
                            "blockCnt": 1,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.600000023841858,
                            "respawnTime": 10
                        }
                    }
                ]
            }
        ],
        "talents": [
            {
                "candidates": [
                    {
                        "unlockCondition": {
                            "phase": "PHASE_0",
                            "level": 1
                        },
                        "requiredPotentialRank": 0,
                        "prefabKey": "1",
                        "name": null,
                        "description": "+3 Tokens",
                        "rangeId": null,
                        "blackboard": [
                            {
                                "key": "max_deck_stack_cnt",
                                "value": 3,
                                "valueStr": null
                            },
                            {
                                "key": "max_deploy_count",
                                "value": 2,
                                "valueStr": null
                            }
                        ]
                    },
                    {
                        "unlockCondition": {
                            "phase": "PHASE_1",
                            "level": 1
                        },
                        "requiredPotentialRank": 0,
                        "prefabKey": "1",
                        "name": null,
                        "description": "+4 Tokens",
                        "rangeId": null,
                        "blackboard": [
                            {
                                "key": "max_deck_stack_cnt",
                                "value": 4,
                                "valueStr": null
                            },
                            {
                                "key": "max_deploy_count",
                                "value": 2,
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
                        "prefabKey": "1",
                        "name": null,
                        "description": "+5 Tokens",
                        "rangeId": null,
                        "blackboard": [
                            {
                                "key": "max_deck_stack_cnt",
                                "value": 5,
                                "valueStr": null
                            },
                            {
                                "key": "max_deploy_count",
                                "value": 2,
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
                            "phase": "PHASE_1",
                            "level": 1
                        },
                        "requiredPotentialRank": 0,
                        "prefabKey": "2",
                        "name": null,
                        "description": null,
                        "rangeId": null,
                        "blackboard": []
                    }
                ]
            }
        ]
    },
    {
        "id": "token_10020_ling_soul3",
        "name": "'Thunderer'",
        "description": "Attacks <@ba.kw>all blocked enemies</>",
        "position": "MELEE",
        "phases": [
            {
                "rangeId": "1-1",
                "maxLevel": 50,
                "attributesKeyFrames": [
                    {
                        "level": 1,
                        "data": {
                            "maxHp": 2402,
                            "atk": 548,
                            "def": 281,
                            "magicResistance": 10,
                            "cost": 23,
                            "blockCnt": 2,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.5,
                            "respawnTime": 20
                        }
                    },
                    {
                        "level": 50,
                        "data": {
                            "maxHp": 2793,
                            "atk": 637,
                            "def": 327,
                            "magicResistance": 10,
                            "cost": 23,
                            "blockCnt": 2,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.5,
                            "respawnTime": 20
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
                            "maxHp": 2793,
                            "atk": 637,
                            "def": 327,
                            "magicResistance": 10,
                            "cost": 23,
                            "blockCnt": 2,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.5,
                            "respawnTime": 20
                        }
                    },
                    {
                        "level": 80,
                        "data": {
                            "maxHp": 3210,
                            "atk": 732,
                            "def": 376,
                            "magicResistance": 10,
                            "cost": 23,
                            "blockCnt": 2,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.5,
                            "respawnTime": 20
                        }
                    }
                ]
            },
            {
                "rangeId": "1-1",
                "maxLevel": 90,
                "attributesKeyFrames": [
                    {
                        "level": 1,
                        "data": {
                            "maxHp": 3210,
                            "atk": 732,
                            "def": 376,
                            "magicResistance": 10,
                            "cost": 23,
                            "blockCnt": 2,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.5,
                            "respawnTime": 20
                        }
                    },
                    {
                        "level": 90,
                        "data": {
                            "maxHp": 3607,
                            "atk": 823,
                            "def": 423,
                            "magicResistance": 10,
                            "cost": 23,
                            "blockCnt": 2,
                            "moveSpeed": 1,
                            "attackSpeed": 100,
                            "baseAttackTime": 1.5,
                            "respawnTime": 20
                        }
                    }
                ]
            }
        ],
        "talents": [
            {
                "candidates": [
                    {
                        "unlockCondition": {
                            "phase": "PHASE_0",
                            "level": 1
                        },
                        "requiredPotentialRank": 0,
                        "prefabKey": "1",
                        "name": null,
                        "description": "+3 Tokens",
                        "rangeId": null,
                        "blackboard": [
                            {
                                "key": "max_deck_stack_cnt",
                                "value": 3,
                                "valueStr": null
                            },
                            {
                                "key": "max_deploy_count",
                                "value": 2,
                                "valueStr": null
                            }
                        ]
                    },
                    {
                        "unlockCondition": {
                            "phase": "PHASE_1",
                            "level": 1
                        },
                        "requiredPotentialRank": 0,
                        "prefabKey": "1",
                        "name": null,
                        "description": "+4 Tokens",
                        "rangeId": null,
                        "blackboard": [
                            {
                                "key": "max_deck_stack_cnt",
                                "value": 4,
                                "valueStr": null
                            },
                            {
                                "key": "max_deploy_count",
                                "value": 2,
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
                        "prefabKey": "1",
                        "name": null,
                        "description": "+5 Tokens",
                        "rangeId": null,
                        "blackboard": [
                            {
                                "key": "max_deck_stack_cnt",
                                "value": 5,
                                "valueStr": null
                            },
                            {
                                "key": "max_deploy_count",
                                "value": 2,
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
                        "prefabKey": "2",
                        "name": null,
                        "description": null,
                        "rangeId": null,
                        "blackboard": []
                    }
                ]
            }
        ]
    }
];

export const ThreeTokens = () => <SummonsSection drones={LING_TOKENS} parentLevel={90} parentPhaseIndex={2} server="en" />;

export const EarlyPromotion = () => <SummonsSection drones={LING_TOKENS} parentLevel={30} parentPhaseIndex={0} server="en" />;

export const SingleToken = () => <SummonsSection drones={[LING_TOKENS[2]]} parentLevel={90} parentPhaseIndex={2} server="en" />;
