import type { Skin, SkinData } from "~/types/api/impl/skin";
import type { UISkin } from "./types";

export function formatSkinsForOperator(skinData: SkinData | Skin[], operatorId: string, operatorSkin: string | undefined, operatorPortrait: string | undefined, phasesLength: number): UISkin[] {
    const skins: UISkin[] = [];

    // Prefer full character art (skin) over portrait (headshot)
    const skinPath = operatorSkin ? `/api/cdn${operatorSkin}` : null;
    const portraitPath = operatorPortrait ? `/api/cdn${operatorPortrait}` : null;

    // Use full character art if available, otherwise fall back to portrait
    const basePath = skinPath ?? portraitPath;
    const e0Path = basePath?.replace(/_2\.png$/, "_1.png") ?? `/api/cdn/upk/chararts/${operatorId}/${operatorId}_1.png`;
    const e2Path = basePath?.replace(/_1\.png$/, "_2.png") ?? `/api/cdn/upk/chararts/${operatorId}/${operatorId}_2.png`;

    // Add default skin (E0/E1 art)
    skins.push({
        id: `${operatorId}_default`,
        name: "Default",
        image: e0Path,
        thumbnail: e0Path,
        isDefault: true,
    });

    // Add E2 art if available (phases > 2 means E2 exists)
    if (phasesLength > 2) {
        skins.push({
            id: `${operatorId}_e2`,
            name: "Elite 2",
            image: e2Path,
            thumbnail: e2Path,
            isDefault: false,
        });
    }

    // Process additional skins from API
    if (Array.isArray(skinData)) {
        for (const skin of skinData) {
            // Use skinId (standard Skin type) - EnrichedSkin also has skinId via extension
            const skinIdentifier = skin.skinId;
            // Skip default skins (those with #1 or #2 suffixes which are E0/E1 and E2 arts)
            // BUT don't skip special skins that contain '@' (e.g., char_332_archet@shining#1)
            const isDefaultSkin = !skinIdentifier?.includes("@") && (skinIdentifier?.endsWith("#1") || skinIdentifier?.endsWith("#2"));
            if (skinIdentifier && !isDefaultSkin) {
                // Format the skin path for CDN
                // skinId format: "char_002_amiya@epoque#4" -> file: "char_002_amiya_epoque#4.png"
                // Replace @ with _ and encode # as %23 (# is a URL fragment identifier and won't be sent to server)
                const formattedSkinId = skinIdentifier.replace(/@/g, "_").replace(/#/g, "%23");

                skins.push({
                    id: skinIdentifier,
                    name: skin.displaySkin?.skinName ?? "Outfit",
                    image: `/api/cdn/upk/skinpack/${operatorId}/${formattedSkinId}.png`,
                    thumbnail: `/api/cdn/upk/skinpack/${operatorId}/${formattedSkinId}.png`,
                    displaySkin: skin.displaySkin
                        ? {
                              skinName: skin.displaySkin.skinName ?? undefined,
                              modelName: skin.displaySkin.modelName,
                              drawerList: skin.displaySkin.drawerList,
                              designerList: skin.displaySkin.designerList ?? undefined,
                              obtainApproach: skin.displaySkin.obtainApproach ?? undefined,
                          }
                        : undefined,
                    isDefault: false,
                });
            }
        }
    }

    return skins;
}
