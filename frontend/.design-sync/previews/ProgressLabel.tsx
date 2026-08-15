import { Progress, ProgressIndicator, ProgressLabel, ProgressTrack, ProgressValue } from "frontend";

/** Label above the track, with the value pinned to the opposite edge. */
export const WithValue = () => (
    <div className="w-80">
        <Progress max={16} value={11}>
            <div className="flex items-baseline justify-between gap-2">
                <ProgressLabel>Bipolar Nanoflake</ProgressLabel>
                <ProgressValue className="font-mono text-muted-foreground text-xs tabular-nums">{(_f: string | null, v: number | null) => `${v} / 16`}</ProgressValue>
            </div>
            <ProgressTrack>
                <ProgressIndicator />
            </ProgressTrack>
        </Progress>
    </div>
);

/** Label plus a secondary caption — the account-progress card on a Doctor profile. */
export const WithCaption = () => (
    <div className="w-80 rounded-lg border bg-card p-4">
        <Progress value={68}>
            <div className="flex items-baseline justify-between gap-2">
                <ProgressLabel>Roster completion</ProgressLabel>
                <ProgressValue className="font-mono text-muted-foreground text-xs">{(_f: string | null, v: number | null) => `${v}%`}</ProgressValue>
            </div>
            <ProgressTrack>
                <ProgressIndicator />
            </ProgressTrack>
            <p className="text-muted-foreground text-xs">212 of 312 operators owned · 41 at E2</p>
        </Progress>
    </div>
);

/** Several labelled bars stacked — the module planner's per-operator breakdown. */
export const StackedLabels = () => (
    <div className="flex w-80 flex-col gap-5">
        {[
            { label: "Młynar · Skill 3 mastery", value: 100 },
            { label: "Skadi · Module CHA-X", value: 55 },
            { label: "Texas · Level 90", value: 22 },
        ].map((row) => (
            <Progress key={row.label} value={row.value}>
                <ProgressLabel>{row.label}</ProgressLabel>
                <ProgressTrack>
                    <ProgressIndicator />
                </ProgressTrack>
            </Progress>
        ))}
    </div>
);
