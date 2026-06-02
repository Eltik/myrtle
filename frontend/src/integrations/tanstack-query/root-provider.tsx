import { QueryClient } from "@tanstack/react-query";

// A fresh QueryClient is created per request on the server (see `getRouter`).
// Every query schedules a `setTimeout(gcTime)` that keeps the query's data — and,
// transitively, the whole client — reachable until it fires. With our long gcTimes
// (up to 24h on static game data) each SSR request's fetched payloads stay resident
// for hours, so traffic accumulates GBs until the OOM killer reaps the `bun` server.
//
// On the server the cache only needs to outlive dehydration (synchronous, within the
// request), so clamp gcTime hard. The browser keeps its long-lived caches untouched.
const isServer = import.meta.env.SSR;
const SERVER_MAX_GC_TIME = 1_000;

export function getContext() {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                staleTime: 5 * 60 * 1000,
                gcTime: isServer ? SERVER_MAX_GC_TIME : 60 * 60 * 1000,
                refetchOnWindowFocus: false,
                refetchOnReconnect: false,
                retry: 1,
            },
        },
    });

    if (isServer) {
        // Per-query `gcTime` overrides (e.g. 24h on operators/stages/enemies) would
        // defeat the default above. Clamp the *resolved* gcTime for every query in one
        // place — this also covers any future call site without touching all of them.
        const resolve = queryClient.defaultQueryOptions.bind(queryClient);
        queryClient.defaultQueryOptions = ((options?: unknown) => {
            const resolved = resolve(options as never);
            resolved.gcTime = Math.min(resolved.gcTime ?? SERVER_MAX_GC_TIME, SERVER_MAX_GC_TIME);
            return resolved;
        }) as typeof queryClient.defaultQueryOptions;
    }

    return {
        queryClient,
    };
}
export default function TanstackQueryProvider() {}
