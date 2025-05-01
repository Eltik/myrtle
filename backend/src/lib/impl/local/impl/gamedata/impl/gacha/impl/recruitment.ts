/**
 * @author Credit to akgcc
 * https://github.com/akgcc/akgcc.github.io/blob/master/cc/js/recruit.js
 */

import { getAll } from "..";
import { getAll as getCharPatch } from "../../charpatch";
import { OperatorPosition, OperatorProfession, type Operator } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { STATIC_DATA } from "../../../../handler";
import { getOperator } from "../../..";
import type { GachaTag } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/gacha";
import { operatorRarityToNumber } from "../../../../../../dps-calculator/impl/helper/operatorRarityToNumber";
import type { RecruitGroup } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/gacha/impl/recruitment";

const CLASS_MAPPING = {
    WARRIOR: "Guard",
    SUPPORT: "Supporter",
    CASTER: "Caster",
    SNIPER: "Sniper",
    TANK: "Defender",
    PIONEER: "Vanguard",
    SPECIAL: "Specialist",
    MEDIC: "Medic",
};

const OP_NAME_SUBSTITUTIONS = {
    "justice knight": "'justice knight'",
    "サーマル-ex": "thrm-ex",
    샤미르: "샤마르",
};

export const getRecruitment = () => {
    const gachaTable = getAll();

    const RECRUIT_POOL: Record<string, Operator> = {};
    const TAGS = gachaTable.gachaTags;
    const TAG_MAP = {} as Record<string, GachaTag>;
    const TAG_NAME_MAP = {} as Record<string, GachaTag>;
    const TAG_CATEGORIES = {
        Rarity: [28, 17, 14, 11],
        Position: [9, 10],
        Class: [8, 1, 3, 2, 6, 4, 5, 7],
        Affix: [15, 16, 19, 21, 23, 20, 22, 24, 26, 12, 13, 27, 25, 18, 29],
    } as Record<string, number[]>;
    const HIDDEN_TAGS = [1012, 1013]; // Male, Female

    const recruitDetail = gachaTable.recruitDetail;

    const chars = getCharTable();

    if (!getRecruitPool(chars, recruitDetail, RECRUIT_POOL)) {
        return null;
    }

    TAGS.forEach((tag) => {
        Object.assign(TAG_MAP, {
            [tag.tagId]: tag,
        });

        Object.assign(TAG_NAME_MAP, {
            [tag.tagName]: tag,
        });
    });

    Object.keys(TAG_CATEGORIES).forEach((category) => {
        TAG_CATEGORIES[category].forEach((tagId) => {
            Object.assign(TAG_MAP[tagId], {
                tagCat: category,
            });
        });
    });

    Object.keys(TAG_MAP).forEach((tagId) => {
        if (!TAG_MAP[tagId].tagCat && !HIDDEN_TAGS.includes(parseInt(tagId))) {
            Object.assign(TAG_MAP[tagId], {
                tagCat: "Affix",
            });

            TAG_CATEGORIES["Affix"].push(parseInt(tagId));
        }
    });

    return {
        TAG_MAP,
        TAG_NAME_MAP,
        TAG_CATEGORIES,
        RECRUIT_POOL,
    };
};

const getRecruitPool = (chars: Record<string, Operator>, recruitDetail: string, recruitPool: Record<string, Operator>) => {
    const NAME_MAP: Record<string, Operator> = {};
    const RECRUIT_NAMES = new Set<string>();
    const ALL_OPS = new Set(Object.values(chars).map((op) => op.name.toLowerCase()));

    Object.values(chars).forEach((op) => (NAME_MAP[op.name.toLowerCase()] = op));

    if (
        ![...recruitDetail.matchAll(/(?<!>\s)<@rc\.eml>([^,，]*?)<\/>|(?:\/\s*|\n\s*|\\n\s*)((?!-)[^\r\/>★]+?(?<!-))(?=\/|$)/gim)].every((m) => {
            let operatorName = (m[1] || m[2]).trim().toLowerCase();
            operatorName = OP_NAME_SUBSTITUTIONS[operatorName as keyof typeof OP_NAME_SUBSTITUTIONS] || operatorName;

            if (Object.keys(NAME_MAP).includes(operatorName)) {
                const operator = NAME_MAP[operatorName];
                Object.assign(operator, {
                    recruitOnly: !!m[1],
                });

                Object.assign(recruitPool, {
                    [operator.id ?? ""]: operator,
                });

                RECRUIT_NAMES.add(operatorName);
                return true;
            }

            return false;
        })
    ) {
        return false;
    }

    const NAME_EXCEPTIONS = ["시"]; // Exclude Dusk in KR

    const OPERATOR_MATCHES = new Set<string>();

    const ALL_WORD_MATCHES = new Set([...recruitDetail.matchAll(/[^\s\\><]+/gim)]);

    ALL_WORD_MATCHES.forEach((m) => {
        const word = m[0].toLowerCase();
        if (ALL_OPS.has(word) && !NAME_EXCEPTIONS.includes(word)) {
            OPERATOR_MATCHES.add(word);
        }
    });

    if (!isSuperset(RECRUIT_NAMES, OPERATOR_MATCHES)) {
        return false;
    }

    return true;
};

