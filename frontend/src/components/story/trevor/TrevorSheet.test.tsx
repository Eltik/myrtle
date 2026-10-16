/**
 * The Trevor sheet rendered from a real published panel (Lingering Echoes,
 * LE-5, rank 776) with the backend faked at `backendFetch`, and the ask box
 * driven through its states with a fake ask, stream and job.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, cleanup, fireEvent, render, waitFor, within } from "@testing-library/react";
import type React from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { APIError } from "#/lib/api/_shared";
import { I18nProvider } from "#/lib/i18n/context";
import sample from "#/lib/trevor/__fixtures__/panel_act18side_05_beg.json";
import type { IAskDeps } from "#/lib/trevor/useTrevorAsk";
import type { AskResponse } from "#/types/generated/AskResponse";
import type { JobView } from "#/types/generated/JobView";
import type { TrevorAnswer } from "#/types/generated/TrevorAnswer";
import type { TrevorStatus } from "#/types/generated/TrevorStatus";

const STORY = "act18side_level_act18side_05_beg";
const auth = vi.hoisted(() => ({ user: null as { uid: string } | null }));
const login = vi.hoisted(() => vi.fn());

vi.mock("@tanstack/react-start", () => ({
    createServerFn: () => {
        const builder = {
            inputValidator: () => builder,
            handler: (fn: (args: { data: unknown }) => unknown) => (opts?: { data?: unknown }) => fn({ data: opts?.data }),
        };
        return builder;
    },
}));
vi.mock("@tanstack/react-start/server", () => ({ getCookie: () => "token", getRequest: () => new Request("http://x") }));
vi.mock("@tanstack/react-router", () => ({
    Link: ({ children, className, params, search }: { children?: React.ReactNode; className?: string; params?: { storyId?: string }; search?: { line?: number } }) => (
        <a href={`/stories/${params?.storyId ?? ""}${search?.line ? `?line=${search.line}` : ""}`} className={className}>
            {children}
        </a>
    ),
}));
vi.mock("#/hooks/use-auth", () => ({ useAuth: () => ({ user: auth.user }) }));
vi.mock("#/lib/auth/store", () => ({ authActions: { openLoginDialog: login } }));

const STATUS: TrevorStatus = { online: true, rungs: { cf: true, cpu: true }, activeVersion: "en-e79e364cfbdeeffa-pef0c5bba", queueDepth: { cf: 0, cpu: 0 }, cfBudget: "ok" };
const INDEX = {
    groups: [
        {
            id: "act18side",
            name: "Lingering Echoes",
            category: "side",
            stories: [
                { id: "act18side_level_act18side_04_end", name: "Rondo", code: "LE-4", sort: 1 },
                { id: STORY, name: "Moonlight", code: "LE-5", sort: 2 },
            ],
        },
    ],
    records: [],
    storylines: [],
    totals: {},
};
const requests: string[] = [];
vi.mock("#/lib/fetch", () => ({
    backendFetch: vi.fn(async (path: string) => {
        requests.push(path);
        if (path.startsWith("/trevor/status")) return Response.json(STATUS);
        if (path.startsWith("/trevor/panels/")) return Response.json({ storyId: STORY, server: "en", version: STATUS.activeVersion, panels: sample });
        if (path.includes("/story/index")) return Response.json(INDEX);
        return new Response("", { status: 404 });
    }),
}));

const { TrevorSheet } = await import("./TrevorSheet");

const SHATTERPOINT = { eraId: "main_10", name: "Shatterpoint", position: 722 };
const LIVE_ANSWER: TrevorAnswer = {
    id: "ans-1",
    version: "v",
    server: "en",
    eraPos: 722,
    rung: "cf",
    source: "live",
    answer: {
        answer: "**Hibiscus** is a nurse [1].",
        citations: [
            { n: 1, storyId: "act18side_level_act18side_04_end", chunkId: "c#1", lineStart: 41, lineEnd: 50, text: "Hibiscus: Hold still." },
            { n: 2, storyId: STORY, chunkId: "c#2", lineStart: 12, lineEnd: 14, text: "Here." },
        ],
    },
};

function deps(over: Partial<IAskDeps> = {}): IAskDeps {
    return {
        ask: vi.fn(async (): Promise<AskResponse> => ({ jobId: "j1", status: "queued", rung: "cpu", spoilerSafeUpTo: SHATTERPOINT, etaSecs: 120, streamUrl: "/api/trevor/jobs/j1/stream" })),
        openStream: vi.fn(async () => Promise.reject(new Error("no stream in this test"))),
        getJob: vi.fn(async (): Promise<JobView> => ({ jobId: "j1", status: "queued", rung: "cpu", position: 2, etaSecs: 120, workerOnline: true, answer: null, error: null })),
        getStatus: vi.fn(async () => STATUS),
        ...over,
    };
}

function mount(askDeps: IAskDeps = deps(), onJumpLine = vi.fn()) {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
        <QueryClientProvider client={client}>
            <I18nProvider locale="en" available={[]} messages={{}}>
                <TrevorSheet storyId={STORY} open onOpenChange={() => {}} askDeps={askDeps} onJumpLine={onJumpLine} />
            </I18nProvider>
        </QueryClientProvider>,
    );
    return { onJumpLine };
}

function sheet(): HTMLElement {
    const el = document.querySelector<HTMLElement>("[data-trevor-sheet]");
    if (!el) throw new Error("the sheet did not render");
    return el;
}

async function openTab(name: string): Promise<void> {
    const tab = within(sheet()).getByRole("tab", { name });
    await act(async () => {
        fireEvent.click(tab);
    });
    await waitFor(() => expect(tab.getAttribute("aria-selected")).toBe("true"));
}

beforeEach(() => {
    auth.user = null;
    requests.length = 0;
    // jsdom has no Web Animations; the sheet's scroll area asks for them.
    if (!("getAnimations" in Element.prototype)) Object.defineProperty(Element.prototype, "getAnimations", { value: () => [], configurable: true });
});
afterEach(() => {
    cleanup();
    vi.clearAllMocks();
});

describe("the Trevor sheet, from a published panel", () => {
    it("asks for the panel under the active version and opens on the recap", async () => {
        mount();
        await waitFor(() => expect(within(sheet()).getByText("Previously")).toBeTruthy());
        expect(requests).toContain(`/trevor/panels/${STORY}?v=${STATUS.activeVersion}`);
        expect(within(sheet()).getByText("Lingering Echoes, LE-5 Moonlight (Before Operation)")).toBeTruthy();
        expect(within(sheet()).getByText("Earlier in Lingering Echoes")).toBeTruthy();
        // 10 earlier stories, 8 listed.
        expect(within(sheet()).getByText("2 earlier stories are not listed.")).toBeTruthy();
        // The story's own summary is a spoiler for it: closed until asked for.
        expect(sheet().querySelector("[data-trevor-this-story]")).toBeNull();
    });

    it("lists who speaks and the terms named", async () => {
        mount();
        await waitFor(() => expect(within(sheet()).getByText("Previously")).toBeTruthy());
        await openTab("Who's here");
        expect(within(sheet()).getByText("Ebenholz, a Leithanien commoner who became infected with Oripathy during the Vyseheim incident.")).toBeTruthy();
        expect(within(sheet()).getAllByText("First seen in Lingering Echoes").length).toBeGreaterThan(0);
        await openTab("Terms");
        expect(within(sheet()).getByText("Afterglow")).toBeTruthy();
        expect(within(sheet()).getAllByText("Place").length).toBeGreaterThan(0);
    });

    it("opens a suggested question's prepared answer in place, with no request", async () => {
        const d = deps();
        mount(d);
        await waitFor(() => expect(within(sheet()).getByText("Previously")).toBeTruthy());
        await openTab("Ask");
        const item = sheet().querySelector<HTMLElement>('[data-trevor-suggestion="ask:b00734"]');
        if (!item) throw new Error("the suggestion did not render");
        const button = within(item).getByRole("button");
        expect(button.getAttribute("aria-expanded")).toBe("false");
        fireEvent.click(button);
        expect(button.getAttribute("aria-expanded")).toBe("true");
        const answer = item.querySelector<HTMLElement>("[data-trevor-bank-answer]");
        expect(answer?.hidden).toBe(false);
        expect(answer?.textContent).toContain("his father was in a bad condition");
        expect(d.ask).not.toHaveBeenCalled();
    });

    it("asks a signed-out reader to sign in rather than showing the box", async () => {
        mount();
        await openTab("Ask");
        expect(sheet().querySelector("[data-trevor-signed-out]")).not.toBeNull();
        expect(sheet().querySelector("[data-trevor-question]")).toBeNull();
        fireEvent.click(within(sheet()).getByRole("button", { name: "Sign in" }));
        expect(login).toHaveBeenCalledTimes(1);
    });
});

describe("the ask box", () => {
    beforeEach(() => {
        auth.user = { uid: "1" };
    });

    async function ask(question: string): Promise<void> {
        await openTab("Ask");
        const box = sheet().querySelector<HTMLTextAreaElement>("[data-trevor-question]");
        if (!box) throw new Error("the question box did not render");
        fireEvent.change(box, { target: { value: question } });
        await act(async () => {
            fireEvent.keyDown(box, { key: "Enter" });
        });
    }

    function state(): string | null {
        return sheet().querySelector("[data-trevor-state]")?.getAttribute("data-trevor-state") ?? null;
    }

    it("streams the answer into a busy live region, then shows it with its sources and spoiler label", async () => {
        const enc = new TextEncoder();
        let push!: (s: string) => void;
        const body = new ReadableStream<Uint8Array>({
            start(c) {
                push = (s) => c.enqueue(enc.encode(s));
            },
        });
        const d = deps({ openStream: vi.fn(async () => new Response(body, { status: 200 })) });
        const { onJumpLine } = mount(d);
        await ask("Who is Hibiscus?");
        expect(d.ask).toHaveBeenCalledWith({ question: "Who is Hibiscus?", horizonStory: STORY, turnstileToken: null });
        await waitFor(() => expect(state()).toBe("queued"));
        expect(within(sheet()).getByText("Queued · about 2 minutes")).toBeTruthy();

        await act(async () => {
            push('event: delta\ndata: {"seq":0,"text":"Hibiscus is"}\n\n');
        });
        await waitFor(() => expect(state()).toBe("streaming"));
        const region = sheet().querySelector<HTMLElement>("[data-trevor-state]");
        expect(region?.getAttribute("aria-live")).toBe("polite");
        expect(region?.getAttribute("aria-busy")).toBe("true");
        expect(region?.textContent).toContain("Hibiscus is");

        const done = { jobId: "j1", status: "done", rung: "cf", position: 0, etaSecs: null, workerOnline: true, answer: LIVE_ANSWER, error: null };
        await act(async () => {
            push(`event: done\ndata: ${JSON.stringify(done)}\n\n`);
        });
        await waitFor(() => expect(state()).toBe("done"));
        expect(sheet().querySelector("[data-trevor-state]")?.getAttribute("aria-busy")).toBe("false");
        expect(within(sheet()).getByText("Spoiler-safe up to Shatterpoint")).toBeTruthy();
        expect(sheet().querySelector("[data-trevor-passages]")).toBeNull();
        // Source 1 is another story: a link that opens it at the cited line.
        await waitFor(() => expect(sheet().querySelector('[data-trevor-citation="1"] a')?.getAttribute("href")).toBe("/stories/act18side_level_act18side_04_end?line=41"));
        expect(within(sheet()).getByRole("link", { name: "Source 1" })).toBeTruthy();
        // Source 2 is the story being read: a jump in place, not a link.
        const here = within(sheet().querySelector<HTMLElement>('[data-trevor-citation="2"]') as HTMLElement).getByRole("button", { name: /Lingering Echoes · LE-5 · Moonlight/ });
        fireEvent.click(here);
        expect(onJumpLine).toHaveBeenCalledWith(12);
    });

    it("labels a passages-only answer", async () => {
        const cached: AskResponse = { jobId: null, status: "done", rung: "cache", spoilerSafeUpTo: null, etaSecs: null, streamUrl: null, answer: { ...LIVE_ANSWER, rung: "passages", source: "passages" } };
        mount(deps({ ask: vi.fn(async () => cached) }));
        await ask("Who is Hibiscus?");
        await waitFor(() => expect(state()).toBe("done"));
        expect(within(sheet()).getByText("Quick look: passages only")).toBeTruthy();
        expect(within(sheet()).getByText("Spoiler-safe up to the very start. Mark stories read to widen it.")).toBeTruthy();
    });

    it.each([
        [429, "You've reached this hour's question limit. Try again later."],
        [503, "Trevor is busy. Try again in a minute."],
        [403, "Couldn't confirm you're not a bot. Try again."],
        [413, "That question is too long."],
    ])("says what a %i means", async (status, text) => {
        mount(
            deps({
                ask: vi.fn(async () => {
                    throw new APIError(status, "refused");
                }),
            }),
        );
        await ask("Who is Hibiscus?");
        await waitFor(() => expect(state()).toBe("refused"));
        expect(within(sheet()).getByText(text)).toBeTruthy();
    });

    it("stops waiting when nobody is online, and checks again on request", async () => {
        const offline = { ...STATUS, online: false };
        const getJob = vi.fn(async (): Promise<JobView> => ({ jobId: "j1", status: "queued", rung: "cpu", position: 1, etaSecs: null, workerOnline: false, answer: null, error: null }));
        mount(deps({ getJob, getStatus: vi.fn(async () => offline) }));
        await ask("Who is Hibiscus?");
        await waitFor(() => expect(state()).toBe("offline"));
        expect(within(sheet()).getByText("Trevor's answerer went offline. Your question is still queued.")).toBeTruthy();
        const polls = getJob.mock.calls.length;
        await act(async () => {
            fireEvent.click(within(sheet()).getByRole("button", { name: "Check again" }));
        });
        await waitFor(() => expect(getJob.mock.calls.length).toBe(polls + 1));
    });
});
