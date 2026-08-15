import { Select, SelectItem, SelectPopup, SelectTrigger, SelectValue } from "frontend";

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

const labelFor = (value: string) => CLASSES.find((c) => c.value === value)?.label ?? "";

export const Default = () => (
    <div className="w-56">
        <Select defaultValue="guard">
            <SelectTrigger>
                <SelectValue placeholder="Class">{(value: string) => labelFor(value)}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                {CLASSES.map((c) => (
                    <SelectItem key={c.value} value={c.value}>
                        {c.label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);

export const Placeholder = () => (
    <div className="w-56">
        {/* No children render fn: it would shadow Base UI's placeholder and leave the trigger blank. */}
        <Select>
            <SelectTrigger>
                <SelectValue placeholder="Filter by class" />
            </SelectTrigger>
            <SelectPopup>
                {CLASSES.map((c) => (
                    <SelectItem key={c.value} value={c.value}>
                        {c.label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);

export const Sizes = () => (
    <div className="flex w-72 flex-col gap-3">
        <Select defaultValue="sniper">
            <SelectTrigger size="sm">
                <SelectValue placeholder="Class">{(value: string) => labelFor(value)}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                {CLASSES.map((c) => (
                    <SelectItem key={c.value} value={c.value}>
                        {c.label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
        <Select defaultValue="medic">
            <SelectTrigger>
                <SelectValue placeholder="Class">{(value: string) => labelFor(value)}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                {CLASSES.map((c) => (
                    <SelectItem key={c.value} value={c.value}>
                        {c.label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);

export const Disabled = () => (
    <div className="w-56">
        <Select defaultValue="caster" disabled>
            <SelectTrigger>
                <SelectValue placeholder="Class">{(value: string) => labelFor(value)}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                {CLASSES.map((c) => (
                    <SelectItem key={c.value} value={c.value}>
                        {c.label}
                    </SelectItem>
                ))}
            </SelectPopup>
        </Select>
    </div>
);
