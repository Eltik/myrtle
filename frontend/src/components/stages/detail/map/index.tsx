import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { GameMap } from "./impl/map-model";
import { buildRouteEntries } from "./impl/route/route-entries";
import type { ILevel, IMapOperator } from "./impl/types";
import { cx } from "./impl/util/cx";
import "./impl/effects.css";
import "./impl/tiles.css";
import "./impl/routes.css";
import { Board } from "./view/Board";
import { RoutesLayer } from "./view/RoutesLayer";
import { Watermark } from "./view/Watermark";

function NavButton({ dir, onClick }: { dir: "prev" | "next"; onClick: () => void }) {
    const prev = dir === "prev";
    const bar = "pointer-events-none absolute top-1/2 left-1/2 -mt-px -ml-3 h-0.5 w-6 rounded-xs bg-white";
    const origin = prev ? "origin-left" : "origin-right";
    const up = "[box-shadow:0_-1px_2px_0_hsla(0,0%,94%,0.7)]";
    const down = "[box-shadow:0_1px_2px_0_hsla(0,0%,94%,0.7)]";
    return (
        <button
            type="button"
            aria-label={prev ? "Previous enemy" : "Next enemy"}
            onClick={onClick}
            className={cx("absolute top-4 z-8999 h-8 w-12 cursor-pointer appearance-none rounded-xs border-0 bg-transparent p-0 opacity-60 transition-colors duration-400 ease-[ease] hover:bg-[hsla(0,0%,78%,0.2)]", prev ? "right-20" : "right-8")}
        >
            <span className={cx(bar, origin, "rotate-20", prev ? up : down)} />
            <span className={cx(bar, origin, "rotate-[-20deg]", prev ? down : up)} />
        </button>
    );
}

export function MapView({ level }: { level: ILevel | null }) {
    if (!level) return null;
    return <MapBoard level={level} />;
}

function MapBoard({ level }: { level: ILevel }) {
    const width = level.mapData.map[0].length;

    const mapObject = useMemo(() => new GameMap({ tiles: level.mapData.tiles, width, height: level.mapData.map.length }, { routes: level.routes, predefines: level.predefines }), [level, width]);
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
    const [hovered, setHovered] = useState(false);
    const mapRef = useRef<HTMLDivElement>(null);

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
            {/* biome-ignore lint/a11y/noStaticElementInteractions: presentational hover-tilt only, no activation semantics */}
            <div
                ref={mapRef}
                onMouseEnter={() => setHovered(true)}
                onMouseLeave={() => setHovered(false)}
                className="perspective-[900px] perspective-origin-[50%] transform-3d relative h-auto w-full overflow-hidden bg-[#181818] bg-[linear-gradient(90deg,#0a0a0a_1.5px,transparent_1%),linear-gradient(#0a0a0a_1.5px,transparent_1%)] bg-position-[50%] bg-size-[2.5px_2.5px] before:pointer-events-none before:absolute before:top-0 before:left-0 before:z-999 before:h-full before:w-full before:bg-[linear-gradient(180deg,hsla(0,0%,50%,0.1),transparent_60%)] before:content-['']"
            >
                <NavButton dir="prev" onClick={prev} />
                <NavButton dir="next" onClick={next} />
                <Board level={level} coordOverride={null} operators={operators} hovered={hovered} />
                <RoutesLayer routes={routes} focus={focus} hovered={hovered} />
                <Watermark text="Map" />
            </div>
        </div>
    );
}
