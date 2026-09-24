/**
 * ONE SCROLL CONTAINER, and the test exists because the defect it pins was
 * invisible to every other check: two nested scrollers type-check, lint clean
 * and render, and the only symptom is a second scrollbar on the right of a
 * 25-row chapter at 1440x900.
 *
 * jsdom lays nothing out, so this asserts on the STRUCTURE rather than on
 * `scrollHeight`: exactly one element under the popup declares an overflow of
 * its own, it is the content box that holds all three bands, and the entries
 * wrapper beneath the meta block declares neither an overflow nor a height cap.
 * The measured counterpart is in the browser, where the three viewports each
 * come back with exactly one overflowing element.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, render } from "@testing-library/react";
import type React from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import type { StoryProgress } from "#/lib/story/progress";
import { messages as archiveMessages } from "./Archive.messages";
import { messages as browseMessages } from "./Browse.messages";
import type { LibEntry, LibGroup } from "./derive";
import { messages as artMessages } from "./IllustrationsTab.messages";
import { messages as marksMessages } from "./ReadToggle.messages";
import { messages as storiesMessages } from "./StoriesTab.messages";

// A server function has neither an RPC boundary nor a backend under a unit
// test; the archive query is the only one this sheet makes and it is answered
// with the 404 the live route answers today.
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
vi.mock("#/lib/fetch", () => ({ backendFetch: vi.fn().mockResolvedValue(new Response("", { status: 404 })) }));

const { ChapterModal } = await import("./ChapterModal");

const catalog = Object.fromEntries(Object.entries({ ...browseMessages, ...archiveMessages, ...artMessages, ...marksMessages, ...storiesMessages }).map(([key, value]) => [`story.${key}`, value.text]));

function entry(id: string, name: string, code: string): LibEntry {
    return { id, name, groupId: "main_8", sort: 0, hasScript: true, code, requiredStages: [], wordCount: 1200 } as unknown as LibEntry;
}

const group = {
    id: "main_8",
    name: "Roaring Flare",
    category: "main",
    entryType: "NONE",
    actType: "NONE",
    startTime: -1,
    stories: [entry("m8a", "Yesterday, the Chaff Cracked", "8-1"), entry("m8b", "Today, Sanguine Overlows", "8-2"), entry("m8c", "Oatstalk, Easily Alight", "8-3")],
} as unknown as LibGroup;

const progress: StoryProgress = { v: 2, read: {} } as StoryProgress;

function mount(): void {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
        <QueryClientProvider client={client}>
            <I18nProvider locale="en" available={[]} messages={catalog}>
                <ChapterModal group={group} progress={progress} gameRead={new Set()} onClose={() => {}} />
            </I18nProvider>
        </QueryClientProvider>,
    );
}

/** The popup is portalled onto the body, so the assertions read from there rather than from the render container. */
function popup(): HTMLElement {
    const el = document.querySelector<HTMLElement>("[data-slot=dialog-popup]");
    if (!el) throw new Error("the dialog popup did not render");
    return el;
}

const SCROLLS = /overflow-y-(auto|scroll)|\boverflow-auto\b/;
const CAPS = /max-h-\[/;

afterEach(() => {
    cleanup();
    vi.clearAllMocks();
});

describe("the chapter sheet's scrolling", () => {
    it("declares exactly one scroll container, and it holds every band", () => {
        mount();
        const scrollers = [...popup().querySelectorAll<HTMLElement>("*")].filter((el) => SCROLLS.test(el.className.toString()));

        expect(scrollers).toHaveLength(1);
        expect(scrollers[0].className).toContain("overscroll-contain");
        // The hero, the meta block and the list are all inside it, which is
        // what makes the hero scroll away rather than sit above a pane.
        expect(scrollers[0].querySelector("ol")).not.toBeNull();
        expect(scrollers[0].querySelector(".sticky")).not.toBeNull();
    });

    it("gives the entries no scroller and no height cap of their own", () => {
        mount();
        const list = popup().querySelector("ol");
        if (!list) throw new Error("the operation list did not render");

        const scroller = [...popup().querySelectorAll<HTMLElement>("*")].find((el) => SCROLLS.test(el.className.toString()));
        for (let el = list as HTMLElement | null; el && el !== scroller; el = el.parentElement) {
            expect(SCROLLS.test(el.className.toString())).toBe(false);
            expect(CAPS.test(el.className.toString())).toBe(false);
        }
    });

    it("sticks the meta block to the top of that container", () => {
        mount();
        const sticky = popup().querySelector<HTMLElement>(".sticky");
        if (!sticky) throw new Error("the meta block did not render");

        expect(sticky.className).toContain("top-0");
        // An opaque surface is what stops the rows reading through it.
        expect(sticky.className).toContain("bg-card");
    });
});
