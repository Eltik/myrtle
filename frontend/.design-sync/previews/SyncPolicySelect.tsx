import { SyncPolicySelect } from "frontend";
import { useEffect, useRef } from "react";

// SyncPolicySelect is the one setting for reading-progress sync: what to do
// when this browser and the account disagree: "Ask me", "Merge" (the
// default), "Account wins" or "Browser wins". It sits
// at the foot of the sync conflict card and on the Progress tab.

/** At rest, on the default policy (Merge). */
export const Default = () => (
    <div style={{ width: 420 }}>
        <SyncPolicySelect />
    </div>
);

/** Open: the four policies. */
export const Open = () => {
    const root = useRef<HTMLDivElement | null>(null);
    useEffect(() => {
        let raf = requestAnimationFrame(() => {
            raf = requestAnimationFrame(() => root.current?.querySelector<HTMLElement>("[aria-haspopup]")?.click());
        });
        return () => cancelAnimationFrame(raf);
    }, []);
    return (
        <div ref={root} style={{ width: 420, minHeight: 280, paddingTop: 96 }}>
            <SyncPolicySelect />
        </div>
    );
};
