import { AdjustBar } from "frontend";
import { useState } from "react";

// AdjustBar is the row under the background editor's preview: the Zoom slider
// with its value, an Elite 1 / Elite 2 toggle for operator art, then Fit
// (zoom back to 100%), Reset (the kind's starting crop) and Remove, and a hint
// saying the preview is dragged to crop (or, for a moment, why a drag along a
// dead axis moved nothing). With no background it is one muted line. The
// collapsed strip uses the compact form: the zoom alone.

type Bg = { kind: "operator" | "skin" | "archive_pic" | "story_cg" | "story_scene"; id: string; focus_x?: number; focus_y?: number; scale?: number; elite?: number };

function Bar({ initial, deadAxis = null, compact = false, width = 820 }: { initial: Bg | null; deadAxis?: "x" | "y" | null; compact?: boolean; width?: number }) {
    const [draft, setDraft] = useState<Bg | null>(initial);
    return (
        <div style={{ width }} className="rounded-lg border bg-background px-4 py-3">
            <AdjustBar draft={draft as never} onChange={(next) => setDraft(next as Bg)} onRemove={() => setDraft(null)} deadAxis={deadAxis} disabled={false} compact={compact} />
        </div>
    );
}

/** Operator art, unzoomed: Fit and Reset disabled, Elite 2 pressed. */
export const OperatorUnzoomed = () => <Bar initial={{ kind: "operator", id: "char_291_aglina" }} />;

/** A story CG zoomed to 165% and moved: every action live. */
export const StoryCgZoomed = () => <Bar initial={{ kind: "story_cg", id: "47_i01", scale: 165, focus_x: 62, focus_y: 40 }} />;

/** A sideways drag on unzoomed operator art: the hint says why nothing moved. */
export const DeadAxisHint = () => <Bar initial={{ kind: "operator", id: "char_291_aglina", elite: 1 }} deadAxis="x" />;

/** The collapsed strip's compact form: the zoom alone. */
export const Compact = () => <Bar initial={{ kind: "skin", id: "char_1012_skadi2@iteration#2", scale: 120 }} compact width={320} />;

/** No background chosen. */
export const NoBackground = () => <Bar initial={null} />;
