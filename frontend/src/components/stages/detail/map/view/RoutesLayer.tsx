import { enemyIconURL } from "#/lib/api/enemies";
import type { IMapSettings } from "../../impl/MapSettings";
import type { RouteEntries, RouteEntry } from "../impl/route/route-entries";
import type { EnemyHoverFn } from "../impl/types";
import { cx } from "../impl/util/cx";

const WINDOW_BACK = 10;
const WINDOW_FWD = 20;
const GROUP_PREV = -10;
const GROUP_NEXT = 0;

function IconBadge({ enemyKey, ring }: { enemyKey: string; ring: string }) {
    return (
        <>
            <circle r={18} style={{ fill: "rgba(10,11,14,0.92)", stroke: ring, strokeWidth: 2 }} />
            <image href={enemyIconURL(enemyKey)} x={-16} y={-16} width={32} height={32} preserveAspectRatio="xMidYMid meet" style={{ stroke: "none" }} />
        </>
    );
}

function RouteEl({ entry, active, inWindow, dims, settings, is3D, speed, onEnemyHover }: { entry: RouteEntry; active: boolean; inWindow: boolean; dims: { width: number; height: number }; settings: IMapSettings; is3D: boolean; speed: number; onEnemyHover?: EnemyHoverFn }) {
    const hoverProps = entry.enemyKey
        ? {
              style: { pointerEvents: "auto" as const, cursor: "help" },
              onPointerEnter: (e: React.PointerEvent) => onEnemyHover?.(entry.enemyKey, e.clientX, e.clientY, e.movementX !== 0 || e.movementY !== 0),
              onPointerMove: (e: React.PointerEvent) => onEnemyHover?.(entry.enemyKey, e.clientX, e.clientY, e.movementX !== 0 || e.movementY !== 0),
              onPointerLeave: () => onEnemyHover?.(null, 0, 0, false),
          }
        : {};
    const vb = `0 0 ${dims.width} ${dims.height}`;
    const dur = Math.round(20 * entry.length ** 0.7);
    const pathId = `enemy-route-${entry.position}`;
    const travelMs = Math.max(800, Math.round((entry.length / speed) * 1000));
    const ring = entry.isAir ? "rgba(240,214,96,0.95)" : "rgba(255,108,108,0.95)";
    const pathStyle: React.CSSProperties = {
        strokeDasharray: active ? `${entry.length} 0` : `0 ${entry.length}`,
        strokeDashoffset: 0,
        transition: `stroke-dasharray ${dur}ms ease-in-out`,
        stroke: settings.showRoutes ? undefined : "transparent",
    };
    const display = inWindow ? undefined : "none";
    const markOpacity = (extra?: React.CSSProperties): React.CSSProperties => ({ opacity: active ? 1 : 0, transition: "opacity 200ms", ...extra });

    const showIcon = active && settings.showEnemyIcons && !!entry.enemyKey && !(is3D && settings.walkingChibis);

    return (
        <>
            <div className={cx("route", active ? "route_active" : "route_inactive", entry.isAir && "route_air")} style={{ display }}>
                <svg width={dims.width} height={dims.height} viewBox={vb}>
                    <title>enemy route</title>
                    <path id={pathId} d={entry.d} style={pathStyle} />
                    {/* biome-ignore lint/suspicious/noArrayIndexKey: static route geometry (never reordered); position alone collides when a path revisits a tile */}
                    {settings.showRoutes && entry.dots.map((p, i) => <circle key={`dot-${i}-${p.x}-${p.y}`} cx={p.x} cy={p.y} r={3.5} style={markOpacity()} />)}
                    {settings.showTimers &&
                        entry.waits.map((p) => (
                            <g key={`wait-${p.x}-${p.y}`} style={markOpacity()}>
                                <circle cx={p.x} cy={p.y} r={15} style={{ fill: "rgba(16,17,22,0.94)", stroke: "none" }} />
                                <circle cx={p.x} cy={p.y} r={15} style={{ fill: "none", stroke: ring, strokeWidth: 2 }} />
                                <text x={p.x} y={p.y} fontSize="13" fontWeight={700} textAnchor="middle" dominantBaseline="central" style={{ fill: "#fff", stroke: "none" }}>
                                    {p.time}s
                                </text>
                            </g>
                        ))}
                    {showIcon && entry.enemyKey && settings.showMovement && (
                        <g className="route_traveler" {...hoverProps}>
                            <IconBadge enemyKey={entry.enemyKey} ring={ring} />
                            <animateMotion dur={`${travelMs}ms`} repeatCount="indefinite" calcMode="linear" rotate="0">
                                <mpath href={`#${pathId}`} />
                            </animateMotion>
                        </g>
                    )}
                    {showIcon && entry.enemyKey && !settings.showMovement && entry.start && (
                        <g transform={`translate(${entry.start.x} ${entry.start.y})`} {...hoverProps}>
                            <IconBadge enemyKey={entry.enemyKey} ring={ring} />
                        </g>
                    )}
                </svg>
            </div>
            <div className={cx("route_shadow", active ? "route_active" : "route_inactive", entry.isAir && "route_air_shadow")} style={{ display }}>
                <svg width={dims.width} height={dims.height} viewBox={vb}>
                    <title>enemy route shadow</title>
                    <path d={entry.d} style={pathStyle} />
                </svg>
            </div>
        </>
    );
}

export function RoutesLayer({ routes, focus, hovered, settings, speedFor, onEnemyHover }: { routes: RouteEntries; focus: number; hovered?: boolean; settings: IMapSettings; speedFor: (key: string | null) => number; onEnemyHover?: EnemyHoverFn }) {
    const { entries, total, dims } = routes;
    const focusTs = focus >= 0 && entries[focus] ? entries[focus].timestamp : null;

    return (
        <div className="transform-3d perspective-[inherit] pointer-events-none absolute inset-0 flex content-center items-center justify-center overflow-hidden">
            <span className="absolute top-4 left-6 w-auto" />
            <span
                className={cx(
                    "absolute top-3 left-1/2 -ml-20 flex h-7 w-40 content-center justify-center rounded-xs border border-[hsla(0,0%,60%,0.8)] bg-[hsla(0,0%,12%,0.82)] py-0.5 font-medium font-mono text-[12px] text-[hsla(0,0%,92%,1)] tabular-nums transition-opacity duration-400 ease-[ease]",
                    focus < 0 ? "opacity-0" : "opacity-100",
                )}
            >
                {focus >= 0 && entries[focus] ? `${entries[focus].enemyIndexRange} / ${total}` : ""}
            </span>
            <div className={cx("routes", hovered && "map-hovered")}>
                {entries.map((entry) => {
                    const inWindow = focus >= entry.position - WINDOW_BACK && focus <= entry.position + WINDOW_FWD;
                    const active = focusTs !== null && entry.timestamp >= focusTs + GROUP_PREV && entry.timestamp <= focusTs + GROUP_NEXT;
                    return <RouteEl key={entry.position} entry={entry} active={active} inWindow={inWindow} dims={dims} settings={settings} is3D={!!hovered} speed={speedFor(entry.enemyKey)} onEnemyHover={onEnemyHover} />;
                })}
            </div>
        </div>
    );
}
