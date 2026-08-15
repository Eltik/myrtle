import { Select, SelectGroup, SelectGroupLabel, SelectItem, SelectPopup, SelectSeparator, SelectTrigger, SelectValue } from "frontend";

const SORT_LABELS: Record<string, string> = {
    rarity: "Rarity",
    name: "Name",
    level: "Level",
    trust: "Trust",
    obtained: "Recently obtained",
    reset: "Reset to default",
};

const VIEW_LABELS: Record<string, string> = {
    detailed: "Detailed cards",
    compact: "Compact grid",
    list: "List",
    unowned: "Unowned only",
};

export const BeforeReset = () => (
    <div className="flex h-96 w-64 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="sep-reset">
            Sort roster by
        </label>
        <Select defaultOpen defaultValue="trust">
            <SelectTrigger id="sep-reset">
                <SelectValue placeholder="Sort by">{(value: string) => SORT_LABELS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectItem value="rarity">Rarity</SelectItem>
                <SelectItem value="name">Name</SelectItem>
                <SelectItem value="level">Level</SelectItem>
                <SelectItem value="trust">Trust</SelectItem>
                <SelectItem value="obtained">Recently obtained</SelectItem>
                <SelectSeparator />
                <SelectItem value="reset">Reset to default</SelectItem>
            </SelectPopup>
        </Select>
    </div>
);

export const BetweenGroups = () => (
    <div className="flex h-96 w-64 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="sep-groups">
            View mode
        </label>
        <Select defaultOpen defaultValue="compact">
            <SelectTrigger id="sep-groups">
                <SelectValue placeholder="View mode">{(value: string) => VIEW_LABELS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectGroup>
                    <SelectGroupLabel>Layout</SelectGroupLabel>
                    <SelectItem value="detailed">Detailed cards</SelectItem>
                    <SelectItem value="compact">Compact grid</SelectItem>
                    <SelectItem value="list">List</SelectItem>
                </SelectGroup>
                <SelectSeparator />
                <SelectGroup>
                    <SelectGroupLabel>Scope</SelectGroupLabel>
                    <SelectItem value="unowned">Unowned only</SelectItem>
                </SelectGroup>
            </SelectPopup>
        </Select>
    </div>
);

export const MultipleDividers = () => (
    <div className="flex h-96 w-72 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="sep-multiple">
            Sanity budget preset
        </label>
        <Select defaultOpen defaultValue="daily">
            <SelectTrigger id="sep-multiple">
                <SelectValue placeholder="Preset">{(value: string) => ({ daily: "Daily — 240 sanity", weekly: "Weekly — 1,680 sanity", event: "Event push — 4,812 sanity", custom: "Custom…" })[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectItem value="daily">Daily — 240 sanity</SelectItem>
                <SelectItem value="weekly">Weekly — 1,680 sanity</SelectItem>
                <SelectSeparator />
                <SelectItem value="event">Event push — 4,812 sanity</SelectItem>
                <SelectSeparator />
                <SelectItem value="custom">Custom…</SelectItem>
            </SelectPopup>
        </Select>
    </div>
);
