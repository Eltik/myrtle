"use client";

import * as PIXI from "pixi.js";
import { memo, useEffect, useMemo, useRef, useState } from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import type { ChibiCharacter, SpineFiles } from "~/types/api/impl/chibi";

const CHIBI_OFFSET_X = 0.5;
const CHIBI_OFFSET_Y = 0.85;
const CHIBI_SCALE = 0.5;
const ANIMATION_SPEED = 0.5;

interface ChibiViewerProps {
    chibi: ChibiCharacter;
    skinName: string;
}

type ViewType = "front" | "back" | "dorm";

function encodeAssetPath(path: string): string {
    if (!path) return "";
    return path
        .split("/")
        .map((segment) => {
            if (segment.includes("%")) return segment;
            return encodeURIComponent(segment);
        })
        .join("/");
}

// Custom texture loader that properly encodes URLs with # characters
async function loadSpineWithEncodedUrls(skelUrl: string, atlasUrl: string, pngUrl: string): Promise<import("pixi-spine").Spine> {
    // Import required classes - Arknights uses Spine 3.8 format
    const { Spine, TextureAtlas } = await import("pixi-spine");
    const { AtlasAttachmentLoader, SkeletonBinary } = await import("@pixi-spine/runtime-3.8");

    // Load all files in parallel
    const [skelResponse, atlasResponse] = await Promise.all([fetch(skelUrl), fetch(atlasUrl)]);

    if (!skelResponse.ok) throw new Error(`Failed to load skeleton: ${skelResponse.status}`);
    if (!atlasResponse.ok) throw new Error(`Failed to load atlas: ${atlasResponse.status}`);

    const [skelData, atlasText] = await Promise.all([skelResponse.arrayBuffer(), atlasResponse.text()]);

    // Load the texture
    const texture = await PIXI.Assets.load(pngUrl);

    // Create a simple texture callback for the atlas
    const textureCallback = (_path: string, callback: (tex: PIXI.BaseTexture) => void) => {
        callback(texture.baseTexture);
    };

    // Parse the atlas with our texture
    const atlas = new TextureAtlas(atlasText, textureCallback);

    // Create the skeleton
    const attachmentLoader = new AtlasAttachmentLoader(atlas);
    const binaryLoader = new SkeletonBinary(attachmentLoader);
    const skeletonData = binaryLoader.readSkeletonData(new Uint8Array(skelData));

    // biome-ignore lint/suspicious/noExplicitAny: runtime-4.1 types are slightly different from base Spine types
    return new Spine(skeletonData as any);
}

function getChibiSkinData(chibi: ChibiCharacter, skinName: string, viewType: ViewType): SpineFiles | null {
    let skin = chibi.skins.find((s) => s.name === skinName);
    if (!skin) {
        skin = chibi.skins.find((s) => s.name === "default");
    }
    if (!skin && chibi.skins.length > 0) {
        skin = chibi.skins[0];
    }

    if (!skin || !skin.hasSpineData) return null;

    const animationType = skin.animationTypes[viewType];
    if (animationType?.atlas && animationType?.skel && animationType?.png) {
        return animationType;
    }

    const fallbackOrder: ViewType[] = ["front", "dorm", "back"];
    for (const fallbackType of fallbackOrder) {
        const fallback = skin.animationTypes[fallbackType];
        if (fallback?.atlas && fallback?.skel && fallback?.png) {
            return fallback;
        }
    }

    return null;
}

function getAvailableViewTypes(chibi: ChibiCharacter, skinName: string): ViewType[] {
    let skin = chibi.skins.find((s) => s.name === skinName);
    if (!skin) {
        skin = chibi.skins.find((s) => s.name === "default");
    }
    if (!skin && chibi.skins.length > 0) {
        skin = chibi.skins[0];
    }

    if (!skin) return [];

    const types: ViewType[] = [];
    if (skin.animationTypes.front?.skel) types.push("front");
    if (skin.animationTypes.back?.skel) types.push("back");
    if (skin.animationTypes.dorm?.skel) types.push("dorm");
    return types;
}

