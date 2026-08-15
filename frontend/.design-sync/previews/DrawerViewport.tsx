import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";

// DrawerViewport is the fixed full-screen stage the popup is laid out in — it is
// rendered for you by DrawerPopup, and `position` / `variant` are what change it.
// These stories sweep that axis: bottom stage, side stage, inset stage.
const GachaBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Pull tracker</span>
            <Badge variant="secondary">EN · Standard</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-3">
            {[
                { label: "Pulls since 6★", value: "62" },
                { label: "Current rate", value: "17.0%" },
                { label: "Orundum banked", value: "8,400" },
            ].map((s) => (
                <div key={s.label} className="rounded-lg border bg-card p-3">
                    <div className="text-muted-foreground text-xs">{s.label}</div>
                    <div className="mt-1 font-mono text-lg tabular-nums">{s.value}</div>
                </div>
            ))}
        </div>
        <div className="space-y-2">
            {[
                { name: "Mlynar", rarity: "6★", at: "pull 62" },
                { name: "Skadi", rarity: "6★", at: "pull 41" },
                { name: "Texas", rarity: "5★", at: "pull 38" },
            ].map((p) => (
                <div key={p.name} className="flex items-center justify-between gap-3 rounded-lg border bg-card p-3">
                    <span className="font-medium text-sm">{p.name}</span>
                    <span className="text-muted-foreground text-xs">
                        {p.rarity} · {p.at}
                    </span>
                </div>
            ))}
        </div>
    </div>
);

export const BottomStage = () => (
    <>
        <GachaBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Record a pull</DrawerTitle>
                    <DrawerDescription>The viewport pins the sheet to the bottom of the screen and reserves 3rem of headroom above it.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel className="space-y-2">
                    {[
                        { label: "Banner", value: "Ideal City" },
                        { label: "Pity counter", value: "62" },
                        { label: "Orundum spent", value: "37,200" },
                    ].map((row) => (
                        <div key={row.label} className="flex items-center justify-between rounded-lg border p-3">
                            <span className="text-muted-foreground text-sm">{row.label}</span>
                            <span className="font-mono text-sm tabular-nums">{row.value}</span>
                        </div>
                    ))}
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Cancel</DrawerClose>
                    <Button>Save pull</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const SideStage = () => (
    <>
        <GachaBehind />
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Banner history</DrawerTitle>
                    <DrawerDescription>A right-positioned viewport becomes a flex row aligned to the end, so the panel runs the full height.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel>
                    <div className="divide-y">
                        {[
                            { label: "Ideal City", value: "142 pulls" },
                            { label: "Lone Trail", value: "90 pulls" },
                            { label: "Under Tides", value: "60 pulls" },
                            { label: "Standard", value: "38 pulls" },
                            { label: "Kernel", value: "20 pulls" },
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
                    <Button>Export CSV</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const InsetStage = () => (
    <>
        <GachaBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar variant="inset">
                <DrawerHeader>
                    <DrawerTitle>Reset pity counter</DrawerTitle>
                    <DrawerDescription>The inset viewport adds a gutter on every side, so the sheet floats clear of the screen edges.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Keep 62</DrawerClose>
                    <Button variant="destructive">Reset to 0</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
