"use client";

import { Tabs as TabsPrimitive } from "@base-ui/react/tabs";
import type React from "react";
import { type RefObject, useEffect, useRef, useState } from "react";
import { cn } from "#/lib/utils";

export type TabsVariant = "default" | "underline";

export function Tabs({ className, ...props }: TabsPrimitive.Root.Props): React.ReactElement {
    return <TabsPrimitive.Root className={cn("flex flex-col gap-2 data-[orientation=vertical]:flex-row", className)} data-slot="tabs" {...props} />;
}

/** Room kept clear past the active tab when it is scrolled into view, so its neighbour shows. */
const SCROLL_STRIP_MARGIN_PX = 24;

/**
 * A horizontal tab strip that scrolls: reports whether it overflows (for
 * `.scroll-shadows-x`) and keeps the selected tab in view whenever the selection
 * changes, without scrolling the page. Exported for strips that are not a kit
 * `TabsList` (the profile bar, a single-select `ToggleGroup` chip strip, whose
 * selection is `aria-pressed`).
 */
export function useTabStripScroll(ref: RefObject<HTMLElement | null>): boolean {
    const [overflowing, setOverflowing] = useState(false);
    useEffect(() => {
        const el = ref.current;
        if (!el) return;
        const measure = () => setOverflowing(el.scrollWidth > el.clientWidth + 1);
        const reveal = (smooth: boolean) => {
            const tab = el.querySelector<HTMLElement>('[role="tab"][aria-selected="true"], [aria-pressed="true"]');
            if (!tab || el.scrollWidth <= el.clientWidth + 1) return;
            const box = el.getBoundingClientRect();
            const r = tab.getBoundingClientRect();
            let delta = 0;
            if (r.left < box.left + SCROLL_STRIP_MARGIN_PX) delta = r.left - box.left - SCROLL_STRIP_MARGIN_PX;
            else if (r.right > box.right - SCROLL_STRIP_MARGIN_PX) delta = r.right - box.right + SCROLL_STRIP_MARGIN_PX;
            if (delta === 0) return;
            const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
            el.scrollBy({ left: delta, behavior: smooth && !reduce ? "smooth" : "auto" });
        };
        measure();
        reveal(false);
        const resize = new ResizeObserver(measure);
        resize.observe(el);
        for (const child of Array.from(el.children)) resize.observe(child);
        // Child-list changes (late tab counts) only re-measure; scrolling back to the selected
        // tab on them yanked a strip the reader had scrolled by hand.
        const selection = new MutationObserver((records) => {
            measure();
            if (records.some((r) => r.type === "attributes")) reveal(true);
        });
        selection.observe(el, { subtree: true, childList: true, attributes: true, attributeFilter: ["aria-selected", "aria-pressed"] });
        return () => {
            resize.disconnect();
            selection.disconnect();
        };
    }, [ref]);
    return overflowing;
}

