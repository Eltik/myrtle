import { Badge, Button, Card, CardDescription, CardFooter, CardHeader, CardPanel, CardTitle, Progress } from "frontend";

export const StatPanel = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Sanity spent</CardTitle>
            <CardDescription>Last 30 days</CardDescription>
        </CardHeader>
        <CardPanel className="font-mono font-semibold text-3xl tabular-nums">4,812</CardPanel>
    </Card>
);

export const ListPanel = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Recent drops</CardTitle>
            <CardDescription>1-7 · 6 runs</CardDescription>
        </CardHeader>
        <CardPanel className="flex flex-col gap-2 text-sm">
            {[
                { name: "Orirock Cube", qty: "12" },
                { name: "Damaged Device", qty: "3" },
                { name: "Sugar Substitute", qty: "2" },
            ].map((drop) => (
                <div className="flex items-center justify-between" key={drop.name}>
                    <span>{drop.name}</span>
                    <span className="font-mono text-muted-foreground tabular-nums">×{drop.qty}</span>
                </div>
            ))}
        </CardPanel>
    </Card>
);

export const PanelBetweenHeaderAndFooter = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Mastery progress</CardTitle>
            <CardDescription>Mlynar · Fervent Blade S3</CardDescription>
        </CardHeader>
        <CardPanel className="flex flex-col gap-3">
            <Progress value={62} />
            <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">M2 → M3</span>
                <span className="font-mono tabular-nums">62%</span>
            </div>
        </CardPanel>
        <CardFooter className="justify-between gap-2">
            <Badge variant="info">14h 40m left</Badge>
            <Button size="sm" variant="outline">
                Adjust plan
            </Button>
        </CardFooter>
    </Card>
);
