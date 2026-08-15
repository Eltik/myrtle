import { Select, SelectGroup, SelectGroupLabel, SelectItem, SelectPopup, SelectSeparator, SelectTrigger, SelectValue } from "frontend";

const STAGE_LABELS: Record<string, string> = {
    "8-1": "8-1 — Roaring Flare",
    "8-9": "8-9 — Burning Bright",
    "8-17": "8-17 — Ashes",
    "9-3": "9-3 — Stormwatch",
    "9-17": "9-17 — Perpetual Hunt",
    "10-8": "10-8 — Ancient Memories",
    "10-16": "10-16 — Shatterpoint",
};

const CLASS_LABELS: Record<string, string> = {
    vanguard: "Vanguard",
    guard: "Guard",
    defender: "Defender",
    sniper: "Sniper",
    caster: "Caster",
    medic: "Medic",
    supporter: "Supporter",
    specialist: "Specialist",
};

export const ByChapter = () => (
    <div className="flex h-96 w-72 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="group-chapter">
            Farming target
        </label>
        <Select defaultOpen defaultValue="9-17">
            <SelectTrigger id="group-chapter">
                <SelectValue placeholder="Stage">{(value: string) => STAGE_LABELS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectGroup>
                    <SelectGroupLabel>Chapter 8 — Roaring Flare</SelectGroupLabel>
                    <SelectItem value="8-1">8-1 — Roaring Flare</SelectItem>
                    <SelectItem value="8-9">8-9 — Burning Bright</SelectItem>
                    <SelectItem value="8-17">8-17 — Ashes</SelectItem>
                </SelectGroup>
                <SelectGroup>
                    <SelectGroupLabel>Chapter 9 — Stormwatch</SelectGroupLabel>
                    <SelectItem value="9-3">9-3 — Stormwatch</SelectItem>
                    <SelectItem value="9-17">9-17 — Perpetual Hunt</SelectItem>
                </SelectGroup>
            </SelectPopup>
        </Select>
    </div>
);

export const WithSeparators = () => (
    <div className="flex h-96 w-72 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="group-separated">
            Filter by class
        </label>
        <Select defaultOpen defaultValue="defender">
            <SelectTrigger id="group-separated">
                <SelectValue placeholder="Class">{(value: string) => CLASS_LABELS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectGroup>
                    <SelectGroupLabel>Frontline</SelectGroupLabel>
                    <SelectItem value="vanguard">Vanguard</SelectItem>
                    <SelectItem value="guard">Guard</SelectItem>
                    <SelectItem value="defender">Defender</SelectItem>
                </SelectGroup>
                <SelectSeparator />
                <SelectGroup>
                    <SelectGroupLabel>Backline</SelectGroupLabel>
                    <SelectItem value="sniper">Sniper</SelectItem>
                    <SelectItem value="caster">Caster</SelectItem>
                    <SelectItem value="medic">Medic</SelectItem>
                </SelectGroup>
            </SelectPopup>
        </Select>
    </div>
);

export const SingleGroup = () => (
    <div className="flex h-80 w-64 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="group-single">
            Recruitment pool
        </label>
        <Select defaultOpen defaultValue="9h">
            <SelectTrigger id="group-single">
                <SelectValue placeholder="Duration">{(value: string) => ({ "9h": "9:00 — Top Operator", "4h": "4:00 — Senior guarantee", "1h": "1:00 — Fastest" })[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectGroup>
                    <SelectGroupLabel>Recruitment duration</SelectGroupLabel>
                    <SelectItem value="9h">9:00 — Top Operator</SelectItem>
                    <SelectItem value="4h">4:00 — Senior guarantee</SelectItem>
                    <SelectItem value="1h">1:00 — Fastest</SelectItem>
                </SelectGroup>
            </SelectPopup>
        </Select>
    </div>
);
