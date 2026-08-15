import { BarChart3 } from "lucide-react";
import { Bar, StatsKicker, StatsStatCard } from "frontend";

const CLASS_COLOR = {
    PIONEER: "oklch(0.70 0.14 200)",
    WARRIOR: "oklch(0.62 0.22 25)",
    TANK: "oklch(0.74 0.16 75)",
    SNIPER: "oklch(0.62 0.20 255)",
};

const PROFESSIONS = [
    { name: "Vanguard", owned: 24, total: 31, color: CLASS_COLOR.PIONEER },
    { name: "Guard", owned: 48, total: 71, color: CLASS_COLOR.WARRIOR },
    { name: "Defender", owned: 27, total: 38, color: CLASS_COLOR.TANK },
    { name: "Sniper", owned: 33, total: 45, color: CLASS_COLOR.SNIPER },
];

const SUB_PROFESSIONS = [
    { name: "Pioneer", owned: 6, total: 7 },
    { name: "Standard Bearer", owned: 3, total: 5 },
    { name: "Charger", owned: 4, total: 6 },
    { name: "Tactician", owned: 5, total: 6 },
    { name: "Agent", owned: 2, total: 4 },
];

export const ClassProgress = () => (
    <div className="flex w-full max-w-md flex-col gap-4">
        {PROFESSIONS.map((p) => (
            <div className="flex w-full min-w-0 flex-col gap-1.5" key={p.name}>
                <div className="flex items-center justify-between">
                    <span className="font-semibold text-sm">{p.name}</span>
                    <div className="flex items-baseline gap-1">
                        <span className="font-bold text-sm tabular-nums">{p.owned}</span>
                        <span className="font-mono text-[10.5px] text-muted-foreground/50">/ {p.total}</span>
                    </div>
                </div>
                <Bar color={p.color} pct={(p.owned / p.total) * 100} />
            </div>
        ))}
    </div>
);

export const ThinAndDimmed = () => (
    <StatsStatCard className="w-full max-w-md" color={CLASS_COLOR.PIONEER}>
        <div className="p-4 pb-2 sm:p-5 sm:pb-2">
            <StatsKicker icon={BarChart3} label="Vanguard Subclasses" />
        </div>
        <div className="flex flex-col gap-2.5 px-4 pb-4 sm:px-5 sm:pb-5">
            {SUB_PROFESSIONS.map((s) => (
                <div className="flex items-center gap-2" key={s.name}>
                    <span className="w-32 shrink-0 truncate font-mono text-[10.5px] text-muted-foreground/70">{s.name}</span>
                    <Bar color={CLASS_COLOR.PIONEER} dim pct={(s.owned / s.total) * 100} thin />
                    <span className="w-10 shrink-0 text-right font-mono text-[10.5px] text-muted-foreground/70 tabular-nums">
                        {s.owned}/{s.total}
                    </span>
                </div>
            ))}
        </div>
    </StatsStatCard>
);

export const FillLevels = () => (
    <div className="flex w-full max-w-md flex-col gap-4">
        {[
            { label: "Modules at Lv 3", pct: 0, value: "0 / 302" },
            { label: "Skins collected", pct: 21.1, value: "87 / 412" },
            { label: "Mastery levels", pct: 49.5, value: "428 / 864" },
            { label: "Operators owned", pct: 73.1, value: "231 / 316" },
            { label: "Chapter 8 cleared", pct: 100, value: "14 / 14" },
        ].map((row) => (
            <div className="space-y-2" key={row.label}>
                <div className="flex items-center justify-between">
                    <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{row.label}</span>
                    <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{row.value}</span>
                </div>
                <Bar color="var(--primary)" pct={row.pct} />
            </div>
        ))}
    </div>
);
