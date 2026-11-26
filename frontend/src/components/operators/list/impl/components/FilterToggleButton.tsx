import { ChevronLeft, ChevronRight, Filter } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./FilterToggleButton.messages";

/**
 * Toolbar toggle for the filter panel. Below md it is the compact 24px chevron, with a 44px hit area on touch; from md up it is
 * the 36px "Filter" button with the funnel icon (desktop, since 2026-09-03). Both surfaces (the
 * /operators page and the profile roster) render this one element. Open draws the kit Toggle's
 * pressed look through `data-pressed`, but it stays a Button: it is a disclosure (aria-expanded),
 * and a Toggle would add aria-pressed on top.
 */
export function FilterToggleButton({ visible, onToggle, activeCount }: { visible: boolean; onToggle: () => void; activeCount: number }) {
    const t: TypedT<typeof messages> = useT("operators");
    const toggleLabel = visible ? t("filters.toggle.hide") : t("filters.toggle.show");

    return (
        <Tooltip>
            <TooltipTrigger
                render={
                    <Button
                        variant="outline"
                        size="icon-xs"
                        data-pressed={visible || undefined}
                        className="text-muted-foreground hover:text-foreground data-pressed:inset-ring data-pressed:inset-ring-ring data-pressed:bg-primary/12 data-pressed:text-foreground md:h-9 md:w-auto md:min-w-20 md:gap-1 md:rounded-lg md:px-1.5 md:before:rounded-[calc(var(--radius-lg)-1px)] dark:data-pressed:bg-primary/16"
                        onClick={onToggle}
                        aria-label={toggleLabel}
                        aria-expanded={visible}
                    />
                }
            >
                <span className="hidden border-border border-r pr-1 font-medium font-mono text-[10px] uppercase leading-none tracking-[0.12em] md:inline">{t("filters.toggle.label")}</span>
                <Filter className="hidden h-3.5 w-3.5 md:block" aria-hidden="true" />
                {visible ? <ChevronLeft className="block h-3.5 w-3.5 md:hidden" aria-hidden="true" /> : <ChevronRight className="block h-3.5 w-3.5 md:hidden" aria-hidden="true" />}
                {!visible && activeCount > 0 && (
                    <Badge className="absolute -top-1.5 -right-1.5 font-mono tabular-nums ring-2 ring-background" size="sm">
                        {activeCount}
                    </Badge>
                )}
            </TooltipTrigger>
            <TooltipPopup side="top" sideOffset={8}>
                {toggleLabel}
            </TooltipPopup>
        </Tooltip>
    );
}
