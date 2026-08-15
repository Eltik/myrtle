import { Badge, Button, Separator } from "frontend";
import { Heart, Search } from "lucide-react";

const STATS: [string, string][] = [
    ["HP", "3,644"],
    ["ATK", "1,178"],
    ["DEF", "462"],
    ["RES", "0"],
    ["Cost", "23"],
    ["Block", "1"],
];

export const BetweenSections = () => (
    <div className="w-full max-w-sm rounded-lg border border-border bg-card p-4">
        <div className="flex items-center justify-between">
            <span className="font-medium text-foreground text-sm">Mlynar</span>
            <Badge variant="secondary">E2 90</Badge>
        </div>
        <Separator className="my-3" />
        <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
            {STATS.map(([label, value]) => (
                <div className="flex items-center justify-between" key={label}>
                    <span className="text-muted-foreground">{label}</span>
                    <span className="font-mono tabular-nums">{value}</span>
                </div>
            ))}
        </div>
        <Separator className="my-3" />
        <p className="text-muted-foreground text-xs">Trust 200% · Potential 1 · Module SWD-X Stage 3</p>
    </div>
);

export const Vertical = () => (
    <div className="flex items-center gap-1 rounded-lg border border-border bg-card px-3 py-2">
        <Button size="sm" variant="ghost">
            <Search />
            Search
        </Button>
        <Separator className="mx-1 h-5" orientation="vertical" />
        <Button size="sm" variant="ghost">
            Operators
        </Button>
        <Button size="sm" variant="ghost">
            Stages
        </Button>
        <Separator className="mx-1 h-5" orientation="vertical" />
        <Button size="sm" variant="ghost">
            <Heart />
            Donate
        </Button>
    </div>
);

export const InlinePill = () => (
    <div className="inline-flex w-max items-center gap-2 rounded-full border border-border bg-muted/60 px-3 py-1.5 pl-2.5">
        <span className="font-medium font-mono text-[11.5px] text-muted-foreground leading-none">v3</span>
        <Separator className="h-3.5 bg-border" orientation="vertical" />
        <span className="font-medium font-sans text-[11.5px] text-primary leading-none">changelog →</span>
    </div>
);

export const StackedList = () => (
    <div className="w-full max-w-sm rounded-lg border border-border bg-card p-4">
        <span className="block font-medium text-foreground text-sm">Recent pulls</span>
        <Separator className="my-3" />
        <div className="space-y-2 text-sm">
            <div className="flex items-center justify-between">
                <span>Mlynar</span>
                <span className="font-mono text-muted-foreground text-xs tabular-nums">6★ · pull 47</span>
            </div>
            <Separator />
            <div className="flex items-center justify-between">
                <span>Texas the Omertosa</span>
                <span className="font-mono text-muted-foreground text-xs tabular-nums">6★ · pull 12</span>
            </div>
            <Separator />
            <div className="flex items-center justify-between">
                <span>Heidi</span>
                <span className="font-mono text-muted-foreground text-xs tabular-nums">5★ · pull 9</span>
            </div>
        </div>
    </div>
);
