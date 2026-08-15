import { Badge, Button, Card, CardAction, CardDescription, CardHeader, CardPanel, CardTitle } from "frontend";

export const TitleAndDescription = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Skadi the Corrupting Heart</CardTitle>
            <CardDescription>Abyssal Hunter Supporter. Her songs heal and buff every ally in range.</CardDescription>
        </CardHeader>
        <CardPanel className="text-muted-foreground text-sm">
            Trust bonus: <span className="font-mono text-foreground tabular-nums">+90 ATK</span>
        </CardPanel>
    </Card>
);

export const WithAction = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Weekly annihilation</CardTitle>
            <CardDescription>Chernobog · resets Monday 04:00 server time</CardDescription>
            <CardAction>
                <Badge variant="success">Capped</Badge>
            </CardAction>
        </CardHeader>
        <CardPanel className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Orundum earned</span>
            <span className="font-mono tabular-nums">1,800 / 1,800</span>
        </CardPanel>
    </Card>
);

export const HeaderOnly = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>No runs recorded</CardTitle>
            <CardDescription>Clear a stage with drop tracking enabled and it will show up here.</CardDescription>
            <CardAction>
                <Button size="sm" variant="outline">
                    Learn how
                </Button>
            </CardAction>
        </CardHeader>
    </Card>
);
