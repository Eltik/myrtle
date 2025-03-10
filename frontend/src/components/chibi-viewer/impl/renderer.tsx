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

type ChibiRendererProps = {
    selectedOperator: FormattedChibis | null;
    selectedSkin: string | null;
    repoBaseUrl: string;
};

export function ChibiRenderer({ selectedOperator, selectedSkin, repoBaseUrl }: ChibiRendererProps) {
    const appRef = useRef<PIXI.Application | null>(null);
    const spineRef = useRef<Spine | null>(null);

    const [selectedAnimation, setSelectedAnimation] = useState<string>("Idle");
    const [availableAnimations, setAvailableAnimations] = useState<string[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isPlaying, setIsPlaying] = useState(true);
    const [speed, setSpeed] = useState(1);
    const [useStaticFallback, setUseStaticFallback] = useState(false);
    const staticImageContainerRef = useRef<HTMLDivElement>(null);
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
            autoDensity: false, // Disable automatic resizing
            resolution: window.devicePixelRatio || 1,
        });

        canvasContainerRef.current.innerHTML = "";
        canvasContainerRef.current.appendChild(pixiApp.view);
        appRef.current = pixiApp; // Store in ref instead of state

        return () => {
            if (appRef.current) {
                appRef.current.destroy(true, true);
                appRef.current = null;
            }
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

    // Get skin data
    const getSkinData = useCallback(() => {
        if (!selectedOperator || !selectedSkin) {
            return null;
        }

        // Find the selected skin
        const skin = selectedOperator.skins.find((s) => s.dorm.path === selectedSkin || s.front.path === selectedSkin || s.back.path === selectedSkin);

        if (!skin) {
            return null;
        }

        // Determine which view to use (dorm, front, or back)
        if (selectedSkin === skin.dorm.path && skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
            return {
                atlas: getAssetUrl(skin.dorm.atlas),
                png: getAssetUrl(skin.dorm.png),
                skel: getAssetUrl(skin.dorm.skel),
                type: "dorm",
            };
        } else if (selectedSkin === skin.front.path && skin.front.atlas && skin.front.png && skin.front.skel) {
            return {
                atlas: getAssetUrl(skin.front.atlas),
                png: getAssetUrl(skin.front.png),
                skel: getAssetUrl(skin.front.skel),
                type: "front",
            };
        } else if (selectedSkin === skin.back.path && skin.back.atlas && skin.back.png && skin.back.skel) {
            return {
                atlas: getAssetUrl(skin.back.atlas),
                png: getAssetUrl(skin.back.png),
                skel: getAssetUrl(skin.back.skel),
                type: "back",
            };
        }

        return null;
    }, [selectedOperator, selectedSkin, getAssetUrl]);

    useEffect(() => {
        const skin = selectedOperator?.skins.find((data) => {
            const isBack = Object.values(data.back).map((data) => data === selectedSkin);
            const isDorm = Object.values(data.dorm).map((data) => data === selectedSkin);
            const isFront = Object.values(data.front).map((data) => data === selectedSkin);

            if (!isBack && !isDorm && !isFront) return false;
            return true;
        });

        const isBack = Object.values(skin?.back ?? {}).map((data) => data === selectedSkin);
        const isDorm = Object.values(skin?.dorm ?? {}).map((data) => data === selectedSkin);
        const isFront = Object.values(skin?.front ?? {}).map((data) => data === selectedSkin);

        const skinData = isBack
            ? skin?.back
            : isDorm
              ? skin?.dorm
              : isFront
                ? skin?.front
                : {
                      path: null,
                      atlas: null,
                      png: null,
                      skel: null,
                  };

        if (!appRef.current || !skinData?.atlas || !skinData.skel || !skinData.png) return;

        const atlasURL = getAssetUrl(skinData.atlas);
        const skelURL = getAssetUrl(skinData.skel);
        const imageURL = getAssetUrl(skinData.png);

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

            loader.load((_, resources) => {
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
                        const spineData = new Spine(skelResource.spineData);

                        // Center the sprite
                        spineData.x = app.screen.width / 2;
                        spineData.y = app.screen.height / 2;

                        // Scale it appropriately
                        spineData.scale.set(0.5);

                        // Add to stage
                        app.stage.addChild(spineData);

                        // Get available animations - casting to our minimal interface
                        const animations = spineData.spineData.animations.map((anim: SpineAnimation) => anim.name);
                        setAvailableAnimations(animations);

                        // Store spine in ref
                        spineRef.current = spineData;

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
                } catch (e) {
                    console.error("Error setting up spine animation:", e);
                    setError("Error setting up animation");
                } finally {
                    setIsLoading(false);
                }
            });

            loader.onError.add(() => {
                setError("Failed to load spine resources");
                setIsLoading(false);
            });
        } catch (e) {
            setError(e.message);
        }
    }, [selectedOperator, selectedSkin, repoBaseUrl, getAssetUrl]);

    useEffect(() => {
        // Only try to change animation if spine exists and animation name is valid
        if (spineRef.current?.state && selectedAnimation && availableAnimations.includes(selectedAnimation)) {
            try {
                spineRef.current.state.setAnimation(0, selectedAnimation, true);
            } catch (error) {
                console.error("Failed to set animation:", error);
            }
        }
    }, [selectedAnimation, availableAnimations]);

    // Handle animation change
    const handleAnimationChange = (value: string) => {
        setSelectedAnimation(value);

        // Reset the image position and transform
        if (canvasContainerRef.current?.firstChild) {
            const imageElement = canvasContainerRef.current.firstChild as HTMLImageElement;
            imageElement.style.left = "50%";
            imageElement.style.top = "50%";
            imageElement.style.transform = "translate(-50%, -50%)";
            imageElement.style.opacity = "1";
            imageElement.dataset.rotation = "0";
        }
    };

    // Handle play/pause
    const handlePlayPause = () => {
        setIsPlaying(!isPlaying);
    };

    // Handle reset
    const handleReset = () => {
        // Reset the image position and transform
        if (canvasContainerRef.current?.firstChild) {
            const imageElement = canvasContainerRef.current.firstChild as HTMLImageElement;
            imageElement.style.left = "50%";
            imageElement.style.top = "50%";
            imageElement.style.transform = "translate(-50%, -50%)";
            imageElement.style.opacity = "1";
            imageElement.dataset.rotation = "0";
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

    // Function to show static image
    const renderStaticImage = useCallback(() => {
        if (!selectedOperator || !selectedSkin) {
            return (
                <div className="flex h-full items-center justify-center">
                    <p className="text-muted-foreground">Select an operator and skin to view animations</p>
                </div>
            );
        }

        const skinData = getSkinData();
        if (!skinData) {
            return (
                <div className="flex h-full items-center justify-center">
                    <p className="text-muted-foreground">No skin data available</p>
                </div>
            );
        }

        return (
            <div className="flex h-full flex-col items-center justify-center">
                <img
                    src={skinData.png}
                    alt={`${selectedOperator.name} - static image`}
                    className="max-h-full max-w-full object-contain"
                    onError={(e) => {
                        e.currentTarget.style.display = "none";
                        if (staticImageContainerRef.current) {
                            const fallbackText = document.createElement("p");
                            fallbackText.className = "text-muted-foreground";
                            fallbackText.innerText = "Character image not available";
                            staticImageContainerRef.current.appendChild(fallbackText);
                        }
                    }}
                />
                <p className="mt-2 text-sm text-muted-foreground">Static image shown - animation type: {selectedAnimation}</p>
            </div>
        );
    }, [selectedOperator, selectedSkin, selectedAnimation, getSkinData]);

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
                    <div ref={canvasContainerRef} className="h-full w-full" style={{ display: !isLoading && !error && !useStaticFallback ? "block" : "none" }}></div>

                    {/* Static image fallback */}
                    <div ref={staticImageContainerRef} className="h-full w-full" style={{ display: (useStaticFallback || !spineData) && !isLoading && !error ? "block" : "none" }}>
                        {renderStaticImage()}
                    </div>
                </div>

                {hasAnimation && !useStaticFallback && !isLoading && !error && (
                    <div className="mt-2">
                        <p className="text-xs text-muted-foreground">Note: This is a simplified animation using CSS transforms. For full Spine animations:</p>
                        <ul className="mt-1 list-disc pl-5 text-xs text-muted-foreground">
                            <li>Add pixi-spine integration to load .skel and .atlas files</li>
                            <li>Use Spine-specific animation controls for rich animations</li>
                            <li>Handle bone animations and skeletal deformations</li>
                        </ul>
                    </div>
                )}

                {useStaticFallback && !isLoading && !error && (
                    <div className="mt-2">
                        <p className="text-xs text-muted-foreground">Note: Static image is shown as a fallback.</p>
                    </div>
                )}
            </CardContent>
        </Card>
    );
}
