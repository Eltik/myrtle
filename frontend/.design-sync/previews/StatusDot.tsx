import { Card, CardContent, CardDescription, CardHeader, CardTitle, StatusDot } from "frontend";

export function ProbeStates() {
    return (
        <div className="w-fit rounded-2xl border border-border bg-card px-4 py-3.5">
            <div className="mb-2.5 font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-widest">Probe states</div>
            <div className="flex flex-col gap-2.5">
                <div className="flex items-center justify-between gap-8">
                    <span className="text-[12.5px] text-muted-foreground">Postgres</span>
                    <StatusDot state="green">
                        <span className="font-mono">connected</span>
                    </StatusDot>
                </div>
                <div className="flex items-center justify-between gap-8">
                    <span className="text-[12.5px] text-muted-foreground">Cache · redis</span>
                    <StatusDot state="amber">
                        <span className="font-mono">degraded</span>
                    </StatusDot>
                </div>
                <div className="flex items-center justify-between gap-8">
                    <span className="text-[12.5px] text-muted-foreground">Asset CDN</span>
                    <StatusDot state="red">
                        <span className="font-mono">unreachable</span>
                    </StatusDot>
                </div>
            </div>
        </div>
    );
}

export function ServiceHealthCard() {
    return (
        <Card className="max-w-sm">
            <CardHeader>
                <CardTitle className="text-sm">Service health</CardTitle>
                <CardDescription className="text-xs">GET /health</CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <div className="flex items-center justify-between gap-3 pb-2 text-[12.5px]">
                    <span className="truncate text-muted-foreground">Postgres</span>
                    <StatusDot state="green">
                        <span className="font-mono tabular-nums">14 ms</span>
                    </StatusDot>
                </div>
                <div className="border-border border-t" />
                <div className="flex items-center justify-between gap-3 py-2 text-[12.5px]">
                    <span className="min-w-0 truncate text-muted-foreground">
                        Cache · <span className="text-foreground/70">redis</span>
                    </span>
                    <StatusDot state="amber">
                        <span className="font-mono tabular-nums">212 ms</span>
                    </StatusDot>
                </div>
                <div className="border-border border-t" />
                <div className="flex items-center justify-between gap-3 py-2 text-[12.5px]">
                    <span className="truncate text-muted-foreground">Game data</span>
                    <StatusDot state="green">
                        <span className="font-mono tabular-nums">438 ops</span>
                    </StatusDot>
                </div>
                <div className="border-border border-t" />
                <div className="flex items-center justify-between gap-3 pt-2 text-[12.5px]">
                    <span className="truncate text-muted-foreground">Asset CDN</span>
                    <StatusDot state="red">
                        <span className="font-mono tabular-nums">timeout</span>
                    </StatusDot>
                </div>
            </CardContent>
        </Card>
    );
}

export function PulsingLiveProbe() {
    return (
        <div className="w-fit rounded-2xl border border-border bg-card px-4 py-3.5">
            <div className="mb-2.5 font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-widest">Live · re-probes every 30s</div>
            <div className="flex flex-col gap-2.5">
                <div className="flex items-center justify-between gap-8">
                    <span className="text-[12.5px] text-muted-foreground">Postgres</span>
                    <StatusDot state="green" pulse>
                        <span className="font-mono">connected</span>
                    </StatusDot>
                </div>
                <div className="flex items-center justify-between gap-8">
                    <span className="text-[12.5px] text-muted-foreground">Cache · redis</span>
                    <StatusDot state="amber" pulse>
                        <span className="font-mono">reconnecting</span>
                    </StatusDot>
                </div>
                <div className="flex items-center justify-between gap-8">
                    <span className="text-[12.5px] text-muted-foreground">Asset CDN</span>
                    <StatusDot state="red" pulse>
                        <span className="font-mono">unknown</span>
                    </StatusDot>
                </div>
            </div>
        </div>
    );
}
