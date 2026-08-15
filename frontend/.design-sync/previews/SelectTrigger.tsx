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

const SORT = [
    { value: "rarity", label: "Rarity" },
    { value: "name", label: "Name" },
    { value: "release", label: "Release date" },
    { value: "level", label: "Level" },
];

const classLabel = (value: string) => CLASSES.find((c) => c.value === value)?.label ?? "";
const sortLabel = (value: string) => SORT.find((c) => c.value === value)?.label ?? "";

const ClassOptions = () => (
    <>
        {CLASSES.map((c) => (
            <SelectItem key={c.value} value={c.value}>
                {c.label}
            </SelectItem>
        ))}
    </>
);

export const Sizes = () => (
    <div className="flex w-72 flex-col gap-3">
        <div className="space-y-1.5">
            <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="trigger-sm">
                Sort operators (sm)
            </label>
            <Select defaultValue="rarity">
                <SelectTrigger id="trigger-sm" size="sm">
                    <SelectValue placeholder="Sort">{(value: string) => sortLabel(value)}</SelectValue>
                </SelectTrigger>
                <SelectPopup>
                    {SORT.map((s) => (
                        <SelectItem key={s.value} value={s.value}>
                            {s.label}
                        </SelectItem>
                    ))}
                </SelectPopup>
            </Select>
        </div>
        <div className="space-y-1.5">
            <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="trigger-default">
                Class (default)
            </label>
            <Select defaultValue="guard">
                <SelectTrigger id="trigger-default">
                    <SelectValue placeholder="Class">{(value: string) => classLabel(value)}</SelectValue>
                </SelectTrigger>
                <SelectPopup>
                    <ClassOptions />
                </SelectPopup>
            </Select>
        </div>
        <div className="space-y-1.5">
            <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="trigger-lg">
                Class (lg)
            </label>
            <Select defaultValue="specialist">
                <SelectTrigger id="trigger-lg" size="lg">
                    <SelectValue placeholder="Class">{(value: string) => classLabel(value)}</SelectValue>
                </SelectTrigger>
                <SelectPopup>
                    <ClassOptions />
                </SelectPopup>
            </Select>
        </div>
    </div>
);

export const Placeholder = () => (
    <div className="w-64 space-y-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="trigger-placeholder">
            Filter by class
        </label>
        {/* No children render fn — it would shadow Base UI's placeholder and leave the trigger blank. */}
        <Select>
            <SelectTrigger id="trigger-placeholder">
                <SelectValue placeholder="All classes" />
            </SelectTrigger>
            <SelectPopup>
                <ClassOptions />
            </SelectPopup>
        </Select>
    </div>
);

export const Disabled = () => (
    <div className="w-64 space-y-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="trigger-disabled">
            Class (roster is syncing)
        </label>
        <Select defaultValue="caster" disabled>
            <SelectTrigger id="trigger-disabled">
                <SelectValue placeholder="Class">{(value: string) => classLabel(value)}</SelectValue>
            </SelectTrigger>
            <SelectPopup>
                <ClassOptions />
            </SelectPopup>
        </Select>
    </div>
);

export const Invalid = () => (
    <div className="w-64 space-y-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="trigger-invalid">
            Server
        </label>
        <Select>
            <SelectTrigger aria-invalid id="trigger-invalid">
                <SelectValue placeholder="Pick a server" />
            </SelectTrigger>
            <SelectPopup>
                <SelectItem value="en">EN (Yostar)</SelectItem>
                <SelectItem value="cn">CN (Hypergryph)</SelectItem>
                <SelectItem value="jp">JP (Yostar)</SelectItem>
                <SelectItem value="kr">KR (Yostar)</SelectItem>
            </SelectPopup>
        </Select>
        <p className="text-destructive text-xs">Select a server before importing your roster.</p>
    </div>
);
