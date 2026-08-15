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
        <p className="mt-2 text-muted-foreground text-xs">Steps down by 50 per press.</p>
    </div>
);

export const AtMinimum = () => (
    <div className="w-40">
        <NumberField defaultValue={0} max={5000} min={0} step={50}>
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Enemy DEF</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-muted-foreground text-xs">Decrement is disabled once the value hits `min`.</p>
    </div>
);

export const LeadingOnly = () => (
    <div className="w-40">
        <NumberField defaultValue={12} max={90} min={0} size="sm">
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Trust bonus</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-muted-foreground text-xs">The decrement keeps the rounded leading edge of the group on its own.</p>
    </div>
);

export const Sizes = () => (
    <div className="flex items-end gap-3">
        {(["sm", "default", "lg"] as const).map((size) => (
            <div className="w-36" key={size}>
                <NumberField defaultValue={20} max={100} min={0} size={size} step={5}>
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
