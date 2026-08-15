import { Select, SelectGroup, SelectGroupLabel, SelectItem, SelectPopup, SelectSeparator, SelectTrigger, SelectValue } from "frontend";

const CLASSES = [
    { value: "vanguard", label: "Vanguard" },
    { value: "guard", label: "Guard" },
    { value: "defender", label: "Defender" },
    { value: "sniper", label: "Sniper" },
    { value: "caster", label: "Caster" },
    { value: "medic", label: "Medic" },
    { value: "supporter", label: "Supporter" },
    { value: "specialist", label: "Specialist" },
];

const STAGES = [
    { value: "1-7", label: "1-7 — Absorption" },
    { value: "2-8", label: "2-8 — Shadow" },
    { value: "4-6", label: "4-6 — Frozen Well" },
    { value: "6-16", label: "6-16 — Broken Wings" },
    { value: "7-18", label: "7-18 — Silent Night" },
    { value: "9-17", label: "9-17 — Perpetual Hunt" },
    { value: "10-8", label: "10-8 — Ancient Memories" },
    { value: "11-14", label: "11-14 — Weathered Stone" },
    { value: "12-17", label: "12-17 — Lonetrail" },
];

const MATERIALS: Record<string, string> = {
    d32: "D32 Steel",
    bipolar: "Bipolar Nanoflake",
    ceu: "Crystalline Electronic Unit",
    grindstone: "Grindstone Pentahydrate",
    rma: "RMA70-24",
    polymer: "Polymerization Preparation",
};

const SERVERS: Record<string, string> = {
    en: "EN (Yostar)",
    cn: "CN (Hypergryph)",
    jp: "JP (Yostar)",
    kr: "KR (Yostar)",
};

const classLabel = (value: string) => CLASSES.find((c) => c.value === value)?.label ?? "";
const stageLabel = (value: string) => STAGES.find((s) => s.value === value)?.label ?? "";

export const Open = () => (
    <div className="flex h-96 w-64 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="popup-open">
            Filter by class
        </label>
        <Select defaultOpen defaultValue="guard">
            <SelectTrigger id="popup-open">
                <SelectValue placeholder="Class">{(value: string) => classLabel(value)}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                {CLASSES.map((c) => (
                    <SelectItem key={c.value} value={c.value}>
                        {c.label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);

export const ScrollingList = () => (
    <div className="flex h-96 w-72 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="popup-scroll">
            Farming target
        </label>
        <Select defaultOpen defaultValue="10-8">
            <SelectTrigger id="popup-scroll">
                <SelectValue placeholder="Stage">{(value: string) => stageLabel(value)}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                {STAGES.map((s) => (
                    <SelectItem key={s.value} value={s.value}>
                        {s.label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);

export const Grouped = () => (
    <div className="flex h-96 w-64 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="popup-grouped">
            Material to farm
        </label>
        <Select defaultOpen defaultValue="d32">
            <SelectTrigger id="popup-grouped">
                <SelectValue placeholder="Material">{(value: string) => MATERIALS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectGroup>
                    <SelectGroupLabel>Tier 5 — Elite</SelectGroupLabel>
                    <SelectItem value="d32">D32 Steel</SelectItem>
                    <SelectItem value="bipolar">Bipolar Nanoflake</SelectItem>
                    <SelectItem value="ceu">Crystalline Electronic Unit</SelectItem>
                </SelectGroup>
                <SelectSeparator />
                <SelectGroup>
                    <SelectGroupLabel>Tier 4 — Advanced</SelectGroupLabel>
                    <SelectItem value="grindstone">Grindstone Pentahydrate</SelectItem>
                    <SelectItem value="rma">RMA70-24</SelectItem>
                    <SelectItem value="polymer">Polymerization Preparation</SelectItem>
                </SelectGroup>
            </SelectPopup>
        </Select>
    </div>
);

export const AboveTrigger = () => (
    // `side="top"` flips the popup above the trigger, so the field's caption sits
    // below it — a label above would be covered by the popup.
    <div className="flex h-96 w-64 flex-col items-start justify-end gap-1.5">
        <Select defaultOpen defaultValue="en">
            <SelectTrigger id="popup-top">
                <SelectValue placeholder="Server">{(value: string) => SERVERS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup align="start" alignItemWithTrigger={false} side="top">
                <SelectItem value="en">EN (Yostar)</SelectItem>
                <SelectItem value="cn">CN (Hypergryph)</SelectItem>
                <SelectItem value="jp">JP (Yostar)</SelectItem>
                <SelectItem value="kr">KR (Yostar)</SelectItem>
            </SelectPopup>
        </Select>
        <p className="text-muted-foreground text-xs">Server your roster syncs from.</p>
    </div>
);
