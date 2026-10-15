// @vitest-environment node
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getContext } from "#/integrations/tanstack-query/root-provider";

// The SSR QueryClient (`retry: 1`, no window) running the operator loader's
// prefetch, against a backend that has no stats for the operator.
const fetchMock = vi.fn();
vi.stubGlobal("fetch", fetchMock);
vi.mock("@tanstack/react-start", () => ({
    createServerFn: () => ({
        inputValidator: () => ({ handler: (fn: (args: { data: unknown }) => unknown) => (opts: { data: unknown }) => fn({ data: opts.data }) }),
        handler: (fn: () => unknown) => () => fn(),
    }),
}));

const { operatorBuildStatsQueryOptions } = await import("./operators");

beforeEach(() => fetchMock.mockReset());

describe("operator build stats during SSR", () => {
    it("asks the backend once and caches the 404 as null", async () => {
        fetchMock.mockImplementation(async () => new Response("{}", { status: 404 }));
        const { queryClient } = getContext();
        const options = operatorBuildStatsQueryOptions("char_4195_radian");
        await queryClient.prefetchQuery(options);
        expect(fetchMock).toHaveBeenCalledTimes(1);
        // Only a success is dehydrated into the page; an error makes the browser ask again.
        expect(queryClient.getQueryState(options.queryKey)?.status).toBe("success");
        expect(queryClient.getQueryData(options.queryKey)).toBeNull();
    });
});
