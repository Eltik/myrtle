import { OfficialTierLists } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// `OfficialTierLists` owns its dialog state with useState and exposes no `open`
// prop, so the only way to photograph the create flow is to click the primary
// action after Base UI has wired it — two rAFs after mount.
const AutoOpen = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => ref.current?.querySelector("button")?.click());
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, []);
    return (
        <div className="relative min-h-[520px] w-full" ref={ref}>
            {children}
        </div>
    );
};

// The browse query is a stubbed server function in a preview, so the table
// settles on its "no official lists yet" branch below a fully rendered toolbar.
export function NoOfficialLists() {
    return <OfficialTierLists />;
}

export function NewListDialogOpen() {
    return (
        <AutoOpen>
            <OfficialTierLists />
        </AutoOpen>
    );
}
