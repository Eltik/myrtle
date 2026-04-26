import { env } from "#/env";

export function asset(path: string | null | undefined, fallback?: string): string {
    if (!path && !fallback) return "";
    return `${env.VITE_BACKEND_URL}/api/assets${path ?? fallback}`;
}

export function operatorHero(operatorId: string, skin: string | null, portrait: string | null): string {
    return asset(skin ?? portrait ?? `/textures/chararts/${operatorId}/${operatorId}_2.png`);
}

export function campLogo(id: string): string {
    return asset(`/textures/spritepack/ui_camp_logo_0/logo_${id}.png`);
}
