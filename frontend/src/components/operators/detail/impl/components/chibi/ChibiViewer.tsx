import * as PIXI from "pixi.js";
import { useEffect, useMemo, useRef, useState } from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Spinner } from "#/components/ui/spinner";
import type { IChibiCharacter, IChibiSkin } from "#/lib/api/chibis";
import { capitalize } from "#/lib/utils";
import { ANIMATION_SPEED, CHIBI_OFFSET_X, CHIBI_OFFSET_Y, type ViewType } from "./constants";
import { DownloadButton } from "./download-button";
import { chibiMaxScale, getAvailableViewTypes, type IAnimationBounds, loadSpineWithEncodedURLs, measureAnimationBounds, resolveChibiView } from "./helpers";
import { useRecorder } from "./use-recorder";

interface IChibiViewerProps {
    chibi: IChibiCharacter | null;
    skin: IChibiSkin | null;
    server?: "en" | "cn";
}

/**
 * Dorm chibis rest on "Relax" when available; other views open on the looping
 * "Idle" (skeletons that ship a "Start" intro clip still begin on "Idle").
 */
function pickInitialAnimation(animations: string[], viewType: ViewType): string {
    if (viewType === "dorm") {
        if (animations.includes("Relax")) return "Relax";
        if (animations.includes("Idle")) return "Idle";
        return animations[0] ?? "Idle";
    }
    if (animations.includes("Start") || animations.includes("Idle")) return "Idle";
    return animations[0] ?? "Idle";
}

