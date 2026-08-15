import { Badge, Card, CardContent, CardDescription, CardHeader, CardTitle } from "frontend";

export const StageBriefing = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle className="font-mono">1-7</CardTitle>
            <CardDescription>Ashes to Ashes · 6 sanity · best LMD-per-sanity in Chapter 1</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-2 text-sm">
            <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Enemies</span>
                <span className="font-mono tabular-nums">27</span>
            </div>
            <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Deployment slots</span>
                <span className="font-mono tabular-nums">6</span>
            </div>
        </CardContent>
    </Card>
);

export const TextContent = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Muelsyse</CardTitle>
            <CardDescription>6★ Vanguard · Rhine Lab</CardDescription>
        </CardHeader>
        <CardContent className="text-muted-foreground text-sm">
            Director of the Ecological Section. Copies the stats of a deployed ally to create an elemental duplicate, then redeploys it wherever the field needs holding.
        </CardContent>
    </Card>
);

export const TagContent = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Recruitment result</CardTitle>
            <CardDescription>9-hour timer · guarantees a 5★ or better</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-1.5">
            <Badge variant="outline">Defense</Badge>
            <Badge variant="outline">Survival</Badge>
            <Badge variant="outline">DP-Recovery</Badge>
            <Badge variant="warning">Senior Operator</Badge>
        </CardContent>
    </Card>
);
