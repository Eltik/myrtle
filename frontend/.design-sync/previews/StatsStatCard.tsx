import { BarChart3, Sparkles, Users } from "lucide-react";
import { Bar, MetricRow, StatsKicker, StatsStatCard, StatsTile } from "frontend";

// Ported from src/components/user/profile/impl/components/tabs/Stats — the
// shared kicker/padding constants and the profile stat palette.
const KICKER = "font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground";
const PAD = "p-4 sm:p-5";

const PALETTE = {
    collection: "var(--primary)",
    mastery: { accent: "oklch(0.62 0.21 295)", m3: "oklch(0.70 0.17 145)", m6: "oklch(0.65 0.18 230)", m9: "oklch(0.74 0.17 75)" },
    classes: ["oklch(0.70 0.14 200)", "oklch(0.62 0.22 25)", "oklch(0.74 0.16 75)", "oklch(0.62 0.20 255)", "oklch(0.62 0.22 295)", "oklch(0.70 0.16 145)", "oklch(0.78 0.16 110)", "oklch(0.65 0.22 340)"] as const,
};

const CLASSES = [
    { name: "Vanguard", owned: 24, total: 31, color: PALETTE.classes[0] },
    { name: "Guard", owned: 48, total: 71, color: PALETTE.classes[1] },
    { name: "Defender", owned: 27, total: 38, color: PALETTE.classes[2] },
    { name: "Sniper", owned: 33, total: 45, color: PALETTE.classes[3] },
];

export const CollectionSummary = () => (
    <StatsStatCard className="max-w-sm" color={PALETTE.collection}>
        <div className={`flex h-full flex-col gap-5 ${PAD}`}>
            <StatsKicker icon={Users} label="Operator Collection" />
            <div className="flex flex-1 flex-col justify-between gap-5">
                <div className="flex flex-col items-center gap-1.5 py-1">
                    <div className="flex items-baseline gap-1.5">
                        <span className="font-bold tabular-nums leading-none" style={{ fontSize: "clamp(2.25rem, 3vw + 1rem, 3rem)", letterSpacing: "-0.03em", color: PALETTE.collection }}>
                            231
                        </span>
                        <span className="font-medium font-mono text-lg text-muted-foreground/50 tabular-nums">/ 316</span>
                    </div>
                    <span className={KICKER}>operators collected</span>
                </div>
                <MetricRow color={PALETTE.collection} label="Completion" pct={73.1} value="73.10%" />
            </div>
        </div>
    </StatsStatCard>
);

export const MasteryBreakdown = () => (
    <StatsStatCard className="max-w-sm" color={PALETTE.mastery.accent}>
        <div className={`flex h-full flex-col gap-5 ${PAD}`}>
            <StatsKicker icon={Sparkles} label="Skill Mastery" />
            <div className="flex flex-1 flex-col justify-center gap-5">
                <div className="grid grid-cols-3 gap-2">
                    <StatsTile color={PALETTE.mastery.m3} sub="M3" value={62} />
                    <StatsTile color={PALETTE.mastery.m6} sub="M6" value={34} />
                    <StatsTile color={PALETTE.mastery.m9} sub="M9" value={21} />
                </div>
                <div className="space-y-2">
                    <div className="flex items-center justify-between">
                        <span className={KICKER}>Total Mastery Levels</span>
                        <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                            <span className="font-semibold text-foreground">428</span>
                            <span className="text-muted-foreground/50"> / 864</span>
                        </span>
                    </div>
                    <Bar color={PALETTE.mastery.accent} pct={49.5} />
                    <p className="text-center font-mono text-[10px] text-muted-foreground/50">49.5% of max possible (E2 only)</p>
                </div>
            </div>
        </div>
    </StatsStatCard>
);

export const MultiColorAccent = () => (
    <StatsStatCard className="max-w-xl" color={PALETTE.classes}>
        <div className={`${PAD} pb-2`}>
            <StatsKicker icon={BarChart3} label="Class Breakdown" />
        </div>
        <div className="flex flex-col gap-3 px-4 pb-4 sm:px-5 sm:pb-5">
            {CLASSES.map((c) => (
                <div className="flex w-full min-w-0 flex-col gap-1.5" key={c.name}>
                    <div className="flex items-center justify-between">
                        <span className="font-semibold text-sm">{c.name}</span>
                        <div className="flex items-baseline gap-1">
                            <span className="font-bold text-sm tabular-nums">{c.owned}</span>
                            <span className="font-mono text-[10.5px] text-muted-foreground/50">/ {c.total}</span>
                        </div>
                    </div>
                    <Bar color={c.color} pct={(c.owned / c.total) * 100} />
                </div>
            ))}
        </div>
    </StatsStatCard>
);
