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

export const FillLevels = () => (
    <div className="flex w-full max-w-sm flex-col gap-5">
        {[
            { label: "Just started", pct: 8 },
            { label: "Halfway", pct: 50 },
            { label: "Nearly capped", pct: 94 },
        ].map((row) => (
            <Meter key={row.label} value={row.pct}>
                <div className="flex items-center justify-between">
                    <MeterLabel className="text-[13px]">{row.label}</MeterLabel>
                    <MeterValue className="font-mono text-[11px] text-muted-foreground">{(_, value) => `${value}%`}</MeterValue>
                </div>
                <MeterTrack className="rounded-full">
                    <MeterIndicator className="rounded-full" />
                </MeterTrack>
            </Meter>
        ))}
    </div>
);

export const SemanticColors = () => (
    <div className="w-full max-w-sm rounded-xl border border-border bg-card p-4">
        <p className="mb-1 font-semibold text-sm">Farming plan health</p>
        <p className="mb-4 text-muted-foreground text-xs">Days of sanity needed per material line.</p>
        <div className="flex flex-col gap-3.5">
            {[
                { label: "Orirock Cluster", pct: 92, cls: "bg-emerald-500" },
                { label: "Polyester Pack", pct: 58, cls: "bg-amber-500" },
                { label: "Crystalline Circuit", pct: 21, cls: "bg-destructive" },
            ].map((row) => (
                <Meter key={row.label} value={row.pct}>
                    <div className="flex items-center justify-between">
                        <MeterLabel className="text-[13px]">{row.label}</MeterLabel>
                        <MeterValue className="font-mono text-[11px] text-muted-foreground">{(_, value) => `${value}%`}</MeterValue>
                    </div>
                    <MeterTrack className="h-1.5 rounded-full">
                        <MeterIndicator className={`rounded-full ${row.cls}`} />
                    </MeterTrack>
                </Meter>
            ))}
        </div>
    </div>
);

export const Complete = () => (
    <div className="w-full max-w-sm">
        <Meter value={9} max={9}>
            <div className="flex items-center justify-between">
                <MeterLabel>D32 Steel — Mlynar E2</MeterLabel>
                <MeterValue className="font-mono text-muted-foreground text-xs">{(_, value) => `${value}/9`}</MeterValue>
            </div>
            <MeterTrack className="rounded-full">
                <MeterIndicator className="rounded-full bg-emerald-500" />
            </MeterTrack>
        </Meter>
        <p className="mt-2 text-emerald-600 text-xs">All requirements met.</p>
    </div>
);
