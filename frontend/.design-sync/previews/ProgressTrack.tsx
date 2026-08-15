import { Progress, ProgressIndicator, ProgressLabel, ProgressTrack, ProgressValue } from "frontend";

/** Default track — 6px, rounded, filled with the input token. */
export const Default = () => (
    <div className="w-80">
        <Progress value={64}>
            <div className="flex items-baseline justify-between gap-2">
                <ProgressLabel>Sanity regenerated</ProgressLabel>
                <ProgressValue className="font-mono text-muted-foreground text-xs">{(_f: string | null, v: number | null) => `${v}%`}</ProgressValue>
            </div>
            <ProgressTrack>
                <ProgressIndicator />
            </ProgressTrack>
        </Progress>
    </div>
);

/** Track height is a className concern — thin for dense rows, thick for a hero stat. */
export const Heights = () => (
    <div className="flex w-80 flex-col gap-5">
        {[
            { label: "Thin — annihilation cap", value: 82, height: "h-1" },
            { label: "Default — chapter clear", value: 45, height: "" },
            { label: "Thick — event shop", value: 30, height: "h-3" },
        ].map((row) => (
            <Progress key={row.label} value={row.value}>
                <ProgressLabel>{row.label}</ProgressLabel>
                <ProgressTrack className={row.height}>
                    <ProgressIndicator />
                </ProgressTrack>
            </Progress>
        ))}
    </div>
);

/** A muted track inside a card, where `bg-input` would disappear against the surface. */
export const OnCard = () => (
    <div className="w-80 rounded-lg border bg-card p-4">
        <Progress max={30} value={19}>
            <div className="flex items-baseline justify-between gap-2">
                <ProgressLabel>Headhunting pity</ProgressLabel>
                <ProgressValue className="font-mono text-muted-foreground text-xs tabular-nums">{(_f: string | null, v: number | null) => `${v} / 30 pulls`}</ProgressValue>
            </div>
            <ProgressTrack className="h-2 bg-muted">
                <ProgressIndicator />
            </ProgressTrack>
            <p className="text-muted-foreground text-xs">6★ rate rises 2% per pull past 50.</p>
        </Progress>
    </div>
);
