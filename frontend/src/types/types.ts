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

export type SearchResponse = {
    nickName: string;
    nickNumber: string;
    uid: string;
    registerTs: number;
    mainStageProgress: string;
    charCnt: number;
    furnCnt: number;
    secretary: string;
    secretarySkinId: string;
    resume: string;
    teamV2: Record<string, number>;
    friendNumLimit: number;
    serverName: string;
    level: number;
    avatarId: string;
    avatar: {
        type: string;
        id: string;
    };
    assistCharList: {
        charId: string;
        skinId: string;
        skills: {
            skillId: string;
            unlock: number;
            state: number;
            specializeLevel: number;
            completeUpgradeTime: number;
        }[];
        mainSkillLvl: number;
        skillIndex: number;
        evolvePhase: number;
        favorPoint: number;
        potentialRank: number;
        level: number;
        crisisRecord: unknown;
        currentEquip: null;
        equip: unknown;
    }[];
    lastOnlineTime: number;
    board: string[];
    infoShare: number;
    medalBoard: {
        type: string;
        custom: null;
        template: null;
    };
    recentVisited: number;
};

export type PlayerData = {
    /**
     * @description: Main stage progress
     */
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

    /**
     * @description General information such as the display name, experience, etc.
     */
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

    /**
     * @description: Currently owned operators data
     */
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
        chars: Record<string, CharacterData>;
        charGroup: Record<
            string,
            {
                favorPoint: number;
            }
        >;
        charMission: Record<string, Record<string, number>>;
    };

    /**
     * @description: For each operator's selected voiceover language.
     */
    npcAudio: Record<
        string,
        {
            npcShowAudioInfoFlag: string; // Could be fixed string like JP | CN | EN | KR | TW etc.
        }
    >;

    /**
     * @description: Recruitment data
     */
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

    /**
     * @description: Unsure what this is
     */
    pushFlags: {
        hasGifts: number;
        hasFriendRequest: number;
        hasClues: number;
        hasFreeLevelGP: number;
        status: number;
    };

    // TODO: This is very incomplete. Need to refer
    /**
     * @description: Module data. Needs to be filled in (incomplete)
     */
    equipment: {
        missions: unknown;
    };

    // TODO: This is very incomplete. Need to refer
    skin: {
        characterSkins: number;
        skinTs: number;
    };

    /**
     * @description: Shop data (what has been bought, amount bought, etc.).
     */
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

    /**
     * @description: Pinboard missions, daily missions, weekly missions, etc.
     */
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

    /**
     * @description: Friends, support slots, clue visits, etc.
     */
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

    /**
     * @description: Base data containing lots, and lots of information. I hated writing the types for this.
     */
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

    /**
     * @description: Unsure but may just be backend/developer data.
     */
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

    /**
     * @description: Unsure what this is, but I think it's CC stages?
     */
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

    /**
     * @description: Unsure what this is
     */
    tshop: Record<
        string,
        {
            coin: number;
            info: []; // TODO: Figure out what this is
            progressInfo: unknown; // TODO: Figure out what this is
        }
    >;

    /**
     * @description: Gacha pulls data
     */
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
    backflow: {
        open: boolean;
        current: null; // TODO: Figure out what this is
        currentV2: null; // TODO: Figure out what this is
    };
    mainline: {
        record: unknown; // TODO: Figure out what this is
        cache: []; // TODO: Figure out what this is
        version: number;
        additionalMission: unknown; // TODO: Figure out what this is
    };
    avatar: Record<
        string,
        {
            ts: number;
            src: string;
        }
    >;
    background: {
        selected: string;
        bgs:
            | Record<
                  string,
                  {
                      unlock: number;
                  }
              >
            | Record<
                  string,
                  {
                      conditions: Record<
                          string,
                          {
                              v: number;
                              t: number;
                          }
                      >;
                  }
              >; // Very strange to be honest.
    };

    /**
     * @description: I think this is data for IS/Pinup stages
     */
    rlv2: {
        current: {
            player: {
                state: string;
                property: {
                    exp: number;
                    level: number;
                    hp: {
                        current: number;
                        max: number;
                    };
                    gold: number;
                    shield: number;
                    capacity: number;
                    population: {
                        cost: number;
                        max: number;
                    };
                    conPerfectBattle: number;
                };
                cursor: {
                    zone: number;
                    position: null; // TODO: Figure out what this is
                };
                trace: []; // TODO: Figure out what this is
                pending: []; // TODO: Figure out what this is
                status: {
                    bankPut: number;
                };
                toEnding: string;
                chgEnding: boolean;
            };
            map: {
                zones: unknown; // TODO: Figure out what this is
            };
            troop: {
                chars: unknown; // TODO: Figure out what this is
                expedition: []; // TODO: Figure out what this is
                expeditionReturn: null; // TODO: Figure out what this is
                hasExpeditionReturn: boolean;
            };
            inventory: {
                relic: unknown; // TODO: Figure out what this is
                recruit: unknown; // TODO: Figure out what this is
                trap: null; // TODO: Figure out what this is
                consumable: unknown; // TODO: Figure out what this is
            };
            game: {
                mode: string;
                predefined: null; // TODO: Figure out what this is
                theme: string;
                outer: {
                    support: boolean;
                };
                start: number;
                modeGrade: number;
                equivalentGrade: number;
            };
            buff: {
                tmpHP: number;
                capsule: null; // TODO: Figure out what this is
                squadBuff: []; // TODO: Figure out what this is
            };
            record: {
                brief: null; // TODO: Figure out what this is
            };
            module: []; // TODO: Figure out what this is
        };
        outer: Record<
            string,
            {
                bank: {
                    show: boolean;
                    current: number;
                    record: number;
                    reward: unknown; // TODO: Figure out what this is
                    totalPut: number;
                };
                bp: {
                    point: number;
                    reward: unknown; // TODO: Figure out what this is
                };
                buff: {
                    pointOwned: number;
                    pointCost: number;
                    unlocked: unknown; // TODO: Figure out what this is
                    score: number;
                };
                collect: {
                    relic: Record<
                        string,
                        {
                            state: number;
                            progress: number[];
                        }
                    >;
                    capsule: Record<
                        string,
                        {
                            state: number;
                            progress: number[];
                        }
                    >;
                    activeTool: Record<
                        string,
                        {
                            state: number;
                            progress: number[];
                        }
                    >;
                    bgm: Record<string, number>;
                    pic: Record<string, number>;
                    chat: Record<string, number>;
                    band: Record<
                        string,
                        {
                            state: number;
                            progress: number[];
                        }
                    >;
                    buff: Record<
                        string,
                        {
                            state: number;
                            progress: number[];
                        }
                    >;
                    endBook: unknown; // TODO: Figure out what this is
                    // NOTE: This could be fixed since there's just EASY, NORMAL, HARD, MONTH_TEAM, and CHALLENGE that I see
                    mode: Record<
                        string,
                        Record<
                            string,
                            {
                                state: number;
                                progress: number[];
                            }
                        >
                    >;
                    recruitSet: Record<
                        string,
                        {
                            state: number;
                            progress: number[];
                        }
                    >;
                    // NOTE: This could be fixed since there's just EASY, NORMAL, HARD, MONTH_TEAM, and CHALLENGE that I see
                    modeGrade: Record<
                        string,
                        Record<
                            string,
                            {
                                state: number;
                                progress: number[];
                            }
                        >
                    >;
                };
                mission: {
                    updateId: string;
                    refresh: number;
                    list: []; // TODO: Figure out what this is
                };
                challenge: {
                    reward: unknown; // TODO: Figure out what this is
                    grade: unknown; // TODO: Figure out what this is
                };
                monthTeam: {
                    valid: string[];
                    reward: unknown; // TODO: Figure out what this is
                    mission: unknown; // TODO: Figure out what this is
                };
                record: {
                    last: number;
                    modeCnt: unknown; // TODO: Figure out what this is
                    endingCnt: unknown; // TODO: Figure out what this is
                    stageCnt: unknown; // TODO: Figure out what this is
                    bandCnt: unknown; // TODO: Figure out what this is
                };
            }
        >;
    };

    /**
     * @description: Maybe this is IS3 I'm not sure
     */
    deepSea: {
        places: Record<string, number>;
        nodes: Record<string, number>;
        choices: Record<string, number[]>;
        events: unknown; // TODO: Figure out what this is
        treasures: unknown; // TODO: Figure out what this is
        stories: unknown; // TODO: Figure out what this is
        techTrees: unknown; // TODO: Figure out what this is
        logs: unknown; // TODO: Figure out what this is
    };

    /**
     * @description: Maybe also IS/Pinup?
     */
    tower: {
        current: {
            status: {
                state: string;
                tower: string;
                coord: number;
                tactical: Record<string, string>;
                strategy: string;
                start: number;
                isHard: boolean;
            };
            layer: []; // TODO: Figure out what this is
            cards: []; // TODO: Figure out what this is
            godCard: {
                id: string;
                subGodCardId: string;
            };
            halftime: {
                count: number;
                candidate: []; // TODO: Figure out what this is
                canGiveUp: boolean;
            };
            trap: []; // TODO: Figure out what this is
            reward: {
                high: number;
                low: number;
            };
        };
        outer: {
            training: unknown; // TODO: Figure out what this is
            towers: unknown; // TODO: Figure out what this is
            hasTowerPass: number;
            pickedGodCard: unknown; // TODO: Figure out what this is
            tactical: Record<string, string>;
            strategy: string;
        };
        season: {
            id: string;
            finishTs: number;
            missions: Record<
                string,
                {
                    value: number;
                    target: number;
                    hasRecv: boolean;
                }
            >;
            passWithGodCard: unknown; // TODO: Figure out what this is
            slots: unknown; // TODO: Figure out what this is
            period: {
                termTs: number;
                items: unknown; // TODO: Figure out what this is
                cur: number;
                len: number;
            };
        };
    };

    /**
     * @description: I have no idea what this is. Maybe it's Lingering Echoes?
     */
    siracusaMap: {
        select: null; // TODO: Figure out what this is
        card: unknown; // TODO: Figure out what this is
        opera: {
            total: number;
            show: null; // TODO: Figure out what this is
            release: Record<string, number>;
            like: unknown; // TODO: Figure out what this is
        };
        area: {
            area_centro: number;
        };
    };

    /**
     * @description: It's that playback item I still don't have lol
     */
    storyreview: {
        groups: Record<
            string,
            {
                rts: number;
                stories: []; // TODO: Figure out what this is
            }
        >;
        tags: unknown; // TODO: Figure out what this is
    };

    /**
     * @description: Obtained/unobtained medals.
     */
    medal: {
        medals: Record<
            string,
            {
                id: string;
                val: number[];
                fts: number;
                rts: number;
            }
        >;
        custom: {
            currentIndex: null; // TODO: Figure out what this is
            customs: unknown; // TODO: Figure out what this is
        };
    };

    /**
     * @description: April Fools stages
     */
    aprilFool: Record<
        string,
        {
            stages: unknown;
            liveEndings?: unknown;
            cameraLv?: number;
            fans?: number;
            posts?: number;
            missions?: Record<
                string,
                {
                    value: number;
                    target: number;
                    finished: boolean;
                    hasRecv: boolean;
                }
            >;
        }
    >;

    /**
     * @description: Intermezzi and Side Story data
     */
    retro: {
        coin: number;
        supplement: number;
        block: Record<
            string,
            {
                locked: number;
                open: number;
            }
        >;
        trail: Record<string, Record<string, number>>;
        lst: number;
        nst: number;
        rewardPerm: []; // TODO: Figure out what this is
    };

    // TODO: This is very incomplete. Need to refer
    /**
     * @description: Unsure what this is. Buffs maybe?
     */
    charm: {
        charms: Record<string, number>;
        squad: []; // TODO: Figure out what this is
    };

    // TODO: This is very incomplete. Need to refer
    /**
     * @description: Furniture data...?
     */
    carousel: {
        furnitureShop: {
            goods: unknown; // TODO: Figure out what this is
            groups: unknown; // TODO: Figure out what this is
        };
    };

    /**
     * @description: I believe this is related to consumable items like supply boxes and stuff
     */
    consumable: Record<
        string,
        Record<
            string,
            {
                ts: number;
                count: number;
            }
        >
    >;

    /**
     * @description: Lol what is this
     */
    event: {
        building: number;
    };

    /**
     * @description: I also have no idea what this is
     */
    collectionReward: {
        team: unknown; // TODO: Figure out what this is
    };

    /**
     * @description: Daily checkins (I think also for the monthly package checkin)
     */
    checkIn: {
        canCheckIn: number; // I think this can be 0 or 1 which is basically false / true
        checkInGroupId: string;
        checkInRewardIndex: number;
        checkInHistory: number[];
        newbiePackage: {
            open: boolean;
            groupId: string;
            finish: number;
            stopSale: number;
            checkInHistory: number[];
        };
    };

    /**
     * @description: I have no idea what this is
     */
    car: {
        battleCar: Record<string, null>; // TODO: Figure out what this is
        exhibitionCar: Record<string, null>; // TODO: Figure out what this is
        accessories: Record<string, null>; // TODO: Figure out what this is
    };

    /**
     * @description: Possibly related to checkins?
     */
    openServer: {
        checkIn: {
            isAvailable: boolean;
            history: number[];
        };
        chainLogin: {
            isAvailable: boolean;
            nowIndex: number;
            history: number[];
            finalReward: number;
        };
    };

    /**
     * @description: Unsure about this, but it looks cool
     */
    campaignsV2: {
        campaignCurrentFee: number;
        campaignTotalFee: number;
        open: {
            permanent: string[];
            rotate: string;
            rGroup: string;
            training: string[];
            tGroup: string;
            tAllOpen: null; // TODO: Figure out what this is
        };
        instances: Record<
            string,
            {
                maxKills: number;
                rewardStatus: number[];
            }
        >;
        missions: Record<string, number>;
        lastRefreshTs: number;
        sweepMaxKills: unknown; // TODO: Figure out what this is
    };

    /**
     * @description: The item ID and then the amount of that item.
     */
    inventory: Record<string, {
        itemId: string;
        name: string;
        description: string;
        rarity: string; // Could be an enum
        iconId: string;
        overrideBkg: null; // TODO: Figure out what this is
        stackIconId: string;
        sortId: number;
        usage: string;
        obtainApproach: null; // TODO: Figure out what this is
        hideInItemGet: boolean;
        classifyType: string; // Could be an enum
        itemType: string;
        stageDropList: { stageId: string; occPer: string; }[];
        buildingProductList: { roomType: string; formulaId: string }[];
        amount: number;
    }>;

    /**
     * @description: Unsure about this
     */
    limitedBuff: {
        dailyUsage: unknown; // TODO: Figure out what this is
        inventory: unknown; // TODO: Figure out what this is
    };

    /**
     * @description: Again unsure about this.
     */
    ticket: unknown; // TODO: Figure out what this is
};

export type CharacterData = {
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
            levels: {
                name: string;
                rangeId: string | null;
                description: string;
                skillType: number;
                durationType: number;
                spData: {
                    spType: number;
                    levelUpCost: [];
                    maxChargeTime: number;
                    spCost: number;
                    initSp: number;
                    increment: number;
                };
                prefabId: string;
                duration: number;
                blackboard: { key: string; value: number; valueStr: string | null }[];
            }[];
            skillId: string;
            iconId: string | null;
            hidden: boolean;
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
        trust: number;
    };
};
