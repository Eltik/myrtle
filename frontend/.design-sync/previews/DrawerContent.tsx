import { Badge, Button, Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";
import type { ReactNode } from "react";

// DrawerContent marks a region of the sheet as content rather than drag surface,
// so text inside it can be selected instead of starting a swipe. DrawerHeader,
// DrawerPanel and DrawerFooter opt into it via `allowSelection`.
const PatchNotesBehind = ({ children }: { children?: ReactNode }) => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Patch notes</span>
            <Badge variant="secondary">EN 24.5.1</Badge>
        </div>
        <div className="space-y-2">
            {[
                { title: "Ideal City rerun", body: "Muelsyse rate-up returns for 14 days." },
                { title: "Base rebalance", body: "Trading post order limit raised to 6." },
                { title: "Bug fixes", body: "Fixed Skadi's S3 losing charge on retreat." },
            ].map((n) => (
                <div key={n.title} className="rounded-lg border bg-card p-3">
                    <div className="font-medium text-sm">{n.title}</div>
                    <div className="mt-1 text-muted-foreground text-xs">{n.body}</div>
                </div>
            ))}
        </div>
        {children}
    </div>
);

export const SelectableBody = () => (
    <>
        <PatchNotesBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>EN 24.5.1 — full notes</DrawerTitle>
                    <DrawerDescription>Released 12 May · 1.4 GB download</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel>
                    <DrawerContent className="space-y-3 text-sm leading-relaxed">
                        <p>Muelsyse rate-up returns for fourteen days alongside the Ideal City rerun. Pity carries over from the previous banner and is not reset by the maintenance.</p>
                        <p className="text-muted-foreground">Copy a patch line straight out of the sheet: wrapping the body in DrawerContent means a drag over this text selects it instead of dismissing the drawer.</p>
                    </DrawerContent>
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Close</DrawerClose>
                    <Button>Open changelog</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const SelectableHeader = () => (
    <>
        <PatchNotesBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader allowSelection>
                    <DrawerTitle>Recruitment tag result</DrawerTitle>
                    <DrawerDescription>Defender · Survival · Debuff — copy the tag string into the tag calculator.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel className="space-y-2">
                    {[
                        { tags: "Defender + Survival", result: "Cuora, Gummy" },
                        { tags: "Debuff + Defender", result: "Hung" },
                        { tags: "Survival + Debuff", result: "Meteorite" },
                    ].map((r) => (
                        <div key={r.tags} className="flex items-center justify-between rounded-lg border p-3">
                            <span className="text-muted-foreground text-sm">{r.tags}</span>
                            <span className="text-sm">{r.result}</span>
                        </div>
                    ))}
                </DrawerPanel>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Dismiss</DrawerClose>
                    <Button>Recruit now</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const NonSelectablePanel = () => (
    <>
        <PatchNotesBehind />
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Maintenance window</DrawerTitle>
                    <DrawerDescription>Servers are down for the 24.5.1 rollout.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel allowSelection={false}>
                    <div className="divide-y">
                        {[
                            { label: "Starts", value: "10:00 UTC-7" },
                            { label: "Ends", value: "14:00 UTC-7" },
                            { label: "Compensation", value: "300 Orundum" },
                            { label: "Affected servers", value: "EN · JP" },
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
                    <Button>Remind me</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
