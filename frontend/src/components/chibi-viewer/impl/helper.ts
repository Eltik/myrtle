import { getCDNURL } from "~/lib/cdn";
import type { AnimationType, ChibiAnimation, FormattedChibis } from "~/types/impl/frontend/impl/chibis";

export function getSkinData(selectedOperator: FormattedChibis, selectedSkin: string, viewType: ChibiAnimation): AnimationType | null {
    console.log("Getting skin data for", { selectedOperator, selectedSkin, viewType });
    console.log("Selected operator skins", selectedOperator.skins);
    const skin = selectedOperator.skins.find((s) => s.dorm.path === selectedSkin || s.front.path === selectedSkin || s.back.path === selectedSkin);
    if (!skin) {
        // Fallback: Try to use the first skin if available
        if (selectedOperator.skins.length > 0) {
            const fallbackSkin = selectedOperator.skins[0];
            if (fallbackSkin) {
                console.log("Using fallback skin", { fallbackSkin: fallbackSkin.name });

                // Try the selected view type first, then fall back to others
                if (viewType === "dorm" && fallbackSkin.dorm?.atlas && fallbackSkin.dorm?.png && fallbackSkin.dorm?.skel) {
                    return {
                        atlas: getCDNURL(fallbackSkin.dorm.atlas, true),
                        png: getCDNURL(fallbackSkin.dorm.png, true),
                        skel: getCDNURL(fallbackSkin.dorm.skel, true),
                    };
                } else if (viewType === "front" && fallbackSkin.front?.atlas && fallbackSkin.front?.png && fallbackSkin.front?.skel) {
                    return {
                        atlas: getCDNURL(fallbackSkin.front.atlas, true),
                        png: getCDNURL(fallbackSkin.front.png, true),
                        skel: getCDNURL(fallbackSkin.front.skel, true),
                    };
                } else if (viewType === "back" && fallbackSkin.back?.atlas && fallbackSkin.back?.png && fallbackSkin.back?.skel) {
                    return {
                        atlas: getCDNURL(fallbackSkin.back.atlas, true),
                        png: getCDNURL(fallbackSkin.back.png, true),
                        skel: getCDNURL(fallbackSkin.back.skel, true),
                    };
                }

                // If the selected view type isn't available, try any available view
                if (fallbackSkin.dorm?.atlas && fallbackSkin.dorm?.png && fallbackSkin.dorm?.skel) {
                    return {
                        atlas: getCDNURL(fallbackSkin.dorm.atlas, true),
                        png: getCDNURL(fallbackSkin.dorm.png, true),
                        skel: getCDNURL(fallbackSkin.dorm.skel, true),
                    };
                } else if (fallbackSkin.front?.atlas && fallbackSkin.front?.png && fallbackSkin.front?.skel) {
                    return {
                        atlas: getCDNURL(fallbackSkin.front.atlas, true),
                        png: getCDNURL(fallbackSkin.front.png, true),
                        skel: getCDNURL(fallbackSkin.front.skel, true),
                    };
                } else if (fallbackSkin.back?.atlas && fallbackSkin.back?.png && fallbackSkin.back?.skel) {
                    return {
                        atlas: getCDNURL(fallbackSkin.back.atlas, true),
                        png: getCDNURL(fallbackSkin.back.png, true),
                        skel: getCDNURL(fallbackSkin.back.skel, true),
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
            atlas: getCDNURL(skin.dorm.atlas, true),
            png: getCDNURL(skin.dorm.png, true),
            skel: getCDNURL(skin.dorm.skel, true),
        };
    } else if (viewType === "front" && skin.front.atlas && skin.front.png && skin.front.skel) {
        console.log("Using front view (selected)", { path: skin.front.path });
        return {
            atlas: getCDNURL(skin.front.atlas, true),
            png: getCDNURL(skin.front.png, true),
            skel: getCDNURL(skin.front.skel, true),
        };
    } else if (viewType === "back" && skin.back.atlas && skin.back.png && skin.back.skel) {
        console.log("Using back view (selected)", { path: skin.back.path });
        return {
            atlas: getCDNURL(skin.back.atlas, true),
            png: getCDNURL(skin.back.png, true),
            skel: getCDNURL(skin.back.skel, true),
        };
    }

    // If the selected view type doesn't have all required assets, check if the path matches a specific view
    if (selectedSkin === skin.dorm.path && skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
        console.log("Using dorm view (path match)", { path: skin.dorm.path });
        return {
            atlas: getCDNURL(skin.dorm.atlas, true),
            png: getCDNURL(skin.dorm.png, true),
            skel: getCDNURL(skin.dorm.skel, true),
        };
    } else if (selectedSkin === skin.front.path && skin.front.atlas && skin.front.png && skin.front.skel) {
        console.log("Using front view (path match)", { path: skin.front.path });
        return {
            atlas: getCDNURL(skin.front.atlas, true),
            png: getCDNURL(skin.front.png, true),
            skel: getCDNURL(skin.front.skel, true),
        };
    } else if (selectedSkin === skin.back.path && skin.back.atlas && skin.back.png && skin.back.skel) {
        console.log("Using back view (path match)", { path: skin.back.path });
        return {
            atlas: getCDNURL(skin.back.atlas, true),
            png: getCDNURL(skin.back.png, true),
            skel: getCDNURL(skin.back.skel, true),
        };
    }

    // If we get here, we found a skin but the selected view doesn't have all required assets
    // Try to use any available view as a fallback
    console.log("Selected view missing assets, trying fallbacks");

    if (skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
        console.log("Falling back to dorm view");
        return {
            atlas: getCDNURL(skin.dorm.atlas, true),
            png: getCDNURL(skin.dorm.png, true),
            skel: getCDNURL(skin.dorm.skel, true),
        };
    } else if (skin.front.atlas && skin.front.png && skin.front.skel) {
        console.log("Falling back to front view");
        return {
            atlas: getCDNURL(skin.front.atlas, true),
            png: getCDNURL(skin.front.png, true),
            skel: getCDNURL(skin.front.skel, true),
        };
    } else if (skin.back.atlas && skin.back.png && skin.back.skel) {
        console.log("Falling back to back view");
        return {
            atlas: getCDNURL(skin.back.atlas, true),
            png: getCDNURL(skin.back.png, true),
            skel: getCDNURL(skin.back.skel, true),
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
