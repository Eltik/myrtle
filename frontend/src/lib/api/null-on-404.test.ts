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

const { archiveOperatorQueryOptions } = await import("./archives");
const { announcementQueryOptions } = await import("./announcements");
const { operatorQueryOptions } = await import("./operators");

const reply = (status: number, body: unknown = {}) => new Response(JSON.stringify(body), { status });

// Each single-record read: the record on 200, `null` on 404, and a thrown
// `Failed to load <what>: <status>` on anything else.
const cases = [
    { name: "archive operator", run: () => new QueryClient().fetchQuery(archiveOperatorQueryOptions("char_1", "jp")), path: "/jp/archives/operator/char_1", what: "personnel file" },
    { name: "announcement", run: () => new QueryClient().fetchQuery(announcementQueryOptions("16 08", "kr")), path: "/announcements/16%2008?server=kr", what: "announcement" },
    { name: "operator", run: () => new QueryClient().fetchQuery(operatorQueryOptions("char_1", "en")), path: "/operators/char_1", what: "operator" },
];

beforeEach(() => fetchMock.mockReset());

describe.each(cases)("$name read", ({ run, path, what }) => {
    it("returns the record from its path", async () => {
        fetchMock.mockResolvedValueOnce(reply(200, { id: "x", Nested_: { A: 1 } }));
        const out = await run();
        expect(fetchMock.mock.calls[0][0]).toMatch(new RegExp(`/api${path.replace(/[?]/g, "\\?")}$`));
        expect(out).toEqual(what === "operator" ? { id: "x", nested: { a: 1 } } : { id: "x", Nested_: { A: 1 } });
    });

    it("returns null on 404", async () => {
        fetchMock.mockResolvedValueOnce(reply(404));
        expect(await run()).toBeNull();
    });

    it("throws with the status otherwise", async () => {
        fetchMock.mockResolvedValue(reply(503));
        await expect(run()).rejects.toThrow(`Failed to load ${what}: 503`);
    });
});
