import { useEffect, useRef, useState } from "react";
import * as PIXI from "pixi.js";
import { Spine } from "pixi-spine";

interface PixiRendererProps {
    atlasUrl: string;
    skelUrl: string;
    imageUrl: string;
    operatorName: string;
}

// Define basic types for Spine animations to reduce 'any' usage
interface SpineAnimation {
    name: string;
}

export function PixiRenderer({ atlasUrl, skelUrl, imageUrl, operatorName }: PixiRendererProps) {
    const canvasRef = useRef<HTMLDivElement>(null);
    const appRef = useRef<PIXI.Application | null>(null); // Use ref instead of state
    const spineRef = useRef<Spine | null>(null); // Use ref instead of state for spine
    const [currentAnimation, setCurrentAnimation] = useState<string>("Idle");
    const [availableAnimations, setAvailableAnimations] = useState<string[]>([]);
    const [isLoading, setIsLoading] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);

    // Initialize PixiJS app
    useEffect(() => {
        if (!canvasRef.current) return;

        // Clean up previous app if it exists
        if (appRef.current) {
            appRef.current.destroy(true, true);
        }

        // Create new PIXI application with autoResize explicitly disabled
        const pixiApp = new PIXI.Application({
            width: 400,
            height: 400,
            backgroundColor: 0x000000,
            antialias: true,
            transparent: true,
            autoDensity: false, // Disable automatic resizing
            resolution: window.devicePixelRatio || 1,
        });

        canvasRef.current.innerHTML = "";
        canvasRef.current.appendChild(pixiApp.view);
        appRef.current = pixiApp; // Store in ref instead of state

        return () => {
            if (appRef.current) {
                appRef.current.destroy(true, true);
                appRef.current = null;
            }
            spineRef.current = null;
        };
    }, []);

    // Load spine data when the app and URLs change
    useEffect(() => {
        if (!appRef.current || !atlasUrl || !skelUrl || !imageUrl) return;

        setIsLoading(true);
        setError(null);

        const app = appRef.current;
        const loader = app.loader;

        // Clear any existing resources
        loader.reset();

        try {
            // Add the resources to load
            loader.add("chibi_atlas", atlasUrl);
            loader.add("chibi_skel", skelUrl);
            loader.add("chibi_image", imageUrl);

            // Load everything
            loader.load((_, resources) => {
                try {
                    // Remove previous spine if it exists
                    if (spineRef.current?.parent) {
                        spineRef.current.parent.removeChild(spineRef.current);
                        spineRef.current = null;
                    }

                    // Check if resources loaded correctly
                    if (!resources.chibi_skel || !resources.chibi_atlas) {
                        setError("Failed to load spine resources");
                        setIsLoading(false);
                        return;
                    }

                    // Create spine animation - handling the case where spineData might be undefined
                    if (resources.chibi_skel.spineData) {
                        const spineData = new Spine(resources.chibi_skel.spineData);

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
                            setCurrentAnimation("Idle");
                        } else if (animations.length > 0) {
                            const defaultAnim = animations[0] ?? "Idle";
                            spineData.state.setAnimation(0, defaultAnim, true);
                            setCurrentAnimation(defaultAnim);
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

            // Handle loading errors
            loader.onError.add(() => {
                setError("Failed to load spine resources");
                setIsLoading(false);
            });
        } catch (e) {
            console.error("Error loading spine data:", e);
            setError("Error loading spine data");
            setIsLoading(false);
        }
    }, [atlasUrl, skelUrl, imageUrl]);

    // Change animation when selected animation changes
    useEffect(() => {
        // Only try to change animation if spine exists and animation name is valid
        if (spineRef.current?.state && currentAnimation && availableAnimations.includes(currentAnimation)) {
            try {
                spineRef.current.state.setAnimation(0, currentAnimation, true);
            } catch (error) {
                console.error("Failed to set animation:", error);
            }
        }
    }, [currentAnimation, availableAnimations]);

    // Play animation buttons
    const handleAnimationChange = (animation: string) => {
        setCurrentAnimation(animation);
    };

    return (
        <div className="flex flex-col items-center gap-4">
            <div ref={canvasRef} className="relative flex h-[400px] w-full items-center justify-center rounded-lg bg-black/5">
                {(error ?? isLoading) && (
                    <div className="absolute inset-0 flex items-center justify-center rounded-lg bg-black/20">
                        <div className="rounded-lg bg-card p-4 text-center shadow-lg">
                            {error && (
                                <>
                                    <p className="text-destructive">{error}</p>
                                    <p className="mt-2 text-sm text-muted-foreground">Unable to load spine animation for {operatorName}</p>
                                </>
                            )}
                            {isLoading && !error && <p>Loading animation...</p>}
                        </div>
                    </div>
                )}
            </div>

            {availableAnimations.length > 0 && (
                <div className="flex flex-wrap justify-center gap-2">
                    {availableAnimations.map((animation) => (
                        <button key={animation} onClick={() => handleAnimationChange(animation)} className={`rounded-full px-3 py-1 text-sm ${currentAnimation === animation ? "bg-primary text-primary-foreground" : "bg-secondary text-secondary-foreground"}`}>
                            {animation}
                        </button>
                    ))}
                </div>
            )}
        </div>
    );
}
