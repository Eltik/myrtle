/**
 * The Trevor sheet rendered from a real published panel (Lingering Echoes,
 * LE-5, rank 776) with the backend faked at `backendFetch`, and the chat
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

const STATUS = { online: true, rungs: { cf: true, cpu: true }, activeVersion: "en-e79e364cfbdeeffa-pef0c5bba", queueDepth: { cf: 0, cpu: 0 }, cfBudget: "ok", workers: { cf: 1, cpu: 1 }, guard: "ok", mode: "on" } as TrevorStatus;
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

/** The live answer with other text, same sources. */
function withText(text: string): TrevorAnswer {
    return { ...LIVE_ANSWER, answer: { ...(LIVE_ANSWER.answer as Record<string, unknown>), answer: text } as TrevorAnswer["answer"] };
}

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
    // Nor a font set; the composer re-measures its rows when fonts load.
    if (!("fonts" in document)) Object.defineProperty(document, "fonts", { value: { addEventListener: () => {}, removeEventListener: () => {} }, configurable: true });
});
afterEach(() => {
    cleanup();
    vi.clearAllMocks();
});

function chip(id: string): HTMLButtonElement {
    const el = sheet().querySelector<HTMLButtonElement>(`[data-trevor-suggestion="${id}"] button`);
    if (!el) throw new Error(`the suggestion ${id} did not render`);
    return el;
}

describe("the Trevor sheet, from a published panel", () => {
    it("opens on the chat, with the recap one tab away under the active version", async () => {
        mount();
        await waitFor(() => expect(sheet().querySelector("[data-trevor-suggestion]")).not.toBeNull());
        expect(within(sheet()).getByRole("tab", { name: "Ask" }).getAttribute("aria-selected")).toBe("true");
        expect(requests).toContain(`/trevor/panels/${STORY}?v=${STATUS.activeVersion}`);
        expect(within(sheet()).getByText("Lingering Echoes, LE-5 Moonlight (Before Operation)")).toBeTruthy();
        await openTab("Story so far");
        expect(within(sheet()).getByText("Previously")).toBeTruthy();
        expect(within(sheet()).getByText("Earlier in Lingering Echoes")).toBeTruthy();
        // 10 earlier stories, 8 listed.
        expect(within(sheet()).getByText("2 earlier stories are not listed.")).toBeTruthy();
        // The story's own summary is a spoiler for it: closed until asked for.
        expect(sheet().querySelector("[data-trevor-this-story]")).toBeNull();
    });

    it("lists who speaks and the terms named", async () => {
        mount();
        await waitFor(() => expect(sheet().querySelector("[data-trevor-suggestion]")).not.toBeNull());
        await openTab("Who's here");
        expect(within(sheet()).getByText("Ebenholz, a Leithanien commoner who became infected with Oripathy during the Vyseheim incident.")).toBeTruthy();
        expect(within(sheet()).getAllByText("First seen in Lingering Echoes").length).toBeGreaterThan(0);
        await openTab("Terms");
        expect(within(sheet()).getByText("Afterglow")).toBeTruthy();
        expect(within(sheet()).getAllByText("Place").length).toBeGreaterThan(0);
    });

    it("adds a suggested question's prepared answer to the thread, with no request", async () => {
        const d = deps();
        mount(d);
        await waitFor(() => expect(sheet().querySelector("[data-trevor-suggestion]")).not.toBeNull());
        const button = chip("ask:b00734");
        expect(button.hasAttribute("data-trevor-prepared")).toBe(true);
        const question = button.textContent ?? "";
        await act(async () => {
            fireEvent.click(button);
        });
        const turn = sheet().querySelector<HTMLElement>('[data-trevor-turn="bank"]');
        expect(turn?.textContent).toContain(question);
        expect(turn?.querySelector("[data-trevor-bank-answer]")?.textContent).toContain("his father was in a bad condition");
        expect(within(turn as HTMLElement).getByText("A prepared answer, spoiler-safe up to this story.")).toBeTruthy();
        // The chips belong to the empty thread.
        expect(sheet().querySelector("[data-trevor-suggestion]")).toBeNull();
        expect(d.ask).not.toHaveBeenCalled();
    });

    it("asks a signed-out reader to sign in in place of the composer", async () => {
        mount();
        await waitFor(() => expect(sheet().querySelector("[data-trevor-signed-out]")).not.toBeNull());
        expect(sheet().querySelector("[data-trevor-question]")).toBeNull();
        fireEvent.click(within(sheet()).getByRole("button", { name: "Sign in" }));
        expect(login).toHaveBeenCalledTimes(1);
    });
});

