import { Select, SelectItem, SelectPopup, SelectTrigger, SelectValue } from "frontend";

const SORT_LABELS: Record<string, string> = {
    rarity: "Rarity",
    name: "Name",
    obtained: "Recently obtained",
    trust: "Trust",
    level: "Level",
};

const STAGES: Record<string, string> = {
    "1-7": "1-7 — Absorption",
    "S4-1": "S4-1 — Ambush",
    "10-8": "10-8 — Ancient Memories",
    "CE-6": "CE-6 — Cargo Escort",
    "AP-5": "AP-5 — Cutting Edge Technology",
};

const PROMOTIONS: Record<string, string> = {
    e0: "Elite 0 · Lv 50",
    e1: "Elite 1 · Lv 80",
    e2: "Elite 2 · Lv 90",
};

export const SelectedValue = () => (
    <div className="w-56 space-y-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="value-selected">
            Sort by
        </label>
        <Select defaultValue="obtained">
            <SelectTrigger id="value-selected">
                <SelectValue placeholder="Sort by">{(value: string) => SORT_LABELS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                {Object.entries(SORT_LABELS).map(([value, label]) => (
                    <SelectItem key={value} value={value}>
                        {label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);

export const PlaceholderOnly = () => (
    <div className="w-56 space-y-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="value-placeholder">
            Sort by
        </label>
        {/* Base UI: a children render fn shadows `placeholder`, so an unselected value must not pass one. */}
        <Select>
            <SelectTrigger id="value-placeholder">
                <SelectValue placeholder="Sort by" />
            </SelectTrigger>
            <SelectPopup>
                {Object.entries(SORT_LABELS).map(([value, label]) => (
                    <SelectItem key={value} value={value}>
                        {label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);

export const TruncatedValue = () => (
    <div className="w-44 space-y-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="value-truncated">
            Farming target
        </label>
        <Select defaultValue="AP-5">
            <SelectTrigger id="value-truncated">
                <SelectValue placeholder="Stage">{(value: string) => STAGES[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                {Object.entries(STAGES).map(([value, label]) => (
                    <SelectItem key={value} value={value}>
                        {label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);

export const FormattedValue = () => (
    <div className="w-64 space-y-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="value-formatted">
            Mlynar — promotion
        </label>
        <Select defaultValue="e2">
            <SelectTrigger id="value-formatted">
                <SelectValue placeholder="Promotion">{(value: string) => <span className="font-mono tabular-nums">{PROMOTIONS[value] ?? value}</span>}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                {Object.entries(PROMOTIONS).map(([value, label]) => (
                    <SelectItem key={value} value={value}>
                        {label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);
