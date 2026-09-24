/**
 * The Community tab against a MOCKED query, because the binary on :3060 while
 * this shipped answers 404 on `/story/community` and the tab has therefore
 * never been seen with live data. The two states that matter are both here:
 * the aggregate the backend will serve, and the null it serves today.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import type React from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import type { StoryCommunity } from "#/types/generated/StoryCommunity";
import { messages as browseMessages } from "./Browse.messages";
import { messages as communityMessages } from "./CommunityTab.messages";
import type { LibEntry, LibGroup, LibIndex } from "./derive";

// A server function has neither an RPC boundary nor a backend under a unit
// test, and this tab's query is never reached anyway: the client is seeded.
vi.mock("@tanstack/react-start", () => ({
    createServerFn: () => ({
        inputValidator: () => ({
            handler: (fn: (args: { data: unknown }) => unknown) => (opts: { data: unknown }) => fn({ data: opts.data }),
        }),
    }),
}));
vi.mock("@tanstack/react-router", () => ({
    Link: ({ children, className }: { children?: React.ReactNode; className?: string }) => (
        <a href="/stories" className={className}>
            {children}
        </a>
    ),
}));

const { CommunityTab } = await import("./CommunityTab");

const t = (key: keyof typeof communityMessages): string => communityMessages[key].text;
const catalog = Object.fromEntries(Object.entries({ ...communityMessages, ...browseMessages }).map(([key, value]) => [`story.${key}`, value.text]));

function story(id: string, name: string, gated: boolean, code?: string): LibEntry {
    return { id, name, groupId: "", sort: 0, hasScript: true, code, requiredStages: gated ? ["main_01-01"] : [] } as LibEntry;
}

function group(id: string, name: string, category: string, stories: LibEntry[], displayType?: string): LibGroup {
    return { id, name, category, displayType, entryType: "NONE", actType: "NONE", startTime: -1, stories: stories.map((s) => ({ ...s, groupId: id })) } as unknown as LibGroup;
}

const index: LibIndex = {
    groups: [group("main_1", "Black Trail", "main", [story("m1a", "Prologue", false, "1-1"), story("m1b", "Trail", true, "1-2"), story("m1c", "Ash", true, "1-3")]), group("act4d0", "Twilight of Wolumonde", "vignette", [story("v1a", "Snow", false), story("v1b", "Thaw", false)], "MINISTORY")],
    records: [],
};

const community: StoryCommunity = {
    players: 2576,
    computedAt: Math.floor(Date.now() / 1000) - 3600,
    stories: [
        { id: "m1a", readers: 2283 },
        { id: "m1b", readers: 2273 },
        { id: "m1c", readers: 2280 },
        { id: "v1a", readers: 120 },
        { id: "v1b", readers: 96 },
    ],
    groups: [
        { id: "main_1", readers: 2283, finished: 2210, depth: [2283, 2273, 2280] },
        { id: "act4d0", readers: 120, finished: 0, depth: [120, 96] },
    ],
};

function mount(node: React.ReactElement, data: StoryCommunity | null) {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    client.setQueryData(["story", "community", "en"], data);
    return render(
        <QueryClientProvider client={client}>
            <I18nProvider locale="en" available={[]} messages={catalog}>
                {node}
            </I18nProvider>
        </QueryClientProvider>,
    );
}

afterEach(cleanup);

/**
 * The rows of one section. A two-group fixture puts the same chapter in the
 * most-read list AND the least-read one, which is not a defect of either list,
 * so every row assertion names the section it means.
 */
function section(heading: string): HTMLElement {
    const found = screen.getByText(heading).closest("section");
    if (!found) throw new Error(`no section headed ${heading}`);
    return found as HTMLElement;
}

describe("CommunityTab", () => {
    it("says the aggregate is not available yet when the backend answered 404, and shows no ranking", () => {
        mount(<CommunityTab index={index} onViewChapter={() => undefined} />, null);
        expect(screen.getByText(t("community.empty.title"))).toBeTruthy();
        expect(screen.queryByText(t("community.section.top"))).toBeNull();
    });

    it("heads the tab with the player count the shares are taken over", () => {
        mount(<CommunityTab index={index} onViewChapter={() => undefined} />, community);
        expect(screen.getByText(/2,576 synced players/)).toBeTruthy();
    });

    it("ranks the chapters, labels every percentage as a share of players, and prints the finisher count where a stage gates the chapter", () => {
        mount(<CommunityTab index={index} onViewChapter={() => undefined} />, community);
        const row = within(section(t("community.section.top"))).getByRole("button", { name: "Open Black Trail" });
        expect(row.textContent).toContain("2,283");
        expect(row.textContent).toContain("2,210");
        expect(row.textContent).toContain("89% of players");
        expect(row.textContent).not.toContain(t("community.notMeasurable"));
    });

    it("refuses to print a finisher count for a chapter no stage gates", () => {
        mount(<CommunityTab index={index} onViewChapter={() => undefined} />, community);
        const row = within(section(t("community.section.top"))).getByRole("button", { name: "Open Twilight of Wolumonde" });
        expect(row.textContent).toContain(t("community.notMeasurable"));
        expect(row.textContent).not.toContain("0 of players");
    });

    it("opens the chapter sheet with the group the row names", () => {
        const opened: string[] = [];
        mount(<CommunityTab index={index} onViewChapter={(id) => opened.push(id)} />, community);
        fireEvent.click(within(section(t("community.section.top"))).getByRole("button", { name: "Open Black Trail" }));
        expect(opened).toEqual(["main_1"]);
    });

    it("links a story row at the reader and names the chapter it belongs to", () => {
        mount(<CommunityTab index={index} onViewChapter={() => undefined} />, community);
        expect(screen.getAllByText("Black Trail").length).toBeGreaterThan(1);
        expect(screen.getAllByRole("link").length).toBeGreaterThan(0);
    });

    it("draws one labelled bar per story of the chapter the strip opens on, and says the drop in words", () => {
        mount(<CommunityTab index={index} onViewChapter={() => undefined} />, community);
        expect(screen.getByLabelText("1-1 Prologue: 2,283 readers, 89% of players")).toBeTruthy();
        expect(screen.getByLabelText("1-3 Ash: 2,280 readers, 89% of players")).toBeTruthy();
        expect(screen.getByText("2,283 start, 2,280 finish")).toBeTruthy();
    });

    it("leaves the least-read list out of the zero-reader chapters and says nothing was skipped when none was", () => {
        mount(<CommunityTab index={index} onViewChapter={() => undefined} />, community);
        expect(screen.getByText(t("community.section.bottom"))).toBeTruthy();
        expect(screen.queryByText(/are not listed/)).toBeNull();
    });
});
