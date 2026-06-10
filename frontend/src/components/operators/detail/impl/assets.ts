import { env } from "#/env";
import type { IEnrichedSkill, IModule } from "#/types/operators";

type AssetServer = "en" | "cn" | undefined;

export function asset(path: string | null | undefined, server?: AssetServer): string {
    if (!path) return "";
    const prefix = server && server !== "en" ? `${server}/` : "";
    return `${env.VITE_BACKEND_URL}/api/${prefix}assets${path}`;
}

export function operatorHero(operatorId: string, skin: string | null, portrait: string | null, server?: AssetServer): string {
    return asset(skin ?? portrait ?? `/textures/chararts/${operatorId}/${operatorId}_2.png`, server);
}

export function operatorElite0(opId: string, skin: string | null, portrait: string | null, server?: AssetServer): string {
    const path = (skin ?? portrait ?? `/textures/chararts/${opId}/${opId}_1.png`).replace(/_2\.png$/, "_1.png");
    return asset(path, server);
}

export function operatorElite2(opId: string, skin: string | null, portrait: string | null, server?: AssetServer): string {
    const path = (skin ?? portrait ?? `/textures/chararts/${opId}/${opId}_2.png`).replace(/_1\.png$/, "_2.png");
    return asset(path, server);
}

export function skinTexture(opId: string, skinId: string, server?: AssetServer): string {
    // Skin ids without `@` are elite/default-art variants (e.g. `char_002_amiya#1+`)
    // and live under `/textures/chararts/`. Skins with `@` are alternate outfits
    // (e.g. `char_002_amiya@winter#1`) and live under `/textures/skinpack/`.
    if (skinId.includes("@")) {
        const file = skinId.replaceAll("@", "_").replaceAll("#", "%23");
        return asset(`/textures/skinpack/${opId}/${file}.png`, server);
    }
    if (skinId.includes("#")) {
        const file = skinId.replace("#", "_");
        return asset(`/textures/chararts/${opId}/${file}.png`, server);
    }
    return asset(`/textures/chararts/${opId}/${skinId}.png`, server);
}

export function campLogo(id: string, server?: AssetServer): string {
    return asset(`/textures/spritepack/ui_camp_logo_0/logo_${id}.png`, server);
}

export function eliteIcon(elite: number, server?: AssetServer): string {
    return asset(`/textures/arts/elite_hub/elite_${elite}.png`, server);
}

export function potentialIcon(rank: number, server?: AssetServer): string {
    return asset(`/textures/arts/potential_hub/potential_${rank}.png`, server);
}

export function itemIcon(id: string, iconId: string | null | undefined, image: string | null | undefined, server?: AssetServer): string {
    if (image) return asset(image, server);
    const prefix = server && server !== "en" ? `${server}/` : "";
    return `${env.VITE_BACKEND_URL}/api/${prefix}item-icon/${encodeURIComponent(iconId ?? id)}`;
}

export function audioURL(relativeURL: string, server?: AssetServer): string {
    const prefix = server && server !== "en" ? `${server}/` : "";
    return `${env.VITE_BACKEND_URL}/api/${prefix}assets/audio${relativeURL}`;
}

export function skillIconURL(skill: IEnrichedSkill, server?: AssetServer): string {
    if (skill.static?.image) return asset(skill.static.image, server);
    const id = skill.static?.iconId ?? skill.static?.skillId ?? skill.skillId;
    return asset(`/textures/skill-icons/${id}.png`, server);
}

export function moduleIconURL(mod: IModule, server?: AssetServer): string {
    if (mod.image) return asset(mod.image, server);
    return asset(`/textures/spritepack/ui_equip_big_img_hub_0/${mod.uniEquipIcon}.png`, server);
}

export function specializedIcon(level: number): string {
    return asset(`/textures/arts/specialized_hub/specialized_${level}.png`);
}

export function baseSkillIcon(skillIconStem: string, server?: AssetServer): string {
    if (!skillIconStem) return "";
    return asset(`/textures/spritepack/building_ui_buff_skills_h1_0/${skillIconStem}.png`, server);
}
