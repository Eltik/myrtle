import { Badge, Button, Drawer, DrawerClose, DrawerFooter, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuRadioGroup, DrawerMenuRadioItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";

const SortedListBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Tier lists</span>
            <Badge variant="secondary">Sorted by newest</Badge>
        </div>
        <div className="space-y-2">
            {[
                { name: "Global Guard Rankings", meta: "312 votes · 3 days ago" },
                { name: "Sniper Rankings — CN meta", meta: "58 votes · 2 weeks ago" },
                { name: "Best base skills", meta: "204 votes · 1 month ago" },
            ].map((l) => (
                <div key={l.name} className="flex items-center justify-between gap-3 rounded-lg border bg-card p-3">
                    <span className="font-medium text-sm">{l.name}</span>
                    <span className="text-muted-foreground text-xs">{l.meta}</span>
                </div>
            ))}
        </div>
    </div>
);

export const SortOrder = () => (
    <>
        <SortedListBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Sort tier lists</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuRadioGroup defaultValue="recent">
                            <DrawerMenuRadioItem value="recent">Newest first</DrawerMenuRadioItem>
                            <DrawerMenuRadioItem value="votes">Most voted</DrawerMenuRadioItem>
                            <DrawerMenuRadioItem value="updated">Recently updated</DrawerMenuRadioItem>
                            <DrawerMenuRadioItem value="alpha">Alphabetical</DrawerMenuRadioItem>
                        </DrawerMenuRadioGroup>
                    </DrawerMenu>
                </DrawerPanel>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Done</DrawerClose>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const ServerSwitch = () => (
    <>
        <SortedListBehind />
        <Drawer open position="left">
            <DrawerPopup className="w-70 max-w-[calc(100vw-3rem)]" showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Game data</DrawerTitle>
                </DrawerHeader>
                <DrawerPanel scrollFade={false}>
                    <DrawerMenu>
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Server</DrawerMenuGroupLabel>
                            <DrawerMenuRadioGroup defaultValue="en">
                                <DrawerMenuRadioItem value="en">EN — Yostar</DrawerMenuRadioItem>
                                <DrawerMenuRadioItem value="cn">CN — Hypergryph</DrawerMenuRadioItem>
                                <DrawerMenuRadioItem value="jp">JP — Yostar</DrawerMenuRadioItem>
                                <DrawerMenuRadioItem disabled value="tw">
                                    TW — discontinued
                                </DrawerMenuRadioItem>
                            </DrawerMenuRadioGroup>
                        </DrawerMenuGroup>
                        <DrawerMenuSeparator />
                        <DrawerMenuGroup>
                            <DrawerMenuGroupLabel>Rarity floor</DrawerMenuGroupLabel>
                            <DrawerMenuRadioGroup defaultValue="6">
                                <DrawerMenuRadioItem value="6">6★ only</DrawerMenuRadioItem>
                                <DrawerMenuRadioItem value="5">5★ and up</DrawerMenuRadioItem>
                                <DrawerMenuRadioItem value="all">All rarities</DrawerMenuRadioItem>
                            </DrawerMenuRadioGroup>
                        </DrawerMenuGroup>
                    </DrawerMenu>
                </DrawerPanel>
            </DrawerPopup>
        </Drawer>
    </>
);
