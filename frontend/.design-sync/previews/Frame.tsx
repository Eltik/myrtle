import { Badge, Button, Frame, FrameDescription, FrameFooter, FrameHeader, FramePanel, FrameTitle } from "frontend";

export const StackedPanels = () => (
    <Frame className="max-w-md">
        <FramePanel>
            <div className="flex items-center justify-between gap-4">
                <div className="min-w-0">
                    <p className="font-medium text-sm">Roster sync</p>
                    <p className="text-muted-foreground text-xs">Last pulled 14 minutes ago · EN (Yostar)</p>
                </div>
                <Badge variant="success">Healthy</Badge>
            </div>
        </FramePanel>
        <FramePanel>
            <div className="flex items-center justify-between gap-4">
                <div className="min-w-0">
                    <p className="font-medium text-sm">Depot snapshot</p>
                    <p className="text-muted-foreground text-xs">Materials counted toward planner totals</p>
                </div>
                <span className="font-mono text-muted-foreground text-sm tabular-nums">1,284</span>
            </div>
        </FramePanel>
        <FramePanel>
            <div className="flex items-center justify-between gap-4">
                <div className="min-w-0">
                    <p className="font-medium text-sm">Base layout</p>
                    <p className="text-muted-foreground text-xs">Needs a re-scan after the last shift change</p>
                </div>
                <Badge variant="warning">Stale</Badge>
            </div>
        </FramePanel>
    </Frame>
);

export const SinglePanel = () => (
    <Frame className="max-w-sm">
        <FramePanel>
            <p className="font-semibold text-sm">Chapter 8 — Roaring Flare</p>
            <p className="mt-1 text-muted-foreground text-sm">14 stages, 3 challenge modes. Recommended average level E2 40.</p>
        </FramePanel>
    </Frame>
);

export const Sectioned = () => (
    <Frame className="max-w-md">
        <FramePanel className="p-0">
            <FrameHeader>
                <FrameTitle>Sanity budget</FrameTitle>
                <FrameDescription>Applied to every plan you generate.</FrameDescription>
            </FrameHeader>
            <div className="border-t px-5 py-4 text-muted-foreground text-sm">
                Spending <span className="font-medium text-foreground">240 sanity/day</span> clears the Mlynar plan in 42 days.
            </div>
            <FrameFooter className="flex items-center justify-between border-t">
                <span className="text-muted-foreground text-sm">Auto-refresh daily</span>
                <Button size="sm" variant="outline">
                    Recalculate
                </Button>
            </FrameFooter>
        </FramePanel>
    </Frame>
);
