import { type ISkeletonData, Spine } from "pixi-spine";
import * as PIXI from "pixi.js";
import { useCallback, useEffect, useRef, useState } from "react";
import { Card, CardContent } from "~/components/ui/card";
import type { ChibisSimplified } from "~/types/impl/api/impl/chibis";
import type { FormattedChibis, ResourceMap, SpineAnimation } from "~/types/impl/frontend/impl/chibis";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";

export function ChibiViewer({ chibi, skinId, repoBaseUrl }: { chibi: ChibisSimplified; skinId: string; repoBaseUrl: string }) {
    const [formattedChibi, setFormattedChibi] = useState<FormattedChibis | null>(null);

    // eslint-disable-next-line @typescript-eslint/no-redundant-type-constituents
    const appRef = useRef<PIXI.Application | null>(null);
    const spineRef = useRef<Spine | null>(null);

    const [selectedAnimation, setSelectedAnimation] = useState<string>("Idle");
    const [availableAnimations, setAvailableAnimations] = useState<string[]>([]);
    const [viewType, setViewType] = useState<"dorm" | "front" | "back">("dorm");

    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const canvasContainerRef = useRef<HTMLDivElement>(null);

    const skinIdToChibiId = (skinId: string) => {
        // Ex.
        // char_332_archet@shining#1
        // Should be:
        // char_332_archet_shining_1

        return skinId.replace("@", "_").replace("#", "_");
    };

    const formatData = useCallback((): FormattedChibis => {
        const operatorCode = chibi.operatorCode.includes("/") ? (chibi.operatorCode.split("/").pop() ?? chibi.operatorCode) : chibi.operatorCode;

        const formattedChibi: FormattedChibis = {
            name: chibi.name,
            operatorCode,
            path: chibi.path,
            skins: [],
        };

        type AnimationType = {
            atlas?: string;
            png?: string;
            skel?: string;
        };

        type SkinData = {
            name: string;
            dorm?: { atlas: string; png: string; skel: string; path: string };
            front?: { atlas: string; png: string; skel: string; path: string };
            back?: { atlas: string; png: string; skel: string; path: string };
        };

        const skinsByName = new Map<string, SkinData>();

        for (const skin of chibi.skins) {
            const skinName = skin.name.startsWith("build_") ? (skin.name.split("build_")[1]?.split("/")[0] ?? chibi.name) : skin.name;

            const existingSkin = skinsByName.get(skinName) ?? { name: skinName };

            const createAnimationData = (animationType: AnimationType | undefined) => ({
                atlas: animationType?.atlas ?? "",
                png: animationType?.png ?? "",
                skel: animationType?.skel ?? "",
                path: skin.path,
            });

            if (skin.animationTypes?.dorm) {
                existingSkin.dorm = createAnimationData(skin.animationTypes.dorm);
            }

            if (skin.animationTypes?.front) {
                existingSkin.front = createAnimationData(skin.animationTypes.front);
            }

            if (skin.animationTypes?.back) {
                existingSkin.back = createAnimationData(skin.animationTypes.back);
            }

            skinsByName.set(skinName, existingSkin);
        }

        const emptyAnimationData = {
            atlas: "",
            png: "",
            skel: "",
            path: "",
        };

        formattedChibi.skins = Array.from(skinsByName.values()).map((skin) => ({
            name: skin.name,
            dorm: skin.dorm ?? emptyAnimationData,
            front: skin.front ?? emptyAnimationData,
            back: skin.back ?? emptyAnimationData,
        }));

        return formattedChibi;
    }, [chibi]);

    useEffect(() => {
        const formattedChibi = formatData();

        setFormattedChibi(formattedChibi);
    }, [formatData]);

    useEffect(() => {
        if (!canvasContainerRef.current) return;

        if (appRef.current) {
            appRef.current.destroy(true, true);
        }

        const pixiApp = new PIXI.Application({
            width: 400,
            height: 400,
            backgroundColor: 0x000000,
            antialias: true,
            transparent: true,
            autoDensity: true, // Enable automatic resizing
            resolution: window.devicePixelRatio || 1,
        });

        canvasContainerRef.current.innerHTML = "";
        canvasContainerRef.current.appendChild(pixiApp.view);
        appRef.current = pixiApp; // Store in ref instead of state

        // Function to center the spine object if it exists
        const centerSpine = () => {
            if (spineRef.current && appRef.current) {
                // Center the spine
                spineRef.current.x = appRef.current.screen.width / 2;
                spineRef.current.y = (appRef.current.screen.height / 2) * 2;
            }
        };

        // Function to resize the canvas and recenter the spine
        const handleResize = () => {
            if (!canvasContainerRef.current || !appRef.current) return;

            // Get the parent element dimensions
            const parentWidth = canvasContainerRef.current.clientWidth || 400;
            const parentHeight = canvasContainerRef.current.clientHeight || 400;

            // Resize the canvas
            appRef.current.renderer.resize(parentWidth, parentHeight);

            // Recenter the spine
            centerSpine();
        };

        // Initial resize
        handleResize();

        // Set up resize listener
        window.addEventListener("resize", handleResize);

        // Also monitor container size changes
        const resizeObserver = new ResizeObserver(() => {
            handleResize();
        });

        if (canvasContainerRef.current) {
            resizeObserver.observe(canvasContainerRef.current);
        }

        // Clean up on unmount
        return () => {
            window.removeEventListener("resize", handleResize);
            resizeObserver.disconnect();

            // Safer destroy to avoid the "cancelResize is not a function" error
            try {
                if (appRef.current) {
                    // Clean up PIXI application
                    appRef.current.destroy(false, { children: true, texture: true, baseTexture: true });
                }
            } catch (err) {
                console.error("Error cleaning up Pixi application:", err);
            }

            // Clear references
            appRef.current = null;
            spineRef.current = null;
        };
    }, []);

    const getAssetUrl = useCallback(
        (path: string) => {
            // Remove the initial "./" if present
            const normalizedPath = path.startsWith("./") ? path.substring(2) : path;
            return `${repoBaseUrl}${normalizedPath}`;
        },
        [repoBaseUrl],
    );

    const getSkinData = useCallback(() => {
        const chibiId = skinIdToChibiId(skinId);

        const skin = formattedChibi?.skins.find((s) => s.dorm.path.includes(chibiId) || s.front.path.includes(chibiId) || s.back.path.includes(chibiId));
        if (!skin) {
            console.log("Could not find matching skin", {
                selectedSkin: skinId,
                availableSkins: formattedChibi?.skins.map((s) => ({
                    dorm: s.dorm.path,
                    front: s.front.path,
                    back: s.back.path,
                })),
            });

            if (formattedChibi && formattedChibi.skins.length > 0) {
                const fallbackSkin = formattedChibi.skins[0];
                if (fallbackSkin) {
                    console.log("Using fallback skin", { fallbackSkin: fallbackSkin.name });

                    // Try the selected view type first, then fall back to others
                    if (viewType === "dorm" && fallbackSkin.dorm?.atlas && fallbackSkin.dorm?.png && fallbackSkin.dorm?.skel) {
                        return {
                            atlas: getAssetUrl(fallbackSkin.dorm.atlas),
                            png: getAssetUrl(fallbackSkin.dorm.png),
                            skel: getAssetUrl(fallbackSkin.dorm.skel),
                            type: "dorm",
                        };
                    } else if (viewType === "front" && fallbackSkin.front?.atlas && fallbackSkin.front?.png && fallbackSkin.front?.skel) {
                        return {
                            atlas: getAssetUrl(fallbackSkin.front.atlas),
                            png: getAssetUrl(fallbackSkin.front.png),
                            skel: getAssetUrl(fallbackSkin.front.skel),
                            type: "front",
                        };
                    } else if (viewType === "back" && fallbackSkin.back?.atlas && fallbackSkin.back?.png && fallbackSkin.back?.skel) {
                        return {
                            atlas: getAssetUrl(fallbackSkin.back.atlas),
                            png: getAssetUrl(fallbackSkin.back.png),
                            skel: getAssetUrl(fallbackSkin.back.skel),
                            type: "back",
                        };
                    }

                    // If the selected view type isn't available, try any available view
                    if (fallbackSkin.dorm?.atlas && fallbackSkin.dorm?.png && fallbackSkin.dorm?.skel) {
                        return {
                            atlas: getAssetUrl(fallbackSkin.dorm.atlas),
                            png: getAssetUrl(fallbackSkin.dorm.png),
                            skel: getAssetUrl(fallbackSkin.dorm.skel),
                            type: "dorm",
                        };
                    } else if (fallbackSkin.front?.atlas && fallbackSkin.front?.png && fallbackSkin.front?.skel) {
                        return {
                            atlas: getAssetUrl(fallbackSkin.front.atlas),
                            png: getAssetUrl(fallbackSkin.front.png),
                            skel: getAssetUrl(fallbackSkin.front.skel),
                            type: "front",
                        };
                    } else if (fallbackSkin.back?.atlas && fallbackSkin.back?.png && fallbackSkin.back?.skel) {
                        return {
                            atlas: getAssetUrl(fallbackSkin.back.atlas),
                            png: getAssetUrl(fallbackSkin.back.png),
                            skel: getAssetUrl(fallbackSkin.back.skel),
                            type: "back",
                        };
                    }
                }
            }

            return null;
        }

        if (viewType === "dorm" && skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
            console.log("Using dorm view (selected)", { path: skin.dorm.path });
            return {
                atlas: getAssetUrl(skin.dorm.atlas),
                png: getAssetUrl(skin.dorm.png),
                skel: getAssetUrl(skin.dorm.skel),
                type: "dorm",
            };
        } else if (viewType === "front" && skin.front.atlas && skin.front.png && skin.front.skel) {
            console.log("Using front view (selected)", { path: skin.front.path });
            return {
                atlas: getAssetUrl(skin.front.atlas),
                png: getAssetUrl(skin.front.png),
                skel: getAssetUrl(skin.front.skel),
                type: "front",
            };
        } else if (viewType === "back" && skin.back.atlas && skin.back.png && skin.back.skel) {
            console.log("Using back view (selected)", { path: skin.back.path });
            return {
                atlas: getAssetUrl(skin.back.atlas),
                png: getAssetUrl(skin.back.png),
                skel: getAssetUrl(skin.back.skel),
                type: "back",
            };
        }

        if (skin.dorm.path.includes(chibiId) && skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
            console.log("Using dorm view (path match)", { path: skin.dorm.path });
            return {
                atlas: getAssetUrl(skin.dorm.atlas),
                png: getAssetUrl(skin.dorm.png),
                skel: getAssetUrl(skin.dorm.skel),
                type: "dorm",
            };
        } else if (skin.front.path.includes(chibiId) && skin.front.atlas && skin.front.png && skin.front.skel) {
            console.log("Using front view (path match)", { path: skin.front.path });
            return {
                atlas: getAssetUrl(skin.front.atlas),
                png: getAssetUrl(skin.front.png),
                skel: getAssetUrl(skin.front.skel),
                type: "front",
            };
        } else if (skin.back.path.includes(chibiId) && skin.back.atlas && skin.back.png && skin.back.skel) {
            console.log("Using back view (path match)", { path: skin.back.path });
            return {
                atlas: getAssetUrl(skin.back.atlas),
                png: getAssetUrl(skin.back.png),
                skel: getAssetUrl(skin.back.skel),
                type: "back",
            };
        }

        // If we get here, we found a skin but the selected view doesn't have all required assets
        // Try to use any available view as a fallback
        console.log("Selected view missing assets, trying fallbacks");

        if (skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
            console.log("Falling back to dorm view");
            return {
                atlas: getAssetUrl(skin.dorm.atlas),
                png: getAssetUrl(skin.dorm.png),
                skel: getAssetUrl(skin.dorm.skel),
                type: "dorm",
            };
        } else if (skin.front.atlas && skin.front.png && skin.front.skel) {
            console.log("Falling back to front view");
            return {
                atlas: getAssetUrl(skin.front.atlas),
                png: getAssetUrl(skin.front.png),
                skel: getAssetUrl(skin.front.skel),
                type: "front",
            };
        } else if (skin.back.atlas && skin.back.png && skin.back.skel) {
            console.log("Falling back to back view");
            return {
                atlas: getAssetUrl(skin.back.atlas),
                png: getAssetUrl(skin.back.png),
                skel: getAssetUrl(skin.back.skel),
                type: "back",
            };
        }

        console.log("No valid view found for skin", {
            selectedSkin: skinId,
            dorm: { path: skin.dorm.path, hasAssets: !!(skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) },
            front: { path: skin.front.path, hasAssets: !!(skin.front.atlas && skin.front.png && skin.front.skel) },
            back: { path: skin.back.path, hasAssets: !!(skin.back.atlas && skin.back.png && skin.back.skel) },
        });

        return null;
    }, [formattedChibi, getAssetUrl, skinId, viewType]);

    useEffect(() => {
        // Get the skin data using our helper function
        const skinData = getSkinData();
        if (!skinData || !appRef.current) {
            // Set error if we have an operator and skin selected but couldn't get skin data
            if (formattedChibi && skinId) {
                setError(`Could not load skin data for ${skinId}`);
            }
            return;
        }

        const atlasURL = skinData.atlas;
        const skelURL = skinData.skel;
        const imageURL = skinData.png;

        setIsLoading(true);
        setError(null);

        const app = appRef.current;
        const loader = app.loader;

        loader.reset();

        try {
            // Generate unique identifiers for this specific load operation with a timestamp
            const loadId = Date.now().toString();
            loader.add(`chibi_atlas_${loadId}`, atlasURL);
            loader.add(`chibi_skel_${loadId}`, skelURL);
            loader.add(`chibi_image_${loadId}`, imageURL);

            loader.load((_: unknown, resources: ResourceMap) => {
                try {
                    if (spineRef.current?.parent) {
                        spineRef.current.parent.removeChild(spineRef.current);
                        spineRef.current = null;
                    }

                    // Update resource references to use our new unique identifiers
                    const skelResource = resources[`chibi_skel_${loadId}`];
                    const atlasResource = resources[`chibi_atlas_${loadId}`];

                    if (!skelResource || !atlasResource) {
                        setError("Failed to load spine resources");
                        setIsLoading(false);
                        return;
                    }

                    if (skelResource.spineData) {
                        const spineData = new Spine(skelResource.spineData as ISkeletonData);

                        // Scale it appropriately
                        spineData.scale.set(0.5);

                        // Add to stage
                        const app = appRef.current;
                        if (app) {
                            app.stage.addChild(spineData);
                        }

                        // Store spine in ref
                        spineRef.current = spineData;

                        // Get available animations - casting to our minimal interface
                        const animations = spineData.spineData.animations.map((anim: SpineAnimation) => anim.name);
                        setAvailableAnimations(animations);

                        // Set default animation if available - after storing spine in ref
                        let initialAnimation = "Idle";
                        if (animations.includes("Idle")) {
                            spineData.state.setAnimation(0, "Idle", true);
                            setSelectedAnimation("Idle");
                            initialAnimation = "Idle";
                        } else if (animations.length > 0) {
                            const defaultAnim = animations[0] ?? "Idle";
                            spineData.state.setAnimation(0, defaultAnim, true);
                            setSelectedAnimation(defaultAnim);
                            initialAnimation = defaultAnim;
                        }

                        // Adjust position based on the animation type
                        if (appRef.current) {
                            // First center the spine
                            spineData.x = appRef.current.screen.width / 2;
                            spineData.y = (appRef.current.screen.height / 2) * 2;

                            // Then adjust for specific animations
                            if (initialAnimation.toLowerCase().includes("sit") || initialAnimation.toLowerCase() === "sitting") {
                                // Move up by only 10% of screen height to match the adjustment function
                                spineData.y = spineData.y * 1.5;
                                console.log("Adjusting initial position for sitting animation");
                            }
                        }

                        setError(null);
                    } else {
                        setError("Invalid spine data format");
                    }
                } catch (e: unknown) {
                    // Handle error properly with type checking
                    const errorMessage = e instanceof Error ? e.message : "Unknown error occurred";
                    setError(errorMessage);
                } finally {
                    setIsLoading(false);
                }
            });

            loader.onError.add(() => {
                setError("Failed to load spine resources");
                setIsLoading(false);
            });
        } catch (e: unknown) {
            // Handle error properly with type checking
            const errorMessage = e instanceof Error ? e.message : "Unknown error occurred";
            setError(errorMessage);
        }
    }, [formattedChibi, skinId, repoBaseUrl, getAssetUrl, getSkinData, viewType]);

    // Function to adjust position based on animation type
    const adjustPositionForAnimation = useCallback(() => {
        if (!spineRef.current || !appRef.current) return;

        // Base position is center of screen
        const baseX = appRef.current.screen.width / 2;
        const baseY = (appRef.current.screen.height / 2) * 1.75;

        // Apply the position
        spineRef.current.x = baseX;
        spineRef.current.y = baseY;
    }, []);

    useEffect(() => {
        // Only try to change animation if spine exists and animation name is valid
        if (spineRef.current?.state && selectedAnimation && availableAnimations.includes(selectedAnimation)) {
            try {
                spineRef.current.state.setAnimation(0, selectedAnimation, true);

                // Adjust position based on the animation type
                adjustPositionForAnimation();
            } catch (error) {
                console.error("Failed to set animation:", error);
            }
        }
    }, [selectedAnimation, availableAnimations, adjustPositionForAnimation]);

    // Handle animation change
    const handleAnimationChange = (value: string) => {
        setSelectedAnimation(value);

        // Apply the animation to the spine object
        if (spineRef.current && value) {
            // Set the animation
            spineRef.current.state.setAnimation(0, value, true);

            // Adjust position based on the animation type
            adjustPositionForAnimation();
        }
    };

    // Handle view type change
    const handleViewTypeChange = (value: string) => {
        setViewType(value as "dorm" | "front" | "back");
    };

    // Get current skin data for UI display
    const spineData = getSkinData();

    return (
        <Card className="w-full">
            <CardContent className="pb-4 pt-6">
                <div className="mb-4 flex flex-col gap-4">
                    <div className="flex items-center gap-2">
                        <Select value={selectedAnimation} onValueChange={handleAnimationChange} disabled={!spineData || availableAnimations.length === 0 || isLoading}>
                            <SelectTrigger className="w-[180px]">
                                <SelectValue placeholder="Select Animation" />
                            </SelectTrigger>
                            <SelectContent>
                                {availableAnimations.map((anim) => (
                                    <SelectItem key={anim} value={anim}>
                                        {anim.charAt(0).toUpperCase() + anim.slice(1)}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>

                        {/* View Type Selector */}
                        <div className="flex items-center gap-2">
                            <Select value={viewType} onValueChange={handleViewTypeChange} disabled={isLoading}>
                                <SelectTrigger className="w-[120px]">
                                    <SelectValue placeholder="View Type" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="dorm">Dorm</SelectItem>
                                    <SelectItem value="front">Front</SelectItem>
                                    <SelectItem value="back">Back</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    </div>
                </div>

                <div className="relative h-[300px] w-full">
                    <div ref={canvasContainerRef} className="h-full w-full" />
                    {isLoading && (
                        <div className="absolute inset-0 flex items-center justify-center bg-black/20">
                            <div className="text-lg font-semibold text-white">Loading...</div>
                        </div>
                    )}
                    {error && (
                        <div className="absolute inset-0 flex items-center justify-center bg-black/20">
                            <div className="max-w-md rounded-md bg-red-500 p-4 text-white">
                                <div className="font-semibold">Error loading animation:</div>
                                <div>{error}</div>
                            </div>
                        </div>
                    )}
                </div>
            </CardContent>
        </Card>
    );
}
