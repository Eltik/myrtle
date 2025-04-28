import { useCallback, useEffect, useRef, useState } from "react";
import * as PIXI from "pixi.js";
import { type ISkeletonData, Spine, type TextureAtlas } from "pixi-spine";
import { getSkinData } from "./helper";

import { Card, CardContent } from "~/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import type { ChibiAnimation, FormattedChibis } from "~/types/impl/frontend/impl/chibis";
import { getCDNURL } from "~/lib/cdn";

// Helper function to create safe Rectangles with number values
const createSafeRectangle = (x: number | string = 0, y: number | string = 0, width: number | string = 0, height: number | string = 0): PIXI.Rectangle => {
    return new PIXI.Rectangle(
        Number(x),
        Number(y),
        Number(width),
        Number(height)
    );
};

// Define resource types for the loader
interface SpineResource {
    spineData?: ISkeletonData;
    name?: string;
}

type ChibiRendererProps = {
    selectedOperator: FormattedChibis | null;
    selectedSkin: string | null;
};

const CHIBI_OFFSET_X = 0.25;
const CHIBI_OFFSET_Y = 0.43;
const CHIBI_SCALE = 0.7;
const ANIMATION_SPEED = 0.5; // Animation speed multiplier (lower = slower)

// Helper to remove out-of-bounds frames from the atlas
const clampAtlasFrames = (atlasContent: string, pngWidth: number, pngHeight: number): string => {
    const lines = atlasContent.split('\n');
    let output: string[] = [];
    let frameStartIdx = -1;
    let currentFrame: string | null = null;
    let xy: [number, number] | null = null;
    let size: [number, number] | null = null;
    for (const line of lines) {
        // Detect start of a frame
        if (line.trim() && !line.startsWith(' ') && !line.includes(':')) {
            if (
                currentFrame &&
                xy && size &&
                (xy[0] + size[0] > pngWidth || xy[1] + size[1] > pngHeight)
            ) {
                output = output.slice(0, frameStartIdx);
            }
            currentFrame = line;
            xy = null;
            size = null;
            frameStartIdx = output.length;
        }
        // Parse xy and size
        if (line.trim().startsWith('xy:')) {
            const match = /xy:\s*(\d+),\s*(\d+)/.exec(line);
            xy = match?.[1] && match?.[2] ? [Number(match[1]), Number(match[2])] : xy;
        }
        if (line.trim().startsWith('size:')) {
            const match = /size:\s*(\d+),\s*(\d+)/.exec(line);
            size = match?.[1] && match?.[2] ? [Number(match[1]), Number(match[2])] : size;
        }
        output.push(line);
    }
    if (
        currentFrame &&
        xy && size &&
        (xy[0] + size[0] > pngWidth || xy[1] + size[1] > pngHeight)
    ) {
        output = output.slice(0, frameStartIdx);
    }
    return output.join('\n');
};

// Function to modify the atlas file to match the PNG size (440x440)
const modifyAtlasFile = async (atlasUrl: string, actualWidth: number, actualHeight: number): Promise<string> => {
    try {
        const response = await fetch(atlasUrl);
        if (!response.ok) throw new Error(`Failed to fetch atlas file: ${response.statusText}`);
        let atlasContent = await response.text();
        // Remove out-of-bounds frames
        atlasContent = clampAtlasFrames(atlasContent, actualWidth, actualHeight);
        const blob = new Blob([atlasContent], { type: 'text/plain' });
        return URL.createObjectURL(blob);
    } catch (err) {
        console.error("Error modifying atlas file:", err);
        return atlasUrl;
    }
};

