import type { RouteEntries, RouteEntry } from "../impl/route/route-entries";
import { cx } from "../impl/util/cx";

const WINDOW_BACK = 10;
const WINDOW_FWD = 20;
const GROUP_PREV = -10;
const GROUP_NEXT = 0;

function RouteEl({ entry, active, inWindow, dims }: { entry: RouteEntry; active: boolean; inWindow: boolean; dims: { width: number; height: number } }) {
    const vb = `0 0 ${dims.width} ${dims.height}`;
    const dur = Math.round(20 * entry.length ** 0.7);
    const pathStyle: React.CSSProperties = {
        strokeDasharray: active ? `${entry.length} 0` : `0 ${entry.length}`,
        strokeDashoffset: 0,
        transition: `stroke-dasharray ${dur}ms ease-in-out`,
    };
    const display = inWindow ? undefined : "none";
    const markOpacity = (extra?: React.CSSProperties): React.CSSProperties => ({ opacity: active ? 1 : 0, transition: "opacity 200ms", ...extra });

    return (
        <>
            <div className={cx("route", active ? "route_active" : "route_inactive", entry.isAir && "route_air")} style={{ display }}>
                <svg width={dims.width} height={dims.height} viewBox={vb}>
                    <title>enemy route</title>
                    <path d={entry.d} style={pathStyle} />
                    {entry.dots.map((p) => (
                        <circle key={`dot-${p.x}-${p.y}`} cx={p.x} cy={p.y} r={3} transform={`rotate(-90 ${p.x} ${p.y})`} style={markOpacity()} />
                    ))}
                    {entry.waits.map((p) => (
                        <g key={`wait-${p.x}-${p.y}`}>
                            <circle cx={p.x} cy={p.y} r={16} transform={`rotate(-90 ${p.x} ${p.y})`} style={markOpacity({ fill: "rgba(120,120,120,1)" })} />
                            <text x={p.x} y={p.y} fontSize="14px" textAnchor="middle" dominantBaseline="central" style={markOpacity({ fill: "rgba(240,240,240,1)" })}>
                                {p.time}s
                            </text>
                        </g>
                    ))}
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

export function RoutesLayer({ routes, focus, hovered }: { routes: RouteEntries; focus: number; hovered?: boolean }) {
    const { entries, total, dims } = routes;
    const focusTs = focus >= 0 && entries[focus] ? entries[focus].timestamp : null;

    return (
        <div className="transform-3d perspective-[inherit] pointer-events-none absolute inset-0 flex content-center items-center justify-center overflow-hidden">
            <span className="absolute top-4 left-6 w-auto" />
            <span className={cx("absolute top-3 left-1/2 -ml-20 flex h-7 w-40 content-center justify-center rounded-xs border border-[hsla(0,0%,60%,0.8)] bg-[hsla(0,0%,48%,0.2)] py-0.5 transition-opacity duration-400 ease-[ease]", focus < 0 ? "opacity-0" : "opacity-100")}>
                {focus >= 0 && entries[focus] ? `${entries[focus].enemyIndexRange} / ${total}` : ""}
            </span>
            <div className={cx("routes", hovered && "map-hovered")}>
                {entries.map((entry) => {
                    const inWindow = focus >= entry.position - WINDOW_BACK && focus <= entry.position + WINDOW_FWD;
                    const active = focusTs !== null && entry.timestamp >= focusTs + GROUP_PREV && entry.timestamp <= focusTs + GROUP_NEXT;
                    return <RouteEl key={entry.position} entry={entry} active={active} inWindow={inWindow} dims={dims} />;
                })}
            </div>
        </div>
    );
}
