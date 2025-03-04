import { FEXLI_REPOSITORY, RESOURCE_REPOSITORY } from "../..";
import type { SkinData, Skin } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/skins";
import { STATIC_DATA } from "../../../handler";

export const getAll = (): SkinData => {
    const data = STATIC_DATA?.SKIN_TABLE as SkinData;
    data.charSkins = Object.fromEntries(Object.entries(data.charSkins).map(([id, skin]) => [id, { ...skin, images: getSkinImages(skin) }]));
    return data;
};

const getSkinImages = (skin: Skin) => {
    // Transform skinId to match repository format
    const formatSkinId = (skinId: string) => {
        // Extract the base character ID and skin code
        const match = skinId.match(/^(char_\d+_\w+)(@\w+)?#(\d+)$/);

        if (!match) return `${skinId}.png`; // Fallback

        const [, charBase, skinType, skinNumber] = match;

        let formattedId;
        // Format depends on whether it has a skin type (like @epoque)
        if (skinType) {
            // Handle special skin types by replacing @ with _ and replacing # with _
            formattedId = `${charBase}_${skinType.substring(1)}_${skinNumber}`;
            // Add 'b' suffix for non-base skins (not #1)
            if (skinNumber !== "1") {
                formattedId += "b";
            }
            formattedId += ".png";
        } else {
            // Handle standard skins by adding _ before number
            formattedId = `${charBase}_${skinNumber}`;
            // Add 'b' suffix for non-base skins (not #1)
            if (skinNumber !== "1") {
                formattedId += "b";
            }
            formattedId += ".png";
        }

        return formattedId;
    };

    return {
        avatar: `https://raw.githubusercontent.com/${RESOURCE_REPOSITORY}/main/avatar/${skin.avatarId}.png`,
        portrait: `https://raw.githubusercontent.com/${RESOURCE_REPOSITORY}/main/portrait/${skin.portraitId}.png`,
        skin: `https://raw.githubusercontent.com/${FEXLI_REPOSITORY}/main/charpack/${formatSkinId(skin.skinId)}`,
    };
};

export default (charId: string): Skin[] => {
    const data = getAll();
    const skins = Object.entries(data.charSkins).map(([id, data]) => ({
        id,
        ...data,
        images: getSkinImages(data),
    }));
    return skins.filter((skin) => skin.charId === charId);
};
