import { FilterSheet } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// `FilterSheet` owns its open state internally (`React.useState`), so there is no
// `open` prop to force. `AutoOpen` clicks the trigger on mount so the side sheet
// itself - the part worth looking at - is what gets photographed.
// The trigger is `lg:hidden`: this is the below-1024px toolbar of the birthday page.
const AutoOpen = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        // Base UI wires the trigger after the first paint, so a click fired
        // straight from the effect is dropped - wait two frames.
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => ref.current?.querySelector("button")?.click());
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, []);
    return (
        <div className="relative min-h-[520px] w-full" ref={ref}>
            {children}
        </div>
    );
};

const NATIONS: [string, string][] = [
    ["columbia", "Columbia"],
    ["kazimierz", "Kazimierz"],
    ["laterano", "Laterano"],
    ["lungmen", "Lungmen"],
    ["rhodes", "Rhodes Island"],
    ["siracusa", "Siracusa"],
    ["victoria", "Victoria"],
    ["yan", "Yan"],
];

const filters = (query: string, rarities: number[], professions: string[], nations: string[]) => ({
    query,
    rarities: new Set(rarities),
    professions: new Set(professions),
    nations: new Set(nations),
});

const noop = () => {};

// Resting sheet: nothing filtered yet, so Reset is disabled and the footer
// offers the full roster.
export const OpenNoFilters = () => (
    <AutoOpen>
        <FilterSheet activeCount={0} filters={filters("", [], [], [])} matched={382} nations={NATIONS} onChange={noop} onReset={noop} total={382} />
    </AutoOpen>
);

// Three facets on: the trigger badge, the header count, the "clear" links inside
// FilterControls and the enabled Reset all light up together.
export const OpenWithActiveFilters = () => (
    <AutoOpen>
        <FilterSheet activeCount={3} filters={filters("silence", [6], ["MEDIC"], ["columbia"])} matched={2} nations={NATIONS} onChange={noop} onReset={noop} total={382} />
    </AutoOpen>
);

// The closed resting state, in the page toolbar it actually lives in: the badge
// is the only signal that filters are applied while the sheet is dismissed.
export const InPageToolbar = () => (
    <div className="flex max-w-2xl items-center justify-between gap-3 border-border border-b pb-4">
        <div>
            <h2 className="m-0 font-bold font-sans text-[17px] text-foreground tracking-tight">Birthday Calendar</h2>
            <p className="mt-1 font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">31 of 382 match</p>
        </div>
        <FilterSheet activeCount={3} filters={filters("silence", [6], [], ["columbia"])} matched={31} nations={NATIONS} onChange={noop} onReset={noop} total={382} />
    </div>
);
