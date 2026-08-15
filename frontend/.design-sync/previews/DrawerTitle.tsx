import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";

const PlannerBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Upgrade planner</span>
            <Badge variant="secondary">14 goals</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
            {[
                { op: "Mlynar", goal: "E2 90 · M3/M3/M3" },
                { op: "Skadi", goal: "E2 80 · M6/M6/M3" },
                { op: "Muelsyse", goal: "E2 90 · Module ω" },
                { op: "Texas", goal: "E2 60 · M3/—/—" },
            ].map((g) => (
                <div key={g.op} className="rounded-lg border bg-card p-3">
                    <div className="font-medium text-sm">{g.op}</div>
                    <div className="mt-1 font-mono text-muted-foreground text-xs">{g.goal}</div>
                </div>
            ))}
        </div>
    </div>
);

export const ShortTitle = () => (
    <>
        <PlannerBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Edit goal</DrawerTitle>
                    <DrawerDescription>Mlynar · currently E2 60 with no masteries.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Cancel</DrawerClose>
                    <Button>Save goal</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const TitleWithBadge = () => (
    <>
        <PlannerBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <div className="flex items-center gap-2">
                        <DrawerTitle>Muelsyse E2 90</DrawerTitle>
                        <Badge variant="warning">Blocked</Badge>
                    </div>
                    <DrawerDescription>Missing 8 × RMA70-12 and 4 × Bipolar Nanoflake for the module stage.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Close</DrawerClose>
                    <Button>Plan farming</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const LongTitleWraps = () => (
    <>
        <PlannerBehind />
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle className="pr-8">Texas the Omertosa — Skill 3 Mastery 3, module DEL-Y</DrawerTitle>
                    <DrawerDescription>Estimated 11 days at your current sanity budget of 240 per day.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel>
                    <div className="divide-y">
                        {[
                            { label: "Sanity cost", value: "2,640" },
                            { label: "LMD cost", value: "480,000" },
                            { label: "Materials", value: "31 items" },
                            { label: "Days remaining", value: "11" },
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
                    <Button>Mark complete</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
