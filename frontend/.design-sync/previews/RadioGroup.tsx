import { Radio, RadioGroup } from "frontend";

const SCOPES = [
    { value: "all", label: "All operators", count: "312 operators" },
    { value: "filtered", label: "Current filters", count: "48 operators" },
    { value: "page", label: "This page only", count: "24 operators" },
];

/** The export dialog's scope picker — a label row per option. */
export const ExportScope = () => (
    <RadioGroup className="flex w-80 flex-col gap-1.5" defaultValue="filtered">
        {SCOPES.map((scope) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: Radio renders the input
            <label className="flex cursor-pointer items-center gap-2.5 rounded-md border border-transparent px-2 py-1.5 hover:bg-accent" key={scope.value}>
                <Radio value={scope.value} />
                <span className="flex flex-1 items-baseline justify-between gap-2">
                    <span className="text-foreground text-sm leading-none">{scope.label}</span>
                    <span className="font-mono text-muted-foreground text-xs leading-none tabular-nums">{scope.count}</span>
                </span>
            </label>
        ))}
    </RadioGroup>
);

/** Descriptive options — the server a Doctor's roster is read from. */
export const ServerRegion = () => (
    <RadioGroup className="flex w-80 flex-col gap-3" defaultValue="en">
        {[
            { value: "en", label: "Global (EN)", desc: "Yostar · patch 2.4.61" },
            { value: "cn", label: "Mainland (CN)", desc: "Hypergryph · patch 2.6.01" },
            { value: "jp", label: "Japan (JP)", desc: "Yostar · patch 2.4.61" },
        ].map((server) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: Radio renders the input
            <label className="flex cursor-pointer items-start gap-2.5" key={server.value}>
                <Radio className="mt-0.5" value={server.value} />
                <span className="flex flex-col gap-0.5">
                    <span className="font-medium text-foreground text-sm leading-none">{server.label}</span>
                    <span className="text-muted-foreground text-xs">{server.desc}</span>
                </span>
            </label>
        ))}
    </RadioGroup>
);

/** Group-level `disabled` — the picker is locked while an export is running. */
export const Disabled = () => (
    <RadioGroup className="flex w-80 flex-col gap-1.5" defaultValue="all" disabled>
        {SCOPES.map((scope) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: Radio renders the input
            <label className="flex cursor-not-allowed items-center gap-2.5 rounded-md border border-transparent px-2 py-1.5 opacity-50" key={scope.value}>
                <Radio value={scope.value} />
                <span className="flex flex-1 items-baseline justify-between gap-2">
                    <span className="text-foreground text-sm leading-none">{scope.label}</span>
                    <span className="font-mono text-muted-foreground text-xs leading-none tabular-nums">{scope.count}</span>
                </span>
            </label>
        ))}
    </RadioGroup>
);

/** A horizontal group — the tier-list editor's sort order. */
export const Horizontal = () => (
    <RadioGroup className="flex flex-row flex-wrap gap-4" defaultValue="rarity">
        {[
            { value: "rarity", label: "Rarity" },
            { value: "class", label: "Class" },
            { value: "release", label: "Release date" },
            { value: "name", label: "Name" },
        ].map((sort) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: Radio renders the input
            <label className="flex cursor-pointer items-center gap-2" key={sort.value}>
                <Radio value={sort.value} />
                <span className="text-foreground text-sm leading-none">{sort.label}</span>
            </label>
        ))}
    </RadioGroup>
);
