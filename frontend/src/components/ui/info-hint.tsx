import { Info } from "lucide-react";
import type React from "react";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";

/**
 * An explanation attached to the thing it explains.
 *
 * Reported in #ui-ux: a note at the BOTTOM of a card was read as a note about the
 * card rather than about the one field it qualified. `openOnHover` keeps it a hover
 * on a pointer, and it is still a button, so a tap opens it and a keyboard reaches it.
 */
export function InfoHint({ label, children }: { label: string; children: React.ReactNode }): React.ReactElement {
    return (
        <Popover>
            <PopoverTrigger
                openOnHover
                delay={120}
                render={(props) => (
                    <button {...props} type="button" aria-label={label} className="inline-flex size-4 cursor-help items-center justify-center rounded-full text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring focus-visible:outline-offset-2">
                        <Info className="size-3.5" aria-hidden="true" />
                    </button>
                )}
            />
            <PopoverPopup className="w-[min(320px,calc(100vw-2rem))]">
                <p className="m-0 font-sans text-[12.5px] text-foreground leading-normal">{children}</p>
            </PopoverPopup>
        </Popover>
    );
}
