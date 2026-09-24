/**
 * The archive as the sheet consumes it: the order the sections are read in,
 * the query that fetches them, and the rule that the second segment does not
 * exist for the 81 groups with no archive.
 *
 * The query is exercised against a mocked `backendFetch` rather than a running
 * backend, because the route it calls answers 404 on the binary that is
 * running while this is written: the shape it must survive is exactly that.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import type { StoryArchive } from "#/types/generated/StoryArchive";
import type { StoryArchiveSection } from "#/types/generated/StoryArchiveSection";
import type { StoryIndex } from "#/types/generated/StoryIndex";
import { messages as archiveMessages } from "./Archive.messages";
import { ArchivePanel, ChapterViewSwitch } from "./ArchivePanel";
import { ARCHIVE_ORDER, archiveSections, archiveTotal, defaultLanguage, hasArchive, orderedTracks } from "./archive";
import { messages as artMessages } from "./IllustrationsTab.messages";
import { messages as marksMessages } from "./ReadToggle.messages";

const { fetchMock } = vi.hoisted(() => ({ fetchMock: vi.fn() }));

// The two things a server function needs that a unit test has neither of: the
// RPC boundary and a backend. The handler is run in-process instead, so the
// path it builds and the 404 it swallows are both observable here.
vi.mock("@tanstack/react-start", () => ({
    createServerFn: () => ({
        inputValidator: () => ({
            handler: (fn: (args: { data: unknown }) => unknown) => (opts: { data: unknown }) => fn({ data: opts.data }),
        }),
    }),
}));
vi.mock("#/lib/fetch", () => ({ backendFetch: fetchMock }));

const { storyArchiveQueryOptions } = await import("#/lib/api/story");

const t = (key: keyof typeof archiveMessages): string => archiveMessages[key].text;

/** The catalog the provider is given, so an assertion reads the message module rather than the extracted bundle. */
const catalog = Object.fromEntries(Object.entries({ ...archiveMessages, ...artMessages, ...marksMessages }).map(([key, value]) => [`story.${key}`, value.text]));

const logs: StoryArchiveSection = {
    kind: "logs",
    count: 2,
    chapters: [
        {
            id: "c1",
            name: "Day One",
            displayId: "01",
            chapterIcon: "NORMAL",
            logs: [
                { id: "l1", text: "A <@lv.item>marked</> line." },
                { id: "l2", text: "Another." },
            ],
        },
    ],
};
const gallery: StoryArchiveSection = { kind: "gallery", count: 1, pictures: [{ id: "p1", title: "Harbour", pictureType: "IMAGE", url: "/textures/avg/imgs/p1.png" }] };
const music: StoryArchiveSection = { kind: "music", count: 1, tracks: [{ id: "m1", name: "Tidal", loopUrl: "/audio/m1_loop.ogg" }] };
const recordings: StoryArchiveSection = {
    kind: "recordings",
    count: 1,
    nodes: [
        {
            id: "n1",
            title: "Tape one",
            clips: [
                {
                    charId: "char_002_amiya",
                    voiceId: "v1",
                    index: 0,
                    tracks: [
                        { language: "EN", url: "/audio/en.ogg" },
                        { language: "JP", url: "/audio/jp.ogg" },
                    ],
                },
            ],
        },
    ],
    hidden: [],
};

function archive(sections: StoryArchiveSection[]): StoryArchive {
    return { groupId: "act17side", sections };
}

function response(status: number, body?: unknown): Response {
    return { ok: status >= 200 && status < 300, status, json: async () => body } as Response;
}

afterEach(() => {
    cleanup();
    fetchMock.mockReset();
});

describe("archiveSections", () => {
    it("puts the sections in reading order whatever order the wire sent them", () => {
        const got = archiveSections(archive([music, gallery, logs]));
        expect(got.map((s) => s.kind)).toEqual(["logs", "gallery", "music"]);
    });

    it("orders every kind the same way the register does", () => {
        const all = ARCHIVE_ORDER.map((kind) => ({ kind, count: 0 }) as unknown as StoryArchiveSection);
        const shuffled = [...all].reverse();
        expect(archiveSections(archive(shuffled)).map((s) => s.kind)).toEqual([...ARCHIVE_ORDER]);
    });

    it("drops a kind this renderer has no section for", () => {
        const stranger = { kind: "puzzles", count: 4 } as unknown as StoryArchiveSection;
        expect(archiveSections(archive([stranger, logs])).map((s) => s.kind)).toEqual(["logs"]);
    });

    it("answers with nothing for a 404, for a group that kept no archive, and while the query is in flight", () => {
        expect(archiveSections(null)).toEqual([]);
        expect(archiveSections(undefined)).toEqual([]);
        expect(archiveSections(archive([]))).toEqual([]);
        expect(hasArchive(null)).toBe(false);
        expect(hasArchive(archive([]))).toBe(false);
        expect(hasArchive(archive([logs]))).toBe(true);
    });

    it("sums the leaf counts the backend carries", () => {
        expect(archiveTotal(archiveSections(archive([logs, gallery, music])))).toBe(4);
        expect(archiveTotal([])).toBe(0);
    });
});

