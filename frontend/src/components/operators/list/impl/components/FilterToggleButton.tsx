import { ChevronLeft, ChevronRight, Filter } from "lucide-react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";

/**
 * Toolbar toggle for the filter panel. Below md it is the compact 24px chevron the
 * /operators toolbar has always shown on phones; from md up it is the 36px "Filter"
 * button with the funnel icon introduced on 2026-09-03 for desktop. Both surfaces
 * (the /operators page and the profile roster) render this one element.
 */
export function FilterToggleButton({ visible, onToggle, activeCount }: { visible: boolean; onToggle: () => void; activeCount: number }) {
    return (
        <Tooltip>
            <TooltipTrigger
                render={
                    <button
                        type="button"
                        data-on={visible || undefined}
                        className="relative box-border inline-flex h-6 w-6 shrink-0 cursor-pointer appearance-none items-center justify-center rounded-md border border-border bg-[color-mix(in_oklch,var(--secondary)_70%,var(--card))] p-0 font-[inherit] text-muted-foreground shadow-[0_1px_2px_color-mix(in_oklch,var(--foreground)_6%,transparent)] transition-[background-color,border-color,color,box-shadow] duration-300 ease-[cubic-bezier(0.32,0.72,0,1)] hover:border-[color-mix(in_oklch,var(--primary)_55%,var(--border))] hover:bg-card hover:text-foreground hover:shadow-[0_2px_8px_color-mix(in_oklch,var(--foreground)_10%,transparent)] focus-visible:border-primary focus-visible:shadow-[0_0_0_3px_color-mix(in_oklch,var(--primary)_24%,transparent)] focus-visible:outline-none motion-reduce:transition-none md:h-9 md:w-auto md:min-w-20 md:gap-1 md:rounded-lg md:bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] md:px-1.5 md:shadow-none md:data-on:border-[color-mix(in_oklch,var(--primary)_55%,var(--border))] md:data-on:bg-card md:data-on:text-foreground md:hover:shadow-none"
                        onClick={onToggle}
                        aria-label={visible ? "Hide filters" : "Show filters"}
                        aria-expanded={visible}
                    />
                }
            >
                <span className="hidden border-border border-r pr-1 font-medium font-mono text-[10px] uppercase leading-none tracking-[0.12em] md:inline">Filter</span>
                <Filter className="hidden h-3.5 w-3.5 md:block" aria-hidden="true" />
                {visible ? <ChevronLeft className="block h-3.5 w-3.5 md:hidden" aria-hidden="true" /> : <ChevronRight className="block h-3.5 w-3.5 md:hidden" aria-hidden="true" />}
                {!visible && activeCount > 0 && <span className="absolute -top-1.25 -right-1.25 inline-flex h-3.5 min-w-3.5 items-center justify-center rounded-full bg-primary px-0.75 font-mono font-semibold text-[9px] text-primary-foreground leading-none shadow-[0_0_0_2px_var(--background)]">{activeCount}</span>}
            </TooltipTrigger>
            <TooltipPopup side="top" sideOffset={8}>
                {visible ? "Hide filters" : "Show filters"}
            </TooltipPopup>
        </Tooltip>
    );
}
