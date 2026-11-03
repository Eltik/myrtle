import type { IEnemyDamageType, IEnemyLevel } from "#/lib/api/enemies";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { ENEMY_LEVEL_LABEL_KEY } from "../constants";
import type { messages as listConstantsMessages } from "../constants.messages";
import { DAMAGE_TOKENS, LEVEL_TOKENS } from "../tokens";

interface ILevelBadgeProps {
    level: IEnemyLevel;
    size?: "sm" | "default";
}

export function LevelBadge({ level, size = "default" }: ILevelBadgeProps) {
    const t: TypedT<typeof listConstantsMessages> = useT("enemies");
    const tok = LEVEL_TOKENS[level];
    if (!tok || level === "NORMAL") return null;
    const sizeClass = size === "sm" ? "px-1.25 py-0.5 text-[10px]" : "px-1.75 py-0.75 text-[10px]";
    return (
        <span
            className={`inline-flex shrink-0 items-center whitespace-nowrap rounded-sm border font-mono font-semibold uppercase leading-none tracking-[0.14em] ${sizeClass}`}
            style={{
                background: tok.badgeBg,
                color: tok.badgeFg,
                borderColor: tok.badgeBorder,
            }}
        >
            {t(ENEMY_LEVEL_LABEL_KEY[level])}
        </span>
    );
}

interface IDamageDotsProps {
    types: IEnemyDamageType[];
    size?: number;
    /** True where the damage types are already written out beside the dots, so the dots stay out of the accessible name. */
    decorative?: boolean;
}

export function DamageDots({ types, size = 6, decorative = false }: IDamageDotsProps) {
    const t: TypedT<typeof listConstantsMessages> = useT("enemies");
    if (!types || types.length === 0) return null;
    return (
        <span className="inline-flex items-center gap-0.75" aria-hidden={decorative || undefined}>
            {types.map((dt) => {
                const tok = DAMAGE_TOKENS[dt];
                const label = tok ? t(tok.labelKey) : dt;
                return (
                    <span key={dt} title={label} className="inline-flex">
                        <span
                            aria-hidden="true"
                            className="rounded-[1.5px]"
                            style={{
                                width: size,
                                height: size,
                                background: tok?.color ?? "var(--dmg-none)",
                            }}
                        />
                        {!decorative && <span className="sr-only">{label}</span>}
                    </span>
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
        <div className="grid items-center gap-1 sm:gap-1.5" style={{ gridTemplateColumns: "24px 1fr 32px" }}>
            <span className="kicker leading-none">{label}</span>
            <div className="relative h-1 overflow-hidden rounded-full bg-[color-mix(in_oklch,var(--muted-foreground)_14%,transparent)]">
                <div className="absolute top-0 bottom-0 left-0 rounded-full" style={{ width: `${pct}%`, background: color }} />
            </div>
            <span className="text-right font-medium font-mono text-[10px] text-foreground tabular-nums leading-none sm:text-[10.5px]">{formatStatValue(value)}</span>
        </div>
    );
}
