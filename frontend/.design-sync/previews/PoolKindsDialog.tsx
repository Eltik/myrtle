import { PoolKindsDialog } from "frontend";
import { type ReactNode, useEffect } from "react";

// Which kinds a tier list's pool offers: a checklist of all 13 kinds with how
// many of each are already on the board. Unticking a kind never removes its
// placements, only the pool tab; at least one kind stays ticked.

const noop = () => {};

/**
 * Full-viewport stage: the popup is `position: fixed` against the story root. An open modal
 * focuses its first control and the brand-red ring reads as an error, so the stage blurs it.
 */
const Stage = ({ children }: { children: ReactNode }) => {
    useEffect(() => {
        const ids = [60, 180, 400].map((ms) => setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => ids.forEach(clearTimeout);
    }, []);
    return <div className="min-h-dvh">{children}</div>;
};

/** A mixed list with placements of three kinds. */
export const MixedList = () => (
    <Stage>
        <PoolKindsDialog open kinds={["operator", "stronghold_bond", "story_sprite"]} placedByKind={{ operator: 36, stronghold_bond: 5, story_sprite: 6 }} onClose={noop} onApply={noop} />
    </Stage>
);

/** A new operator-only list: one kind ticked, nothing placed. */
export const OperatorsOnly = () => (
    <Stage>
        <PoolKindsDialog open kinds={["operator"]} placedByKind={{}} onClose={noop} onApply={noop} />
    </Stage>
);
