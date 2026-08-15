import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";

const DepotBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Depot</span>
            <Badge variant="secondary">187 material types</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-3">
            {[
                { name: "Orirock Cluster", have: 142, need: 96 },
                { name: "Polyester Pack", have: 38, need: 61 },
                { name: "RMA70-12", have: 4, need: 12 },
                { name: "Manganese Ore", have: 27, need: 18 },
                { name: "Grindstone", have: 9, need: 21 },
                { name: "Sugar Pack", have: 55, need: 40 },
            ].map((m) => (
                <div key={m.name} className="rounded-lg border bg-card p-3">
                    <div className="truncate font-medium text-sm">{m.name}</div>
                    <div className="mt-1 font-mono text-muted-foreground text-xs tabular-nums">
                        {m.have} / {m.need}
                    </div>
                </div>
            ))}
        </div>
    </div>
);

export const BottomSheet = () => (
    <>
        <DepotBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Edit depot count</DrawerTitle>
                    <DrawerDescription>RMA70-12 · Tier 4 · used by 23 of your planned upgrades.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel className="space-y-3">
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
    </>
);

export const InsetVariant = () => (
    <>
        <DepotBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar variant="inset">
                <DrawerHeader>
                    <DrawerTitle>Import from game data</DrawerTitle>
                    <DrawerDescription>Pulls your live depot from the EN server. Manual edits made after 12 May will be overwritten.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Keep manual counts</DrawerClose>
                    <Button>Import 187 items</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const StraightVariant = () => (
    <>
        <DepotBehind />
        <Drawer open position="bottom">
            <DrawerPopup variant="straight">
                <DrawerHeader>
                    <DrawerTitle>3 materials below plan</DrawerTitle>
                    <DrawerDescription>Squared corners edge-to-edge — the straight variant is for sheets that sit flush against the viewport.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Dismiss</DrawerClose>
                    <Button>Plan farming route</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const RightPanel = () => (
    <>
        <DepotBehind />
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Orirock Cluster</DrawerTitle>
                    <DrawerDescription>Tier 3 · crafted from 5 Orirock Cube · byproduct rate 20%</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel>
                    <div className="divide-y">
                        {[
                            { label: "In depot", value: "142" },
                            { label: "Planned use", value: "96" },
                            { label: "Best stage", value: "1-7" },
                            { label: "Sanity per craft", value: "42.8" },
                            { label: "Craft cost", value: "300 LMD" },
                        ].map((row) => (
                            <div key={row.label} className="flex items-center justify-between py-2">
                                <span className="text-muted-foreground text-sm">{row.label}</span>
                                <span className="font-mono text-sm tabular-nums">{row.value}</span>
                            </div>
                        ))}
                    </div>
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Close</DrawerClose>
                    <Button>Open item page</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
