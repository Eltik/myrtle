import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuSeparator, DrawerMenuTrigger, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";
import { BellIcon, GaugeIcon, PackageIcon, UserIcon } from "lucide-react";

const SettingsBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Settings</span>
            <Badge variant="secondary">EN server</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
            {["Account", "Notifications", "Display", "Data & sync"].map((s) => (
                <div key={s} className="rounded-lg border bg-card p-3">
                    <div className="font-medium text-sm">{s}</div>
                    <div className="mt-1 text-muted-foreground text-xs">Last changed 3 days ago</div>
                </div>
            ))}
        </div>
    </div>
);

export const SubmenuTriggers = () => (
    <>
        <SettingsBehind />
        <Drawer open position="left">
            <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Settings</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Sections</DrawerMenuGroupLabel>
                            <Drawer position="left">
                                <DrawerMenuTrigger>
                                    <UserIcon />
                                    Account
                                </DrawerMenuTrigger>
                                <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                                    <DrawerHeader>
                                        <DrawerTitle>Account</DrawerTitle>
                                    </DrawerHeader>
                                </DrawerPopup>
                            </Drawer>
                            <Drawer position="left">
                                <DrawerMenuTrigger>
                                    <BellIcon />
                                    Notifications
                                </DrawerMenuTrigger>
                                <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                                    <DrawerHeader>
                                        <DrawerTitle>Notifications</DrawerTitle>
                                    </DrawerHeader>
                                </DrawerPopup>
                            </Drawer>
                            <Drawer position="left">
                                <DrawerMenuTrigger>
                                    <PackageIcon />
                                    Depot sync
                                </DrawerMenuTrigger>
                                <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                                    <DrawerHeader>
                                        <DrawerTitle>Depot sync</DrawerTitle>
                                    </DrawerHeader>
                                </DrawerPopup>
                            </Drawer>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Shortcuts</DrawerMenuGroupLabel>
                            <DrawerMenuItem>
                                <GaugeIcon />
                                Sanity budget
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);

export const SheetSubmenus = () => (
    <>
        <SettingsBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Data & sync</DrawerTitle>
                    <DrawerDescription>Each chevron row opens a nested drawer that stacks above this one.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <Drawer position="bottom">
                            <DrawerMenuTrigger>
                                <PackageIcon />
                                Depot import — 187 items
                            </DrawerMenuTrigger>
                            <DrawerPopup className="mx-auto max-w-lg" showBar>
                                <DrawerHeader>
                                    <DrawerTitle>Depot import</DrawerTitle>
                                </DrawerHeader>
                            </DrawerPopup>
                        </Drawer>
                        <Drawer position="bottom">
                            <DrawerMenuTrigger>
                                <GaugeIcon />
                                Sanity budget — 240 per day
                            </DrawerMenuTrigger>
                            <DrawerPopup className="mx-auto max-w-lg" showBar>
                                <DrawerHeader>
                                    <DrawerTitle>Sanity budget</DrawerTitle>
                                </DrawerHeader>
                            </DrawerPopup>
                        </Drawer>
                        <Drawer position="bottom">
                            <DrawerMenuTrigger>
                                <BellIcon />
                                Notifications — 3 enabled
                            </DrawerMenuTrigger>
                            <DrawerPopup className="mx-auto max-w-lg" showBar>
                                <DrawerHeader>
                                    <DrawerTitle>Notifications</DrawerTitle>
                                </DrawerHeader>
                            </DrawerPopup>
                        </Drawer>
                    </DrawerMenu>
                </DrawerPanel>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Done</DrawerClose>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
