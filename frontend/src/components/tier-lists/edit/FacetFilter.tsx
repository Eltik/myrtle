import { useEffect, useId, useMemo, useRef, useState } from "react";
import { Combobox, ComboboxChip, ComboboxChips, ComboboxChipsInput, ComboboxEmpty, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxValue } from "#/components/ui/combobox";
import { Field, FieldLabel } from "#/components/ui/field";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import { cn } from "#/lib/utils";
import { type IFacetOption, useKindT } from "../kinds";
import type { IPoolFacet } from "./poolKinds";

// A pool facet's filter row, shared by the tier-list pool dialog (`KindPool`)
// and the grid picker. Icon options show their label in a tooltip, so the
// caller renders it under a `TooltipProvider`.

const EDGE_FADE_PX = 24;

/**
 * Fades whichever edge of a horizontal scroller has content hidden past it, so
 * a cut-off row does not look complete. Also the grid picker's tabs and the
 * grid board's own scroller.
 */
export function useEdgeFade<T extends HTMLElement>() {
    const ref = useRef<T>(null);
    const [edges, setEdges] = useState({ start: false, end: false });
    useEffect(() => {
        const el = ref.current;
        if (!el) return;
        const update = () => {
            const start = el.scrollLeft > 1;
            const end = el.scrollLeft + el.clientWidth < el.scrollWidth - 1;
            setEdges((prev) => (prev.start === start && prev.end === end ? prev : { start, end }));
        };
        update();
        el.addEventListener("scroll", update, { passive: true });
        // The row's own box stays the same width when its chips change, so watch the content too.
        const observer = new ResizeObserver(update);
        observer.observe(el);
        if (el.firstElementChild) observer.observe(el.firstElementChild);
        return () => {
            el.removeEventListener("scroll", update);
            observer.disconnect();
        };
    }, []);
    if (!edges.start && !edges.end) return { ref, style: undefined };
    const mask = `linear-gradient(to right, ${edges.start ? "transparent" : "black"}, black ${EDGE_FADE_PX}px, black calc(100% - ${EDGE_FADE_PX}px), ${edges.end ? "transparent" : "black"})`;
    return { ref, style: { maskImage: mask, WebkitMaskImage: mask } };
}

/** One facet's row of toggles: the pool dialog's, and the grid picker's. */
export function FacetFilter({ facet, value, onChange }: { facet: IPoolFacet; value: string[]; onChange: (next: string[]) => void }) {
    // A data-fed menu with nothing to offer (a backend from before the facet) shows no row.
    if (facet.variant === "menu") return facet.options.length > 0 ? <FacetMenu facet={facet} value={value} onChange={onChange} /> : null;
    return <FacetToggles facet={facet} value={value} onChange={onChange} />;
}

/**
 * A long facet (19 nations, 25 factions, 37 races) as a searchable dropdown whose
 * choices sit as chips in its field: a row of that many toggles would scroll
 * most of them out of sight.
 */
function FacetMenu({ facet, value, onChange }: { facet: IPoolFacet; value: string[]; onChange: (next: string[]) => void }) {
    const t = useKindT();
    const id = useId();
    const byValue = useMemo(() => new Map(facet.options.map((o) => [o.value, o])), [facet.options]);
    // A chosen value the catalogue no longer offers (another server's list) drops out of the chips, not out of the filter.
    const selected = useMemo(() => value.map((v) => byValue.get(v)).filter((o): o is IFacetOption => o !== undefined), [value, byValue]);
    return (
        <Field className="min-w-0 max-w-full gap-1.5 sm:flex-row sm:items-center sm:gap-2">
            <FieldLabel htmlFor={id} className="whitespace-nowrap font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.16em]">
                {facet.label}
            </FieldLabel>
            <Combobox<IFacetOption, true> multiple items={facet.options} value={selected} onValueChange={(next) => onChange(next.map((o) => o.value))} itemToStringLabel={(o) => o.label} itemToStringValue={(o) => o.value} isItemEqualToValue={(a, b) => a.value === b.value}>
                <ComboboxChips className="min-w-0 sm:max-w-md">
                    <ComboboxValue>
                        {(chips: IFacetOption[]) => (
                            <>
                                {chips.map((o) => (
                                    <ComboboxChip key={o.value} aria-label={o.label}>
                                        {o.label}
                                    </ComboboxChip>
                                ))}
                                <ComboboxChipsInput id={id} size="sm" aria-label={facet.groupLabel} placeholder={chips.length === 0 ? facet.placeholder : ""} />
                            </>
                        )}
                    </ComboboxValue>
                </ComboboxChips>
                <ComboboxPopup className="w-[min(300px,calc(100vw-2rem))]">
                    <ComboboxEmpty>{t("edit.pool.menu.noMatches")}</ComboboxEmpty>
                    <ComboboxList>
                        {(o: IFacetOption) => (
                            <ComboboxItem key={o.value} value={o}>
                                <span className="flex items-center gap-2">
                                    {o.icon && (
                                        <span aria-hidden="true" className="inline-flex size-[18px] shrink-0 items-center justify-center">
                                            {o.icon}
                                        </span>
                                    )}
                                    <span className="truncate">{o.label}</span>
                                </span>
                            </ComboboxItem>
                        )}
                    </ComboboxList>
                </ComboboxPopup>
            </Combobox>
        </Field>
    );
}

function FacetToggles({ facet, value, onChange }: { facet: IPoolFacet; value: string[]; onChange: (next: string[]) => void }) {
    const fade = useEdgeFade<HTMLDivElement>();
    return (
        // min-w-0: a long facet (24 skin brands) has to shrink to the dialog so its row scrolls instead of overflowing.
        // The row's vertical padding, cancelled by its margin, is room for a toggle's 44 px touch area: the scroller clips it otherwise.
        <Field className="min-w-0 max-w-full gap-1.5 sm:flex-row sm:items-center sm:gap-2">
            <FieldLabel className="whitespace-nowrap font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.16em]">{facet.label}</FieldLabel>
            <div ref={fade.ref} style={fade.style} className="-mx-1 -my-1.5 flex min-w-0 overflow-x-auto px-1 py-1.5 [-ms-overflow-style:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                <ToggleGroup value={value} onValueChange={(v) => onChange(v as string[])} aria-label={facet.groupLabel} multiple variant="outline" size="sm" className="flex-nowrap">
                    {facet.options.map((option) =>
                        facet.variant === "icon" ? (
                            <Tooltip key={option.value}>
                                <TooltipTrigger
                                    render={
                                        <ToggleGroupItem value={option.value} aria-label={option.ariaLabel ?? option.label} className="shrink-0 px-1.5 [&:not([data-pressed])>img]:opacity-40">
                                            {option.icon}
                                        </ToggleGroupItem>
                                    }
                                />
                                <TooltipContent>{option.label}</TooltipContent>
                            </Tooltip>
                        ) : (
                            <ToggleGroupItem key={option.value} value={option.value} aria-label={option.ariaLabel} className={cn("shrink-0 [&:not([data-pressed])]:opacity-55", facet.variant === "mono" && "font-mono tabular-nums")}>
                                {option.label}
                            </ToggleGroupItem>
                        ),
                    )}
                </ToggleGroup>
            </div>
        </Field>
    );
}
