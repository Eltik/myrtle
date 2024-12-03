import { OperatorProfession, type Operator } from "~/types/impl/api/static/operator";
import { checkSpecs } from "./check-specs";
import type { CalculateDPSParams, SkillBuff } from "~/types/impl/frontend/impl/dps-calculator";

export function extractDamageType(charData: Operator, chr: Operator, isSkill: boolean, skillDesc: string, skillBlackboard: SkillBuff, options: CalculateDPSParams["options"]) {
    const charId = chr.id ?? "";
    const spId = charData.subProfessionId;
    const skillId = skillBlackboard.id;
    let ret: number | boolean = 0;
    if (charData.profession === OperatorProfession.MEDIC && spId !== "incantationmedic") ret = 2;
    else if (spId == "bard") {
        ret = 2;
    } else if (options.annie) {
        ret = 1;
    } else if (charData.description.includes("法术伤害") && spId != "artsprotector") {
        ret = 1;
    }
    if (isSkill) {
        /**
         * @todo: Convert to English
         */
        if (["法术伤害", "法术</>伤害", "伤害类型变为"].some((x) => skillDesc.includes(x))) ret = 1;
        else if (
            ["治疗", "恢复", "每秒回复"].some((x) => skillDesc.includes(x)) &&
            !(
                skillBlackboard as unknown as {
                    hp_recovery_per_sec_by_max_hp_ratio: number;
                }
            ).hp_recovery_per_sec_by_max_hp_ratio
        ) {
            ret = 2;
        }
        // special character/skill overrides
        ret = checkSpecs(charId, "damage_type") || checkSpecs(skillId, "damage_type") || ret;
        if (skillId == "skchr_nearl2_3") {
            ret = options.block ? 3 : 0;
        }
        if (options.token) {
            const _r = checkSpecs(skillId, "token_damage_type");
            if (_r != null) ret = _r;
            if (skillId == "skchr_ling_3" && options.ling_fusion) ret = 1;
        }
    } else if (options.token) {
        ret = checkSpecs(charId, "token_damage_type") || ret;
        if (["skchr_mgllan_3"].includes(skillId)) ret = 0;
        else if (skillId == "skchr_ling_2" || (skillId == "skchr_ling_3" && options.ling_fusion)) ret = 1;
    }
    return ~~ret;
}
