import type { Operator } from "../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { checkSpecs } from "./check-specs";

export function extractDamageType(charData: Operator, chr: Operator, isSkill: boolean, skillDesc: string, skillBlackboard: any, options: any) {
    const charId = chr.id ?? "";
    const spId = charData.subProfessionId;
    const skillId = skillBlackboard.id;
    let ret: number | boolean = 0;
    if (charData.profession == "MEDIC" && spId != "incantationmedic") ret = 2;
    else if (spId == "bard") {
        ret = 2;
    } else if (options.annie) {
        ret = 1;
    } else if (charData.description.includes("法术伤害") && spId != "artsprotector") {
        ret = 1;
    }
    if (isSkill) {
        if (["法术伤害", "法术</>伤害", "伤害类型变为"].some((x) => skillDesc.includes(x))) ret = 1;
        else if (["治疗", "恢复", "每秒回复"].some((x) => skillDesc.includes(x)) && !skillBlackboard["hp_recovery_per_sec_by_max_hp_ratio"]) {
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
            if (skillId == "skchr_ling_3" && (chr as unknown as { options: any }).options.ling_fusion) ret = 1;
        }
    } else if (true) {
        // Originally if (chr.options.token)
        ret = checkSpecs(charId, "token_damage_type") || ret;
        if (["skchr_mgllan_3"].includes(skillId)) ret = 0;
        else if (skillId == "skchr_ling_2" || (skillId == "skchr_ling_3" && (chr as unknown as { options: any }).options.ling_fusion)) ret = 1;
    }
    return ~~ret;
}
