import { Progress, ProgressIndicator, ProgressLabel, ProgressTrack, ProgressValue } from "frontend";

/** Default indicator — brand primary, width driven by `value`. */
export const Default = () => (
    <div className="w-80">
        <Progress value={62}>
            <div className="flex items-baseline justify-between gap-2">
                <ProgressLabel>Chibi export — encoding</ProgressLabel>
                <ProgressValue className="font-mono text-muted-foreground text-xs">{(_f: string | null, v: number | null) => `${v}%`}</ProgressValue>
            </div>
            <ProgressTrack>
                <ProgressIndicator />
            </ProgressTrack>
        </Progress>
    </div>
);

/** The fill is a className concern — a destructive tint marks an over-budget plan. */
export const Tints = () => (
    <div className="flex w-80 flex-col gap-5">
        {[
            { label: "Sanity budget — on track", value: 48, tint: "" },
            { label: "LMD reserve — running low", value: 88, tint: "bg-destructive" },
            { label: "Archived plan", value: 35, tint: "bg-muted-foreground" },
        ].map((row) => (
            <Progress key={row.label} value={row.value}>
                <ProgressLabel>{row.label}</ProgressLabel>
                <ProgressTrack>
                    <ProgressIndicator className={row.tint} />
                </ProgressTrack>
            </Progress>
        ))}
    </div>
);

/** Fill extremes — an untouched goal and a finished one. */
export const Extremes = () => (
    <div className="flex w-80 flex-col gap-5">
        <Progress value={0}>
            <ProgressLabel>Chapter 14 — Hortus de Escapismo</ProgressLabel>
            <ProgressTrack>
                <ProgressIndicator />
            </ProgressTrack>
        </Progress>
        <Progress value={100}>
            <ProgressLabel>Chapter 7 — Preluding Lights</ProgressLabel>
            <ProgressTrack>
                <ProgressIndicator />
            </ProgressTrack>
        </Progress>
    </div>
);
