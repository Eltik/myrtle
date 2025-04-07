import type { FormattedChibis } from "~/types/impl/frontend/impl/chibis";

/*
export const getAssetURL = (repoBaseURL: string, path: string) => {
    // Remove the initial "./" if present
    const normalizedPath = path.startsWith("./") ? path.substring(2) : path;
    return `${repoBaseURL}${normalizedPath}`;
};
*/

export const getAssetURL = (repoBaseURL: string, path: string) => {
    console.log("Getting asset URL", { repoBaseURL, path });
    const plannerId = 'reedalter2';
    const perspective = 'front';
    const skinId = 'char_1020_reed2_summer_17';
    const chibiExntension = '.skel';
    return `https://raw.githubusercontent.com/HermitzPlanner/${'chibi-assets'}/main/${plannerId}/${perspective}/${skinId}${chibiExntension}`
};

export const getSkinData = (selectedOperator: FormattedChibis | null, selectedSkin: string | null, repoBaseURL: string, viewType: "dorm" | "front" | "back"): {
    atlas: string;
    png: string;
    skel: string;
    type: "dorm" | "front" | "back";
} | null => {
    if (!selectedOperator || !selectedSkin) {
        console.log("No operator or skin selected", { selectedOperator, selectedSkin });
        return null;
    }

    // Find the selected skin
    const skin = selectedOperator.skins.find((s) => s.dorm.path === selectedSkin || s.front.path === selectedSkin || s.back.path === selectedSkin);

    if (!skin) {
        console.log("Could not find matching skin", {
            selectedSkin,
            availableSkins: selectedOperator.skins.map((s) => ({
                dorm: s.dorm.path,
                front: s.front.path,
                back: s.back.path,
            })),
        });

        // Fallback: Try to use the first skin if available
        if (selectedOperator.skins.length > 0) {
            const fallbackSkin = selectedOperator.skins[0];
            if (fallbackSkin) {
                console.log("Using fallback skin", { fallbackSkin: fallbackSkin.name });

                // Try the selected view type first, then fall back to others
                if (viewType === "dorm" && fallbackSkin.dorm?.atlas && fallbackSkin.dorm?.png && fallbackSkin.dorm?.skel) {
                    return {
                        atlas: getAssetURL(repoBaseURL, fallbackSkin.dorm.atlas),
                        png: getAssetURL(repoBaseURL, fallbackSkin.dorm.png),
                        skel: getAssetURL(repoBaseURL, fallbackSkin.dorm.skel),
                        type: "dorm",
                    };
                } else if (viewType === "front" && fallbackSkin.front?.atlas && fallbackSkin.front?.png && fallbackSkin.front?.skel) {
                    return {
                        atlas: getAssetURL(repoBaseURL, fallbackSkin.front.atlas),
                        png: getAssetURL(repoBaseURL, fallbackSkin.front.png),
                        skel: getAssetURL(repoBaseURL, fallbackSkin.front.skel),
                        type: "front",
                    };
                } else if (viewType === "back" && fallbackSkin.back?.atlas && fallbackSkin.back?.png && fallbackSkin.back?.skel) {
                    return {
                        atlas: getAssetURL(repoBaseURL, fallbackSkin.back.atlas),
                        png: getAssetURL(repoBaseURL, fallbackSkin.back.png),
                        skel: getAssetURL(repoBaseURL, fallbackSkin.back.skel),
                        type: "back",
                    };
                }

                // If the selected view type isn't available, try any available view
                if (fallbackSkin.dorm?.atlas && fallbackSkin.dorm?.png && fallbackSkin.dorm?.skel) {
                    return {
                        atlas: getAssetURL(repoBaseURL, fallbackSkin.dorm.atlas),
                        png: getAssetURL(repoBaseURL, fallbackSkin.dorm.png),
                        skel: getAssetURL(repoBaseURL, fallbackSkin.dorm.skel),
                        type: "dorm",
                    };
                } else if (fallbackSkin.front?.atlas && fallbackSkin.front?.png && fallbackSkin.front?.skel) {
                    return {
                        atlas: getAssetURL(repoBaseURL, fallbackSkin.front.atlas),
                        png: getAssetURL(repoBaseURL, fallbackSkin.front.png),
                        skel: getAssetURL(repoBaseURL, fallbackSkin.front.skel),
                        type: "front",
                    };
                } else if (fallbackSkin.back?.atlas && fallbackSkin.back?.png && fallbackSkin.back?.skel) {
                    return {
                        atlas: getAssetURL(repoBaseURL, fallbackSkin.back.atlas),
                        png: getAssetURL(repoBaseURL, fallbackSkin.back.png),
                        skel: getAssetURL(repoBaseURL, fallbackSkin.back.skel),
                        type: "back",
                    };
                }
            }
        }

        return null;
    }

    // First, try to use the selected view type if it has all required assets
    if (viewType === "dorm" && skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
        console.log("Using dorm view (selected)", { path: skin.dorm.path });
        return {
            atlas: getAssetURL(repoBaseURL, skin.dorm.atlas),
            png: getAssetURL(repoBaseURL, skin.dorm.png),
            skel: getAssetURL(repoBaseURL, skin.dorm.skel),
            type: "dorm",
        };
    } else if (viewType === "front" && skin.front.atlas && skin.front.png && skin.front.skel) {
        console.log("Using front view (selected)", { path: skin.front.path });
        return {
            atlas: getAssetURL(repoBaseURL, skin.front.atlas),
            png: getAssetURL(repoBaseURL, skin.front.png),
            skel: getAssetURL(repoBaseURL, skin.front.skel),
            type: "front",
        };
    } else if (viewType === "back" && skin.back.atlas && skin.back.png && skin.back.skel) {
        console.log("Using back view (selected)", { path: skin.back.path });
        return {
            atlas: getAssetURL(repoBaseURL, skin.back.atlas),
            png: getAssetURL(repoBaseURL, skin.back.png),
            skel: getAssetURL(repoBaseURL, skin.back.skel),
            type: "back",
        };
    }

    // If the selected view type doesn't have all required assets, check if the path matches a specific view
    if (selectedSkin === skin.dorm.path && skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
        console.log("Using dorm view (path match)", { path: skin.dorm.path });
        return {
            atlas: getAssetURL(repoBaseURL, skin.dorm.atlas),
            png: getAssetURL(repoBaseURL, skin.dorm.png),
            skel: getAssetURL(repoBaseURL, skin.dorm.skel),
            type: "dorm",
        };
    } else if (selectedSkin === skin.front.path && skin.front.atlas && skin.front.png && skin.front.skel) {
        console.log("Using front view (path match)", { path: skin.front.path });
        return {
            atlas: getAssetURL(repoBaseURL, skin.front.atlas),
            png: getAssetURL(repoBaseURL, skin.front.png),
            skel: getAssetURL(repoBaseURL, skin.front.skel),
            type: "front",
        };
    } else if (selectedSkin === skin.back.path && skin.back.atlas && skin.back.png && skin.back.skel) {
        console.log("Using back view (path match)", { path: skin.back.path });
        return {
            atlas: getAssetURL(repoBaseURL, skin.back.atlas),
            png: getAssetURL(repoBaseURL, skin.back.png),
            skel: getAssetURL(repoBaseURL, skin.back.skel),
            type: "back",
        };
    }

    // If we get here, we found a skin but the selected view doesn't have all required assets
    // Try to use any available view as a fallback
    console.log("Selected view missing assets, trying fallbacks");

    if (skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
        console.log("Falling back to dorm view");
        return {
            atlas: getAssetURL(repoBaseURL, skin.dorm.atlas),
            png: getAssetURL(repoBaseURL, skin.dorm.png),
            skel: getAssetURL(repoBaseURL, skin.dorm.skel),
            type: "dorm",
        };
    } else if (skin.front.atlas && skin.front.png && skin.front.skel) {
        console.log("Falling back to front view");
        return {
            atlas: getAssetURL(repoBaseURL, skin.front.atlas),
            png: getAssetURL(repoBaseURL, skin.front.png),
            skel: getAssetURL(repoBaseURL, skin.front.skel),
            type: "front",
        };
    } else if (skin.back.atlas && skin.back.png && skin.back.skel) {
        console.log("Falling back to back view");
        return {
            atlas: getAssetURL(repoBaseURL, skin.back.atlas),
            png: getAssetURL(repoBaseURL, skin.back.png),
            skel: getAssetURL(repoBaseURL, skin.back.skel),
            type: "back",
        };
    }

    console.log("No valid view found for skin", {
        selectedSkin,
        dorm: { path: skin.dorm.path, hasAssets: !!(skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) },
        front: { path: skin.front.path, hasAssets: !!(skin.front.atlas && skin.front.png && skin.front.skel) },
        back: { path: skin.back.path, hasAssets: !!(skin.back.atlas && skin.back.png && skin.back.skel) },
    });
    return null;
}