import { Badge, Checkbox, Label } from "frontend";

export const StageFilters = () => (
    <div className="flex max-w-xs flex-col gap-3">
        <p className="font-medium font-sans text-[11px] text-muted-foreground uppercase leading-none tracking-widest">Stage pool</p>
        {[
            { id: "main", label: "Mainline", count: "246", checked: true },
            { id: "event", label: "Side stories", count: "118", checked: true },
            { id: "sss", label: "Stationary Security Service", count: "24", checked: false },
            { id: "cm", label: "Challenge modes", count: "31", checked: false },
        ].map((row) => (
            <Label className="cursor-pointer gap-2.5" key={row.id}>
                <Checkbox aria-label={`Toggle ${row.label}`} defaultChecked={row.checked} />
                <span className="flex-1 text-sm">{row.label}</span>
                <span className="font-mono text-muted-foreground text-xs tabular-nums">{row.count}</span>
            </Label>
        ))}
    </div>
);

export const States = () => (
    <div className="flex max-w-xs flex-col gap-3">
        <Label className="gap-2.5">
            <Checkbox defaultChecked />
            <span className="text-sm">Include Annihilation</span>
        </Label>
        <Label className="gap-2.5">
            <Checkbox />
            <span className="text-sm">Include Contingency Contract</span>
        </Label>
        <Label className="gap-2.5">
            <Checkbox indeterminate />
            <span className="text-sm">Side stories (partly selected)</span>
        </Label>
        <Label className="gap-2.5">
            <Checkbox defaultChecked disabled />
            <span className="text-sm text-muted-foreground">Sync with EN account (locked)</span>
        </Label>
    </div>
);

export const GroupWithParent = () => (
    <div className="flex max-w-xs flex-col gap-2">
        <Label className="gap-2 font-medium font-sans text-[11px] text-muted-foreground uppercase leading-none tracking-widest">
            <Checkbox aria-label="Toggle all operator fields" indeterminate />
            <span>Operator</span>
            <span className="font-mono text-[10px] tracking-normal">2/4</span>
        </Label>
        <div className="flex flex-col gap-1 pl-6">
            {[
                { id: "name", label: "Name", checked: true },
                { id: "rarity", label: "Rarity", checked: true },
                { id: "module", label: "Module stage", checked: false },
                { id: "trust", label: "Trust", checked: false },
            ].map((field) => (
                <Label className="cursor-pointer gap-2 py-1 font-normal" key={field.id}>
                    <Checkbox defaultChecked={field.checked} />
                    <span className="text-[12.5px]">{field.label}</span>
                </Label>
            ))}
        </div>
    </div>
);

export const Invalid = () => (
    <div className="flex max-w-xs flex-col gap-2">
        <Label className="gap-2.5">
            <Checkbox aria-invalid />
            <span className="text-sm">I understand this clears my saved plan</span>
        </Label>
        <p className="pl-7 text-destructive-foreground text-xs">Confirm before deleting 3 planned operators.</p>
        <Label className="mt-2 gap-2.5">
            <Checkbox aria-invalid defaultChecked />
            <span className="text-sm">Overwrite depot from CSV</span>
            <Badge size="sm" variant="error">
                Destructive
            </Badge>
        </Label>
    </div>
);
