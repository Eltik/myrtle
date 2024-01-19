export type Server = "en" | "jp" | "kr" | "cn" | "bili" | "tw";

export type CodeResponse = {
    result: number;
};

export type LoginResponse = {
    channelUID: string;
    token: string;
    uid: string;
    secret: string;
    seqnum: number;
};

export type LoginData = LoginResponse & {
    email: string;
};

export type GetPlayerData = {
    nickName: string;
    nickNumber: string;
    uid: string;
    friendNumLimit: number;
    serverName: string;
    level: number;
    avatarId: string;
    avatar: {}; // TODO: Figure out what this is
    assistCharList: [null]; // TODO: Figure out what this is
    lastOnlineTime: number;
    medalBoard: {
        type: string;
        custom: null; // TODO: Figure out what this is
        template: null; // TODO: Figure out what this is
    }
}

export type GetPlayerResponse = {
    players: GetPlayerData[];
    friendStatusList: number[]
    resultIdList: string[];
    playerDataDelta: {
        modified: {}; // TODO: Figure out what this is
        deleted: {}; // TODO: Figure out what this is
    }
}

export type PlayerData = {
    dungeon: {
        stages: Record<string, {
                stageId: string;
                completeTimes: number;
                startTimes: number;
                practiceTimes: number;
                state: number;
                hasBattleReplay: number;
                noCostCnt: number;
            }>;
        cowLevel: Record<string, {
                id: string;
                type: string; // TODO: This could be an enum?
                val: string[]; // TODO: Unsure whether this is an array of strings or not
                fts: number;
                rts: number;
            }>;
    };
    // TODO: This is very incomplete. Need to refer to
    // a more advanced player's data to see what else
    activity: {
        TYPE_ACT9D0: {
            act18sre: {
                coin: 0,
                favorList: [],
                news: {}
            }
        };
        CHECKIN_VS: {}
        TYPE_ACT27SIDE: {
            act27side: {
                day: number;
                signedIn: boolean;
                stock: {};
                reward: null;
                milestone: {
                    point: number;
                    got: [];
                };
                coin: number;
                campaignCnt: number;
                state: number;
                sale: {
                    stateSell: number;
                    inquire: {
                        cur: number;
                        max: number;
                    };
                    groupId: null;
                    buyers: {};
                    purchasesTmp: {};
                    purchases: {};
                    sellsTmp: {};
                    sells: {};
                };
                favorList: string[]
            }
        };
        LOGIN_ONLY: {
            act16login: {
                reward: number;
            }
        };
        CHECKIN_ONLY: {
            act34sign: {
                lastTs: number; // Date number
                history: number[];
            }
        };
        PRAY_ONLY: {
            act6pray: {
                lastTs: number;
                extraCount: number;
                prayDaily: number;
                praying: number;
                insureCount: number;
                prayCount: number;
                prayMax: number;
                prayMaxIndex: number;
                prayArray: [];
            }
        };
    };
    status: {
        nickName: string;
        nickNumber: string;
        level: number;
        exp: number;
        socialPoint: number;
        gachaTicket: number;
        classicGachaTicket: number;
        tenGachaTicket: number;
        classicTenGachaTicket: number;
        instantFinishTicket: number;
        hggShard: number;
        lggShard: number;
        classicShard: number;
        recruitLicense: number;
        progress: number;
        buyApRemainTimes: number;
        apLimitUpFlag: number;
        uid: string;
        flags: {
            init: number;
            "obt/guide/l0-0/0_home_ui": number;
            "obt/guide/l0-0/1_recruit_adv": number;
            "obt/guide/l0-0/2_make_squad": number;
            "obt/guide/l0-0/3_battle_ui": number;
            "obt/main/level_main_00-01_beg": number;
            "obt/main/level_main_00-01_end": number;
            "obt/guide/l0-1/0_mission_main": number;
            "obt/guide/l0-1/1_training_level": number;
            "obt/guide/train/tr_01": number;
            "obt/main/level_main_00-02_beg": number;
            "obt/main/level_main_00-02_end": number;
            "obt/guide/l0-2/0_recruit_normal": number;
            "obt/guide/stage/stagepage_v2_intro": number;
            "obt/guide/train/tr_03": number;
            "obt/main/level_main_00-04_beg": number;
            "obt/guide/l0-4/0_upgrade_char": number;
        };
        ap: number;
        maxAp: number;
        payDiamond: number;
        freeDiamond: number;
        diamondShard: number;
        gold: number;
        practiceTicket: number;
        lastRefreshTs: number;
        lastApAddTime: number;
        mainStageProgress: string;
        registerTs: number;
        lastOnlineTs: number;
        serverName: string;
        avatarId: string;
        avatar: {}; // TODO: View more advanced player for actual type
        resume: string;
        friendNumLimit: number;
        monthlySubscriptionStartTime: number;
        monthlySubscriptionEndTime: number;
        tipMonthlyCardExpireTs: number;
        secretary: string;
        secretarySkinId: string;
    };
    troop: {
        curCharInstId: number;
        curSquadCount: number;
        squads: Record<string, {
                squadId: string;
                name: string;
                // TODO: Figure out what currentEquip is. Maybe its module.
                slots: ({ charInstId: number; skillIndex: number; currentEquip: null } | null)[];
            }>;
        chars: Record<string, {
                instId: number;
                charId: string;
                favorPoint: number;
                potentialRank: number;
                mainSkillLvl: number;
                skin: string;
                level: number;
                exp: number;
                evolvePhase: number;
                defaultSkillIndex: number;
                gainTime: number;
                skills: {
                    skillId: string;
                    unlock: number;
                    state: number;
                    specializeLevel: number;
                    completeUpgradeTime: number;
                }[];
                voiceLan: string; // Could be fixed string like JP | CN | EN | KR | TW etc.
                currentEquip: null; // TODO: Figure out what currentEquip is lol. Maybe its module
                equip: Record<string, {
                        hide: number;
                        locked: number;
                        level: number;
                    }>;
            }>;
        charGroup: Record<string, {
                favorPoint: number;
            }>;
        charMission: Record<string, Record<string, number>>;
    };
    npcAudio: Record<string, {
            npcShowAudioInfoFlag: string; // Could be fixed string like JP | CN | EN | KR | TW etc.
        }>;
    recruit: {
        normal: {
            slots: Record<string, {
                    state: number;
                    tags: number[];
                    selectTags: { tagId: number; pick: number }[];
                    startTs: number;
                    durationInSec: number;
                    maxFinishTs: number;
                    realFinishTs: number;
                }>
        }
    };
    pushFlags: {
        hasGifts: number;
        hasFriendRequest: number;
        hasClues: number;
        hasFreeLevelGP: number;
        status: number;
    };
    // TODO: This is very incomplete. Need to refer
    equipment: {
        missions: {};
    };
    // TODO: This is very incomplete. Need to refer
    skin: {
        characterSkins: number;
        skinTs: number;
    };
    shop: {
        LS: {
            curShopId: string;
            curGroupId: string;
            // TODO: Incomplete
            info: [];
        };
        HS: {
            curShopId: string;
            // TODO: Incomplete
            info: [];
            // TODO: Incomplete
            progressInfo: {};
        };
        ES: {
            curShopId: string;
            // TODO: Incomplete
            info: [];
        };
        CASH: {
            // TODO: Incomplete
            info: [];
        };
        GP: {
            oneTime: {
                // TODO: Incomplete
                info: [];
            };
            level: {
                // TODO: Incomplete
                info: [];
            };
            weekly: {
                curGroupId: string;
                // TODO: Incomplete
                info: [];
            };
            monthly: {
                curGroupId: string;
                // TODO: Incomplete
                info: [];
            };
            choose: {
                // TODO: Incomplete
                info: [];
            };
            backflow: {
                // TODO: Incomplete
                info: [];
            };
        };
        FURNI: {
            info: { id: string; count: number }[];
        };
        SOCIAL: {
            curShopId: string;
            // TODO: Incomplete
            info: [];
        };
        CLASSIC: {
            // TODO: Incomplete
            info: [];
            // TODO: Incomplete
            progressInfo: {};
        };
    };
    mission: {
        missions: {
            OPENSERVER: Record<string, {
                    state: number;
                    progress: { target: number; value: number }[];
                }>;
            DAILY: Record<string, {
                    state: number;
                    progress: { target: number; value: number }[];
                }>;
            WEEKLY: Record<string, {
                    state: number;
                    progress: { target: number; value: number }[];
                }>;
            GUIDE: Record<string, {
                    state: number;
                    progress: { target: number; value: number }[];
                }>;
            MAIN: Record<string, {
                    state: number;
                    progress: { target: number; value: number }[];
                }>;
            ACTIVITY: Record<string, {
                    state: number;
                    progress: { target: number; value: number }[];
                }>;
            SUB: Record<string, {
                    state: number;
                    progress: { target: number; value: number }[];
                }>;
        };
        missionRewards: {
            dailyPoint: number;
            weeklyPoint: number;
            rewards: {
                DAILY: Record<string, number>;
                WEEKLY: Record<string, number>;
            };
        };
        missionGroups: Record<string, number>;
    };
    social: {
        assitCharList: [null]; // TODO: Figure out what this is
        yesterdayReward: {
            canReceive: number;
            assistAmount: number;
            comfortAmount: number;
            first: number;
        };
        yCrisisSs: string;
    };
    building: {
        status: {
            labor: {
                buffSpeed: number;
                processPoint: number;
                value: number;
                lastUpdateTime: number;
                maxValue: number;
            };
            workshop: {
                bonusActive: number;
                bonus: {}; // TODO: Figure out what this is
            }
        };
        chars: Record<string, {
                charId: string;
                lastApAddTime: number;
                ap: number;
                roomSlotId: string;
                index: number;
                changeScale: number;
                bubble: {
                    normal: {
                        add: number;
                        ts: number;
                    };
                    assist: {
                        add: number;
                        ts: number;
                    };
                };
                workTime: number;
            }>;
        roomSlots: Record<string, {
                level: number;
                state: number;
                roomId: string;
                charInstIds: number[];
                completeConstructTime: number;
            }>;
        rooms: {
            CONTROL: Record<string, {
                    buff: {
                        global: {
                            apCost: number;
                            roomCnt: {}; // TODO: Figure out what this is
                        };
                        manufacture: {
                            speed: number;
                            sSpeed: number;
                            roomSpeed: {}; // TODO: Figure out what this is
                            apCost: number;
                        };
                        trading: {
                            speed: number;
                            sSpeed: number;
                            roomSpeed: {}; // TODO: Figure out what this is
                            charSpeed: {}; // TODO: Figure out what this is
                            charLimit: {}; // TODO: Figure out what this is
                            apCost: number;
                        };
                        meeting: {
                            clue: number;
                            speedUp: number;
                            weight: {}; // TODO: Figure out what this is
                            apCost: number;
                        };
                        hire: {
                            sUp: {
                                base: number;
                                up: number;
                            };
                            apCost: number;
                        };
                        power: {
                            apCost: number;
                        };
                        dormitory: {
                            recover: number;
                        };
                        apCost: {} // TODO: Figure out what this is
                        point: {} // TODO: Figure out what this is
                    };
                    apCost: number;
                    lastUpdateTime: number;
                }>;
            ELEVATOR: Record<string, {}>;
        };
        furniture: {}; // TODO: Figure out what this is
        assist: number[];
        diyPresentSolutions: {}; // TODO: Figure out what this is
        solution: {
            furnitureTs: {}; // TODO: Figure out what this is
        };
    };
    dexNav: {
        character: Record<string, {
            charInstId: number;
            count: number;
            classicCount: number;
        }>;
        formula: {
            shop: {}; // TODO: Figure out what this is
            manufacture: {}; // TODO: Figure out what this is
            workshop: {}; // TODO: Figure out what this is
        };
        teamV2: {
            rhodes: Record<string, number>;
            action4: Record<string, number>;
            sami: Record<string, number>;
            reserve1: Record<string, number>;
            reserve4: Record<string, number>;
            blacksteel: Record<string, number>;
            columbia: Record<string, number>;
            laterano: Record<string, number>;
            lungmen: Record<string, number>;
            leithanien: Record<string, number>;
            higashi: Record<string, number>;
        };
        enemy: {
            enemies: Record<string, number>;
            stage: {}; // TODO: Figure out what this is
        };
    };
    crisis: {
        current: string;
        lst: number;
        nst: number;
        map: Record<string, {
            rank: number;
            confirmed: number;
        }>;
        shop: {
            coin: number;
            info: []; // TODO: Figure out what this is
            progressInfo: {}; // TODO: Figure out what this is
        };
        training: {
            currentStage: string[];
            stage: Record<string, {
                point: number;
            }>;
            nst: number;
        };
        season: {}; // TODO: Figure out what this is
        box: []; // TODO: Figure out what this is
    };
    // ended at tshop
}
