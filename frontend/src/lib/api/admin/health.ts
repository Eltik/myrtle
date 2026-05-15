import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

export interface IHealthResponse {
    status: "ok" | "degraded";
    cache: {
        backend: "redis" | "memory";
        status: "connected" | "disconnected";
        responseTimeMs: number;
    };
    database: {
        status: "connected" | "disconnected";
        responseTimeMs: number;
    };
    timestamp: string;
    responseTimeMs: number;
}

export function formatResponseTimeMs(ms: number | null | undefined): string {
    if (ms == null || !Number.isFinite(ms)) return "-";
    if (ms >= 10) return ms.toFixed(0);
    if (ms >= 1) return ms.toFixed(1);
    return ms.toFixed(2);
}

export const getHealthFn = createServerFn({ method: "GET" }).handler(async (): Promise<IHealthResponse> => {
    const res = await backendFetch("/health");
    if (!res.ok) throw new Error(`Failed to load health: ${res.status}`);
    return (await res.json()) as IHealthResponse;
});

export function healthQueryOptions() {
    return queryOptions({
        queryKey: ["admin", "health"],
        queryFn: () => getHealthFn(),
        staleTime: 15 * 1000,
        gcTime: 60 * 1000,
        refetchInterval: 30 * 1000,
    });
}
