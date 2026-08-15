import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";
import { CompassIcon, LayersIcon, ListChecksIcon, SettingsIcon, UsersIcon } from "lucide-react";

const AccountBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Account</span>
            <Badge variant="secondary">EN · Doctor Kal'tsit</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
            {[
                { label: "Linked since", value: "14 Jan 2024" },
                { label: "Last sync", value: "6 minutes ago" },
                { label: "Public profile", value: "On" },
                { label: "Saved plans", value: "14" },
            ].map((r) => (
                <div key={r.label} className="flex items-center justify-between rounded-lg border bg-card p-3">
                    <span className="text-muted-foreground text-sm">{r.label}</span>
                    <span className="font-mono text-sm">{r.value}</span>
                </div>
            ))}
        </div>
    </div>
);

export const FooterCancel = () => (
    <>
        <AccountBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Unlink your EN account?</DrawerTitle>
                    <DrawerDescription>Your roster of 231 operators, your depot and 14 saved plans stop syncing. Nothing is deleted.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Keep linked</DrawerClose>
                    <Button variant="destructive">Unlink account</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const CloseButtonInPopup = () => (
    <>
        <AccountBehind />
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Sync log</DrawerTitle>
                    <DrawerDescription>The built-in close button is a DrawerClose rendered as a ghost icon button.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel>
                    <div className="divide-y">
                        {[
                            { when: "12 May 21:04", what: "Depot · 187 items" },
                            { when: "12 May 09:12", what: "Roster · 231 operators" },
                            { when: "11 May 21:50", what: "Depot · 184 items" },
                            { when: "11 May 08:03", what: "Roster · 230 operators" },
                        ].map((l) => (
                            <div key={l.when} className="flex items-center justify-between py-2">
                                <span className="text-sm">{l.what}</span>
                                <span className="font-mono text-muted-foreground text-xs">{l.when}</span>
                            </div>
                        ))}
                    </div>
                </DrawerPanel>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Done</DrawerClose>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const ClosingMenuItems = () => (
    <>
        <AccountBehind />
        <Drawer open position="left">
            <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Myrtle</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Navigation</DrawerMenuGroupLabel>
                            <DrawerMenuItem render={<DrawerClose />}>
                                <UsersIcon />
                                Operators
                            </DrawerMenuItem>
                            <DrawerMenuItem render={<DrawerClose />}>
                                <CompassIcon />
                                Stages
                            </DrawerMenuItem>
                            <DrawerMenuItem render={<DrawerClose />}>
                                <ListChecksIcon />
                                Tier lists
                            </DrawerMenuItem>
                            <DrawerMenuItem render={<DrawerClose />}>
                                <LayersIcon />
                                Depot
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Account</DrawerMenuGroupLabel>
                            <DrawerMenuItem render={<DrawerClose />}>
                                <SettingsIcon />
                                Settings
                            </DrawerMenuItem>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);
