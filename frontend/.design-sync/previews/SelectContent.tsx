import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "frontend";

/**
 * `SelectContent` is the alias `src/` imports most often — the same popup as
 * `SelectPopup`. Composition ported from `tools/shared/AxisControls.tsx` and
 * `user/profile/.../ItemsTab.tsx`.
 */

const METRICS: Record<string, string> = {
    dps: "DPS vs. defence",
    dph: "Damage per hit",
    total: "Total damage",
    ehp: "Effective HP",
};

const SORT_LABELS: Record<string, string> = {
    rarity: "Rarity",
    name: "Name",
    quantity: "Quantity owned",
    craftable: "Craftable first",
};

const RARITY_LABELS: Record<string, string> = {
    all: "All rarities",
    "6": "6★ only",
    "5": "5★ only",
    "4": "4★ only",
};

export const MetricPicker = () => (
    <div className="flex h-80 w-64 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="content-metric">
            Y metric
        </label>
        <Select defaultOpen defaultValue="dps">
            <SelectTrigger id="content-metric" size="sm">
                <SelectValue placeholder="Metric">{(value: string) => METRICS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectContent alignItemWithTrigger={false}>
                {Object.entries(METRICS).map(([value, label]) => (
                    <SelectItem key={value} value={value}>
                        {label}
                    </SelectItem>
                ))}
            </SelectContent>
        </Select>
    </div>
);

export const SortBy = () => (
    <div className="flex h-80 w-64 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="content-sort">
            Sort by
        </label>
        <Select defaultOpen defaultValue="quantity">
            <SelectTrigger id="content-sort">
                <SelectValue placeholder="Sort by">{(value: string) => SORT_LABELS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectContent alignItemWithTrigger={false}>
                {Object.entries(SORT_LABELS).map(([value, label]) => (
                    <SelectItem key={value} value={value}>
                        {label}
                    </SelectItem>
                ))}
            </SelectContent>
        </Select>
    </div>
);

export const ClosedInFilterRow = () => (
    <div className="flex w-full max-w-2xl flex-wrap items-end gap-2 rounded-lg border border-border bg-card p-4">
        <div className="w-44 space-y-1.5">
            <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="content-row-sort">
                Sort by
            </label>
            <Select defaultValue="rarity">
                <SelectTrigger id="content-row-sort">
                    <SelectValue placeholder="Sort by">{(value: string) => SORT_LABELS[value] ?? value}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                    {Object.entries(SORT_LABELS).map(([value, label]) => (
                        <SelectItem key={value} value={value}>
                            {label}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </div>
        <div className="w-36 space-y-1.5">
            <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="content-row-rarity">
                Rarity
            </label>
            <Select defaultValue="6">
                <SelectTrigger id="content-row-rarity">
                    <SelectValue placeholder="Rarity">{(value: string) => RARITY_LABELS[value] ?? value}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                    {Object.entries(RARITY_LABELS).map(([value, label]) => (
                        <SelectItem key={value} value={value}>
                            {label}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </div>
    </div>
);
