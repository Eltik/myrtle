import { Label, NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "frontend";

export const Basic = () => (
    <div className="w-40">
        <NumberField defaultValue={600} max={5000} min={0} step={50}>
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Enemy DEF</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
    </div>
);

export const Sizes = () => (
    <div className="flex items-end gap-3">
        {(["sm", "default", "lg"] as const).map((size) => (
            <div className="w-36" key={size}>
                <NumberField defaultValue={600} max={5000} min={0} size={size} step={50}>
                    <Label className="block font-medium text-[11px] text-muted-foreground leading-none">{size === "default" ? "Default" : size === "sm" ? "Small" : "Large"}</Label>
                    <NumberFieldGroup>
                        <NumberFieldDecrement />
                        <NumberFieldInput />
                        <NumberFieldIncrement />
                    </NumberFieldGroup>
                </NumberField>
            </div>
        ))}
    </div>
);

export const SweepRange = () => (
    <div className="w-full max-w-sm rounded-xl border border-border bg-card p-4">
        <p className="mb-1 font-semibold text-sm">Enemy DEF sweep</p>
        <p className="mb-4 text-muted-foreground text-xs">Range plotted on the DPS curve for Mlynar.</p>
        <div className="flex items-end gap-2">
            <NumberField className="w-32" defaultValue={0} max={4999} min={0} size="sm" step={100}>
                <Label className="block text-[10.5px] text-muted-foreground leading-none">From</Label>
                <NumberFieldGroup>
                    <NumberFieldDecrement />
                    <NumberFieldInput />
                    <NumberFieldIncrement />
                </NumberFieldGroup>
            </NumberField>
            <NumberField className="w-32" defaultValue={2000} max={5000} min={1} size="sm" step={100}>
                <Label className="block text-[10.5px] text-muted-foreground leading-none">To</Label>
                <NumberFieldGroup>
                    <NumberFieldDecrement />
                    <NumberFieldInput />
                    <NumberFieldIncrement />
                </NumberFieldGroup>
            </NumberField>
        </div>
    </div>
);

export const DisabledAndReadOnly = () => (
    <div className="flex items-end gap-3">
        <div className="w-32">
            <NumberField defaultValue={90} disabled min={0} size="sm">
                <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Trust (locked)</Label>
                <NumberFieldGroup>
                    <NumberFieldDecrement />
                    <NumberFieldInput />
                    <NumberFieldIncrement />
                </NumberFieldGroup>
            </NumberField>
        </div>
        <div className="w-32">
            <NumberField defaultValue={135} min={0} readOnly size="sm">
                <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Max sanity</Label>
                <NumberFieldGroup>
                    <NumberFieldDecrement />
                    <NumberFieldInput />
                    <NumberFieldIncrement />
                </NumberFieldGroup>
            </NumberField>
        </div>
    </div>
);

export const BoundsReached = () => (
    <div>
        <div className="flex items-end gap-3">
            <div className="w-32">
                <NumberField defaultValue={1} max={6} min={1} size="sm">
                    <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Potential</Label>
                    <NumberFieldGroup>
                        <NumberFieldDecrement />
                        <NumberFieldInput />
                        <NumberFieldIncrement />
                    </NumberFieldGroup>
                </NumberField>
            </div>
            <div className="w-32">
                <NumberField defaultValue={3} max={3} min={0} size="sm">
                    <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Mastery</Label>
                    <NumberFieldGroup>
                        <NumberFieldDecrement />
                        <NumberFieldInput />
                        <NumberFieldIncrement />
                    </NumberFieldGroup>
                </NumberField>
            </div>
        </div>
        <p className="mt-2 text-muted-foreground text-xs">Potential sits at its minimum, mastery at its maximum — each field disables the stepper it can no longer use.</p>
    </div>
);
