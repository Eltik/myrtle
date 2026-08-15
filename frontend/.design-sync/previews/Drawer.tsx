import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerMenu, DrawerMenuGroup, DrawerMenuGroupLabel, DrawerMenuItem, DrawerMenuSeparator, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";
import { CompassIcon, ExternalLinkIcon, HeartIcon, LayersIcon, ListChecksIcon, LogOutIcon, SettingsIcon, TriangleAlertIcon, UsersIcon } from "lucide-react";

// A slice of the stage-browser page, so the drawer reads over real product
// chrome instead of a blank sheet.
const StageBrowserBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-start justify-between gap-3">
            <div className="flex flex-col gap-1">
                <span className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Chapter 8</span>
                <span className="font-heading font-semibold text-xl">Roaring Flare</span>
            </div>
            <Badge variant="secondary">14 stages</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
            {[
                { stage: "8-4", name: "Ashen Gale", sanity: 18, drop: "Polyester Pack" },
                { stage: "8-9", name: "Cinder Wall", sanity: 21, drop: "Manganese Ore" },
                { stage: "8-14", name: "Roaring Flare", sanity: 21, drop: "RMA70-12" },
                { stage: "S8-2", name: "Scorched Path", sanity: 18, drop: "Grindstone" },
                { stage: "8-16", name: "Ashen Sky", sanity: 21, drop: "Orirock Cluster" },
                { stage: "S8-4", name: "Last Ember", sanity: 21, drop: "Sugar Pack" },
            ].map((s) => (
                <div key={s.stage} className="flex items-center justify-between gap-3 rounded-lg border bg-card p-3">
                    <div className="flex min-w-0 flex-col gap-1">
                        <span className="font-medium text-sm">
                            <span className="font-mono">{s.stage}</span> · {s.name}
                        </span>
                        <span className="truncate text-muted-foreground text-xs">{s.drop}</span>
                    </div>
                    <span className="font-mono text-muted-foreground text-xs tabular-nums">{s.sanity} sanity</span>
                </div>
            ))}
        </div>
    </div>
);

const RosterBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Roster</span>
            <Badge variant="secondary">231 owned</Badge>
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
            {[
                { id: "char_4064_mlynar", name: "Mlynar", sub: "E2 90 · M3 / M3 / M3" },
                { id: "char_263_skadi", name: "Skadi", sub: "E2 90 · M3 / M6 / M6" },
                { id: "char_1028_texas2", name: "Texas the Omertosa", sub: "E2 80 · M3 / M3 / M6" },
                { id: "char_180_amgoat", name: "Eyjafjalla", sub: "E2 90 · M3 / M3 / M3" },
                { id: "char_4045_heidi", name: "Heidi", sub: "E2 70 · M3 / M6 / —" },
                { id: "char_002_amiya", name: "Amiya", sub: "E2 90 · M3 / M3 / M3" },
            ].map((op) => (
                <div key={op.id} className="flex items-center gap-3 rounded-lg border bg-card p-3">
                    <img alt={op.name} className="size-10 shrink-0 rounded-sm object-cover" src={`https://api.myrtle.moe/api/avatar/${op.id}`} />
                    <div className="flex min-w-0 flex-col gap-1">
                        <span className="truncate font-medium text-sm">{op.name}</span>
                        <span className="truncate font-mono text-muted-foreground text-xs">{op.sub}</span>
                    </div>
                </div>
            ))}
        </div>
    </div>
);

export const BottomSheet = () => (
    <>
        <StageBrowserBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Farming route for RMA70-12</DrawerTitle>
                    <DrawerDescription>Sorted by sanity per craft, counting byproducts from the Workshop.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel className="space-y-3">
                    {[
                        { stage: "8-14", rate: "18.4%", cost: "114 sanity / craft" },
                        { stage: "S8-2", rate: "12.1%", cost: "148 sanity / craft" },
                        { stage: "4-8", rate: "9.6%", cost: "163 sanity / craft" },
                    ].map((r) => (
                        <div key={r.stage} className="flex items-center justify-between gap-3 rounded-lg border p-3">
                            <span className="font-medium font-mono text-sm">{r.stage}</span>
                            <span className="text-muted-foreground text-xs">drop {r.rate}</span>
                            <span className="font-mono text-xs tabular-nums">{r.cost}</span>
                        </div>
                    ))}
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Not now</DrawerClose>
                    <Button>Add to plan</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const NavigationLeft = () => (
    <>
        <RosterBehind />
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

export const DetailPanelRight = () => (
    <>
        <RosterBehind />
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <div className="flex items-center gap-3">
                        <img alt="Mlynar" className="size-12 shrink-0 rounded-sm object-cover" src="https://api.myrtle.moe/api/avatar/char_4064_mlynar" />
                        <div className="flex min-w-0 flex-col gap-1">
                            <DrawerTitle>Mlynar</DrawerTitle>
                            <DrawerDescription>6★ Guard · Soldier · Sarkaz</DrawerDescription>
                        </div>
                    </div>
                </DrawerHeader>
                <DrawerPanel className="space-y-4">
                    <div className="flex flex-wrap gap-2">
                        <Badge variant="outline">Nuker</Badge>
                        <Badge variant="outline">Survival</Badge>
                        <Badge variant="outline">DPS</Badge>
                        <Badge variant="success">Owned</Badge>
                    </div>
                    <div className="divide-y">
                        {[
                            { label: "Promotion", value: "E2 90" },
                            { label: "Trust", value: "200%" },
                            { label: "Skill masteries", value: "M3 / M3 / M3" },
                            { label: "Module", value: "GUA-Y · Stage 3" },
                            { label: "Potential", value: "6" },
                            { label: "ATK / DEF", value: "1,050 / 505" },
                            { label: "Deploy cost", value: "26 DP" },
                        ].map((row) => (
                            <div key={row.label} className="flex items-center justify-between py-2">
                                <span className="text-muted-foreground text-sm">{row.label}</span>
                                <span className="font-mono text-sm tabular-nums">{row.value}</span>
                            </div>
                        ))}
                    </div>
                    <div className="rounded-lg border bg-muted/40 p-3">
                        <div className="font-medium text-sm">Used in 6 of your 14 plans</div>
                        <div className="mt-1 text-muted-foreground text-xs">Most recently: Chapter 8 clear route, CE-6 auto-deploy</div>
                    </div>
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Close</DrawerClose>
                    <Button>Open full profile</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const NoticeTop = () => (
    <>
        <StageBrowserBehind />
        <Drawer open position="top">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <div className="flex items-center gap-3">
                        <span className="inline-flex size-9 shrink-0 items-center justify-center rounded-full bg-warning/12 text-warning-foreground">
                            <TriangleAlertIcon />
                        </span>
                        <div className="flex min-w-0 flex-col gap-1">
                            <DrawerTitle>Drop rates are 3 days stale</DrawerTitle>
                            <DrawerDescription>Penguin Stats last responded on 12 May. Efficiency numbers may be off for Chapter 8.</DrawerDescription>
                        </div>
                    </div>
                </DrawerHeader>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Dismiss</DrawerClose>
                    <Button>Retry sync</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
