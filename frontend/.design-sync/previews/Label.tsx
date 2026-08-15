import { Checkbox, Input, Label, Switch } from "frontend";

export const FormFields = () => (
    <div className="flex w-full max-w-sm flex-col gap-4">
        <div className="flex flex-col gap-1.5">
            <Label htmlFor="doctor-name">Doctor name</Label>
            <Input id="doctor-name" defaultValue="Dr. Kal'tsit" />
        </div>
        <div className="flex flex-col gap-1.5">
            <Label htmlFor="sync-token">Roster sync token</Label>
            <Input id="sync-token" defaultValue="ak_live_8f2c…4d91" />
            <span className="font-sans text-muted-foreground text-xs">Used once to import your operator list from the game client.</span>
        </div>
    </div>
);

export const WithControls = () => (
    <div className="flex w-full max-w-sm flex-col gap-4">
        <Label>
            <Switch defaultChecked />
            Show unreleased CN operators
        </Label>
        <Label>
            <Switch />
            Use dynamic artwork backgrounds
        </Label>
        <Label>
            <Checkbox defaultChecked />
            Only operators I own
        </Label>
        <Label>
            <Checkbox />
            Hide E0 operators from the roster grid
        </Label>
    </div>
);

export const DenseToolLabels = () => (
    <div className="w-full max-w-sm rounded-lg border border-border bg-card px-4 py-3">
        <Label className="mb-1.5 block font-medium text-[11px] text-muted-foreground leading-none">Quick presets</Label>
        <div className="grid grid-cols-2 gap-2">
            <div className="flex flex-col gap-1.5">
                <Label className="block font-medium text-[12px] text-muted-foreground leading-none">Enemy defense</Label>
                <Input defaultValue="1200" size="sm" />
            </div>
            <div className="flex flex-col gap-1.5">
                <Label className="block font-medium text-[12px] text-muted-foreground leading-none">Enemy resistance</Label>
                <Input defaultValue="40" size="sm" />
            </div>
        </div>
    </div>
);

export const AsSpan = () => (
    <div className="flex w-full max-w-sm items-center justify-between gap-4 rounded-lg border border-border bg-card px-4 py-3">
        <Label render={<span />}>Trust level</Label>
        <span className="font-mono text-foreground text-sm tabular-nums">200%</span>
    </div>
);
