import { ChevronLeft, ChevronRight, Filter } from "lucide-react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";

export function FilterToggleButton({ visible, onToggle, activeCount }: { visible: boolean; onToggle: () => void; activeCount: number }) {
    return (
        <Tooltip>
            <TooltipTrigger
                render={
                    <button
                        type="button"
                        data-on={visible || undefined}
                        className="relative box-border inline-flex h-9 w-9 shrink-0 cursor-pointer appearance-none items-center justify-center gap-1 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] p-0 font-[inherit] text-muted-foreground transition-[background-color,border-color,color,box-shadow] duration-300 ease-[cubic-bezier(0.32,0.72,0,1)] hover:border-[color-mix(in_oklch,var(--primary)_55%,var(--border))] hover:bg-card hover:text-foreground focus-visible:border-primary focus-visible:shadow-[0_0_0_3px_color-mix(in_oklch,var(--primary)_24%,transparent)] focus-visible:outline-none data-on:border-[color-mix(in_oklch,var(--primary)_55%,var(--border))] data-on:bg-card data-on:text-foreground motion-reduce:transition-none md:w-auto md:min-w-20 md:px-1.5"
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
