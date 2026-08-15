import { Button, Frame, FrameDescription, FrameFooter, FrameHeader, FramePanel, FrameTitle } from "frontend";

export const WithActions = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Publish tier list</FrameTitle>
                <FrameDescription>Anyone with the link can view it once published.</FrameDescription>
            </FrameHeader>
            <FrameFooter className="flex items-center justify-between border-t">
                <span className="text-muted-foreground text-sm">Draft saved 2 min ago</span>
                <div className="flex gap-2">
                    <Button size="sm" variant="outline">
                        Discard
                    </Button>
                    <Button size="sm">Publish</Button>
                </div>
            </FrameFooter>
        </FramePanel>
    </Frame>
);

export const TotalsRow = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Planner queue</FrameTitle>
                <FrameDescription>3 operators · 42 days at 240 sanity/day</FrameDescription>
            </FrameHeader>
            <div className="border-t px-5 py-4 text-muted-foreground text-sm">Mlynar, Muelsyse and Texas the Omertosa are queued in that order.</div>
            <FrameFooter className="flex items-center justify-between border-t">
                <span className="text-muted-foreground text-sm">Total sanity</span>
                <span className="font-mono font-medium text-sm tabular-nums">4,655</span>
            </FrameFooter>
        </FramePanel>
    </Frame>
);

export const FooterOnly = () => (
    <Frame className="max-w-sm">
        <FramePanel className="p-0">
            <div className="px-5 py-4 text-muted-foreground text-sm">Bipolar Nanoflake is short by 4 units at your current farming rate.</div>
            <FrameFooter className="border-t text-muted-foreground text-xs">Recomputed whenever your depot syncs.</FrameFooter>
        </FramePanel>
    </Frame>
);