export function TabsList({
    variant = "default",
    className,
    children,
    ...props
}: TabsPrimitive.List.Props & {
    variant?: TabsVariant;
}): React.ReactElement {
    const ref = useRef<HTMLDivElement>(null);
    const overflowing = useTabStripScroll(ref);
    return (
        <TabsPrimitive.List
            ref={ref}
            data-overflow={overflowing ? "" : undefined}
            className={cn(
                "relative z-0 flex w-fit items-center justify-center gap-x-0.5 text-muted-foreground",
                "data-[orientation=vertical]:flex-col",
                // A horizontal strip wider than its parent scrolls instead of being cut: no
                // scrollbar (it was a grey bar under the story tabs at 375px), an edge shadow
                // while there is more past it, and every tab still reached by Tab and arrows.
                // `overflow-y-hidden`: `overflow-x: auto` turns the y axis to auto too, and the
                // coarse 44px hit areas overhang the list, which made it a vertical scroller.
                "[scrollbar-width:none] data-[orientation=horizontal]:max-w-full data-[orientation=horizontal]:overflow-x-auto data-[orientation=horizontal]:overflow-y-hidden data-[orientation=horizontal]:overscroll-x-contain [&::-webkit-scrollbar]:hidden",
                "data-[orientation=horizontal]:data-overflow:scroll-shadows-x data-[orientation=horizontal]:data-overflow:justify-start",
                variant === "default"
                    ? "rounded-lg bg-muted p-0.5 text-muted-foreground [--scroll-shadow-bg:var(--muted)] *:data-[slot=tabs-tab]:not-data-active:hover:bg-background/60 dark:*:data-[slot=tabs-tab]:not-data-active:hover:bg-foreground/6"
                    : "data-[orientation=vertical]:px-1 data-[orientation=horizontal]:py-1 *:data-[slot=tabs-tab]:hover:bg-accent",
                className,
            )}
            data-slot="tabs-list"
            {...props}
        >
            {children}
            {/* THE INDICATOR MUST NOT LEAVE THE PADDING BOX. It used to be nudged
                one pixel down (`translate-y-px`) so the 2 px underline covered the
                1 px `border-b` a caller adds, and an absolutely positioned child
                one pixel past the padding box is SCROLLABLE OVERFLOW. The two tab
                strips that are also horizontal scrollers (`overflow-x: auto`
                computes `overflow-y: visible` to `auto`) therefore measured
                scrollHeight 41 against clientHeight 40 and drew a permanent 8 px
                vertical scrollbar, on the story Illustrations strip and in the
                operator dialog alike. `translate-y-0` pins the bar to the padding
                box edge, which is where the hairline starts: it sits ON the
                border instead of over it, one pixel of difference, and the strip
                stops being a vertical scroller.

                The zero is EXPLICIT and it is not a no-op. Both this and the
                base class's `-translate-y-(--active-tab-bottom)` write
                `--tw-translate-y`, so they do not add; the variant wins on
                specificity. Dropping the rule entirely would hand the base class
                back and lift the bar by `--active-tab-bottom`, measured 4 px,
                leaving it floating clear of the hairline it is meant to sit on. */}
            <TabsPrimitive.Indicator
                className={cn(
                    "absolute bottom-0 left-0 h-(--active-tab-height) w-(--active-tab-width) translate-x-(--active-tab-left) -translate-y-(--active-tab-bottom) transition-[width,translate] duration-200 ease-in-out",
                    variant === "underline" ? "z-10 bg-primary data-[orientation=horizontal]:h-0.5 data-[orientation=vertical]:w-0.5 data-[orientation=vertical]:-translate-x-px data-[orientation=horizontal]:translate-y-0" : "-z-1 rounded-md bg-background shadow-sm/5 dark:bg-input",
                )}
                data-slot="tab-indicator"
            />
        </TabsPrimitive.List>
    );
}

export function TabsTab({ className, ...props }: TabsPrimitive.Tab.Props): React.ReactElement {
    return (
        <TabsPrimitive.Tab
            className={cn(
                "relative flex h-9 pointer-coarse:h-11 shrink-0 grow cursor-pointer items-center justify-center gap-1.5 whitespace-nowrap rounded-md border border-transparent px-[calc(--spacing(2.5)-1px)] font-medium text-base outline-none transition-[color,background-color,box-shadow] pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:h-11 pointer-coarse:after:-translate-y-1/2 hover:text-foreground focus-visible:inset-ring-2 focus-visible:inset-ring-ring data-[orientation=vertical]:w-full data-disabled:cursor-not-allowed data-[orientation=vertical]:justify-start data-active:text-foreground data-disabled:opacity-64 sm:h-8 sm:text-sm [&_svg:not([class*='size-'])]:size-4.5 sm:[&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:-mx-0.5 [&_svg]:shrink-0",
                className,
            )}
            data-slot="tabs-tab"
            {...props}
        />
    );
}

export function TabsPanel({ className, ...props }: TabsPrimitive.Panel.Props): React.ReactElement {
    return <TabsPrimitive.Panel className={cn("flex-1 outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background", className)} data-slot="tabs-content" {...props} />;
}

export { TabsPrimitive, TabsTab as TabsTrigger, TabsPanel as TabsContent };
