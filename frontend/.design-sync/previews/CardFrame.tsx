import { Badge, Card, CardFrame, CardFrameAction, CardFrameDescription, CardFrameFooter, CardFrameHeader, CardFrameTitle, CardPanel } from "frontend";

const QUEUE = [
    { name: "Mlynar", goal: "E2 60 → E2 90 · S3M3", sanity: "1,840" },
    { name: "Muelsyse", goal: "E1 55 → E2 40 · S2M3", sanity: "1,275" },
    { name: "Texas the Omertosa", goal: "E2 30 → E2 70 · S3M3", sanity: "1,540" },
];

export const PlannerQueue = () => (
    <CardFrame className="max-w-md">
        <CardFrameHeader>
            <CardFrameTitle>Planner queue</CardFrameTitle>
            <CardFrameDescription>3 operators · 42 days at 24 sanity/day</CardFrameDescription>
            <CardFrameAction>
                <Badge variant="outline">Auto-sync</Badge>
            </CardFrameAction>
        </CardFrameHeader>
        {QUEUE.map((op) => (
            <Card key={op.name}>
                <CardPanel className="flex items-center justify-between gap-4 p-4">
                    <div className="min-w-0">
                        <p className="truncate font-medium text-sm">{op.name}</p>
                        <p className="truncate text-muted-foreground text-xs">{op.goal}</p>
                    </div>
                    <span className="shrink-0 font-mono text-muted-foreground text-xs tabular-nums">{op.sanity}</span>
                </CardPanel>
            </Card>
        ))}
        <CardFrameFooter className="relative flex items-center justify-between">
            <span className="text-muted-foreground text-sm">Total sanity</span>
            <span className="font-mono font-medium text-sm tabular-nums">4,655</span>
        </CardFrameFooter>
    </CardFrame>
);

export const StackedSections = () => (
    <CardFrame className="max-w-md">
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <div>
                    <p className="font-medium text-sm">Sanity potions</p>
                    <p className="text-muted-foreground text-xs">Counted toward the daily budget</p>
                </div>
                <span className="font-mono text-sm tabular-nums">12</span>
            </CardPanel>
        </Card>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <div>
                    <p className="font-medium text-sm">Originium Prime</p>
                    <p className="text-muted-foreground text-xs">Never spent automatically</p>
                </div>
                <span className="font-mono text-sm tabular-nums">4</span>
            </CardPanel>
        </Card>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <div>
                    <p className="font-medium text-sm">Annihilation cap</p>
                    <p className="text-muted-foreground text-xs">Resets Monday 04:00 server time</p>
                </div>
                <span className="font-mono text-sm tabular-nums">1,800</span>
            </CardPanel>
        </Card>
    </CardFrame>
);

export const SingleSection = () => (
    <CardFrame className="max-w-sm">
        <CardFrameHeader>
            <CardFrameTitle>Farming route</CardFrameTitle>
            <CardFrameDescription>Best sanity value for Bipolar Nanoflake</CardFrameDescription>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <div>
                    <p className="font-mono font-medium text-sm">CE-6</p>
                    <p className="text-muted-foreground text-xs">Cargo Escort · 30 sanity</p>
                </div>
                <Badge variant="success">Recommended</Badge>
            </CardPanel>
        </Card>
    </CardFrame>
);
