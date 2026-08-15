import { Badge, Button, Card, CardFrame, CardFrameAction, CardFrameDescription, CardFrameHeader, CardFrameTitle, CardPanel } from "frontend";

export const TitleAndDescription = () => (
    <CardFrame className="max-w-md">
        <CardFrameHeader>
            <CardFrameTitle>Recruitment tags</CardFrameTitle>
            <CardFrameDescription>Tag combinations that guarantee a 5★ or better.</CardFrameDescription>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex flex-wrap gap-1.5 p-4">
                <Badge variant="outline">Defense</Badge>
                <Badge variant="outline">Survival</Badge>
                <Badge variant="outline">DP-Recovery</Badge>
                <Badge variant="warning">Senior Operator</Badge>
            </CardPanel>
        </Card>
    </CardFrame>
);

export const TitleOnly = () => (
    <CardFrame className="max-w-sm">
        <CardFrameHeader>
            <CardFrameTitle>Squad — Chapter 8</CardFrameTitle>
        </CardFrameHeader>
        <Card>
            <CardPanel className="p-4 text-muted-foreground text-sm">
                Mlynar, Ines, Eyjafjalla, Skadi the Corrupting Heart, Ling, Texas the Omertosa
            </CardPanel>
        </Card>
    </CardFrame>
);

export const WithAction = () => (
    <CardFrame className="max-w-md">
        <CardFrameHeader>
            <CardFrameTitle>Depot sync</CardFrameTitle>
            <CardFrameDescription>Last imported 2 hours ago from the EN server.</CardFrameDescription>
            <CardFrameAction>
                <Button size="sm" variant="outline">
                    Re-import
                </Button>
            </CardFrameAction>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <span className="text-muted-foreground text-sm">Items tracked</span>
                <span className="font-mono text-sm tabular-nums">248</span>
            </CardPanel>
        </Card>
    </CardFrame>
);
