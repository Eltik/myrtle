import { Badge, Card, CardDescription, CardHeader, CardPanel, CardTitle } from "frontend";

export const OperatorTitle = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Młynar</CardTitle>
            <CardDescription>6★ Guard · Liberator · Kjerag</CardDescription>
        </CardHeader>
        <CardPanel className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Max ATK at E2 90</span>
            <span className="font-mono tabular-nums">1,043</span>
        </CardPanel>
    </Card>
);

export const TitleWithBadge = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle className="flex items-center gap-2">
                Under Tides
                <Badge variant="warning">
                    Rerun
                </Badge>
            </CardTitle>
            <CardDescription>Side story · ends in 9 days</CardDescription>
        </CardHeader>
        <CardPanel className="text-muted-foreground text-sm">Farmable: Crystalline Component, Manganese Ore, Grindstone.</CardPanel>
    </Card>
);

export const NumericTitle = () => (
    <div className="grid max-w-2xl gap-4 sm:grid-cols-2">
        <Card>
            <CardHeader>
                <CardDescription>Sanity spent</CardDescription>
                <CardTitle className="font-mono text-3xl tabular-nums">4,812</CardTitle>
            </CardHeader>
            <CardPanel className="text-muted-foreground text-sm">Last 30 days across 214 runs.</CardPanel>
        </Card>
        <Card>
            <CardHeader>
                <CardDescription>Operators at E2</CardDescription>
                <CardTitle className="font-mono text-3xl tabular-nums">63</CardTitle>
            </CardHeader>
            <CardPanel className="text-muted-foreground text-sm">Of 231 owned on the EN server.</CardPanel>
        </Card>
    </div>
);
