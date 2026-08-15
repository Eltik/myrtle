import { Meter, MeterIndicator, MeterLabel, MeterTrack, MeterValue } from "frontend";

export const Basic = () => (
    <div className="w-full max-w-sm">
        <Meter value={87} max={135}>
            <MeterLabel>Sanity</MeterLabel>
            <MeterTrack className="rounded-full">
                <MeterIndicator className="rounded-full" />
            </MeterTrack>
        </Meter>
    </div>
);

export const WithValue = () => (
    <div className="w-full max-w-sm">
        <Meter value={44} max={60}>
            <div className="flex items-center justify-between">
                <MeterLabel>Base trust — Rhodes Island</MeterLabel>
                <MeterValue className="font-mono text-muted-foreground text-xs">{(_, value) => `${value}/60`}</MeterValue>
            </div>
            <MeterTrack className="rounded-full">
                <MeterIndicator className="rounded-full" />
            </MeterTrack>
        </Meter>
    </div>
);

export const WithDescription = () => (
    <div className="w-full max-w-sm rounded-xl border border-border bg-card p-4">
        <Meter value={68} max={100}>
            <MeterLabel className="text-base">Annihilation 4 — Wasteland</MeterLabel>
            <p className="text-muted-foreground text-xs">Weekly cap resets Monday 04:00 server time.</p>
            <MeterTrack className="mt-1 rounded-full">
                <MeterIndicator className="rounded-full" />
            </MeterTrack>
            <MeterValue className="font-mono text-[11px] text-muted-foreground">{(_, value) => `${value} of 100 kills`}</MeterValue>
        </Meter>
    </div>
);

export const Muted = () => (
    <div className="flex w-full max-w-sm flex-col gap-5">
        {[
            { label: "Trust", pct: 92 },
            { label: "Potential", pct: 33 },
        ].map((row) => (
            <Meter key={row.label} value={row.pct}>
                <div className="flex items-center justify-between">
                    <MeterLabel className="font-mono text-[11px] text-muted-foreground uppercase tracking-tight">{row.label}</MeterLabel>
                    <MeterValue className="font-mono text-[11px] text-muted-foreground">{(_, value) => `${value}%`}</MeterValue>
                </div>
                <MeterTrack className="h-1.5 rounded-full">
                    <MeterIndicator className="rounded-full" />
                </MeterTrack>
            </Meter>
        ))}
    </div>
);
