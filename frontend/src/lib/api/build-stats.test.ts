import { QueryClient } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";

// The real `backendFetch` runs; only the network under it is stubbed.
const fetchMock = vi.fn();
vi.stubGlobal("fetch", fetchMock);
vi.mock("@tanstack/react-start", () => ({
    createServerFn: () => ({
        inputValidator: () => ({ handler: (fn: (args: { data: unknown }) => unknown) => (opts: { data: unknown }) => fn({ data: opts.data }) }),
        handler: (fn: () => unknown) => () => fn(),
    }),
}));

const { operatorBuildStatsQueryOptions } = await import("./operators");

const reply = (status: number, body: unknown = {}) => new Response(JSON.stringify(body), { status });

beforeEach(() => fetchMock.mockReset());

describe("operator build stats", () => {
    // An operator nobody has built yet 404s. React Query rejects a query whose
    // function resolves `undefined`, which turned "no stats" into an error the
    // client retried; `null` is a value it caches.
    it("resolves null, not an error, on 404", async () => {
        fetchMock.mockResolvedValueOnce(reply(404));
        const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
        const options = operatorBuildStatsQueryOptions("char_4195_radian");
        await expect(client.fetchQuery(options)).resolves.toBeNull();
        expect(client.getQueryState(options.queryKey)?.status).toBe("success");
    });

    it("throws with the status otherwise", async () => {
        fetchMock.mockResolvedValue(reply(503));
        const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
        await expect(client.fetchQuery(operatorBuildStatsQueryOptions("char_002_amiya"))).rejects.toThrow("Failed to load operator build stats: 503");
    });
});
