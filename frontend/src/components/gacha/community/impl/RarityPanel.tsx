import { Kicker } from "#/components/ui/kicker";
import type { IGachaEnhancedStats } from "#/lib/api/gacha";
import { formatNumber, formatNumberCompact } from "#/lib/utils";
import { fmtPct } from "./format";

interface IRarityPanelProps {
    data: IGachaEnhancedStats | null;
}

const RARITY_ROWS = [
    { rarity: 6, key: "totalSixStars" as const, color: "oklch(0.85 0.18 80)", starColor: "#f7a452" },
    { rarity: 5, key: "totalFiveStars" as const, color: "#f7e79e", starColor: "#f7e79e" },
    { rarity: 4, key: "totalFourStars" as const, color: "#bcabdb", starColor: "#bcabdb" },
    { rarity: 3, key: "totalThreeStars" as const, color: "#88c8e3", starColor: "#88c8e3" },
];

export function RarityPanel({ data }: IRarityPanelProps) {
    const cs = data?.collectiveStats;
    const total = cs?.totalPulls ?? 0;

    return (
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-[22px_24px]">
            <header className="flex items-start justify-between gap-4">
                <div>
                    <Kicker className="mb-1.5">{cs ? `Outcome mix · ${formatNumberCompact(total)} pulls` : "Outcome mix"}</Kicker>
                    <h2 className="m-0 font-sans text-[22px] font-semibold leading-[1.15] tracking-[-0.02em] text-foreground text-balance">Where every pull lands.</h2>
                </div>
            </header>

            <div className="grid items-center gap-5.5 grid-cols-[auto_1fr] max-[520px]:grid-cols-1 max-[520px]:justify-items-center">
                <Donut value={cs?.totalSixStars ?? 0} total={total || 1} label="6★" color="oklch(0.85 0.18 80)" size={140} />
                <div className="flex min-w-0 flex-col gap-3">
                    {RARITY_ROWS.map((row) => {
                        const count = cs ? cs[row.key] : 0;
                        const frac = total > 0 ? count / total : 0;
                        return (
                            <div key={row.rarity} className="flex flex-col gap-1">
                                <div className="flex items-baseline justify-between gap-2 font-mono text-xs uppercase tracking-[0.08em] text-muted-foreground">
                                    <span className="inline-flex items-center gap-1.5 text-foreground">
                                        <span style={{ color: row.starColor }} className="text-lg tracking-wider">
                                            {"★".repeat(row.rarity)}
                                        </span>
                                    </span>
                                    <span className="font-semibold tabular-nums">{cs ? fmtPct(frac, 2) : "-"}</span>
                                </div>
                                <StackBar fillPct={frac * 100} fillColor={row.color} />
                                <div className="mt-0.5 font-mono text-[9.5px] uppercase tracking-widest text-muted-foreground">{cs ? `${formatNumber(count)} pulls` : "-"}</div>
                            </div>
                        );
                    })}
                </div>
            </div>
        </section>
    );
}

function StackBar({ fillPct, fillColor }: { fillPct: number; fillColor: string }) {
    return (
        <div className="flex h-2 w-full overflow-hidden rounded-full bg-muted">
            <div className="h-full" style={{ width: `${fillPct}%`, background: fillColor }} />
        </div>
    );
}

function Donut({ value, total, label, color, size }: { value: number; total: number; label: string; color: string; size: number }) {
    const r = 42;
    const c = 2 * Math.PI * r;
    const pct = total > 0 ? value / total : 0;
    return (
        <div className="relative" style={{ width: size, height: size }}>
            <svg width={size} height={size} viewBox="0 0 100 100" role="presentation" aria-hidden="true">
                <circle cx="50" cy="50" r={r} stroke="oklch(0.88 0.005 285)" strokeWidth="9" fill="none" className="dark:stroke-[oklch(0.28_0.005_285)]" />
                <circle cx="50" cy="50" r={r} stroke={color} strokeWidth="9" fill="none" strokeDasharray={`${pct * c} ${c}`} strokeLinecap="round" transform="rotate(-90 50 50)" />
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center gap-0.5">
                <div className="font-sans text-[26px] font-bold tracking-[-0.03em] tabular-nums text-foreground">
                    {(pct * 100).toFixed(2)}
                    <span className="ml-px font-mono text-xs text-muted-foreground">%</span>
                </div>
                <div className="font-mono text-[9.5px] uppercase tracking-[0.14em] text-muted-foreground">{label}</div>
            </div>
        </div>
    );
}
