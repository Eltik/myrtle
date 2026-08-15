import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";

const RecruitmentBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Recruitment</span>
            <Badge variant="secondary">4 slots free</Badge>
        </div>
        <div className="flex flex-wrap gap-2">
            {["Senior Operator", "Defense", "Survival", "DPS", "Healing", "Melee", "Ranged", "Slow", "Nuker"].map((tag) => (
                <Badge key={tag} variant="outline">
                    {tag}
                </Badge>
            ))}
        </div>
        <div className="space-y-2">
            {[
                { combo: "Senior Operator + Defense", result: "Guaranteed 5★ · Nearl" },
                { combo: "Survival + Nuker", result: "Ceobe · Eyjafjalla" },
                { combo: "Slow + Ranged", result: "Podenco · Angelina" },
            ].map((r) => (
                <div key={r.combo} className="flex items-center justify-between gap-3 rounded-lg border bg-card p-3">
                    <span className="font-medium text-sm">{r.combo}</span>
                    <span className="truncate text-muted-foreground text-xs">{r.result}</span>
                </div>
            ))}
        </div>
    </div>
);

export const SupportingCopy = () => (
    <>
        <RecruitmentBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Senior Operator + Defense</DrawerTitle>
                    <DrawerDescription>This tag pair guarantees a 5★. Set the recruitment timer to 9 hours to keep the 4★ pool out of the roll.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Close</DrawerClose>
                    <Button>Start 9:00 recruit</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const WithInlineEmphasis = () => (
    <>
        <RecruitmentBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Clear this tag combination?</DrawerTitle>
                    <DrawerDescription>
                        <span className="font-medium text-foreground">Senior Operator + Defense</span> is the only guaranteed 5★ pair in this refresh. Clearing it costs an Expedited Plan to roll new tags.
                    </DrawerDescription>
                </DrawerHeader>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Keep tags</DrawerClose>
                    <Button variant="destructive">Refresh tags</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const InSidePanel = () => (
    <>
        <RecruitmentBehind />
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Survival + Nuker</DrawerTitle>
                    <DrawerDescription>11 operators match both tags. Rarity is capped at 5★ unless Top Operator is also present, so a 9-hour timer is wasted here.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel>
                    <div className="divide-y">
                        {[
                            { label: "Ceobe", value: "6★" },
                            { label: "Eyjafjalla", value: "6★" },
                            { label: "Amiya", value: "5★" },
                            { label: "Leonhardt", value: "5★" },
                            { label: "Gitano", value: "4★" },
                        ].map((row) => (
                            <div key={row.label} className="flex items-center justify-between py-2">
                                <span className="text-sm">{row.label}</span>
                                <span className="font-mono text-muted-foreground text-sm">{row.value}</span>
                            </div>
                        ))}
                    </div>
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Close</DrawerClose>
                    <Button>Start 3:50 recruit</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
