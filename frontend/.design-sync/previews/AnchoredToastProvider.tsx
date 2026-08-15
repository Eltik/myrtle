import { Toast } from "@base-ui/react/toast";
import { AnchoredToastProvider, Button } from "frontend";
import { CopyIcon, Share2Icon } from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";

// The anchored provider positions each toast against the element that raised it,
// instead of a fixed viewport corner: it is for "copied", "saved" and other
// confirmations that belong next to the control the Doctor just pressed.
// `data.tooltipStyle` switches the popup to the compact, tooltip-shaped surface.

type Seed = {
    title: string;
    description?: string;
    type?: "success" | "error" | "info" | "warning";
    data?: { tooltipStyle?: boolean };
};

function Stage({ seed, side, children }: { seed: Seed; side: "top" | "bottom"; children: ReactNode }) {
    const [manager] = useState(() => Toast.createToastManager());
    const [anchor, setAnchor] = useState<HTMLElement | null>(null);

    useEffect(() => {
        if (!anchor) {
            return;
        }
        manager.add({ ...seed, timeout: 0, positionerProps: { anchor, side, sideOffset: 8 } });
    }, [anchor, manager, seed, side]);

    return (
        <AnchoredToastProvider timeout={0} toastManager={manager}>
            <div className="flex min-h-[520px] w-full items-start justify-center pt-16">
                <span className="mt-16 inline-flex" ref={setAnchor}>
                    {children}
                </span>
            </div>
        </AnchoredToastProvider>
    );
}

const COPIED: Seed = { title: "Build link copied to clipboard", data: { tooltipStyle: true } };

const SAVED: Seed = { title: "Roster snapshot saved", description: "231 operators captured from the EN server.", type: "success" };

/** `tooltipStyle` — the compact confirmation that sits right under the button. */
export const TooltipStyle = () => (
    <Stage seed={COPIED} side="bottom">
        <Button variant="outline">
            <CopyIcon />
            Copy build link
        </Button>
    </Stage>
);

/** The full toast surface, anchored above the control that raised it. */
export const AnchoredAbove = () => (
    <Stage seed={SAVED} side="top">
        <Button>
            <Share2Icon />
            Save roster snapshot
        </Button>
    </Stage>
);
