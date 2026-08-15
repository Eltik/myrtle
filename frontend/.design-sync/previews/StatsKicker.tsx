import { BarChart3, CalendarCheck, Layers, Palette, Sparkles, Trophy, Users } from "lucide-react";
import { MetricRow, StatsKicker, StatsStatCard, StatsTile } from "frontend";

const KICKER = "font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground";
const PAD = "p-4 sm:p-5";

export const SectionLabels = () => (
    <div className="flex max-w-xs flex-col gap-4 rounded-xl border border-border bg-card p-4">
        <StatsKicker icon={Users} label="Operator Collection" />
        <StatsKicker icon={Sparkles} label="Skill Mastery" />
        <StatsKicker icon={Layers} label="Modules" />
        <StatsKicker icon={Palette} label="Skins" />
        <StatsKicker icon={BarChart3} label="Class Breakdown" />
        <StatsKicker icon={Trophy} label="Top Operators" />
        <StatsKicker icon={CalendarCheck} label="Sign-in Streak" />
    </div>
);

export const InStatCard = () => (
    <StatsStatCard className="max-w-sm" color="oklch(0.66 0.14 200)">
        <div className={`flex h-full flex-col gap-5 ${PAD}`}>
            <StatsKicker icon={Layers} label="Modules" />
            <div className="grid grid-cols-2 gap-2">
                <StatsTile
                    color="oklch(0.66 0.14 200)"
                    sub="Unlocked"
                    value={
                        <span>
                            148
                            <span className="ml-1 font-medium text-muted-foreground/50 text-sm">/ 302</span>
                        </span>
                    }
                />
                <StatsTile color="oklch(0.72 0.15 175)" sub="Max Lv" value={71} />
            </div>
            <MetricRow color="oklch(0.66 0.14 200)" label="Unlock Rate" pct={49.0} value="49.0%" />
        </div>
    </StatsStatCard>
);

export const StackedSections = () => (
    <StatsStatCard className="max-w-sm" color={["oklch(0.66 0.14 200)", "var(--primary)"]}>
        <div className={`flex h-full flex-col gap-5 ${PAD}`}>
            <div className="space-y-3">
                <StatsKicker icon={Layers} label="Modules" />
                <MetricRow color="oklch(0.66 0.14 200)" label="Unlock Rate" pct={49.0} value="49.0%" />
            </div>
            <div className="border-border/60 border-t" />
            <div className="space-y-3">
                <StatsKicker icon={Palette} label="Skins" />
                <MetricRow color="var(--primary)" label="Collected" pct={21.1} value="21.1%" />
            </div>
            <div className="border-border/60 border-t" />
            <div className="space-y-3">
                <StatsKicker icon={CalendarCheck} label="Sign-in Streak" />
                <span className={KICKER}>18 days · best 64</span>
            </div>
        </div>
    </StatsStatCard>
);
