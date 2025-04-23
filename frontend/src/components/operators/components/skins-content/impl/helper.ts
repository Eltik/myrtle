import type { Skin } from "~/types/impl/api/static/skins";
import type { UISkin } from "~/types/impl/frontend/impl/operators";
import type { ChibisSimplified } from "~/types/impl/api/impl/chibis";

// Fetch skins data from API
export async function fetchSkins(id: string) {
    const data = (await (
        await fetch("/api/static", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                type: "skins",
                id,
            }),
        })
    ).json()) as { skins: Skin[] };
    return data.skins;
}

// Fetch chibi data from API
export async function fetchChibi(id: string) {
    const response = await fetch("/api/chibis", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            format: "simplified",
            id,
        }),
    });

    return (await response.json()) as ChibisSimplified[];
}

// Convert API skins to UI skins
export function convertToUISkins(skins: Skin[]): UISkin[] {
    return skins.map((skin) => ({
        id: skin.skinId,
        name: skin.displaySkin.skinName ?? skin.displaySkin.skinGroupName ?? "Default",
        description: skin.displaySkin.description ?? skin.displaySkin.content ?? "Default",
        image: skin.images.skin,
        obtainMethod: skin.displaySkin.obtainApproach ?? "Default",
        releaseDate: skin.displaySkin.getTime ? new Date(skin.displaySkin.getTime * 1000).toLocaleDateString() : "Default",
        artists: skin.displaySkin.drawerList ?? [],
        voiceLines: skin.voiceId !== null,
        animations: (skin.dynIllustId?.length ?? 0) > 0,
        available: skin.isBuySkin,
        isDefault: skin.displaySkin.skinGroupName === "Default Outfit",
    }));
}
