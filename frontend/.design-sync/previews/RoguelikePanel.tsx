import { RoguelikePanel } from "frontend";
import type { ReactNode } from "react";

const ROGUE_ACCENT = "oklch(0.65 0.22 340)";

/** The expanded Roguelike subscore card the panel always lives inside. */
const PanelFrame = ({ pct, children }: { pct: string; children: ReactNode }) => (
    <div className="mx-auto max-w-2xl overflow-hidden rounded-xl border border-border bg-card sm:rounded-2xl">
        <div className="flex items-center justify-between p-4 sm:p-5">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Roguelike · IS endings &amp; relics</span>
            <span className="font-bold text-2xl tabular-nums" style={{ color: ROGUE_ACCENT }}>
                {pct}
            </span>
        </div>
        <div className="border-border/30 border-t">{children}</div>
    </div>
);

const withThemes = {
    roguelike: [
        {
            theme_id: "rogue_3",
            theme_name: "Mizuki & Caerula Arbor",
            endings: { current: 3, max: 5 },
            difficulty: { highest_cleared: 12, max: 15 },
            collectibles: { relics: { current: 214, max: 286 }, capsules: { current: 18, max: 24 }, bands: { current: 9, max: 12 } },
            bp: { current: 180, max: 200 },
            challenges: { current: 22, max: 30 },
        },
        {
            theme_id: "rogue_4",
            theme_name: "Expeditioner's Jǫklumarkar",
            endings: { current: 2, max: 5 },
            difficulty: { highest_cleared: 8, max: 15 },
            collectibles: { relics: { current: 141, max: 268 }, capsules: { current: 11, max: 24 }, bands: { current: 6, max: 12 } },
            bp: { current: 96, max: 200 },
            challenges: { current: 9, max: 30 },
        },
    ],
};

const noThemes = { roguelike: [] };

export const ThemeProgress = () => (
    <PanelFrame pct="74.3%">
        <RoguelikePanel improvements={withThemes} accent={ROGUE_ACCENT} />
    </PanelFrame>
);

// Nothing synced for Integrated Strategies — the panel falls back to its hint
// rather than rendering empty progress bars.
export const NoThemesSynced = () => (
    <PanelFrame pct="0.0%">
        <RoguelikePanel improvements={noThemes} accent={ROGUE_ACCENT} />
    </PanelFrame>
);
