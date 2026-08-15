import { Badge, Button, Card, CardAction, CardDescription, CardHeader, CardPanel, CardTitle, Switch } from "frontend";
import { MoreHorizontal } from "lucide-react";

export const BadgeAction = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Global tier list</CardTitle>
            <CardDescription>Community-voted rankings, updated weekly.</CardDescription>
            <CardAction>
                <Badge variant="secondary">v12</Badge>
            </CardAction>
        </CardHeader>
        <CardPanel className="text-muted-foreground text-sm">1,284 placements across 6 tiers, contributed by 312 doctors.</CardPanel>
    </Card>
);

export const IconButtonAction = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Saved squad — CC#12</CardTitle>
            <CardDescription>Mlynar, Ines, Eyjafjalla, Ling, Texas the Omertosa, Skadi</CardDescription>
            <CardAction>
                <Button aria-label="Squad options" size="icon-sm" variant="outline">
                    <MoreHorizontal aria-hidden="true" />
                </Button>
            </CardAction>
        </CardHeader>
        <CardPanel className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Risk cleared</span>
            <span className="font-mono tabular-nums">18</span>
        </CardPanel>
    </Card>
);

export const SwitchAction = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Sanity reminders</CardTitle>
            <CardDescription>Notify me when sanity is about to overflow.</CardDescription>
            <CardAction>
                <Switch defaultChecked />
            </CardAction>
        </CardHeader>
        <CardPanel className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Threshold</span>
            <span className="font-mono tabular-nums">120 / 135</span>
        </CardPanel>
    </Card>
);
