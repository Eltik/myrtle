import { Button, Card, CardFrame, CardFrameDescription, CardFrameFooter, CardFrameHeader, CardFrameTitle, CardPanel } from "frontend";

export const TotalsFooter = () => (
    <CardFrame className="max-w-md">
        <CardFrameHeader>
            <CardFrameTitle>Upgrade cost</CardFrameTitle>
            <CardFrameDescription>Skadi the Corrupting Heart · E2 0 → E2 90</CardFrameDescription>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <span className="text-sm">LMD</span>
                <span className="font-mono text-sm tabular-nums">1,116,000</span>
            </CardPanel>
        </Card>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <span className="text-sm">EXP cards</span>
                <span className="font-mono text-sm tabular-nums">1,048,320</span>
            </CardPanel>
        </Card>
        <CardFrameFooter className="relative flex items-center justify-between">
            <span className="text-muted-foreground text-sm">Estimated sanity</span>
            <span className="font-mono font-medium text-sm tabular-nums">2,940</span>
        </CardFrameFooter>
    </CardFrame>
);

export const ActionsFooter = () => (
    <CardFrame className="max-w-md">
        <CardFrameHeader>
            <CardFrameTitle>Export depot</CardFrameTitle>
            <CardFrameDescription>28 of 44 fields selected.</CardFrameDescription>
        </CardFrameHeader>
        <Card>
            <CardPanel className="p-4 text-muted-foreground text-sm">
                Output is a CSV compatible with Penguin Stats and the Krooster importer.
            </CardPanel>
        </Card>
        <CardFrameFooter className="relative flex gap-2">
            <Button size="sm">Download CSV</Button>
            <Button size="sm" variant="ghost">
                Copy to clipboard
            </Button>
        </CardFrameFooter>
    </CardFrame>
);

export const MetaFooter = () => (
    <CardFrame className="max-w-sm">
        <Card>
            <CardPanel className="p-4">
                <p className="font-medium text-sm">Chapter 8 — Roaring Flare</p>
                <p className="mt-1 text-muted-foreground text-xs">14 stages · 3 challenge modes</p>
            </CardPanel>
        </Card>
        <CardFrameFooter className="relative text-muted-foreground text-xs">Progress synced from your EN account · 12/14 cleared</CardFrameFooter>
    </CardFrame>
);
