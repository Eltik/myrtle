import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";

const StageTableBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex flex-wrap items-center gap-2">
            <Badge>Chapter 8</Badge>
            <Badge variant="secondary">Roaring Flare</Badge>
            <Badge variant="secondary">14 stages</Badge>
            <Badge variant="secondary">3 challenge modes</Badge>
        </div>
        <table className="w-full text-sm">
            <thead className="text-muted-foreground text-xs">
                <tr className="border-b">
                    <th className="py-2 text-left font-medium">Stage</th>
                    <th className="py-2 text-left font-medium">Sanity</th>
                    <th className="py-2 text-right font-medium">Efficiency</th>
                </tr>
            </thead>
            <tbody className="font-mono tabular-nums">
                {[
                    { stage: "8-14", sanity: "21", eff: "0.94" },
                    { stage: "S8-2", sanity: "18", eff: "0.68" },
                    { stage: "CE-6", sanity: "36", eff: "1.00" },
                    { stage: "1-7", sanity: "6", eff: "0.72" },
                ].map((r) => (
                    <tr key={r.stage} className="border-b">
                        <td className="py-1.5 font-sans">{r.stage}</td>
                        <td className="py-1.5">{r.sanity}</td>
                        <td className="py-1.5 text-right">{r.eff}</td>
                    </tr>
                ))}
            </tbody>
        </table>
    </div>
);

export const OverPageContent = () => (
    <>
        <StageTableBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Add 8-14 to today's plan</DrawerTitle>
                    <DrawerDescription>The backdrop dims and blurs the stage table behind the sheet and swallows outside clicks.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Cancel</DrawerClose>
                    <Button>Add 5 runs</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const OverSidePanel = () => (
    <>
        <StageTableBehind />
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>CE-6 Bounty</DrawerTitle>
                    <DrawerDescription>36 sanity · 10,000 LMD · the LMD benchmark every other stage is scored against.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel>
                    <div className="divide-y">
                        {[
                            { label: "LMD per run", value: "10,000" },
                            { label: "LMD per sanity", value: "277.8" },
                            { label: "Runs today", value: "4" },
                            { label: "Open days", value: "Tue · Thu · Sat" },
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
                    <Button>Log a run</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const UnderInsetSheet = () => (
    <>
        <StageTableBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar variant="inset">
                <DrawerHeader>
                    <DrawerTitle>Clear today's run history?</DrawerTitle>
                    <DrawerDescription>With the inset variant the backdrop still covers the whole screen, so the dim reaches past the sheet's gutters to the viewport edges.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Keep history</DrawerClose>
                    <Button variant="destructive">Clear 14 runs</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
