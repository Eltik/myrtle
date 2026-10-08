import { ScheduleControls } from "frontend";
import { useState } from "react";

// The Calendar tab's filter row: one toggle per schedule kind with its row count,
// the stage-only switch, and the dot legend. Counts are the live /release
// payloads' upcoming rows.
type Kind = "event" | "banner" | "skin" | "rerun" | "review";
const COUNTS: Record<Kind, number> = { event: 29, banner: 11, skin: 14, rerun: 9, review: 2 };

function Live({ initial, stageOnly }: { initial: Kind[]; stageOnly: boolean }) {
    const [kinds, setKinds] = useState(new Set<Kind>(initial));
    const [stage, setStage] = useState(stageOnly);
    return <ScheduleControls kinds={kinds as never} onKindsChange={setKinds as never} stageOnly={stage} onStageOnlyChange={setStage} counts={COUNTS} />;
}

// Every kind on: each swatch carries its kind's tint.
export const AllKinds = () => (
    <div className="w-full max-w-3xl p-4">
        <Live initial={["event", "banner", "skin", "rerun", "review"]} stageOnly={false} />
    </div>
);

// Events and banners only, stage events only: off kinds drop to an empty
// outlined swatch and muted text.
export const Filtered = () => (
    <div className="w-full max-w-3xl p-4">
        <Live initial={["event", "banner"]} stageOnly={true} />
    </div>
);
