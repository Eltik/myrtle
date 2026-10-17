/**
 * THE KILL SWITCH. With `VITE_TREVOR` unset (the default) the reader and the
 * chapter sheet render nothing of Trevor, even when the backend has a panel to
 * give; with `VITE_TREVOR=1` the toolbar button and the chapter primer appear.
 *
 * This file checks the structure. Whether the flag-off render is byte for
 * byte the render without Trevor (`document.body.innerHTML` of the tutorial
 * reader and the chapter sheet, compared with `cmp`) is checked by hand.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import type React from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import welcomeJson from "#/lib/story/__fixtures__/main_0_0_welcome_to_guide.json";
import type { StoryProgress } from "#/lib/story/progress";
import sample from "#/lib/trevor/__fixtures__/panel_act18side_05_beg.json";
import type { StoryScript } from "#/types/generated/StoryScript";
import type { LibEntry, LibGroup } from "../library/impl/derive";

vi.mock("@tanstack/react-start", () => ({
    createServerFn: () => {
        const builder = {
            inputValidator: () => builder,
            handler: (fn: (args: { data: unknown }) => unknown) => (opts?: { data?: unknown }) => fn({ data: opts?.data }),
        };
        return builder;
    },
}));
vi.mock("@tanstack/react-start/server", () => ({ getCookie: () => undefined, getRequest: () => new Request("http://x") }));
vi.mock("@tanstack/react-router", () => ({
    Link: ({ children, className }: { children?: React.ReactNode; className?: string }) => (
        <a href="/stories" className={className}>
            {children}
        </a>
    ),
    useNavigate: () => () => undefined,
}));
vi.mock("#/hooks/use-auth", () => ({ useAuth: () => ({ user: null }) }));
vi.mock("#/lib/story/audio", () => ({
    createStoryAudio: () => new Proxy({}, { get: (_t, k) => (k === "armed" || k === "lastFallback" ? false : () => undefined) }),
}));

const PRIMER = "In the Kazimierz Major, Margaret Nearl defeated the Blood Knight.";
const panelRequests: string[] = [];
vi.mock("#/lib/fetch", () => ({
    backendFetch: vi.fn(async (path: string) => {
        if (path.startsWith("/trevor/panels/")) {
            panelRequests.push(path);
            const panels = { ...sample, recap: { ...sample.recap, beforeYouStart: { primer: PRIMER, prior: [] } } };
            return Response.json({ storyId: "m8a", server: "en", version: "v", panels });
        }
        if (path.startsWith("/trevor/status")) return Response.json({ online: true, rungs: { cf: true, cpu: true }, activeVersion: "v", queueDepth: { cf: 0, cpu: 0 }, cfBudget: "ok" });
        return new Response("", { status: 404 });
    }),
}));

const { StoryReader } = await import("../reader/StoryReader");
const { ChapterModal } = await import("../library/impl/ChapterModal");

function client(): QueryClient {
    return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}

function mountReader(): void {
    render(
        <QueryClientProvider client={client()}>
            <I18nProvider locale="en" available={[]} messages={{}}>
                <StoryReader script={welcomeJson as StoryScript} entry={null} groupName="Prologue" category="main" previous={null} next={null} />
            </I18nProvider>
        </QueryClientProvider>,
    );
}

function entry(id: string, code: string): LibEntry {
    return { id, name: id, groupId: "main_8", sort: 0, hasScript: true, code, requiredStages: [], wordCount: 1200 } as unknown as LibEntry;
}
const group = { id: "main_8", name: "Roaring Flare", category: "main", entryType: "NONE", actType: "NONE", startTime: -1, stories: [entry("m8a", "8-1"), entry("m8b", "8-2")] } as unknown as LibGroup;

function mountChapter(): void {
    render(
        <QueryClientProvider client={client()}>
            <I18nProvider locale="en" available={[]} messages={{}}>
                <ChapterModal group={group} progress={{ v: 2, read: {} } as StoryProgress} gameRead={new Set()} onClose={() => {}} />
            </I18nProvider>
        </QueryClientProvider>,
    );
}

/** Let lazy chunks and queries settle. */
async function settle(): Promise<void> {
    await act(async () => {
        await new Promise((r) => setTimeout(r, 30));
    });
}

const leftPillButtons = () => document.querySelectorAll('[data-story-pill="left"] button, [data-story-pill="left"] a').length;

beforeEach(() => {
    panelRequests.length = 0;
    if (!("getAnimations" in Element.prototype)) Object.defineProperty(Element.prototype, "getAnimations", { value: () => [], configurable: true });
});
afterEach(() => {
    cleanup();
    vi.unstubAllEnvs();
});

describe("Trevor off (VITE_TREVOR unset)", () => {
    it("leaves the reader's toolbar without the button, and mounts no sheet", async () => {
        vi.stubEnv("VITE_TREVOR", "");
        mountReader();
        await settle();
        expect(document.querySelector('[data-story-nav="trevor"]')).toBeNull();
        expect(document.querySelector("[data-trevor-sheet]")).toBeNull();
        // Library, Settings, Log, Hide, Previous, Next, Collapse.
        expect(leftPillButtons()).toBe(7);
        expect(panelRequests).toEqual([]);
    });

    it("leaves the chapter sheet without a primer, and asks the backend for nothing", async () => {
        vi.stubEnv("VITE_TREVOR", "");
        mountChapter();
        await settle();
        expect(document.querySelector("[data-trevor-primer]")).toBeNull();
        expect(panelRequests).toEqual([]);
    });

    it("treats any value but 1 as off", async () => {
        vi.stubEnv("VITE_TREVOR", "true");
        mountReader();
        await settle();
        expect(document.querySelector('[data-story-nav="trevor"]')).toBeNull();
    });
});

describe("Trevor on (VITE_TREVOR=1)", () => {
    it("adds one toolbar button that opens the sheet", async () => {
        vi.stubEnv("VITE_TREVOR", "1");
        mountReader();
        await settle();
        expect(leftPillButtons()).toBe(8);
        const button = document.querySelector<HTMLElement>('[data-story-nav="trevor"]');
        expect(button?.getAttribute("aria-label")).toBe("Trevor");
        await act(async () => {
            fireEvent.click(button as HTMLElement);
        });
        await waitFor(() => expect(document.querySelector("[data-trevor-sheet]")).not.toBeNull());
        await waitFor(() => expect(panelRequests).toEqual(["/trevor/panels/main_0_0_welcome_to_guide?v=v"]));
    });

    it("shows the first story's primer in the chapter sheet", async () => {
        vi.stubEnv("VITE_TREVOR", "1");
        mountChapter();
        await waitFor(() => expect(document.querySelector("[data-trevor-primer]")).not.toBeNull());
        expect(panelRequests).toEqual(["/trevor/panels/m8a"]);
        expect(document.querySelector("[data-trevor-primer]")?.textContent).toContain("Before you start");
    });
});
