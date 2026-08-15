import { Badge, Button, Card, CardDescription, CardFooter, CardHeader, CardPanel, CardTitle } from "frontend";

export const ActionsFooter = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Global tier list</CardTitle>
            <CardDescription>Community-voted rankings, updated weekly.</CardDescription>
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

export const JustifiedFooter = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Annihilation 3</CardTitle>
            <CardDescription>Tundra · 400 kills for the full weekly cap</CardDescription>
        </CardHeader>
        <CardPanel className="font-mono font-semibold text-3xl tabular-nums">1,800</CardPanel>
        <CardFooter className="justify-between">
            <span className="text-muted-foreground text-sm">Orundum this week</span>
            <Badge variant="success">Capped</Badge>
        </CardFooter>
    </Card>
);

export const MetaFooter = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>D32 Steel</CardTitle>
            <CardDescription>T5 material · 12 in depot, 4 short of the plan</CardDescription>
        </CardHeader>
        <CardPanel className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Best stage</span>
            <span className="font-mono tabular-nums">4-8 · 21 sanity</span>
        </CardPanel>
        <CardFooter className="text-muted-foreground text-xs">Drop rates last refreshed 6 hours ago from Penguin Stats.</CardFooter>
    </Card>
);
