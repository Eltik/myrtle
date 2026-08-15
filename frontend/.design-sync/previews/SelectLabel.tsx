import { Select, SelectItem, SelectLabel, SelectPopup, SelectTrigger, SelectValue } from "frontend";
import { Info } from "lucide-react";

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

const SERVER_LABELS: Record<string, string> = {
    en: "EN (Yostar)",
    cn: "CN (Hypergryph)",
    jp: "JP (Yostar)",
    kr: "KR (Yostar)",
};

const PROMOTION_LABELS: Record<string, string> = {
    e0: "Elite 0 · Lv 50",
    e1: "Elite 1 · Lv 80",
    e2: "Elite 2 · Lv 90",
};

const ClassOptions = () => (
    <>
        {Object.entries(CLASS_LABELS).map(([value, label]) => (
            <SelectItem key={value} value={value}>
                {label}
            </SelectItem>
        ))}
    </>
);

export const LabelledField = () => (
    <div className="w-64">
        <Select defaultValue="sniper">
            <SelectLabel>Filter by class</SelectLabel>
            <SelectTrigger>
                <SelectValue placeholder="Class">{(value: string) => CLASS_LABELS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                <ClassOptions />
            </SelectPopup>
        </Select>
    </div>
);

export const WithHintIcon = () => (
    <div className="w-72">
        <Select defaultValue="en">
            <SelectLabel>
                Sync roster from
                <Info className="size-4 text-muted-foreground" />
            </SelectLabel>
            <SelectTrigger>
                <SelectValue placeholder="Server">{(value: string) => SERVER_LABELS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                {Object.entries(SERVER_LABELS).map(([value, label]) => (
                    <SelectItem key={value} value={value}>
                        {label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
        <p className="mt-1.5 text-muted-foreground text-xs">We only read your support unit and clear history.</p>
    </div>
);

export const StackedFields = () => (
    <div className="w-full max-w-sm space-y-4 rounded-lg border border-border bg-card p-4">
        <span className="block font-medium text-foreground text-sm">Stat calculator</span>
        <Select defaultValue="guard">
            <SelectLabel>Class</SelectLabel>
            <SelectTrigger size="sm">
                <SelectValue placeholder="Class">{(value: string) => CLASS_LABELS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                <ClassOptions />
            </SelectPopup>
        </Select>
        <Select defaultValue="e2">
            <SelectLabel>Promotion</SelectLabel>
            <SelectTrigger size="sm">
                <SelectValue placeholder="Promotion">{(value: string) => PROMOTION_LABELS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                {Object.entries(PROMOTION_LABELS).map(([value, label]) => (
                    <SelectItem key={value} value={value}>
                        {label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);

export const DisabledField = () => (
    <div className="w-64">
        <Select defaultValue="medic" disabled>
            <SelectLabel>Class (roster is syncing)</SelectLabel>
            <SelectTrigger>
                <SelectValue placeholder="Class">{(value: string) => CLASS_LABELS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                <ClassOptions />
            </SelectPopup>
        </Select>
    </div>
);
