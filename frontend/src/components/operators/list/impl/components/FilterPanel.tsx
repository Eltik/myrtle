import { Filter, X } from "lucide-react";
import type { CSSProperties, ReactNode } from "react";
import { Sheet, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { useMediaQuery } from "#/hooks/use-media-query";
import { cn } from "#/lib/utils";
import styles from "./OperatorFilters.module.css";

interface IFilterPanelProps {
    collapsed?: boolean;
    onToggle?: () => void;
    hasActiveFilters: boolean;
    onClearAll: () => void;
    activeFilterCount?: number;
    /** aria-label for the desktop <aside>. */
    ariaLabel: string;
    /**
     * Bottom edge, in px, of the sticky chrome the page keeps above the panel.
     * Defaults to the 64px site header; a page with more sticky rows (the profile
     * tab bar) passes its own so the sidebar sticks below them instead of under.
     */
    stickyOffset?: number;
    children: ReactNode;
}

/**
 * Ensures every operator-list surface shares one sidebar/sheet chrome instead of
 * re-deriving the breakpoint, inert handling, and header.
 */
export function FilterPanel(props: IFilterPanelProps) {
    const isMobile = useMediaQuery("max-md");
    const isOpen = !props.collapsed;

    if (isMobile) {
        return (
            <Sheet open={isOpen} onOpenChange={(open) => !open && props.onToggle?.()}>
                <SheetPopup side="left" variant="inset" className="max-w-80">
                    <SheetHeader className="pb-4">
                        <div className="flex items-center justify-between">
                            <SheetTitle className="flex items-center gap-2 text-base">
                                <Filter className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
                                Filters
                                {props.activeFilterCount && props.activeFilterCount > 0 ? <span className="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 font-mono font-semibold text-[10px] text-primary-foreground">{props.activeFilterCount}</span> : null}
                            </SheetTitle>
                            {props.hasActiveFilters && (
                                <button type="button" className="inline-flex items-center gap-1 font-medium text-[11.5px] text-muted-foreground hover:text-foreground" onClick={props.onClearAll}>
                                    <X className="h-2.5 w-2.5" aria-hidden="true" />
                                    Clear all
                                </button>
                            )}
                        </div>
                    </SheetHeader>
                    <SheetPanel className="px-5 pb-6">{props.children}</SheetPanel>
                </SheetPopup>
            </Sheet>
        );
    }

    const style = props.stickyOffset === undefined ? undefined : ({ "--filter-sidebar-offset": `${props.stickyOffset}px` } as CSSProperties);

    return (
        <aside className={cn(styles.filterSidebar, props.collapsed && styles.filterSidebarCollapsed)} style={style} aria-label={props.ariaLabel} aria-hidden={props.collapsed || undefined} {...(props.collapsed ? { inert: "" as unknown as boolean } : {})}>
            <div className={styles.filterSidebarInner}>
                <div className={styles.fpHead}>
                    <h3>
                        <Filter className="h-3.5 w-3.5" aria-hidden="true" />
                        Filters
                    </h3>
                    {props.hasActiveFilters && (
                        <button type="button" className={styles.clear} onClick={props.onClearAll}>
                            <X className="h-2.5 w-2.5" aria-hidden="true" />
                            Clear all
                        </button>
                    )}
                </div>
                {props.children}
            </div>
        </aside>
    );
}
