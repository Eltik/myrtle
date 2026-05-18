import type { IEnemyDamageType, IEnemyLevel } from "#/lib/api/enemies";
import { ENEMY_LEVEL_DISPLAY } from "../constants";
import { DAMAGE_TOKENS, LEVEL_TOKENS } from "../tokens";

interface ILevelBadgeProps {
    level: IEnemyLevel;
    size?: "sm" | "default";
}

export function LevelBadge({ level, size = "default" }: ILevelBadgeProps) {
    const tok = LEVEL_TOKENS[level];
    if (!tok || level === "NORMAL") return null;
    const sizeClass = size === "sm" ? "px-1.25 py-0.5 text-[8.5px]" : "px-1.75 py-0.75 text-[9.5px]";
    return (
        <span
            className={`inline-flex shrink-0 items-center whitespace-nowrap rounded-sm border font-mono font-semibold uppercase leading-none tracking-[0.14em] ${sizeClass}`}
            style={{
                background: tok.badgeBg,
                color: tok.badgeFg,
                borderColor: tok.badgeBorder,
            }}
        >
            {ENEMY_LEVEL_DISPLAY[level]}
        </span>
    );
}

interface IDamageDotsProps {
    types: IEnemyDamageType[];
    size?: number;
}

export function DamageDots({ types, size = 6 }: IDamageDotsProps) {
    if (!types || types.length === 0) return null;
    return (
        <span className="inline-flex items-center gap-0.75">
            {types.map((t) => {
                const tok = DAMAGE_TOKENS[t];
                return (
                    <span
                        key={t}
                        title={tok?.label ?? t}
                        className="rounded-[1.5px]"
                        style={{
                            width: size,
                            height: size,
                            background: tok?.color ?? "var(--dmg-none)",
                        }}
                    />
                );
            })}
        </span>
    );
}

interface IStatBarProps {
    label: string;
    value: number;
    max: number;
    color: string;
}

function formatStatValue(value: number): string {
    if (value >= 10000) return `${(value / 1000).toFixed(0)}k`;
    if (value >= 1000) return `${(value / 1000).toFixed(1)}k`;
    return String(value);
}

export function StatBar({ label, value, max, color }: IStatBarProps) {
    const pct = max > 0 ? Math.min(100, (value / max) * 100) : 0;
    return (
        <div className="grid items-center gap-1 sm:gap-1.5" style={{ gridTemplateColumns: "20px 1fr 30px" }}>
            <span className="font-medium font-mono text-[9px] text-muted-foreground uppercase leading-none tracking-widest">{label}</span>
            <div className="relative h-0.75 overflow-hidden rounded-full bg-[color-mix(in_oklch,var(--muted-foreground)_14%,transparent)]">
                <div className="absolute top-0 bottom-0 left-0 rounded-full" style={{ width: `${pct}%`, background: color }} />
            </div>
            <span className="text-right font-medium font-mono text-[9.5px] text-foreground tabular-nums leading-none sm:text-[10.5px]">{formatStatValue(value)}</span>
        </div>
    );
}
