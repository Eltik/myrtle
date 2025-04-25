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

type ChibiRendererProps = {
    selectedOperator: FormattedChibis | null;
    selectedSkin: string | null;
};

const CHIBI_OFFSET_X = 0.25;
const CHIBI_OFFSET_Y = 0.43;
const CHIBI_SCALE = 0.7;
const ANIMATION_SPEED = 0.5; // Animation speed multiplier (lower = slower)

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

        // First, load the atlas file to get the expected image dimensions
        fetch(skinData.atlas)
            .then(response => response.text())
            .then(atlasText => {
                // Parse the atlas file to extract image dimensions
                const sizeRegex = /size: (\d+),(\d+)/;
                const imageMatch = sizeRegex.exec(atlasText);
                if (!imageMatch) {
                    throw new Error("Could not find image dimensions in atlas file");
                }
                
                const [_, width, height] = imageMatch;
                console.log("Expected image dimensions:", { width, height });
                
                // Use a more direct approach to load the image and create a texture with the correct dimensions
                return new Promise<void>((resolve, reject) => {
                    // Create a unique key for the texture
                    const textureKey = `custom_${skinData.png}`;
                    
                    // Load the image directly
                    const img = new Image();
                    img.onload = () => {
                        try {
                            // Create a texture from the image
                            const texture = PIXI.Texture.from(img);
                            
                            // Create a new texture with the correct dimensions
                            const newTexture = new PIXI.Texture(
                                texture.baseTexture,
                                new PIXI.Rectangle(0, 0, Number(width), Number(height))
                            );
                            
                            // Add the texture to the cache
                            PIXI.Assets.cache.set(textureKey, newTexture);
                            
                            // Now load the skeleton
                            PIXI.Assets.load(skinData.skel)
                                .then(() => {
                                    resolve();
                                })
                                .catch((error) => {
                                    reject(error instanceof Error ? error : new Error(String(error)));
                                });
                        } catch (error) {
                            reject(error instanceof Error ? error : new Error(String(error)));
                        }
                    };
                    
                    img.onerror = () => {
                        reject(new Error("Failed to load image"));
                    };
                    
                    img.src = skinData.png;
                });
            })
            .then(() => {
                // Now that we've loaded the assets, we can render the spine
                const spineAsset = PIXI.Assets.cache.get(skinData.skel);
                console.log("Loaded asset:", spineAsset);
                
                // Check if the asset has the required properties
                if (spineAsset && 
                    typeof spineAsset === 'object' && 
                    'spineData' in spineAsset && 
                    'spineAtlas' in spineAsset) {
                    // We've verified the structure, now we can safely cast
                    const typedSpineAsset = spineAsset as { 
                        spineAtlas: TextureAtlas; 
                        spineData: ISkeletonData 
                    };
                    renderSkinSpine(typedSpineAsset);
                } else {
                    throw new Error("Invalid spine data loaded");
                }
            })
            .catch(err => {
                console.error("Failed to load assets:", err);
                setError(`Failed to load assets: ${err instanceof Error ? err.message : String(err)}`);
                setIsLoading(false);

                // If we get the specific "Cannot read properties of undefined (reading 'image')" error,
                // provide a more helpful error message
                if (err instanceof Error && err.message.includes("Cannot read properties of undefined (reading 'image')")) {
                    setError("Error loading spine: The atlas file couldn't find the image. Please check that image paths match.");
                }
            });
    }, [selectedOperator, selectedSkin, viewType, renderSkinSpine]);

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
