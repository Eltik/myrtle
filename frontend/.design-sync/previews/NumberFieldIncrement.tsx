import { Label, NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "frontend";

export const Basic = () => (
    <div className="w-40">
        <NumberField defaultValue={20} max={100} min={0} step={5}>
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Enemy RES</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-muted-foreground text-xs">Steps up by 5 per press.</p>
    </div>
);

export const AtMaximum = () => (
    <div className="w-40">
        <NumberField defaultValue={3} max={3} min={0}>
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Skill mastery</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-muted-foreground text-xs">Increment is disabled once the value hits `max`.</p>
    </div>
);

export const TrailingOnly = () => (
    <div className="w-40">
        <NumberField defaultValue={4} max={12} min={0} size="sm">
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Squad slots</Label>
            <NumberFieldGroup>
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-muted-foreground text-xs">The increment keeps the rounded trailing edge of the group on its own.</p>
    </div>
);

export const InRosterPanel = () => (
    <div className="w-full max-w-sm rounded-xl border border-border bg-card p-4">
        <p className="mb-1 font-semibold text-sm">Planner targets — Skadi the Corrupting Heart</p>
        <p className="mb-4 text-muted-foreground text-xs">Raise the goal, the material bill recalculates.</p>
        <div className="grid grid-cols-2 gap-3">
            {[
                { label: "Target level", value: 80, max: 90 },
                { label: "Module stage", value: 2, max: 3 },
            ].map((row) => (
                <NumberField defaultValue={row.value} key={row.label} max={row.max} min={1} size="sm">
                    <Label className="block font-medium text-[11px] text-muted-foreground leading-none">{row.label}</Label>
                    <NumberFieldGroup>
                        <NumberFieldDecrement />
                        <NumberFieldInput />
                        <NumberFieldIncrement />
                    </NumberFieldGroup>
                </NumberField>
            ))}
        </div>
    </div>
);
