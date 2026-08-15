import { Badge, Drawer, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";
import { CompassIcon, DownloadIcon, ExternalLinkIcon, HeartIcon, LayersIcon, ListChecksIcon, LogOutIcon, PinIcon, SettingsIcon, ShareIcon, TrophyIcon, UsersIcon } from "lucide-react";

const SiteBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Operators</span>
            <Badge variant="secondary">231 owned</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-3">
            {["Mlynar", "Skadi", "Muelsyse", "Texas", "Eyjafjalla", "Amiya"].map((op) => (
                <div key={op} className="rounded-lg border bg-card p-3">
                    <div className="font-medium text-sm">{op}</div>
                    <div className="mt-1 font-mono text-muted-foreground text-xs">E2 90 · M3/M3/M3</div>
                </div>
            ))}
        </div>
    </div>
);

export const SiteNavigation = () => (
    <>
        <SiteBehind />
        <Drawer open position="left">
            <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Myrtle</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Navigation</DrawerMenuGroupLabel>
                            <DrawerMenuItem className="bg-accent text-accent-foreground">
                                <UsersIcon />
                                Operators
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <CompassIcon />
                                Stages
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <ListChecksIcon />
                                Tier lists
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <LayersIcon />
                                Depot
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <TrophyIcon />
                                Leaderboard
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Account</DrawerMenuGroupLabel>
                            <DrawerMenuItem>
                                <SettingsIcon />
                                Settings
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <HeartIcon />
                                Donate
                                <ExternalLinkIcon className="ms-auto" />
                            </DrawerMenuItem>
                            <DrawerMenuItem variant="destructive">
                                <LogOutIcon />
                                Sign out
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);

export const ActionSheet = () => (
    <>
        <SiteBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Global Guard Rankings</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuItem>
                            <ShareIcon />
                            Copy share link
                        </DrawerMenuItem>
                        <DrawerMenuItem>
                            <PinIcon />
                            Pin to your profile
                        </DrawerMenuItem>
                        <DrawerMenuItem>
                            <DownloadIcon />
                            Export as image
                        </DrawerMenuItem>
                        <DrawerMenuSeparator />
                        <DrawerMenuItem variant="destructive">
                            <LogOutIcon />
                            Leave this list
                        </DrawerMenuItem>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);
