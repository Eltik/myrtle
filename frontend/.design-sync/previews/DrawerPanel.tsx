import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle, Input, Label, Switch } from "frontend";

const StatsBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Sanity log</span>
            <Badge variant="secondary">Last 30 days</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-3">
            {[
                { label: "Spent", value: "4,812" },
                { label: "Wasted", value: "126" },
                { label: "Efficiency", value: "97.4%" },
            ].map((s) => (
                <div key={s.label} className="rounded-lg border bg-card p-3">
                    <div className="text-muted-foreground text-xs">{s.label}</div>
                    <div className="mt-1 font-mono font-semibold text-2xl tabular-nums">{s.value}</div>
                </div>
            ))}
        </div>
    </div>
);

const runs = [
    { stage: "8-14", sanity: 21, drops: "RMA70-12 ×1", when: "12 May 21:04" },
    { stage: "8-14", sanity: 21, drops: "Manganese Ore ×2", when: "12 May 20:58" },
    { stage: "CE-6", sanity: 36, drops: "LMD ×10,000", when: "12 May 20:31" },
    { stage: "1-7", sanity: 6, drops: "Orirock Cube ×3", when: "12 May 19:44" },
    { stage: "S4-1", sanity: 18, drops: "Polyester ×1", when: "11 May 22:12" },
    { stage: "AP-5", sanity: 30, drops: "Skill Summary ×4", when: "11 May 21:50" },
    { stage: "4-8", sanity: 21, drops: "Grindstone ×1", when: "11 May 21:20" },
    { stage: "PR-D-2", sanity: 30, drops: "Guard Chip ×2", when: "11 May 20:05" },
];

export const ScrollingList = () => (
    <>
        <StatsBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Runs on 12 May</DrawerTitle>
                    <DrawerDescription>The panel scrolls inside the sheet — the header and footer stay pinned.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel className="space-y-2">
                    {runs.map((r) => (
                        <div key={`${r.stage}-${r.when}`} className="flex items-center justify-between gap-3 rounded-lg border p-3">
                            <div className="flex min-w-0 flex-col gap-1">
                                <span className="font-medium font-mono text-sm">{r.stage}</span>
                                <span className="truncate text-muted-foreground text-xs">{r.drops}</span>
                            </div>
                            <span className="shrink-0 font-mono text-muted-foreground text-xs tabular-nums">{r.sanity} sanity</span>
                        </div>
                    ))}
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Close</DrawerClose>
                    <Button>Export day</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const ScrollFadeOff = () => (
    <>
        <StatsBehind />
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Drop history</DrawerTitle>
                    <DrawerDescription>scrollFade off — rows stay at full contrast against the panel edge.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel className="space-y-2" scrollFade={false}>
                    {runs.map((r) => (
                        <div key={`${r.stage}-${r.when}`} className="rounded-lg border p-3">
                            <div className="flex items-center justify-between gap-3">
                                <span className="font-medium font-mono text-sm">{r.stage}</span>
                                <span className="font-mono text-muted-foreground text-xs tabular-nums">{r.sanity} sanity</span>
                            </div>
                            <div className="mt-1 text-muted-foreground text-xs">
                                {r.drops} · {r.when}
                            </div>
                        </div>
                    ))}
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Close</DrawerClose>
                    <Button>Open full log</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const FormFields = () => (
    <>
        <StatsBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Sanity budget</DrawerTitle>
                    <DrawerDescription>Used to estimate how long each planner goal will take.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel className="space-y-4">
                    <div className="flex flex-col gap-2">
                        <Label htmlFor="drawer-budget">Daily sanity</Label>
                        <Input defaultValue="240" id="drawer-budget" />
                    </div>
                    <div className="flex flex-col gap-2">
                        <Label htmlFor="drawer-potions">Potions per week</Label>
                        <Input defaultValue="6" id="drawer-potions" />
                    </div>
                    <div className="flex items-center justify-between gap-3 rounded-lg border p-3">
                        <div className="flex min-w-0 flex-col gap-1">
                            <span className="font-medium text-sm">Count event stages</span>
                            <span className="text-muted-foreground text-xs">Includes limited-time farming in the estimate.</span>
                        </div>
                        <Switch defaultChecked />
                    </div>
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Cancel</DrawerClose>
                    <Button>Save budget</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