describe("the ask box", () => {
    beforeEach(() => {
        auth.user = { uid: "1" };
    });

    function box(): HTMLTextAreaElement {
        const el = sheet().querySelector<HTMLTextAreaElement>("[data-trevor-question]");
        if (!el) throw new Error("the composer did not render");
        return el;
    }

    async function ask(question: string): Promise<void> {
        await waitFor(() => box());
        await act(async () => {
            box().value = question;
            fireEvent.input(box());
        });
        // jsdom reports no fine pointer, so Enter is a new line there: send with
        // the button, focused first as a keyboard user's would be.
        const send = within(sheet()).getByRole("button", { name: "Ask" });
        await act(async () => {
            send.focus();
            fireEvent.click(send);
        });
    }

    /** The question being followed: the live turn's reply region. */
    function live(): HTMLElement | null {
        return sheet().querySelector<HTMLElement>("[data-trevor-live]");
    }

    function state(): string | null {
        return live()?.getAttribute("data-trevor-state") ?? null;
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
        expect(sheet().querySelector("[data-trevor-asked]")?.textContent).toBe("Who is Hibiscus?");
        // The draft was sent and cleared: the button waits for the next question, the box stays open.
        expect((within(sheet()).getByRole("button", { name: "Ask" }) as HTMLButtonElement).disabled).toBe(true);
        expect(box().disabled).toBe(false);

        await act(async () => {
            push('event: delta\ndata: {"seq":0,"text":"Hibiscus is"}\n\n');
        });
        await waitFor(() => expect(state()).toBe("streaming"));
        const region = live();
        expect(region?.getAttribute("aria-live")).toBe("polite");
        expect(region?.getAttribute("aria-busy")).toBe("true");
        expect(region?.textContent).toContain("Hibiscus is");

        const done = { jobId: "j1", status: "done", rung: "cf", position: 0, etaSecs: null, workerOnline: true, answer: LIVE_ANSWER, error: null };
        await act(async () => {
            push(`event: done\ndata: ${JSON.stringify(done)}\n\n`);
        });
        await waitFor(() => expect(state()).toBe("done"));
        expect(live()?.getAttribute("aria-busy")).toBe("false");
        // The answer hands the keyboard back to the composer.
        await waitFor(() => expect(document.activeElement).toBe(box()));
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

    it("keeps the earlier questions in the thread as the next one is asked", async () => {
        const cached: AskResponse = { jobId: null, status: "done", rung: "cache", spoilerSafeUpTo: SHATTERPOINT, etaSecs: null, streamUrl: null, answer: LIVE_ANSWER };
        const askFn = vi.fn().mockResolvedValueOnce(cached).mockRejectedValueOnce(new APIError(503, "Trevor is busy"));
        mount(deps({ ask: askFn }));
        await ask("Who is Hibiscus?");
        await waitFor(() => expect(state()).toBe("done"));
        await ask("And her sister?");
        await waitFor(() => expect(state()).toBe("refused"));
        const asked = [...sheet().querySelectorAll("[data-trevor-asked]")].map((e) => e.textContent);
        expect(asked).toEqual(["Who is Hibiscus?", "And her sister?"]);
        // The first answer is still there, frozen, with its sources; the spoiler pill keeps the last horizon.
        expect(sheet().querySelectorAll('[data-trevor-state="done"]').length).toBe(1);
        expect(sheet().querySelector('[data-trevor-state="done"] [data-trevor-citation="1"]')).not.toBeNull();
        expect(within(sheet()).getByText("Trevor is busy. Try again in a minute.")).toBeTruthy();
        expect(within(sheet()).getByText("Spoiler-safe up to Shatterpoint")).toBeTruthy();
    });

    it("finishes an answer whose stream drops mid-sentence, from the job's own answer", async () => {
        const enc = new TextEncoder();
        let ctrl!: ReadableStreamDefaultController<Uint8Array>;
        const body = new ReadableStream<Uint8Array>({
            start(c) {
                ctrl = c;
            },
        });
        const full = withText("On this evidence, she is the one keeping the clinic running [1].");
        const getJob = vi.fn(async (): Promise<JobView> => ({ jobId: "j1", status: "done", rung: "cf", position: 0, etaSecs: null, workerOnline: true, answer: full, error: null }));
        mount(deps({ openStream: vi.fn(async () => new Response(body, { status: 200 })), getJob }));
        await ask("Who keeps the clinic running?");
        await waitFor(() => expect(state()).toBe("queued"));
        await act(async () => {
            ctrl.enqueue(enc.encode('event: delta\ndata: {"seq":0,"text":"On this evidence, she"}\n\n'));
            ctrl.close();
        });
        await waitFor(() => expect(state()).toBe("done"));
        expect(live()?.textContent).toContain("she is the one keeping the clinic running");
    });

    it("answers several questions at once, in the order asked, and waits past the open-job cap", async () => {
        const streams = new Map<string, { push: (s: string) => void; close: () => void }>();
        const enc = new TextEncoder();
        let n = 0;
        const askFn = vi.fn(async (): Promise<AskResponse> => {
            n += 1;
            return { jobId: `j${n}`, status: "queued", rung: "cf", spoilerSafeUpTo: SHATTERPOINT, etaSecs: 20, streamUrl: null };
        });
        const openStream = vi.fn(async (jobId: string) => {
            const body = new ReadableStream<Uint8Array>({
                start(c) {
                    streams.set(jobId, { push: (s) => c.enqueue(enc.encode(s)), close: () => c.close() });
                },
            });
            return new Response(body, { status: 200 });
        });
        mount(deps({ ask: askFn, openStream }));
        for (const q of ["First?", "Second?", "Third?", "Fourth?"]) await ask(q);
        await waitFor(() => expect(streams.size).toBe(3));
        const states = () => [...sheet().querySelectorAll("[data-trevor-state]")].map((e) => e.getAttribute("data-trevor-state"));
        // Three in flight, the fourth waits here and says why.
        expect(askFn).toHaveBeenCalledTimes(3);
        expect(states()).toEqual(["queued", "queued", "queued", "held"]);
        expect(sheet().querySelector("[data-trevor-held]")?.textContent).toContain("3 questions are already with Trevor");
        // All three stream at once.
        const delta = (id: string, text: string) => streams.get(id)?.push(`event: delta\ndata: ${JSON.stringify({ seq: 0, text })}\n\n`);
        await act(async () => {
            delta("j1", "One");
            delta("j2", "Two");
            delta("j3", "Three");
        });
        await waitFor(() => expect(states()).toEqual(["streaming", "streaming", "streaming", "held"]));
        // The third finishes first; the second fails; neither disturbs the first.
        const doneView = (jobId: string, text: string) => ({ jobId, status: "done", rung: "cf", position: 0, etaSecs: null, workerOnline: true, answer: withText(text), error: null });
        await act(async () => {
            streams.get("j3")?.push(`event: done\ndata: ${JSON.stringify(doneView("j3", "Third answer."))}\n\n`);
            streams.get("j2")?.push('event: error\ndata: {"message":"worker died"}\n\n');
        });
        await waitFor(() => expect(states()).toEqual(["streaming", "failed", "done", "queued"]));
        expect(askFn).toHaveBeenCalledTimes(4);
        const asked = [...sheet().querySelectorAll("[data-trevor-asked]")].map((e) => e.textContent);
        expect(asked).toEqual(["First?", "Second?", "Third?", "Fourth?"]);
        expect(sheet().querySelectorAll('[data-trevor-state="streaming"]')[0]?.textContent).toContain("One");
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
