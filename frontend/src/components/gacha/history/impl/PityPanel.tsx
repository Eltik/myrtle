import { Kicker } from "#/components/ui/kicker";
import type { ClientGachaGroup, IClientGachaRecords, IGachaItem } from "#/lib/api/gacha";

interface IPityPanelProps {
    records: IClientGachaRecords | null;
    isLoading: boolean;
}

interface IBannerPity {
    key: ClientGachaGroup;
    label: string;
    pity: number;
    softPityAt: number;
    hardPityAt: number;
    color: string;
}

function computePity(items: IGachaItem[]): number {
    const sorted = [...items].sort((a, b) => b.at - a.at);
    let pity = 0;
    for (const item of sorted) {
        if (item.star === "6") break;
        pity++;
    }
    return pity;
}

const BANNER_CONFIGS: { key: ClientGachaGroup; label: string; softPityAt: number; hardPityAt: number; color: string }[] = [
    { key: "limited", label: "Limited", softPityAt: 50, hardPityAt: 99, color: "oklch(0.85 0.18 80)" },
    // Collab/joint operation banners: separate pity from limited, hard guarantee at 120 pulls.
    { key: "linkage", label: "Collab", softPityAt: 50, hardPityAt: 120, color: "oklch(0.78 0.16 320)" },
    { key: "regular", label: "Standard", softPityAt: 50, hardPityAt: 99, color: "#bcabdb" },
    { key: "special", label: "Kernel", softPityAt: 45, hardPityAt: 80, color: "#88c8e3" },
];

function PityMeter({ pity, softPityAt, hardPityAt, color }: { pity: number; softPityAt: number; hardPityAt: number; color: string }) {
    const pct = Math.min((pity / hardPityAt) * 100, 100);
    const softPct = (softPityAt / hardPityAt) * 100;
    const isSoftPity = pity >= softPityAt;
    const isNearHard = pity >= hardPityAt - 5;

    return (
        <div className="flex flex-col gap-1.5">
            <div className="relative h-3 w-full rounded-full bg-muted overflow-visible">
                <div className="h-full overflow-hidden rounded-full">
                    <div
                        className="h-full rounded-full transition-all duration-500"
                        style={{
                            width: `${pct}%`,
                            background: isNearHard ? "linear-gradient(90deg, oklch(0.78 0.18 25), oklch(0.65 0.22 15))" : isSoftPity ? `linear-gradient(90deg, ${color}, oklch(0.78 0.18 25))` : `linear-gradient(90deg, color-mix(in oklch, ${color} 60%, transparent), ${color})`,
                        }}
                    />
                </div>
                <div className="pointer-events-none absolute top-0 bottom-0 w-px bg-foreground/30" style={{ left: `${softPct}%` }} title={`Soft pity at ${softPityAt}`} aria-hidden />
            </div>
            <div className="flex items-center justify-between font-mono text-[10px] text-muted-foreground">
                <span>soft pity at {softPityAt}</span>
                <span>guaranteed at {hardPityAt}</span>
            </div>
        </div>
    );
}

function PityCard({ label, pity, softPityAt, hardPityAt, color, total }: { label: string; pity: number; softPityAt: number; hardPityAt: number; color: string; total: number }) {
    const isSoftPity = pity >= softPityAt;
    const isNearHard = pity >= hardPityAt - 5;
    const statusColor = isNearHard ? "text-[oklch(0.78_0.18_25)]" : isSoftPity ? "text-[oklch(0.78_0.18_80)]" : "text-foreground";

    return (
        <div className="flex flex-col gap-3 rounded-xl border border-border bg-muted/30 p-4">
            <div className="flex items-start justify-between gap-3">
                <div>
                    <div className="font-mono text-[10px] uppercase tracking-[0.16em] text-muted-foreground mb-1">{label} banner</div>
                    <div className={`font-sans text-[32px] font-bold leading-none tracking-[-0.04em] tabular-nums ${statusColor}`}>
                        {pity}
                        <span className="ml-1 font-mono text-[13px] font-medium text-muted-foreground">pulls</span>
                    </div>
                </div>
                <div className="flex flex-col items-end gap-1 text-right">
                    {isSoftPity ? (
                        <span className="inline-flex items-center gap-1 rounded-md border border-amber-500/30 bg-amber-500/10 px-2 py-0.5 font-mono text-[9.5px] uppercase tracking-[0.14em] text-amber-600 dark:text-amber-400">
                            <span className="block h-1.5 w-1.5 rounded-full bg-current" aria-hidden />
                            soft pity
                        </span>
                    ) : null}
                    {isNearHard ? (
                        <span className="inline-flex items-center gap-1 rounded-md border border-red-500/30 bg-red-500/10 px-2 py-0.5 font-mono text-[9.5px] uppercase tracking-[0.14em] text-red-600 dark:text-red-400">
                            <span className="block h-1.5 w-1.5 rounded-full bg-current" aria-hidden />
                            near guaranteed
                        </span>
                    ) : null}
                    <span className="font-mono text-[10px] text-muted-foreground tabular-nums">{total} total</span>
                </div>
            </div>
            <PityMeter pity={pity} softPityAt={softPityAt} hardPityAt={hardPityAt} color={color} />
        </div>
    );
}

export function PityPanel({ records, isLoading }: IPityPanelProps) {
    if (isLoading) {
        return (
            <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-[22px_24px]">
                <div className="h-4 w-28 rounded bg-muted animate-pulse" />
                <div className="grid gap-3 grid-cols-1 sm:grid-cols-2 lg:grid-cols-4">
                    {Array.from({ length: 4 }).map((_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                        <div key={i} className="h-32 rounded-xl bg-muted animate-pulse" />
                    ))}
                </div>
            </section>
        );
    }

    if (!records) return null;

    const pities: IBannerPity[] = BANNER_CONFIGS.map((cfg) => ({
        ...cfg,
        pity: computePity(records[cfg.key].records),
    }));

    return (
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-[22px_24px]">
            <header>
                <Kicker className="mb-1.5">Current pity</Kicker>
                <h2 className="m-0 font-sans text-[20px] font-semibold leading-[1.15] tracking-[-0.02em] text-foreground sm:text-[22px]">Pulls since last 6★.</h2>
            </header>
            <div className="grid gap-3 grid-cols-1 sm:grid-cols-2 lg:grid-cols-4">
                {pities.map((p) => (
                    <PityCard key={p.key} label={p.label} pity={p.pity} softPityAt={p.softPityAt} hardPityAt={p.hardPityAt} color={p.color} total={records[p.key].total} />
                ))}
            </div>
        </section>
    );
}
