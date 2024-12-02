import { Operator } from "../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { checkSpecs } from "./check-specs";
import { getBlackboard } from "./get-blackboard";

export function applyEquip(
    char: Operator,
    operatorData: {
        equipId: string;
        equipLevel: number;
        potentialRank: number;
    },
    basic: { [key: string]: number },
) {
    const equipId = operatorData.equipId;
    const phase = operatorData.equipLevel - 1;
    let cand = 0;
    const blackboard = {};
    let attr = {};

    const moduleData = char.modules.find((module) => module.id === equipId);

    if (equipId && moduleData) {
        const item = moduleData.data.phases[phase];
        attr = getBlackboard(item.attributeBlackboard);
        Object.assign(blackboard, {
            attr: attr,
        });

        if (item.tokenAttributeBlackboard) {
            const tb = {};
            Object.keys(item.tokenAttributeBlackboard).forEach((tok) => {
                Object.assign(tb, {
                    [tok]: getBlackboard(item.tokenAttributeBlackboard[tok]),
                });
            });
            Object.assign(blackboard, {
                token: tb,
            });
        }
        let talents = {},
            traits = {};

        item.parts.forEach((pt) => {
            const talentBundle = pt.addOrOverrideTalentDataBundle;
            const traitBundle = pt.overrideTraitDataBundle;
            // 天赋变更
            if (talentBundle && talentBundle.candidates) {
                for (cand = talentBundle.candidates.length - 1; cand > 0; --cand) {
                    if (operatorData.potentialRank >= talentBundle.candidates[cand].requiredPotentialRank) break;
                }
                talents = {
                    ...talents,
                    ...getBlackboard(talentBundle.candidates[cand].blackboard),
                };
            }
            // 特性变更
            if (traitBundle && traitBundle.candidates) {
                for (cand = traitBundle.candidates.length - 1; cand > 0; --cand) {
                    if (operatorData.potentialRank >= traitBundle.candidates[cand].requiredPotentialRank) break;
                }
                traits = {
                    ...traits,
                    ...getBlackboard(traitBundle.candidates[cand].blackboard),
                };
            }
        });
        Object.assign(blackboard, {
            talent: talents,
            trait: traits,
        });
        // 查询额外数据，获得修改的是哪个天赋的面板
        // which = 1, 2, "1+"
        const which = checkSpecs(equipId, "override_talent");
        if (which && which.toString().length > 0 && operatorData.equipLevel > 1)
            Object.assign(blackboard, {
                override_talent: which.toString(),
                override_trait: checkSpecs(equipId, "override_trait"),
                remove_keys: checkSpecs(equipId, "remove_keys") || [],
            });
    }
    //  console.log(attr, blackboard);
    const attrKeys = {
        max_hp: "maxHp",
        atk: "atk",
        def: "def",
        magic_resistance: "magicResistance",
        attack_speed: "attackSpeed",
        block_cnt: "blockCnt",
    };

    if (!false) {
        // Originally if (!char.options.token), need to fix this
        Object.keys(attr).forEach((x) => {
            basic[attrKeys[x as keyof typeof attrKeys]] += attr[x as keyof typeof attr];
            if (attr[x as keyof typeof attr] !== 0)
                console.write(`模组 Lv${operatorData.equipLevel}: ${attrKeys[x as keyof typeof attrKeys]} ${basic[attrKeys[x as keyof typeof attrKeys]] - attr[x as keyof typeof attr]} -> ${basic[attrKeys[x as keyof typeof attrKeys]]} (+${attr[x as keyof typeof attr]})`);
        });
    }

    Object.assign(basic, {
        equip_blackboard: blackboard,
    });
}
