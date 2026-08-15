import { Layers, Palette, Users } from "lucide-react";
import { MetricRow, StatsKicker, StatsStatCard, StatsTile } from "frontend";

const KICKER = "font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground";
const PAD = "p-4 sm:p-5";

export const CompletionRates = () => (
    <div className="flex w-full max-w-md flex-col gap-4 rounded-xl border border-border bg-card p-4">
        <MetricRow color="var(--primary)" label="Completion" pct={73.1} value="73.10%" />
        <MetricRow color="oklch(0.66 0.14 200)" label="Unlock Rate" pct={49.0} value="49.0%" />
        <MetricRow color="oklch(0.62 0.21 295)" label="Mastery Levels" pct={49.5} value="428 / 864" />
        <MetricRow color="var(--primary)" label="Collected" pct={21.1} value="21.1%" />
        <MetricRow color="oklch(0.70 0.15 162)" label="Modules at Lv 3" pct={0} value="0 / 302" />
    </div>
);

export const InCollectionCard = () => (
    <StatsStatCard className="max-w-sm" color="var(--primary)">
        <div className={`flex h-full flex-col gap-5 ${PAD}`}>
            <StatsKicker icon={Users} label="Operator Collection" />
            <div className="flex flex-1 flex-col justify-between gap-5">
                <div className="flex flex-col items-center gap-1.5 py-1">
                    <div className="flex items-baseline gap-1.5">
                        <span className="font-bold tabular-nums leading-none" style={{ fontSize: "clamp(2.25rem, 3vw + 1rem, 3rem)", letterSpacing: "-0.03em", color: "var(--primary)" }}>
                            231
                        </span>
                        <span className="font-medium font-mono text-lg text-muted-foreground/50 tabular-nums">/ 316</span>
                    </div>
                    <span className={KICKER}>operators collected</span>
                </div>
                <MetricRow color="var(--primary)" label="Completion" pct={73.1} value="73.10%" />
            </div>
        </div>
    </StatsStatCard>
);

export const ModulesAndSkins = () => (
    <StatsStatCard className="max-w-sm" color={["oklch(0.66 0.14 200)", "var(--primary)"]}>
        <div className={`flex h-full flex-col gap-5 ${PAD}`}>
            <div className="space-y-4">
                <StatsKicker icon={Layers} label="Modules" />
                <div className="grid grid-cols-2 gap-2">
                    <StatsTile color="oklch(0.66 0.14 200)" sub="Unlocked" value={148} />
                    <StatsTile color="oklch(0.72 0.15 175)" sub="Max Lv" value={71} />
                </div>
                <MetricRow color="oklch(0.66 0.14 200)" label="Unlock Rate" pct={49.0} value="49.0%" />
            </div>
            <div className="border-border/60 border-t" />
            <div className="space-y-4">
                <StatsKicker icon={Palette} label="Skins" />
                <MetricRow color="var(--primary)" label="Collected" pct={21.1} value="21.1%" />
            </div>
        </div>
    </StatsStatCard>
);
