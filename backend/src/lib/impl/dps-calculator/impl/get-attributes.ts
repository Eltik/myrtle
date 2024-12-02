import { AttributeKeys } from "..";
import { Operator, OperatorPhase } from "../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
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
    let attributesKeyFrames: { [key: string]: any } = {};
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
    if (char.id?.startsWith("token")) console.write("【召唤物属性】");
    else console.write("【基础属性】");

    console.write("----");
    if (operatorData.level === charData.phases[phaseIndex].maxLevel) {
        attributesKeyFrames = Object.assign(attributesKeyFrames, phaseData.attributesKeyFrames[1].data);
    } else {
        AttributeKeys.forEach((key) => {
            attributesKeyFrames[key] = getAttribute(phaseData.attributesKeyFrames, operatorData.level, 1, key);
        });
    }
    if (charData.favorKeyFrames && (charData.profession as unknown as string) !== "TOKEN") {
        // token不计信赖
        const favorLevel = Math.floor(Math.min(operatorData.favor, 100) / 2);
        AttributeKeys.forEach((key) => {
            attributesKeyFrames[key] += getAttribute(charData.favorKeyFrames, favorLevel, 0, key);

            Object.assign(buffs, {
                [key]: 0,
            });
        });
    }

    // 计算潜能和模组
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

    // 计算天赋/特性，记为Buff
    if (charData.trait) {
        Object.assign(charData, {
            has_trait: true,
        });
        charData.talents.push(charData.trait as unknown as any);
    }
    if (charData.talents) {
        charData.talents.forEach((talentData) => {
            if (talentData.candidates) {
                // mon3tr!!
                for (let i = talentData.candidates.length - 1; i >= 0; i--) {
                    const cd = talentData.candidates[i];
                    if (phaseIndex >= operatorPhaseToNumber(cd.unlockCondition.phase) && operatorData.level >= cd.unlockCondition.level && operatorData.potentialRank >= cd.requiredPotentialRank) {
                        // 找到了当前生效的天赋
                        const blackboard = getBlackboard(cd.blackboard);
                        if (!cd.prefabKey || Number(cd.prefabKey) < 0) {
                            cd.prefabKey = "trait"; // trait as talent
                            cd.name = "特性";
                        }
                        const prefabKey = "tachr_" + char.id?.slice(5) + "_" + cd.prefabKey;

                        // 如果天赋被模组修改，覆盖对应面板
                        if (attributesKeyFrames.equip_blackboard) {
                            const ebb = attributesKeyFrames.equip_blackboard;
                            if (ebb.override_talent == cd.prefabKey) {
                                const tb = ebb.talent;
                                ebb.remove_keys.forEach((k: string) => {
                                    delete tb[k];
                                });
                                Object.keys(tb).forEach((k) => {
                                    blackboard[k] = tb[k];
                                });
                                console.write(`[模组] 强化天赋 - ${cd.name}: ${JSON.stringify(blackboard)}`);
                            }
                            if (cd.prefabKey == "trait" && ebb.override_trait) {
                                const tb = ebb.trait;
                                Object.keys(tb).forEach((k) => {
                                    blackboard[k] = tb[k];
                                });
                                console.write(`[模组] 强化特性: ${JSON.stringify(blackboard)}`);
                            }
                        }
                        // bufflist处理
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
