import { AllowedKindsDialog } from "frontend";
import { type ReactNode, useEffect } from "react";

// The editor's types dialog: a draft of the allowed-types checklist, applied on
// Apply. The counts beside each kind are the cells already holding a pick of it.

const noop = () => {};

/**
 * Full-viewport stage: the popup is `position: fixed` against the story root, so a short stage
 * crops it. An open modal focuses its first control after the open transition and the brand-red
 * ring reads as a validation error, so the stage blurs it at 60/180/400 ms.
 */
const Stage = ({ children }: { children: ReactNode }) => {
    useEffect(() => {
        const ids = [60, 180, 400].map((ms) => setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => ids.forEach(clearTimeout);
    }, []);
    return <div className="min-h-dvh">{children}</div>;
};

export const WithPlacedPicks = () => (
    <Stage>
        <AllowedKindsDialog open kinds={["operator", "skin", "enemy"]} placedByKind={{ operator: 6, skin: 2, enemy: 1 }} onClose={noop} onApply={noop} />
    </Stage>
);

/** A grid that has nothing placed yet: no counts beside the kinds. */
export const EmptyGrid = () => (
    <Stage>
        <AllowedKindsDialog open kinds={["operator"]} placedByKind={{}} onClose={noop} onApply={noop} />
    </Stage>
);
