import { Progress, ProgressIndicator, ProgressLabel, ProgressTrack, ProgressValue } from "frontend";

/** Bare value — renders the formatted number for the current `value`. */
export const Default = () => (
    <div className="w-80">
        <Progress value={45}>
            <div className="flex items-baseline justify-between gap-2">
                <ProgressLabel>Chapter 8 — Roaring Flare</ProgressLabel>
                <ProgressValue className="font-mono text-muted-foreground text-xs tabular-nums" />
            </div>
            <ProgressTrack>
                <ProgressIndicator />
            </ProgressTrack>
        </Progress>
    </div>
);

/** A children function reshapes the text — the raw value, not the "%" default. */
export const AsFraction = () => (
    <div className="flex w-80 flex-col gap-5">
        {[
            { name: "Bipolar Nanoflake", have: 11, need: 16 },
            { name: "Polymerization Preparation", have: 3, need: 8 },
        ].map((mat) => (
            <Progress key={mat.name} max={mat.need} value={mat.have}>
                <div className="flex items-baseline justify-between gap-2">
                    <ProgressLabel>{mat.name}</ProgressLabel>
                    <ProgressValue className="font-mono text-muted-foreground text-xs tabular-nums">{(_f: string | null, v: number | null) => `${v} / ${mat.need}`}</ProgressValue>
                </div>
                <ProgressTrack>
                    <ProgressIndicator />
                </ProgressTrack>
            </Progress>
        ))}
    </div>
);

/** `format` on the root feeds `Intl.NumberFormat` — LMD with grouping separators. */
export const Formatted = () => (
    <div className="w-80 rounded-lg border bg-card p-4">
        <Progress format={{ useGrouping: true }} max={1500000} value={962400}>
            <div className="flex items-baseline justify-between gap-2">
                <ProgressLabel>LMD toward E2 batch</ProgressLabel>
                <ProgressValue className="font-mono text-muted-foreground text-xs tabular-nums">{(formatted: string | null) => `${formatted} LMD`}</ProgressValue>
            </div>
            <ProgressTrack>
                <ProgressIndicator />
            </ProgressTrack>
            <p className="text-muted-foreground text-xs">Target 1,500,000 LMD for six E2 promotions.</p>
        </Progress>
    </div>
);

/** The value read as a large stat, with the bar as supporting detail. */
export const AsHeadline = () => (
    <div className="w-80 rounded-lg border bg-card p-4">
        <Progress value={68}>
            <ProgressValue className="font-mono font-semibold text-3xl text-foreground tabular-nums">{(_f: string | null, v: number | null) => `${v}%`}</ProgressValue>
            <ProgressLabel className="text-muted-foreground">Roster completion</ProgressLabel>
            <ProgressTrack>
                <ProgressIndicator />
            </ProgressTrack>
        </Progress>
    </div>
);
