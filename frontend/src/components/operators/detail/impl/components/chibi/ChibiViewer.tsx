import { ListEndIcon, RepeatIcon } from "lucide-react";
import * as PIXI from "pixi.js";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { FrameStepper, type IFramePosition } from "#/components/ui/frame-stepper";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Spinner } from "#/components/ui/spinner";
import { Toggle } from "#/components/ui/toggle";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import type { IChibiCharacter, IChibiSkin } from "#/lib/api/chibis";
import { fileStem, framePngFilename, framePngSizeOn, type IFrameRegion, renderFramePng, unionRegion } from "#/lib/frame-png";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { capitalize, cn, downloadBlob } from "#/lib/utils";
import { clipFrameCount, clipInfo, groupAnimations, type IAnimationGroup, type IAnimationLike, nextClipName } from "./animation-list";
import { AnimationPanel } from "./animation-panel";
import type { messages } from "./ChibiViewer.messages";
import { CHIBI_OFFSET_X, CHIBI_OFFSET_Y, SPINE_FALLBACK_FPS, type ViewType } from "./constants";
import { ExportGroup, type IRecordOptions } from "./download-button";
import { chibiMaxScale, getAvailableViewTypes, loadSpineWithEncodedURLs, poseBounds, resolveChibiView } from "./helpers";
import type { ExportFormat } from "./recorder";
import { BackgroundSwatches, type StageBackground, StageHint, StageTools, stageBackgroundColor, stageBackgroundCss } from "./stage-tools";
import { useRecorder } from "./use-recorder";

type Spine = import("pixi-spine").Spine;

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

const VIEW_MESSAGE_KEY: Partial<Record<ViewType, keyof typeof messages & string>> = {
    front: "chibi.view.front",
    back: "chibi.view.back",
    dorm: "chibi.view.dorm",
    down: "chibi.view.down",
};

const SPEEDS = [0.25, 0.5, 1, 2] as const;
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 10;
/** Share of the stage a fitted clip fills. */
const FIT_MARGIN = 0.9;
/** Grid pitch and reach, in skeleton units around the skeleton's origin (its feet). */
const GRID_STEP = 50;
const GRID_EXTENT = 1500;

/** What the header line says about the loaded skeleton. */
interface ISkeletonMeta {
    file: string;
    version: string;
    bones: number;
}

/** Pan in stage pixels and zoom as a factor, around the stage centre. */
interface IView {
    x: number;
    y: number;
    zoom: number;
}

const HOME_VIEW: IView = { x: 0, y: 0, zoom: 1 };

