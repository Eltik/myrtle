import { asset, eliteIcon, potentialIcon } from "#/components/operators/detail/impl/assets";
import type { IEnrichedSkill, IOperatorModule } from "#/types/operators";

export { asset, eliteIcon, potentialIcon };

export function skillIconURL(skill: IEnrichedSkill): string {
    if (skill.static?.image) return asset(skill.static.image);
    const id = skill.static?.iconId ?? skill.static?.skillId ?? skill.skillId;
    return asset(`/textures/skill-icons/${id}.png`);
}

export function moduleIconURL(mod: IOperatorModule): string {
    if (mod.image) return asset(mod.image);
    return asset(`/textures/spritepack/ui_equip_big_img_hub_0/${mod.uniEquipIcon}.png`);
}

export function specializedIcon(level: number): string {
    return asset(`/textures/arts/specialized_hub/specialized_${level}.png`);
}
