import { Badge, Button, Card, CardAction, CardDescription, CardFooter, CardHeader, CardPanel, CardTitle } from "frontend";

export const Basic = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Chapter 8 — Roaring Flare</CardTitle>
            <CardDescription>14 stages, 3 challenge modes. First-clear rewards include 2 Chip Catalysts.</CardDescription>
        </CardHeader>
        <CardPanel className="text-muted-foreground text-sm">
            Recommended average level: <span className="font-medium text-foreground">E2 40</span>
        </CardPanel>
    </Card>
);

export const WithAction = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Global tier list</CardTitle>
            <CardDescription>Community-voted rankings, updated weekly.</CardDescription>
            <CardAction>
                <Badge variant="secondary">v12</Badge>
            </CardAction>
        </CardHeader>
        <CardPanel className="text-muted-foreground text-sm">1,284 placements across 6 tiers, contributed by 312 doctors.</CardPanel>
        <CardFooter className="gap-2">
            <Button size="sm">Open list</Button>
            <Button size="sm" variant="outline">
                Version history
            </Button>
        </CardFooter>
    </Card>
);

export const Compact = () => (
    <div className="grid max-w-2xl gap-4 sm:grid-cols-2">
        <Card>
            <CardHeader>
                <CardTitle>Sanity spent</CardTitle>
                <CardDescription>Last 30 days</CardDescription>
            </CardHeader>
            <CardPanel className="font-semibold text-3xl tabular-nums">4,812</CardPanel>
        </Card>
        <Card>
            <CardHeader>
                <CardTitle>Operators owned</CardTitle>
                <CardDescription>Across all servers</CardDescription>
            </CardHeader>
            <CardPanel className="font-semibold text-3xl tabular-nums">231</CardPanel>
        </Card>
    </div>
);
