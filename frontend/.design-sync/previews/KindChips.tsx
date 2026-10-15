import { KindChips } from "frontend";
import type { ReactNode } from "react";

// A grid's allowed types as small chips, in tab order, as the grid page header,
// the editor and the browse cards show them. `max` caps the row with a dashed
// `+N` chip.

const Row = ({ children }: { children: ReactNode }) => <div className="w-full max-w-md p-4">{children}</div>;

/** Three kinds, all shown. */
export const Default = () => (
    <Row>
        <KindChips kinds={["enemy", "operator", "event"]} />
    </Row>
);

/** A card's capped row: six kinds with `max={3}` shows three and a `+3` chip. */
export const Capped = () => (
    <Row>
        <KindChips kinds={["operator", "skin", "module", "skill", "enemy", "event"]} max={3} />
    </Row>
);

/** One over the cap shows whole: a `+1` chip would take the room of the chip it hides. */
export const OneOverCap = () => (
    <Row>
        <KindChips kinds={["operator", "skin", "module", "skill"]} max={3} />
    </Row>
);

/** A single kind. */
export const Single = () => (
    <Row>
        <KindChips kinds={["operator"]} />
    </Row>
);
