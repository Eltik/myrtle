import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
// Generated from `backend/src/core/maintenance/mod.rs`.
// To change a field, edit the Rust struct and run `bun run gen:types`.
import type { MaintenanceStatus } from "#/types/generated/MaintenanceStatus";
import { backendFetch } from "../fetch";

export const getMaintenanceFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/maintenance");
    if (!res.ok) throw new Error(`Failed to load maintenance: ${res.status}`);
    return (await res.json()) as MaintenanceStatus;
});

/**
 * The backend polls Yostar every 5 minutes (every minute while a window runs), so asking more
 * often than once a minute only re-reads its memory. The countdown itself ticks on the client.
 */
export function maintenanceQueryOptions() {
    return queryOptions({
        queryKey: ["maintenance"],
        queryFn: () => getMaintenanceFn(),
        staleTime: 30 * 1000,
        refetchInterval: 60 * 1000,
        // A failed read just means no countdown; never retry into the error toast path.
        retry: false,
    });
}
