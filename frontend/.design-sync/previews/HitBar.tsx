import { HitBar } from "frontend";

const cacheKeys = [
    { key: "operators:index", hit: 99 },
    { key: "tier-lists:browse", hit: 96 },
    { key: "user:roster:*", hit: 88 },
    { key: "gacha:banners:cn", hit: 71 },
];

export function CacheHitRates() {
    return (
        <div className="flex max-w-md flex-col gap-2.5">
            {cacheKeys.map((r) => (
                <div key={r.key} className="flex items-center justify-between gap-4">
                    <span className="font-mono text-[12.5px]">{r.key}</span>
                    <HitBar value={r.hit} />
                </div>
            ))}
        </div>
    );
}

export function InCacheTable() {
    return (
        <div className="relative max-w-xl overflow-hidden rounded-2xl border border-border bg-card">
            <table className="w-full border-collapse text-[13px]">
                <thead>
                    <tr>
                        {["Cache key", "Entries", "Hit rate"].map((h) => (
                            <th key={h} className="border-border border-b px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                {h}
                            </th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {[
                        { key: "operators:index", entries: "1", hit: 99 },
                        { key: "stages:zone:main_08", entries: "26", hit: 94 },
                        { key: "user:roster:*", entries: "12,908", hit: 88 },
                        { key: "gacha:banners:cn", entries: "184", hit: 71 },
                    ].map((r) => (
                        <tr key={r.key} className="border-border border-b last:border-0">
                            <td className="px-3.5 py-2.5 font-mono">{r.key}</td>
                            <td className="px-3.5 py-2.5 tabular-nums">{r.entries}</td>
                            <td className="px-3.5 py-2.5">
                                <HitBar value={r.hit} width={90} />
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}

export function WidthVariants() {
    return (
        <div className="flex flex-col gap-3">
            <div className="flex items-center gap-3">
                <span className="w-24 font-mono text-[11px] text-muted-foreground uppercase tracking-widest">compact</span>
                <HitBar value={96} width={40} />
            </div>
            <div className="flex items-center gap-3">
                <span className="w-24 font-mono text-[11px] text-muted-foreground uppercase tracking-widest">default</span>
                <HitBar value={87} />
            </div>
            <div className="flex items-center gap-3">
                <span className="w-24 font-mono text-[11px] text-muted-foreground uppercase tracking-widest">wide</span>
                <HitBar value={62} width={160} />
            </div>
        </div>
    );
}
