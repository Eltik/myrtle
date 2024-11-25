import { range, isEqual } from "lodash-es";
import type { CharacterData, User } from "~/types/impl/api";
import { type BattleEquip } from "~/types/impl/api/static/modules";
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

/**
 * @author Huge thanks and credit to
 * https://github.com/SanityGoneAK/sanity-gone/blob/main/src/utils/character-stats.ts#L80
 * All credit goes to them for this function.
 */
export const getAttributeStats = (character: CharacterData, level: number, moduleData?: BattleEquip): Operator["phases"][number]["attributesKeyFrames"][number]["data"] | null => {
    const doStatsChange = (data: CharacterData): boolean => {
        const phase = getCurrentPhase(data);
        if (!phase) return false;

        const startingKeyFrame = phase.attributesKeyFrames[0];
        const finalKeyFrame = phase.attributesKeyFrames[phase.attributesKeyFrames.length - 1];

        return !isEqual(startingKeyFrame?.data, finalKeyFrame?.data);
    };

    const getStatIncreaseAtTrust = (
        data: CharacterData,
        rawTrust: number,
    ): {
        maxHp: number;
        atk: number;
        def: number;
        magicResistance: number;
    } => {
        if (data.static?.favorKeyFrames == null) {
            throw new Error(`Can't get trust stat increase, favorKeyFrames is null.`);
        }

        const trust = Math.min(100, rawTrust);
        const maxTrust = data.static?.favorKeyFrames[data.static?.favorKeyFrames.length - 1]?.data;

        return {
            maxHp: Math.round((trust * (maxTrust?.maxHp ?? 0)) / 100),
            atk: Math.round((trust * (maxTrust?.atk ?? 0)) / 100),
            def: Math.round((trust * (maxTrust?.def ?? 0)) / 100),
            magicResistance: Math.round((trust * (maxTrust?.magicResistance ?? 0)) / 100),
        };
    };

    const getStatIncreaseAtPotential = (
        data: CharacterData,
        potential: number,
    ): {
        health: number;
        attackPower: number;
        defense: number;
        artsResistance: number;
        dpCost: number;
        attackSpeed: number;
        redeployTimeInSeconds: number;
        description: string | null;
    } => {
        const initialIncreases = {
            health: 0,
            attackPower: 0,
            defense: 0,
            artsResistance: 0,
            dpCost: 0,
            attackSpeed: 0,
            redeployTimeInSeconds: 0,
            description: null,
        };
        if (potential === 0) {
            return initialIncreases;
        }

        const relevantStatIncreases = range(1, potential + 1).map((p) => getPotentialStatIncrease(data, p));
        return relevantStatIncreases.reduce((vals, previous) => {
            return {
                health: vals.health + previous.health,
                attackPower: vals.attackPower + previous.attackPower,
                defense: vals.defense + previous.defense,
                artsResistance: vals.artsResistance + previous.artsResistance,
                dpCost: vals.dpCost + previous.dpCost,
                attackSpeed: vals.attackSpeed + previous.attackSpeed,
                redeployTimeInSeconds: vals.redeployTimeInSeconds + previous.redeployTimeInSeconds,
                description: null,
            };
        }, initialIncreases);
    };

    const getModuleStatIncrease = (
        characterObject: CharacterData,
        moduleId: string,
        moduleLevel: number,
        moduleData: BattleEquip,
    ): {
        atk: number;
        max_hp: number;
        def: number;
        attack_speed: number;
        magic_resistance: number;
        cost: number;
        respawn_time: number;
        block_cnt: number;
    } => {
        const statChanges = {
            atk: 0,
            max_hp: 0,
            def: 0,
            attack_speed: 0,
            magic_resistance: 0,
            cost: 0,
            respawn_time: 0,
            block_cnt: 0,
        };

        if (characterObject.equip == null) {
            throw new Error(`CharacterObject doesn't have modules, cannot get stat increase.`);
        }

        const operatorModule = moduleData[moduleId];

        const modulePhase = operatorModule?.phases[moduleLevel - 1];

        const toCheck = modulePhase?.attributeBlackboard;

        if (!toCheck) {
            return statChanges;
        }

        toCheck.forEach((iv) => {
            if (!(iv.key in statChanges)) {
                throw new Error(`Unknown attribute modified: ${iv.key} with value of ${iv.value} on module ${characterObject.charId}`);
            }
            // @ts-expect-error - This is fine
            statChanges[iv.key] += iv.value;
        });

        return statChanges;
    };

    const linearInterpolateByLevel = (level: number, maxLevel: number, baseValue: number, maxValue: number): number => {
        return Math.round(baseValue + ((level - 1) * (maxValue - baseValue)) / (maxLevel - 1));
    };

    const calculateSecondsPerAttack = (baseAttackTime: number, aspd: number): number => {
        return Math.round((baseAttackTime * 30) / (aspd / 100.0)) / 30;
    };

    const getPotentialStatIncrease = (
        data: CharacterData,
        potential: number,
    ): {
        health: number;
        attackPower: number;
        defense: number;
        artsResistance: number;
        dpCost: number;
        attackSpeed: number;
        redeployTimeInSeconds: number;
        description: string | null;
    } => {
        if (potential === 0) {
            return {
                artsResistance: 0,
                attackPower: 0,
                attackSpeed: 0,
                defense: 0,
                description: null,
                dpCost: 0,
                health: 0,
                redeployTimeInSeconds: 0,
            };
        }

        const potentialRanks = data.static?.potentialRanks;
        const pot = potentialRanks?.[potential - 1];

        const statChanges = {
            health: 0,
            attackPower: 0,
            defense: 0,
            artsResistance: 0,
            dpCost: 0,
            attackSpeed: 0,
            redeployTimeInSeconds: 0,
            description: null,
        };

        if (pot?.buff == null) {
            let desc = pot?.description ?? "";
            if (desc.startsWith("Improves ")) {
                desc = desc.replace("Improves ", "") + " Enhancement";
            } else if (desc === "天赋效果增强") {
                desc = "Talent Enhancement";
            } else if (desc === "第一天赋效果增强") {
                desc = "First Talent Enhancement";
            } else if (desc === "第二天赋效果增强") {
                desc = "Second Talent Enhancement";
            }
            return statChanges;
        }

        const attribType = pot.buff.attributes.attributeModifiers?.[0]?.attributeType;
        const attribChange = pot.buff.attributes.attributeModifiers?.[0]?.value;

        switch (attribType) {
            case "MAX_HP":
                statChanges.health += attribChange ?? 0;
                break;
            case "ATK":
                statChanges.attackPower += attribChange ?? 0;
                break;
            case "DEF":
                statChanges.defense += attribChange ?? 0;
                break;
            case "MAGIC_RESISTANCE":
                statChanges.artsResistance += attribChange ?? 0;
                break;
            case "COST":
                statChanges.dpCost += attribChange ?? 0;
                break;
            case "ATTACK_SPEED":
                statChanges.attackSpeed += attribChange ?? 0;
                break;
            case "RESPAWN_TIME":
                statChanges.redeployTimeInSeconds += attribChange ?? 0;
                break;
            default:
                console.warn("Unrecognized attribute in potentials");
                break;
        }

        return statChanges;
    };

    const phase = getCurrentPhase(character);
    if (!phase) return null;

    const keyFrames = phase.attributesKeyFrames;
    if (!keyFrames || keyFrames.length === 0) {
        return null;
    }

    if (level < 1 || level > phase.maxLevel) {
        return null;
    }

    const trust = character.favorPoint;
    const potential = character.potentialRank;
    const moduleId = character.currentEquip;
    const moduleLevel = character.equip[moduleId as unknown as keyof typeof character.equip]?.level ?? 0;

    const maxLevel = phase.maxLevel;

    const startingKeyFrame = phase.attributesKeyFrames[0];
    const finalKeyFrame = phase.attributesKeyFrames[phase.attributesKeyFrames.length - 1];

    const { maxHp, atk, def, magicResistance: res, cost: dp, blockCnt, respawnTime: redeploy, baseAttackTime } = startingKeyFrame!.data;
    const { maxHp: finalMaxHp, atk: finalMaxAtk, def: finalMaxDef, magicResistance: finalMaxRes } = finalKeyFrame!.data;

    const {
        maxHp: trustHp,
        atk: trustAtk,
        def: trustDef,
        magicResistance: trustRes,
    } = doStatsChange(character)
        ? getStatIncreaseAtTrust(character, trust)
        : {
              maxHp: 0,
              atk: 0,
              def: 0,
              magicResistance: 0,
          };

    const {
        health: potHealth,
        attackPower: potAttack,
        defense: potDefense,
        artsResistance: potRes,
        attackSpeed: potASPD,
        dpCost: potDp,
        redeployTimeInSeconds: potRedeploy,
    } = doStatsChange(character)
        ? getStatIncreaseAtPotential(character, potential)
        : {
              health: 0,
              attackPower: 0,
              defense: 0,
              artsResistance: 0,
              attackSpeed: 0,
              dpCost: 0,
              redeployTimeInSeconds: 0,
          };

    const {
        atk: modAttack,
        max_hp: modHealth,
        def: modDefense,
        attack_speed: modASPD,
        magic_resistance: modRes,
        cost: modDp,
        respawn_time: modRedeploy,
        block_cnt: modBlock,
    } = moduleId && moduleData
        ? getModuleStatIncrease(character, moduleId, moduleLevel, moduleData)
        : {
              atk: 0,
              max_hp: 0,
              def: 0,
              attack_speed: 0,
              magic_resistance: 0,
              cost: 0,
              respawn_time: 0,
              block_cnt: 0,
          };

    const health = linearInterpolateByLevel(level, maxLevel, maxHp, finalMaxHp) + trustHp + potHealth + modHealth;
    const attackPower = linearInterpolateByLevel(level, maxLevel, atk, finalMaxAtk) + trustAtk + potAttack + modAttack;
    const defense = linearInterpolateByLevel(level, maxLevel, def, finalMaxDef) + trustDef + potDefense + modDefense;
    const artsResistance = linearInterpolateByLevel(level, maxLevel, res, finalMaxRes) + trustRes + potRes + modRes;

    const redeployTimeInSeconds = redeploy + potRedeploy + modRedeploy;
    const dpCost = dp + potDp + modDp;
    const blockCount = blockCnt + modBlock;

    // ASPD
    const secondsPerAttack = calculateSecondsPerAttack(baseAttackTime, 100 + potASPD + modASPD);

    const stats: Operator["phases"][number]["attributesKeyFrames"][number]["data"] = {
        atk: attackPower,
        attackSpeed: secondsPerAttack,
        baseAttackTime,
        baseForceLevel: finalKeyFrame!.data.baseForceLevel,
        blockCnt: blockCount,
        cost: dpCost,
        def: defense,
        disarmedCombatImmune: finalKeyFrame!.data.disarmedCombatImmune,
        frozenImmune: finalKeyFrame!.data.frozenImmune,
        hpRecoveryPerSec: finalKeyFrame!.data.hpRecoveryPerSec,
        levitateImmune: finalKeyFrame!.data.levitateImmune,
        magicResistance: artsResistance,
        massLevel: finalKeyFrame!.data.massLevel,
        maxDeckStackCnt: finalKeyFrame!.data.maxDeckStackCnt,
        maxDeployCount: finalKeyFrame!.data.maxDeployCount,
        maxHp: health,
        moveSpeed: finalKeyFrame!.data.moveSpeed,
        respawnTime: redeployTimeInSeconds,
        silenceImmune: finalKeyFrame!.data.silenceImmune,
        sleepImmune: finalKeyFrame!.data.sleepImmune,
        spRecoveryPerSec: finalKeyFrame!.data.spRecoveryPerSec,
        stunImmune: finalKeyFrame!.data.stunImmune,
        tauntLevel: finalKeyFrame!.data.tauntLevel,
    };

    return stats;
};

export const getMaxAttributeStats = (character: CharacterData): Operator["phases"][number]["attributesKeyFrames"][number]["data"] | null => {
    return getCurrentPhase(character)?.attributesKeyFrames[(getCurrentPhase(character)?.attributesKeyFrames ?? []).length - 1]?.data ?? null;
};

export const getCurrentPhase = (character: CharacterData) => {
    const phase = character.evolvePhase;
    return character.static?.phases[phase];
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
