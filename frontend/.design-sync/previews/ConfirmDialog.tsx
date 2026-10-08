import { ConfirmDialog } from "frontend";
import { type ReactNode, useEffect } from "react";

// The in-app confirmation every grid screen asks through (never
// `window.confirm`). The caller supplies the words, already translated; the
// copy below is the product's own (delete, shrink). Every product use is
// `destructive`.

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

/** Deleting a grid: the destructive form, with the warning badge and a red confirm. */
export const DeleteGrid = () => (
    <Stage>
        <ConfirmDialog
            open
            destructive
            title="Delete this grid?"
            body={'"Best Kazimierz operators" will be deleted for good. Grids made from it keep their own copy.'}
            confirmLabel="Delete"
            cancelLabel="Cancel"
            onConfirm={noop}
            onCancel={noop}
        />
    </Stage>
);

/** The editor shrinking a grid past filled cells. */
export const ShrinkGrid = () => (
    <Stage>
        <ConfirmDialog
            open
            destructive
            title="Remove filled cells?"
            body="2 cells with a label or pick fall outside a 3 by 3 grid and will be removed."
            confirmLabel="Remove and resize"
            cancelLabel="Keep size"
            onConfirm={noop}
            onCancel={noop}
        />
    </Stage>
);

/** The confirm in flight: spinner on the action, cancel locked. */
export const Pending = () => (
    <Stage>
        <ConfirmDialog open destructive pending title="Delete this grid?" body={'"Best Kazimierz operators" will be deleted for good. Grids made from it keep their own copy.'} confirmLabel="Delete" cancelLabel="Cancel" onConfirm={noop} onCancel={noop} />
    </Stage>
);

/** The server refused: the error line sits above the footer. */
export const Failed = () => (
    <Stage>
        <ConfirmDialog
            open
            destructive
            title="Delete this grid?"
            body={'"Best Kazimierz operators" will be deleted for good. Grids made from it keep their own copy.'}
            confirmLabel="Delete"
            cancelLabel="Cancel"
            errorMessage="Couldn't reach the server. Check your connection and try again."
            onConfirm={noop}
            onCancel={noop}
        />
    </Stage>
);