export const calculateRecruitment = (recruitment: Set<string>, RECRUIT_POOL: Record<string, Operator>, TAG_MAP: Record<string, GachaTag>, showRobots: boolean = true) => {
    const combinations = getCombinations(recruitment);

    const groups: RecruitGroup[] = [];

    for (const combination of combinations) {
        if (combination.length === 0) continue;

        const tags = combination.map((tag) => TAG_MAP[tag]);
        const matches = [];

        const hasTopOp = combination.includes("11");

        const filteredOps = Object.values(RECRUIT_POOL)
            .filter((op) => hasTopOp || operatorRarityToNumber(op.rarity) < 5)
            .filter(Boolean);

        for (const op of filteredOps) {
            const match = tags.every((tag) => {
                switch (tag.tagCat) {
                    case "Position":
                        switch (tag.tagId) {
                            case 9:
                                return op.position === OperatorPosition.MELEE;
                            case 10:
                                return op.position === OperatorPosition.RANGED;
                        }
                    case "Rarity":
                        switch (tag.tagId) {
                            case 17:
                                return operatorRarityToNumber(op.rarity) === 1;
                            case 14:
                                return operatorRarityToNumber(op.rarity) === 4;
                            case 11:
                                return operatorRarityToNumber(op.rarity) === 5;
                            case 28:
                                return operatorRarityToNumber(op.rarity) === 0;
                        }
                    case "Class":
                        switch (tag.tagId) {
                            case 1:
                                return op.profession === OperatorProfession.GUARD;
                            case 2:
                                return op.profession === OperatorProfession.SNIPER;
                            case 3:
                                return op.profession === OperatorProfession.DEFENDER;
                            case 4:
                                return op.profession === OperatorProfession.MEDIC;
                            case 5:
                                return op.profession === OperatorProfession.SUPPORTER;
                            case 6:
                                return op.profession === OperatorProfession.CASTER;
                            case 7:
                                return op.profession === OperatorProfession.SPECIALIST;
                            case 8:
                                return op.profession === OperatorProfession.VANGUARD;
                        }
                    case "Affix":
                        switch (tag.tagId) {
                            default:
                                return op.tagList?.includes(tag.tagName);
                        }
                }
            });

            if (match) {
                matches.push(op);
            }

            const newGroup: RecruitGroup = {
                tags,
                matches,
                lowest9hrRarity: matches.reduce((minr, op) => (operatorRarityToNumber(op.rarity) < 2 ? minr : Math.min(minr, operatorRarityToNumber(op.rarity))), 99),
                highestRarity: matches.reduce((maxr, op) => Math.max(maxr, operatorRarityToNumber(op.rarity)), 0),
                nineHourOpCount: matches.reduce((count, op) => (operatorRarityToNumber(op.rarity) > 1 ? count + 1 : count), 0),
            };

            // Handle special cases with "Starter" and "Robot" tags.
            // Uses 3.5 rarity for robots which puts them above 4* but below 5*.
            if (newGroup.lowest9hrRarity === 99) {
                const lowestRarity = newGroup.matches.reduce((minr, op) => Math.min(minr, operatorRarityToNumber(op.rarity)), 99);

                newGroup.lowest9hrRarity = lowestRarity === 0 && showRobots ? 3.5 : lowestRarity;
                newGroup.nineHourOpCount = newGroup.matches.length;
            }

            newGroup.matches.sort((a, b) => {
                if (a.rarity !== b.rarity) {
                    if (showRobots && newGroup.lowest9hrRarity === 3) {
                        if (operatorRarityToNumber(a.rarity) === 0) return -1;
                        if (operatorRarityToNumber(b.rarity) === 0) return 1;
                    }

                    if (operatorRarityToNumber(a.rarity) < 2) {
                        return operatorRarityToNumber(b.rarity) < 2 ? operatorRarityToNumber(b.rarity) - operatorRarityToNumber(a.rarity) : 1;
                    } else if (operatorRarityToNumber(b.rarity) < 2) {
                        return -1;
                    } else {
                        return operatorRarityToNumber(a.rarity) - operatorRarityToNumber(b.rarity);
                    }
                }

                return a.name.localeCompare(b.name);
            });

            groups.push(newGroup);
        }
    }

    return groups;
};

const getCombinations = (recruitment: Set<string>) => {
    const elements = Array.from(recruitment);
    const combinations: string[][] = [[]];

    for (let i = 0; i < elements.length; i++) {
        const currentSubsetLength = combinations.length;

        for (let j = 0; j < currentSubsetLength; j++) {
            const subset = [...combinations[j], elements[i]];
            combinations.push(subset);
        }
    }

    return combinations;
};

const getCharTable = () => {
    const characterTable = STATIC_DATA?.CHARACTER_TABLE as Record<string, Operator>;
    const charPatchTable = getCharPatch();

    const chars = mergeCharTable(characterTable, charPatchTable.patchChars);

    Object.keys(chars).forEach((op) => {
        const char = getOperator(op);
        if (char) {
            chars[op] = char;
            if (op.includes("_amiya")) chars[op].name = `${chars[op].name} (${CLASS_MAPPING[chars[op].profession] || chars[op].profession})`;
        }
    });

    return chars;
};

const mergeCharTable = (dest: any, src: any, existingOnly: boolean = false): Record<string, Operator> => {
    for (const key in src) {
        if (typeof dest[key] == "object" && typeof src[key] == "object") dest[key] = mergeCharTable(dest[key], src[key], false);
        else if (!existingOnly || key in dest) dest[key] = src[key];
    }
    return dest;
};

function isSuperset(set: Set<string>, subset: Set<string>) {
    for (const elem of subset) {
        if (!set.has(elem)) {
            return false;
        }
    }
    return true;
}
