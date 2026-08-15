import { Badge, Card, CardFrame, CardFrameDescription, CardFrameHeader, CardFrameTitle, CardPanel } from "frontend";

export const SectionTitle = () => (
    <CardFrame className="max-w-sm">
        <CardFrameHeader>
            <CardFrameTitle>Base skills</CardFrameTitle>
            <CardFrameDescription>Applies while stationed in a Trading Post.</CardFrameDescription>
        </CardFrameHeader>
        <Card>
            <CardPanel className="p-4 text-muted-foreground text-sm">
                Order Reception <span className="text-foreground">β</span> — order acquisition efficiency +30%.
            </CardPanel>
        </Card>
    </CardFrame>
);

export const TitleWithCount = () => (
    <CardFrame className="max-w-sm">
        <CardFrameHeader>
            <CardFrameTitle className="flex items-center gap-2">
                Drop table
                <Badge variant="secondary">
                    9
                </Badge>
            </CardFrameTitle>
            <CardFrameDescription>Sampled from 12,480 community runs.</CardFrameDescription>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <span className="text-sm">Polyester Pack</span>
                <span className="font-mono text-muted-foreground text-sm tabular-nums">18.4%</span>
            </CardPanel>
        </Card>
    </CardFrame>
);

export const MonospaceTitle = () => (
    <CardFrame className="max-w-sm">
        <CardFrameHeader>
            <CardFrameTitle className="font-mono">S4-1</CardFrameTitle>
            <CardFrameDescription>Under Tides · 21 sanity · 3 first-clear rewards</CardFrameDescription>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <span className="text-muted-foreground text-sm">Recommended level</span>
                <span className="font-mono text-sm tabular-nums">E2 40</span>
            </CardPanel>
        </Card>
    </CardFrame>
);
