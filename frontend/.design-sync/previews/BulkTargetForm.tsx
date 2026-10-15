import { BulkTargetForm } from "frontend";
import { useState } from "react";

// The bulk-add dialog's rarity-free target: promotion, level (capped by the
// chosen Elite and noted as such), the shared skill level, a mastery per skill
// slot and one module stage. Each operator later stops at what its own rarity
// and kit allow; the footer note says so.

type Target = { elite: number; level: number | null; skill_level: number; masteries: [number, number, number]; module_stage: number; display_on_profile: boolean };

function Stage({ initial }: { initial: Target }) {
    const [target, setTarget] = useState<Target>(initial);
    return (
        <div className="max-w-xl">
            <BulkTargetForm target={target} onChange={setTarget} />
        </div>
    );
}

/** The usual end-game target: E2 Lv90, SL7, S3 M3, module stage 3. */
export const EndGame = () => <Stage initial={{ elite: 2, level: 90, skill_level: 7, masteries: [0, 0, 3], module_stage: 3, display_on_profile: false }} />;

/** A budget pass: E1 at its cap, SL7, no masteries, no module. */
export const E1Budget = () => <Stage initial={{ elite: 1, level: null, skill_level: 7, masteries: [0, 0, 0], module_stage: 0, display_on_profile: false }} />;

/** Masteries spread over all three slots and module stage 1. */
export const SpreadMasteries = () => <Stage initial={{ elite: 2, level: 60, skill_level: 7, masteries: [1, 2, 3], module_stage: 1, display_on_profile: true }} />;
