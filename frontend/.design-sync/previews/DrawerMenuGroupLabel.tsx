import { Badge, Drawer, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";
import { StarIcon } from "lucide-react";

const OperatorListBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Support units</span>
            <Badge variant="secondary">3 slots</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-3">
            {["Mlynar", "Skadi", "Muelsyse"].map((op) => (
                <div key={op} className="rounded-lg border bg-card p-3">
                    <div className="font-medium text-sm">{op}</div>
                    <div className="mt-1 font-mono text-muted-foreground text-xs">Lent 412 times</div>
                </div>
            ))}
        </div>
    </div>
);

export const RarityHeaders = () => (
    <>
        <OperatorListBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Pick a support unit</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>6★ Guard</DrawerMenuGroupLabel>
                            <DrawerMenuItem>
                                <StarIcon />
                                Mlynar — E2 90, M3/M3/M3
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <StarIcon />
                                Thorns — E2 90, M3/M6/M3
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>6★ Sniper</DrawerMenuGroupLabel>
                            <DrawerMenuItem>
                                <StarIcon />
                                Ash — E2 80, M6/M3/M3
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <StarIcon />
                                Rosmontis — E2 90, M3/M3/M6
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);

export const ServerSections = () => (
    <>
        <OperatorListBehind />
        <Drawer open position="left">
            <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Linked accounts</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Global servers</DrawerMenuGroupLabel>
                            <DrawerMenuItem className="bg-accent text-accent-foreground">EN · Doctor Kal'tsit</DrawerMenuItem>
                            <DrawerMenuItem>JP · not linked</DrawerMenuItem>
                            <DrawerMenuItem>KR · not linked</DrawerMenuItem>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Mainland</DrawerMenuGroupLabel>
                            <DrawerMenuItem>CN Bilibili · not linked</DrawerMenuItem>
                            <DrawerMenuItem>CN Official · Dr. Myrtle</DrawerMenuItem>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);
