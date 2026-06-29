import { useQuery } from "@tanstack/react-query";
import { Box, ChevronLeft, ChevronRight } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getChibiSkinData } from "#/components/operators/detail/impl/components/chibi/helpers";
import { Button } from "#/components/ui/button";
import { enemyChibisQueryOptions, type IChibiCharacter } from "#/lib/api/chibis";
import type { IEnemy } from "#/lib/api/enemies";
import { DEFAULT_MAP_SETTINGS, type IMapSettings } from "../impl/MapSettings";
import { GameMap } from "./impl/map-model";
import { buildRouteEntries } from "./impl/route/route-entries";
import type { ILevel, IMapOperator } from "./impl/types";
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
/** Screen px per (tile/second) of enemy move speed — tuned for a lively-but-readable pace. */
const SPEED_SCALE = 100;
/** Fallback move speed (tiles/sec) when an enemy's stats are missing. */
const DEFAULT_MOVE_SPEED = 0.72;
/** Board px per map tile (tile + gap). */
const PX_PER_TILE = TILE_SIZE + SPACING;
/** Walk is sped up vs. real time by SPEED_SCALE/PX_PER_TILE; scale wait durations to match. */
const WAIT_TIME_SCALE = PX_PER_TILE / SPEED_SCALE;

const BOARD_PADDING_Y = 48;
const H_GUTTER = 24;
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

export function MapView({ level, code, settings = DEFAULT_MAP_SETTINGS, onSettingsChange, enemyData }: { level: ILevel | null; code?: string; settings?: IMapSettings; onSettingsChange?: (next: IMapSettings) => void; enemyData?: Record<string, IEnemy> }) {
    if (!level) return null;
    return <MapBoard code={code} enemyData={enemyData} level={level} onSettingsChange={onSettingsChange} settings={settings} />;
}

function MapBoard({ level, code, settings, onSettingsChange, enemyData }: { level: ILevel; code?: string; settings: IMapSettings; onSettingsChange?: (next: IMapSettings) => void; enemyData?: Record<string, IEnemy> }) {
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

    const toggle3D = useCallback(() => {
        setIs3D((v) => {
            const next3D = !v;
            onSettingsChange?.({ ...settings, walkingChibis: next3D, showEnemyIcons: !next3D, showMovement: !next3D });
            return next3D;
        });
    }, [onSettingsChange, settings]);

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
                        <RoutesLayer focus={focus} hovered={is3D} routes={routes} settings={settings} speedFor={speedFor} />
                        {chibiActive && walkers.length > 0 && (
                            <div className="transform-3d pointer-events-none absolute top-0 left-0 rotate-x-30 transition-transform duration-400 ease-[ease]" style={{ width: naturalWidth, height: naturalHeight }}>
                                <DynamicChibiLayer padY={BOARD_PADDING_Y} walkers={walkers} width={naturalWidth} height={naturalHeight} />
                            </div>
                        )}
                    </div>
                </div>
                <Watermark text={code ?? "Map"} />
            </div>
        </div>
    );
}
