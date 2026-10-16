/**
 * Cloudflare Turnstile, rendered only when `VITE_TURNSTILE_SITE_KEY` is set
 * (the ask box does not mount this otherwise, so no script loads and no token
 * is sent, which is what a backend without `TURNSTILE_SECRET` expects).
 *
 * A token is single-use: the ask box bumps `resetKey` after each question and
 * the widget issues a fresh one.
 */
import type React from "react";
import { useEffect, useRef } from "react";

interface ITurnstileApi {
    render: (el: HTMLElement, opts: Record<string, unknown>) => string;
    reset: (id: string) => void;
    remove: (id: string) => void;
}

const SCRIPT = "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit";
let loading: Promise<ITurnstileApi> | null = null;

function loadTurnstile(): Promise<ITurnstileApi> {
    const w = window as unknown as { turnstile?: ITurnstileApi };
    if (w.turnstile) return Promise.resolve(w.turnstile);
    loading ??= new Promise<ITurnstileApi>((resolve, reject) => {
        const s = document.createElement("script");
        s.src = SCRIPT;
        s.async = true;
        s.onload = () => (w.turnstile ? resolve(w.turnstile) : reject(new Error("turnstile did not load")));
        s.onerror = () => {
            loading = null;
            reject(new Error("turnstile did not load"));
        };
        document.head.appendChild(s);
    });
    return loading;
}

export function Turnstile({ siteKey, onToken, resetKey }: { siteKey: string; onToken: (token: string | null) => void; resetKey: number }): React.ReactElement {
    const el = useRef<HTMLDivElement>(null);
    const widget = useRef<{ api: ITurnstileApi; id: string } | null>(null);
    const onTokenRef = useRef(onToken);
    onTokenRef.current = onToken;

    useEffect(() => {
        let live = true;
        void loadTurnstile()
            .then((api) => {
                if (!live || !el.current) return;
                const id = api.render(el.current, {
                    sitekey: siteKey,
                    callback: (token: string) => onTokenRef.current(token),
                    "expired-callback": () => onTokenRef.current(null),
                    "error-callback": () => onTokenRef.current(null),
                });
                widget.current = { api, id };
            })
            .catch(() => onTokenRef.current(null));
        return () => {
            live = false;
            if (widget.current) widget.current.api.remove(widget.current.id);
            widget.current = null;
        };
    }, [siteKey]);

    useEffect(() => {
        if (resetKey === 0 || !widget.current) return;
        onTokenRef.current(null);
        widget.current.api.reset(widget.current.id);
    }, [resetKey]);

    return <div ref={el} data-trevor-turnstile className="min-h-[65px]" />;
}
