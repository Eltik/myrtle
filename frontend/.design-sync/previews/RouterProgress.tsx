import type { ReactNode } from "react";
import { RouterProgress } from "frontend";

// The bar is `position: fixed; top: 0`. `translateZ(0)` on this frame makes it
// the containing block, so the bar pins to the top of the mock app shell
// instead of the story root — the same place it sits under the real header.
const AppShell = ({ children, className }: { children: ReactNode; className?: string }) => (
    <div className={`relative overflow-hidden rounded-xl border border-border bg-background ${className ?? ""}`} style={{ transform: "translateZ(0)" }}>
        {children}
        <header className="flex h-14 items-center justify-between border-border border-b px-4">
            <div className="inline-flex items-center gap-2.5 font-sans font-semibold text-foreground text-sm leading-none">
                <span className="inline-flex size-6 items-center justify-center rounded-md bg-primary font-bold font-mono text-[11px] text-primary-foreground">M</span>
                myrtle.moe
            </div>
            <nav className="inline-flex items-center gap-4 font-sans text-[12.5px] text-muted-foreground">
                <span className="text-foreground">Operators</span>
                <span>Stages</span>
                <span>Gacha</span>
                <span>Base</span>
            </nav>
        </header>
        <div className="flex flex-col gap-3 p-6">
            <span className="font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Operators · 438 indexed</span>
            <div className="grid grid-cols-4 gap-3">
                {["Mlynar", "Skadi", "Muelsyse", "Texas the Omertosa"].map((n) => (
                    <div key={n} className="flex flex-col gap-2 rounded-xl border border-border bg-card px-4 py-3.5">
                        <span className="font-sans font-semibold text-[13px] text-foreground">{n}</span>
                        <span className="font-mono text-[10.5px] text-muted-foreground">6★ · E2 90</span>
                    </div>
                ))}
            </div>
        </div>
    </div>
);

/** Resting state: the router is idle, so the bar is transparent and unscaled. */
export const AtRest = () => (
    <AppShell>
        <RouterProgress />
    </AppShell>
);

/**
 * Mid-navigation. The component derives its own visibility from
 * `useRouterState`, which never leaves the idle state in a static screenshot,
 * so the story pins the trickle appearance with a scoped stylesheet override.
 */
export const Navigating = () => (
    <AppShell className="ds-routerprogress-active">
        <style>{`.ds-routerprogress-active > div[aria-hidden] { opacity: 1 !important; transform: scaleX(0.68) !important; }`}</style>
        <RouterProgress />
    </AppShell>
);
