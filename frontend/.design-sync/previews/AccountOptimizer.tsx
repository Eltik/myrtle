import { AccountOptimizer } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// AccountOptimizer is the profile Optimizer tab's first card: what it takes to
// bring every owned operator to its level cap, or (switch) only to the level
// its module unlocks at. The figure is computed on demand: the Calculate
// button runs the server's max-level cost walk, then headline figures (EXP and
// LMD needed vs. held, income per day, days to afford) and a per-operator
// table appear. The server function is stubbed to fail in the design bundle,
// so after Calculate the honest render is the card's error line. The switch
// is a per-viewer preference kept in localStorage (set here during render, so
// each story owns its state).

const KEY = "account-optimizer-module-target";

function Stage({ moduleTarget, children }: { moduleTarget: boolean; children: ReactNode }) {
    try {
        window.localStorage.setItem(KEY, JSON.stringify(moduleTarget));
    } catch {
        // Storage blocked: the switch reads off.
    }
    return <div style={{ width: 820 }}>{children}</div>;
}

/** Presses Calculate after first paint, then drops the focus ring. */
function CalculateOnMount({ children }: { children: ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let f2 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                const buttons = ref.current?.querySelectorAll<HTMLButtonElement>("button");
                buttons?.[buttons.length - 1]?.click();
                (document.activeElement as HTMLElement | null)?.blur();
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
        };
    }, []);
    return <div ref={ref}>{children}</div>;
}

const roster: never[] = [];

/** Before anything runs: the blurb, the module-target switch off, Calculate. */
export const Idle = () => (
    <Stage moduleTarget={false}>
        <AccountOptimizer uid="18220561" roster={roster} operatorsStatic={[]} />
    </Stage>
);

/** The module-target switch on: the blurb says the walk stops at each module's unlock level. */
export const ModuleTarget = () => (
    <Stage moduleTarget>
        <AccountOptimizer uid="18220561" roster={roster} operatorsStatic={[]} />
    </Stage>
);

/** Calculate pressed and the cost walk failed: the error line under the card. */
export const CalculateFailed = () => (
    <Stage moduleTarget={false}>
        <CalculateOnMount>
            <AccountOptimizer uid="18220561" roster={roster} operatorsStatic={[]} />
        </CalculateOnMount>
    </Stage>
);
