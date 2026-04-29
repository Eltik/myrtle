import type { IDisplaySkin, ISkin } from "#/lib/api/skins";
import { operatorElite0, operatorElite2, skinTexture } from "./assets";

export interface IUISkin {
    id: string;
    name: string;
    /** Heading shown above the skin name in the side card. */
    kicker: string;
    /** Sub text shown below the skin name in the side card. */
    sub: string;
    image: string;
    thumbnail: string;
    isDefault: boolean;
    displaySkin?: IDisplaySkin;
}

interface IBuildArgs {
    skinsFromBackend: ISkin[];
    operatorId: string;
    operatorSkin: string | null;
    operatorPortrait: string | null;
    phasesLength: number;
    artistFallback?: string;
}

export function buildOperatorSkinList({ skinsFromBackend, operatorId, operatorSkin, operatorPortrait, phasesLength, artistFallback }: IBuildArgs): IUISkin[] {
    const skins: IUISkin[] = [];

    const e0 = operatorElite0(operatorId, operatorSkin, operatorPortrait);
    const e2 = operatorElite2(operatorId, operatorSkin, operatorPortrait);

    skins.push({
        id: `${operatorId}_default`,
        name: "Default",
        kicker: phasesLength > 2 ? "Elite 0 / Elite 1" : "Elite 0",
        sub: artistFallback ? `Artist · ${artistFallback}` : "Unlocked by default",
        image: e0,
        thumbnail: e0,
        isDefault: true,
    });

    if (phasesLength > 2) {
        skins.push({
            id: `${operatorId}_e2`,
            name: "Evolved Art",
            kicker: "Elite 2",
            sub: artistFallback ? `Artist · ${artistFallback}` : "Elite 2 Promotion",
            image: e2,
            thumbnail: e2,
            isDefault: false,
        });
    }

    for (const skin of skinsFromBackend) {
        const id = skin.skinId;
        const isDefault = !id?.includes("@") && (id?.endsWith("#1") || id?.endsWith("#2"));
        if (!id || isDefault) continue;

        const display = skin.displaySkin;
        const tex = skinTexture(operatorId, id);
        skins.push({
            id,
            name: display?.skinName ?? "Outfit",
            kicker: display?.skinGroupName ?? "Skin",
            sub: artistFallback ? `Artist · ${artistFallback}` : (display?.obtainApproach ?? "Special Outfit"),
            image: tex,
            thumbnail: tex,
            isDefault: false,
            displaySkin: display,
        });
    }

    return skins;
}

export function chibiSkinKey(skinId: string): string {
    if (!skinId) return "default";
    if (skinId.endsWith("_default") || skinId.endsWith("_e2")) return "default";
    const at = skinId.indexOf("@");
    return at !== -1 ? skinId.slice(at + 1) : "default";
}
