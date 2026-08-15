import { Badge, Drawer, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";
import { BarChart3Icon, DownloadIcon, ExternalLinkIcon, PinIcon, ShareIcon, Trash2Icon, UserIcon, WrenchIcon } from "lucide-react";

const ProfileBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Doctor Kal'tsit</span>
            <Badge variant="secondary">UID 10238471</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-3">
            {[
                { label: "Total score", value: "18,402" },
                { label: "Operators", value: "231" },
                { label: "Skins", value: "88" },
            ].map((s) => (
                <div key={s.label} className="rounded-lg border bg-card p-3">
                    <div className="text-muted-foreground text-xs">{s.label}</div>
                    <div className="mt-1 font-mono font-semibold text-2xl tabular-nums">{s.value}</div>
                </div>
            ))}
        </div>
    </div>
);

export const NavigationItems = () => (
    <>
        <ProfileBehind />
        <Drawer open position="left">
            <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Your profile</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Profile</DrawerMenuGroupLabel>
                            <DrawerMenuItem className="bg-accent text-accent-foreground">
                                <UserIcon />
                                Overview
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <BarChart3Icon />
                                Statistics
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <WrenchIcon />
                                Base layout
                            </DrawerMenuItem>
                            <DrawerMenuItem>
                                <ExternalLinkIcon />
                                Open public page
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);

export const WithDestructive = () => (
    <>
        <ProfileBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Sniper Rankings — CN meta</DrawerTitle>
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
                        <DrawerMenuSeparator />
                        <DrawerMenuItem variant="destructive">
                            <Trash2Icon />
                            Delete tier list
                        </DrawerMenuItem>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);

export const DisabledItems = () => (
    <>
        <ProfileBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Export roster</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuItem>
                            <DownloadIcon />
                            Download as CSV
                        </DrawerMenuItem>
                        <DrawerMenuItem disabled>
                            <DownloadIcon />
                            Download as PNG — needs a public profile
                        </DrawerMenuItem>
                        <DrawerMenuItem disabled>
                            <ShareIcon />
                            Share to Discord — link your account first
                        </DrawerMenuItem>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);
