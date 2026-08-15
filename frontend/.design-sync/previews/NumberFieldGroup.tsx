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

export const InputOnly = () => (
    <div className="w-40">
        <NumberField defaultValue={1250} min={0} step={10}>
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Enemy HP</Label>
            <NumberFieldGroup>
                <NumberFieldInput />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-muted-foreground text-xs">The group also frames a bare input, with no steppers.</p>
    </div>
);

export const Invalid = () => (
    <div className="w-40">
        <NumberField defaultValue={7} max={6} min={1}>
            <Label className="block font-medium text-[11px] text-muted-foreground leading-none">Potential</Label>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput aria-invalid />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-destructive-foreground text-xs">Potential ranges from 1 to 6.</p>
    </div>
);

export const EnemyStatGrid = () => (
    <div className="w-full max-w-sm rounded-xl border border-border bg-card p-4">
        <p className="mb-1 font-semibold text-sm">Enemy preset — Sarkaz Greatsword</p>
        <p className="mb-4 text-muted-foreground text-xs">Overrides applied to every operator in the DPS run.</p>
        <div className="grid grid-cols-2 gap-3">
            {[
                { label: "DEF", value: 600, step: 50 },
                { label: "RES", value: 20, step: 5 },
                { label: "HP", value: 12000, step: 500 },
                { label: "Count", value: 3, step: 1 },
            ].map((stat) => (
                <NumberField defaultValue={stat.value} key={stat.label} min={0} size="sm" step={stat.step}>
                    <Label className="block font-medium text-[11px] text-muted-foreground leading-none">{stat.label}</Label>
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
