import type { CharacterData, User } from "~/types/impl/api";
import { OperatorPhase, OperatorProfession, OperatorRarity } from "~/types/impl/api/static/operator";

export const parsePhase = (phase: string) => {
    switch (phase) {
        case "PHASE_2":
            return OperatorPhase.ELITE_2;
        case "PHASE_1":
            return OperatorPhase.ELITE_1;
        case "PHASE_0":
            return OperatorPhase.ELITE_0;
        default:
            return OperatorPhase.ELITE_0;
    }
};

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

export const formatNationId = (nationId: string) => {
    switch (nationId.toLowerCase()) {
        case "rhodes":
            return "Rhodes Island";
        case "kazimierz":
            return "Kazimierz";
        case "columbia":
            return "Columbia";
        case "laterano":
            return "Laterano";
        case "victoria":
            return "Victoria";
        case "sami":
            return "Sami";
        case "bolivar":
            return "Bolivar";
        case "iberia":
            return "Iberia";
        case "siracusa":
            return "Siracusa";
        case "higashi":
            return "Higashi";
        case "sargon":
            return "Sargon";
        case "kjerag":
            return "Kjerag";
        case "minos":
            return "Minos";
        case "yan":
            return "Yan";
        case "lungmen":
            return "Lungmen";
        case "ursus":
            return "Ursus";
        case "egir":
            return "Ægir";
        case "leithanien":
            return "Leithanien";
        case "rim":
            return "Rim Billiton";
        default:
            return capitalize(nationId);
    }
};

export const formatGroupId = (groupId: string) => {
    switch (groupId.toLowerCase()) {
        case "pinus":
            return "Pinus Sylvestris";
        case "blacksteel":
            return "Blacksteel";
        case "karlan":
            return "Karlan Trade";
        case "sweep":
            return "S.W.E.E.P.";
        case "rhine":
            return "Rhine Lab";
        case "penguin":
            return "Penguin Logistics";
        case "siesta":
            return "Siesta";
        case "lgd":
            return "L.G.D.";
        case "glasgow":
            return "Glasgow Gang";
        case "abyssal":
            return "Abyssal Hunters";
        case "dublinn":
            return "Dublinn";
        case "elite":
            return "Rhodes Island Elite Operators";
        case "sui":
            return "Yan Sui";
        default:
            return capitalize(groupId);
    }
};

export const formatTeamId = (teamId: string) => {
    switch (teamId.toLowerCase()) {
        case "action4":
            return "Op Team A4";
        case "reserve1":
            return "Reserve Op Team A1";
        case "reserve4":
            return "Reserve Op Team A4";
        case "reserve6":
            return "Reserve Op Team A6";
        case "student":
            return "Ursus Student Self-Governing Group";
        case "chiave":
            return "Chiave's Gang";
        case "rainbow":
            return "Team Rainbow";
        case "followers":
            return "Followers";
        case "lee":
            return "Lee's Detective Agency";
        default:
            return capitalize(teamId);
    }
};

export const formatProfession = (profession: string): string => {
    switch (profession.toLowerCase()) {
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

export const sortProfessions = (professions: OperatorProfession[]): OperatorProfession[] => {
    return professions.sort((a, b) => {
        const order = [OperatorProfession.VANGUARD, OperatorProfession.GUARD, OperatorProfession.DEFENDER, OperatorProfession.SNIPER, OperatorProfession.CASTER, OperatorProfession.SUPPORTER, OperatorProfession.MEDIC, OperatorProfession.SPECIALIST];
        return order.indexOf(a) - order.indexOf(b);
    });
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
            return "Dollkeeper";
        case "agent":
            return "Agent";
        case "fighter":
            return "Brawler";
        case "librator":
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
        case "alchemist":
            return "Alchemist";
        default:
            return subProfession;
    }
};

export const formatSkillType = (type: string | number): string => {
    return type === 8 ? "Passive" : type === "INCREASE_WHEN_ATTACK" ? "Offensive" : type === "INCREASE_WITH_TIME" ? "Auto" : "Defensive";
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

export function getAvatar(char: CharacterData) {
    let skinId = "";

    if (!char.skin || char.skin.endsWith("#1")) {
        skinId = normalizeSkinId(char.charId);
    } else if (char.skin.endsWith("#2")) {
        skinId = normalizeSkinId(char.charId) + "_2";
    } else if (char.skin.includes("@")) {
        skinId = normalizeSkinId(char.skin);
    } else {
        skinId = normalizeSkinId(char.charId);
    }

    const icon = `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${skinId}.png`;
    return skinId.length === 0 ? `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${normalizeSkinId(char.charId)}.png` : icon;
}

export function normalizeSkinId(skinId: string) {
    if (skinId.includes("@")) {
        return encodeURIComponent(skinId.replaceAll("@", "_"));
    } else {
        return encodeURIComponent(skinId.replaceAll("#", "_"));
    }
}

export function getAvatarById(charId: string) {
    return `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${normalizeSkinId(charId)}.png`;
}

export function getAvatarSkinId(user: User) {
    if (!user.status) return "https://static.wikia.nocookie.net/mrfz/images/4/46/Symbol_profile.png/revision/latest?cb=20220418145951";

    const secretaryId = user.status.secretary;
    const secretarySkinId = user.status.secretarySkinId;

    const skinId = !secretarySkinId.includes("@") && secretarySkinId.endsWith("#1") ? secretaryId : secretarySkinId;

    return getAvatarById(skinId);
}
