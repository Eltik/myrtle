import { Badge, Card, CardFrame, CardFrameDescription, CardFrameHeader, CardFrameTitle, CardPanel } from "frontend";

export const HelperText = () => (
    <CardFrame className="max-w-md">
        <CardFrameHeader>
            <CardFrameTitle>Sanity budget</CardFrameTitle>
            <CardFrameDescription>Assumes 240 natural sanity per day plus one 80-sanity potion.</CardFrameDescription>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <span className="text-sm">Projected completion</span>
                <span className="font-mono text-sm tabular-nums">Nov 12</span>
            </CardPanel>
        </Card>
    </CardFrame>
);

export const WithInlineEmphasis = () => (
    <CardFrame className="max-w-md">
        <CardFrameHeader>
            <CardFrameTitle>Pull history</CardFrameTitle>
            <CardFrameDescription>
                <span className="font-medium text-foreground">312 pulls</span> since your last 6★ · pity at 50
            </CardFrameDescription>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <span className="text-sm">Ines</span>
                <Badge variant="warning">6★</Badge>
            </CardPanel>
        </Card>
    </CardFrame>
);

export const LongDescription = () => (
    <CardFrame className="max-w-md">
        <CardFrameHeader>
            <CardFrameTitle>Module MLY-X</CardFrameTitle>
            <CardFrameDescription>
                Unlocks at E2 40 after clearing 9-16 and the associated module mission. Stage 3 grants an additional trait upgrade on top of the stat line.
            </CardFrameDescription>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <span className="text-muted-foreground text-sm">Data blocks needed</span>
                <span className="font-mono text-sm tabular-nums">18</span>
            </CardPanel>
        </Card>
    </CardFrame>
);
