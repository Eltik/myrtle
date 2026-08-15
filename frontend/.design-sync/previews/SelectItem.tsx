import { Select, SelectItem, SelectPopup, SelectTrigger, SelectValue } from "frontend";
import { Ban, Globe, Star } from "lucide-react";

// An array, not a Record — integer-like object keys would be reordered ascending by JS.
const RARITY = [
    { value: "r6", label: "6★ — Top Operator" },
    { value: "r5", label: "5★ — Senior Operator" },
    { value: "r4", label: "4★ — Operator" },
    { value: "r3", label: "3★ — Operator" },
    { value: "r2", label: "2★ — Robot" },
];

const rarityLabel = (value: string) => RARITY.find((r) => r.value === value)?.label ?? value;

const SERVERS: Record<string, string> = {
    en: "EN (Yostar)",
    cn: "CN (Hypergryph)",
    jp: "JP (Yostar)",
    kr: "KR (Yostar)",
    tw: "TW (Longcheng)",
};

const MODULES: Record<string, string> = {
    base: "No module",
    x: "SWD-X — Stage 3",
    y: "SWD-Y — Stage 2",
    d: "SWD-D — Stage 1",
};

export const SelectedIndicator = () => (
    <div className="flex h-80 w-64 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="item-selected">
            Rarity filter
        </label>
        <Select defaultOpen defaultValue="r5">
            <SelectTrigger id="item-selected">
                <SelectValue placeholder="Rarity">{(value: string) => rarityLabel(value)}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                {RARITY.map((r) => (
                    <SelectItem key={r.value} value={r.value}>
                        {r.label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);

export const DisabledItem = () => (
    <div className="flex h-80 w-64 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="item-disabled">
            Server
        </label>
        <Select defaultOpen defaultValue="en">
            <SelectTrigger id="item-disabled">
                <SelectValue placeholder="Server">{(value: string) => SERVERS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectItem value="en">EN (Yostar)</SelectItem>
                <SelectItem value="cn">CN (Hypergryph)</SelectItem>
                <SelectItem value="jp">JP (Yostar)</SelectItem>
                <SelectItem value="kr">KR (Yostar)</SelectItem>
                <SelectItem disabled value="tw">
                    TW (Longcheng) — sunset
                </SelectItem>
            </SelectPopup>
        </Select>
    </div>
);

export const WithIcons = () => (
    <div className="flex h-80 w-72 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="item-icons">
            Mlynar — module
        </label>
        <Select defaultOpen defaultValue="x">
            <SelectTrigger id="item-icons">
                <SelectValue placeholder="Module">{(value: string) => MODULES[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectItem value="base">
                    <span className="flex items-center gap-2">
                        <Ban className="text-muted-foreground" />
                        No module
                    </span>
                </SelectItem>
                <SelectItem value="x">
                    <span className="flex items-center gap-2">
                        <Star className="text-primary" />
                        SWD-X — Stage 3
                    </span>
                </SelectItem>
                <SelectItem value="y">
                    <span className="flex items-center gap-2">
                        <Star className="text-primary" />
                        SWD-Y — Stage 2
                    </span>
                </SelectItem>
                <SelectItem value="d">
                    <span className="flex items-center gap-2">
                        <Globe className="text-muted-foreground" />
                        SWD-D — Stage 1 (CN only)
                    </span>
                </SelectItem>
            </SelectPopup>
        </Select>
    </div>
);

export const TruncatedItems = () => (
    <div className="flex h-80 w-56 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="item-truncated">
            Farming target
        </label>
        <Select defaultOpen defaultValue="ap5">
            <SelectTrigger id="item-truncated">
                <SelectValue placeholder="Stage">{(value: string) => (value === "ap5" ? "AP-5 — Cutting Edge Technology" : value)}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectItem value="ap5">AP-5 — Cutting Edge Technology</SelectItem>
                <SelectItem value="ce6">CE-6 — Cargo Escort</SelectItem>
                <SelectItem value="ls6">LS-6 — Long-Term Supply</SelectItem>
                <SelectItem value="sk5">SK-5 — Resource Search</SelectItem>
            </SelectPopup>
        </Select>
    </div>
);
