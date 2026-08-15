import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";

const EventBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Ideal City</span>
            <Badge variant="secondary">Ends in 6 days</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
            {[
                { name: "IC-7 Rebar Yard", sanity: 18 },
                { name: "IC-8 Foundry", sanity: 21 },
                { name: "IC-EX-6 Skyline", sanity: 25 },
                { name: "IC-EX-8 Groundwork", sanity: 25 },
            ].map((s) => (
                <div key={s.name} className="flex items-center justify-between rounded-lg border bg-card p-3">
                    <span className="font-medium text-sm">{s.name}</span>
                    <span className="font-mono text-muted-foreground text-xs tabular-nums">{s.sanity}</span>
                </div>
            ))}
        </div>
    </div>
);

export const BottomSheetBar = () => (
    <>
        <EventBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerTitle className="sr-only">Event shop</DrawerTitle>
                <DrawerPanel className="space-y-3">
                    <div className="font-heading font-semibold text-xl">Event shop</div>
                    <div className="text-muted-foreground text-sm">The grab bar sits above the content and marks the sheet as swipe-dismissable.</div>
                    <div className="divide-y">
                        {[
                            { item: "Bipolar Nanoflake", cost: "300 tokens" },
                            { item: "Polymerization Preparation", cost: "300 tokens" },
                            { item: "Skill Summary 3", cost: "35 tokens" },
                        ].map((r) => (
                            <div key={r.item} className="flex items-center justify-between py-2">
                                <span className="text-sm">{r.item}</span>
                                <span className="font-mono text-sm tabular-nums">{r.cost}</span>
                            </div>
                        ))}
                    </div>
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Close</DrawerClose>
                    <Button>Plan token spend</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const TopSheetBar = () => (
    <>
        <EventBehind />
        <Drawer open position="top">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Event ends in 6 days</DrawerTitle>
                    <DrawerDescription>You still have 1,240 unspent tokens. For position=top the bar renders along the bottom edge.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Dismiss</DrawerClose>
                    <Button>Open event shop</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const SidePanelBar = () => (
    <>
        <EventBehind />
        <Drawer open position="right">
            <DrawerPopup showBar>
                <DrawerHeader>
                    <DrawerTitle>IC-EX-8 Groundwork</DrawerTitle>
                    <DrawerDescription>25 sanity · 3 challenge ratings · the bar runs vertically along the leading edge.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel>
                    <div className="divide-y">
                        {[
                            { label: "Tokens per run", value: "48" },
                            { label: "First clear", value: "Done" },
                            { label: "Challenge", value: "2 of 3" },
                            { label: "Best time", value: "1:52" },
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
                    <Button>Add to plan</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
