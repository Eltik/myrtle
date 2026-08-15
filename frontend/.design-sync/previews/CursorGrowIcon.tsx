import { CursorGrowIcon, NumberField, NumberFieldGroup, NumberFieldInput, NumberFieldScrubArea } from "frontend";

export const Basic = () => (
    <div className="flex w-full max-w-sm items-center gap-3 rounded-xl border border-border bg-muted p-4">
        <CursorGrowIcon className="drop-shadow-[0_1px_1px_#0008] filter" />
        <div>
            <p className="font-medium text-sm">Scrub cursor</p>
            <p className="text-muted-foreground text-xs">Replaces the pointer while a number field is being dragged.</p>
        </div>
    </div>
);

export const Sizes = () => (
    <div className="flex w-full max-w-sm items-center gap-6 rounded-xl border border-border bg-muted p-4">
        {[
            { w: 20, h: 11, label: "Compact" },
            { w: 26, h: 14, label: "Default" },
            { w: 39, h: 21, label: "Large" },
        ].map((size) => (
            <div className="flex flex-col items-center gap-2" key={size.label}>
                <CursorGrowIcon className="drop-shadow-[0_1px_1px_#0008] filter" height={size.h} width={size.w} />
                <span className="font-mono text-[10.5px] text-muted-foreground leading-none">{size.label}</span>
            </div>
        ))}
    </div>
);

export const OnScrubField = () => (
    <div className="w-full max-w-sm rounded-xl border border-border bg-card p-4">
        <p className="mb-1 font-semibold text-sm">Enemy DEF sweep</p>
        <p className="mb-4 text-muted-foreground text-xs">
            Dragging the label swaps the pointer for <CursorGrowIcon className="inline-block align-middle drop-shadow-[0_1px_1px_#0008] filter" height={11} width={20} />.
        </p>
        <div className="w-32">
            <NumberField defaultValue={600} max={5000} min={0} step={50}>
                <NumberFieldScrubArea label="Enemy DEF" />
                <NumberFieldGroup>
                    <NumberFieldInput />
                </NumberFieldGroup>
            </NumberField>
        </div>
    </div>
);
