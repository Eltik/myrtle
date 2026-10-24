import { useRouter } from "@tanstack/react-router";
import { Link2 } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { toastManager } from "#/components/ui/toast";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./ShareLink.messages";
import type { IOperatorListEntry } from "./types";
import type { ExternalHydration } from "./usePersistedReducer";

type ShareT = TypedT<typeof messages>;

/** How long the page waits after the last change before rewriting `?s=`. */
const URL_WRITE_DELAY_MS = 400;

/**
 * Resolve the `?s=` param present at mount into page state for `usePersistedReducer`.
 * Waits for the operator list (cards carry operator ids only). A link that does not
 * decode falls back to the stored setup with a warning; cards whose operator is gone
 * are dropped and counted. When the operator list fails to load the stored setup is used
 * (the page already shows that failure). Later changes to `param` are the page's own
 * writes and are ignored.
 */
export function useShareHydration<S, TShare extends { invalid: number }>(
    param: string | undefined,
    decode: (text: string) => TShare | null,
    resolve: (share: TShare, operators: readonly IOperatorListEntry[]) => { state: S; missing: number },
    operators: readonly IOperatorListEntry[] | undefined,
    operatorsFailed = false,
): ExternalHydration<S> {
    const t: ShareT = useT("tools");
    const [initialParam] = React.useState(param);
    const decoded = React.useMemo(() => (initialParam ? decode(initialParam) : null), [initialParam, decode]);
    const [result, setResult] = React.useState<ExternalHydration<S>>(initialParam ? { status: "pending" } : { status: "none" });

    React.useEffect(() => {
        if (result.status !== "pending") return;
        if (!decoded) {
            toastManager.add({ id: `calc-share-invalid-${Date.now()}`, title: t("calc.share.invalid"), type: "warning" });
            setResult({ status: "ready", state: null });
            return;
        }
        if (!operators) {
            if (operatorsFailed) setResult({ status: "ready", state: null });
            return;
        }
        const { state, missing } = resolve(decoded, operators);
        const skipped = missing + decoded.invalid;
        if (skipped > 0) toastManager.add({ id: `calc-share-skipped-${Date.now()}`, title: t("calc.share.skipped", { count: skipped }), type: "warning" });
        setResult({ status: "ready", state });
    }, [result.status, decoded, operators, operatorsFailed, resolve, t]);

    return result;
}

/**
 * Mirror the page state into `?s=` once hydrated: debounced, and through a history
 * replace (the caller's `write`), so editing a card does not stack history entries.
 * A state that encodes the same as the page default clears the param.
 */
export function useShareUrlSync<S>(state: S, hydrationToken: number, encode: (state: S) => string, defaultEncoded: string, initialParam: string | undefined, write: ((param: string | undefined) => void) | undefined): void {
    const lastRef = React.useRef(initialParam);
    React.useEffect(() => {
        if (hydrationToken === 0 || !write) return;
        const id = window.setTimeout(() => {
            const encoded = encode(state);
            const next = encoded === defaultEncoded ? undefined : encoded;
            if (next === lastRef.current) return;
            lastRef.current = next;
            write(next);
        }, URL_WRITE_DELAY_MS);
        return () => window.clearTimeout(id);
    }, [state, hydrationToken, encode, defaultEncoded, write]);
}

/** Copies an absolute link to `to?s=<encoded current state>`; encodes on click, so it never waits on the debounce. */
export function CopyLinkButton({ to, encode }: { to: "/tools/dps" | "/tools/hps"; encode: () => string }): React.ReactElement {
    const t: ShareT = useT("tools");
    const router = useRouter();
    const onCopy = React.useCallback(async () => {
        // The router's own encoder, so the param round-trips exactly as the route reads it.
        const href = router.buildLocation({ to, search: { s: encode() } as never }).href;
        try {
            await navigator.clipboard.writeText(`${window.location.origin}${href}`);
            toastManager.add({ id: `calc-share-copied-${Date.now()}`, title: t("calc.share.copied"), type: "success" });
        } catch {
            toastManager.add({ id: `calc-share-copy-err-${Date.now()}`, title: t("calc.share.copyFailed"), type: "error" });
        }
    }, [router, to, encode, t]);
    return (
        <Tooltip>
            <TooltipTrigger
                render={(p) => (
                    <Button {...p} aria-label={t("calc.share.copyLink")} variant="outline" size="icon-sm" onClick={onCopy} className="mb-1">
                        <Link2 />
                    </Button>
                )}
            />
            <TooltipPopup>{t("calc.share.copyLink")}</TooltipPopup>
        </Tooltip>
    );
}
