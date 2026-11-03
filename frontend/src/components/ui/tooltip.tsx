"use client";

import { Tooltip as TooltipPrimitive } from "@base-ui/react/tooltip";
import React from "react";
import { cn } from "#/lib/utils";

export const TooltipCreateHandle: typeof TooltipPrimitive.createHandle = TooltipPrimitive.createHandle;

export const TooltipProvider: typeof TooltipPrimitive.Provider = TooltipPrimitive.Provider;

/**
 * Base UI opens a tooltip on mouse hover and keyboard focus only, so on a phone the hint
 * never shows. The kit Root owns the open state (unless the caller controls it) and hands
 * a setter to TooltipTrigger, which opens on a TAP when the trigger has no action of its
 * own. Base UI's dismiss already closes it on an outside press.
 */
type TouchToggle = { open: boolean; setOpen: (open: boolean) => void };
const TouchToggleContext = React.createContext<TouchToggle | null>(null);

export function Tooltip({ open: openProp, onOpenChange, ...props }: TooltipPrimitive.Root.Props): React.ReactElement {
    const [openState, setOpenState] = React.useState(props.defaultOpen ?? false);
    const controlled = openProp !== undefined;
    const open = controlled ? openProp : openState;
    const handleOpenChange = React.useCallback<NonNullable<TooltipPrimitive.Root.Props["onOpenChange"]>>(
        (next, details) => {
            if (!controlled) setOpenState(next);
            onOpenChange?.(next, details);
        },
        [controlled, onOpenChange],
    );
    // A controlled caller decides when it opens, so it gets no tap path: there is no
    // Base UI event-details object to hand its onOpenChange for a tap we synthesize.
    const touch = React.useMemo<TouchToggle | null>(() => (controlled ? null : { open, setOpen: setOpenState }), [controlled, open]);
    return (
        <TouchToggleContext.Provider value={touch}>
            <TooltipPrimitive.Root {...props} onOpenChange={handleOpenChange} open={open} />
        </TouchToggleContext.Provider>
    );
}

const INTERACTIVE_SELECTOR = 'a[href], button, input, select, textarea, summary, label, [role="button"], [role="link"], [role="tab"], [role="menuitem"], [role="option"], [role="checkbox"], [role="switch"], [role="radio"], [role="combobox"], [tabindex]:not([tabindex="-1"])';
// Handlers that only observe the pointer or focus; any other `on*` prop is an action.
const PASSIVE_HANDLER = /^on(Mouse(Enter|Leave|Over|Out|Move)|Pointer(Enter|Leave|Over|Out|Move)|Focus|Blur)/;

function declaresAction(props: Record<string, unknown> | undefined): boolean {
    if (!props) return false;
    if (props.href !== undefined || props.to !== undefined) return true;
    return Object.keys(props).some((key) => /^on[A-Z]/.test(key) && !PASSIVE_HANDLER.test(key) && typeof props[key] === "function");
}

/**
 * True when tapping the trigger already does something: a handler on the trigger or its
 * render element, a link or form control, a button that is not the bare default trigger,
 * or an interactive child under the finger. Those keep their click untouched.
 */
function hasOwnAction(props: TooltipPrimitive.Trigger.Props, element: HTMLElement, target: EventTarget | null): boolean {
    const { render } = props;
    if (declaresAction(props as Record<string, unknown>)) return true;
    if (React.isValidElement(render) && declaresAction(render.props as Record<string, unknown>)) return true;
    const inner = target instanceof Element ? target.closest(INTERACTIVE_SELECTOR) : null;
    if (inner && inner !== element && element.contains(inner)) return true;
    // With no render prop Base UI draws a plain `<button>` whose only job is the hint.
    if (render === undefined) return false;
    // A function render hides its props, so an interactive element is assumed to act.
    if (typeof render === "function") return element.matches(INTERACTIVE_SELECTOR);
    if (element.matches('a[href], input, select, textarea, summary, label, [role="link"], [role="tab"], [role="menuitem"], [role="option"], [role="checkbox"], [role="switch"], [role="radio"], [role="combobox"]')) return true;
    return element instanceof HTMLButtonElement && ((element.type === "submit" && element.form !== null) || element.hasAttribute("aria-pressed") || element.hasAttribute("aria-expanded") || element.hasAttribute("aria-haspopup"));
}

