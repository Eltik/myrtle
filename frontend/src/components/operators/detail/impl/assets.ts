import { env } from "#/env";
import type { IEnrichedSkill, IModule } from "#/types/operators";

export function asset(path: string | null | undefined, fallback?: string): string {
    if (!path && !fallback) return "";
    return `${env.VITE_BACKEND_URL}/api/assets${path ?? fallback}`;
}

export function operatorHero(operatorId: string, skin: string | null, portrait: string | null): string {
    return asset(skin ?? portrait ?? `/textures/chararts/${operatorId}/${operatorId}_2.png`);
}

export function operatorElite0(opId: string, skin: string | null, portrait: string | null): string {
    const path = (skin ?? portrait ?? `/textures/chararts/${opId}/${opId}_1.png`).replace(/_2\.png$/, "_1.png");
    return asset(path);
}

export function operatorElite2(opId: string, skin: string | null, portrait: string | null): string {
    const path = (skin ?? portrait ?? `/textures/chararts/${opId}/${opId}_2.png`).replace(/_1\.png$/, "_2.png");
    return asset(path);
}

export function skinTexture(opId: string, skinId: string): string {
    const file = skinId.replaceAll("@", "_").replaceAll("#", "%23");
    return asset(`/textures/skinpack/${opId}/${file}.png`);
}

export function campLogo(id: string): string {
    return asset(`/textures/spritepack/ui_camp_logo_0/logo_${id}.png`);
}

export function eliteIcon(elite: number): string {
    return asset(`/textures/arts/elite_hub/elite_${elite}.png`);
}

export function potentialIcon(rank: number): string {
    return asset(`/textures/arts/potential_hub/potential_${rank}.png`);
}

export function itemIcon(id: string, iconId: string | null | undefined, image: string | null | undefined): string {
    if (image) return asset(image);
    return asset(`/textures/arts/items/icons/${iconId ?? id}.png`);
}

export function voiceAudio(voiceURL: string): string {
    return `${env.VITE_BACKEND_URL}/api/assets/audio${voiceURL}`;
}

export function skillIconURL(skill: IEnrichedSkill): string {
    if (skill.static?.image) return asset(skill.static.image);
    const id = skill.static?.iconId ?? skill.static?.skillId ?? skill.skillId;
    return asset(`/textures/skill-icons/${id}.png`);
}

export function moduleIconURL(mod: IModule): string {
    if (mod.image) return asset(mod.image);
    return asset(`/textures/spritepack/ui_equip_big_img_hub_0/${mod.uniEquipIcon}.png`);
}

export function specializedIcon(level: number): string {
    return asset(`/textures/arts/specialized_hub/specialized_${level}.png`);
}
