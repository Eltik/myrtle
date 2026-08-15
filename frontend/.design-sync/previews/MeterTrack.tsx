import { Meter, MeterIndicator, MeterLabel, MeterTrack, MeterValue } from "frontend";

export const Basic = () => (
    <div className="w-full max-w-sm">
        <Meter value={73} max={135}>
            <div className="flex items-center justify-between">
                <MeterLabel>Sanity</MeterLabel>
                <MeterValue className="font-mono text-muted-foreground text-xs">{(_, value) => `${value} / 135`}</MeterValue>
            </div>
            <MeterTrack>
                <MeterIndicator />
            </MeterTrack>
        </Meter>
    </div>
);

export const Rounded = () => (
    <div className="w-full max-w-sm">
        <Meter value={73} max={135}>
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

export const Thicknesses = () => (
    <div className="flex w-full max-w-sm flex-col gap-5">
        {[
            { label: "Thin — inline stat row", cls: "h-1.5 rounded-full" },
            { label: "Default — 8px track", cls: "" },
            { label: "Thick — headline progress", cls: "h-3 rounded-full" },
        ].map((row) => (
            <Meter key={row.label} value={64}>
                <MeterLabel className="text-[13px]">{row.label}</MeterLabel>
                <MeterTrack className={row.cls}>
                    <MeterIndicator className={row.cls ? "rounded-full" : ""} />
                </MeterTrack>
            </Meter>
        ))}
    </div>
);

export const InStatPanel = () => (
    <div className="w-full max-w-sm rounded-xl border border-border bg-card p-4">
        <div className="mb-3 flex items-baseline justify-between">
            <p className="font-semibold text-sm">Stage clear rate</p>
            <span className="font-mono text-muted-foreground text-xs">S4-1 · 42 runs</span>
        </div>
        <div className="flex flex-col gap-3.5">
            {[
                { label: "3-star", pct: 81 },
                { label: "Retreat", pct: 14 },
                { label: "Failed", pct: 5 },
            ].map((row) => (
                <Meter key={row.label} value={row.pct}>
                    <div className="flex items-center justify-between">
                        <MeterLabel className="text-[13px]">{row.label}</MeterLabel>
                        <MeterValue className="font-mono text-[11px] text-muted-foreground">{(_, value) => `${value}%`}</MeterValue>
                    </div>
                    <MeterTrack className="h-1.5 rounded-full bg-muted">
                        <MeterIndicator className="rounded-full" />
                    </MeterTrack>
                </Meter>
            ))}
        </div>
    </div>
);
