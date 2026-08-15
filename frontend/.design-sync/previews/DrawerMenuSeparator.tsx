import { Badge, Drawer, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";
import { ArrowUpRightIcon, ClockIcon, DownloadIcon, FlameIcon, PinIcon, ShareIcon, Trash2Icon, ZapIcon } from "lucide-react";

const StageDetailBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">8-14 Roaring Flare</span>
            <Badge variant="secondary">21 sanity</Badge>
        </div>
        <div className="space-y-2">
            {[
                { drop: "RMA70-12", rate: "18.4%" },
                { drop: "Manganese Ore", rate: "24.0%" },
                { drop: "Orirock Cluster", rate: "31.2%" },
            ].map((d) => (
                <div key={d.drop} className="flex items-center justify-between gap-3 rounded-lg border bg-card p-3">
                    <span className="font-medium text-sm">{d.drop}</span>
                    <span className="font-mono text-muted-foreground text-xs tabular-nums">{d.rate}</span>
                </div>
            ))}
        </div>
    </div>
);

export const BetweenActionGroups = () => (
    <>
        <StageDetailBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Stage actions</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuItem>
                            <ZapIcon />
                            Add 5 runs to today's plan
                        </DrawerMenuItem>
                        <DrawerMenuItem>
                            <PinIcon />
                            Pin to sidebar
                        </DrawerMenuItem>
                        <DrawerMenuSeparator />
                        <DrawerMenuItem>
                            <ShareIcon />
                            Copy stage link
                        </DrawerMenuItem>
                        <DrawerMenuItem>
                            <DownloadIcon />
                            Export drop table
                        </DrawerMenuItem>
                        <DrawerMenuSeparator />
                        <DrawerMenuItem variant="destructive">
                            <Trash2Icon />
                            Clear run history
                        </DrawerMenuItem>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);

export const BetweenLabelledGroups = () => (
    <>
        <StageDetailBehind />
        <Drawer open position="left">
            <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Farming</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Best for RMA70-12</DrawerMenuGroupLabel>
                            <DrawerMenuItem className="bg-accent text-accent-foreground">
                                <FlameIcon />
                                8-14 · 114 sanity
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <FlameIcon />
                                S8-2 · 148 sanity
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Recently run</DrawerMenuGroupLabel>
                            <DrawerMenuItem>
                                <ClockIcon />
                                CE-6 · 4 hours ago
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <ClockIcon />
                                1-7 · yesterday
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>External</DrawerMenuGroupLabel>
                            <DrawerMenuItem>
                                <ArrowUpRightIcon />
                                Penguin Stats
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);
