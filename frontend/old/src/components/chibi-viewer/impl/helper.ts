import { getCDNURL } from "~/lib/cdn";
import type { AnimationType, ChibiAnimation, FormattedChibis } from "~/types/impl/frontend/impl/chibis";

// Helper function to ensure consistent URL encoding of paths
function encodeAssetPath(path: string): string {
    return path
        .split("/")
        .map((segment) => encodeURIComponent(segment))
        .join("/");
}

// Helper function to ensure URLs are properly encoded
export function encodeURL(url: string): string {
    try {
        if (!url) return "";

        // Split the URL into parts
        const parts = url.split("?");
        const basePath = parts[0] ?? "";
        const query = parts[1];

        // Split the path into segments and encode each segment
        const pathSegments = basePath.split("/");
        const encodedPath = pathSegments
            .map((segment) => {
                // If the segment already contains encoded characters, don't encode it again
                if (segment.includes("%")) {
                    return segment;
                }
                return encodeURIComponent(segment);
            })
            .join("/");

        // Reconstruct the URL
        return query ? `${encodedPath}?${query}` : encodedPath;
    } catch (error) {
        console.error("Error encoding URL:", error);
        return url;
    }
}

export function getSkinData(selectedOperator: FormattedChibis, selectedSkin: string, viewType: ChibiAnimation): AnimationType | null {
    console.log("Getting skin data for", {
        operatorName: selectedOperator.name,
        selectedSkin,
        viewType,
        availableSkins: selectedOperator.skins.map((s) => ({
            name: s.name,
            paths: {
                dorm: s.dorm.path,
                front: s.front.path,
                back: s.back.path,
            },
        })),
    });

    // First try to find a skin by name
    let skin = selectedOperator.skins.find((s) => s.name === selectedSkin);

    // If not found by name, try to find by path
    if (!skin) {
        console.log("Skin not found by name, trying to find by path");
        const encodedSelectedSkin = encodeAssetPath(selectedSkin);
        skin = selectedOperator.skins.find((s) => encodeAssetPath(s.dorm.path) === encodedSelectedSkin || encodeAssetPath(s.front.path) === encodedSelectedSkin || encodeAssetPath(s.back.path) === encodedSelectedSkin);
    }

    // If still not found, try to find by partial path match
    if (!skin) {
        console.log("Skin not found by exact path, trying partial match");
        const encodedSelectedSkin = encodeAssetPath(selectedSkin);
        skin = selectedOperator.skins.find((s) => encodeAssetPath(s.dorm.path).includes(encodedSelectedSkin) || encodeAssetPath(s.front.path).includes(encodedSelectedSkin) || encodeAssetPath(s.back.path).includes(encodedSelectedSkin));
    }

    if (!skin) {
        console.log("No matching skin found, using fallback");
        // Fallback: Try to use the first skin if available
        if (selectedOperator.skins.length > 0) {
            const fallbackSkin = selectedOperator.skins[0];
            if (fallbackSkin) {
                console.log("Using fallback skin", { fallbackSkin: fallbackSkin.name });

                // Try the selected view type first, then fall back to others
                if (viewType === "dorm" && fallbackSkin.dorm?.atlas && fallbackSkin.dorm?.png && fallbackSkin.dorm?.skel) {
                    return {
                        atlas: getCDNURL(encodeAssetPath(fallbackSkin.dorm.atlas), false),
                        png: getCDNURL(encodeAssetPath(fallbackSkin.dorm.png), false),
                        skel: getCDNURL(encodeAssetPath(fallbackSkin.dorm.skel), false),
                    };
                } else if (viewType === "front" && fallbackSkin.front?.atlas && fallbackSkin.front?.png && fallbackSkin.front?.skel) {
                    return {
                        atlas: getCDNURL(encodeAssetPath(fallbackSkin.front.atlas), false),
                        png: getCDNURL(encodeAssetPath(fallbackSkin.front.png), false),
                        skel: getCDNURL(encodeAssetPath(fallbackSkin.front.skel), false),
                    };
                } else if (viewType === "back" && fallbackSkin.back?.atlas && fallbackSkin.back?.png && fallbackSkin.back?.skel) {
                    return {
                        atlas: getCDNURL(encodeAssetPath(fallbackSkin.back.atlas), false),
                        png: getCDNURL(encodeAssetPath(fallbackSkin.back.png), false),
                        skel: getCDNURL(encodeAssetPath(fallbackSkin.back.skel), false),
                    };
                }

                // If the selected view type isn't available, try any available view
                if (fallbackSkin.dorm?.atlas && fallbackSkin.dorm?.png && fallbackSkin.dorm?.skel) {
                    return {
                        atlas: getCDNURL(encodeAssetPath(fallbackSkin.dorm.atlas), false),
                        png: getCDNURL(encodeAssetPath(fallbackSkin.dorm.png), false),
                        skel: getCDNURL(encodeAssetPath(fallbackSkin.dorm.skel), false),
                    };
                } else if (fallbackSkin.front?.atlas && fallbackSkin.front?.png && fallbackSkin.front?.skel) {
                    return {
                        atlas: getCDNURL(encodeAssetPath(fallbackSkin.front.atlas), false),
                        png: getCDNURL(encodeAssetPath(fallbackSkin.front.png), false),
                        skel: getCDNURL(encodeAssetPath(fallbackSkin.front.skel), false),
                    };
                } else if (fallbackSkin.back?.atlas && fallbackSkin.back?.png && fallbackSkin.back?.skel) {
                    return {
                        atlas: getCDNURL(encodeAssetPath(fallbackSkin.back.atlas), false),
                        png: getCDNURL(encodeAssetPath(fallbackSkin.back.png), false),
                        skel: getCDNURL(encodeAssetPath(fallbackSkin.back.skel), false),
                    };
                }
            }
        }

        console.log("No valid fallback skin found");
        return null;
    }

    console.log("Found matching skin:", { skinName: skin.name });

    // First, try to use the selected view type if it has all required assets
    if (viewType === "dorm" && skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
        console.log("Using dorm view (selected)", { path: skin.dorm.path });
        return {
            atlas: getCDNURL(encodeAssetPath(skin.dorm.atlas), false),
            png: getCDNURL(encodeAssetPath(skin.dorm.png), false),
            skel: getCDNURL(encodeAssetPath(skin.dorm.skel), false),
        };
    } else if (viewType === "front" && skin.front.atlas && skin.front.png && skin.front.skel) {
        console.log("Using front view (selected)", { path: skin.front.path });
        return {
            atlas: getCDNURL(encodeAssetPath(skin.front.atlas), false),
            png: getCDNURL(encodeAssetPath(skin.front.png), false),
            skel: getCDNURL(encodeAssetPath(skin.front.skel), false),
        };
    } else if (viewType === "back" && skin.back.atlas && skin.back.png && skin.back.skel) {
        console.log("Using back view (selected)", { path: skin.back.path });
        return {
            atlas: getCDNURL(encodeAssetPath(skin.back.atlas), false),
            png: getCDNURL(encodeAssetPath(skin.back.png), false),
            skel: getCDNURL(encodeAssetPath(skin.back.skel), false),
        };
    }

    // If the selected view type doesn't have all required assets, check if the path matches a specific view
    const encodedSelectedSkin = encodeAssetPath(selectedSkin);
    if (encodeAssetPath(skin.dorm.path) === encodedSelectedSkin && skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
        console.log("Using dorm view (path match)", { path: skin.dorm.path });
        return {
            atlas: getCDNURL(encodeAssetPath(skin.dorm.atlas), false),
            png: getCDNURL(encodeAssetPath(skin.dorm.png), false),
            skel: getCDNURL(encodeAssetPath(skin.dorm.skel), false),
        };
    } else if (encodeAssetPath(skin.front.path) === encodedSelectedSkin && skin.front.atlas && skin.front.png && skin.front.skel) {
        console.log("Using front view (path match)", { path: skin.front.path });
        return {
            atlas: getCDNURL(encodeAssetPath(skin.front.atlas), false),
            png: getCDNURL(encodeAssetPath(skin.front.png), false),
            skel: getCDNURL(encodeAssetPath(skin.front.skel), false),
        };
    } else if (encodeAssetPath(skin.back.path) === encodedSelectedSkin && skin.back.atlas && skin.back.png && skin.back.skel) {
        console.log("Using back view (path match)", { path: skin.back.path });
        return {
            atlas: getCDNURL(encodeAssetPath(skin.back.atlas), false),
            png: getCDNURL(encodeAssetPath(skin.back.png), false),
            skel: getCDNURL(encodeAssetPath(skin.back.skel), false),
        };
    }

    // If we get here, we found a skin but the selected view doesn't have all required assets
    // Try to use any available view as a fallback
    console.log("Selected view missing assets, trying fallbacks");

    if (skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
        console.log("Falling back to dorm view");
        return {
            atlas: getCDNURL(encodeAssetPath(skin.dorm.atlas), false),
            png: getCDNURL(encodeAssetPath(skin.dorm.png), false),
            skel: getCDNURL(encodeAssetPath(skin.dorm.skel), false),
        };
    } else if (skin.front.atlas && skin.front.png && skin.front.skel) {
        console.log("Falling back to front view");
        return {
            atlas: getCDNURL(encodeAssetPath(skin.front.atlas), false),
            png: getCDNURL(encodeAssetPath(skin.front.png), false),
            skel: getCDNURL(encodeAssetPath(skin.front.skel), false),
        };
    } else if (skin.back.atlas && skin.back.png && skin.back.skel) {
        console.log("Falling back to back view");
        return {
            atlas: getCDNURL(encodeAssetPath(skin.back.atlas), false),
            png: getCDNURL(encodeAssetPath(skin.back.png), false),
            skel: getCDNURL(encodeAssetPath(skin.back.skel), false),
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
