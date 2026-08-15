import { Badge, Drawer, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";
import { BellIcon, BoxesIcon, CalendarIcon, CompassIcon, FlaskConicalIcon, GaugeIcon, MapIcon, PackageIcon, SettingsIcon, SwordsIcon, UsersIcon } from "lucide-react";

const ToolsBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Tools</span>
            <Badge variant="secondary">9 calculators</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
            {["Sanity planner", "Base optimizer", "Recruitment", "Gacha tracker"].map((t) => (
                <div key={t} className="rounded-lg border bg-card p-3">
                    <div className="font-medium text-sm">{t}</div>
                    <div className="mt-1 text-muted-foreground text-xs">Updated for the 12 May patch</div>
                </div>
            ))}
        </div>
    </div>
);

export const ThreeGroups = () => (
    <>
        <ToolsBehind />
        <Drawer open position="left">
            <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Browse</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Game data</DrawerMenuGroupLabel>
                            <DrawerMenuItem>
                                <UsersIcon />
                                Operators
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <SwordsIcon />
                                Enemies
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <MapIcon />
                                Stages
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Tools</DrawerMenuGroupLabel>
                            <DrawerMenuItem className="bg-accent text-accent-foreground">
                                <GaugeIcon />
                                Sanity planner
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <FlaskConicalIcon />
                                Base optimizer
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <PackageIcon />
                                Depot scanner
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Preferences</DrawerMenuGroupLabel>
                            <DrawerMenuItem>
                                <SettingsIcon />
                                Settings
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <BellIcon />
                                Notifications
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);

export const TwoGroupsInSheet = () => (
    <>
        <ToolsBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Jump to</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Recently viewed</DrawerMenuGroupLabel>
                            <DrawerMenuItem>
                                <CompassIcon />
                                Chapter 8 — Roaring Flare
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <BoxesIcon />
                                RMA70-12
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Events</DrawerMenuGroupLabel>
                            <DrawerMenuItem>
                                <CalendarIcon />
                                Ideal City — ends in 6 days
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <CalendarIcon />
                                Annihilation reset — 2 days
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);
