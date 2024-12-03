import { type Operator, OperatorPhase } from "~/types/impl/api/static/operator";
import { AttributeKeys } from "..";
import { applyEquip } from "./apply-equip";
import { applyPotential } from "./apply-potential";
import { getAttribute } from "./get-attribute";
import { getBlackboard } from "./get-blackboard";

const operatorPhaseToNumber = (phase: OperatorPhase) => {
    switch (phase) {
        case OperatorPhase.ELITE_0:
            return 0;
        case OperatorPhase.ELITE_1:
            return 1;
        case OperatorPhase.ELITE_2:
            return 2;
        default:
            return 0;
    }
};

export function getAttributes(
    char: Operator,
    operatorData: {
        level: number;
        favor: number;
        potentialRank: number;
        equipId: string;
        equipLevel: number;
        skillId: string;
    },
    phaseIndex: number,
) {
    const charData = char;
    const phaseData = charData.phases[phaseIndex];
    let attributesKeyFrames: Record<string, number> = {};
    const buffs = {
        atk_scale: 1,
        def_scale: 1,
        heal_scale: 1,
        damage_scale: 1,
        maxTarget: 1,
        times: 1,
        edef: 0,
        edef_scale: 1,
        edef_pene: 0,
        edef_pene_scale: 0,
        emr_pene: 0,
        emr: 0,
        emr_scale: 1,
        atk: 0,
        def: 0,
        attackSpeed: 0,
        maxHp: 0,
        baseAttackTime: 0,
        spRecoveryPerSec: 0,
        spRecoverRatio: 0,
        spRecoverIntervals: [],
        applied: {},
    };
    const buffList = {};

    if (operatorData.level === charData.phases[phaseIndex]?.maxLevel) {
        attributesKeyFrames = Object.assign(attributesKeyFrames, phaseData?.attributesKeyFrames[1]?.data);
    } else {
        AttributeKeys.forEach((key) => {
            attributesKeyFrames[key] = getAttribute(phaseData?.attributesKeyFrames ?? [], operatorData.level, 1, key);
        });
    }
    if (charData.favorKeyFrames && (charData.profession as unknown as string) !== "TOKEN") {
        const favorLevel = Math.floor(Math.min(operatorData.favor, 100) / 2);
        AttributeKeys.forEach((key) => {
            Object.assign(attributesKeyFrames, {
                [key]: (attributesKeyFrames[key] ?? 0) + getAttribute(charData.favorKeyFrames, favorLevel, 0, key),
            });

            Object.assign(buffs, {
                [key]: 0,
            });
        });
    }

    applyPotential(charData, operatorData.potentialRank, attributesKeyFrames);
    if (operatorData.equipId && phaseIndex >= 2) {
        applyEquip(
            char,
            {
                equipId: operatorData.equipId,
                equipLevel: operatorData.equipLevel,
                potentialRank: operatorData.potentialRank,
            },
            attributesKeyFrames,
        );

        Object.assign(buffList, {
            [operatorData.equipId]: attributesKeyFrames.equip_blackboard,
        });
    }

    if (charData.trait) {
        Object.assign(charData, {
            has_trait: true,
        });
        charData.talents.push(charData.trait as unknown as Operator["talents"][0]);
    }
    if (charData.talents) {
        charData.talents.forEach((talentData) => {
            if (talentData.candidates) {
                // mon3tr!!
                for (let i = talentData.candidates.length - 1; i >= 0; i--) {
                    const cd = talentData.candidates[i];
                    if (phaseIndex >= operatorPhaseToNumber(cd?.unlockCondition.phase ?? OperatorPhase.ELITE_0) && operatorData.level >= (cd?.unlockCondition.level ?? 0) && operatorData.potentialRank >= (cd?.requiredPotentialRank ?? 0)) {
                        const blackboard = getBlackboard(cd?.blackboard ?? []);
                        if (!cd?.prefabKey || Number(cd?.prefabKey) < 0) {
                            Object.assign(cd ?? {}, {
                                prefabKey: "trait",
                                name: "特性",
                            });
                        }
                        const prefabKey = "tachr_" + char.id?.slice(5) + "_" + cd?.prefabKey;

                        if (attributesKeyFrames.equip_blackboard) {
                            const ebb = attributesKeyFrames.equip_blackboard;
                            if (
                                String(
                                    (
                                        ebb as unknown as {
                                            override_talent: string | number;
                                        }
                                    ).override_talent,
                                ) === String(cd?.prefabKey)
                            ) {
                                const tb = (
                                    ebb as unknown as {
                                        talent: Record<string, number>;
                                    }
                                ).talent;
                                (
                                    ebb as unknown as {
                                        remove_keys: string[];
                                    }
                                ).remove_keys.forEach((k) => {
                                    delete tb[k];
                                });
                                Object.keys(tb).forEach((k) => {
                                    blackboard[k] = tb[k] ?? 0;
                                });
                            }
                            if (
                                cd?.prefabKey == "trait" &&
                                (
                                    ebb as unknown as {
                                        override_trait: string | number;
                                    }
                                ).override_trait
                            ) {
                                const tb = (
                                    ebb as unknown as {
                                        trait: Record<string, number>;
                                    }
                                ).trait;
                                Object.keys(tb).forEach((k) => {
                                    blackboard[k] = tb[k] ?? 0;
                                });
                            }
                        }
                        Object.assign(buffList, {
                            [prefabKey]: blackboard,
                        });
                        break;
                    }
                }
            }
        }); // foreach
    }

    /* For Ling specifically wow
    // 令3
    if (operatorData.skillId == "skchr_ling_3" && char.options.ling_fusion && char.options.token) {
      log.write("“弦惊” - 高级形态: 添加合体Buff");
      buffList["fusion_buff"] = checkSpecs(char.skillId, "fusion_buff");
      displayNames["fusion_buff"] = "高级形态";
    }
    */

    return {
        basic: attributesKeyFrames,
        buffs: buffs,
        buffList: buffList,
        char: char,
    };
}
