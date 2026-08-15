import { Badge, Button, Drawer, DrawerClose, DrawerCreateHandle, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle, DrawerTrigger } from "frontend";
import { PencilIcon, PlusIcon, SlidersHorizontalIcon } from "lucide-react";

// DrawerCreateHandle() makes a handle that lets triggers live anywhere in the
// tree — a toolbar button, a row action, a keyboard shortcut — and still drive
// one drawer declared elsewhere. Each story gets its own handle.
const depotSheet = DrawerCreateHandle();
const filterSheet = DrawerCreateHandle();
const closedSheet = DrawerCreateHandle();

const materials = [
    { name: "Orirock Cluster", have: 142, need: 96 },
    { name: "Polyester Pack", have: 38, need: 61 },
    { name: "RMA70-12", have: 4, need: 12 },
    { name: "Manganese Ore", have: 27, need: 18 },
];

export const DetachedRowTriggers = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Depot</span>
            <DrawerTrigger handle={depotSheet} render={<Button size="sm" variant="outline" />}>
                <PlusIcon />
                Add material
            </DrawerTrigger>
        </div>
        <div className="space-y-2">
            {materials.map((m) => (
                <div key={m.name} className="flex items-center justify-between gap-3 rounded-lg border bg-card p-3">
                    <div className="flex min-w-0 flex-col">
                        <span className="font-medium text-sm">{m.name}</span>
                        <span className="font-mono text-muted-foreground text-xs tabular-nums">
                            {m.have} / {m.need}
                        </span>
                    </div>
                    <DrawerTrigger aria-label={`Edit ${m.name}`} handle={depotSheet} render={<Button size="icon" variant="ghost" />}>
                        <PencilIcon />
                    </DrawerTrigger>
                </div>
            ))}
        </div>
        <Drawer handle={depotSheet} open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Edit depot count</DrawerTitle>
                    <DrawerDescription>Every row action above shares one handle, so a single sheet serves the whole list.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel className="space-y-2">
                    <div className="flex items-center justify-between rounded-lg border p-3">
                        <span className="text-muted-foreground text-sm">In depot</span>
                        <span className="font-mono text-lg tabular-nums">4</span>
                    </div>
                    <div className="flex items-center justify-between rounded-lg border p-3">
                        <span className="text-muted-foreground text-sm">Still needed</span>
                        <span className="font-mono text-lg tabular-nums">8</span>
                    </div>
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Cancel</DrawerClose>
                    <Button>Save count</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </div>
);

export const ToolbarHandle = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex flex-wrap items-center gap-2">
            <Badge>Chapter 8</Badge>
            <Badge variant="secondary">14 stages</Badge>
            <DrawerTrigger className="ms-auto" handle={filterSheet} render={<Button size="sm" variant="outline" />}>
                <SlidersHorizontalIcon />
                Filters
            </DrawerTrigger>
        </div>
        <div className="grid gap-3 sm:grid-cols-3">
            {[
                { stage: "8-14", sanity: "21", eff: "0.94" },
                { stage: "S8-2", sanity: "18", eff: "0.68" },
                { stage: "CE-6", sanity: "36", eff: "1.00" },
            ].map((s) => (
                <div key={s.stage} className="rounded-lg border bg-card p-3">
                    <div className="font-medium text-sm">{s.stage}</div>
                    <div className="mt-1 font-mono text-muted-foreground text-xs tabular-nums">
                        {s.sanity} sanity · {s.eff} eff
                    </div>
                </div>
            ))}
        </div>
        <Drawer handle={filterSheet} open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Stage filters</DrawerTitle>
                    <DrawerDescription>Opened from the toolbar button through a handle, not a nested trigger.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel>
                    <div className="divide-y">
                        {[
                            { label: "Min efficiency", value: "0.60" },
                            { label: "Max sanity", value: "36" },
                            { label: "Drop tier", value: "T3+" },
                            { label: "Challenge modes", value: "Included" },
                        ].map((row) => (
                            <div key={row.label} className="flex items-center justify-between py-2">
                                <span className="text-muted-foreground text-sm">{row.label}</span>
                                <span className="font-mono text-sm tabular-nums">{row.value}</span>
                            </div>
                        ))}
                    </div>
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Reset</DrawerClose>
                    <Button>Apply filters</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </div>
);

export const HandleTriggersClosed = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Sanity log</span>
            <DrawerTrigger handle={closedSheet} render={<Button size="sm" />}>
                <PlusIcon />
                Log a run
            </DrawerTrigger>
        </div>
        <p className="text-muted-foreground text-sm">Detached triggers render as ordinary buttons while the drawer they own is closed — the handle carries the association, so no wrapper is needed around them.</p>
        <div className="space-y-2">
            {[
                { at: "09:12", stage: "1-7", sanity: 6 },
                { at: "09:31", stage: "8-14", sanity: 21 },
                { at: "10:04", stage: "CE-6", sanity: 36 },
            ].map((r) => (
                <div key={r.at} className="flex items-center justify-between gap-3 rounded-lg border bg-card p-3">
                    <span className="font-mono text-muted-foreground text-xs tabular-nums">{r.at}</span>
                    <span className="font-medium text-sm">{r.stage}</span>
                    <span className="font-mono text-sm tabular-nums">−{r.sanity}</span>
                </div>
            ))}
        </div>
        <Drawer handle={closedSheet} position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Log a run</DrawerTitle>
                </DrawerHeader>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Cancel</DrawerClose>
                    <Button>Save</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </div>
);
