import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";
import { SparklesIcon } from "lucide-react";

const GachaBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Gacha history</span>
            <Badge variant="secondary">1,284 pulls</Badge>
        </div>
        <div className="space-y-2">
            {[
                { banner: "Ideal City", pulls: 120, six: 3 },
                { banner: "Lone Trail", pulls: 90, six: 2 },
                { banner: "Under Tides", pulls: 60, six: 1 },
                { banner: "Standard", pulls: 300, six: 4 },
            ].map((b) => (
                <div key={b.banner} className="flex items-center justify-between gap-3 rounded-lg border bg-card p-3">
                    <span className="font-medium text-sm">{b.banner}</span>
                    <span className="font-mono text-muted-foreground text-xs tabular-nums">
                        {b.pulls} pulls · {b.six} × 6★
                    </span>
                </div>
            ))}
        </div>
    </div>
);

export const TitleAndDescription = () => (
    <>
        <GachaBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Pity counter</DrawerTitle>
                    <DrawerDescription>You are 68 pulls into the Ideal City banner. The 6★ rate starts climbing at 50.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Close</DrawerClose>
                    <Button>Log a pull</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const WithLeadingIcon = () => (
    <>
        <GachaBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <div className="flex items-center gap-3">
                        <span className="inline-flex size-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary">
                            <SparklesIcon />
                        </span>
                        <div className="flex min-w-0 flex-col gap-1">
                            <DrawerTitle>Mlynar joined your roster</DrawerTitle>
                            <DrawerDescription>Pulled on the Ideal City banner at 68 pity · 6★ Guard · Soldier</DrawerDescription>
                        </div>
                    </div>
                </DrawerHeader>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Later</DrawerClose>
                    <Button>Set upgrade plan</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const HeaderWithBadgeRow = () => (
    <>
        <GachaBehind />
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Ideal City</DrawerTitle>
                    <DrawerDescription>Limited banner · 12 May – 26 May</DrawerDescription>
                    <div className="flex flex-wrap gap-2">
                        <Badge variant="outline">120 pulls</Badge>
                        <Badge variant="outline">3 × 6★</Badge>
                        <Badge variant="success">Above average</Badge>
                    </div>
                </DrawerHeader>
                <DrawerPanel>
                    <div className="divide-y">
                        {[
                            { label: "6★ rate", value: "2.50%" },
                            { label: "Your rate", value: "3.33%" },
                            { label: "Orundum spent", value: "72,000" },
                            { label: "Current pity", value: "68" },
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
                    <Button>Export as CSV</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