export function ChibiViewer({ chibi, skin, server }: IChibiViewerProps) {
    const t: TypedT<typeof messages> = useT("operators");
    const appRef = useRef<PIXI.Application | null>(null);
    const spineRef = useRef<Spine | null>(null);
    // Pan, zoom and flip act on this container, so the skeleton's own transform stays the
    // fitted layout the recorder and the PNG export read.
    const viewRootRef = useRef<PIXI.Container | null>(null);
    const gridRef = useRef<PIXI.Graphics | null>(null);
    const canvasContainerRef = useRef<HTMLDivElement>(null);
    const mountedRef = useRef(true);
    const loadIdRef = useRef(0);
    const recordingRef = useRef(false);
    const viewerRef = useRef<HTMLDivElement>(null);
    // Read by the tick loop, so a pause takes effect on the next frame without a re-render.
    const pausedRef = useRef(false);
    const [paused, setPaused] = useState(false);
    const loopRef = useRef(true);
    const [loop, setLoop] = useState(true);
    const chainRef = useRef(false);
    const [chain, setChain] = useState(false);
    const speedRef = useRef(1);
    const [speed, setSpeed] = useState(1);

    const viewRef = useRef<IView>(HOME_VIEW);
    const [zoom, setZoom] = useState(1);
    const flipRef = useRef(false);
    const [flipped, setFlipped] = useState(false);
    const showGridRef = useRef(false);
    const [showGrid, setShowGrid] = useState(false);
    const [background, setBackground] = useState<StageBackground>("checker");
    const [customBackground, setCustomBackground] = useState("#3b82f6");
    const [isFullscreen, setIsFullscreen] = useState(false);

    const [selectedAnimation, setSelectedAnimation] = useState<string>("Idle");
    const [groups, setGroups] = useState<IAnimationGroup[]>([]);
    const groupsRef = useRef<IAnimationGroup[]>([]);
    groupsRef.current = groups;
    const [meta, setMeta] = useState<ISkeletonMeta | null>(null);
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

    /** Loops unless chain play is on: a chained clip has to end for the next to start. */
    const entryLoops = useCallback(() => loopRef.current && !chainRef.current, []);

    /** Starts a clip from its first frame with the current loop and speed. */
    const playAnimation = useCallback(
        (name: string) => {
            const spine = spineRef.current;
            if (!spine) return;
            setSelectedAnimation(name);
            spine.state.setAnimation(0, name, entryLoops());
            spine.state.timeScale = speedRef.current;
        },
        [entryLoops],
    );

    useEffect(() => {
        if (!canvasContainerRef.current) return;
        if (!chibi || !skin) return;

        const skinName = skin.name;

        mountedRef.current = true;
        const currentLoadId = ++loadIdRef.current;
        let animationFrameId: number | null = null;

        const cleanup = () => {
            if (animationFrameId) cancelAnimationFrame(animationFrameId);

            if (spineRef.current) {
                spineRef.current.parent?.removeChild(spineRef.current);
                spineRef.current.destroy();
                spineRef.current = null;
            }
            if (appRef.current) {
                appRef.current.destroy(true, { children: true, texture: true });
                appRef.current = null;
                viewRootRef.current = null;
                gridRef.current = null;
            }
        };

        const loadSpine = async () => {
            if (currentLoadId !== loadIdRef.current || !mountedRef.current) return;

            const resolved = resolveChibiView(chibi, skinName, viewType);
            const skinData = resolved?.files;
            if (!resolved || !skinData?.atlas || !skinData.skel || !skinData.png) {
                setError(t("chibi.error.noData"));
                setIsLoading(false);
                return;
            }

            setIsLoading(true);
            setError(null);

            try {
                if (currentLoadId !== loadIdRef.current || !mountedRef.current) return;

                if (spineRef.current) {
                    spineRef.current.parent?.removeChild(spineRef.current);
                    spineRef.current.destroy();
                    spineRef.current = null;
                }

                const spine = await loadSpineWithEncodedURLs(skinData.skel, skinData.atlas, server);

                if (currentLoadId !== loadIdRef.current || !mountedRef.current || !appRef.current || !viewRootRef.current) {
                    spine.destroy();
                    return;
                }

                spineRef.current = spine;
                spine.autoUpdate = false;
                spine.state.timeScale = speedRef.current;

                const fps = spineFps(spine);
                const animations = spine.spineData.animations as unknown as IAnimationLike[];
                setGroups(groupAnimations(animations.map((animation) => clipInfo(animation, fps))));
                setMeta({ file: fileName(skinData.skel), version: spine.spineData.version ?? "", bones: spine.spineData.bones.length });

                const initialAnim = pickInitialAnimation(
                    animations.map((a) => a.name),
                    viewType,
                );
                layoutBase(spine, appRef.current.screen.width, appRef.current.screen.height);
                spine.state.addListener({
                    // Chain play: a clip that ends hands over to the next one in the list. Without
                    // chain or loop it stops on its last frame, paused, so play starts it over.
                    complete: (entry) => {
                        if (recordingRef.current || spineRef.current !== spine) return;
                        if (!chainRef.current) {
                            if (!loopRef.current) {
                                pausedRef.current = true;
                                setPaused(true);
                            }
                            return;
                        }
                        const current = (entry as unknown as { animation?: { name?: string } }).animation?.name;
                        const next = current ? nextClipName(groupsRef.current, current) : null;
                        if (next) playAnimation(next);
                    },
                });
                playAnimation(initialAnim);

                viewRootRef.current.addChild(spine);
                setIsLoading(false);
            } catch (err) {
                console.error("Failed to load Spine:", err);
                if (currentLoadId === loadIdRef.current && mountedRef.current) {
                    setError(t("chibi.error.failed"));
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

            let app: PIXI.Application;
            try {
                app = new PIXI.Application({
                    width: containerWidth,
                    height: containerHeight,
                    backgroundAlpha: 0,
                    antialias: true,
                    resolution: typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1,
                    autoDensity: true,
                    // The tick below updates and renders; PIXI's own ticker would draw the stage a
                    // second time every frame.
                    autoStart: false,
                });
            } catch (err) {
                // pixi.js 7 has no canvas fallback, so a browser without WebGL throws here.
                console.error("Failed to create the chibi renderer:", err);
                setError(t("chibi.error.webgl"));
                setIsLoading(false);
                return;
            }
            app.ticker.stop();

            if (currentLoadId !== loadIdRef.current || !mountedRef.current) {
                app.destroy(true, { children: true, texture: true });
                return;
            }

            appRef.current = app;
            const viewRoot = new PIXI.Container();
            const grid = buildGrid();
            grid.visible = showGridRef.current;
            viewRoot.addChild(grid);
            app.stage.addChild(viewRoot);
            viewRootRef.current = viewRoot;
            gridRef.current = grid;
            const canvas = app.view as HTMLCanvasElement;
            canvas.style.display = "block";
            container.appendChild(canvas);

            let lastTick = performance.now();
            const tick = (now: number) => {
                if (!mountedRef.current) return;
                const dt = Math.min((now - lastTick) / 1000, 0.1);
                lastTick = now;
                // Paused still applies the pose at zero delta, so a step, an
                // animation change or the recorder's restore shows at once.
                if (!recordingRef.current && spineRef.current) spineRef.current.update(pausedRef.current ? 0 : dt);
                const current = appRef.current;
                if (current?.renderer) {
                    applyView(current, viewRef.current, flipRef.current);
                    // Rendered while recording too: the recorder has moved the
                    // spine onto its own app, so the live canvas shows it gone, not a frozen frame.
                    current.renderer.render(current.stage);
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
    }, [chibi, skin, viewType, server, t, playAnimation]);

    // The stage follows its box: wider on a wide page, the whole screen in fullscreen.
    useEffect(() => {
        const container = canvasContainerRef.current;
        if (!container || typeof ResizeObserver === "undefined") return;
        const observer = new ResizeObserver(() => {
            const app = appRef.current;
            const width = container.clientWidth;
            const height = container.clientHeight;
            if (!app || width === 0 || height === 0) return;
            if (app.screen.width === width && app.screen.height === height) return;
            app.renderer.resize(width, height);
            if (spineRef.current && !recordingRef.current) layoutBase(spineRef.current, width, height);
        });
        observer.observe(container);
        return () => observer.disconnect();
    }, []);

    useEffect(() => {
        const onChange = () => setIsFullscreen(document.fullscreenElement === viewerRef.current && viewerRef.current !== null);
        document.addEventListener("fullscreenchange", onChange);
        return () => document.removeEventListener("fullscreenchange", onChange);
    }, []);

    const setView = useCallback((next: IView) => {
        viewRef.current = next;
        setZoom(next.zoom);
    }, []);

    // Ctrl (or a trackpad pinch, which arrives as Ctrl + wheel) zooms about the pointer; a
    // plain wheel is left to scroll the page. Bound natively: React's wheel handler is passive.
    useEffect(() => {
        const container = canvasContainerRef.current;
        if (!container) return;
        const onWheel = (event: WheelEvent) => {
            if (!event.ctrlKey && !event.metaKey) return;
            event.preventDefault();
            const rect = container.getBoundingClientRect();
            const view = viewRef.current;
            const zoomTo = clamp(view.zoom * Math.exp(-event.deltaY * 0.002), MIN_ZOOM, MAX_ZOOM);
            const ratio = zoomTo / view.zoom;
            // The point under the pointer stays put: offsets from the centre scale by the ratio.
            const px = event.clientX - rect.left - rect.width / 2;
            const py = event.clientY - rect.top - rect.height / 2;
            setView({ zoom: zoomTo, x: px - (px - view.x) * ratio, y: py - (py - view.y) * ratio });
        };
        container.addEventListener("wheel", onWheel, { passive: false });
        return () => container.removeEventListener("wheel", onWheel);
    }, [setView]);

    const dragRef = useRef<{ id: number; x: number; y: number } | null>(null);
    const [dragging, setDragging] = useState(false);
    const handlePointerDown = (event: React.PointerEvent<HTMLDivElement>) => {
        // Touch keeps scrolling the page, except in fullscreen where there is no page.
        if (event.button !== 0 || (event.pointerType === "touch" && !isFullscreen)) return;
        event.currentTarget.setPointerCapture(event.pointerId);
        dragRef.current = { id: event.pointerId, x: event.clientX, y: event.clientY };
        setDragging(true);
    };
    const handlePointerMove = (event: React.PointerEvent<HTMLDivElement>) => {
        const drag = dragRef.current;
        if (!drag || drag.id !== event.pointerId) return;
        const view = viewRef.current;
        viewRef.current = { ...view, x: view.x + event.clientX - drag.x, y: view.y + event.clientY - drag.y };
        dragRef.current = { ...drag, x: event.clientX, y: event.clientY };
    };
    const endDrag = (event: React.PointerEvent<HTMLDivElement>) => {
        if (dragRef.current?.id !== event.pointerId) return;
        dragRef.current = null;
        setDragging(false);
    };

    // Picking the playing clip again restarts it.
    const handleAnimationChange = (value: string) => playAnimation(value);

    const handleViewTypeChange = (value: ViewType | null) => {
        setViewType(value as ViewType);
    };

    const handlePausedChange = useCallback((next: boolean) => {
        const entry = spineRef.current?.state.tracks[0];
        // A clip that played once and stopped starts over on play.
        if (!next && entry && !entry.loop && entry.trackTime >= entry.animationEnd) entry.trackTime = 0;
        pausedRef.current = next;
        setPaused(next);
    }, []);

    const handleLoopChange = (next: boolean) => {
        loopRef.current = next;
        setLoop(next);
        const entry = spineRef.current?.state.tracks[0];
        if (entry) entry.loop = entryLoops();
    };

    const handleChainChange = (next: boolean) => {
        chainRef.current = next;
        setChain(next);
        const entry = spineRef.current?.state.tracks[0];
        if (entry) entry.loop = entryLoops();
    };

    const handleSpeedChange = (next: number) => {
        speedRef.current = next;
        setSpeed(next);
        if (spineRef.current) spineRef.current.state.timeScale = next;
    };

    const getPosition = useCallback((): IFramePosition | null => {
        const spine = spineRef.current;
        const entry = spine?.state.tracks[0];
        if (!spine || !entry) return null;
        const fps = spineFps(spine);
        const duration = entry.animationEnd;
        const total = clipFrameCount(duration, fps);
        if (!entry.loop) {
            // A clip played once holds its last frame instead of wrapping to the first.
            const seconds = Math.min(entry.trackTime, duration);
            return { frame: Math.min(Math.round(seconds * fps), total - 1), total, seconds, duration };
        }
        const seconds = duration > 0 ? entry.trackTime % duration : 0;
        return { frame: Math.round(seconds * fps) % total, total, seconds, duration };
    }, []);

    // Seeks the track to a frame. Track time restarts inside the clip, which a
    // looping entry wraps the same way; the paused tick applies the new pose.
    const handleSeek = useCallback((frame: number) => {
        const spine = spineRef.current;
        const entry = spine?.state.tracks[0];
        if (!spine || !entry) return;
        entry.trackTime = frame / spineFps(spine);
    }, []);

    const handleStep = useCallback(
        (direction: -1 | 1) => {
            const pos = getPosition();
            if (!pos) return;
            handleSeek((pos.frame + direction + pos.total) % pos.total);
        },
        [getPosition, handleSeek],
    );

    // The clip's extent joined with this frame's, in skeleton units: every
    // frame of one clip crops to the same box, so a stepped sequence lines up.
    // Keyed by the track entry: every `setAnimation` makes a new one.
    const clipRegionRef = useRef<{ entry: object; region: IFrameRegion | null } | null>(null);
    const frameRegion = useCallback((): IFrameRegion | null => {
        const spine = spineRef.current;
        const entry = spine?.state.tracks[0];
        if (!spine || !entry) return null;
        let clip = clipRegionRef.current;
        if (clip?.entry !== entry) {
            clip = { entry, region: measureClipRegion(spine) };
            clipRegionRef.current = clip;
        }
        return unionRegion(clip.region, poseBounds(spine));
    }, []);

    const handleFit = () => {
        const spine = spineRef.current;
        const app = appRef.current;
        const region = frameRegion();
        if (!spine || !app || !region) return;
        const { width, height } = app.screen;
        const s = spine.scale.y;
        const fitZoom = clamp(Math.min((width * FIT_MARGIN) / (region.width * s), (height * FIT_MARGIN) / (region.height * s)), MIN_ZOOM, MAX_ZOOM);
        // The region's centre on the unzoomed stage, as an offset from the stage centre;
        // the view scales that offset (mirrored when flipped), so pan cancels it.
        const cx = spine.x + (region.x + region.width / 2) * s - width / 2;
        const cy = spine.y + (region.y + region.height / 2) * s - height / 2;
        setView({ zoom: fitZoom, x: -cx * fitZoom * (flipRef.current ? -1 : 1), y: -cy * fitZoom });
    };

    const handleFlip = () => {
        flipRef.current = !flipRef.current;
        setFlipped(flipRef.current);
    };

    const handleGrid = () => {
        showGridRef.current = !showGridRef.current;
        setShowGrid(showGridRef.current);
        if (gridRef.current) gridRef.current.visible = showGridRef.current;
    };

    const handleFullscreen = () => {
        if (document.fullscreenElement) void document.exitFullscreen();
        else void viewerRef.current?.requestFullscreen();
    };

    const fileParts = useCallback((animation: string) => [chibi?.name, skin?.name, viewType, animation], [chibi?.name, skin?.name, viewType]);

    const frameSizeOf = useCallback(
        (scale: number) => {
            const region = frameRegion();
            const renderer = appRef.current?.renderer;
            return region && renderer ? framePngSizeOn(renderer, region, scale) : null;
        },
        [frameRegion],
    );

    const handleSaveFrame = useCallback(
        async (scale: number) => {
            const spine = spineRef.current;
            const renderer = appRef.current?.renderer;
            const region = frameRegion();
            const pos = getPosition();
            if (!spine || !renderer || !region || !pos || recordingRef.current) return;
            // Zero delta, as the paused tick applies it: a step taken since the
            // last tick is posed before the draw, and the clip does not move.
            spine.update(0);
            try {
                const blob = await renderFramePng(renderer, spine, region, scale, flipRef.current);
                downloadBlob(blob, framePngFilename({ parts: fileParts(selectedAnimation), ...pos, scale }));
            } catch (err) {
                console.error("Failed to save the chibi frame:", err);
            }
        },
        [frameRegion, getPosition, fileParts, selectedAnimation],
    );

    const handleRecord = async (format: ExportFormat, { scale, fps, loopCount }: IRecordOptions) => {
        const color = stageBackgroundColor(background, customBackground);
        const stem = fileStem(fileParts(selectedAnimation));
        await startRecording(format, { scale, fps, loopCount, transparentBg: color === null, backgroundColor: color ?? undefined, flipX: flipRef.current }, scale === 1 ? stem : `${stem}@${scale}x`);
        // The recorder restarts the clip looping; put the viewer's own playback back.
        const entry = spineRef.current?.state.tracks[0];
        if (entry) entry.loop = entryLoops();
    };

    const disabled = isLoading || !!error || isRecording;

    return (
        <div className={cn("@container w-full rounded-lg border border-border bg-card/30 p-3", isFullscreen && "flex h-full flex-col overflow-auto bg-background")} ref={viewerRef}>
            <div className="mb-2 flex flex-wrap items-start justify-between gap-2">
                <div className="min-w-0">
                    <h4 className="m-0 font-medium text-foreground">{t("chibi.title")}</h4>
                    {meta && (
                        <p className="m-0 mt-0.5 flex flex-wrap items-baseline gap-x-2 font-mono text-[10px] text-muted-foreground">
                            <span className="truncate text-foreground/80 text-xs">{meta.file}</span>
                            <span className="uppercase tracking-wider">{t("chibi.meta", { version: meta.version, bones: meta.bones })}</span>
                        </p>
                    )}
                </div>
                <Select disabled={isLoading || availableViewTypes.length <= 1 || isRecording} onValueChange={handleViewTypeChange} value={viewType}>
                    <SelectTrigger className="h-8 w-22.5 text-xs">
                        <SelectValue placeholder={t("chibi.view")}>
                            {(value: string) => {
                                const messageKey = VIEW_MESSAGE_KEY[value as ViewType];
                                return messageKey ? t(messageKey) : capitalize(value);
                            }}
                        </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                        {availableViewTypes.includes("front") && <SelectItem value="front">{t("chibi.view.front")}</SelectItem>}
                        {availableViewTypes.includes("back") && <SelectItem value="back">{t("chibi.view.back")}</SelectItem>}
                        {availableViewTypes.includes("dorm") && <SelectItem value="dorm">{t("chibi.view.dorm")}</SelectItem>}
                        {availableViewTypes.includes("down") && <SelectItem value="down">{t("chibi.view.down")}</SelectItem>}
                    </SelectContent>
                </Select>
            </div>

            <div className={cn("grid @2xl:grid-cols-[minmax(0,1fr)_15rem] gap-3", isFullscreen && "min-h-0 flex-1")}>
                <div className={cn("flex min-w-0 flex-col gap-2", isFullscreen && "min-h-0")}>
                    <div className={cn("relative @2xl:h-72 h-64 w-full overflow-hidden rounded-md", isFullscreen && "@2xl:h-auto h-auto min-h-72 flex-1")} style={{ background: stageBackgroundCss(background, customBackground) }}>
                        <div
                            aria-label={t("chibi.stage")}
                            className={cn("h-full w-full", !disabled && (dragging ? "cursor-grabbing" : "cursor-grab"), isFullscreen && "touch-none")}
                            onDoubleClick={() => setView(HOME_VIEW)}
                            onLostPointerCapture={endDrag}
                            onPointerCancel={endDrag}
                            onPointerDown={handlePointerDown}
                            onPointerMove={handlePointerMove}
                            onPointerUp={endDrag}
                            ref={canvasContainerRef}
                            role="application"
                        />
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
                        <StageTools flipped={flipped} fullscreen={isFullscreen} grid={showGrid} onFit={handleFit} onFlip={handleFlip} onFullscreen={handleFullscreen} onGrid={handleGrid} zoom={zoom} />
                        <StageHint />
                        <BackgroundSwatches custom={customBackground} onChange={setBackground} onCustomChange={setCustomBackground} value={background} />
                    </div>

                    <FrameStepper disabled={disabled} getPosition={getPosition} onPausedChange={handlePausedChange} onSeek={handleSeek} onStep={handleStep} paused={paused} scopeRef={viewerRef}>
                        <Tooltip>
                            <TooltipTrigger render={<Toggle aria-label={t("chibi.loop")} className="h-7 min-w-0 gap-1 px-1.5 text-xs sm:h-6 sm:min-w-0 sm:text-xs" disabled={disabled} onPressedChange={handleLoopChange} pressed={loop} size="sm" />}>
                                <RepeatIcon />
                                {t("chibi.loop")}
                            </TooltipTrigger>
                            <TooltipPopup>{t("chibi.loopHint")}</TooltipPopup>
                        </Tooltip>
                        <Tooltip>
                            <TooltipTrigger render={<Toggle aria-label={t("chibi.chain")} className="h-7 min-w-0 gap-1 px-1.5 text-xs sm:h-6 sm:min-w-0 sm:text-xs" disabled={disabled} onPressedChange={handleChainChange} pressed={chain} size="sm" />}>
                                <ListEndIcon />
                                {t("chibi.chain")}
                            </TooltipTrigger>
                            <TooltipPopup>{t("chibi.chainHint")}</TooltipPopup>
                        </Tooltip>
                        <ToggleGroup aria-label={t("chibi.speed")} disabled={disabled} onValueChange={(next) => next[0] && handleSpeedChange(Number(next[0]))} size="sm" value={[String(speed)]} variant="outline">
                            {SPEEDS.map((option) => (
                                <ToggleGroupItem className="h-7 min-w-0 px-1.5 font-mono text-[11px] sm:h-6 sm:min-w-0 sm:text-[11px]" key={option} value={String(option)}>
                                    {t("chibi.speedOption", { speed: option })}
                                </ToggleGroupItem>
                            ))}
                        </ToggleGroup>
                    </FrameStepper>

                    <ExportGroup disabled={disabled || groups.length === 0} isRecording={isRecording} onCancel={cancelRecording} onOpenPng={() => handlePausedChange(true)} onRecord={handleRecord} onSavePng={handleSaveFrame} pngSizeOf={frameSizeOf} progress={progress} />
                </div>

                {/* Beside the stage the list spans the whole column, stage to export row, and scrolls
                    inside it: laid out absolutely, its length never stretches the row. */}
                <div className="relative @2xl:min-h-72">
                    <AnimationPanel className="@2xl:absolute @2xl:inset-0 @2xl:max-h-none max-h-72" disabled={disabled} groups={groups} onSelect={handleAnimationChange} selected={selectedAnimation} />
                </div>
            </div>
        </div>
    );
}

function clamp(value: number, min: number, max: number): number {
    return Math.min(max, Math.max(min, value));
}

/** The last path segment, URL-decoded: `char_002_amiya.skel`. */
function fileName(path: string): string {
    const last = path.split(/[\\/]/).pop() ?? path;
    try {
        return decodeURIComponent(last);
    } catch {
        return last;
    }
}

/** The fitted skeleton: centred, feet near the bottom, sized to the stage. The view zooms from here. */
function layoutBase(spine: Spine, width: number, height: number): void {
    spine.x = width * CHIBI_OFFSET_X;
    spine.y = height * CHIBI_OFFSET_Y;
    spine.scale.set(chibiMaxScale(width, height));
}

/** Pan, zoom and flip about the stage centre; the grid follows the skeleton's transform. */
function applyView(app: PIXI.Application, view: IView, flip: boolean): void {
    const root = app.stage.children[0];
    if (!(root instanceof PIXI.Container)) return;
    const { width, height } = app.screen;
    root.pivot.set(width / 2, height / 2);
    root.position.set(width / 2 + view.x, height / 2 + view.y);
    root.scale.set(view.zoom * (flip ? -1 : 1), view.zoom);
    const [grid, spine] = root.children;
    if (grid?.visible && spine) {
        grid.position.copyFrom(spine.position);
        grid.scale.copyFrom(spine.scale);
    }
}

/** Guide lines in skeleton units: a light grid, the ground (y = 0) and centre (x = 0) stronger. Native lines stay 1px at any zoom. */
function buildGrid(): PIXI.Graphics {
    const grid = new PIXI.Graphics();
    grid.lineStyle({ width: 1, color: 0x8a8a8a, alpha: 0.25, native: true });
    for (let v = -GRID_EXTENT; v <= GRID_EXTENT; v += GRID_STEP) {
        if (v === 0) continue;
        grid.moveTo(v, -GRID_EXTENT).lineTo(v, GRID_EXTENT);
        grid.moveTo(-GRID_EXTENT, v).lineTo(GRID_EXTENT, v);
    }
    grid.lineStyle({ width: 1, color: 0x8a8a8a, alpha: 0.7, native: true });
    grid.moveTo(0, -GRID_EXTENT).lineTo(0, GRID_EXTENT);
    grid.moveTo(-GRID_EXTENT, 0).lineTo(GRID_EXTENT, 0);
    return grid;
}

/** Most poses a clip's extent is sampled at; a longer clip is sampled evenly. */
const CLIP_REGION_SAMPLES = 120;

/**
 * The union of the current clip's poses, one per frame up to
 * {@link CLIP_REGION_SAMPLES}. Seeks the track the way a frame step does and
 * puts it back, so the pose on screen is the one it found.
 */
function measureClipRegion(spine: Spine): IFrameRegion | null {
    const entry = spine.state.tracks[0];
    if (!entry) return null;
    const duration = entry.animationEnd;
    const samples = Math.min(CLIP_REGION_SAMPLES, clipFrameCount(duration, spineFps(spine)));
    const saved = entry.trackTime;
    let region: IFrameRegion | null = null;
    try {
        for (let i = 0; i < samples; i++) {
            entry.trackTime = (i / samples) * duration;
            spine.update(0);
            region = unionRegion(region, poseBounds(spine));
        }
    } finally {
        entry.trackTime = saved;
        spine.update(0);
    }
    return region;
}

/**
 * The skeleton's own dopesheet rate when its export kept it (`fps` is a
 * nonessential field), else 30. Census of all 4,496 EN skeletons on
 * 2026-10-11: the 1,307 that record a rate all record 30.0 (419 BattleFront,
 * 399 BattleBack, 487 Enemy, 2 DynIllust), none records anything else, 3,169
 * (every parsed Building one among them) were exported without it, and 20 did
 * not parse as 3.8 headers. So one frame is 1/30 s wherever it can be read,
 * and 30 is the only rate the data offers for the rest.
 */
function spineFps(spine: Spine): number {
    const fps = (spine.spineData as { fps?: number }).fps;
    return fps !== undefined && fps > 0 ? fps : SPINE_FALLBACK_FPS;
}
