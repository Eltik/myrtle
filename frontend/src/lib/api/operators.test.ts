/**
 * The operator queries must never resolve to `undefined`.
 *
 * TanStack Query fails any query whose queryFn resolves to `undefined`, and on
 * production that logged `["operators","detail","null"] data is undefined` and
 * `["operators","detail","list"] data is undefined` for every request to an id
 * the backend answers 404 for. A miss is `null`; a backend failure throws.
 */
import { QueryClient } from "@tanstack/react-query";
import { afterEach, describe, expect, it, vi } from "vitest";

const { fetchMock } = vi.hoisted(() => ({ fetchMock: vi.fn() }));

// The RPC boundary and the backend are replaced: the handler runs in-process,
// so the status it maps to a value is observable here.
vi.mock("@tanstack/react-start", () => ({
    createServerFn: () => ({
        inputValidator: () => ({
            handler: (fn: (args: { data: unknown }) => unknown) => (opts: { data: unknown }) => fn({ data: opts.data }),
        }),
    }),
}));
vi.mock("#/lib/fetch", () => ({ backendFetch: fetchMock }));

const { operatorBuildStatsQueryOptions, operatorQueryOptions } = await import("./operators");

function respond(status: number, body: unknown = {}) {
    fetchMock.mockResolvedValue(new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } }));
}

function client() {
    return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}

afterEach(() => {
    fetchMock.mockReset();
});

describe("operatorQueryOptions", () => {
    it("resolves an unknown id to null, which TanStack Query accepts", async () => {
        respond(404);
        const qc = client();
        await expect(qc.fetchQuery(operatorQueryOptions("null"))).resolves.toBeNull();
        expect(qc.getQueryState(operatorQueryOptions("null").queryKey)?.status).toBe("success");
    });

    it("throws on a backend failure rather than resolving to nothing", async () => {
        respond(502);
        await expect(client().fetchQuery(operatorQueryOptions("char_002_amiya"))).rejects.toThrow("Failed to load operator: 502");
    });

    it("camelizes a found operator", async () => {
        respond(200, { id: "char_002_amiya", Name_: "Amiya" });
        await expect(client().fetchQuery(operatorQueryOptions("char_002_amiya"))).resolves.toEqual({ id: "char_002_amiya", name: "Amiya" });
    });
});

describe("operatorBuildStatsQueryOptions", () => {
    it("resolves a 404 to null", async () => {
        respond(404);
        await expect(client().fetchQuery(operatorBuildStatsQueryOptions("char_002_amiya"))).resolves.toBeNull();
    });
});
