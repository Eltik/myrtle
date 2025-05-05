import type { AKServer, AuthSession } from "../../../../../types/impl/lib/impl/authentication";
import type { User, UserResponse } from "../../../../../types/impl/lib/impl/user/impl/get";
import { authRequest } from "../../../authentication/impl/request/impl/auth";
import { calculateTrust, getMaterial, getOperator, getSkill } from "../../../local/impl/gamedata";

export default async (session: AuthSession, server: AKServer): Promise<User | null> => {
    const body = {
        platform: 1,
    };

    const data = (await (
        await authRequest(
            "account/syncData",
            session,
            {
                method: "POST",
                body: body,
            },
            server,
        )
    ).json()) as UserResponse;

    if (!data.user) return null;
    return await formatUser(data.user);
};

const formatUser = async (data: User) => {
    const inventoryPromises = Object.keys(data.inventory).map(async (key) => {
        const item = await getMaterial(key);
        const value = data.inventory[key];
        if (!item || !value) return;

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

    const charsPromises = Object.values(data.troop.chars).map(async (character) => {
        const staticData = await getOperator(character.charId);
        if (staticData) {
            const trust = await calculateTrust(character.favorPoint);

            Object.assign(character, {
                static: {
                    ...staticData,
                    trust,
                },
            });

            const skillsPromises = character.skills.map(async (skill) => {
                const staticData = await getSkill(skill.skillId);
                if (!staticData) return;

                Object.assign(skill, {
                    static: {
                        ...staticData,
                        name: staticData.levels[character.mainSkillLvl - 1 + skill.specializeLevel].name,
                        description: staticData.levels[character.mainSkillLvl - 1 + skill.specializeLevel].description,
                        duration: staticData.levels[character.mainSkillLvl - 1 + skill.specializeLevel].duration,
                        spData: staticData.levels[character.mainSkillLvl - 1 + skill.specializeLevel].spData,
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