export const ChibiViewer = memo(function ChibiViewer({ chibi, skinName }: ChibiViewerProps) {
    const appRef = useRef<PIXI.Application | null>(null);
    const spineRef = useRef<import("pixi-spine").Spine | null>(null);
    const canvasContainerRef = useRef<HTMLDivElement>(null);
    const mountedRef = useRef(true);
    const loadIdRef = useRef(0);

    const [selectedAnimation, setSelectedAnimation] = useState<string>("Idle");
    const [availableAnimations, setAvailableAnimations] = useState<string[]>([]);
    const [viewType, setViewType] = useState<ViewType>("front");
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const availableViewTypes = useMemo(() => getAvailableViewTypes(chibi, skinName), [chibi, skinName]);

    // Initialize view type based on available types
    useEffect(() => {
        if (availableViewTypes.length > 0 && !availableViewTypes.includes(viewType)) {
            setViewType(availableViewTypes[0] as ViewType);
        }
    }, [availableViewTypes, viewType]);

    // Main effect for app initialization and spine loading
    useEffect(() => {
        if (!canvasContainerRef.current) return;

        mountedRef.current = true;
        // Increment load ID to invalidate any in-flight loads from previous effect runs
        const currentLoadId = ++loadIdRef.current;
        let animationFrameId: number | null = null;

        const cleanup = () => {
            if (animationFrameId) {
                cancelAnimationFrame(animationFrameId);
            }

            if (spineRef.current && appRef.current) {
                appRef.current.stage.removeChild(spineRef.current);
                spineRef.current.destroy();
                spineRef.current = null;
            }

            if (appRef.current) {
                appRef.current.destroy(true, { children: true, texture: true });
                appRef.current = null;
            }
        };

        const loadSpine = async () => {
            // Check if this load is still valid (not superseded by a newer effect run)
            if (currentLoadId !== loadIdRef.current || !mountedRef.current) return;

            const skinData = getChibiSkinData(chibi, skinName, viewType);

            if (!skinData || !skinData.atlas || !skinData.skel || !skinData.png) {
                setError("No spine data available");
                setIsLoading(false);
                return;
            }

            const skelUrl = `/api/cdn${encodeAssetPath(skinData.skel)}`;
            const atlasUrl = `/api/cdn${encodeAssetPath(skinData.atlas)}`;
            const pngUrl = `/api/cdn${encodeAssetPath(skinData.png)}`;

            setIsLoading(true);
            setError(null);

            try {
                // Check again before async operation
                if (currentLoadId !== loadIdRef.current || !mountedRef.current) return;

                // Clean up previous spine
                if (spineRef.current && appRef.current) {
                    appRef.current.stage.removeChild(spineRef.current);
                    spineRef.current.destroy();
                    spineRef.current = null;
                }

                // Load spine with custom URL encoding to handle # characters
                const spine = await loadSpineWithEncodedUrls(skelUrl, atlasUrl, pngUrl);

                // Critical check after async load - abort if this load is stale
                if (currentLoadId !== loadIdRef.current || !mountedRef.current || !appRef.current) {
                    spine.destroy();
                    return;
                }

                spineRef.current = spine;

                // Configure spine
                spine.state.timeScale = ANIMATION_SPEED;

                const animations = spine.spineData.animations.map((a: { name: string }) => a.name);
                setAvailableAnimations(animations);

                // Position and scale
                const { width, height } = appRef.current.screen;
                spine.x = width * CHIBI_OFFSET_X;
                spine.y = height * CHIBI_OFFSET_Y;
                const scale = Math.min(width / 600, height / 400) * CHIBI_SCALE;
                spine.scale.set(scale);

                // Set initial animation
                let initialAnim = "Idle";
                if (viewType === "dorm") {
                    initialAnim = animations.includes("Relax") ? "Relax" : animations.includes("Idle") ? "Idle" : animations[0] || "Idle";
                } else {
                    initialAnim = animations.includes("Start") ? "Start" : animations.includes("Idle") ? "Idle" : animations[0] || "Idle";
                }

                const isStartAnim = initialAnim === "Start" || initialAnim === "Start_A";
                spine.state.setAnimation(0, initialAnim, !isStartAnim);
                setSelectedAnimation(initialAnim);

                // Transition from Start to Idle
                if (isStartAnim) {
                    spine.state.addListener({
                        complete: (entry) => {
                            const animName = (entry as unknown as { animation?: { name?: string } })?.animation?.name;
                            if ((animName === "Start" || animName === "Start_A") && spineRef.current) {
                                if (animations.includes("Idle")) {
                                    spineRef.current.state.setAnimation(0, "Idle", true);
                                    setSelectedAnimation("Idle");
                                }
                            }
                        },
                    });
                }

                appRef.current.stage.addChild(spine);
                setIsLoading(false);
            } catch (err) {
                console.error("Failed to load Spine:", err);
                // Only update state if this load is still current
                if (currentLoadId === loadIdRef.current && mountedRef.current) {
                    setError("Failed to load chibi");
                    setIsLoading(false);
                }
            }
        };

        const initApp = () => {
            const container = canvasContainerRef.current;
            if (!container) return;

            cleanup();

            // Clear any existing canvas elements
            while (container.firstChild) {
                container.removeChild(container.firstChild);
            }

            const containerWidth = container.clientWidth || 300;
            const containerHeight = container.clientHeight || 180;

            // PixiJS v7 uses constructor options
            const app = new PIXI.Application({
                width: containerWidth,
                height: containerHeight,
                backgroundColor: 0x111014,
                antialias: true,
                resolution: typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1,
                autoDensity: true,
            });

            // Check if this initialization is still valid
            if (currentLoadId !== loadIdRef.current || !mountedRef.current) {
                app.destroy(true, { children: true, texture: true });
                return;
            }

            appRef.current = app;
            container.appendChild(app.view as HTMLCanvasElement);

            // Animation loop
            const tick = () => {
                if (!mountedRef.current) return;
                if (spineRef.current) {
                    spineRef.current.update(0.016); // ~60fps
                }
                if (appRef.current?.renderer) {
                    appRef.current.renderer.render(appRef.current.stage);
                }
                animationFrameId = requestAnimationFrame(tick);
            };
            animationFrameId = requestAnimationFrame(tick);

            loadSpine();
        };

        initApp();

        return () => {
            mountedRef.current = false;
            cleanup();
        };
    }, [chibi, skinName, viewType]);

    const handleAnimationChange = (value: string) => {
        setSelectedAnimation(value);
        if (spineRef.current && value) {
            spineRef.current.state.setAnimation(0, value, true);
            spineRef.current.state.timeScale = ANIMATION_SPEED;
        }
    };

    const handleViewTypeChange = (value: string) => {
        setViewType(value as ViewType);
    };

    return (
        <div className="w-full rounded-lg border border-border bg-card/30 p-3">
            <h4 className="mb-2 font-medium text-foreground text-sm">Chibi Preview</h4>

            <div className="mb-3 flex flex-wrap gap-2">
                <Select disabled={isLoading || availableViewTypes.length <= 1} onValueChange={handleViewTypeChange} value={viewType}>
                    <SelectTrigger className="h-8 w-[90px] text-xs">
                        <SelectValue placeholder="View" />
                    </SelectTrigger>
                    <SelectContent>
                        {availableViewTypes.includes("front") && <SelectItem value="front">Front</SelectItem>}
                        {availableViewTypes.includes("back") && <SelectItem value="back">Back</SelectItem>}
                        {availableViewTypes.includes("dorm") && <SelectItem value="dorm">Dorm</SelectItem>}
                    </SelectContent>
                </Select>

                <Select disabled={isLoading || availableAnimations.length === 0} onValueChange={handleAnimationChange} value={selectedAnimation}>
                    <SelectTrigger className="h-8 min-w-[100px] flex-1 text-xs">
                        <SelectValue placeholder="Animation" />
                    </SelectTrigger>
                    <SelectContent>
                        {availableAnimations.map((anim) => (
                            <SelectItem key={anim} value={anim}>
                                {anim}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>

            <div className="relative h-[180px] w-full overflow-hidden rounded-md bg-[#111014]">
                <div className="h-full w-full" ref={canvasContainerRef} />
                {isLoading && (
                    <div className="absolute inset-0 flex items-center justify-center">
                        <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                    </div>
                )}
                {error && (
                    <div className="absolute inset-0 flex items-center justify-center p-2">
                        <div className="text-center text-muted-foreground text-xs">{error}</div>
                    </div>
                )}
            </div>
        </div>
    );
});
