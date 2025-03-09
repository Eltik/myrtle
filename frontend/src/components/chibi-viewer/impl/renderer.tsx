import { useCallback, useEffect, useRef, useState } from "react";
import type { FormattedChibis } from "~/types/impl/frontend/impl/chibis";
import { Card, CardContent } from "~/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { Slider } from "~/components/ui/slider";
import { Button } from "~/components/ui/button";
import { PauseIcon, PlayIcon, RepeatIcon } from "lucide-react";
import { SpinePlayer } from "@esotericsoftware/spine-player";

type AnimationType = "idle" | "skill" | "move" | "die" | "attack" | "special";

type ChibiRendererProps = {
    selectedOperator: FormattedChibis | null;
    selectedSkin: string | null;
    repoBaseUrl: string;
};

export function ChibiRenderer({ selectedOperator, selectedSkin, repoBaseUrl }: ChibiRendererProps) {
    const canvasContainerRef = useRef<HTMLDivElement>(null);
    const staticImageContainerRef = useRef<HTMLDivElement>(null);
    const spinePlayerRef = useRef<SpinePlayer | null>(null);
    const [selectedAnimation, setSelectedAnimation] = useState<AnimationType>("idle");
    const [availableAnimations, setAvailableAnimations] = useState<AnimationType[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isPlaying, setIsPlaying] = useState(true);
    const [speed, setSpeed] = useState(1);
    const [useStaticFallback, setUseStaticFallback] = useState(false);

    // Helper function to find a matching animation name
    const findMatchingAnimation = useCallback((animations: string[], patterns: string[]): string | null => {
        for (const pattern of patterns) {
            const match = animations.find(anim => 
                anim.toLowerCase().includes(pattern.toLowerCase())
            );
            if (match) return match;
        }
        return null;
    }, []);

    // Function to get asset URL from the path
    const getAssetUrl = useCallback((path: string) => {
        // Remove the initial "./" if present
        const normalizedPath = path.startsWith("./") ? path.substring(2) : path;
        return `${repoBaseUrl}${normalizedPath}`;
    }, [repoBaseUrl]);

    // Function to show static image
    const showStaticImage = useCallback((operator: FormattedChibis, skinData: FormattedChibis["skins"][0]) => {
        if (!staticImageContainerRef.current) return;
        
        // Clear existing content
        staticImageContainerRef.current.innerHTML = "";
        
        if (skinData.dorm.png) {
            try {
                // Create an image element
                const img = document.createElement("img");
                img.src = getAssetUrl(skinData.dorm.png);
                img.alt = `${operator.name} - ${skinData.name}`;
                img.className = "w-full h-full object-contain";
                
                // Handle loading errors
                img.onerror = () => {
                    createFallbackCanvas(operator, skinData);
                };
                
                // Append the image to the container
                staticImageContainerRef.current.appendChild(img);
                
                // Show this container
                staticImageContainerRef.current.style.display = "block";
                
                // If we have a canvas container, hide it
                if (canvasContainerRef.current) {
                    canvasContainerRef.current.style.display = "none";
                }
            } catch (err) {
                console.error("Error showing static image:", err);
                createFallbackCanvas(operator, skinData);
            }
        } else {
            createFallbackCanvas(operator, skinData);
        }
    }, [getAssetUrl]);

    // Create a fallback canvas with text when no image can be shown
    const createFallbackCanvas = useCallback((operator: FormattedChibis, skinData: FormattedChibis["skins"][0]) => {
        if (!staticImageContainerRef.current) return;
        
        // Create a canvas for placeholder
        const canvas = document.createElement("canvas");
        canvas.width = 400;
        canvas.height = 400;
        canvas.className = "w-full h-full";
        staticImageContainerRef.current.appendChild(canvas);

        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        // Draw placeholder text
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.font = "16px Arial";
        ctx.fillStyle = "#888";
        ctx.textAlign = "center";
        ctx.fillText(`${operator.name} - ${skinData.name}`, canvas.width / 2, canvas.height / 2 - 20);
        ctx.fillText(`Animation: ${selectedAnimation}`, canvas.width / 2, canvas.height / 2 + 20);
        ctx.fillText("Spine data not available", canvas.width / 2, canvas.height / 2 + 60);
        
        // Show this container
        staticImageContainerRef.current.style.display = "block";
        
        // If we have a canvas container, hide it
        if (canvasContainerRef.current) {
            canvasContainerRef.current.style.display = "none";
        }
    }, [selectedAnimation]);

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
        setUseStaticFallback(false);

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

        // Clear containers
        if (canvasContainerRef.current) {
            canvasContainerRef.current.innerHTML = "";
            canvasContainerRef.current.style.display = "block";
        }
        
        if (staticImageContainerRef.current) {
            staticImageContainerRef.current.innerHTML = "";
            staticImageContainerRef.current.style.display = "none";
        }

        const loadSpinePlayer = () => {
            try {
                // Determine available animations based on skeleton data
                const defaultAnimations: AnimationType[] = ["idle", "skill", "move", "die", "attack", "special"];
                setAvailableAnimations(defaultAnimations);

                // Check if we have atlas, png, and skel files
                if (skinData.dorm.atlas && skinData.dorm.png && skinData.dorm.skel) {
                    // Create spine player
                    const atlasUrl = getAssetUrl(skinData.dorm.atlas);
                    const skelUrl = getAssetUrl(skinData.dorm.skel);
                    const pngUrl = getAssetUrl(skinData.dorm.png);
                    
                    console.log("Loading spine data:", {
                        format: 'Binary',
                        atlas: atlasUrl,
                        skel: skelUrl,
                        png: pngUrl
                    });

                    // Check if we're dealing with a binary .skel file
                    // These are known to be incompatible with our Spine player version
                    if (skelUrl.toLowerCase().endsWith('.skel')) {
                        console.log("Detected binary .skel file, skipping Spine player and using static image");
                        // Skip Spine player creation entirely and go straight to static image
                        setUseStaticFallback(true);
                        setIsLoading(false);
                        return;
                    }

                    // If we're here, we're dealing with a non-binary file (like .json)
                    // First fetch the PNG to make sure it exists
                    fetch(pngUrl)
                        .then(response => {
                            if (!response.ok) {
                                throw new Error(`Failed to load PNG file: ${response.status} ${response.statusText}`);
                            }
                            
                            // Pre-load static image
                            const img = new Image();
                            img.src = pngUrl;
                            
                            return fetch(skelUrl);
                        })
                        .then(response => {
                            if (!response.ok) {
                                throw new Error(`Failed to load skeleton file: ${response.status} ${response.statusText}`);
                            }
                            return fetch(atlasUrl);
                        })
                        .then(response => {
                            if (!response.ok) {
                                throw new Error(`Failed to load atlas file: ${response.status} ${response.statusText}`);
                            }
                            
                            // We need to wait for the next tick to ensure the container is ready
                            setTimeout(() => {
                                if (!canvasContainerRef.current) return;

                                try {
                                    // Configure Spine Player specifically for binary format
                                    const playerConfig = {
                                        // For binary .skel files, we must use skelUrl, not jsonUrl
                                        skelUrl,
                                        atlasUrl,
                                        alpha: true,
                                        backgroundColor: "#00000000",
                                        premultipliedAlpha: true,
                                        showControls: false,
                                        animation: selectedAnimation,
                                        preserveDrawingBuffer: false,
                                        viewport: {
                                            debugRender: false,
                                            x: 0,
                                            y: 0,
                                            width: canvasContainerRef.current.clientWidth,
                                            height: canvasContainerRef.current.clientHeight,
                                            padLeft: 0,
                                            padRight: 0,
                                            padTop: 0,
                                            padBottom: 0
                                        },
                                        success: (spinePlayer: SpinePlayer) => {
                                            spinePlayerRef.current = spinePlayer;
                                            
                                            // Get available animations from skeleton
                                            if (spinePlayer.skeleton) {
                                                const skeletonData = spinePlayer.skeleton.data;
                                                const animations = skeletonData.animations.map((a: { name: string }) => a.name);
                                                console.log("Available animations:", animations);
                                                
                                                // Better animation matching with common spine animation naming patterns
                                                // Map our animation types to potential matching patterns in the skeleton
                                                const animationPatterns: Record<AnimationType, string[]> = {
                                                    "idle": ["idle", "stand", "wait", "default", "loop"],
                                                    "skill": ["skill", "ability", "cast", "spell"],
                                                    "move": ["move", "walk", "run", "dash"],
                                                    "die": ["die", "death", "defeat", "down"],
                                                    "attack": ["attack", "hit", "strike", "combat"],
                                                    "special": ["special", "ultimate", "unique", "victory"]
                                                };
                                                
                                                // Find available animations by checking for each pattern
                                                const availableAnims: AnimationType[] = [];
                                                
                                                for (const animType of Object.keys(animationPatterns) as AnimationType[]) {
                                                    const patterns = animationPatterns[animType];
                                                    // Check if any animation in the skeleton matches our patterns
                                                    const hasAnimation = animations.some((animName: string) => 
                                                        patterns.some(pattern => animName.toLowerCase().includes(pattern))
                                                    );
                                                    
                                                    if (hasAnimation) {
                                                        availableAnims.push(animType);
                                                    }
                                                }
                                                
                                                // If no matches found with patterns, fallback to our standard detection
                                                if (availableAnims.length === 0) {
                                                    const defaultAnims = ["idle", "skill", "move", "die", "attack", "special"] as AnimationType[];
                                                    const fallbackAnims = defaultAnims.filter(anim => 
                                                        animations.some((a: string) => a.toLowerCase().includes(anim))
                                                    );
                                                    
                                                    if (fallbackAnims.length > 0) {
                                                        availableAnims.push(...fallbackAnims);
                                                    } else if (animations.length > 0) {
                                                        // If still no matches but animations exist, use the first animation
                                                        setSelectedAnimation(animations[0] as AnimationType);
                                                        availableAnims.push("idle"); // Use idle as a placeholder
                                                    }
                                                }
                                                
                                                if (availableAnims.length > 0) {
                                                    setAvailableAnimations(availableAnims);
                                                    
                                                    // Check if selected animation is in available animations, otherwise use first available
                                                    const animToUse = availableAnims.includes(selectedAnimation) ? 
                                                        selectedAnimation : (availableAnims[0] ?? "idle");
                                                    
                                                    // Find the actual animation name in the skeleton that matches our type
                                                    const patterns = animationPatterns[animToUse] || [];
                                                    const actualAnimationName = findMatchingAnimation(animations, patterns);
                                                    
                                                    if (actualAnimationName) {
                                                        // Set the animation using the actual name from the skeleton
                                                        spinePlayer.setAnimation(actualAnimationName);
                                                        setSelectedAnimation(animToUse);
                                                    } else {
                                                        // Fallback to our generic name
                                                        spinePlayer.setAnimation(animToUse);
                                                    }
                                                    
                                                    // Set playback speed
                                                    try {
                                                        // Access timeScale through the animation state
                                                        if (spinePlayer.animationState) {
                                                            spinePlayer.animationState.timeScale = speed;
                                                        }
                                                    } catch (err) {
                                                        console.error("Error setting animation speed:", err);
                                                    }
                                                    
                                                    // Set play state
                                                    if (isPlaying) {
                                                        spinePlayer.play();
                                                    } else {
                                                        spinePlayer.pause();
                                                    }
                                                } else {
                                                    setError("No compatible animations found");
                                                    setUseStaticFallback(true);
                                                }
                                            }
                                            
                                            setIsLoading(false);
                                        },
                                        error: (errorEvent: unknown) => {
                                            console.error("Error loading spine data:", errorEvent);
                                            
                                            // Get error message safely
                                            let errorMessage = "Unknown error";
                                            
                                            // Check for specific error conditions
                                            if (errorEvent !== null && errorEvent !== undefined && typeof errorEvent === 'object') {
                                                // We now know it's an object
                                                const errorObj = errorEvent as Record<string, unknown>;
                                                
                                                if ('config' in errorObj && 'skeleton' in errorObj) {
                                                    console.log("Detected SpinePlayer instance as error. This usually indicates a compatibility issue.");
                                                    errorMessage = "Spine file format error: The file format may be incompatible with this player version.";
                                                } else if ('message' in errorObj && typeof errorObj.message === 'string') {
                                                    errorMessage = errorObj.message;
                                                    
                                                    // Check for specific error patterns
                                                    if (errorMessage.includes("String in string table must not be null")) {
                                                        errorMessage = "Spine file format error: The binary file uses a Spine version that's incompatible with this player.";
                                                    } else if (errorMessage.includes("Couldn't load atlas")) {
                                                        errorMessage = "Failed to load the atlas file.";
                                                    } else if (errorMessage.includes("Couldn't load skeleton")) {
                                                        errorMessage = "Failed to load the skeleton file.";
                                                    }
                                                }
                                            } else if (typeof errorEvent === 'string') {
                                                errorMessage = errorEvent;
                                            }
                                            
                                            setError(errorMessage);
                                            setUseStaticFallback(true);
                                            setIsLoading(false);
                                        }
                                    };
                                    
                                    // Create Spine player with the correct configuration
                                    const player = new SpinePlayer(canvasContainerRef.current, playerConfig);
                                } catch (err) {
                                    console.error("Error creating Spine player:", err);
                                    setUseStaticFallback(true);
                                    setIsLoading(false);
                                }
                            }, 100);
                        })
                        .catch(err => {
                            console.error("Error fetching spine files:", err);
                            
                            // Get error message safely
                            let errorMessage = "Failed to load spine files";
                            
                            // Use a more specific type check
                            if (err !== null && err !== undefined && typeof err === 'object') {
                                // We now know it's an object
                                const errorObj = err as Record<string, unknown>;
                                
                                if ('message' in errorObj && typeof errorObj.message === 'string') {
                                    errorMessage = errorObj.message;
                                }
                            } else if (typeof err === 'string') {
                                // Handle case where the error might be a string directly
                                errorMessage = err;
                            }
                            
                            setError(errorMessage);
                            setUseStaticFallback(true);
                            setIsLoading(false);
                        });
                } else {
                    setError("Missing spine data files (atlas, png, or skel)");
                    setUseStaticFallback(true);
                    setIsLoading(false);
                }
            } catch (err) {
                console.error("Error loading spine data:", err);
                
                // Get error message safely
                let errorMessage = "Unknown error loading spine data";
                
                // Use a more specific type check
                if (err !== null && err !== undefined && typeof err === 'object') {
                    // We now know it's an object
                    const errorObj = err as Record<string, unknown>;
                    
                    if ('message' in errorObj && typeof errorObj.message === 'string') {
                        errorMessage = errorObj.message;
                    }
                } else if (typeof err === 'string') {
                    // Handle case where the error might be a string directly
                    errorMessage = err;
                }
                
                setError(errorMessage);
                setUseStaticFallback(true);
                setIsLoading(false);
            }
        };

        // Start loading Spine player
        loadSpinePlayer();
    }, [selectedOperator, selectedSkin, selectedAnimation, repoBaseUrl, speed, getAssetUrl, isPlaying, showStaticImage, createFallbackCanvas, findMatchingAnimation]);

    // Effect to handle static fallback display
    useEffect(() => {
        if (useStaticFallback && selectedOperator && selectedSkin) {
            const skinData = selectedOperator.skins.find((skin) => skin.dorm.path === selectedSkin);
            if (skinData) {
                showStaticImage(selectedOperator, skinData);
            }
        }
    }, [useStaticFallback, selectedOperator, selectedSkin, showStaticImage]);

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
                // Set speed using the animation state
                if (spinePlayerRef.current.animationState) {
                    spinePlayerRef.current.animationState.timeScale = newSpeed;
                }
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

                    {/* Canvas container and static image container */}
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

                        {/* Spine Player container */}
                        <div ref={canvasContainerRef} className="h-full w-full"></div>
                        
                        {/* Static image container */}
                        <div ref={staticImageContainerRef} className="h-full w-full" style={{ display: 'none' }}></div>
                    </div>

                    {/* Animation controls */}
                    <div className="flex items-center justify-between">
                        <div className="flex gap-2">
                            <Button variant="outline" size="icon" onClick={handlePlayPause} disabled={!selectedOperator || isLoading || useStaticFallback}>
                                {isPlaying ? <PauseIcon className="h-4 w-4" /> : <PlayIcon className="h-4 w-4" />}
                            </Button>
                            <Button variant="outline" size="icon" onClick={handleReset} disabled={!selectedOperator || isLoading || useStaticFallback}>
                                <RepeatIcon className="h-4 w-4" />
                            </Button>
                        </div>

                        <div className="ml-4 flex max-w-[200px] flex-1 items-center gap-2">
                            <span className="text-xs">Speed:</span>
                            <Slider value={[speed]} min={0.1} max={2} step={0.1} onValueChange={handleSpeedChange} disabled={!selectedOperator || isLoading || useStaticFallback} />
                            <span className="w-8 text-xs">{speed.toFixed(1)}x</span>
                        </div>
                    </div>

                    {/* Information text */}
                    {selectedOperator && useStaticFallback && (
                        <div className="text-xs text-muted-foreground">
                            <p>Note: Static image is shown because the spine animation could not be loaded.</p>
                        </div>
                    )}
                </div>
            </CardContent>
        </Card>
    );
}
