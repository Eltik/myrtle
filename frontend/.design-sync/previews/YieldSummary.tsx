import { YieldSummary } from "frontend";

const asn = (lmd: number, exp: number, eff: number) => ({
    rooms: [],
    total_production_efficiency: eff,
    yield_lmd_per_day: lmd,
    yield_exp_per_day: exp,
    yield_total_value: lmd + exp,
});

/** The header bar above the peak room grid, exactly as `PeakGrid` composes it. */
export const PeakYieldHeader = () => (
    <div className="flex max-w-md items-center justify-between rounded-md border border-border/35 bg-muted/10 px-2.5 py-1.5">
        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Peak yield</span>
        <YieldSummary assignment={asn(41_200, 8_640, 118.5)} />
    </div>
);

/** A trading-only base: no factory on EXP, so the EXP half of the summary drops out. */
export const LmdOnly = () => (
    <div className="flex max-w-md items-center justify-between rounded-md border border-border/35 bg-muted/10 px-2.5 py-1.5">
        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Peak yield</span>
        <YieldSummary assignment={asn(25_700, 0, 86)} />
    </div>
);

/** Current vs optimal side by side — the same figure rendered for both assignments. */
export const CurrentAndOptimal = () => (
    <div className="flex max-w-md flex-col gap-1.5 rounded-md border border-border/40 bg-muted/10 p-3">
        <div className="flex items-center justify-between gap-2">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">As stationed</span>
            <YieldSummary assignment={asn(31_400, 5_200, 92.4)} />
        </div>
        <div className="flex items-center justify-between gap-2">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Optimal</span>
            <YieldSummary assignment={asn(41_200, 8_640, 118.5)} />
        </div>
    </div>
);
