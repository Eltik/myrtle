/**
 * THE SHELL EVERY SETTINGS SECTION IS BUILT OUT OF: one row, and the four class
 * strings that make its controls touchable.
 *
 * It sits apart from `SettingsDialog.tsx` because four files now render rows
 * into that dialog, and a row's layout is a decision about the DIALOG, not
 * about any one setting in it.
 */
import type React from "react";

/**
 * A settings row. At 640 px and up it is the three-column grid the desktop
 * dialog has always used. Under 640 px the label and its value sit on one line
 * and the control takes the full width below them: the 10rem label plus a
 * 4rem value column left 175 px for a slider at 375 px wide, which is not a
 * usable target. Every control in the sheet is at least 44 px tall.
 */
export function Row({ label, value, hint, children }: { label: string; value?: string; hint?: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-1.5 text-sm sm:grid sm:grid-cols-[minmax(0,10rem)_1fr_auto] sm:items-center sm:gap-3">
            <div className="flex items-baseline justify-between gap-3 sm:contents">
                <span className="text-muted-foreground">{label}</span>
                <span className="order-last font-mono text-muted-foreground text-xs tabular-nums sm:w-16 sm:text-end">{value}</span>
            </div>
            <div className="flex w-full min-w-0 items-center max-sm:min-h-11">{children}</div>
            {/* `order-last` matches the value column's: without it a row that
                carries BOTH a value and a hint places the hint before the
                value, and the value drops onto a line of its own. */}
            {hint ? <p className="order-last text-muted-foreground text-xs sm:col-span-3">{hint}</p> : null}
        </div>
    );
}

/**
 * Touch sizing for the sheet. A slider's draggable thumb is 20 px and a switch
 * is 22 px tall by design across the whole site, so rather than inflate the
 * controls themselves these grow the ROW that accepts the pointer: the slider
 * control box and the label wrapping a switch both become 44 px targets.
 */
export const TOUCH_SLIDER = "max-sm:[&_[data-slot=slider-control]]:h-11 max-sm:[&_[data-slot=slider-control]]:items-center";
export const TOUCH_INPUT = "max-sm:h-11 max-sm:[&_[data-slot=input]]:h-11 max-sm:[&_[data-slot=input]]:leading-11";
export const TOUCH_CLOSE = "absolute end-2 top-2 max-sm:size-11";

// A switch is 22 px tall everywhere on this site. On a phone the track grows to
// 30 px and an invisible `after` box, the same device `button.tsx` uses for its
// coarse-pointer targets, carries the 44 px touch height without changing how
// the control looks.
export const TOUCH_SWITCH = "max-sm:relative max-sm:[--thumb-size:--spacing(7)] max-sm:after:absolute max-sm:after:inset-x-0 max-sm:after:top-1/2 max-sm:after:h-11 max-sm:after:-translate-y-1/2 max-sm:after:content-['']";

/** The one button size every control in the sheet uses, for the same 44 px reason. */
export const TOUCH_BUTTON = "max-sm:h-11";
