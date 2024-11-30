export type SearchResponse = {
    players: {
        nickName: string;
        nickNumber: string;
        uid: string;
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
            crisisRecord: Record<any, any>; // Unsure
            crisisV2Record: Record<any, any>; // Unsure
            currentEquip: string;
            equip: Record<
                string,
                {
                    hide: number;
                    locked: number;
                    level: number;
                }
            >;
        }[];
        lastOnlineTime: number;
        medalBoard: {
            type: string;
            custom: string | null;
            template: {
                groupId: string;
                medalList: string[];
            };
        };
        skin: {
            selected: string;
            state: any; // Unsure
        };
    }[];
    friendStatusList: number[];
    resultIdList: string[];
    playerDataDelta: {
        modified: any; // Unsure
        deleted: any; // Unsure
    };
};
