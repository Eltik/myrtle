import type React from "react";

/**
 * Empty state when nothing has been rolled yet. Avoids generic "press the button"
 * spinner empties — uses an isometric stack of dice rendered as inline SVG with
 * the project's lagoon/palm/sea-ink palette.
 */
export function EmptyState(): React.ReactElement {
    return (
        <section className="relative overflow-hidden rounded-2xl border border-dashed border-border/70 bg-card/60 px-6 py-10 shadow-xs/5 sm:py-14">
            <div className="mx-auto flex max-w-md flex-col items-center text-center">
                <DiceStack />
                <p className="mt-5 font-mono text-[11px] uppercase tracking-[0.22em] text-muted-foreground">No orders rolled</p>
                <h2 className="mt-2 font-[var(--font-display)] text-2xl font-semibold text-foreground sm:text-3xl">Awaiting orders.</h2>
                <p className="mt-2 max-w-prose text-sm text-muted-foreground">
                    Hit <span className="font-mono">Roll Squad</span> above to draw a random stage, a squad of operators, and a challenge modifier.
                </p>
            </div>
        </section>
    );
}

function DiceStack(): React.ReactElement {
    return (
        <svg
            aria-hidden="true"
            viewBox="0 0 200 130"
            className="h-24 w-auto text-foreground sm:h-28"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
        >
            {/* die 1 — back-left */}
            <g style={{ color: "var(--sea-ink-soft)" }}>
                <path d="M40 32 L72 18 L104 32 L72 46 Z" />
                <path d="M40 32 L40 78 L72 92 L72 46 Z" />
                <path d="M72 46 L104 32 L104 78 L72 92 Z" fill="var(--surface, rgba(255,255,255,.5))" />
                <circle cx="56" cy="62" r="2.2" fill="currentColor" />
                <circle cx="88" cy="56" r="2.2" fill="currentColor" />
                <circle cx="88" cy="70" r="2.2" fill="currentColor" />
            </g>
            {/* die 2 — middle */}
            <g style={{ color: "var(--lagoon-deep)" }}>
                <path d="M86 50 L120 34 L154 50 L120 66 Z" />
                <path d="M86 50 L86 96 L120 112 L120 66 Z" />
                <path d="M120 66 L154 50 L154 96 L120 112 Z" fill="var(--surface, rgba(255,255,255,.5))" />
                <circle cx="103" cy="78" r="2.2" fill="currentColor" />
                <circle cx="137" cy="74" r="2.2" fill="currentColor" />
                <circle cx="137" cy="86" r="2.2" fill="currentColor" />
                <circle cx="137" cy="98" r="2.2" fill="currentColor" />
            </g>
            {/* die 3 — front-right */}
            <g style={{ color: "var(--palm)" }}>
                <path d="M130 32 L162 18 L194 32 L162 46 Z" />
                <path d="M130 32 L130 78 L162 92 L162 46 Z" />
                <path d="M162 46 L194 32 L194 78 L162 92 Z" fill="var(--surface, rgba(255,255,255,.5))" />
                <circle cx="146" cy="55" r="2.2" fill="currentColor" />
                <circle cx="146" cy="69" r="2.2" fill="currentColor" />
                <circle cx="178" cy="48" r="2.2" fill="currentColor" />
                <circle cx="178" cy="62" r="2.2" fill="currentColor" />
                <circle cx="178" cy="76" r="2.2" fill="currentColor" />
            </g>
        </svg>
    );
}
