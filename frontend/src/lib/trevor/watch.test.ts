/**
 * The stream-then-poll fallback, driven with a fake SSE body and fake timers:
 * which transport carries the job, when polling starts, the 5 s interval, and
 * the three ways polling stops (done, failed, offline per /status).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { APIError } from "#/lib/api/_shared";
import type { JobView } from "#/types/generated/JobView";
import type { TrevorStatus } from "#/types/generated/TrevorStatus";
import { createDeltaBuffer, type IWatchDeps, type IWatchHandlers, POLL_MS, STALL_MS, watchJob } from "./watch";

const ANSWER = { id: "a1", version: "v", server: "en", eraPos: 722, rung: "cf", source: "live", answer: { answer: "Hibiscus is a nurse [1].", citations: [] } } as const;

function view(over: Partial<JobView> = {}): JobView {
    return { jobId: "j1", status: "queued", rung: "cpu", position: 2, etaSecs: 40, workerOnline: true, answer: null, error: null, ...over } as JobView;
}

const ONLINE = { online: true, rungs: { cf: true, cpu: true }, activeVersion: "v", queueDepth: { cf: 0, cpu: 1 }, cfBudget: "ok", workers: { cf: 1, cpu: 1 }, guard: "ok", mode: "on" } as TrevorStatus;
const OFFLINE: TrevorStatus = { ...ONLINE, online: false, rungs: { cf: false, cpu: false } };

/** An SSE response whose events are written as the test pushes them. */
function controllableStream() {
    const enc = new TextEncoder();
    let ctrl!: ReadableStreamDefaultController<Uint8Array>;
    const body = new ReadableStream<Uint8Array>({
        start(c) {
            ctrl = c;
        },
    });
    return {
        response: new Response(body, { status: 200, headers: { "content-type": "text/event-stream" } }),
        send: (event: string, data: unknown) => ctrl.enqueue(enc.encode(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`)),
        comment: () => ctrl.enqueue(enc.encode(": keepalive\n\n")),
        close: () => ctrl.close(),
    };
}

function handlers(): IWatchHandlers & { log: string[] } {
    const log: string[] = [];
    return {
        log,
        onText: (t) => log.push(`text:${t}`),
        onProgress: (v) => log.push(`progress:${v.status}:${v.position}`),
        onDone: (v) => log.push(`done:${v.answer?.id ?? "none"}`),
        onJobFailed: (m) => log.push(`failed:${m}`),
        onRequestError: (e) => log.push(`requestError:${(e as { status?: number }).status ?? "?"}`),
        onOffline: () => log.push("offline"),
        onMode: (m) => log.push(`mode:${m}`),
    };
}

/** Let pending promises (stream reads, awaited fetches) run. */
async function flush(): Promise<void> {
    for (let i = 0; i < 10; i++) await Promise.resolve();
}

beforeEach(() => {
    vi.useFakeTimers();
});
afterEach(() => {
    vi.useRealTimers();
});

describe("watchJob over the stream", () => {
    it("assembles deltas in seq order and ends on done without ever polling", async () => {
        const s = controllableStream();
        const getJob = vi.fn();
        const h = handlers();
        watchJob("j1", h, { openStream: async () => s.response, getJob, getStatus: async () => ONLINE });
        await flush();
        s.comment();
        s.send("delta", { seq: 0, text: "Hibiscus " });
        s.send("delta", { seq: 2, text: "nurse." });
        s.send("delta", { seq: 1, text: "is a " });
        s.send("delta", { seq: 1, text: "DUPLICATE " });
        s.send("done", view({ status: "done", answer: ANSWER as unknown as JobView["answer"] }));
        await vi.runAllTimersAsync();
        await flush();
        expect(h.log).toEqual(["mode:stream", "text:Hibiscus ", "text:Hibiscus nurse.", "text:Hibiscus is a nurse.", "text:Hibiscus is a nurse.", "done:a1"]);
        expect(getJob).not.toHaveBeenCalled();
    });

    it("reports a stream error event as the job failing", async () => {
        const s = controllableStream();
        const h = handlers();
        watchJob("j1", h, { openStream: async () => s.response, getJob: vi.fn(), getStatus: async () => ONLINE });
        await flush();
        s.send("error", { message: "worker crashed" });
        await flush();
        expect(h.log).toEqual(["mode:stream", "failed:worker crashed"]);
    });
});

describe("watchJob never leaves an answer half written", () => {
    it("finishes from a poll when the stream drops mid-sentence, the job's answer replacing the streamed text", async () => {
        const s = controllableStream();
        const getJob = vi.fn(async () => view({ status: "done", answer: ANSWER as unknown as JobView["answer"] }));
        const h = handlers();
        watchJob("j1", h, { openStream: async () => s.response, getJob, getStatus: async () => ONLINE });
        await flush();
        s.send("delta", { seq: 0, text: "On this evidence, " });
        s.send("delta", { seq: 1, text: "she" });
        s.close();
        await flush();
        await flush();
        expect(h.log).toEqual(["mode:stream", "text:On this evidence, ", "text:On this evidence, she", "mode:poll", "done:a1"]);
        expect(getJob).toHaveBeenCalledTimes(1);
    });

    it("drops a stream that goes silent past the stall limit and polls, while keep-alives hold a quiet one open", async () => {
        const s = controllableStream();
        const getJob = vi.fn(async () => view({ status: "done", answer: ANSWER as unknown as JobView["answer"] }));
        const h = handlers();
        watchJob("j1", h, { openStream: async () => s.response, getJob, getStatus: async () => ONLINE });
        await flush();
        s.send("delta", { seq: 0, text: "On this evidence, she" });
        await flush();
        // Keep-alives every 15 s: three quiet minutes are not a stall.
        for (let i = 0; i < 12; i++) {
            await vi.advanceTimersByTimeAsync(15_000);
            s.comment();
            await flush();
        }
        expect(getJob).not.toHaveBeenCalled();
        // Then nothing at all: the stream is dropped and the job polled.
        await vi.advanceTimersByTimeAsync(STALL_MS);
        await flush();
        expect(getJob).toHaveBeenCalledTimes(1);
        expect(h.log).toEqual(["mode:stream", "text:On this evidence, she", "mode:poll", "done:a1"]);
    });

    it("keeps the text when a watch on the same job is restarted with its delta buffer", async () => {
        const first = controllableStream();
        const second = controllableStream();
        const opens = [first.response, second.response];
        const buffer = createDeltaBuffer();
        const deps: IWatchDeps = { openStream: async () => opens.shift() as Response, getJob: vi.fn(), getStatus: async () => ONLINE };
        const h = handlers();
        const w = watchJob("j1", h, deps, buffer);
        await flush();
        first.send("delta", { seq: 0, text: "On this " });
        await flush();
        w.stop();
        watchJob("j1", h, deps, buffer);
        await flush();
        second.send("delta", { seq: 1, text: "evidence." });
        await flush();
        expect(h.log.filter((l) => l.startsWith("text:"))).toEqual(["text:On this ", "text:On this evidence."]);
    });
});

describe("watchJob's polling fallback", () => {
    it("polls every 5 s when the stream cannot open, and stops on done", async () => {
        const replies = [view(), view({ status: "running", position: 0 }), view({ status: "done", answer: ANSWER as unknown as JobView["answer"] })];
        const getJob = vi.fn(async () => replies.shift() as JobView);
        const h = handlers();
        watchJob("j1", h, { openStream: async () => Promise.reject(new TypeError("Failed to fetch")), getJob, getStatus: async () => ONLINE });
        await flush();
        expect(getJob).toHaveBeenCalledTimes(1);
        await vi.advanceTimersByTimeAsync(POLL_MS - 1);
        expect(getJob).toHaveBeenCalledTimes(1);
        await vi.advanceTimersByTimeAsync(1);
        expect(getJob).toHaveBeenCalledTimes(2);
        await vi.advanceTimersByTimeAsync(POLL_MS);
        expect(getJob).toHaveBeenCalledTimes(3);
        await vi.advanceTimersByTimeAsync(POLL_MS * 4);
        expect(getJob).toHaveBeenCalledTimes(3);
        expect(h.log).toEqual(["mode:poll", "progress:queued:2", "progress:running:0", "done:a1"]);
    });

    it("falls back when the stream closes before done, keeping the text it had", async () => {
        const s = controllableStream();
        const getJob = vi.fn(async () => view({ status: "failed", error: "expired" }));
        const h = handlers();
        watchJob("j1", h, { openStream: async () => s.response, getJob, getStatus: async () => ONLINE });
        await flush();
        s.send("delta", { seq: 0, text: "Partial" });
        s.close();
        await flush();
        await flush();
        expect(h.log).toEqual(["mode:stream", "text:Partial", "mode:poll", "failed:expired"]);
    });

    it("falls back on a non-2xx stream response", async () => {
        const getJob = vi.fn(async () => view({ status: "done", answer: ANSWER as unknown as JobView["answer"] }));
        const h = handlers();
        watchJob("j1", h, { openStream: async () => new Response("nope", { status: 500 }), getJob, getStatus: async () => ONLINE });
        await flush();
        expect(h.log).toEqual(["mode:poll", "done:a1"]);
    });

    it("stops polling when the job has no worker and /status says offline", async () => {
        const getJob = vi.fn(async () => view({ workerOnline: false, etaSecs: null }));
        const getStatus = vi.fn(async () => OFFLINE);
        const h = handlers();
        watchJob("j1", h, { openStream: async () => Promise.reject(new Error("x")), getJob, getStatus });
        await flush();
        await vi.advanceTimersByTimeAsync(POLL_MS * 5);
        expect(getJob).toHaveBeenCalledTimes(1);
        expect(getStatus).toHaveBeenCalledTimes(1);
        expect(h.log).toEqual(["mode:poll", "progress:queued:2", "offline"]);
    });

    it("keeps polling when the job lacks a worker but /status still reports one", async () => {
        const getJob = vi.fn(async () => view({ workerOnline: false }));
        const h = handlers();
        watchJob("j1", h, { openStream: async () => Promise.reject(new Error("x")), getJob, getStatus: async () => ONLINE });
        await flush();
        await vi.advanceTimersByTimeAsync(POLL_MS * 2);
        expect(getJob).toHaveBeenCalledTimes(3);
        expect(h.log).not.toContain("offline");
    });

    it("retries a transient poll failure but gives up on a 403", async () => {
        const errs = [new APIError(502, "bad gateway"), new APIError(403, "not yours")];
        const getJob = vi.fn(async () => {
            throw errs.shift();
        });
        const h = handlers();
        watchJob("j1", h, { openStream: async () => Promise.reject(new Error("x")), getJob, getStatus: async () => ONLINE });
        await flush();
        await vi.advanceTimersByTimeAsync(POLL_MS);
        await vi.advanceTimersByTimeAsync(POLL_MS * 3);
        expect(getJob).toHaveBeenCalledTimes(2);
        expect(h.log).toEqual(["mode:poll", "requestError:403"]);
    });

    it("stop() ends polling and makes no further requests", async () => {
        const getJob = vi.fn(async () => view());
        const deps: IWatchDeps = { openStream: async () => Promise.reject(new Error("x")), getJob, getStatus: async () => ONLINE };
        const w = watchJob("j1", handlers(), deps);
        await flush();
        w.stop();
        await vi.advanceTimersByTimeAsync(POLL_MS * 10);
        expect(getJob).toHaveBeenCalledTimes(1);
    });

    it("gives up after the poll budget", async () => {
        const getJob = vi.fn(async () => view());
        const h = handlers();
        watchJob("j1", h, { openStream: async () => Promise.reject(new Error("x")), getJob, getStatus: async () => ONLINE, maxPolls: 3 });
        await flush();
        await vi.advanceTimersByTimeAsync(POLL_MS * 10);
        expect(getJob).toHaveBeenCalledTimes(3);
        expect(h.log.at(-1)).toBe("requestError:?");
    });
});

describe("the delta buffer", () => {
    it("orders by seq and keeps the first copy of a repeated seq", () => {
        const b = createDeltaBuffer();
        b.add(1, "b");
        b.add(0, "a");
        b.add(1, "X");
        expect(b.text()).toBe("ab");
    });
});
