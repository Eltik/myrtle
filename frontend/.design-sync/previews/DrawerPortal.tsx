import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";
import type { ReactNode } from "react";

// DrawerPortal is what lifts the popup out of the DOM subtree it was declared in.
// These stories declare the drawer inside clipping / stacking containers that would
// otherwise crop it, and pass portal options through DrawerPopup's `portalProps`.
const OperatorCard = ({ children }: { children: ReactNode }) => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Roster</span>
            <Badge variant="secondary">318 operators</Badge>
        </div>
        <div className="overflow-hidden rounded-lg border bg-card">
            <div className="flex items-center gap-3 border-b p-3">
                <img alt="Mlynar" className="size-10 rounded-md" height={40} src="https://api.myrtle.moe/api/avatar/char_4064_mlynar" width={40} />
                <div className="flex min-w-0 flex-col">
                    <span className="font-medium text-sm">Mlynar</span>
                    <span className="text-muted-foreground text-xs">6★ Guard · Liberator · E2 90</span>
                </div>
            </div>
            <div className="space-y-2 p-3">
                <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">Trust</span>
                    <span className="font-mono text-sm tabular-nums">200%</span>
                </div>
                <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">Potential</span>
                    <span className="font-mono text-sm tabular-nums">P3</span>
                </div>
                {children}
            </div>
        </div>
    </div>
);

export const EscapesClippingCard = () => (
    <OperatorCard>
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Mlynar · upgrade plan</DrawerTitle>
                    <DrawerDescription>Declared inside an overflow-hidden roster card; the portal lifts the sheet to the document root so nothing clips it.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel className="space-y-2">
                    {[
                        { label: "Skill 3 · M3", value: "8 Bipolar Nanoflake" },
                        { label: "Module GUA-X", value: "4 Crystalline Circuit" },
                        { label: "LMD remaining", value: "1,286,000" },
                    ].map((row) => (
                        <div key={row.label} className="flex items-center justify-between rounded-lg border p-3">
                            <span className="text-muted-foreground text-sm">{row.label}</span>
                            <span className="font-mono text-sm tabular-nums">{row.value}</span>
                        </div>
                    ))}
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Cancel</DrawerClose>
                    <Button>Add to plan</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </OperatorCard>
);

export const SidePanelFromCard = () => (
    <OperatorCard>
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Skill 3 — Reflection</DrawerTitle>
                    <DrawerDescription>Mastery 3 · 40 SP · 20s duration</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel>
                    <div className="divide-y">
                        {[
                            { label: "ATK multiplier", value: "460%" },
                            { label: "Attack interval", value: "1.6s" },
                            { label: "Targets", value: "3" },
                            { label: "True damage", value: "Yes" },
                            { label: "DPS vs 1000 DEF", value: "4,812" },
                        ].map((row) => (
                            <div key={row.label} className="flex items-center justify-between py-2">
                                <span className="text-muted-foreground text-sm">{row.label}</span>
                                <span className="font-mono text-sm tabular-nums">{row.value}</span>
                            </div>
                        ))}
                    </div>
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Close</DrawerClose>
                    <Button>Compare operators</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </OperatorCard>
);

export const KeepMountedPortal = () => (
    <OperatorCard>
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" portalProps={{ keepMounted: true }} showBar variant="inset">
                <DrawerHeader>
                    <DrawerTitle>Trust farming reminder</DrawerTitle>
                    <DrawerDescription>portalProps keepMounted keeps the sheet in the DOM while it is closed, so a long recipe list does not re-mount on every open.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Not now</DrawerClose>
                    <Button>Set reminder</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </OperatorCard>
);
