import { OperatorRange } from "frontend";

// The attack-range grid used on the Information and Skills tabs. It takes a
// `/static/ranges` entry straight from the API — the grids below are the real
// payloads for range ids `1-2`, `3-1`, `5-1` and `x-2`. The filled square is the
// operator's own tile; hatched tiles are covered.
//
// The panel chrome is copied from SkillsContent's "Range" block so the cell
// reads the way it does in the app.

const RANGES = {
    // Guard / Defender melee arc
    "1-2": {
        id: "1-2",
        direction: 1,
        grids: [
            { row: 1, col: 0 },
            { row: 0, col: 0 },
            { row: 0, col: 1 },
            { row: -1, col: 0 },
        ],
    },
    // Standard ranged block (Caster, Medic)
    "3-1": {
        id: "3-1",
        direction: 1,
        grids: [
            { row: 1, col: 0 },
            { row: 1, col: 1 },
            { row: 1, col: 2 },
            { row: 0, col: 0 },
            { row: 0, col: 1 },
            { row: 0, col: 2 },
            { row: 0, col: 3 },
            { row: -1, col: 0 },
            { row: -1, col: 1 },
            { row: -1, col: 2 },
        ],
    },
    // Artilleryman single-lane reach
    "5-1": {
        id: "5-1",
        direction: 1,
        grids: [
            { row: 0, col: 0 },
            { row: 0, col: 1 },
            { row: 0, col: 2 },
            { row: 0, col: 3 },
            { row: 0, col: 4 },
            { row: 0, col: 5 },
        ],
    },
    // Skill-expanded global-ish coverage
    "x-2": {
        id: "x-2",
        direction: 1,
        grids: [
            { row: 2, col: -1 },
            { row: 2, col: 0 },
            { row: 2, col: 1 },
            { row: 1, col: -2 },
            { row: 1, col: -1 },
            { row: 1, col: 0 },
            { row: 1, col: 1 },
            { row: 1, col: 2 },
            { row: 0, col: -2 },
            { row: 0, col: -1 },
            { row: 0, col: 0 },
            { row: 0, col: 1 },
            { row: 0, col: 2 },
            { row: -1, col: -2 },
            { row: -1, col: -1 },
            { row: -1, col: 0 },
            { row: -1, col: 1 },
            { row: -1, col: 2 },
            { row: -2, col: -1 },
            { row: -2, col: 0 },
            { row: -2, col: 1 },
        ],
    },
};

function RangePanel({ label, children }: { label: string; children: React.ReactNode }) {
    return (
        <div className="w-fit rounded-sm border border-border/50 bg-secondary/10 p-4">
            <span className="mb-3 block font-medium text-muted-foreground text-xs uppercase tracking-wide">{label}</span>
            {children}
        </div>
    );
}

export const MeleeArc = () => (
    <RangePanel label="Range">
        <OperatorRange range={RANGES["1-2"]} />
    </RangePanel>
);

export const RangedBlock = () => (
    <RangePanel label="Range">
        <OperatorRange range={RANGES["3-1"]} />
    </RangePanel>
);

export const ArtilleryLane = () => (
    <RangePanel label="Range">
        <OperatorRange range={RANGES["5-1"]} />
    </RangePanel>
);

export const SkillRangeDiff = () => (
    <RangePanel label="Range">
        <div className="flex flex-wrap items-start gap-6">
            <div className="space-y-1.5">
                <span className="block font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">Original</span>
                <OperatorRange range={RANGES["3-1"]} />
            </div>
            <div className="space-y-1.5">
                <span className="block font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wide">On Skill</span>
                <OperatorRange range={RANGES["x-2"]} />
            </div>
        </div>
    </RangePanel>
);
