import { useCallback, useEffect, useRef, useState } from "react";
import type { FormattedChibis } from "~/types/impl/frontend/impl/chibis";
import { Card, CardContent } from "~/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { Slider } from "~/components/ui/slider";
import { Button } from "~/components/ui/button";
import { PauseIcon, PlayIcon, RepeatIcon } from "lucide-react";
import { Assets, Texture, Sprite } from 'pixi.js';

type AnimationType = "idle" | "skill" | "move" | "die" | "attack" | "special";

type ChibiRendererProps = {
    selectedOperator: FormattedChibis | null;
    selectedSkin: string | null;
    repoBaseUrl: string;
};

export function ChibiRenderer({ selectedOperator, selectedSkin, repoBaseUrl }: ChibiRendererProps) {
    const [selectedAnimation, setSelectedAnimation] = useState<AnimationType>("idle");
    const [availableAnimations, setAvailableAnimations] = useState<AnimationType[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isPlaying, setIsPlaying] = useState(true);
    const [speed, setSpeed] = useState(1);
    const [useStaticFallback, setUseStaticFallback] = useState(false);
    const staticImageContainerRef = useRef<HTMLDivElement>(null);
    const canvasContainerRef = useRef<HTMLDivElement>(null);
    
    // Simple animation refs - no PIXI Application
    const spriteRef = useRef<Sprite | null>(null);
    const requestRef = useRef<number | null>(null);
    const previousTimeRef = useRef<number>(0);
    const animationActiveRef = useRef<boolean>(true);
    
    // Function to get asset URL from the path
    const getAssetUrl = useCallback((path: string) => {
        // Remove the initial "./" if present
        const normalizedPath = path.startsWith("./") ? path.substring(2) : path;
        return `${repoBaseUrl}${normalizedPath}`;
    }, [repoBaseUrl]);
    
    // Get skin data
    const getSkinData = useCallback(() => {
        if (!selectedOperator || !selectedSkin) {
            return null;
        }
        
        // Find the selected skin
        const skin = selectedOperator.skins.find(s => s.dorm.path === selectedSkin || 
                                                     s.front.path === selectedSkin || 
                                                     s.back.path === selectedSkin);
        
        if (!skin) {
            return null;
        }
        
        // Determine which view to use (dorm, front, or back)
        if (selectedSkin === skin.dorm.path && skin.dorm.atlas && skin.dorm.png && skin.dorm.skel) {
            return {
                atlas: getAssetUrl(skin.dorm.atlas),
                png: getAssetUrl(skin.dorm.png),
                skel: getAssetUrl(skin.dorm.skel),
                type: 'dorm'
            };
        } else if (selectedSkin === skin.front.path && skin.front.atlas && skin.front.png && skin.front.skel) {
            return {
                atlas: getAssetUrl(skin.front.atlas),
                png: getAssetUrl(skin.front.png),
                skel: getAssetUrl(skin.front.skel),
                type: 'front'
            };
        } else if (selectedSkin === skin.back.path && skin.back.atlas && skin.back.png && skin.back.skel) {
            return {
                atlas: getAssetUrl(skin.back.atlas),
                png: getAssetUrl(skin.back.png),
                skel: getAssetUrl(skin.back.skel),
                type: 'back'
            };
        }
        
        return null;
    }, [selectedOperator, selectedSkin, getAssetUrl]);
    
    // Cleanup animation
    const cleanupAnimation = useCallback(() => {
        // Cancel animation frame
        if (requestRef.current !== null) {
            cancelAnimationFrame(requestRef.current);
            requestRef.current = null;
        }
        
        // Clean up canvas container
        if (canvasContainerRef.current) {
            canvasContainerRef.current.innerHTML = '';
        }
        
        // Reset animation state
        animationActiveRef.current = false;
    }, []);
    
    // Setup animation when operator or skin changes
    useEffect(() => {
        console.log("Operator or skin changed, resetting state");
        setError(null);
        setUseStaticFallback(false);
        setIsLoading(true);
        
        // Set default animations
        if (!selectedOperator) {
            setAvailableAnimations([]);
            setIsLoading(false);
            cleanupAnimation();
            return;
        }
        
        // Default animations
        const defaultAnimations: AnimationType[] = ["idle", "skill", "move", "die", "attack", "special"];
        setAvailableAnimations(defaultAnimations);
        
        // Get skin data
        const skinData = getSkinData();
        if (!skinData) {
            console.log("No skin data found, using fallback");
            setIsLoading(false);
            setUseStaticFallback(true);
            cleanupAnimation();
            return;
        }
        
        // Pre-check if texture is loadable
        console.log("Pre-checking if texture can be loaded:", skinData.png);
        
        // Try to load image directly first
        const img = new Image();
        img.onload = () => {
            console.log("Image preload successful");
            // Create an img element to show
            const imageElement = document.createElement('img');
            imageElement.src = skinData.png;
            imageElement.style.position = 'absolute';
            imageElement.style.left = '50%';
            imageElement.style.top = '50%';
            imageElement.style.transform = 'translate(-50%, -50%)';
            imageElement.style.maxHeight = '100%';
            imageElement.style.maxWidth = '100%';
            
            // Clear container and add the image
            if (canvasContainerRef.current) {
                canvasContainerRef.current.innerHTML = '';
                canvasContainerRef.current.appendChild(imageElement);
                
                // Setup animation
                animationActiveRef.current = true;
                
                // Animate the image based on the selected animation
                const animate = (time: number) => {
                    if (!animationActiveRef.current || !imageElement || !isPlaying) {
                        // If animation is no longer active or element is gone, stop
                        return;
                    }
                    
                    const deltaTime = time - (previousTimeRef.current || time);
                    previousTimeRef.current = time;
                    
                    // Apply different animations based on the selected type
                    switch (selectedAnimation) {
                        case "idle":
                            // Gentle bobbing up and down
                            imageElement.style.top = `calc(50% + ${Math.sin(time / 500) * 5}px)`;
                            break;
                        case "skill":
                            // Rotation
                            const currentRotation = parseFloat(imageElement.dataset.rotation || '0');
                            const newRotation = currentRotation + (0.01 * speed);
                            imageElement.style.transform = `translate(-50%, -50%) rotate(${newRotation}rad)`;
                            imageElement.dataset.rotation = newRotation.toString();
                            break;
                        case "move":
                            // Left-right movement
                            imageElement.style.left = `calc(50% + ${Math.sin(time / 300) * 20}px)`;
                            break;
                        case "attack":
                            // Quick forward motion and scaling
                            imageElement.style.left = `calc(50% + ${Math.sin(time / 200) * 10}px)`;
                            const scale = 1 + Math.sin(time / 200) * 0.1;
                            imageElement.style.transform = `translate(-50%, -50%) scale(${scale})`;
                            break;
                        case "die":
                            // Fall and fade
                            const opacity = parseFloat(imageElement.style.opacity || '1');
                            if (opacity > 0.5) {
                                imageElement.style.opacity = `${1 - (Math.sin(time / 1000) * 0.5)}`;
                                imageElement.style.top = `calc(50% + ${Math.sin(time / 1000) * 20}px)`;
                            }
                            break;
                        case "special":
                            // Spin and scale
                            const specialRotation = parseFloat(imageElement.dataset.rotation || '0');
                            const newSpecialRotation = specialRotation + (0.005 * speed);
                            const specialScale = 1 + Math.sin(time / 400) * 0.2;
                            imageElement.style.transform = `translate(-50%, -50%) rotate(${newSpecialRotation}rad) scale(${specialScale})`;
                            imageElement.dataset.rotation = newSpecialRotation.toString();
                            break;
                        default:
                            break;
                    }
                    
                    // Continue the animation loop
                    requestRef.current = requestAnimationFrame(animate);
                };
                
                // Start the animation
                requestRef.current = requestAnimationFrame(animate);
                
                // Loading complete
                setIsLoading(false);
                console.log("Animation setup complete");
            }
        };
        img.onerror = () => {
            console.error("Image preload failed, falling back to static image");
            setError("Failed to load character texture");
            setIsLoading(false);
            setUseStaticFallback(true);
            cleanupAnimation();
        };
        img.src = skinData.png;
        
        return () => {
            cleanupAnimation();
        };
    }, [selectedOperator, selectedSkin, getSkinData, cleanupAnimation, selectedAnimation, isPlaying, speed]);
    
    // Handle animation change
    const handleAnimationChange = (value: string) => {
        setSelectedAnimation(value as AnimationType);
        
        // Reset the image position and transform
        if (canvasContainerRef.current?.firstChild) {
            const imageElement = canvasContainerRef.current.firstChild as HTMLImageElement;
            imageElement.style.left = '50%';
            imageElement.style.top = '50%';
            imageElement.style.transform = 'translate(-50%, -50%)';
            imageElement.style.opacity = '1';
            imageElement.dataset.rotation = '0';
        }
    };
    
    // Handle play/pause
    const handlePlayPause = () => {
        setIsPlaying(!isPlaying);
        
        // If we're resuming and no animation frame is active, restart it
        if (!isPlaying && !requestRef.current) {
            const animate = (time: number) => {
                if (!animationActiveRef.current || !canvasContainerRef.current?.firstChild) {
                    return;
                }
                
                const imageElement = canvasContainerRef.current.firstChild as HTMLImageElement;
                const deltaTime = time - (previousTimeRef.current || time);
                previousTimeRef.current = time;
                
                // Apply different animations based on the selected type
                switch (selectedAnimation) {
                    case "idle":
                        imageElement.style.top = `calc(50% + ${Math.sin(time / 500) * 5}px)`;
                        break;
                    case "skill":
                        const currentRotation = parseFloat(imageElement.dataset.rotation || '0');
                        const newRotation = currentRotation + (0.01 * speed);
                        imageElement.style.transform = `translate(-50%, -50%) rotate(${newRotation}rad)`;
                        imageElement.dataset.rotation = newRotation.toString();
                        break;
                    case "move":
                        imageElement.style.left = `calc(50% + ${Math.sin(time / 300) * 20}px)`;
                        break;
                    case "attack":
                        imageElement.style.left = `calc(50% + ${Math.sin(time / 200) * 10}px)`;
                        const scale = 1 + Math.sin(time / 200) * 0.1;
                        imageElement.style.transform = `translate(-50%, -50%) scale(${scale})`;
                        break;
                    case "die":
                        const opacity = parseFloat(imageElement.style.opacity || '1');
                        if (opacity > 0.5) {
                            imageElement.style.opacity = `${1 - (Math.sin(time / 1000) * 0.5)}`;
                            imageElement.style.top = `calc(50% + ${Math.sin(time / 1000) * 20}px)`;
                        }
                        break;
                    case "special":
                        const specialRotation = parseFloat(imageElement.dataset.rotation || '0');
                        const newSpecialRotation = specialRotation + (0.005 * speed);
                        const specialScale = 1 + Math.sin(time / 400) * 0.2;
                        imageElement.style.transform = `translate(-50%, -50%) rotate(${newSpecialRotation}rad) scale(${specialScale})`;
                        imageElement.dataset.rotation = newSpecialRotation.toString();
                        break;
                    default:
                        break;
                }
                
                requestRef.current = requestAnimationFrame(animate);
            };
            
            requestRef.current = requestAnimationFrame(animate);
        }
    };
    
    // Handle reset
    const handleReset = () => {
        // Reset the image position and transform
        if (canvasContainerRef.current?.firstChild) {
            const imageElement = canvasContainerRef.current.firstChild as HTMLImageElement;
            imageElement.style.left = '50%';
            imageElement.style.top = '50%';
            imageElement.style.transform = 'translate(-50%, -50%)';
            imageElement.style.opacity = '1';
            imageElement.dataset.rotation = '0';
        }
        
        // Ensure it's playing
        if (!isPlaying) {
            setIsPlaying(true);
        }
    };
    
    // Handle speed change
    const handleSpeedChange = (value: number[]) => {
        if (value.length > 0 && typeof value[0] === 'number') {
            setSpeed(value[0]);
        }
    };
    
    // Function to show static image
    const renderStaticImage = useCallback(() => {
        if (!selectedOperator || !selectedSkin) {
            return (
                <div className="flex items-center justify-center h-full">
                    <p className="text-muted-foreground">Select an operator and skin to view animations</p>
                </div>
            );
        }
        
        const skinData = getSkinData();
        if (!skinData) {
            return (
                <div className="flex items-center justify-center h-full">
                    <p className="text-muted-foreground">No skin data available</p>
                </div>
            );
        }
        
        return (
            <div className="flex flex-col items-center justify-center h-full">
                <img 
                    src={skinData.png}
                    alt={`${selectedOperator.name} - static image`}
                    className="max-h-full max-w-full object-contain"
                    onError={(e) => {
                        e.currentTarget.style.display = 'none';
                        if (staticImageContainerRef.current) {
                            const fallbackText = document.createElement('p');
                            fallbackText.className = 'text-muted-foreground';
                            fallbackText.innerText = 'Character image not available';
                            staticImageContainerRef.current.appendChild(fallbackText);
                        }
                    }}
                />
                <p className="mt-2 text-sm text-muted-foreground">
                    Static image shown - animation type: {selectedAnimation}
                </p>
            </div>
        );
    }, [selectedOperator, selectedSkin, selectedAnimation, getSkinData]);
    
    // Get current skin data for UI display
    const spineData = getSkinData();
    const hasAnimation = Boolean(canvasContainerRef.current?.firstChild && animationActiveRef.current);
    
    return (
        <Card className="w-full">
            <CardContent className="pt-6 pb-4">
                {/* Animation controls */}
                <div className="flex flex-col gap-4 mb-4">
                    <div className="flex items-center gap-2">
                        <Select 
                            value={selectedAnimation} 
                            onValueChange={handleAnimationChange}
                            disabled={!spineData || availableAnimations.length === 0 || isLoading}
                        >
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
                        
                        <Button 
                            variant="outline" 
                            size="icon" 
                            onClick={handlePlayPause}
                            disabled={!hasAnimation || isLoading}
                        >
                            {isPlaying ? <PauseIcon className="h-4 w-4" /> : <PlayIcon className="h-4 w-4" />}
                        </Button>
                        
                        <Button 
                            variant="outline" 
                            size="icon" 
                            onClick={handleReset}
                            disabled={!hasAnimation || isLoading}
                        >
                            <RepeatIcon className="h-4 w-4" />
                        </Button>
                    </div>
                    
                    <div className="flex items-center gap-2">
                        <span className="text-sm w-10">Speed:</span>
                        <Slider
                            defaultValue={[1]}
                            min={0.1}
                            max={2}
                            step={0.1}
                            value={[speed]}
                            onValueChange={handleSpeedChange}
                            disabled={!hasAnimation || isLoading}
                            className="w-full max-w-56"
                        />
                        <span className="text-sm w-8">{speed}x</span>
                    </div>
                </div>
                
                {/* Display area */}
                <div 
                    className="w-full h-64 bg-background rounded-md border relative"
                >
                    {/* Loading state */}
                    {isLoading && (
                        <div className="absolute inset-0 flex items-center justify-center z-10">
                            <div className="w-6 h-6 border-t-2 border-primary rounded-full animate-spin" />
                        </div>
                    )}
                    
                    {/* Error message */}
                    {error && (
                        <div className="absolute inset-0 flex items-center justify-center text-center p-4 z-10">
                            <p className="text-destructive">{error}</p>
                        </div>
                    )}
                    
                    {/* Animation container */}
                    <div 
                        ref={canvasContainerRef}
                        className="w-full h-full"
                        style={{ display: (!isLoading && !error && !useStaticFallback) ? 'block' : 'none' }}
                    ></div>
                    
                    {/* Static image fallback */}
                    <div 
                        ref={staticImageContainerRef}
                        className="w-full h-full"
                        style={{ display: (useStaticFallback || !spineData) && !isLoading && !error ? 'block' : 'none' }}
                    >
                        {renderStaticImage()}
                    </div>
                </div>
                
                {hasAnimation && !useStaticFallback && !isLoading && !error && (
                    <div className="mt-2">
                        <p className="text-xs text-muted-foreground">
                            Note: This is a simplified animation using CSS transforms. For full Spine animations:
                        </p>
                        <ul className="text-xs text-muted-foreground list-disc pl-5 mt-1">
                            <li>Add pixi-spine integration to load .skel and .atlas files</li>
                            <li>Use Spine-specific animation controls for rich animations</li>
                            <li>Handle bone animations and skeletal deformations</li>
                        </ul>
                    </div>
                )}
                
                {useStaticFallback && !isLoading && !error && (
                    <div className="mt-2">
                        <p className="text-xs text-muted-foreground">
                            Note: Static image is shown as a fallback.
                        </p>
                    </div>
                )}
            </CardContent>
        </Card>
    );
}
