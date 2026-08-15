import type { CSSProperties } from "react";
import { AccentKicker, YieldHeadline } from "frontend";

const ACCENT = { "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties;

const asn = (lmd: number, exp: number, eff: number) => ({
    rooms: [],
    total_production_efficiency: eff,
    yield_lmd_per_day: lmd,
    yield_exp_per_day: exp,
    yield_total_value: lmd + exp,
});

/** The Base panel's headline block: current → optimal daily yield with the efficiency delta. */
export const CurrentToOptimal = () => (
    <div className="flex max-w-lg flex-col gap-2 rounded-md border border-border/40 bg-muted/10 p-3" style={ACCENT}>
        <div className="flex flex-wrap items-center justify-between gap-2">
            <AccentKicker>Current vs optimal</AccentKicker>
            <span className="inline-flex items-center gap-1 rounded-md border border-border/50 px-1.5 py-0.5 font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em]" style={{ background: "color-mix(in oklch, var(--imp-accent) 8%, transparent)", color: "color-mix(in oklch, var(--imp-accent) 75%, var(--foreground))" }}>
                +13.2k value/day
            </span>
        </div>
        <YieldHeadline current={asn(31_400, 5_200, 92.4)} optimal={asn(41_200, 8_640, 118.5)} />
    </div>
);

/** An already well-optimised base: a small positive delta and no EXP factory. */
export const MarginalGain = () => (
    <div className="flex max-w-lg flex-col gap-2 rounded-md border border-border/40 bg-muted/10 p-3" style={ACCENT}>
        <AccentKicker>Current vs optimal</AccentKicker>
        <YieldHeadline current={asn(24_900, 0, 84.1)} optimal={asn(25_700, 0, 86.4)} />
    </div>
);

/** As the plan dialog's header renders it — under the layout legend, no card chrome. */
export const PlanHeader = () => (
    <div className="flex max-w-lg flex-col gap-2.5 border-border/40 border-b pb-3" style={ACCENT}>
        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Base optimization plan</span>
        <YieldHeadline current={asn(18_200, 3_100, 61.5)} optimal={asn(41_200, 8_640, 118.5)} />
    </div>
);
