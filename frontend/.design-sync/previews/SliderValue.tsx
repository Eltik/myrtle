import { Label, Slider, SliderValue } from "frontend";

/** The value readout sits above the track, right-aligned, inside the Slider root. */
export const WithLabel = () => (
    <div className="w-full max-w-sm">
        <Slider defaultValue={[72]} max={100} min={0}>
            <div className="mb-1.5 flex items-baseline justify-between gap-4">
                <Label className="font-medium text-[11px] text-muted-foreground leading-none">Trust</Label>
                <SliderValue className="font-mono text-[10.5px] text-foreground tabular-nums" />
            </div>
        </Slider>
    </div>
);

/** Two thumbs — the value renders both ends of the range. */
export const RangeValue = () => (
    <div className="w-full max-w-sm">
        <Slider defaultValue={[6, 21]} max={30} min={0}>
            <div className="mb-1.5 flex items-baseline justify-between gap-4">
                <Label className="font-medium text-[11px] text-muted-foreground leading-none">Sanity cost</Label>
                <SliderValue className="font-mono text-[10.5px] text-foreground tabular-nums" />
            </div>
        </Slider>
    </div>
);

/** A children render function turns the raw numbers into the game's own promotion wording. */
export const CustomFormat = () => (
    <div className="w-full max-w-sm">
        <Slider defaultValue={[80]} max={90} min={1}>
            <div className="mb-1.5 flex items-baseline justify-between gap-4">
                <Label className="font-medium text-[11px] text-muted-foreground leading-none">Level</Label>
                <SliderValue className="font-mono text-[10.5px] text-foreground tabular-nums">{(_formatted, values) => `E2 ${values[0]} / 90`}</SliderValue>
            </div>
        </Slider>
    </div>
);

/** Stacked in a settings panel, so the readouts line up down the right edge. */
export const InPanel = () => (
    <div className="flex w-full max-w-sm flex-col gap-5 rounded-xl border bg-card p-4">
        <Slider defaultValue={[12]} max={20} min={0}>
            <div className="mb-1.5 flex items-baseline justify-between gap-4">
                <Label className="font-medium text-[11px] text-muted-foreground leading-none">Base sanity threshold</Label>
                <SliderValue className="font-mono text-[10.5px] text-foreground tabular-nums" />
            </div>
        </Slider>
        <Slider defaultValue={[3]} max={6} min={0}>
            <div className="mb-1.5 flex items-baseline justify-between gap-4">
                <Label className="font-medium text-[11px] text-muted-foreground leading-none">Potential</Label>
                <SliderValue className="font-mono text-[10.5px] text-foreground tabular-nums" />
            </div>
        </Slider>
    </div>
);
