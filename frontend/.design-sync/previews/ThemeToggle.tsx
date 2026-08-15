import { ThemeToggle } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The appearance control in the header toolbar: a ghost icon button whose
// popover carries the light/dark/auto switch, the six accent presets, a custom
// colour swatch and the dynamic-art (L2D) toggle.
//
// The popover is uncontrolled, so the open stories click the trigger on mount -
// deferred two frames, because Base UI wires the trigger only after first paint.
const AutoOpen = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => {
                ref.current?.querySelector("button")?.click();
                // Opening moves focus onto the first control (Light), whose brand-red
                // ring otherwise reads as "Light is the active mode" - it is not, Auto
                // is. Drop the ring once the popup has settled.
                requestAnimationFrame(() => requestAnimationFrame(() => (document.activeElement as HTMLElement | null)?.blur()));
            });
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, []);
    return (
        <div className="flex min-h-[520px] w-full justify-center" ref={ref}>
            {children}
        </div>
    );
};

export const AppearanceMenu = () => (
    <AutoOpen>
        <ThemeToggle />
    </AutoOpen>
);

export const HeaderToolbar = () => (
    <div className="flex w-fit items-center gap-1.5 rounded-lg border border-border bg-background px-2 py-1.5">
        <span className="px-1 font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-widest">Appearance</span>
        <ThemeToggle />
    </div>
);
