/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
import { useCallback, useEffect, useRef, useState } from "react";
import type { FormattedChibis } from "~/types/impl/frontend/impl/chibis";
import { Card, CardContent } from "~/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { Slider } from "~/components/ui/slider";
import { Button } from "~/components/ui/button";
import { PauseIcon, PlayIcon, RepeatIcon } from "lucide-react";
import * as PIXI from "pixi.js";
import { Spine } from "pixi-spine";

// Add this interface for spine animations
interface SpineAnimation {
    name: string;
}

// Define resource interface to help with type safety
type ResourceMap = Record<
    string,
    {
        spineData?: unknown;
        data?: unknown;
    }
>;

type ChibiRendererProps = {
    selectedOperator: FormattedChibis | null;
    selectedSkin: string | null;
    repoBaseUrl: string;
};

export function ChibiRenderer({ selectedOperator, selectedSkin, repoBaseUrl }: ChibiRendererProps) {
    // eslint-disable-next-line @typescript-eslint/no-redundant-type-constituents
    const appRef = useRef<PIXI.Application | null>(null);
    const spineRef = useRef<Spine | null>(null);

    const [selectedAnimation, setSelectedAnimation] = useState<string>("Idle");
    const [availableAnimations, setAvailableAnimations] = useState<string[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isPlaying, setIsPlaying] = useState(true);
    const [speed, setSpeed] = useState(1);
    const [viewType, setViewType] = useState<"dorm" | "front" | "back">("dorm");
    const canvasContainerRef = useRef<HTMLDivElement>(null);

    const animationActiveRef = useRef<boolean>(true);

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

    // Function to get asset URL from the path
    const getAssetUrl = useCallback(
        (path: string) => {
            // Remove the initial "./" if present
            const normalizedPath = path.startsWith("./") ? path.substring(2) : path;
            return `${repoBaseUrl}${normalizedPath}`;
        },
        [repoBaseUrl],
    );

    // Get skin data with fallback
    const getSkinData = useCallback(() => {
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

        // First, try to use the selected view type if it has all required assets
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

        // If the selected view type doesn't have all required assets, check if the path matches a specific view
        if (selectedSkin === skin.dorm.path && skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
            console.log("Using dorm view (path match)", { path: skin.dorm.path });
            return {
                atlas: getAssetUrl(skin.dorm.atlas),
                png: getAssetUrl(skin.dorm.png),
                skel: getAssetUrl(skin.dorm.skel),
                type: "dorm",
            };
        } else if (selectedSkin === skin.front.path && skin.front.atlas && skin.front.png && skin.front.skel) {
            console.log("Using front view (path match)", { path: skin.front.path });
            return {
                atlas: getAssetUrl(skin.front.atlas),
                png: getAssetUrl(skin.front.png),
                skel: getAssetUrl(skin.front.skel),
                type: "front",
            };
        } else if (selectedSkin === skin.back.path && skin.back.atlas && skin.back.png && skin.back.skel) {
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
            selectedSkin,
            dorm: { path: skin.dorm.path, hasAssets: !!(skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) },
            front: { path: skin.front.path, hasAssets: !!(skin.front.atlas && skin.front.png && skin.front.skel) },
            back: { path: skin.back.path, hasAssets: !!(skin.back.atlas && skin.back.png && skin.back.skel) },
        });
        return null;
    }, [selectedOperator, selectedSkin, getAssetUrl, viewType]);

    useEffect(() => {
        // Get the skin data using our helper function
        const skinData = getSkinData();
        if (!skinData || !appRef.current) {
            // Set error if we have an operator and skin selected but couldn't get skin data
            if (selectedOperator && selectedSkin) {
                setError(`Could not load skin data for ${selectedSkin}`);
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
                        const spineData = new Spine(skelResource.spineData as any);

                        // Scale it appropriately
                        spineData.scale.set(0.5);

                        // Add to stage
                        const app = appRef.current;
                        if (app) {
                            app.stage.addChild(spineData);
                        }

                        // Store spine in ref
                        spineRef.current = spineData;

                        // Center the spine after adding it to stage
                        if (app) {
                            spineData.x = app.screen.width / 2;
                            spineData.y = app.screen.height / 2;
                        }

                        // Get available animations - casting to our minimal interface
                        const animations = spineData.spineData.animations.map((anim: SpineAnimation) => anim.name);
                        setAvailableAnimations(animations);

                        // Set default animation if available - after storing spine in ref
                        if (animations.includes("Idle")) {
                            spineData.state.setAnimation(0, "Idle", true);
                            setSelectedAnimation("Idle");
                        } else if (animations.length > 0) {
                            const defaultAnim = animations[0] ?? "Idle";
                            spineData.state.setAnimation(0, defaultAnim, true);
                            setSelectedAnimation(defaultAnim);
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
    }, [selectedOperator, selectedSkin, repoBaseUrl, getAssetUrl, getSkinData, viewType]);

    useEffect(() => {
        // Only try to change animation if spine exists and animation name is valid
        if (spineRef.current?.state && selectedAnimation && availableAnimations.includes(selectedAnimation)) {
            try {
                spineRef.current.state.setAnimation(0, selectedAnimation, true);

                // Ensure the spine stays centered after animation change
                if (appRef.current) {
                    spineRef.current.x = appRef.current.screen.width / 2;
                    spineRef.current.y = (appRef.current.screen.height / 2) * 2;
                }
            } catch (error) {
                console.error("Failed to set animation:", error);
            }
        }
    }, [selectedAnimation, availableAnimations]);

    // Handle animation change
    const handleAnimationChange = (value: string) => {
        setSelectedAnimation(value);

        // Apply the animation to the spine object
        if (spineRef.current && value) {
            // Set the animation
            spineRef.current.state.setAnimation(0, value, true);

            // Ensure the spine stays centered
            if (appRef.current) {
                spineRef.current.x = appRef.current.screen.width / 2;
                spineRef.current.y = appRef.current.screen.height / 2;
            }
        }
    };

    // Handle play/pause
    const handlePlayPause = () => {
        setIsPlaying(!isPlaying);
    };

    // Handle reset
    const handleReset = () => {
        // Reset animation and ensure chibi stays centered
        if (spineRef.current && appRef.current) {
            // Reset scale if needed
            spineRef.current.scale.set(0.5);

            // Center the sprite
            spineRef.current.x = appRef.current.screen.width / 2;
            spineRef.current.y = (appRef.current.screen.height / 2) * 2;

            // Reset any other transformations
            spineRef.current.rotation = 0;
        }

        // Ensure it's playing
        if (!isPlaying) {
            setIsPlaying(true);
        }
    };

    // Handle speed change
    const handleSpeedChange = (value: number[]) => {
        if (value.length > 0 && typeof value[0] === "number") {
            setSpeed(value[0]);
        }
    };

    // Handle view type change
    const handleViewTypeChange = (value: string) => {
        setViewType(value as "dorm" | "front" | "back");
    };

    // Get current skin data for UI display
    const spineData = getSkinData();
    const hasAnimation = Boolean(canvasContainerRef.current?.firstChild && animationActiveRef.current);

    return (
        <Card className="w-full">
            <CardContent className="pb-4 pt-6">
                {/* Animation controls */}
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

                        <Button variant="outline" size="icon" onClick={handlePlayPause} disabled={!hasAnimation || isLoading}>
                            {isPlaying ? <PauseIcon className="h-4 w-4" /> : <PlayIcon className="h-4 w-4" />}
                        </Button>

                        <Button variant="outline" size="icon" onClick={handleReset} disabled={!hasAnimation || isLoading}>
                            <RepeatIcon className="h-4 w-4" />
                        </Button>
                    </div>

                    {/* View Type Selector */}
                    <div className="flex items-center gap-2">
                        <span className="w-10 text-sm">View:</span>
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

                    <div className="flex items-center gap-2">
                        <span className="w-10 text-sm">Speed:</span>
                        <Slider defaultValue={[1]} min={0.1} max={2} step={0.1} value={[speed]} onValueChange={handleSpeedChange} disabled={!hasAnimation || isLoading} className="w-full max-w-56" />
                        <span className="w-8 text-sm">{speed}x</span>
                    </div>
                </div>

                {/* Display area */}
                <div className="relative h-64 w-full rounded-md border bg-background">
                    {/* Loading state */}
                    {isLoading && (
                        <div className="absolute inset-0 z-10 flex items-center justify-center">
                            <div className="h-6 w-6 animate-spin rounded-full border-t-2 border-primary" />
                        </div>
                    )}

                    {/* Error message */}
                    {error && (
                        <div className="absolute inset-0 z-10 flex items-center justify-center p-4 text-center">
                            <p className="text-destructive">{error}</p>
                        </div>
                    )}

                    {/* Animation container */}
                    <div ref={canvasContainerRef} className="h-full w-full" style={{ display: !isLoading && !error ? "block" : "none" }}></div>
                </div>
            </CardContent>
        </Card>
    );
}
