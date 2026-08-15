import { Badge, Card, CardHeader, CardTitle } from "frontend";
import { CheckIcon, SkullIcon, SparklesIcon, TriangleAlertIcon } from "lucide-react";

export const Variants = () => (
    <div className="flex flex-wrap items-center gap-2">
        <Badge>6★</Badge>
        <Badge variant="secondary">Guard</Badge>
        <Badge variant="outline">Module X</Badge>
        <Badge variant="success">healthy</Badge>
        <Badge variant="warning">unsaved changes</Badge>
        <Badge variant="error">Boss</Badge>
        <Badge variant="info">CN only</Badge>
        <Badge variant="destructive">deprecated</Badge>
    </div>
);

export const Sizes = () => (
    <div className="flex flex-wrap items-center gap-2">
        <Badge size="sm" variant="outline">
            E2 90
        </Badge>
        <Badge size="default" variant="outline">
            E2 90
        </Badge>
        <Badge size="lg" variant="outline">
            E2 90
        </Badge>
    </div>
);

export const WithIcons = () => (
    <div className="flex flex-wrap items-center gap-2">
        <Badge variant="success">
            <CheckIcon />
            Synced 4m ago
        </Badge>
        <Badge variant="warning">
            <TriangleAlertIcon />
            Sanity low
        </Badge>
        <Badge className="font-mono uppercase tracking-[0.12em]" variant="error">
            <SkullIcon />
            Boss
        </Badge>
        <Badge>
            <SparklesIcon />
            New skin
        </Badge>
    </div>
);

export const InContext = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle className="flex items-center gap-2">
                Mlynar
                <Badge size="sm">6★</Badge>
            </CardTitle>
        </CardHeader>
        <div className="flex flex-wrap gap-1.5 px-4 pb-4">
            <Badge variant="secondary">Guard</Badge>
            <Badge variant="secondary">Soloblade</Badge>
            <Badge variant="outline">Karlan Trade</Badge>
            <Badge className="font-mono uppercase" size="sm" variant="outline">
                EN
            </Badge>
            <Badge size="sm" variant="success">
                Owned
            </Badge>
        </div>
    </Card>
);

export const AsCounter = () => (
    <div className="flex flex-wrap items-center gap-4 text-foreground text-sm">
        <span className="inline-flex items-center gap-2">
            Chip Catalyst
            <Badge className="font-mono tabular-nums" size="sm" variant="secondary">
                12
            </Badge>
        </span>
        <span className="inline-flex items-center gap-2">
            Orirock Cube
            <Badge className="font-mono tabular-nums" size="sm" variant="secondary">
                348
            </Badge>
        </span>
        <span className="inline-flex items-center gap-2">
            Pending pulls
            <Badge className="font-mono tabular-nums" size="sm">
                9
            </Badge>
        </span>
    </div>
);