describe("clip languages", () => {
    const clip = {
        tracks: [
            { language: "EN" as const, url: "e" },
            { language: "CN_TOPOLECT" as const, url: "c" },
            { language: "JP" as const, url: "j" },
        ],
    };

    it("orders the tracks the way the voices tab does and opens on the first", () => {
        expect(orderedTracks(clip).map((track) => track.language)).toEqual(["JP", "EN", "CN_TOPOLECT"]);
        expect(defaultLanguage(clip)).toBe("JP");
    });

    it("keeps a language the order does not name, at the end", () => {
        const odd = {
            tracks: [
                { language: "UNKNOWN" as const, url: "u" },
                { language: "EN" as const, url: "e" },
            ],
        };
        expect(orderedTracks(odd).map((track) => track.language)).toEqual(["EN", "UNKNOWN"]);
    });

    it("has no language to open on when a clip carries no track", () => {
        expect(defaultLanguage({ tracks: [] })).toBeNull();
    });
});

describe("storyArchiveQueryOptions", () => {
    it("keys by server and group, beside the illustrations query", () => {
        expect(storyArchiveQueryOptions("act13side").queryKey).toEqual(["story", "archive", "en", "act13side"]);
        expect(storyArchiveQueryOptions("act13side", "jp").queryKey).toEqual(["story", "archive", "jp", "act13side"]);
        expect(storyArchiveQueryOptions("act13side", "nonsense").queryKey).toEqual(["story", "archive", "en", "act13side"]);
    });

    it("fetches the default server unprefixed and another server under its own prefix", async () => {
        fetchMock.mockResolvedValue(response(200, archive([logs])));
        await storyArchiveQueryOptions("act13side").queryFn?.({} as never);
        expect(fetchMock).toHaveBeenCalledWith("/story/group/act13side/archive");
        await storyArchiveQueryOptions("act13side", "jp").queryFn?.({} as never);
        expect(fetchMock).toHaveBeenCalledWith("/jp/story/group/act13side/archive");
    });

    it("answers null on the 404 a backend without the route gives", async () => {
        fetchMock.mockResolvedValue(response(404));
        await expect(storyArchiveQueryOptions("act13side").queryFn?.({} as never)).resolves.toBeNull();
    });

    it("hands back the archive on a 200, empty sections included", async () => {
        fetchMock.mockResolvedValue(response(200, archive([])));
        await expect(storyArchiveQueryOptions("act29side").queryFn?.({} as never)).resolves.toEqual({ groupId: "act17side", sections: [] });
    });

    it("throws on any other status, which is a backend fault rather than a missing archive", async () => {
        fetchMock.mockResolvedValue(response(500));
        await expect(storyArchiveQueryOptions("act13side").queryFn?.({} as never)).rejects.toThrow("Failed to load archive: 500");
    });
});

function mount(node: React.ReactElement) {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    client.setQueryData(["story", "index", "en"], { groups: [], records: [{ charId: "char_002_amiya", name: "Amiya" }], storylines: [], totals: {} } as unknown as StoryIndex);
    return render(
        <QueryClientProvider client={client}>
            <I18nProvider locale="en" available={[]} messages={catalog}>
                {node}
            </I18nProvider>
        </QueryClientProvider>,
    );
}

describe("ChapterViewSwitch", () => {
    it("is not rendered at all while the archive has no sections", () => {
        const { container } = mount(<ChapterViewSwitch view="entries" onView={() => undefined} sections={[]} />);
        expect(container.textContent).toBe("");
        expect(screen.queryByText(t("archive.view.archive"))).toBeNull();
    });

    it("offers both views once the archive has one", () => {
        mount(<ChapterViewSwitch view="entries" onView={() => undefined} sections={[logs]} />);
        expect(screen.getByText(t("archive.view.entries")).getAttribute("aria-pressed")).toBe("true");
        expect(screen.getByText(t("archive.view.archive")).getAttribute("aria-pressed")).toBe("false");
    });
});

describe("ArchivePanel", () => {
    it("heads every section with its own leaf count, in reading order", () => {
        mount(<ArchivePanel sections={archiveSections(archive([music, gallery, logs]))} audio={{ playing: null, toggle: () => undefined, stop: () => undefined }} />);
        const headings = screen.getAllByRole("button").map((button) => button.textContent ?? "");
        expect(headings[0]).toContain(t("archive.section.logs"));
        expect(headings[0]).toContain("2");
        expect(headings[1]).toContain(t("archive.section.gallery"));
        expect(headings[2]).toContain(t("archive.section.music"));
        expect(screen.getByText(t("archive.total").replace("{count}", "4"))).toBeTruthy();
    });

    it("renders a log's game rich text as text, never as its tags", () => {
        mount(<ArchivePanel sections={[logs]} audio={{ playing: null, toggle: () => undefined, stop: () => undefined }} />);
        expect(screen.getByText(/A marked line\./)).toBeTruthy();
        expect(document.body.textContent).not.toContain("<@lv.item>");
    });

    it("names the operator a clip belongs to and offers its languages", () => {
        mount(<ArchivePanel sections={[recordings]} audio={{ playing: null, toggle: () => undefined, stop: () => undefined }} />);
        expect(screen.getByText("Amiya")).toBeTruthy();
        const picker = screen.getByLabelText(t("archive.clip.language")) as HTMLSelectElement;
        expect(picker.value).toBe("JP");
        expect([...picker.options].map((option) => option.value)).toEqual(["JP", "EN"]);
    });
});
