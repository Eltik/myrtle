import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

// Generated from `backend/src/core/gamedata/types/range.rs`. To change a field,
// edit the Rust struct and run `bun run gen:types` - do not redeclare it here.
import type { Grid } from "#/types/generated/Grid";
import type { Range } from "#/types/generated/Range";

export type IRangeGrid = Grid;
export type IRange = Range;
export type IRangesMap = Record<string, IRange>;

export const getRangesFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/static/ranges");
    if (!res.ok) throw new Error(`Failed to load ranges: ${res.status}`);
    return (await res.json()) as IRangesMap;
});

export function rangesQueryOptions() {
    return queryOptions({
        queryKey: ["ranges"],
        queryFn: () => getRangesFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
