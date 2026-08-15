import { Badge, Card, CardDescription, CardHeader, CardPanel, CardTitle, Label, Switch } from "frontend";

/** The settings page composition: a titled row, a description, and the switch on the right. */
export const SettingsRows = () => (
    <div className="flex w-full max-w-lg flex-col divide-y rounded-xl border bg-card">
        {[
            { id: "dyn", title: "Animate dynamic art", description: "Plays multi-megabyte Spine animations in place of static art.", checked: true },
            { id: "cn", title: "Show CN-only operators", description: "Includes operators that have not reached the Global server yet.", checked: true },
            { id: "spoil", title: "Hide story spoilers", description: "Blurs operator files and stage dialogue until you opt in.", checked: false },
        ].map((row) => (
            <div className="flex items-start justify-between gap-4 p-4" key={row.id}>
                <div className="flex min-w-0 flex-col gap-1">
                    <Label className="font-medium text-sm">{row.title}</Label>
                    <span className="text-muted-foreground text-xs">{row.description}</span>
                </div>
                <Switch aria-label={row.title} defaultChecked={row.checked} />
            </div>
        ))}
    </div>
);

/** Both states plus disabled, which the account panel shows for unlinked servers. */
export const States = () => (
    <div className="flex flex-col gap-4">
        <div className="flex items-center gap-3">
            <Switch aria-label="Checked" defaultChecked />
            <span className="text-sm">Checked</span>
        </div>
        <div className="flex items-center gap-3">
            <Switch aria-label="Unchecked" />
            <span className="text-sm">Unchecked</span>
        </div>
        <div className="flex items-center gap-3">
            <Switch aria-label="Checked and disabled" defaultChecked disabled />
            <span className="text-muted-foreground text-sm">Checked · disabled</span>
        </div>
        <div className="flex items-center gap-3">
            <Switch aria-label="Disabled" disabled />
            <span className="text-muted-foreground text-sm">Unchecked · disabled</span>
        </div>
    </div>
);

/** Inside a card, next to the localStorage key the preference writes to. */
export const InCard = () => (
    <Card className="max-w-md">
        <CardHeader>
            <CardTitle>Dynamic artwork</CardTitle>
            <CardDescription>
                Animate L2D operator illustrations across operator and profile pages.{" "}
                <Badge className="ml-1 font-mono" size="sm" variant="outline">
                    localStorage · myrtle-dynamic-art
                </Badge>
            </CardDescription>
        </CardHeader>
        <CardPanel className="flex items-center justify-between gap-4">
            <span className="text-muted-foreground text-sm">On by default; turn it off to save bandwidth.</span>
            <Switch aria-label="Animate dynamic art" defaultChecked />
        </CardPanel>
    </Card>
);

/** Compact toggles in a filter popover, label first. */
export const WithInlineLabels = () => (
    <div className="flex w-full max-w-xs flex-col gap-3 rounded-xl border bg-card p-4">
        <span className="font-medium text-muted-foreground text-xs uppercase tracking-tight">Roster filters</span>
        <div className="flex items-center justify-between gap-3">
            <Label className="text-sm">Owned only</Label>
            <Switch aria-label="Owned only" defaultChecked />
        </div>
        <div className="flex items-center justify-between gap-3">
            <Label className="text-sm">E2 only</Label>
            <Switch aria-label="E2 only" />
        </div>
        <div className="flex items-center justify-between gap-3">
            <Label className="text-sm">Has module</Label>
            <Switch aria-label="Has module" defaultChecked />
        </div>
    </div>
);
