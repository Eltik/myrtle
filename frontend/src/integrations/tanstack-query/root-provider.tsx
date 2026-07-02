import { QueryClient } from "@tanstack/react-query";

const isServer = typeof window === "undefined";
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
        // Many call sites set a 24h per-query `gcTime`, which supported `defaultOptions`
        // can't override. Clamp the *resolved* gcTime for every query in one place so
        // every current and future call site is covered without editing all 60+ of them.
        //
        // Load-bearing: this monkey-patch depends on TanStack Query internals (the shape
        // of `defaultQueryOptions`), verified on 5.100.1. Min-clamp (not force 0) so SSR
        // queries survive long enough to dehydrate; `queryClient.clear()` on render finish
        // (router.tsx) remains the primary OOM defense.
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
