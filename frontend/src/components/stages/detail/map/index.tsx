import { useQuery } from "@tanstack/react-query";
import { Box, ChevronLeft, ChevronRight } from "lucide-react";
import { forwardRef, useCallback, useEffect, useImperativeHandle, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { getChibiSkinData } from "#/components/operators/detail/impl/components/chibi/helpers";
import { Button } from "#/components/ui/button";
import { enemyChibisQueryOptions, type IChibiCharacter } from "#/lib/api/chibis";
import type { IEnemy } from "#/lib/api/enemies";
import { EnemyStatsCard } from "../impl/EnemyStatsCard";
import { DEFAULT_MAP_SETTINGS, type IMapSettings } from "../impl/MapSettings";
import type { IStageEnemyStats } from "../impl/types";
import { GameMap } from "./impl/map-model";
import { buildRouteEntries } from "./impl/route/route-entries";
import type { EnemyHoverFn, ILevel, IMapOperator } from "./impl/types";
import { SPACING, TILE_SIZE } from "./impl/util/geometry";
import "./impl/effects.css";
import "./impl/tiles.css";
import "./impl/routes.css";
import { Board } from "./view/Board";
import type { IChibiWalker } from "./view/ChibiLayer";
import { DynamicChibiLayer } from "./view/ChibiLayer.lazy";
import { RoutesLayer } from "./view/RoutesLayer";
import { Watermark } from "./view/Watermark";

/** Enemies in the focused group spawn within this many seconds of the focus. */
const GROUP_WINDOW = 10;
/** Screen px per (tile/second) of enemy move speed - tuned for a lively-but-readable pace. */
const SPEED_SCALE = 100;
/** Fallback move speed (tiles/sec) when an enemy's stats are missing. */
const DEFAULT_MOVE_SPEED = 0.72;
/** Board px per map tile (tile + gap). */
const PX_PER_TILE = TILE_SIZE + SPACING;
/** Walk is sped up vs. real time by SPEED_SCALE/PX_PER_TILE; scale wait durations to match. */
const WAIT_TIME_SCALE = PX_PER_TILE / SPEED_SCALE;

const BOARD_PADDING_Y = 48;
const H_GUTTER = 24;
/** Hover-intent delay (ms) before the enemy tooltip opens - long enough that a chibi merely walking
 * past the cursor (or loading in under it) never flashes the tooltip, short enough to feel instant on a real hover. */
const HOVER_OPEN_DELAY = 180;
/** Keep the board within this fraction of the viewport height so tall maps stay fully visible without dominating the page. */
const MAX_VIEWPORT_HEIGHT_FRACTION = 0.82;

function MapControls({ onPrev, onNext, atStart, atEnd, hasRoutes, is3D, onToggle3D }: { onPrev: () => void; onNext: () => void; atStart: boolean; atEnd: boolean; hasRoutes: boolean; is3D: boolean; onToggle3D: () => void }) {
    return (
        <div className="absolute top-3 right-3 z-50 flex items-center gap-2 sm:top-4 sm:right-4 sm:gap-1.5">
            <Button aria-label="Toggle 3D view" aria-pressed={is3D} onClick={onToggle3D} size="icon" variant={is3D ? "secondary" : "outline"}>
                <Box />
            </Button>
            {hasRoutes && (
                <>
                    <Button aria-label="Previous enemy" disabled={atStart} onClick={onPrev} size="icon" variant="outline">
                        <ChevronLeft />
                    </Button>
                    <Button aria-label="Next enemy" disabled={atEnd} onClick={onNext} size="icon" variant="outline">
                        <ChevronRight />
                    </Button>
                </>
            )}
        </div>
    );
}

/** Imperative controls a parent can drive the map with. */
export interface IMapViewHandle {
    /** Advance the map to the route where `enemyId` spawns; when `time` is given, picks the spawn closest to it. Returns false if no matching route exists. */
    focusSpawn(target: { enemyId: string; time?: number }): boolean;
}

/** Resolves an enemy's stage-effective stats for the hover tooltip. */
type StatsForFn = (id: string, enemy: IEnemy | null) => IStageEnemyStats | null;

export const MapView = forwardRef<IMapViewHandle, { level: ILevel | null; code?: string; settings?: IMapSettings; onSettingsChange?: (next: IMapSettings) => void; enemyData?: Record<string, IEnemy>; statsFor?: StatsForFn }>(function MapView(
    { level, code, settings = DEFAULT_MAP_SETTINGS, onSettingsChange, enemyData, statsFor },
    ref,
) {
    if (!level) return null;
    return <MapBoard ref={ref} code={code} enemyData={enemyData} level={level} onSettingsChange={onSettingsChange} settings={settings} statsFor={statsFor} />;
});

const MapBoard = forwardRef<IMapViewHandle, { level: ILevel; code?: string; settings: IMapSettings; onSettingsChange?: (next: IMapSettings) => void; enemyData?: Record<string, IEnemy>; statsFor?: StatsForFn }>(function MapBoard({ level, code, settings, onSettingsChange, enemyData, statsFor }, ref) {
    const width = level.mapData.map[0].length;
    const height = level.mapData.map.length;

    const mapObject = useMemo(() => new GameMap({ tiles: level.mapData.tiles, width, height }, { routes: level.routes, predefines: level.predefines }), [level, width, height]);
    const routes = useMemo(() => buildRouteEntries(level, mapObject), [level, mapObject]);
    const operators = useMemo(() => {
        const map = new Map<number, IMapOperator>();
        for (const token of level.predefines?.tokenInsts ?? []) {
            const charKey = token.inst?.characterKey;
            if (token.hidden || !charKey || charKey.startsWith("trap")) continue;
            map.set(token.position.row * width + token.position.col, { is_token: true, char_key: charKey });
        }
        return map;
    }, [level, width]);

    const maxIndex = routes.entries.length - 1;
    const [focus, setFocus] = useState(-1);
    const [is3D, setIs3D] = useState(true);

    // Per-enemy travel speed (px/sec), derived from the enemy's move-speed stat and the stage multiplier.
    const speedByKey = useMemo(() => {
        const mult = level.options?.moveMultiplier ?? 1;
        const map = new Map<string, number>();
        for (const [id, e] of Object.entries(enemyData ?? {})) {
            const ms = e.stats?.levels?.[0]?.attributes?.moveSpeed;
            if (ms != null) map.set(id, Math.max(10, ms * mult * SPEED_SCALE));
        }
        return map;
    }, [enemyData, level]);
    const speedFor = useCallback((key: string | null) => (key && speedByKey.get(key)) || DEFAULT_MOVE_SPEED * (level.options?.moveMultiplier ?? 1) * SPEED_SCALE, [speedByKey, level]);

    const chibiActive = is3D && settings.walkingChibis;
    const { data: chibiData } = useQuery({ ...enemyChibisQueryOptions(), enabled: chibiActive });
    const chibiMap = useMemo(() => {
        const map = new Map<string, IChibiCharacter>();
        for (const c of chibiData?.characters ?? []) map.set(c.operatorCode, c);
        return map;
    }, [chibiData]);

    const walkers = useMemo<IChibiWalker[]>(() => {
        if (!chibiActive || focus < 0) return [];
        const focused = routes.entries[focus];
        if (!focused) return [];
        const lo = focused.timestamp - GROUP_WINDOW;
        const out: IChibiWalker[] = [];
        for (const e of routes.entries) {
            if (e.timestamp < lo || e.timestamp > focused.timestamp || !e.enemyKey) continue;
            const character = chibiMap.get(e.enemyKey);
            if (!character) continue;
            const files = getChibiSkinData(character, "default", "front");
            if (!files?.skel || !files.atlas) continue;
            const waits = e.waits.filter((w) => w.time > 0).map((w) => ({ dist: w.dist, wait: w.time * WAIT_TIME_SCALE }));
            out.push({ position: e.position, enemyKey: e.enemyKey, d: e.d, skel: files.skel, atlas: files.atlas, speed: speedFor(e.enemyKey), waits });
        }
        return out.slice(0, 8);
    }, [chibiActive, focus, routes, chibiMap, speedFor]);

    // Enemy keys actually on the board this render (walking chibis in 3D, active icon badges in 2D).
    // The hover tooltip is gated on this so it vanishes in the SAME frame a chibi spawns out - no
    // lingering/flashing tooltip during the wave transition.
    const hoverableKeys = useMemo(() => {
        const keys = new Set<string>();
        if (chibiActive) {
            for (const w of walkers) keys.add(w.enemyKey);
        } else if (settings.showEnemyIcons && focus >= 0) {
            const focusTs = routes.entries[focus]?.timestamp;
            if (focusTs != null) {
                for (const e of routes.entries) {
                    if (e.enemyKey && e.timestamp >= focusTs - GROUP_WINDOW && e.timestamp <= focusTs) keys.add(e.enemyKey);
                }
            }
        }
        return keys;
    }, [chibiActive, walkers, settings.showEnemyIcons, focus, routes]);

    const mapRef = useRef<HTMLDivElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const [scale, setScale] = useState(1);

    const naturalWidth = width * TILE_SIZE + (width - 1) * SPACING;
    const naturalHeight = height * TILE_SIZE + (height - 1) * SPACING + BOARD_PADDING_Y * 2;

    useEffect(() => {
        const el = containerRef.current;
        if (!el) return;
        const update = () => {
            const widthScale = (el.clientWidth - H_GUTTER) / naturalWidth;
            const viewportH = typeof window === "undefined" ? Infinity : window.innerHeight;
            const heightScale = (viewportH * MAX_VIEWPORT_HEIGHT_FRACTION) / naturalHeight;
            setScale(Math.min(1, Math.max(0.1, Math.min(widthScale, heightScale))));
        };
        update();
        const ro = new ResizeObserver(update);
        ro.observe(el);
        // ResizeObserver only fires on element-size changes; a pure viewport-height
        // change (e.g. mobile orientation flip) needs the window listener too.
        window.addEventListener("resize", update);
        return () => {
            ro.disconnect();
            window.removeEventListener("resize", update);
        };
    }, [naturalWidth, naturalHeight]);

    const step = useCallback((delta: number) => setFocus((f) => Math.max(-1, Math.min(maxIndex, f + delta))), [maxIndex]);
    const prev = useCallback(() => step(-1), [step]);
    const next = useCallback(() => step(1), [step]);

    useImperativeHandle(
        ref,
        () => ({
            focusSpawn({ enemyId, time }) {
                const candidates = routes.entries.filter((e) => e.enemyKey === enemyId);
                if (candidates.length === 0) return false;
                const target = time == null ? candidates[0] : candidates.reduce((best, e) => (Math.abs(e.timestamp - time) < Math.abs(best.timestamp - time) ? e : best));
                setFocus(target.position);
                const node = containerRef.current;
                if (node) {
                    const rect = node.getBoundingClientRect();
                    const viewportH = window.innerHeight;
                    const fullyVisible = rect.top >= 0 && rect.bottom <= viewportH;
                    if (!fullyVisible) node.scrollIntoView({ behavior: "smooth", block: rect.height > viewportH ? "start" : "center" });
                }
                return true;
            },
        }),
        [routes],
    );

    // Cursor-following tooltip for whichever enemy chibi/icon is hovered on the board.
    // Position is driven imperatively (per pointer move) so movement never re-renders the map;
    // only switching which enemy is hovered updates React state.
    const [hoverKey, setHoverKey] = useState<string | null>(null);
    const hoverPanelRef = useRef<HTMLDivElement>(null);
    const hoverKeyRef = useRef<string | null>(null);
    const hoverPendingRef = useRef<string | null>(null);
    const hoverTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const hoverPosRef = useRef<{ x: number; y: number }>({ x: 0, y: 0 });

    const clearHover = useCallback(() => {
        if (hoverTimerRef.current) clearTimeout(hoverTimerRef.current);
        hoverTimerRef.current = null;
        hoverPendingRef.current = null;
        if (hoverKeyRef.current !== null) {
            hoverKeyRef.current = null;
            setHoverKey(null);
        }
    }, []);

    const positionHoverPanel = useCallback((x: number, y: number) => {
        const el = hoverPanelRef.current;
        if (!el) return;
        const pad = 16;
        const w = el.offsetWidth || 248;
        const h = el.offsetHeight || 160;
        let left = x + pad;
        let top = y + pad;
        if (left + w > window.innerWidth - 8) left = x - pad - w;
        if (left < 8) left = 8;
        if (top + h > window.innerHeight - 8) top = window.innerHeight - h - 8;
        if (top < 8) top = 8;
        el.style.left = `${left}px`;
        el.style.top = `${top}px`;
    }, []);

    // Chibis are large, constantly-moving hit targets: as they walk/spawn/despawn under a *still*
    // cursor the browser keeps firing pointer events, which would flash the tooltip on every pass.
    // So we only open from genuine cursor movement onto an enemy - when the chibi moves under a
    // stationary cursor (same coords as the last event), we ignore it.
    const handleEnemyHover = useCallback<EnemyHoverFn>(
        (key, x, y, pointerMoved) => {
            if (!key) {
                clearHover();
                return;
            }
            hoverPosRef.current = { x, y };
            if (hoverKeyRef.current === key) {
                positionHoverPanel(x, y);
                return;
            }
            if (hoverPendingRef.current === key) return; // timer already counting down for this enemy
            if (!pointerMoved) return; // enemy moved under a stationary cursor - not a real hover
            if (hoverTimerRef.current) clearTimeout(hoverTimerRef.current);
            hoverPendingRef.current = key;
            hoverTimerRef.current = setTimeout(() => {
                hoverPendingRef.current = null;
                hoverKeyRef.current = key;
                setHoverKey(key);
            }, HOVER_OPEN_DELAY);
        },
        [positionHoverPanel, clearHover],
    );

    useEffect(() => () => void (hoverTimerRef.current && clearTimeout(hoverTimerRef.current)), []);

    // Stepping/scrolling to another spawn group swaps the chibis out; a hovered chibi can unmount
    // without ever firing pointerleave, which would leave a stale tooltip lingering (and flashing as
    // you keep scrolling). Drop any open/pending hover whenever the focused group changes.
    // biome-ignore lint/correctness/useExhaustiveDependencies: intentionally keyed on focus only - this resets hover state on navigation.
    useEffect(() => {
        if (hoverTimerRef.current) clearTimeout(hoverTimerRef.current);
        hoverTimerRef.current = null;
        hoverPendingRef.current = null;
        hoverKeyRef.current = null;
        setHoverKey(null);
    }, [focus]);

    const toggle3D = useCallback(() => {
        const next3D = !is3D;
        setIs3D(next3D);
        onSettingsChange?.({ ...settings, walkingChibis: next3D, showEnemyIcons: !next3D, showMovement: !next3D });
    }, [is3D, onSettingsChange, settings]);

    useEffect(() => {
        const el = mapRef.current;
        if (!el) return;
        let accum = 0;
        const onWheel = (e: WheelEvent) => {
            e.preventDefault();
            accum += e.deltaY;
            if (Math.abs(accum) <= 50) return;
            accum = 0;
            if (e.deltaY < 0) prev();
            else next();
        };
        el.addEventListener("wheel", onWheel, { passive: false });
        return () => el.removeEventListener("wheel", onWheel);
    }, [prev, next]);

    useEffect(() => {
        const onKey = (e: KeyboardEvent) => {
            if (e.key === "ArrowLeft") prev();
            else if (e.key === "ArrowRight") next();
        };
        window.addEventListener("keydown", onKey);
        return () => window.removeEventListener("keydown", onKey);
    }, [prev, next]);

    return (
        <div className="flex h-full min-h-full w-full items-center justify-center">
            <div
                ref={(node) => {
                    mapRef.current = node;
                    containerRef.current = node;
                }}
                className="relative w-full overflow-hidden bg-[#181818] bg-[linear-gradient(90deg,#0a0a0a_1.5px,transparent_1%),linear-gradient(#0a0a0a_1.5px,transparent_1%)] bg-position-[50%] bg-size-[2.5px_2.5px] before:pointer-events-none before:absolute before:top-0 before:left-0 before:z-999 before:h-full before:w-full before:bg-[linear-gradient(180deg,hsla(0,0%,50%,0.1),transparent_60%)] before:content-['']"
            >
                <MapControls atEnd={focus >= maxIndex} atStart={focus <= -1} hasRoutes={maxIndex >= 0} is3D={is3D} onNext={next} onPrev={prev} onToggle3D={toggle3D} />
                <div className="relative mx-auto" style={{ width: naturalWidth * scale, height: naturalHeight * scale }}>
                    <div className="perspective-[900px] perspective-origin-[50%] transform-3d absolute top-0 left-0" style={{ width: naturalWidth, height: naturalHeight, transform: `scale(${scale})`, transformOrigin: "top left" }}>
                        <Board coordOverride={null} hovered={is3D} level={level} operators={operators} />
                        <RoutesLayer focus={focus} hovered={is3D} onEnemyHover={handleEnemyHover} routes={routes} settings={settings} speedFor={speedFor} />
                        {chibiActive && walkers.length > 0 && (
                            <div className="transform-3d pointer-events-none absolute top-0 left-0 rotate-x-30 transition-transform duration-400 ease-[ease]" style={{ width: naturalWidth, height: naturalHeight }}>
                                <DynamicChibiLayer onEnemyHover={handleEnemyHover} padY={BOARD_PADDING_Y} walkers={walkers} width={naturalWidth} height={naturalHeight} />
                            </div>
                        )}
                    </div>
                </div>
                <Watermark text={code ?? "Map"} />
            </div>
            {hoverKey &&
                hoverableKeys.has(hoverKey) &&
                typeof document !== "undefined" &&
                createPortal(
                    <div ref={hoverPanelRef} className="pointer-events-none fixed z-1000 rounded-md border bg-popover px-3 py-2.5 text-popover-foreground shadow-md" style={{ left: hoverPosRef.current.x + 16, top: hoverPosRef.current.y + 16, animation: "mapTooltipIn 110ms ease-out" }}>
                        <EnemyStatsCard enemy={enemyData?.[hoverKey] ?? null} id={hoverKey} stats={statsFor?.(hoverKey, enemyData?.[hoverKey] ?? null) ?? null} />
                    </div>,
                    document.body,
                )}
        </div>
    );
});
