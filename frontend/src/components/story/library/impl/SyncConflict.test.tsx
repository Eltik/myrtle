/**
 * The conflict panel as the reader meets it: three ways out, each quoting what
 * it costs, and the standing policy under them.
 *
 * The panel was never seen on a real page while it was written, because it
 * only exists for a signed-in account whose document disagrees with the
 * browser's and the harness here is signed out. This test is what stands in
 * for that: the summary is fixed, so every number on screen is one this file
 * chose.
 */
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import type { SyncConflictSummary } from "#/lib/story/sync";

// The account side of the sync crosses an RPC boundary that a unit test has
// no server for. The panel never calls it, but importing the engine loads it.
vi.mock("@tanstack/react-start", () => ({
    createServerFn: () => ({
        inputValidator: () => ({ handler: (fn: (args: { data: unknown }) => unknown) => (opts: { data: unknown }) => fn({ data: opts.data }) }),
        handler: (fn: () => unknown) => () => fn(),
    }),
}));
vi.mock("@tanstack/react-start/server", () => ({ getCookie: () => null }));

const { SyncConflict } = await import("./SyncConflict");
const { setStorySyncPolicy } = await import("#/lib/story/sync");
const { messages } = await import("./SyncConflict.messages");

const catalog = Object.fromEntries(Object.entries(messages).map(([key, value]) => [`story.${key}`, value.text]));

const summary: SyncConflictSummary = {
    local: { marks: 12, positions: 3, last: "main_0_level_main_00-01_beg", newest: 1_700_000_900_000 },
    remote: { marks: 5, positions: 1, last: "ghost_story", newest: 1_700_000_100_000 },
    merged: { marks: 15, positions: 4 },
    dropsLocal: { marks: 10, positions: 3 },
    dropsRemote: { marks: 3, positions: 1 },
    newer: "local",
};

const index = {
    groups: [{ id: "main_0", stories: [{ id: "main_0_level_main_00-01_beg", name: "Evacuation" }] }],
    records: [],
} as unknown as Parameters<typeof SyncConflict>[0]["index"];

afterEach(cleanup);

describe("SyncConflict", () => {
    it("offers the three outcomes with their costs, names the newer side, and carries the policy select", () => {
        // A reader only ever sees this panel with the policy on `ask`: the
        // shipped default is `merge`, which never stops a pull.
        setStorySyncPolicy("ask");
        render(
            <I18nProvider locale="en" available={[{ code: "en", nativeName: "English" }]} messages={catalog}>
                <SyncConflict summary={summary} index={index} onResolve={() => {}} />
            </I18nProvider>,
        );

        expect(screen.getByRole("button", { name: "Merge both" })).toBeTruthy();
        expect(screen.getByRole("button", { name: "Use account" })).toBeTruthy();
        expect(screen.getByRole("button", { name: "Keep browser" })).toBeTruthy();

        expect(screen.getByText("Keeps everything from both: 15 finished stories and 4 saved positions.")).toBeTruthy();
        expect(screen.getByText("Replaces this browser: 10 finished stories and 3 saved positions are dropped.")).toBeTruthy();
        expect(screen.getByText("Replaces your account: 3 finished stories and 1 saved positions are dropped.")).toBeTruthy();

        // The id resolves through the index on one side and falls back to
        // itself on the other, which is the case a shelf this build does not
        // list produces.
        expect(screen.getByText("Evacuation")).toBeTruthy();
        expect(screen.getByText("ghost_story")).toBeTruthy();
        expect(screen.getAllByText("newer")).toHaveLength(1);

        const policy = screen.getByRole("combobox", { name: "When they differ next time" });
        expect(policy.textContent).toContain("Ask me");
    });
});
