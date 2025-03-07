import { useEffect, useRef, useState } from "react";
import type { FormattedChibis } from "~/types/impl/frontend/impl/chibis";
import { Card, CardContent } from "~/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { Slider } from "~/components/ui/slider";
import { Button } from "~/components/ui/button";
import { PauseIcon, PlayIcon, RepeatIcon } from "lucide-react";

// We'll need to install this package:
// npm install @esotericsoftware/spine-player
// The following import will work after installation:
// import { SpinePlayer as SpinePlayerLib } from '@esotericsoftware/spine-player';
// import '@esotericsoftware/spine-player/dist/spine-player.css';

// Type definitions for Spine objects - will make TypeScript happy until we have actual imports
type SpineAnimation = {
    name: string;
};

// The actual Spine player in the library
type SpinePlayer = {
    dispose: () => void;
    setAnimation: (animationName: string) => void;
    play: () => void;
    pause: () => void;
    timeScale: number;
    skeleton: {
        data: {
            animations: SpineAnimation[];
        };
    };
};

// Options for creating a Spine player instance
type SpinePlayerOptions = {
    jsonUrl: string;
    atlasUrl: string;
    png?: string;
    animation?: string;
    premultipliedAlpha?: boolean;
    backgroundColor?: string;
    showControls?: boolean;
    preserveDrawingBuffer?: boolean;
    alpha?: boolean;
    success?: (player: SpinePlayer) => void;
    error?: (player: SpinePlayer, message: string) => void;
    viewport?: {
        debugRender?: boolean;
        showBackground?: boolean;
    };
};

// The constructor function
type SpinePlayerConstructor = new (container: HTMLElement, options: SpinePlayerOptions) => SpinePlayer;

// Declare SpinePlayer as a global to avoid TS errors
declare global {
    interface Window {
        SpinePlayer?: SpinePlayerConstructor;
    }
}

const SpinePlayer: SpinePlayerConstructor | undefined = typeof window !== "undefined" ? window.SpinePlayer : undefined;

type AnimationType = "idle" | "skill" | "move" | "die" | "attack" | "special";

type ChibiRendererProps = {
    selectedOperator: FormattedChibis | null;
    selectedSkin: string | null;
    repoBaseUrl: string;
};

