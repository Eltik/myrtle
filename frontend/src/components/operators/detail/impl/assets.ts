import { env } from "#/env";

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

export function voiceAudio(voiceUrl: string): string {
    return `${env.VITE_BACKEND_URL}/api/assets/audio${voiceUrl}`;
}
