import type { User } from "~/types/impl/api";

export const formatSkillType = (type: string): string => {
    return type === "INCREASE_WHEN_ATTACK" ? "Offensive" : type === "INCREASE_WITH_TIME" ? "Auto" : "Defensive";
};

export function capitalize(s: string) {
    s = s?.toLowerCase();
    return s && (s[0]?.toUpperCase() ?? "") + s.slice(1);
}

/**
 * @author All credit to https://github.com/Awedtan/HellaBot/
 *
 * @param text
 * @param blackboard
 * @returns string | null
 */
export function insertBlackboard(
    text: string,
    blackboard: {
        key: string;
        value: number;
        valueStr?: string | null;
    }[],
): string | null {
    // Note: check these every so often to see if their skills still display properly
    // silverash s2/s3
    // eyjafjalla s2
    // lin s1
    // tachanka s1/s2
    // mizuki s1
    // mostima s3
    // irene s1
    // utage s2

    const formatVariable = (
        chunk: string,
        blackboard: {
            key: string;
            value: number;
            valueStr?: string | null;
        }[],
    ) => {
        // {tag} {tag:0} {tag:0%} {tag:0.0} {tag:0.0%}
        const tag = chunk.split(":")[0]?.toLowerCase() ?? "";
        const negative = tag.startsWith("-");
        const key = negative ? tag.slice(1) : tag;
        const variable = blackboard.find((variable) => variable.key === key);

        if (!variable) return chunk;

        const value = negative ? -variable.value : variable.value;
        return chunk.endsWith("%") ? `<b>${Math.round(value * 100)}%</b>` : `${value}`;
    };

    const textArr = removeStyleTags(text.trim()).split(/{|}/);

    if (textArr.join("") === "") return null;

    for (let i = 0; i < textArr.length; i++) {
        textArr[i] = formatVariable(textArr[i] ?? "", blackboard);
    }

    return textArr.join("").replaceAll("-<b>", "<b>-").replaceAll("+<b>", "<b>+");
}

/**
 * @author All credit to https://github.com/Awedtan/HellaBot/
 * @param text
 * @returns string
 */

export function removeStyleTags(text: string): string {
    return text.replace(/<.[a-z]{2,5}?\.[^<]+>|<\/[^<]*>|<color=[^>]+>/g, "");
}

export function parseSkillStaticLevel(mainSkillLevel: number, skillSpecializationLevel: number): number {
    if (skillSpecializationLevel > 0) {
        return mainSkillLevel - 1 + skillSpecializationLevel;
    } else {
        return mainSkillLevel - 1;
    }
}

export function getAvatarSkinId(user: User) {
    if (!user.status) return "https://static.wikia.nocookie.net/mrfz/images/4/46/Symbol_profile.png/revision/latest?cb=20220418145951";

    let skinId = "";
    if (user.status.secretarySkinId) {
        if (user.status.secretarySkinId.includes("@")) {
            skinId = user.status.secretarySkinId.replaceAll("@", "_");
        } else {
            skinId = user.status.secretarySkinId.replaceAll("#", "_");
        }
    } else {
        if (user.status.avatarId) {
            if (user.status.avatar.type === "ASSISTANT") {
                if ((Object.values(user.troop.chars).find((item) => item.skin === user.status.avatar.id)?.charId ?? "").includes("@")) {
                    skinId =
                        Object.values(user.troop.chars)
                            .find((item) => item.skin === user.status.avatar.id)
                            ?.charId?.replaceAll("@", "_") ?? "";
                } else {
                    skinId =
                        Object.values(user.troop.chars)
                            .find((item) => item.skin === user.status.avatar.id)
                            ?.charId?.replaceAll("#", "_") ?? "";
                }
            }
        }
    }

    if (skinId.length > 0) {
        return `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${encodeURIComponent(skinId)}.png`;
    } else {
        return "https://static.wikia.nocookie.net/mrfz/images/4/46/Symbol_profile.png/revision/latest?cb=20220418145951";
    }
}
