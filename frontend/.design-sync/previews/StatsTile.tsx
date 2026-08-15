import { Layers, Sparkles } from "lucide-react";
import { MetricRow, StatsKicker, StatsStatCard, StatsTile } from "frontend";

const PAD = "p-4 sm:p-5";
const MASTERY = { accent: "oklch(0.62 0.21 295)", m3: "oklch(0.70 0.17 145)", m6: "oklch(0.65 0.18 230)", m9: "oklch(0.74 0.17 75)" };
const MODULES = { accent: "oklch(0.66 0.14 200)", unlocked: "oklch(0.66 0.14 200)", max: "oklch(0.72 0.15 175)" };

export const MasteryTiles = () => (
    <div className="grid w-full max-w-sm grid-cols-3 gap-2">
        <StatsTile color={MASTERY.m3} sub="M3" tooltip="Operators with at least one Mastery 3 skill" value={62} />
        <StatsTile color={MASTERY.m6} sub="M6" tooltip="Operators with 2 skills at Mastery 3" value={34} />
        <StatsTile color={MASTERY.m9} sub="M9" tooltip="Operators with all 3 skills at Mastery 3" value={21} />
    </div>
);

export const FractionValues = () => (
    <div className="grid w-full max-w-sm grid-cols-2 gap-2">
        <StatsTile
            color={MODULES.unlocked}
            sub="Unlocked"
            tooltip="148 of 302 available modules unlocked"
            value={
                <span>
                    148
                    <span className="ml-1 font-medium text-muted-foreground/50 text-sm">/ 302</span>
                </span>
            }
        />
        <StatsTile
            color="var(--primary)"
            sub="Skins Collected"
            tooltip="87 of 412 non-default skins collected"
            value={
                <span>
                    87
                    <span className="ml-1 font-medium text-muted-foreground/50 text-sm">/ 412</span>
                </span>
            }
        />
    </div>
);

export const ElitePromotions = () => (
    <div className="grid w-full max-w-md grid-cols-4 gap-2">
        <StatsTile color="oklch(0.74 0.17 75)" sub="E0" value={38} />
        <StatsTile color="oklch(0.74 0.17 75)" sub="E1" value={97} />
        <StatsTile color="oklch(0.74 0.17 75)" sub="E2" value={96} />
        <StatsTile color="oklch(0.74 0.17 75)" sub="Lv 90" value={12} />
    </div>
);

export const WithinStatCard = () => (
    <StatsStatCard className="max-w-sm" color={MASTERY.accent}>
        <div className={`flex h-full flex-col gap-5 ${PAD}`}>
            <StatsKicker icon={Sparkles} label="Skill Mastery" />
            <div className="grid grid-cols-3 gap-2">
                <StatsTile color={MASTERY.m3} sub="M3" value={62} />
                <StatsTile color={MASTERY.m6} sub="M6" value={34} />
                <StatsTile color={MASTERY.m9} sub="M9" value={21} />
            </div>
            <div className="border-border/60 border-t" />
            <StatsKicker icon={Layers} label="Modules" />
            <div className="grid grid-cols-2 gap-2">
                <StatsTile color={MODULES.unlocked} sub="Unlocked" value={148} />
                <StatsTile color={MODULES.max} sub="Max Lv" value={71} />
            </div>
            <MetricRow color={MODULES.accent} label="Unlock Rate" pct={49.0} value="49.0%" />
        </div>
    </StatsStatCard>
);
