import { Layers, Palette, Sparkles } from "lucide-react";
import { Bar, DialogDescription, DialogHeader, DialogPopup, DialogTitle, GapList, StatsKicker, StatsStatCard, StatsTile } from "frontend";

const KICKER = "font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground";
const PAD = "p-4 sm:p-5";
const MASTERY = { accent: "oklch(0.62 0.21 295)", m3: "oklch(0.70 0.17 145)", m6: "oklch(0.65 0.18 230)", m9: "oklch(0.74 0.17 75)" };

const PENDING_M3 = [
    { id: "m3-1", operatorId: "char_4064_mlynar", charId: "char_4064_mlynar", name: "Mlynar", rarity: 6, sub: "S3 Lv7" },
    { id: "m3-2", operatorId: "char_263_skadi", charId: "char_263_skadi", name: "Skadi", rarity: 6, sub: "S2 Lv7" },
    { id: "m3-3", operatorId: "char_180_amgoat", charId: "char_180_amgoat", name: "Eyjafjalla", rarity: 6, sub: "S3 Lv7" },
];

const PENDING_M6 = [
    { id: "m6-1", operatorId: "char_1028_texas2", charId: "char_1028_texas2", name: "Texas the Omertosa", rarity: 6, sub: "S2 M3" },
    { id: "m6-2", operatorId: "char_2023_ling", charId: "char_2023_ling", name: "Ling", rarity: 6, sub: "S3 M3" },
];

const MASTERY_GAPS = [
    { key: "levels", label: "levels", value: 436, color: MASTERY.accent, tooltip: "Skill levels remaining to reach Mastery 3 across all E2 operators" },
    { key: "pendingM3", label: "pending M3", value: 34, color: MASTERY.m3, tooltip: "Click to view E2 operators without any skill at Mastery 3 yet", details: PENDING_M3 },
    { key: "pendingM6", label: "pending M6", value: 28, color: MASTERY.m6, tooltip: "Click to view operators with 1 skill at M3 still needing a second", details: PENDING_M6 },
    { key: "pendingM9", label: "pending M9", value: 13, color: MASTERY.m9, tooltip: "Click to view operators with 2 skills at M3 still needing a third", details: PENDING_M6 },
];

export const MasteryGaps = () => (
    <div className="flex w-full max-w-md flex-col gap-3 rounded-xl border border-border bg-card p-4">
        <div className="flex items-center justify-between">
            <StatsKicker icon={Sparkles} label="Skill Mastery" />
            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                <span className="font-semibold text-foreground">428</span>
                <span className="text-muted-foreground/50"> / 864</span>
            </span>
        </div>
        <div className="border-border/60 border-t pt-3">
            <GapList items={MASTERY_GAPS} />
        </div>
    </div>
);

export const AllComplete = () => (
    <div className="flex w-full max-w-md flex-col gap-3 rounded-xl border border-border bg-card p-4">
        <div className="flex items-center justify-between">
            <StatsKicker icon={Layers} label="Modules" />
            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                <span className="font-semibold text-foreground">302</span>
                <span className="text-muted-foreground/50"> / 302</span>
            </span>
        </div>
        <div className="border-border/60 border-t pt-3">
            <GapList
                items={[
                    { key: "locked", label: "locked", value: 0, color: "oklch(0.66 0.14 200)" },
                    { key: "belowMax", label: "below max", value: 0, color: "oklch(0.72 0.15 175)" },
                ]}
            />
        </div>
    </div>
);

export const DrilldownPill = () => (
    <div className="flex w-full max-w-md flex-col gap-3 rounded-xl border border-border bg-card p-4">
        <div className="flex items-center justify-between">
            <StatsKicker icon={Palette} label="Skin Collection" />
            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                <span className="font-semibold text-foreground">87</span>
                <span className="text-muted-foreground/50"> / 412</span>
            </span>
        </div>
        <div className="border-border/60 border-t pt-3">
            <GapList
            items={[
                {
                    key: "missing",
                    label: "missing",
                    value: 325,
                    color: "var(--primary)",
                    tooltip: "Open the skin collection viewer",
                    dialogContent: (
                        <DialogPopup>
                            <DialogHeader>
                                <DialogTitle>Skin Collection</DialogTitle>
                                <DialogDescription>87 of 412 non-default skins collected.</DialogDescription>
                            </DialogHeader>
                        </DialogPopup>
                    ),
                },
                ]}
                label="Skins"
            />
        </div>
    </div>
);

export const WithinMasteryCard = () => (
    <StatsStatCard className="max-w-sm" color={MASTERY.accent}>
        <div className={`flex h-full flex-col gap-5 ${PAD}`}>
            <StatsKicker icon={Sparkles} label="Skill Mastery" />
            <div className="grid grid-cols-3 gap-2">
                <StatsTile color={MASTERY.m3} sub="M3" value={62} />
                <StatsTile color={MASTERY.m6} sub="M6" value={34} />
                <StatsTile color={MASTERY.m9} sub="M9" value={21} />
            </div>
            <div className="space-y-2">
                <div className="flex items-center justify-between">
                    <span className={KICKER}>Total Mastery Levels</span>
                    <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                        <span className="font-semibold text-foreground">428</span>
                        <span className="text-muted-foreground/50"> / 864</span>
                    </span>
                </div>
                <Bar color={MASTERY.accent} pct={49.5} />
            </div>
            <div className="border-border/60 border-t pt-3">
                <GapList items={MASTERY_GAPS} />
            </div>
        </div>
    </StatsStatCard>
);
