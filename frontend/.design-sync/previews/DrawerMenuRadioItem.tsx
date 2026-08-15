import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuRadioGroup, DrawerMenuRadioItem, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";

const PlanBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Mlynar</span>
            <Badge variant="secondary">E2 60 today</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-3">
            {[
                { label: "Sanity to goal", value: "2,640" },
                { label: "LMD to goal", value: "480k" },
                { label: "Days", value: "11" },
            ].map((s) => (
                <div key={s.label} className="rounded-lg border bg-card p-3">
                    <div className="text-muted-foreground text-xs">{s.label}</div>
                    <div className="mt-1 font-mono font-semibold text-2xl tabular-nums">{s.value}</div>
                </div>
            ))}
        </div>
    </div>
);

export const TargetPromotion = () => (
    <>
        <PlanBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Target promotion</DrawerTitle>
                    <DrawerDescription>The selected row carries the check glyph; the rest stay blank.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuRadioGroup defaultValue="e2-90">
                            <DrawerMenuRadioItem value="e1-55">E1 55 — 480 sanity</DrawerMenuRadioItem>
                            <DrawerMenuRadioItem value="e2-60">E2 60 — 1,120 sanity</DrawerMenuRadioItem>
                            <DrawerMenuRadioItem value="e2-90">E2 90 — 2,640 sanity</DrawerMenuRadioItem>
                        </DrawerMenuRadioGroup>
                    </DrawerMenu>
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Cancel</DrawerClose>
                    <Button>Save goal</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const WithDisabledOption = () => (
    <>
        <PlanBehind />
        <Drawer open position="left">
            <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Module</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Target stage</DrawerMenuGroupLabel>
                            <DrawerMenuRadioGroup defaultValue="stage2">
                                <DrawerMenuRadioItem value="none">No module</DrawerMenuRadioItem>
                                <DrawerMenuRadioItem value="stage1">GUA-Y stage 1</DrawerMenuRadioItem>
                                <DrawerMenuRadioItem value="stage2">GUA-Y stage 2</DrawerMenuRadioItem>
                                <DrawerMenuRadioItem disabled value="stage3">
                                    GUA-Y stage 3 — needs trust 100
                                </DrawerMenuRadioItem>
                            </DrawerMenuRadioGroup>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);
