import type { Skin, SkinData } from "~/types/api/impl/skin";
import type { UISkin } from "./types";

export function formatSkinsForOperator(skinData: SkinData | Skin[], operatorId: string, operatorSkin: string | undefined, operatorPortrait: string | undefined, phasesLength: number): UISkin[] {
    const skins: UISkin[] = [];

    const skinPath = operatorSkin ? `/api/cdn${operatorSkin}` : null;
    const portraitPath = operatorPortrait ? `/api/cdn${operatorPortrait}` : null;
    const basePath = skinPath ?? portraitPath;
    const e0Path = basePath?.replace(/_2\.png$/, "_1.png") ?? `/api/cdn/upk/chararts/${operatorId}/${operatorId}_1.png`;
    const e2Path = basePath?.replace(/_1\.png$/, "_2.png") ?? `/api/cdn/upk/chararts/${operatorId}/${operatorId}_2.png`;

    skins.push({
        id: `${operatorId}_default`,
        name: "Default",
        image: e0Path,
        thumbnail: e0Path,
        isDefault: true,
    });

    if (phasesLength > 2) {
        skins.push({
            id: `${operatorId}_e2`,
            name: "Elite 2",
            image: e2Path,
            thumbnail: e2Path,
            isDefault: false,
        });
    }

    if (Array.isArray(skinData)) {
        for (const skin of skinData) {
            const skinIdentifier = skin.skinId;
            // Skip default skins (those with #1/#2 suffixes) but not special skins containing '@'
            const isDefaultSkin = !skinIdentifier?.includes("@") && (skinIdentifier?.endsWith("#1") || skinIdentifier?.endsWith("#2"));
            if (skinIdentifier && !isDefaultSkin) {
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
                              colorList: skin.displaySkin.colorList?.filter((c) => c !== "") ?? undefined,
                              content: skin.displaySkin.content ?? undefined,
                              description: skin.displaySkin.description ?? undefined,
                              dialog: skin.displaySkin.dialog ?? undefined,
                              skinGroupId: skin.displaySkin.skinGroupId ?? undefined,
                              skinGroupName: skin.displaySkin.skinGroupName ?? undefined,
                              skinGroupSortIndex: skin.displaySkin.skinGroupSortIndex?.toString() ?? undefined,
                              usage: skin.displaySkin.usage ?? undefined,
                              titleList: skin.displaySkin.titleList?.filter((t) => t !== "") ?? undefined,
                              getTime: skin.displaySkin.getTime ?? undefined,
                              onYear: skin.displaySkin.onYear ?? undefined,
                              onPeriod: skin.displaySkin.onPeriod ?? undefined,
                              sortId: skin.displaySkin.sortId ?? undefined,
                          }
                        : undefined,
                    isDefault: false,
                });
            }
        }
    }

    return skins;
}