export function ChibiRenderer({ selectedOperator, selectedSkin, repoBaseUrl }: ChibiRendererProps) {
    const canvasContainerRef = useRef<HTMLDivElement>(null);
    const spinePlayerRef = useRef<SpinePlayer | null>(null);
    const [selectedAnimation, setSelectedAnimation] = useState<AnimationType>("idle");
    const [availableAnimations, setAvailableAnimations] = useState<AnimationType[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isPlaying, setIsPlaying] = useState(true);
    const [speed, setSpeed] = useState(1);

    // Function to get asset URL from the path
    const getAssetUrl = (path: string) => {
        // Remove the initial "./" if present
        const normalizedPath = path.startsWith("./") ? path.substring(2) : path;
        return `${repoBaseUrl}${normalizedPath}`;
    };

    // Effect to dispose of Spine player on unmount
    useEffect(() => {
        return () => {
            if (spinePlayerRef.current) {
                try {
                    spinePlayerRef.current.dispose();
                } catch (err) {
                    console.error("Error disposing Spine player:", err);
                }
            }
        };
    }, []);

    // Effect to load Spine data when operator, skin, or animation changes
    useEffect(() => {
        if (!selectedOperator || !selectedSkin) return;

        setIsLoading(true);
        setError(null);

        // Find the selected skin data
        const skinData = selectedOperator.skins.find((skin) => skin.dorm.path === selectedSkin);

        if (!skinData) {
            setError("Selected skin data not found");
            setIsLoading(false);
            return;
        }

        // Clean up previous spine player if exists
        if (spinePlayerRef.current) {
            try {
                spinePlayerRef.current.dispose();
                spinePlayerRef.current = null;
            } catch (err) {
                console.error("Error disposing previous spine player:", err);
            }
        }

        // Clear container
        if (canvasContainerRef.current) {
            canvasContainerRef.current.innerHTML = "";
        }

        const loadSpineData = async () => {
            try {
                // In a real implementation with the package installed, we would do:
                if (typeof SpinePlayer !== "undefined") {
                    const atlas = getAssetUrl(skinData.dorm.atlas);
                    const skel = getAssetUrl(skinData.dorm.skel);
                    const png = getAssetUrl(skinData.dorm.png);

                    // Create new spine player
                    if (canvasContainerRef.current) {
                        spinePlayerRef.current = new SpinePlayer(canvasContainerRef.current, {
                            jsonUrl: skel,
                            atlasUrl: atlas,
                            png,
                            animation: selectedAnimation,
                            premultipliedAlpha: true,
                            backgroundColor: "#00000000",
                            showControls: false,
                            preserveDrawingBuffer: true,
                            alpha: true,
                            viewport: {
                                debugRender: false,
                                showBackground: false,
                            },
                            success: (player) => {
                                // Get available animations from spine data
                                const animations = player.skeleton.data.animations;
                                const animNames = animations.map((anim) => anim.name);

                                // Filter to our recognized animation types
                                const availableAnims: AnimationType[] = [];
                                const recognizedTypes: AnimationType[] = ["idle", "move", "attack", "skill", "die", "special"];

                                for (const type of recognizedTypes) {
                                    // Look for animations containing the type name
                                    const matchingAnims = animNames.filter((name) => name.toLowerCase().includes(type));

                                    if (matchingAnims.length > 0) {
                                        availableAnims.push(type);
                                    }
                                }

                                // If no recognized animations found, just use the first animation
                                if (availableAnims.length === 0 && animations.length > 0) {
                                    availableAnims.push("idle");
                                }

                                setAvailableAnimations(availableAnims);

                                // Auto-select first available animation if current is not available
                                if (!availableAnims.includes(selectedAnimation) && availableAnims.length > 0) {
                                    // Use non-null assertion (!) as per linter suggestion
                                    const firstAnim = availableAnims[0]!;
                                    setSelectedAnimation(firstAnim);
                                    player.setAnimation(firstAnim);
                                }

                                // Set speed
                                player.timeScale = speed;

                                setIsLoading(false);
                            },
                            error: (_, errorMessage) => {
                                console.error("Error loading spine data:", errorMessage);
                                setError(`Failed to load animation: ${errorMessage}`);
                                setIsLoading(false);

                                // Fallback to placeholder
                                drawPlaceholder(selectedOperator, skinData);
                            },
                        });
                    }
                } else {
                    // Fallback when SpinePlayer is not available (package not installed)
                    console.warn("SpinePlayer not found. Using placeholder instead.");
                    drawPlaceholder(selectedOperator, skinData);

                    // Set default available animations for placeholder
                    setAvailableAnimations(["idle", "move", "attack", "skill", "die", "special"]);
                    setIsLoading(false);
                }
            } catch (err) {
                console.error("Error loading spine data:", err);
                setError("Failed to load animation data");
                setIsLoading(false);

                // Fallback to placeholder
                drawPlaceholder(selectedOperator, skinData);
            }
        };

        void loadSpineData();
    }, [selectedOperator, selectedSkin, selectedAnimation, repoBaseUrl, speed]);

    // Helper function to draw placeholder when spine is not available
    const drawPlaceholder = (operator: FormattedChibis, skinData: FormattedChibis["skins"][0]) => {
        if (!canvasContainerRef.current) return;

        // Create a canvas for placeholder
        const canvas = document.createElement("canvas");
        canvas.width = 400;
        canvas.height = 400;
        canvas.className = "w-full h-full";
        canvasContainerRef.current.appendChild(canvas);

        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        // Draw placeholder text
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.font = "16px Arial";
        ctx.fillStyle = "#888";
        ctx.textAlign = "center";
        ctx.fillText(`${operator.name} - ${skinData.name}`, canvas.width / 2, canvas.height / 2 - 20);
        ctx.fillText(`Animation: ${selectedAnimation}`, canvas.width / 2, canvas.height / 2 + 20);
        ctx.fillText("Spine WebGL renderer would display animation here", canvas.width / 2, canvas.height / 2 + 60);
        ctx.fillText("Install @esotericsoftware/spine-player", canvas.width / 2, canvas.height / 2 + 90);
    };

    const handleAnimationChange = (value: string) => {
        setSelectedAnimation(value as AnimationType);

        // If spine player exists, update the animation
        if (spinePlayerRef.current) {
            try {
                spinePlayerRef.current.setAnimation(value);
            } catch (err) {
                console.error("Error setting animation:", err);
            }
        }
    };

    const handlePlayPause = () => {
        if (!spinePlayerRef.current) return;

        try {
            if (isPlaying) {
                spinePlayerRef.current.pause();
            } else {
                spinePlayerRef.current.play();
            }
            setIsPlaying(!isPlaying);
        } catch (err) {
            console.error("Error playing/pausing animation:", err);
        }
    };

    const handleReset = () => {
        if (!spinePlayerRef.current) return;

        try {
            spinePlayerRef.current.setAnimation(selectedAnimation);
            if (!isPlaying) {
                spinePlayerRef.current.play();
                setIsPlaying(true);
            }
        } catch (err) {
            console.error("Error resetting animation:", err);
        }
    };

    const handleSpeedChange = (value: number[]) => {
        if (value.length === 0) return;

        // Use non-null assertion (!) as per linter suggestion
        const newSpeed = value[0]!;
        setSpeed(newSpeed);

        if (spinePlayerRef.current) {
            try {
                spinePlayerRef.current.timeScale = newSpeed;
            } catch (err) {
                console.error("Error setting animation speed:", err);
            }
        }
    };

    return (
        <Card className="h-full">
            <CardContent className="p-4">
                <div className="flex flex-col space-y-4">
                    {/* Animation selection */}
                    <div className="flex items-center justify-between">
                        <div className="font-medium">Animation:</div>
                        <Select value={selectedAnimation} onValueChange={handleAnimationChange} disabled={isLoading || !selectedOperator || availableAnimations.length === 0}>
                            <SelectTrigger className="w-[180px]">
                                <SelectValue placeholder="Select animation" />
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

                    {/* Canvas container */}
                    <div className="relative aspect-square w-full overflow-hidden rounded-md border">
                        {isLoading && (
                            <div className="absolute inset-0 z-10 flex items-center justify-center bg-background/80">
                                <div className="text-primary">Loading...</div>
                            </div>
                        )}

                        {error && (
                            <div className="absolute inset-0 z-10 flex items-center justify-center bg-background/80">
                                <div className="text-destructive">{error}</div>
                            </div>
                        )}

                        <div ref={canvasContainerRef} className="h-full w-full"></div>
                    </div>

                    {/* Animation controls */}
                    <div className="flex items-center justify-between">
                        <div className="flex gap-2">
                            <Button variant="outline" size="icon" onClick={handlePlayPause} disabled={!selectedOperator || isLoading}>
                                {isPlaying ? <PauseIcon className="h-4 w-4" /> : <PlayIcon className="h-4 w-4" />}
                            </Button>
                            <Button variant="outline" size="icon" onClick={handleReset} disabled={!selectedOperator || isLoading}>
                                <RepeatIcon className="h-4 w-4" />
                            </Button>
                        </div>

                        <div className="ml-4 flex max-w-[200px] flex-1 items-center gap-2">
                            <span className="text-xs">Speed:</span>
                            <Slider value={[speed]} min={0.1} max={2} step={0.1} onValueChange={handleSpeedChange} disabled={!selectedOperator || isLoading} />
                            <span className="w-8 text-xs">{speed.toFixed(1)}x</span>
                        </div>
                    </div>

                    {/* Information text */}
                    {selectedOperator && (
                        <div className="text-xs text-muted-foreground">
                            <p>Note: Ensure you have installed @esotericsoftware/spine-player for full functionality.</p>
                        </div>
                    )}
                </div>
            </CardContent>
        </Card>
    );
}
