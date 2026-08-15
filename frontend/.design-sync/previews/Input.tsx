import { Input, Label } from "frontend";

export const Sizes = () => (
    <div className="flex max-w-sm flex-col gap-4">
        <div className="flex flex-col gap-2">
            <Label htmlFor="preview-input-sm">Small</Label>
            <Input id="preview-input-sm" placeholder="Search operators…" size="sm" />
        </div>
        <div className="flex flex-col gap-2">
            <Label htmlFor="preview-input-default">Default</Label>
            <Input id="preview-input-default" placeholder="Search operators…" size="default" />
        </div>
        <div className="flex flex-col gap-2">
            <Label htmlFor="preview-input-lg">Large</Label>
            <Input id="preview-input-lg" placeholder="Search operators…" size="lg" />
        </div>
    </div>
);

export const States = () => (
    <div className="flex max-w-sm flex-col gap-3">
        <Input defaultValue="Endgame DPS rankings" />
        <Input placeholder="e.g. Endgame DPS rankings" />
        <Input aria-invalid defaultValue="" placeholder="A list name is required" />
        <Input disabled placeholder="Sync your account to edit" />
        <Input readOnly value="Dr. Kal'tsit#4417" />
    </div>
);

export const Types = () => (
    <div className="flex max-w-sm flex-col gap-3">
        <Input aria-label="Search tier lists" placeholder="Search lists..." type="search" />
        <Input aria-label="Promotion level" className="w-24 text-center font-mono" defaultValue="60" max={90} min={1} type="number" />
        <Input aria-label="Account password" defaultValue="originium" type="password" />
    </div>
);

export const WithLabels = () => (
    <div className="flex max-w-sm flex-col gap-4">
        <div className="flex flex-col gap-2">
            <Label htmlFor="preview-input-nickname">Arknights nickname</Label>
            <Input id="preview-input-nickname" readOnly value="Dr. Kal'tsit#4417" />
            <span className="text-muted-foreground text-xs">Shown on your profile, leaderboard, and tier lists you publish.</span>
        </div>
        <div className="flex flex-col gap-2">
            <Label htmlFor="preview-input-level">Account level</Label>
            <Input className="w-32" id="preview-input-level" readOnly value="Lv. 120" />
            <span className="text-muted-foreground text-xs">Doctor level from the in-game profile.</span>
        </div>
    </div>
);
