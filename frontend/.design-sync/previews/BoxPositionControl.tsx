import { BoxPositionControl, Row } from "frontend";
import type { ReactNode } from "react";

// Where the text box sits: a 16:9 miniature of the stage whose red thumb is
// the box (press or drag anywhere on the pad to move it, arrows nudge it),
// with the five presets and Reset beneath. The pressed preset is the one the
// position IS; between presets none is pressed. It lives in the Settings
// sheet's "Box position" row, so each story rebuilds that row.

const noop = () => {};

function Sheet({ children }: { children: ReactNode }) {
    return <div className="flex w-full max-w-xl flex-col gap-4 rounded-2xl border bg-popover p-6 text-popover-foreground shadow-lg">{children}</div>;
}

// The default: bottom centre, the position Reset writes.
export const BottomCentre = () => (
    <Sheet>
        <Row label="Box position" value="50%, 0%" hint="Drag the box on the stage, or drag the thumbnail here. Arrow keys nudge it.">
            <BoxPositionControl position={{ x: 0, y: 0 }} onChange={noop} />
        </Row>
    </Sheet>
);

// The top-centre preset: the thumb at the top of its travel.
export const TopCentre = () => (
    <Sheet>
        <Row label="Box position" value="50%, 100%" hint="Drag the box on the stage, or drag the thumbnail here. Arrow keys nudge it.">
            <BoxPositionControl position={{ x: 0, y: 1 }} onChange={noop} />
        </Row>
    </Sheet>
);

// A dragged position between presets: no preset button is pressed.
export const DraggedBetween = () => (
    <Sheet>
        <Row label="Box position" value="80%, 25%" hint="Drag the box on the stage, or drag the thumbnail here. Arrow keys nudge it.">
            <BoxPositionControl position={{ x: 0.6, y: 0.25 }} onChange={noop} />
        </Row>
    </Sheet>
);
