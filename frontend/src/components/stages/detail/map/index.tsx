import { Box, ChevronLeft, ChevronRight } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { GameMap } from "./impl/map-model";
import { buildRouteEntries } from "./impl/route/route-entries";
import type { ILevel, IMapOperator } from "./impl/types";
import { SPACING, TILE_SIZE } from "./impl/util/geometry";
import "./impl/effects.css";
import "./impl/tiles.css";
import "./impl/routes.css";
import { Board } from "./view/Board";
import { RoutesLayer } from "./view/RoutesLayer";
import { Watermark } from "./view/Watermark";

const BOARD_PADDING_Y = 48;
const H_GUTTER = 24;

function MapControls({ onPrev, onNext, atStart, atEnd, hasRoutes, is3D, onToggle3D }: { onPrev: () => void; onNext: () => void; atStart: boolean; atEnd: boolean; hasRoutes: boolean; is3D: boolean; onToggle3D: () => void }) {
    return (
        <div className="absolute top-3 right-3 z-50 flex items-center gap-1.5 sm:top-4 sm:right-4">
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

export function MapView({ level }: { level: ILevel | null }) {
    if (!level) return null;
    return <MapBoard level={level} />;
}

function MapBoard({ level }: { level: ILevel }) {
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
    const [is3D, setIs3D] = useState(false);
    const mapRef = useRef<HTMLDivElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const [scale, setScale] = useState(1);

    const naturalWidth = width * TILE_SIZE + (width - 1) * SPACING;
    const naturalHeight = height * TILE_SIZE + (height - 1) * SPACING + BOARD_PADDING_Y * 2;

    useEffect(() => {
        const el = containerRef.current;
        if (!el) return;
        const update = () => setScale(Math.min(1, Math.max(0.1, (el.clientWidth - H_GUTTER) / naturalWidth)));
        update();
        const ro = new ResizeObserver(update);
        ro.observe(el);
        return () => ro.disconnect();
    }, [naturalWidth]);

    const step = useCallback((delta: number) => setFocus((f) => Math.max(-1, Math.min(maxIndex, f + delta))), [maxIndex]);
    const prev = useCallback(() => step(-1), [step]);
    const next = useCallback(() => step(1), [step]);

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
        <div className="flex min-h-screen w-full items-center justify-center">
            <div
                ref={(node) => {
                    mapRef.current = node;
                    containerRef.current = node;
                }}
                className="relative w-full overflow-hidden bg-[#181818] bg-[linear-gradient(90deg,#0a0a0a_1.5px,transparent_1%),linear-gradient(#0a0a0a_1.5px,transparent_1%)] bg-position-[50%] bg-size-[2.5px_2.5px] before:pointer-events-none before:absolute before:top-0 before:left-0 before:z-999 before:h-full before:w-full before:bg-[linear-gradient(180deg,hsla(0,0%,50%,0.1),transparent_60%)] before:content-['']"
            >
                <MapControls atEnd={focus >= maxIndex} atStart={focus <= -1} hasRoutes={maxIndex >= 0} is3D={is3D} onNext={next} onPrev={prev} onToggle3D={() => setIs3D((v) => !v)} />
                <div className="relative mx-auto" style={{ width: naturalWidth * scale, height: naturalHeight * scale }}>
                    <div className="perspective-[900px] perspective-origin-[50%] transform-3d absolute top-0 left-0" style={{ width: naturalWidth, height: naturalHeight, transform: `scale(${scale})`, transformOrigin: "top left" }}>
                        <Board coordOverride={null} hovered={is3D} level={level} operators={operators} />
                        <RoutesLayer focus={focus} hovered={is3D} routes={routes} />
                    </div>
                </div>
                <Watermark text="Map" />
            </div>
        </div>
    );
}
