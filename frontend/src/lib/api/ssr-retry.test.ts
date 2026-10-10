// @vitest-environment node
import type { QueryClient } from "@tanstack/react-query";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// The three queries the operator and tier-list loaders await during SSR, run through the
// server's QueryClient against a backend that answers 500 to everything.
const { backendFetch, serverFn } = vi.hoisted(() => {
    const builder: Record<string, unknown> = {};
    builder.inputValidator = () => builder;
    builder.middleware = () => builder;
    builder.handler = (fn: (args: { data: unknown }) => unknown) => (opts?: { data: unknown }) => fn({ data: opts?.data });
    return { backendFetch: vi.fn(), serverFn: builder };
});
vi.mock("#/lib/fetch", () => ({ backendFetch: (...args: unknown[]) => backendFetch(...args) }));
vi.mock("#/env", () => ({ env: { NODE_ENV: "test" } }));
vi.mock("@tanstack/react-start", () => ({ createServerFn: () => serverFn }));
vi.mock("@tanstack/react-start/server", () => ({ getCookie: () => "token", getRequestIP: () => null, setCookie: () => undefined }));

const { getContext, noRetryOnServer } = await import("#/integrations/tanstack-query/root-provider");
const { operatorBuildStatsQueryOptions } = await import("./operators");
const { myTierListFavoriteQueryOptions, tierListVersionsQueryOptions } = await import("./tier-lists");

describe("queries an SSR loader awaits", () => {
    beforeEach(() => {
        backendFetch.mockReset();
        backendFetch.mockImplementation(async () => new Response("{}", { status: 500 }));
    });
    afterEach(() => vi.unstubAllGlobals());

    it.each([
        ["operator build stats", (qc: QueryClient) => qc.prefetchQuery(operatorBuildStatsQueryOptions("char_002_amiya"))],
        ["tier-list versions", (qc: QueryClient) => qc.prefetchQuery(tierListVersionsQueryOptions("some-list"))],
        ["tier-list favorite", (qc: QueryClient) => qc.prefetchQuery(myTierListFavoriteQueryOptions("some-list", true))],
    ])("%s asks the backend once on the server when it answers 500", async (_name, prefetch) => {
        const { queryClient } = getContext();
        await prefetch(queryClient);
        expect(backendFetch).toHaveBeenCalledTimes(1);
    });

    it("leaves the browser on the client default", () => {
        expect(noRetryOnServer()).toEqual({ retry: 0 });
        vi.stubGlobal("window", {});
        expect(noRetryOnServer()).toEqual({});
        expect("retry" in operatorBuildStatsQueryOptions("char_002_amiya")).toBe(false);
    });
});
