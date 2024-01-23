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
    avatar: {
        type?: string;
        id?: string;
    };
    assistCharList: [null]; // TODO: Figure out what this is
    lastOnlineTime: number;
    medalBoard: {
        type: string;
        custom: null; // TODO: Figure out what this is
        template: null; // TODO: Figure out what this is
    };
};

export type GetPlayerResponse = {
    players: GetPlayerData[];
    friendStatusList: number[];
    resultIdList: string[];
    playerDataDelta: {
        modified: unknown; // TODO: Figure out what this is
        deleted: unknown; // TODO: Figure out what this is
    };
};

export type PlayerData = {
    dungeon: {
        stages: Record<
            string,
            {
                stageId: string;
                completeTimes: number;
                startTimes: number;
                practiceTimes: number;
                state: number;
                hasBattleReplay: number;
                noCostCnt: number;
            }
        >;
        cowLevel: Record<
            string,
            {
                id: string;
                type: string; // TODO: This could be an enum?
                val: string[]; // TODO: Unsure whether this is an array of strings or not
                fts: number;
                rts: number;
            }
        >;
    };
    // TODO: This is very incomplete. Need to refer to
    // a more advanced player's data to see what else
    activity: Record<string, unknown>;
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
        avatar: {
            type?: string;
            id?: string;
        };
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
        squads: Record<
            string,
            {
                squadId: string;
                name: string;
                // TODO: Figure out what currentEquip is. Maybe its module.
                slots: ({ charInstId: number; skillIndex: number; currentEquip: null } | null)[];
            }
        >;
        chars: Record<
            string,
            {
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
                    static: {
                        name: string;
                        description: string;
                        duration: number;
                    };
                }[];
                voiceLan: string; // Could be fixed string like JP | CN | EN | KR | TW etc.
                currentEquip: null; // TODO: Figure out what currentEquip is lol. Maybe its module
                equip: Record<
                    string,
                    {
                        hide: number;
                        locked: number;
                        level: number;
                    }
                >;
                static: {
                    name: string;
                    description: string;
                    nationId: string;
                    appellation: string;
                    position: string;
                    profession: string;
                    subProfessionId: string;
                };
            }
        >;
        charGroup: Record<
            string,
            {
                favorPoint: number;
            }
        >;
        charMission: Record<string, Record<string, number>>;
    };
    npcAudio: Record<
        string,
        {
            npcShowAudioInfoFlag: string; // Could be fixed string like JP | CN | EN | KR | TW etc.
        }
    >;
    recruit: {
        normal: {
            slots: Record<
                string,
                {
                    state: number;
                    tags: number[];
                    selectTags: { tagId: number; pick: number }[];
                    startTs: number;
                    durationInSec: number;
                    maxFinishTs: number;
                    realFinishTs: number;
                }
            >;
        };
    };
    pushFlags: {
        hasGifts: number;
        hasFriendRequest: number;
        hasClues: number;
        hasFreeLevelGP: number;
        status: number;
    };
    equipment: {
        missions: unknown;
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
            info: { id: string; count: number }[];
        };
        HS: {
            curShopId: string;
            info: { id: string; count: number }[];
            // TODO: Incomplete
            progressInfo: unknown;
        };
        ES: {
            curShopId: string;
            info: { id: string; count: number }[];
        };
        CASH: {
            info: { id: string; count: number }[];
        };
        GP: {
            oneTime: {
                info: { id: string; count: number }[];
            };
            level: {
                info: { id: string; count: number }[];
            };
            weekly: {
                curGroupId: string;
                info: { id: string; count: number }[];
            };
            monthly: {
                curGroupId: string;
                info: { id: string; count: number }[];
            };
            choose: {
                info: { id: string; count: number }[];
            };
            backflow: {
                info: { id: string; count: number }[];
            };
        };
        FURNI: {
            info: { id: string; count: number }[];
        };
        SOCIAL: {
            curShopId: string;
            info: { id: string; count: number }[];
        };
        CLASSIC: {
            info: { id: string; count: number }[];
            progressInfo: unknown;
        };
    };
    mission: {
        missions: {
            OPENSERVER: Record<
                string,
                {
                    state: number;
                    progress: { target: number; value: number }[];
                }
            >;
            DAILY: Record<
                string,
                {
                    state: number;
                    progress: { target: number; value: number }[];
                }
            >;
            WEEKLY: Record<
                string,
                {
                    state: number;
                    progress: { target: number; value: number }[];
                }
            >;
            GUIDE: Record<
                string,
                {
                    state: number;
                    progress: { target: number; value: number }[];
                }
            >;
            MAIN: Record<
                string,
                {
                    state: number;
                    progress: { target: number; value: number }[];
                }
            >;
            ACTIVITY: Record<
                string,
                {
                    state: number;
                    progress: { target: number; value: number }[];
                }
            >;
            SUB: Record<
                string,
                {
                    state: number;
                    progress: { target: number; value: number }[];
                }
            >;
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
        assistCharList:
            | {
                  charInstId: number;
                  skillIndex: number;
                  currentEquip: null; // TODO: Figure out what this is
              }[]
            | null[]; // TODO: Figure out what this is
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
                bonus: unknown; // TODO: Figure out what this is
            };
        };
        chars: Record<
            string,
            {
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
            }
        >;
        roomSlots: Record<
            string,
            {
                level: number;
                state: number;
                roomId: string;
                charInstIds: number[];
                completeConstructTime: number;
            }
        >;
        rooms: {
            CONTROL: Record<
                string,
                {
                    buff: {
                        global: {
                            apCost: number;
                            roomCnt: unknown; // TODO: Figure out what this is
                        };
                        manufacture: {
                            speed: number;
                            sSpeed: number;
                            roomSpeed: unknown; // TODO: Figure out what this is
                            apCost: number;
                        };
                        trading: {
                            speed: number;
                            sSpeed: number;
                            roomSpeed: unknown; // TODO: Figure out what this is
                            charSpeed: unknown; // TODO: Figure out what this is
                            charLimit: unknown; // TODO: Figure out what this is
                            apCost: number;
                        };
                        meeting: {
                            clue: number;
                            speedUp: number;
                            weight: unknown; // TODO: Figure out what this is
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
                        apCost: unknown; // TODO: Figure out what this is
                        point: unknown; // TODO: Figure out what this is
                    };
                    apCost: number;
                    lastUpdateTime: number;
                }
            >;
            ELEVATOR: Record<string, unknown>;
            POWER: Record<
                string,
                {
                    buff: {
                        global: {
                            roomCnt: unknown; // TODO: Figure out what this is
                        };
                        manufacture: {
                            charSpeed: unknown; // TODO: Figure out what this is
                        };
                        laborSpeed: number;
                        apCost: {
                            self: unknown; // TODO: Figure out what this is
                        };
                    };
                }
            >;
            MANUFACTURE: Record<
                string,
                {
                    buff: {
                        apCost: {
                            self: unknown; // TODO: Figure out what this is
                        };
                        speed: number;
                        capacity: number;
                        sSpeed: number;
                        tSpeed: unknown; // TODO: Figure out what this is
                        cSpeed: number;
                        capFrom: unknown; // TODO: Figure out what this is
                        maxSpeed: number;
                        point: unknown; // TODO: Figure out what this is
                        flag: unknown; // TODO: Figure out what this is
                        skillExtend: unknown; // TODO: Figure out what this is
                    };
                    state: number;
                    formulaId: string;
                    remainSolutionCnt: number;
                    outputSolutionCnt: number;
                    lastUpdateTime: number;
                    saveTime: number;
                    tailTime: number;
                    apCost: number;
                    completeWorkTime: number;
                    capacity: number;
                    processPoint: number;
                    display: {
                        base: number;
                        buff: number;
                    };
                }
            >;
            TRADING: Record<
                string,
                {
                    buff: {
                        speed: number;
                        limit: number;
                        apCost: {
                            all: number;
                            single: unknown; // TODO: Figure out what this is
                            self: unknown; // TODO: Figure out what this is
                        };
                        rate: unknown; // TODO: Figure out what this is
                        tgw: []; // TODO: Figure out what this is
                        point: unknown; // TODO: Figure out what this is
                        manuLines: unknown; // TODO: Figure out what this is
                        orderWtBuff: []; // TODO: Figure out what this is
                        orderBuff: []; // TODO: Figure out what this is
                        violatedInfo: {
                            orderChecker: []; // TODO: Figure out what this is
                            cntBuff: []; // TODO: Figure out what this is
                        };
                    };
                    state: number;
                    lastUpdateTime: number;
                    strategy: string; // Might be an enum such as O_GOLD. Unsure what Originium would be
                    stockLimit: number;
                    apCost: number;
                    stock: {
                        instId: number;
                        // "type" might also be an enum such as MATERIAL
                        delivery: { type: string; id: string; count: number }[];
                        gain: {
                            type: string; // Again, might be an enum such as GOLD
                            id: string;
                            count: number;
                        };
                        type: string; // same type as the one in delivery I think
                        buff: []; // TODO: Figure out what this is
                    }[];
                    next: {
                        order: number;
                        processPoint: number;
                        maxPoint: number;
                        speed: number;
                    };
                    completeWorkTime: number;
                    display: {
                        base: number;
                        buff: number;
                    };
                }
            >;
            DORMITORY: Record<
                string,
                {
                    buff: {
                        apCost: {
                            all: number;
                            single: {
                                target: null; // TODO: Figure out what this is
                                value: number;
                            };
                            self: unknown; // TODO: Figure out what this is
                            exclude: unknown; // TODO: Figure out what this is
                        };
                        point: unknown; // TODO: Figure out what this is
                    };
                    comfort: number;
                    diySolution: {
                        wallPaper: null; // TODO: Figure out what this is
                        floor: null; // TODO: Figure out what this is
                        carpet: {
                            id: string;
                            coordinate: {
                                x: number;
                                y: number;
                            };
                        }[];
                        other: {
                            id: string;
                            coordinate: {
                                x: number;
                                y: number;
                            };
                        }[];
                    };
                }
            >;
            CORRIDOR: Record<string, unknown>; // TODO: Figure out what this is
            WORKSHOP: Record<
                string,
                {
                    buff: {
                        rate: {
                            all: number;
                        };
                        cost: {
                            type: string;
                            limit: number;
                            reduction: number;
                        };
                        costRe: {
                            type: string;
                            from: number;
                            change: number;
                        };
                        frate: []; // TODO: Figure out what this is
                        recovery: {
                            type: string;
                            pace: number;
                            recover: number;
                        };
                        goldFree: unknown; // TODO: Figure out what this is
                        costForce: {
                            type: string;
                            cost: number;
                        };
                        fFix: {
                            asRarity: unknown; // TODO: Figure out what this is
                        };
                        activeBonus: unknown; // TODO: Figure out what this is
                        apRate: unknown; // TODO: Figure out what this is
                        costDevide: {
                            type: string;
                            limit: number;
                            denominator: number;
                        };
                    };
                    statistic: {
                        noAddition: number;
                    };
                }
            >;
        };
        furniture: Record<
            string,
            {
                count: number;
                inUse: number;
            }
        >;
        assist: number[];
        diyPresentSolutions: unknown; // TODO: Figure out what this is
        solution: {
            furnitureTs: Record<string, number>;
        };
    };
    dexNav: {
        character: Record<
            string,
            {
                charInstId: number;
                count: number;
                classicCount: number;
            }
        >;
        formula: {
            shop: Record<string, number>; // TODO: Figure out what this is
            manufacture: Record<string, number>;
            workshop: Record<string, number>;
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
            stage: Record<string, number>;
        };
    };
    crisis: {
        current: string;
        lst: number;
        nst: number;
        map: Record<
            string,
            {
                rank: number;
                confirmed: number;
            }
        >;
        shop: {
            coin: number;
            info: []; // TODO: Figure out what this is
            progressInfo: unknown; // TODO: Figure out what this is
        };
        training: {
            currentStage: string[];
            stage: Record<
                string,
                {
                    point: number;
                }
            >;
            nst: number;
        };
        season: unknown; // TODO: Figure out what this is
        box: []; // TODO: Figure out what this is
    };
    tshop: Record<
        string,
        {
            coin: number;
            info: []; // TODO: Figure out what this is
            progressInfo: unknown; // TODO: Figure out what this is
        }
    >;
    gacha: {
        newbee: {
            openFlag: number;
            cnt: number;
            poolId: string;
        };
        normal: Record<
            string,
            {
                cnt: number;
                maxCnt: number;
                rarity: number;
                avail: boolean;
            }
        >;
        attain: unknown; // TODO: Figure out what this is
        limit: Record<
            string,
            {
                leastFree: number;
            }
        >;
        single: Record<
            string,
            {
                singleEnsureCnt: number;
                singleEnsureUse: boolean;
                singleEnsureChar: string;
            }
        >;
        fesClassic: unknown; // TODO: Figure out what this is
    };
    // finished at campaignsV2
};
