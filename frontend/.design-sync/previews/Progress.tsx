import { Button, Progress, ProgressIndicator, ProgressLabel, ProgressTrack, ProgressValue } from "frontend";
import { XIcon } from "lucide-react";

/** The chibi recorder's inline bar — `className` shrinks the root, value drives the fill. */
export const ExportRecording = () => (
    <div className="flex items-center gap-2">
        <div className="flex min-w-30 items-center gap-2">
            <Progress className="h-2 w-full" value={62} />
            <span className="text-muted-foreground text-xs tabular-nums">62%</span>
        </div>
        <Button className="h-8 w-8 p-0" size="icon" variant="ghost">
            <XIcon className="h-3.5 w-3.5" />
        </Button>
    </div>
);

/** Labelled rows in the material planner — label, value and track in one root. */
export const PlannerGoals = () => (
    <div className="flex w-80 flex-col gap-5">
        {[
            { name: "Bipolar Nanoflake", have: 11, need: 16 },
            { name: "D32 Steel", have: 4, need: 9 },
            { name: "Crystalline Electronic Unit", have: 0, need: 6 },
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

/** The value axis, from untouched to finished. */
export const ValueRange = () => (
    <div className="flex w-80 flex-col gap-5">
        {[
            { label: "Chapter 9 — Stultifera Navis", value: 0 },
            { label: "Chapter 8 — Roaring Flare", value: 45 },
            { label: "Chapter 7 — Preluding Lights", value: 100 },
        ].map((row) => (
            <Progress key={row.label} value={row.value}>
                <div className="flex items-baseline justify-between gap-2">
                    <ProgressLabel>{row.label}</ProgressLabel>
                    <ProgressValue className="font-mono text-muted-foreground text-xs">{(_f: string | null, v: number | null) => `${v}%`}</ProgressValue>
                </div>
                <ProgressTrack>
                    <ProgressIndicator />
                </ProgressTrack>
            </Progress>
        ))}
    </div>
);

/** Bare root — with no children the default track + indicator is rendered for you. */
export const Bare = () => (
    <div className="flex w-80 flex-col gap-2 rounded-lg border bg-card p-4">
        <div className="flex items-baseline justify-between gap-2">
            <span className="font-medium text-foreground text-sm">Depot sync</span>
            <span className="font-mono text-muted-foreground text-xs tabular-nums">78%</span>
        </div>
        <Progress value={78} />
        <span className="text-muted-foreground text-xs">Reading 312 operators and 1,940 depot items from the EN server.</span>
    </div>
);
