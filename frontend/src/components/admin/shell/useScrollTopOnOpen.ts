import { useEffect, useRef } from "react";

/**
 * Opening a row from a stacked (phone) layout swaps the list out for the
 * detail pane; this starts the page at its top. Attach `listRef` to the list
 * pane: its display (not the viewport) says which layout is showing. Call
 * `arm()` when a click opens a row; `openId` is the URL's open row.
 */
export function useScrollTopOnOpen(openId: string | undefined): { listRef: React.RefObject<HTMLDivElement | null>; arm: () => void } {
    const listRef = useRef<HTMLDivElement | null>(null);
    const armed = useRef(false);
    useEffect(() => {
        if (!armed.current || openId === undefined) return;
        armed.current = false;
        if (listRef.current?.offsetParent === null) window.scrollTo({ top: 0 });
    }, [openId]);
    return {
        listRef,
        arm: () => {
            armed.current = true;
        },
    };
}