export function TooltipTrigger({ onPointerDown, onClick, ...props }: TooltipPrimitive.Trigger.Props): React.ReactElement {
    const touch = React.useContext(TouchToggleContext);
    // Read at pointerdown, before Base UI's dismiss closes an open tooltip on the same
    // press, so a second tap closes it instead of closing and reopening.
    const tapRef = React.useRef<{ wasOpen: boolean } | null>(null);
    const usesTouchPath = touch !== null && props.handle === undefined;
    return (
        <TooltipPrimitive.Trigger
            data-slot="tooltip-trigger"
            {...props}
            onClick={(event) => {
                onClick?.(event);
                const tap = tapRef.current;
                tapRef.current = null;
                // Keyboard activation has no pointerdown, so it never reaches the toggle.
                if (!usesTouchPath || tap === null || event.defaultPrevented) return;
                if (hasOwnAction({ ...props, onClick, onPointerDown }, event.currentTarget, event.target)) return;
                touch.setOpen(!tap.wasOpen);
            }}
            onPointerDown={(event) => {
                onPointerDown?.(event);
                tapRef.current = usesTouchPath && event.pointerType !== "mouse" ? { wasOpen: touch.open } : null;
            }}
        />
    );
}

export function TooltipPopup({
    className,
    align = "center",
    sideOffset = 4,
    side = "top",
    anchor,
    children,
    portalProps,
    ...props
}: TooltipPrimitive.Popup.Props & {
    align?: TooltipPrimitive.Positioner.Props["align"];
    side?: TooltipPrimitive.Positioner.Props["side"];
    sideOffset?: TooltipPrimitive.Positioner.Props["sideOffset"];
    anchor?: TooltipPrimitive.Positioner.Props["anchor"];
    portalProps?: TooltipPrimitive.Portal.Props;
}): React.ReactElement {
    // A popup portalled to <body> is invisible while another element is fullscreen (the
    // story reader, the archives network chart), so it joins the fullscreen element instead.
    const fullscreenContainer = typeof document !== "undefined" ? (document.fullscreenElement as HTMLElement | null) : null;
    return (
        <TooltipPrimitive.Portal container={fullscreenContainer ?? undefined} {...portalProps}>
            <TooltipPrimitive.Positioner align={align} anchor={anchor} className="z-70 h-(--positioner-height) w-(--positioner-width) max-w-(--available-width) transition-[top,left,right,bottom,transform] data-instant:transition-none" data-slot="tooltip-positioner" side={side} sideOffset={sideOffset}>
                <TooltipPrimitive.Popup
                    className={cn(
                        "relative flex h-(--popup-height,auto) w-(--popup-width,auto) origin-(--transform-origin) text-balance rounded-md border bg-popover not-dark:bg-clip-padding text-popover-foreground text-xs shadow-md/5 transition-[width,height,scale,opacity] before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--radius-md)-1px)] before:shadow-[0_1px_--theme(--color-black/4%)] data-ending-style:scale-98 data-starting-style:scale-98 data-ending-style:opacity-0 data-starting-style:opacity-0 data-instant:duration-0 dark:before:shadow-[0_-1px_--theme(--color-white/6%)]",
                        className,
                    )}
                    data-slot="tooltip-popup"
                    {...props}
                >
                    <TooltipPrimitive.Viewport
                        className="relative size-full overflow-clip px-(--viewport-inline-padding) py-1 [--viewport-inline-padding:--spacing(2)] data-instant:transition-none **:data-current:data-ending-style:opacity-0 **:data-current:data-starting-style:opacity-0 **:data-previous:data-ending-style:opacity-0 **:data-previous:data-starting-style:opacity-0 **:data-current:w-[calc(var(--popup-width)-2*var(--viewport-inline-padding)-2px)] **:data-previous:w-[calc(var(--popup-width)-2*var(--viewport-inline-padding)-2px)] **:data-previous:truncate **:data-current:opacity-100 **:data-previous:opacity-100 **:data-current:transition-opacity **:data-previous:transition-opacity"
                        data-slot="tooltip-viewport"
                    >
                        {children}
                    </TooltipPrimitive.Viewport>
                </TooltipPrimitive.Popup>
            </TooltipPrimitive.Positioner>
        </TooltipPrimitive.Portal>
    );
}

export { TooltipPrimitive, TooltipPopup as TooltipContent };