export function ChibiRenderer({ selectedOperator, selectedSkin }: ChibiRendererProps) {
    const appRef = useRef<PIXI.Application | null>(null);
    const spineRef = useRef<Spine | null>(null);

    const [selectedAnimation, setSelectedAnimation] = useState<string>("Idle");
    const [availableAnimations, setAvailableAnimations] = useState<string[]>([]);
    const [viewType, setViewType] = useState<ChibiAnimation>("front");

    const canvasContainerRef = useRef<HTMLDivElement>(null);
    const [isDragging, setIsDragging] = useState(false);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [dragStart, setDragStart] = useState<{ x: number; y: number } | null>(null);

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

    // Function to handle mouse down for dragging
    const handleMouseDown = useCallback((e: React.MouseEvent) => {
        if (!spineRef.current) return;

        setIsDragging(true);
        setDragStart({
            x: e.clientX - spineRef.current.x,
            y: e.clientY - spineRef.current.y,
        });
    }, []);

    // Function to handle mouse move for dragging
    const handleMouseMove = useCallback(
        (e: React.MouseEvent) => {
            if (!spineRef.current || !isDragging || !dragStart) return;

            spineRef.current.x = e.clientX - dragStart.x;
            spineRef.current.y = e.clientY - dragStart.y;

            if (appRef.current) {
                appRef.current.renderer.render(appRef.current.stage);
            }
        },
        [isDragging, dragStart],
    );

    // Function to handle mouse up to stop dragging
    const handleMouseUp = useCallback(() => {
        setIsDragging(false);
        setDragStart(null);
    }, []);

    // Function to reset position
    const resetPosition = useCallback(() => {
        if (!spineRef.current || !appRef.current) return;

        const { width, height } = appRef.current.screen;
        
        // Reset to adjusted center position
        spineRef.current.x = width * CHIBI_OFFSET_X; // Move further left from center
        spineRef.current.y = height * CHIBI_OFFSET_Y; // Keep the good vertical position

        appRef.current.renderer.render(appRef.current.stage);
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
                // Create a new spine object
                spineRef.current = new Spine(skinAsset.spineData);
                
                // Set the animation speed to 1x (slower)
                spineRef.current.state.timeScale = ANIMATION_SPEED;

                // Set available animations
                setAvailableAnimations(skinAsset.spineData.animations.map((animation) => animation.name));

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
                appRef.current?.stage.addChild(spineRef.current);

                // Render the stage
                appRef.current?.renderer.render(appRef.current?.stage);

                console.log("Finished rendering");
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
        if (!selectedOperator) {
            setError("No operator selected");
            setIsLoading(false);
            return;
        }
        // Get the skin data using the updated function
        const skinData = getSkinData(selectedOperator, selectedSkin ?? "default", viewType);
        if (!skinData) {
            console.error("Failed to load skin data", { selectedOperator, selectedSkin, viewType });
            setError("Failed to load skin data - check console for details");
            setIsLoading(false);
            return;
        }
        Object.assign(skinData, {
            atlas: getCDNURL(skinData.atlas, true),
            png: getCDNURL(skinData.png, true),
            skel: getCDNURL(skinData.skel, true),
        });
        setIsLoading(true);
        setError(null);
        console.log("Loading skin assets:", {
            atlas: skinData.atlas,
            png: skinData.png,
            skel: skinData.skel,
        });
        if (!appRef.current) {
            setError("PIXI application not initialized");
            setIsLoading(false);
            return;
        }
        const app = appRef.current;
        const loader = app.loader as PIXI.Loader;
        loader.reset();
        try {
            const loadId = Date.now().toString();
            // Patch the atlas file to match the PNG size (440x440)
            void (async () => {
                const modifiedAtlasUrl = await modifyAtlasFile(skinData.atlas, 440, 440);
                if (typeof loader.add === 'function') loader.add(`chibi_atlas_${loadId}`, modifiedAtlasUrl);
                if (typeof loader.add === 'function') loader.add(`chibi_skel_${loadId}`, skinData.skel);
                if (typeof loader.add === 'function') loader.add(`chibi_image_${loadId}`, skinData.png);
                if (typeof loader.load === 'function') {
                    void loader.load((_: unknown, resources: Record<string, SpineResource>) => {
                        try {
                            if (spineRef.current?.parent) {
                                spineRef.current.parent.removeChild(spineRef.current);
                                spineRef.current.destroy();
                                spineRef.current = null;
                            }
                            const skelResource = resources[`chibi_skel_${loadId}`];
                            const atlasResource = resources[`chibi_atlas_${loadId}`];
                            if (!skelResource || !atlasResource) {
                                setError("Failed to load spine resources");
                                setIsLoading(false);
                                return;
                            }
                            if (skelResource.spineData) {
                                const spineData = new Spine(skelResource.spineData);
                                spineData.state.timeScale = ANIMATION_SPEED;
                                const animations = spineData.spineData.animations.map((animation: { name: string }) => animation.name);
                                setAvailableAnimations(animations);
                                if (selectedAnimation && spineData.spineData.findAnimation(selectedAnimation)) {
                                    spineData.state.setAnimation(0, selectedAnimation, true);
                                } else {
                                    if (viewType !== "dorm") {
                                        if (spineData.spineData.findAnimation("Start")) {
                                            spineData.state.setAnimation(0, "Start", false);
                                        } else if (spineData.spineData.findAnimation("Start_A")) {
                                            spineData.state.setAnimation(0, "Start_A", false);
                                        } else {
                                            spineData.state.setAnimation(0, "Idle", true);
                                        }
                                    } else {
                                        if (spineData.spineData.findAnimation("Relax")) {
                                            spineData.state.setAnimation(0, "Relax", true);
                                        } else {
                                            spineData.state.setAnimation(0, "Idle", true);
                                        }
                                    }
                                }
                                if (appRef.current) {
                                    const { width, height } = appRef.current.screen;
                                    spineData.x = width * CHIBI_OFFSET_X;
                                    spineData.y = height * CHIBI_OFFSET_Y;
                                    const scale = Math.min(width / 1000, height / 800) * CHIBI_SCALE;
                                    spineData.scale.set(scale);
                                }
                                spineData.state.addListener({
                                    complete: (event) => {
                                        if (event.animationEnd && viewType !== "dorm") {
                                            const track = spineData.state.tracks[0];
                                            if (track && ((track as unknown as { animation: { name: string } }).animation.name === "Start" || (track as unknown as { animation: { name: string } }).animation.name === "Start_A")) {
                                                spineData.state.setAnimation(0, "Idle", true);
                                            }
                                        }
                                    },
                                });
                                appRef.current?.stage.addChild(spineData);
                                spineRef.current = spineData;
                                appRef.current?.renderer.render(appRef.current?.stage);
                                console.log("Finished rendering");
                                setIsLoading(false);
                            } else {
                                setError("Invalid spine data format");
                                setIsLoading(false);
                            }
                        } catch (err) {
                            console.error("Spine rendering error:", err);
                            setError(`Failed to render chibi: ${err instanceof Error ? err.message : String(err)}`);
                            setIsLoading(false);
                        }
                    });
                }
                if (loader.onError && typeof loader.onError.add === 'function') {
                    loader.onError.add((_: unknown, resource: unknown) => {
                        let name = 'unknown';
                        if (resource && typeof resource === 'object' && 'name' in resource && typeof (resource as { name?: unknown }).name === 'string') {
                            name = (resource as { name: string }).name;
                        }
                        console.error("Loader error:", resource);
                        setError(`Failed to load resource: ${name}`);
                        setIsLoading(false);
                    });
                }
            })();
        } catch (err) {
            console.error("Loader setup error:", err);
            setError(`Failed to set up loader: ${err instanceof Error ? err.message : String(err)}`);
            setIsLoading(false);
        }
    }, [selectedOperator, selectedSkin, viewType, selectedAnimation]);

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

        window.addEventListener('resize', handleResize);
        
        renderCanvas();

        return () => {
            window.removeEventListener('resize', handleResize);
            
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
        <>
            <Card className="w-full">
                <CardContent className="pb-4 pt-6">
                    <div className="mb-4 flex flex-col gap-4">
                        <div className="flex items-center gap-2">
                            <Select value={selectedAnimation} onValueChange={handleAnimationChange} disabled={availableAnimations.length === 0 || isLoading}>
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

                        {/* Reset Position Button */}
                        <div className="flex justify-end">
                            <button onClick={resetPosition} className="rounded-md bg-gray-200 px-3 py-1 text-sm hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600" disabled={isLoading}>
                                Center
                            </button>
                        </div>
                    </div>
                    {/* Display area */}
                    <div className="relative h-[400px] w-full overflow-hidden bg-[#111014]" onMouseDown={handleMouseDown} onMouseMove={handleMouseMove} onMouseUp={handleMouseUp} onMouseLeave={handleMouseUp}>
                        <div ref={canvasContainerRef} className="h-full w-full" style={{ cursor: isDragging ? "grabbing" : "grab" }} />
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
                        <div className="absolute bottom-2 right-2 text-xs text-gray-500">Drag to reposition • Click &quot;Center&quot; to reset</div>
                    </div>
                </CardContent>
            </Card>
        </>
    );
}
