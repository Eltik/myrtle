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

export const Placeholder = () => (
    <div className="w-40">
        <NumberField min={0} step={50}>
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Enemy DEF</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput placeholder="Auto" />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-muted-foreground text-xs">Empty field falls back to the stage's own enemy stats.</p>
    </div>
);

export const Formatted = () => (
    <div className="w-40">
        <NumberField defaultValue={1200000} format={{ notation: "compact" }} min={0} step={100000}>
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">LMD reserve</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-muted-foreground text-xs">Intl formatting flows into the input's display value.</p>
    </div>
);

export const Invalid = () => (
    <div className="w-40">
        <NumberField defaultValue={999} max={90} min={1}>
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Operator level</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput aria-invalid />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-destructive-foreground text-xs">Max level at E2 is 90.</p>
    </div>
);
