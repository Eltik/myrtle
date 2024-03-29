import { authRequest } from "../../../helper/request";
import type { PlayerData } from "../../../types/types";
import type { AKServer } from "../authentication/auth";
import type { AuthSession } from "../authentication/auth-session";
import { calculateTrust, getItem, getOperator, getSkill } from "../gamedata";

export const getRawData = async (session: AuthSession, server: AKServer) => {
    const data = await (
        await authRequest(
            "account/syncData",
            session,
            {
                body: JSON.stringify({
                    platform: 1,
                }),
            },
            server,
        )
    ).json();
    return data;
};

export const getSocialSortList = async (session: AuthSession, server: AKServer, type: number, sortKey = ["level"], param = {}) => {
    const data = (await (
        await authRequest(
            "social/getSortListInfo",
            session,
            {
                body: JSON.stringify({
                    type,
                    sortKeyList: sortKey,
                    param,
                }),
            },
            server,
        )
    ).json()) as {
        result: {
            uid: string;
            level: number;
            infoShare: number;
        }[];
    };

    return data.result.sort((a, b) => {
        const aValue = sortKey.map((key) => a[key as keyof typeof a]);
        const bValue = sortKey.map((key) => b[key as keyof typeof b]);
        for (let i = 0; i < aValue.length; i++) {
            if (aValue[i] < bValue[i]) return 1;
            if (aValue[i] > bValue[i]) return -1;
        }
        return 0;
    });
};

export const getRawFriendInfo = async (session: AuthSession, server: AKServer, ids: string[]) => {
    return await (
        await authRequest(
            "social/getFriendList",
            session,
            {
                body: JSON.stringify({ idList: ids }),
            },
            server,
        )
    ).json();
};

export const getRawPlayerInfo = async (session: AuthSession, server: AKServer, ids: string[]) => {
    return await (
        await authRequest(
            "social/searchPlayer",
            session,
            {
                body: JSON.stringify({ idList: ids }),
            },
            server,
        )
    ).json();
};

export const getRawFriendIds = async (session: AuthSession, server: AKServer) => {
    return await getSocialSortList(session, server, 1, ["level", "infoShare"], {});
};

export const searchRawPlayerIDs = async (session: AuthSession, server: AKServer, nickname: string, nickNumber?: string) => {
    return await getSocialSortList(session, server, 0, ["level"], {
        nickName: nickname,
        nickNumber: nickNumber || "",
    });
};

export const getRawBattleReplay = async (session: AuthSession, server: AKServer, battleType: string, stageId: string) => {
    const data = await await authRequest(
        `${battleType}/getBattleReplay`,
        session,
        {
            body: JSON.stringify({ stageId }),
        },
        server,
    );
    console.log(data);

    /*
    replay_data = base64.b64decode(data["battleReplay"])
    with zipfile.ZipFile(io.BytesIO(replay_data), "r") as z, z.open("default_entry") as f:
        return json.load(f)
    */
};

export const searchPlayers = async (session: AuthSession, server: AKServer, nickname: string, nicknumber?: string, limit?: number) => {
    if (nickname.includes("#")) nickname = nickname.split("#")[0];

    const uidData = await searchRawPlayerIDs(session, server, nickname, nicknumber);
    const uidList = uidData.slice(0, limit ? limit : uidData.length).map((item: { uid: string }) => item.uid);

    const data = (await getRawFriendInfo(session, server, uidList)) as { friends: PlayerData[] };
    return data.friends ?? [];
};

export const getPlayers = async (session: AuthSession, server: AKServer, ids: string[]) => {
    const data = (await getRawPlayerInfo(session, server, ids)) as { friends: PlayerData[] };
    return data.friends ?? [];
};

export const getPartialPlayers = async (session: AuthSession, server: AKServer, ids: string[]) => {
    const data = (await getRawPlayerInfo(session, server, ids)) as { players: PlayerData[] };
    return data.players ?? [];
};

// TODO: Test
export const getFriends = async (session: AuthSession, server: AKServer, limit?: number) => {
    const uidData = await getRawFriendIds(session, server);
    const uidList = uidData.slice(0, limit ?? uidData.length ?? 1).map((item: { uid: string }) => item.uid);

    const data = await getRawFriendInfo(session, server, uidList);
    return data;
};

export const getData = async (session: AuthSession, server: AKServer): Promise<PlayerData | null> => {
    const data = (await getRawData(session, server)) as { user: PlayerData };
    if (!data.user) return null;

    return await formatUser(data.user);
};

export const formatUser = async (data: PlayerData) => {
    const inventoryPromises = Object.keys(data.inventory).map(async (key) => {
        const item = await getItem(key);
        const value = data.inventory[key];
        if (!value) return;

        Object.assign(item, {
            amount: value,
        });

        delete data.inventory[key];
        Object.assign(data, {
            inventory: {
                ...data.inventory,
                [item.itemId]: item,
            },
        });
    });

    const charsPromises = Object.values(data.troop.chars).map(async (character: any) => {
        const staticData = await getOperator(character.charId);
        if (staticData) {
            const trust = await calculateTrust(character.favorPoint);

            Object.assign(character, {
                static: {
                    name: staticData.name,
                    description: staticData.description,
                    nationId: staticData.nationId,
                    appellation: staticData.appellation,
                    position: staticData.position,
                    profession: staticData.profession,
                    subProfessionId: staticData.subProfessionId,
                    rarity: staticData.rarity,
                    trust,
                },
            });

            const skillsPromises = character.skills.map(async (skill: any) => {
                const staticData = await getSkill(skill.skillId);
                Object.assign(skill, {
                    static: {
                        name: staticData.levels[character.mainSkillLvl - 1 + skill.specializeLevel].name,
                        description: staticData.levels[character.mainSkillLvl - 1 + skill.specializeLevel].description,
                        duration: staticData.levels[character.mainSkillLvl - 1 + skill.specializeLevel].duration,
                        spData: staticData.levels[character.mainSkillLvl - 1 + skill.specializeLevel].spData,
                        levels: staticData.levels,
                        skillId: staticData.skillId,
                        iconId: staticData.iconId,
                        hidden: staticData.hidden,
                    },
                });
            });
            await Promise.all(skillsPromises);
        }
    });

    await Promise.all(charsPromises);
    await Promise.all(inventoryPromises);
    return data;
};
