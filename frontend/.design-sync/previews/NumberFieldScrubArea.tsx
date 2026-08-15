import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput, NumberFieldScrubArea } from "frontend";

export const Basic = () => (
    <div className="w-40">
        <NumberField defaultValue={600} max={5000} min={0} step={50}>
            <NumberFieldScrubArea label="Enemy DEF" />
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-muted-foreground text-xs">Drag the label sideways to scrub the value.</p>
    </div>
);

export const WithoutSteppers = () => (
    <div className="w-40">
        <NumberField defaultValue={20} max={100} min={0} step={5}>
            <NumberFieldScrubArea label="Enemy RES" />
            <NumberFieldGroup>
                <NumberFieldInput />
            </NumberFieldGroup>
        </NumberField>
        <p className="mt-2 text-muted-foreground text-xs">Scrubbing replaces the steppers entirely on dense panels.</p>
    </div>
);

export const Sizes = () => (
    <div className="flex items-end gap-3">
        {(["sm", "default", "lg"] as const).map((size) => (
            <div className="w-36" key={size}>
                <NumberField defaultValue={30} max={100} min={0} size={size} step={5}>
                    <NumberFieldScrubArea label={size === "default" ? "Default" : size === "sm" ? "Small" : "Large"} />
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

export const EnemyStatGrid = () => (
    <div className="w-full max-w-sm rounded-xl border border-border bg-card p-4">
        <p className="mb-1 font-semibold text-sm">Enemy preset — Sarkaz Greatsword</p>
        <p className="mb-4 text-muted-foreground text-xs">Every label is a scrub handle; drag to sweep a stat.</p>
        <div className="grid grid-cols-2 gap-3">
            {[
                { label: "DEF", value: 600, step: 50 },
                { label: "RES", value: 20, step: 5 },
                { label: "HP", value: 12000, step: 500 },
                { label: "Count", value: 3, step: 1 },
            ].map((stat) => (
                <NumberField defaultValue={stat.value} key={stat.label} min={0} size="sm" step={stat.step}>
                    <NumberFieldScrubArea label={stat.label} />
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
