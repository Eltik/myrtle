import { Badge, Button, Card, CardFrame, CardFrameAction, CardFrameDescription, CardFrameHeader, CardFrameTitle, CardPanel, Switch } from "frontend";
import { RefreshCw } from "lucide-react";

export const BadgeAction = () => (
    <CardFrame className="max-w-md">
        <CardFrameHeader>
            <CardFrameTitle>Tier list</CardFrameTitle>
            <CardFrameDescription>Community-voted rankings, updated weekly.</CardFrameDescription>
            <CardFrameAction>
                <Badge variant="outline">v12</Badge>
            </CardFrameAction>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <span className="text-sm">Mlynar</span>
                <span className="font-mono text-muted-foreground text-sm">S+</span>
            </CardPanel>
        </Card>
    </CardFrame>
);

export const ButtonAction = () => (
    <CardFrame className="max-w-md">
        <CardFrameHeader>
            <CardFrameTitle>Depot snapshot</CardFrameTitle>
            <CardFrameDescription>Synced 2 hours ago from the EN server.</CardFrameDescription>
            <CardFrameAction>
                <Button size="sm" variant="outline">
                    <RefreshCw aria-hidden="true" />
                    Sync
                </Button>
            </CardFrameAction>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <span className="text-muted-foreground text-sm">Orundum</span>
                <span className="font-mono text-sm tabular-nums">14,720</span>
            </CardPanel>
        </Card>
    </CardFrame>
);

export const SwitchAction = () => (
    <CardFrame className="max-w-md">
        <CardFrameHeader>
            <CardFrameTitle>Challenge mode drops</CardFrameTitle>
            <CardFrameDescription>Include CM stages in the randomizer pool.</CardFrameDescription>
            <CardFrameAction>
                <Switch defaultChecked />
            </CardFrameAction>
        </CardFrameHeader>
        <Card>
            <CardPanel className="flex items-center justify-between gap-4 p-4">
                <span className="font-mono text-sm">S4-1 CM</span>
                <span className="text-muted-foreground text-sm">21 sanity</span>
            </CardPanel>
        </Card>
    </CardFrame>
);
