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
    const [app, setApp] = useState<PIXI.Application | null>(null);
    const [currentAnimation, setCurrentAnimation] = useState<string>("Idle");
    const [availableAnimations, setAvailableAnimations] = useState<string[]>([]);
    const [spine, setSpine] = useState<Spine | null>(null);
    const [error, setError] = useState<string | null>(null);

    // Initialize PixiJS app
    useEffect(() => {
        if (!canvasRef.current) return;

        // Clean up previous app if it exists
        if (app) {
            app.destroy(true, true);
        }

        // Create new PIXI application
        const pixiApp = new PIXI.Application({
            width: 400,
            height: 400,
            backgroundColor: 0x000000,
            antialias: true,
            transparent: true,
        });
        

        canvasRef.current.innerHTML = "";
        canvasRef.current.appendChild(pixiApp.view);
        setApp(pixiApp);

        return () => {
            if (pixiApp) {
                pixiApp.destroy(true, true);
            }
        };
    }, [canvasRef]);

    // Load spine data when the app and URLs change
    useEffect(() => {
        if (!app || !atlasUrl || !skelUrl || !imageUrl) return;

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
                    if (spine?.parent) {
                        spine.parent.removeChild(spine);
                    }

                    // Check if resources loaded correctly
                    if (!resources.chibi_skel || !resources.chibi_atlas) {
                        setError("Failed to load spine resources");
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

                        // Set default animation if available
                        if (animations.includes("Idle")) {
                            spineData.state.setAnimation(0, "Idle", true);
                        } else if (animations.length > 0) {
                            spineData.state.setAnimation(0, animations[0] ?? "Idle", true);
                            setCurrentAnimation(animations[0]);
                        }

                        setSpine(spineData);
                        setError(null);
                    } else {
                        setError("Invalid spine data format");
                    }
                } catch (e) {
                    console.error("Error setting up spine animation:", e);
                    setError("Error setting up animation");
                }
            });

            // Handle loading errors
            loader.onError.add(() => {
                setError("Failed to load spine resources");
            });
        } catch (e) {
            console.error("Error loading spine data:", e);
            setError("Error loading spine data");
        }
    }, [app, atlasUrl, skelUrl, imageUrl]);

    // Change animation when selected animation changes
    useEffect(() => {
        if (spine && currentAnimation) {
            spine.state.setAnimation(0, currentAnimation, true);
        }
    }, [spine, currentAnimation]);

    // Play animation buttons
    const handleAnimationChange = (animation: string) => {
        setCurrentAnimation(animation);
    };

    return (
        <div className="flex flex-col items-center gap-4">
            <div ref={canvasRef} className="relative flex h-[400px] w-full items-center justify-center rounded-lg bg-black/5">
                {error && (
                    <div className="absolute inset-0 flex items-center justify-center rounded-lg bg-black/20">
                        <div className="rounded-lg bg-card p-4 text-center shadow-lg">
                            <p className="text-destructive">{error}</p>
                            <p className="mt-2 text-sm text-muted-foreground">Unable to load spine animation for {operatorName}</p>
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
