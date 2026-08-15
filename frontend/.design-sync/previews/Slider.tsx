import { Label, Slider } from "frontend";

/** Ported from the DPS calculator's InstanceCard: level and trust, each with a mono readout. */
export const LevelAndTrust = () => (
    <div className="flex w-full max-w-sm flex-col gap-5">
        <div>
            <div className="mb-1.5 flex items-baseline justify-between">
                <Label className="font-medium text-[11px] text-muted-foreground leading-none">Level</Label>
                <span className="font-mono text-[10.5px] text-foreground tabular-nums">
                    80
                    <span className="text-muted-foreground"> / 90</span>
                </span>
            </div>
            <Slider defaultValue={[80]} max={90} min={1} step={1} />
        </div>
        <div>
            <div className="mb-1.5 flex items-baseline justify-between">
                <Label className="font-medium text-[11px] text-muted-foreground leading-none">Trust</Label>
                <span className="font-mono text-[10.5px] text-foreground tabular-nums">100%</span>
            </div>
            <Slider defaultValue={[100]} max={100} min={0} step={5} />
        </div>
    </div>
);

/** Two thumbs — the stage filter's sanity-cost range. */
export const Range = () => (
    <div className="flex w-full max-w-sm flex-col gap-2">
        <div className="flex items-baseline justify-between">
            <Label className="font-medium text-[11px] text-muted-foreground leading-none">Sanity cost</Label>
            <span className="font-mono text-[10.5px] text-foreground tabular-nums">6 – 21</span>
        </div>
        <Slider defaultValue={[6, 21]} max={30} min={0} step={1} />
    </div>
);

/** Coarse steps — promotion-level weighting in 5% increments. */
export const Stepped = () => (
    <div className="flex w-full max-w-sm flex-col gap-2">
        <div className="flex items-baseline justify-between">
            <Label className="font-medium text-[11px] text-muted-foreground leading-none">Drop-rate confidence</Label>
            <span className="font-mono text-[10.5px] text-foreground tabular-nums">65%</span>
        </div>
        <Slider defaultValue={[65]} max={100} min={0} step={5} />
    </div>
);

/** Disabled — module level is locked until the operator reaches E2 40. */
export const Disabled = () => (
    <div className="flex w-full max-w-sm flex-col gap-2">
        <div className="flex items-baseline justify-between">
            <Label className="font-medium text-[11px] text-muted-foreground leading-none">Module level</Label>
            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">Locked</span>
        </div>
        <Slider defaultValue={[1]} disabled max={3} min={1} step={1} />
    </div>
);
