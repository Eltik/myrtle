import { Meter, MeterIndicator, MeterLabel, MeterTrack, MeterValue } from "frontend";

export const Basic = () => (
    <div className="w-full max-w-sm">
        <Meter value={87} max={135}>
            <div className="flex items-center justify-between">
                <MeterLabel>Sanity</MeterLabel>
                <MeterValue className="font-mono text-muted-foreground text-xs">{(_, value) => `${value} / 135`}</MeterValue>
            </div>
            <MeterTrack className="rounded-full">
                <MeterIndicator className="rounded-full" />
            </MeterTrack>
        </Meter>
    </div>
);

export const DefaultTrack = () => (
    <div className="w-full max-w-sm">
        <p className="mb-2 font-medium text-foreground text-sm">Chapter 8 — Roaring Flare</p>
        <Meter value={62} />
        <p className="mt-2 text-muted-foreground text-xs">With no children, Meter renders its own track and indicator.</p>
    </div>
);

export const MaterialStock = () => (
    <div className="w-full max-w-sm rounded-xl border border-border bg-card p-4">
        <p className="mb-1 font-semibold text-sm">Elite material stock</p>
        <p className="mb-4 text-muted-foreground text-xs">Progress toward Mlynar E2 · Skill 3 M3</p>
        <div className="flex flex-col gap-3.5">
            {[
                { name: "Bipolar Nanoflake", have: 4, need: 6 },
                { name: "D32 Steel", have: 9, need: 9 },
                { name: "Crystalline Electronic Unit", have: 1, need: 5 },
            ].map((mat) => (
                <Meter key={mat.name} max={mat.need} value={mat.have}>
                    <div className="flex items-baseline justify-between gap-2">
                        <MeterLabel className="truncate text-[13px]">{mat.name}</MeterLabel>
                        <MeterValue className="shrink-0 font-mono text-[11px] text-muted-foreground">{(_, value) => `${value}/${mat.need}`}</MeterValue>
                    </div>
                    <MeterTrack className="h-1.5 rounded-full">
                        <MeterIndicator className="rounded-full" />
                    </MeterTrack>
                </Meter>
            ))}
        </div>
    </div>
);

export const Levels = () => (
    <div className="flex w-full max-w-sm flex-col gap-5">
        {[
            { label: "Chip Catalyst", pct: 12 },
            { label: "Orirock Cluster", pct: 54 },
            { label: "LMD reserve", pct: 96 },
        ].map((row) => (
            <Meter key={row.label} value={row.pct}>
                <div className="flex items-center justify-between">
                    <MeterLabel>{row.label}</MeterLabel>
                    <MeterValue className="font-mono text-muted-foreground text-xs">{(_, value) => `${value}%`}</MeterValue>
                </div>
                <MeterTrack className="rounded-full">
                    <MeterIndicator className="rounded-full" />
                </MeterTrack>
            </Meter>
        ))}
    </div>
);
