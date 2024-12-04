import type { Operator } from "../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import type { Skill } from "../../../../../types/impl/lib/impl/local/impl/gamedata/impl/skills";

export function transformSkill(skill: Operator["skills"][0]): Skill {
    return {
        id: skill.skillId,
        skillId: skill.skillId,
        hidden: skill.static?.hidden ?? false,
        iconId: skill.static?.iconId ?? "",
        levels: (skill.static?.levels ?? []).map((level) => ({
            blackboard: level.blackboard,
            description: level.description,
            duration: level.duration,
            name: level.name,
            prefabId: level.prefabId,
            rangeId: level.rangeId,
            skillType: String(level.skillType),
            spData: {
                spType: level.spData?.spType ?? "normal",
                levelUpCost: null,
                maxChargeTime: level.spData?.maxChargeTime ?? 0,
                spCost: level.spData?.spCost ?? 0,
                initSp: level.spData?.initSp ?? 0,
                increment: level.spData?.increment ?? 0,
            },
        })),
    };
}