export function ChibiViewer({ chibi, skin, server }: IChibiViewerProps) {
    const appRef = useRef<PIXI.Application | null>(null);
    const spineRef = useRef<import("pixi-spine").Spine | null>(null);
    const canvasContainerRef = useRef<HTMLDivElement>(null);
    const mountedRef = useRef(true);
    const loadIdRef = useRef(0);
    const recordingRef = useRef(false);

    const [selectedAnimation, setSelectedAnimation] = useState<string>("Idle");
    const [availableAnimations, setAvailableAnimations] = useState<string[]>([]);
    const [animationBounds, setAnimationBounds] = useState<IAnimationBounds | null>(null);
    const [viewType, setViewType] = useState<ViewType>("front");
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const { isRecording, progress, startRecording, cancelRecording } = useRecorder({
        appRef,
        spineRef,
        selectedAnimation,
        recordingRef,
    });

    const availableViewTypes = useMemo(() => getAvailableViewTypes(chibi, skin?.name ?? null), [chibi, skin?.name]);

    useEffect(() => {
        if (availableViewTypes.length > 0 && !availableViewTypes.includes(viewType)) {
            setViewType(availableViewTypes[0] as ViewType);
        }
    }, [availableViewTypes, viewType]);

    useEffect(() => {
        if (!canvasContainerRef.current) return;
        if (!chibi || !skin) return;

        const skinName = skin.name;

        mountedRef.current = true;
        const currentLoadId = ++loadIdRef.current;
        let animationFrameId: number | null = null;

        const cleanup = () => {
            if (animationFrameId) cancelAnimationFrame(animationFrameId);

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
            if (currentLoadId !== loadIdRef.current || !mountedRef.current) return;

            const resolved = resolveChibiView(chibi, skinName, viewType);
            const skinData = resolved?.files;
            if (!resolved || !skinData?.atlas || !skinData.skel || !skinData.png) {
                setError("No spine data available");
                setIsLoading(false);
                return;
            }

            setIsLoading(true);
            setError(null);

            try {
                if (currentLoadId !== loadIdRef.current || !mountedRef.current) return;

                if (spineRef.current && appRef.current) {
                    appRef.current.stage.removeChild(spineRef.current);
                    spineRef.current.destroy();
                    spineRef.current = null;
                }

                const spine = await loadSpineWithEncodedURLs(skinData.skel, skinData.atlas, server);

                if (currentLoadId !== loadIdRef.current || !mountedRef.current || !appRef.current) {
                    spine.destroy();
                    return;
                }

                spineRef.current = spine;
                spine.autoUpdate = false;
                spine.state.timeScale = ANIMATION_SPEED;

                const animations = spine.spineData.animations.map((a: { name: string }) => a.name);
                setAvailableAnimations(animations);

                const initialAnim = pickInitialAnimation(animations, viewType);

                const bounds = measureAnimationBounds(spine, initialAnim);
                const { width, height } = appRef.current.screen;
                spine.x = width * CHIBI_OFFSET_X;
                spine.y = height * CHIBI_OFFSET_Y;
                spine.scale.set(chibiMaxScale(width, height));

                setAnimationBounds(bounds);
                spine.state.setAnimation(0, initialAnim, true);
                setSelectedAnimation(initialAnim);

                appRef.current.stage.addChild(spine);
                setIsLoading(false);
            } catch (err) {
                console.error("Failed to load Spine:", err);
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
            while (container.firstChild) container.removeChild(container.firstChild);

            const containerWidth = container.clientWidth || 300;
            const containerHeight = container.clientHeight || 180;

            const app = new PIXI.Application({
                width: containerWidth,
                height: containerHeight,
                backgroundAlpha: 0,
                antialias: true,
                resolution: typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1,
                autoDensity: true,
            });

            if (currentLoadId !== loadIdRef.current || !mountedRef.current) {
                app.destroy(true, { children: true, texture: true });
                return;
            }

            appRef.current = app;
            container.appendChild(app.view as HTMLCanvasElement);

            let lastTick = performance.now();
            const tick = (now: number) => {
                if (!mountedRef.current) return;
                const dt = Math.min((now - lastTick) / 1000, 0.1);
                lastTick = now;
                if (!recordingRef.current) {
                    if (spineRef.current) spineRef.current.update(dt);
                    if (appRef.current?.renderer) appRef.current.renderer.render(appRef.current.stage);
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
    }, [chibi, skin, viewType, server]);

    const handleAnimationChange = (value: string | null) => {
        setSelectedAnimation(value ?? selectedAnimation);
        if (spineRef.current && value) {
            setAnimationBounds(measureAnimationBounds(spineRef.current, value));
            spineRef.current.state.setAnimation(0, value, true);
            spineRef.current.state.timeScale = ANIMATION_SPEED;
        }
    };

    const handleViewTypeChange = (value: ViewType | null) => {
        setViewType(value as ViewType);
    };

    return (
        <div className="w-full rounded-lg border border-border bg-card/30 p-3">
            <h4 className="mb-2 font-medium text-foreground">Chibi Preview</h4>

            <div className="mb-3 flex flex-wrap items-center gap-2">
                <Select disabled={isLoading || availableViewTypes.length <= 1 || isRecording} onValueChange={handleViewTypeChange} value={viewType}>
                    <SelectTrigger className="h-8 w-22.5 text-xs">
                        <SelectValue placeholder="View">{(value: string) => capitalize(value)}</SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                        {availableViewTypes.includes("front") && <SelectItem value="front">Front</SelectItem>}
                        {availableViewTypes.includes("back") && <SelectItem value="back">Back</SelectItem>}
                        {availableViewTypes.includes("dorm") && <SelectItem value="dorm">Dorm</SelectItem>}
                    </SelectContent>
                </Select>

                <Select disabled={isLoading || availableAnimations.length === 0 || isRecording} onValueChange={handleAnimationChange} value={selectedAnimation}>
                    <SelectTrigger className="h-8 min-w-25 flex-1 text-xs">
                        <SelectValue placeholder="Animation" />
                    </SelectTrigger>
                    <SelectContent className="max-h-48">
                        {availableAnimations.map((anim) => (
                            <SelectItem key={anim} value={anim}>
                                {anim}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>

                <DownloadButton animationBounds={animationBounds} disabled={isLoading || !!error || availableAnimations.length === 0} isRecording={isRecording} onCancel={cancelRecording} onDownload={startRecording} progress={progress} />
            </div>
            <div className="relative h-45 w-full overflow-hidden rounded-md bg-muted">
                <div className="h-full w-full" ref={canvasContainerRef} />
                {isLoading && (
                    <div className="absolute inset-0 flex items-center justify-center">
                        <Spinner className="text-primary" />
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
}
