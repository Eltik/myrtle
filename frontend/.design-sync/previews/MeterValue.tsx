import { Meter, MeterIndicator, MeterLabel, MeterTrack, MeterValue } from "frontend";

export const Basic = () => (
    <div className="w-full max-w-sm">
        <Meter value={87}>
            <div className="flex items-center justify-between">
                <MeterLabel>Recruitment tag hit rate</MeterLabel>
                <MeterValue className="font-mono text-muted-foreground text-xs" />
            </div>
            <MeterTrack className="rounded-full">
                <MeterIndicator className="rounded-full" />
            </MeterTrack>
        </Meter>
        <p className="mt-2 text-muted-foreground text-xs">With no children and no format, MeterValue prints the value as a percentage.</p>
    </div>
);

export const Formatted = () => (
    <div className="w-full max-w-sm">
        <Meter format={{ maximumFractionDigits: 0, style: "percent" }} max={1} value={0.735}>
            <div className="flex items-center justify-between">
                <MeterLabel>Pity progress — Ambience Synesthesia</MeterLabel>
                <MeterValue className="font-mono text-muted-foreground text-xs" />
            </div>
            <MeterTrack className="rounded-full">
                <MeterIndicator className="rounded-full" />
            </MeterTrack>
        </Meter>
        <p className="mt-2 text-muted-foreground text-xs">
            <code className="font-mono">format</code> passes Intl.NumberFormat options through the root.
        </p>
    </div>
);

export const CustomRender = () => (
    <div className="w-full max-w-sm">
        <Meter max={135} value={87}>
            <div className="flex items-baseline justify-between">
                <MeterLabel>Sanity</MeterLabel>
                <MeterValue>
                    {(_, value) => (
                        <span className="font-mono text-sm tabular-nums">
                            <span className="font-semibold text-foreground">{value}</span>
                            <span className="text-muted-foreground"> / 135</span>
                        </span>
                    )}
                </MeterValue>
            </div>
            <MeterTrack className="rounded-full">
                <MeterIndicator className="rounded-full" />
            </MeterTrack>
        </Meter>
    </div>
);

export const Prominent = () => (
    <div className="w-full max-w-sm rounded-xl border border-border bg-card p-4">
        <Meter max={4812} value={3164}>
            <MeterValue className="font-mono font-semibold text-3xl tabular-nums">{(_, value) => value.toLocaleString("en-US")}</MeterValue>
            <MeterLabel className="text-muted-foreground">Sanity spent this month, of 4,812 budgeted</MeterLabel>
            <MeterTrack className="mt-1 rounded-full">
                <MeterIndicator className="rounded-full" />
            </MeterTrack>
        </Meter>
    </div>
);
