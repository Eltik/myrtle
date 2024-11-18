import type { CharacterData, User } from "~/types/impl/api";
import { type Operator, OperatorRarity } from "~/types/impl/api/static/operator";

export const rarityToNumber = (rarity: OperatorRarity): number => {
    switch (rarity) {
        case OperatorRarity.sixStar:
            return 6;
        case OperatorRarity.fiveStar:
            return 5;
        case OperatorRarity.fourStar:
            return 4;
        case OperatorRarity.threeStar:
            return 3;
        case OperatorRarity.twoStar:
            return 2;
        case OperatorRarity.oneStar:
            return 1;
        default:
            return 0;
    }
};

export const stringToOperatorRarity = (string: string): OperatorRarity => {
    switch (string) {
        case "TIER_6":
            return OperatorRarity.sixStar;
        case "TIER_5":
            return OperatorRarity.fiveStar;
        case "TIER_4":
            return OperatorRarity.fourStar;
        case "TIER_3":
            return OperatorRarity.threeStar;
        case "TIER_2":
            return OperatorRarity.twoStar;
        case "TIER_1":
            return OperatorRarity.oneStar;
        default:
            return OperatorRarity.sixStar;
    }
};

export const getAttributeStats = (character: CharacterData, level: number): Operator["phases"][number]["attributesKeyFrames"][number]["data"] | null => {
    const phase = getCurrentPhase(character);
    if (!phase) return null;

    const keyFrames = phase.attributesKeyFrames;
    if (!keyFrames || keyFrames.length === 0) {
        return null;
    }

    if (level < 1 || level > phase.maxLevel) {
        return null;
    }

    const lowerFrame = keyFrames.find((frame) => frame.level <= level);
    const upperFrame = [...keyFrames].reverse().find((frame) => frame.level >= level);

    if (!lowerFrame || !upperFrame) {
        return null;
    }

    if (lowerFrame.level === upperFrame.level) {
        return lowerFrame.data;
    }

    const interpolate = (lower: number, upper: number, ratio: number): number => {
        return lower + (upper - lower) * ratio;
    };

    const ratio = (level - lowerFrame.level) / (upperFrame.level - lowerFrame.level);
    const interpolatedStats: Record<string, number | boolean> = {};
    for (const key in lowerFrame.data) {
        const lowerValue = lowerFrame.data[key as keyof typeof lowerFrame.data];
        const upperValue = upperFrame.data[key as keyof typeof upperFrame.data];

        if (typeof lowerValue === "number" && typeof upperValue === "number") {
            interpolatedStats[key] = interpolate(lowerValue, upperValue, ratio);
        } else {
            interpolatedStats[key] = lowerValue;
        }
    }

    return interpolatedStats as Operator["phases"][number]["attributesKeyFrames"][number]["data"];
};

export const getMaxAttributeStats = (character: CharacterData): Operator["phases"][number]["attributesKeyFrames"][number]["data"] | null => {
    return getCurrentPhase(character)?.attributesKeyFrames[(getCurrentPhase(character)?.attributesKeyFrames ?? []).length - 1]?.data ?? null;
};

export const getCurrentPhase = (character: CharacterData) => {
    const phase = character.evolvePhase;
    return character.static?.phases[phase];
};

export const formatProfession = (profession: string): string => {
    switch (profession.toLocaleLowerCase()) {
        case "pioneer":
            return "Vanguard";
        case "tank":
            return "Defender";
        case "sniper":
            return "Sniper";
        case "warrior":
            return "Guard";
        case "caster":
            return "Caster";
        case "support":
            return "Supporter";
        case "special":
            return "Specialist";
        case "medic":
            return "Medic";
        default:
            return profession;
    }
};

export const formatSubProfession = (subProfession: string): string => {
    switch (subProfession) {
        case "physician":
            return "ST Medic";
        case "fearless":
            return "Dreadnought";
        case "executor":
            return "Executor";
        case "fastshot":
            return "Marksman Sniper";
        case "bombarder":
            return "Flinger";
        case "bard":
            return "Bard";
        case "protector":
            return "Protector";
        case "ritualist":
            return "Ritualist";
        case "pioneer":
            return "Pioneer";
        case "corecaster":
            return "Core Caster";
        case "splashcaster":
            return "AOE Caster";
        case "charger":
            return "Charger";
        case "centurion":
            return "Centurion";
        case "guardian":
            return "Guardian";
        case "slower":
            return "Decel Binder";
        case "funnel":
            return "Mech-Accord Caster";
        case "mystic":
            return "Mystic Caster";
        case "chain":
            return "Chain Caster";
        case "aoesniper":
            return "AOE Sniper";
        case "reaperrange":
            return "Spreadshooter";
        case "longrange":
            return "Deadeye Sniper";
        case "closerange":
            return "Heavyshooter";
        case "siegesniper":
            return "Besieger";
        case "loopshooter":
            return "Loopshooter";
        case "bearer":
            return "Flag Bearer";
        case "tactician":
            return "Tactician";
        case "instructor":
            return "Instructor";
        case "lord":
            return "Lord";
        case "artsfghter":
            return "Arts Fighter";
        case "sword":
            return "Swordmaster";
        case "musha":
            return "Musha";
        case "crusher":
            return "Crusher";
        case "reaper":
            return "Reaper";
        case "merchant":
            return "Merchant";
        case "hookmaster":
            return "Hookmaster";
        case "ringhealer":
            return "AOE Medic";
        case "healer":
            return "Therapist";
        case "wandermedic":
            return "Wandering Medic";
        case "unyield":
            return "Juggernaught";
        case "artsprotector":
            return "Arts Protector";
        case "summoner":
            return "Summoner";
        case "craftsman":
            return "Artificer";
        case "stalker":
            return "Ambusher";
        case "pusher":
            return "Pusher";
        case "dollkeeper":
            return "Doll Keeper";
        case "agent":
            return "Agent";
        case "fighter":
            return "Brawler";
        case "liberator":
            return "Liberator";
        case "hammer":
            return "Earthshaker";
        case "phalanx":
            return "Phalanx";
        case "blastcaster":
            return "Blast Caster";
        case "primcaster":
            return "Primal Caster";
        case "incantationmedic":
            return "Incantation Medic";
        case "chainhealer":
            return "Chain Healer";
        case "shotprotector":
            return "Sentinel";
        case "fortress":
            return "Fortress";
        case "duelist":
            return "Duelist";
        case "hunter":
            return "Hunter";
        case "geek":
            return "Geek";
        case "underminer":
            return "Hexer";
        case "blessing":
            return "Abjurer";
        case "traper":
            return "Trapmaster";
        default:
            return subProfession;
    }
};

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
