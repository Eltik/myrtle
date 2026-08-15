import { Button, RadioGroup, RadioGroupItem } from "frontend";

const FIELDS = [
    { value: "csv", label: "CSV", desc: "Comma-separated, one row per operator" },
    { value: "json", label: "JSON", desc: "Nested records with skills and modules" },
    { value: "md", label: "Markdown", desc: "A table you can paste into a guide" },
];

/** The export dialog's format picker — `RadioGroupItem` is the `Radio` alias. */
export const ExportFormat = () => (
    <div className="flex w-80 flex-col gap-3">
        <h3 className="font-semibold text-foreground text-sm uppercase tracking-wide">Format</h3>
        <RadioGroup className="flex flex-col gap-1.5" defaultValue="csv">
            {FIELDS.map((field) => (
                // biome-ignore lint/a11y/noLabelWithoutControl: RadioGroupItem renders the input
                <label className="flex cursor-pointer items-start gap-2.5 rounded-md border border-transparent px-2 py-1.5 hover:bg-accent" key={field.value}>
                    <RadioGroupItem className="mt-0.5" value={field.value} />
                    <span className="flex flex-col gap-0.5">
                        <span className="font-medium text-foreground text-sm leading-none">{field.label}</span>
                        <span className="text-muted-foreground text-xs">{field.desc}</span>
                    </span>
                </label>
            ))}
        </RadioGroup>
        <Button className="self-start" size="sm">
            Export 48 operators
        </Button>
    </div>
);

/** Counts on the trailing edge — the scope section of the same dialog. */
export const WithCounts = () => (
    <RadioGroup className="flex w-80 flex-col gap-1.5" defaultValue="filtered">
        {[
            { value: "all", label: "All operators", count: "312" },
            { value: "filtered", label: "Current filters", count: "48" },
            { value: "page", label: "This page only", count: "24" },
        ].map((scope) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: RadioGroupItem renders the input
            <label className="flex cursor-pointer items-center gap-2.5 rounded-md border border-transparent px-2 py-1.5 hover:bg-accent" key={scope.value}>
                <RadioGroupItem value={scope.value} />
                <span className="flex flex-1 items-baseline justify-between gap-2">
                    <span className="text-foreground text-sm leading-none">{scope.label}</span>
                    <span className="font-mono text-muted-foreground text-xs leading-none tabular-nums">{scope.count} operators</span>
                </span>
            </label>
        ))}
    </RadioGroup>
);

/** A disabled item — "this page only" is unavailable when the table is empty. */
export const OneDisabled = () => (
    <RadioGroup className="flex w-80 flex-col gap-1.5" defaultValue="all">
        {[
            { value: "all", label: "All operators", count: "312", disabled: false },
            { value: "filtered", label: "Current filters", count: "48", disabled: false },
            { value: "page", label: "This page only", count: "0", disabled: true },
        ].map((scope) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: RadioGroupItem renders the input
            <label className={scope.disabled ? "flex cursor-not-allowed items-center gap-2.5 rounded-md border border-transparent px-2 py-1.5 opacity-50" : "flex cursor-pointer items-center gap-2.5 rounded-md border border-transparent px-2 py-1.5 hover:bg-accent"} key={scope.value}>
                <RadioGroupItem disabled={scope.disabled} value={scope.value} />
                <span className="flex flex-1 items-baseline justify-between gap-2">
                    <span className="text-foreground text-sm leading-none">{scope.label}</span>
                    <span className="font-mono text-muted-foreground text-xs leading-none tabular-nums">{scope.count} operators</span>
                </span>
            </label>
        ))}
    </RadioGroup>
);
