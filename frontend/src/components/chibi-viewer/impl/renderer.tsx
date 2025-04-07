import { useCallback, useEffect, useRef, useState } from "react";
import type { FormattedChibis } from "~/types/impl/frontend/impl/chibis";
import * as PIXI from "pixi.js";
import { type ISkeletonData, Spine, type TextureAtlas } from "pixi-spine";
import { getSkinData } from "./helper";

import { Card, CardContent } from "~/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";

type ChibiRendererProps = {
    selectedOperator: FormattedChibis | null;
    selectedSkin: string | null;
    repoBaseURL: string;
};

export function ChibiRenderer({ selectedOperator, selectedSkin, repoBaseURL }: ChibiRendererProps) {
    const appRef = useRef<PIXI.Application | null>(null);
    const spineRef = useRef<Spine | null>(null);
    
    const [selectedAnimation, setSelectedAnimation] = useState<string>("Idle");
    const [availableAnimations, setAvailableAnimations] = useState<string[]>([]);
    const [viewType, setViewType] = useState<"dorm" | "front" | "back">("front");

    const canvasContainerRef = useRef<HTMLDivElement>(null);
    const [isDragging, setIsDragging] = useState(false);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [dragStart, setDragStart] = useState<{ x: number; y: number } | null>(null);

    // Function to handle mouse down for dragging
    const handleMouseDown = useCallback((e: React.MouseEvent) => {
        if (!spineRef.current) return;
        
        setIsDragging(true);
        setDragStart({
            x: e.clientX - spineRef.current.x,
            y: e.clientY - spineRef.current.y
        });
    }, []);

    // Function to handle mouse move for dragging
    const handleMouseMove = useCallback((e: React.MouseEvent) => {
        if (!spineRef.current || !isDragging || !dragStart) return;
        
        spineRef.current.x = e.clientX - dragStart.x;
        spineRef.current.y = e.clientY - dragStart.y;
        
        if (appRef.current) {
            appRef.current.renderer.render(appRef.current.stage);
        }
    }, [isDragging, dragStart]);

    // Function to handle mouse up to stop dragging
    const handleMouseUp = useCallback(() => {
        setIsDragging(false);
        setDragStart(null);
    }, []);

    // Function to reset position
    const resetPosition = useCallback(() => {
        if (!spineRef.current || !appRef.current) return;
        
        // Reset to center
        spineRef.current.x = appRef.current.screen.width / 2;
        spineRef.current.y = appRef.current.screen.height * 0.5;
        
        appRef.current.renderer.render(appRef.current.stage);
    }, []);

    const renderSkinSpine = useCallback((skinAsset: {
        spineAtlas: TextureAtlas;
        spineData: ISkeletonData;
    }) => {
        spineRef.current = new Spine(skinAsset.spineData);

        setAvailableAnimations(skinAsset.spineData.animations.map((animation) => animation.name));

        if (appRef.current) {
            spineRef.current.x = appRef.current.screen.width / 2;
            
            if (selectedAnimation === "Sit" || selectedAnimation === "Sitting") {
                spineRef.current.y = appRef.current.screen.height * 0.6;
            } else {
                spineRef.current.y = appRef.current.screen.height * 0.5;
            }
        }

        spineRef.current.scale.set(0.8);

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

        spineRef.current.interactive = true;
        spineRef.current.alpha = 1;

        spineRef.current.state.addListener({
            complete: (event) => {
                console.log(event);
                if (event.animationEnd && viewType !== 'dorm') {
                    spineRef.current?.state.setAnimation(0, 'Idle', true);
                }
            }
        })

        appRef.current?.stage.addChild(spineRef.current);
        appRef.current?.renderer.render(appRef.current?.stage);

        console.log("Finished rendering");

        setIsLoading(false);
    }, [selectedAnimation, viewType]);

    const renderCanvas = useCallback(() => {
        const skinData = getSkinData(selectedOperator, selectedSkin, repoBaseURL, viewType);
        if (!skinData) {
            setError("Failed to load skin data");
            return;
        }

        setIsLoading(true);
        setError(null);

        void PIXI.Assets.load(skinData.skel).then((skinAsset: {
            spineAtlas: TextureAtlas;
            spineData: ISkeletonData;
        }) => {
            console.log("Skin asset loaded:", skinAsset);
            renderSkinSpine(skinAsset);
        }).catch((err) => {
            console.error("Failed to load primary asset:", err);
            // Try and load build version
        });
    }, [selectedOperator, selectedSkin, repoBaseURL, viewType, renderSkinSpine]);

    useEffect(() => {
        if (!canvasContainerRef.current) {
            return;
        }

        if (appRef.current) {
            appRef.current.destroy(true, true);
        }

        const pixiApp = new PIXI.Application({
            width: 800,
            height: 400,
            backgroundColor: 0x111014,
            antialias: true,
            premultipliedAlpha: true,
            backgroundAlpha: 1,
            clearBeforeRender: true,
            hello: true,
            powerPreference: "high-performance",
            preserveDrawingBuffer: true,
            context: null,
        });
        appRef.current = pixiApp;

        renderCanvas();
        canvasContainerRef.current?.appendChild(pixiApp.view as unknown as Node);

        setIsLoading(false);

        return () => {
            if (spineRef.current) {
                spineRef.current.destroy();
                spineRef.current = null;
            }
            
            if (appRef.current) {
                appRef.current.destroy(true, true);
                appRef.current = null;
            }
        };
    }, [renderCanvas]);

    const handleAnimationChange = (value: string) => {
        setSelectedAnimation(value);

        // Apply the animation to the spine object
        if (spineRef.current && value) {
            // Set the animation
            spineRef.current.state.setAnimation(0, value, true);
        }
    };

    const handleViewTypeChange = (value: string) => {
        setViewType(value as "dorm" | "front" | "back");
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
                            <button 
                                onClick={resetPosition}
                                className="rounded-md bg-gray-200 px-3 py-1 text-sm hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600"
                                disabled={isLoading}
                            >
                                Center
                            </button>
                        </div>
                    </div>
                    {/* Display area */}
                    <div 
                        className="relative h-[400px] w-full bg-[#111014]"
                        onMouseDown={handleMouseDown}
                        onMouseMove={handleMouseMove}
                        onMouseUp={handleMouseUp}
                        onMouseLeave={handleMouseUp}
                    >
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
    )
}
