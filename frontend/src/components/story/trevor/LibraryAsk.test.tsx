/**
 * The stories page's Ask Trevor tab: signed out it explains and requests
 * nothing; signed in it is the chat with the era's popular questions and My
 * questions, and both lists are optional (a 404 leaves the chat working).
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type React from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import type { IAskDeps } from "#/lib/trevor/useTrevorAsk";
import type { TrevorAnswer } from "#/types/generated/TrevorAnswer";
import type { TrevorStatus } from "#/types/generated/TrevorStatus";

vi.mock("@tanstack/react-start", () => ({
    createServerFn: () => {
        const builder = {
            inputValidator: () => builder,
            handler: (fn: (args: { data: unknown }) => unknown) => (opts?: { data?: unknown }) => fn({ data: opts?.data }),
        };
        return builder;
    },
}));
vi.mock("@tanstack/react-start/server", () => ({ getCookie: () => "tok", getRequest: () => new Request("http://x") }));
vi.mock("@tanstack/react-router", () => ({
    Link: ({ children, className }: { children?: React.ReactNode; className?: string }) => (
        <a href="/stories" className={className}>
            {children}
        </a>
    ),
    useNavigate: () => () => undefined,
}));
const auth = { user: null as null | { id: string } };
vi.mock("#/hooks/use-auth", () => ({ useAuth: () => auth }));

const ERA = { eraId: "main_10", name: "Shatterpoint", position: 722 };
const stored = (text: string): TrevorAnswer => ({ id: `a-${text}`, version: "v", server: "en", eraPos: 722, rung: "cpu", source: "live", answer: { answer: text, citations: [{ n: 1, storyId: "s1", chunkId: "c1", lineStart: 3, lineEnd: 5, text: "Quote." }] } });
let routes: Record<string, () => Response> = {};
const requests: string[] = [];
vi.mock("#/lib/fetch", () => ({
    backendFetch: vi.fn(async (path: string) => {
        requests.push(path.split("?")[0] ?? path);
        for (const [prefix, answer] of Object.entries(routes)) if (path.startsWith(prefix)) return answer();
        return new Response("", { status: 404 });
    }),
}));

const { LibraryAsk } = await import("./LibraryAsk");

const asked: { question: string; horizonStory?: string | null }[] = [];
const deps: IAskDeps = {
    ask: async (input) => {
        asked.push(input);
        return { jobId: null, status: "done", rung: "cache", spoilerSafeUpTo: ERA, answer: stored("Asked."), etaSecs: null, streamUrl: null };
    },
    openStream: async () => new Response("", { status: 500 }),
    getJob: async (jobId) => ({ jobId, status: "done", rung: "cpu", position: 0, etaSecs: null, workerOnline: true, answer: stored("Followed in full."), error: null }),
    getStatus: async () => ({ online: true, rungs: { cf: false, cpu: true }, activeVersion: "v", queueDepth: { cf: 0, cpu: 0 }, cfBudget: "ok", workers: { cf: 0, cpu: 1 }, guard: "ok", mode: "on" }) as TrevorStatus,
};

function mount(): void {
    render(
        <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
            <I18nProvider locale="en" available={[]} messages={{}}>
                <LibraryAsk active askDeps={deps} />
            </I18nProvider>
        </QueryClientProvider>,
    );
}

beforeEach(() => {
    routes = {
        "/trevor/status": () => Response.json({ online: true, rungs: { cf: false, cpu: true }, activeVersion: "v", queueDepth: { cf: 0, cpu: 0 }, cfBudget: "ok" }),
    };
    requests.length = 0;
    asked.length = 0;
    auth.user = { id: "u1" };
    if (!("getAnimations" in Element.prototype)) Object.defineProperty(Element.prototype, "getAnimations", { value: () => [], configurable: true });
    if (!("fonts" in document)) Object.defineProperty(document, "fonts", { value: { addEventListener: () => {}, removeEventListener: () => {} }, configurable: true });
});
afterEach(() => cleanup());

describe("Ask Trevor on the stories page", () => {
    it("signed out: explains, offers sign in, and requests nothing", async () => {
        auth.user = null;
        mount();
        await act(async () => {
            await new Promise((r) => setTimeout(r, 20));
        });
        expect(document.querySelector("[data-trevor-signed-out]")?.textContent).toContain("follow your reading progress");
        expect(document.querySelector("[data-trevor-chat]")).toBeNull();
        expect(requests).toEqual([]);
    });

    it("works with neither list when the backend has no such routes (404)", async () => {
        mount();
        await waitFor(() => expect(requests).toEqual(expect.arrayContaining(["/trevor/popular", "/trevor/me/jobs"])));
        expect(document.querySelector("[data-trevor-chat]")).not.toBeNull();
        await waitFor(() => expect(document.querySelector("[data-trevor-mine]")).toBeNull());
        expect(document.querySelector("[data-trevor-popular]")).toBeNull();
        expect(document.querySelector("[data-trevor-library-empty]")?.textContent).toContain("Answers stop at the furthest story you have read.");
        expect(document.querySelector("[data-trevor-question]")).not.toBeNull();
    });

    it("shows the era's popular questions and My questions, and opens each into the thread with no story horizon", async () => {
        routes["/trevor/popular"] = () =>
            Response.json({
                items: [
                    { question: "Who is Amiya?", answer: stored("Prepared."), askers: 3 },
                    { question: "Why Chernobog?", answer: null, askers: 2 },
                ],
                spoilerSafeUpTo: ERA,
            });
        routes["/trevor/me/jobs"] = () => Response.json({ jobs: [{ jobId: "j7", question: "What is Originium?", status: "queued", rung: "cpu", deferred: true, createdAt: "2026-10-17T10:00:00Z", answer: null, quickLook: stored("Passages.") }], nextBefore: null });
        mount();
        await waitFor(() => expect(document.querySelectorAll("[data-trevor-popular] button").length).toBe(2));
        expect(document.querySelector("[data-trevor-spoiler]")?.textContent).toContain("Shatterpoint");
        expect(document.querySelectorAll('[data-trevor-mine-job="j7"]').length).toBe(1);
        expect(document.querySelector('[data-trevor-mine-job="j7"]')?.textContent).toContain("Quick look given, full answer coming");

        await act(async () => {
            fireEvent.click(document.querySelector("[data-trevor-popular] [data-trevor-prepared]") as HTMLElement);
        });
        expect(asked).toEqual([]);
        expect(document.querySelector("[data-trevor-turn]")?.textContent).toContain("Prepared.");

        await act(async () => {
            fireEvent.click(document.querySelector('[data-trevor-mine-job="j7"]') as HTMLElement);
        });
        // The open past question is followed again until its full answer replaces the quick look.
        await waitFor(() => expect(document.body.textContent).toContain("Followed in full."));
    });

    it("asks a popular question that has no stored answer, with no story horizon", async () => {
        routes["/trevor/popular"] = () => Response.json({ items: [{ question: "Why Chernobog?", answer: null, askers: 2 }], spoilerSafeUpTo: ERA });
        mount();
        await waitFor(() => expect(document.querySelectorAll("[data-trevor-popular] button").length).toBe(1));
        await act(async () => {
            fireEvent.click(document.querySelector("[data-trevor-popular] button") as HTMLElement);
        });
        await waitFor(() => expect(asked).toEqual([{ question: "Why Chernobog?", horizonStory: null, turnstileToken: null }]));
        await waitFor(() => expect(document.body.textContent).toContain("Asked."));
    });

    it("keeps the unasked popular questions above the composer after the first question, and asks from there", async () => {
        routes["/trevor/popular"] = () =>
            Response.json({
                items: [
                    { question: "Who is Amiya?", answer: stored("Prepared."), askers: 3 },
                    { question: "Why Chernobog?", answer: null, askers: 2 },
                    { question: "What is Rhodes Island?", answer: null, askers: 1 },
                ],
                spoilerSafeUpTo: ERA,
            });
        mount();
        await waitFor(() => expect(document.querySelectorAll("[data-trevor-popular] button").length).toBe(3));
        expect(document.querySelector("[data-trevor-chip-row]")).toBeNull();
        await act(async () => {
            fireEvent.click(document.querySelector("[data-trevor-popular] [data-trevor-prepared]") as HTMLElement);
        });
        const row = () => [...document.querySelectorAll("[data-trevor-chip-row] [data-trevor-chip]")].map((b) => b.textContent);
        // The prepared one was opened: the other 2 stay, labelled as the era's popular questions.
        expect(row()).toEqual(["Why Chernobog?", "What is Rhodes Island?"]);
        expect(document.querySelector("[data-trevor-chip-row]")?.getAttribute("aria-label")).toBe("Asked often in your era");
        await act(async () => {
            fireEvent.click(document.querySelector("[data-trevor-chip-row] [data-trevor-chip]") as HTMLElement);
        });
        await waitFor(() => expect(asked).toEqual([{ question: "Why Chernobog?", horizonStory: null, turnstileToken: null }]));
        expect(row()).toEqual(["What is Rhodes Island?"]);
        // A typed question that matches a chip (case and spaces aside) hides it too.
        const box = document.querySelector("[data-trevor-question]") as HTMLTextAreaElement;
        await act(async () => {
            box.value = "  what is rhodes island? ";
            fireEvent.input(box);
        });
        // jsdom reports no fine pointer, so Enter is a new line there: send with the button.
        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "Ask" }));
        });
        await waitFor(() => expect(asked.map((x) => x.question)).toEqual(["Why Chernobog?", "what is rhodes island?"]));
        expect(document.querySelector("[data-trevor-chip-row]")).toBeNull();
    });
});
