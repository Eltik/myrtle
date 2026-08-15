import { Badge, Button, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHeader, DrawerPanel, DrawerPopup, DrawerTitle } from "frontend";

const TierListBehind = () => (
    <div className="min-h-screen space-y-4 p-6">
        <div className="flex items-center justify-between gap-3">
            <span className="font-heading font-semibold text-xl">Global Guard Rankings</span>
            <Badge variant="secondary">148 placements</Badge>
        </div>
        <div className="space-y-2">
            {[
                { tier: "S+", ops: "Mlynar · Surtr · Thorns" },
                { tier: "S", ops: "Blaze · Irene · Hoederer" },
                { tier: "A", ops: "SilverAsh · Ch'en · Specter" },
                { tier: "B", ops: "Broca · Estelle · Popukar" },
            ].map((t) => (
                <div key={t.tier} className="flex items-center gap-3 rounded-lg border bg-card p-3">
                    <span className="w-11 shrink-0 text-center font-mono font-semibold text-sm">{t.tier}</span>
                    <span className="truncate text-muted-foreground text-sm">{t.ops}</span>
                </div>
            ))}
        </div>
    </div>
);

export const DefaultFooter = () => (
    <>
        <TierListBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Publish this tier list?</DrawerTitle>
                    <DrawerDescription>Published lists appear in community rankings and can be voted on by any signed-in doctor.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Keep private</DrawerClose>
                    <Button>Publish list</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const BareFooter = () => (
    <>
        <TierListBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Saved as draft</DrawerTitle>
                    <DrawerDescription>7 placements were moved since your last save. Drafts are kept for 30 days.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter variant="bare">
                    <DrawerClose render={<Button variant="ghost" />}>Dismiss</DrawerClose>
                    <Button>Continue editing</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const DestructiveActions = () => (
    <>
        <TierListBehind />
        <Drawer open position="bottom">
            <DrawerPopup className="mx-auto max-w-lg" showBar>
                <DrawerHeader>
                    <DrawerTitle>Delete “Global Guard Rankings”?</DrawerTitle>
                    <DrawerDescription>All 148 placements, 6 tiers and 312 community votes are removed. Anyone with the link will see a 404 page.</DrawerDescription>
                </DrawerHeader>
                <DrawerFooter>
                    <DrawerClose render={<Button variant="outline" />}>Cancel</DrawerClose>
                    <Button variant="destructive">Delete list</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);

export const StackedInSidePanel = () => (
    <>
        <TierListBehind />
        <Drawer open position="right">
            <DrawerPopup showCloseButton>
                <DrawerHeader>
                    <DrawerTitle>Mlynar — S+ tier</DrawerTitle>
                    <DrawerDescription>Placed by 288 of 312 voters in the top tier.</DrawerDescription>
                </DrawerHeader>
                <DrawerPanel>
                    <div className="divide-y">
                        {[
                            { label: "S+ votes", value: "288" },
                            { label: "S votes", value: "19" },
                            { label: "A votes", value: "5" },
                            { label: "Consensus", value: "92.3%" },
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
                    <Button variant="destructive">Remove placement</Button>
                </DrawerFooter>
            </DrawerPopup>
        </Drawer>
    </>
);
