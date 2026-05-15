import type React from "react";

export function EmptyState(): React.ReactElement {
    return (
        <section className="relative overflow-hidden rounded-2xl border border-border/70 border-dashed bg-card/60 px-6 py-10 shadow-xs/5 sm:py-14">
            <div className="mx-auto flex max-w-md flex-col items-center text-center">
                <DiceStack />
                <p className="mt-5 font-mono text-[11px] text-muted-foreground uppercase tracking-[0.22em]">No squads rolled</p>
                <h2 className="mt-2 font-(--font-display) text-2xl text-foreground sm:text-3xl">Awaiting randomizer.</h2>
                <p className="mt-3 max-w-prose text-pretty text-muted-foreground text-sm leading-relaxed">
                    Hit <kbd className="mx-0.5 inline-flex -translate-y-px items-center rounded-sm border border-border/70 bg-foreground/5 px-1.5 py-px font-medium font-mono text-[11px] text-foreground tracking-tight shadow-xs/5">Roll Squad</kbd> above to draw a random stage, a squad of operators, and a challenge
                    modifier.
                </p>
            </div>
        </section>
    );
}

type Matrix = readonly [number, number, number, number, number, number];

const FACE_SIZE = 32;
const FACE_SKEW = 14;
const FACE_HEIGHT = 46;
const CORNER_R = 5;
const PIP_R = 1.7;

const PIP_PATTERNS: Record<number, ReadonlyArray<[number, number]>> = {
    1: [[0.5, 0.5]],
    2: [
        [0.3, 0.3],
        [0.7, 0.7],
    ],
    3: [
        [0.25, 0.25],
        [0.5, 0.5],
        [0.75, 0.75],
    ],
    4: [
        [0.3, 0.3],
        [0.7, 0.3],
        [0.3, 0.7],
        [0.7, 0.7],
    ],
    5: [
        [0.25, 0.25],
        [0.75, 0.25],
        [0.5, 0.5],
        [0.25, 0.75],
        [0.75, 0.75],
    ],
    6: [
        [0.27, 0.28],
        [0.73, 0.28],
        [0.27, 0.5],
        [0.73, 0.5],
        [0.27, 0.72],
        [0.73, 0.72],
    ],
};

function DiceStack(): React.ReactElement {
    return (
        <svg aria-hidden="true" viewBox="0 0 200 130" className="h-24 w-auto text-foreground sm:h-28" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <Die ftx={72} fty={46} top={5} left={2} right={3} color="var(--sea-ink-soft)" />
            <Die ftx={120} fty={66} top={3} left={4} right={6} color="var(--lagoon-deep)" />
            <Die ftx={162} fty={46} top={2} left={1} right={4} color="var(--palm)" />
        </svg>
    );
}

interface IDieProps {
    ftx: number;
    fty: number;
    top: number;
    left: number;
    right: number;
    color: string;
}

function Die({ ftx, fty, top, left, right, color }: IDieProps): React.ReactElement {
    const ax = FACE_SIZE;
    const ay = FACE_SKEW;
    const h = FACE_HEIGHT;

    const ft: [number, number] = [ftx, fty];
    const rt: [number, number] = [ftx + ax, fty - ay];
    const lt: [number, number] = [ftx - ax, fty - ay];
    const bt: [number, number] = [ftx, fty - 2 * ay];
    const fb: [number, number] = [ftx, fty + h];
    const rb: [number, number] = [ftx + ax, fty - ay + h];
    const lb: [number, number] = [ftx - ax, fty - ay + h];

    // Face-local (u, v) ∈ [0,1]² → screen (x, y) for pip placement.
    const topMat: Matrix = [ax, -ay, -ax, -ay, ftx, fty];
    const leftMat: Matrix = [-ax, -ay, 0, h, ftx, fty];
    const rightMat: Matrix = [ax, -ay, 0, h, ftx, fty];

    return (
        <g style={{ color }}>
            <path d={roundedPath([lt, bt, rt, ft], CORNER_R)} />
            <path d={roundedPath([lt, ft, fb, lb], CORNER_R)} />
            <path d={roundedPath([ft, rt, rb, fb], CORNER_R)} fill="var(--surface, rgba(255,255,255,.5))" />
            <Pips count={top} matrix={topMat} />
            <Pips count={left} matrix={leftMat} />
            <Pips count={right} matrix={rightMat} />
        </g>
    );
}

function Pips({ count, matrix }: { count: number; matrix: Matrix }): React.ReactElement {
    const pattern = PIP_PATTERNS[count] ?? [];
    return (
        <>
            {pattern.map(([u, v]) => {
                const x = matrix[0] * u + matrix[2] * v + matrix[4];
                const y = matrix[1] * u + matrix[3] * v + matrix[5];
                return <circle key={`${u}-${v}`} cx={x} cy={y} r={PIP_R} fill="currentColor" stroke="none" />;
            })}
        </>
    );
}

function roundedPath(points: ReadonlyArray<[number, number]>, radius: number): string {
    const n = points.length;
    let d = "";
    for (let i = 0; i < n; i++) {
        const prev = points[(i - 1 + n) % n];
        const curr = points[i];
        const next = points[(i + 1) % n];
        const inX = curr[0] - prev[0];
        const inY = curr[1] - prev[1];
        const lenIn = Math.hypot(inX, inY);
        const outX = next[0] - curr[0];
        const outY = next[1] - curr[1];
        const lenOut = Math.hypot(outX, outY);
        const rIn = Math.min(radius, lenIn / 2);
        const rOut = Math.min(radius, lenOut / 2);
        const sx = curr[0] - (inX / lenIn) * rIn;
        const sy = curr[1] - (inY / lenIn) * rIn;
        const ex = curr[0] + (outX / lenOut) * rOut;
        const ey = curr[1] + (outY / lenOut) * rOut;
        d += i === 0 ? `M ${sx} ${sy}` : ` L ${sx} ${sy}`;
        d += ` Q ${curr[0]} ${curr[1]} ${ex} ${ey}`;
    }
    return `${d} Z`;
}
