import { type ISkeletonData, Spine, type TextureAtlas } from "pixi-spine";
import * as PIXI from "pixi.js";
import { useCallback, useEffect, useRef, useState } from "react";
import { Card, CardContent } from "~/components/ui/card";
import type { ChibiAnimation, FormattedChibis } from "~/types/impl/frontend/impl/chibis";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { encodeURL, getSkinData } from "./helper";

const CHIBI_OFFSET_X = 0.25;
const CHIBI_OFFSET_Y = 0.43;
const CHIBI_SCALE = 0.7;
const ANIMATION_SPEED = 0.5; // Animation speed multiplier (lower = slower)

export function ChibiViewer({ chibi, skinId }: { chibi: FormattedChibis; skinId: string }) {
    const appRef = useRef<PIXI.Application | null>(null);
    const spineRef = useRef<Spine | null>(null);

    const [selectedAnimation, setSelectedAnimation] = useState<string>("Idle");
    const [availableAnimations, setAvailableAnimations] = useState<string[]>([]);
    const [viewType, setViewType] = useState<ChibiAnimation>("front");

    const canvasContainerRef = useRef<HTMLDivElement>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    // Function to update canvas size based on container
    const updateCanvasSize = useCallback(() => {
        if (!canvasContainerRef.current || !appRef.current) return;

        const containerWidth = canvasContainerRef.current.clientWidth;
        const containerHeight = canvasContainerRef.current.clientHeight;

        appRef.current.renderer.resize(containerWidth, containerHeight);

        // Reposition spine if it exists
        if (spineRef.current) {
            // Adjust positioning to account for the chibi's built-in offset
            spineRef.current.x = containerWidth * CHIBI_OFFSET_X; // Move further left from center
            spineRef.current.y = containerHeight * CHIBI_OFFSET_Y; // Keep the good vertical position

            // Calculate appropriate scale based on container size
            const scale = Math.min(containerWidth / 1000, containerHeight / 800) * CHIBI_SCALE;
            spineRef.current.scale.set(scale);
        }
    }, []);

    const renderSkinSpine = useCallback(
        (skinAsset: { spineAtlas: TextureAtlas; spineData: ISkeletonData }) => {
            // Clear any existing spine object
            if (spineRef.current) {
                appRef.current?.stage.removeChild(spineRef.current);
                spineRef.current.destroy();
                spineRef.current = null;
            }

            try {
                // Validate spine data
                if (!skinAsset.spineData) {
                    throw new Error("Spine data is missing");
                }

                // Create a new spine object
                spineRef.current = new Spine(skinAsset.spineData);

                // Set the animation speed to 1x (slower)
                spineRef.current.state.timeScale = ANIMATION_SPEED;

                // Set available animations
                const animations = skinAsset.spineData.animations.map((animation) => animation.name);
                setAvailableAnimations(animations);

                if (appRef.current) {
                    const { width, height } = appRef.current.screen;

                    // Adjust positioning to account for the chibi's built-in offset
                    spineRef.current.x = width * CHIBI_OFFSET_X; // Move further left from center
                    spineRef.current.y = height * CHIBI_OFFSET_Y; // Keep the good vertical position

                    // Calculate appropriate scale based on container size
                    const scale = Math.min(width / 1000, height / 800) * CHIBI_SCALE;
                    spineRef.current.scale.set(scale);
                }

                // Set animation based on selected animation if it exists in available animations
                if (selectedAnimation && spineRef.current.spineData.findAnimation(selectedAnimation)) {
                    spineRef.current.state.setAnimation(0, selectedAnimation, true);
                } else {
                    // Fall back to default animations based on view type
                    if (viewType !== "dorm") {
                        if (spineRef.current.spineData.findAnimation("Start")) {
                            spineRef.current.state.setAnimation(0, "Start", false);
                        } else if (spineRef.current.spineData.findAnimation("Start_A")) {
                            spineRef.current.state.setAnimation(0, "Start_A", false);
                        } else {
                            spineRef.current.state.setAnimation(0, "Idle", true);
                        }
                    } else {
                        if (spineRef.current.spineData.findAnimation("Relax")) {
                            spineRef.current.state.setAnimation(0, "Relax", true);
                        } else {
                            spineRef.current.state.setAnimation(0, "Idle", true);
                        }
                    }
                }

                spineRef.current.interactive = true;
                spineRef.current.alpha = 1;

                spineRef.current.state.addListener({
                    complete: (event) => {
                        // Only handle animation completion for non-dorm view
                        if (event.animationEnd && viewType !== "dorm") {
                            // Get the track entry and check the animation name
                            const track = spineRef.current?.state.tracks[0];
                            if (track && ((track as unknown as { animation: { name: string } }).animation.name === "Start" || (track as unknown as { animation: { name: string } }).animation.name === "Start_A")) {
                                spineRef.current?.state.setAnimation(0, "Idle", true);
                            }
                        }
                    },
                });

                // Add the spine object to the stage
                if (!appRef.current) {
                    throw new Error("PIXI application is not initialized");
                }

                appRef.current.stage.addChild(spineRef.current);

                // Render the stage
                appRef.current.renderer.render(appRef.current.stage);

                setIsLoading(false);
            } catch (err) {
                console.error("Spine rendering error:", err);
                setError(`Failed to render chibi: ${err instanceof Error ? err.message : String(err)}`);
                setIsLoading(false);
            }
        },
        [selectedAnimation, viewType],
    );

    const renderCanvas = useCallback(() => {
        // If we don't have an operator selected, abort
        if (!chibi) {
            setError("No operator selected");
            setIsLoading(false);
            return;
        }

        // Get the skin data using the updated function
        const skinData = getSkinData(chibi, skinId ?? "default", viewType);

        if (!skinData) {
            console.error("Failed to load skin data", {
                operatorName: chibi.name,
                selectedSkin: skinId,
                viewType,
                availableSkins: chibi.skins.map((s) => s.name),
            });
            setError("Failed to load skin data. Skin data is missing.");
            setIsLoading(false);
            return;
        }

        // Check if any of the required assets are empty
        if (!skinData.atlas || !skinData.png || !skinData.skel) {
            console.error("Skin data is missing required assets", {
                atlas: !!skinData.atlas,
                png: !!skinData.png,
                skel: !!skinData.skel,
            });
            setError("Skin data is missing required assets.");
            setIsLoading(false);
            return;
        }

        // Get CDN URLs for assets (encodeURL handles basic encoding)
        const assetUrls = {
            atlas: encodeURL(skinData.atlas),
            png: encodeURL(skinData.png), // Keep png for potential future use, but loader uses atlas
            skel: encodeURL(skinData.skel),
        };

        setIsLoading(true);
        setError(null);

        // Directly load the skeleton asset using PIXI.Assets
        // The spine loader will use the .atlas reference within the .skel
        // and automatically load the associated texture (.png)
        PIXI.Assets.load(assetUrls.skel)
            .then((spineAsset: { spineAtlas: TextureAtlas; spineData: ISkeletonData }) => {
                // The loaded asset should contain both atlas and skeleton data
                if (!spineAsset?.spineData) {
                    // Check specifically for spineData
                    setError("Failed to load skeleton data from asset.");
                    setIsLoading(false);
                    return;
                }

                renderSkinSpine(spineAsset);
            })
            .catch((err: Error) => {
                console.error("Failed to load Spine asset:", err);
                setError(`Failed to load skeleton/atlas: ${err.message}`);
                setIsLoading(false);
            });
    }, [chibi, skinId, viewType, renderSkinSpine]);

    useEffect(() => {
        if (!canvasContainerRef.current) {
            return;
        }

        if (appRef.current) {
            appRef.current.destroy(true, true);
        }

        // Get container dimensions
        const containerWidth = canvasContainerRef.current.clientWidth;
        // Use a taller height for the container
        const containerHeight = canvasContainerRef.current.clientHeight || 450;

        const pixiApp = new PIXI.Application({
            width: containerWidth,
            height: containerHeight,
            backgroundColor: 0x111014,
            antialias: true,
            resolution: window.devicePixelRatio || 1,
            backgroundAlpha: 1,
            clearBeforeRender: true,
            premultipliedAlpha: true,
            powerPreference: "high-performance",
        } as PIXI.IApplicationOptions);
        appRef.current = pixiApp;

        // Add the view to the DOM first
        canvasContainerRef.current?.appendChild(pixiApp.view as unknown as Node);

        // Set up the animation loop
        const tick = () => {
            if (spineRef.current && pixiApp?.renderer) {
                // Update the spine animation
                spineRef.current.update(0.016); // Update with approximate 60fps delta time
                // Render the stage
                pixiApp.renderer.render(pixiApp.stage);
            }
            requestAnimationFrame(tick);
        };

        // Start the animation loop
        requestAnimationFrame(tick);

        // Handle window resize
        const handleResize = () => {
            updateCanvasSize();
        };

        window.addEventListener("resize", handleResize);

        renderCanvas();

        return () => {
            window.removeEventListener("resize", handleResize);

            if (spineRef.current) {
                appRef.current?.stage.removeChild(spineRef.current);
                spineRef.current.destroy();
                spineRef.current = null;
            }

            if (appRef.current) {
                appRef.current.destroy(true, { children: true, texture: true, baseTexture: true });
                appRef.current = null;
            }
        };
    }, [renderCanvas, updateCanvasSize]);

    // Effect to reset animation if it becomes invalid after view type change
    useEffect(() => {
        // Only run if animations are loaded and the selected one is no longer valid
        if (availableAnimations.length > 0 && !availableAnimations.includes(selectedAnimation)) {
            let newAnimation: string | undefined = undefined;

            if (viewType === "dorm") {
                if (availableAnimations.includes("Relax")) {
                    newAnimation = "Relax";
                } else if (availableAnimations.includes("Idle")) {
                    newAnimation = "Idle";
                } else {
                    // Fallback: If neither Relax nor Idle exist, pick the first available one.
                    // We already know availableAnimations.length > 0 from the outer check.
                    newAnimation = availableAnimations[0];
                }
            } else {
                // Front or Back view
                if (availableAnimations.includes("Idle")) {
                    newAnimation = "Idle";
                } else {
                    // Fallback: If Idle doesn't exist, pick the first available one.
                    // We already know availableAnimations.length > 0 from the outer check.
                    newAnimation = availableAnimations[0];
                }
            }

            // Only update state and spine if we found a valid new animation
            // and it's different from the current one
            if (newAnimation && newAnimation !== selectedAnimation) {
                setSelectedAnimation(newAnimation);
                // Also apply it immediately to the spine if it exists
                if (spineRef.current) {
                    spineRef.current.state.setAnimation(0, newAnimation, true);
                    spineRef.current.state.timeScale = ANIMATION_SPEED;
                }
            }
        }
    }, [viewType, availableAnimations, selectedAnimation]);

    const handleAnimationChange = (value: string) => {
        setSelectedAnimation(value);

        // Apply the animation to the spine object
        if (spineRef.current && value) {
            // Set the animation
            spineRef.current.state.setAnimation(0, value, true);
            // Ensure the timeScale is maintained
            spineRef.current.state.timeScale = ANIMATION_SPEED;
        }
    };

    const handleViewTypeChange = (value: string) => {
        setViewType(value as ChibiAnimation);
    };

    return (
        <Card className="w-full">
            <CardContent className="pb-4 pt-6">
                <div className="mb-4 flex flex-col gap-4">
                    <div className="flex items-center gap-2">
                        <Select value={selectedAnimation} onValueChange={handleAnimationChange} disabled={!spineRef.current || availableAnimations.length === 0 || isLoading}>
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

                <div className="relative h-[200px] w-full">
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
