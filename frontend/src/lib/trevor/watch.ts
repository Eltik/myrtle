/**
 * FOLLOW ONE JOB TO ITS ANSWER: the stream first, polling when it fails.
 *
 * The stream (`GET /trevor/jobs/{id}/stream`, contract v1 section 4) carries
 * `delta` `{seq, text}`, then one `done` (the job view) or `error`
 * `{message}`. When it cannot be opened, or closes without either, the job is
 * polled (`GET /trevor/jobs/{id}`) every `pollMs` (5 s) until it is done or
 * failed. Polling also stops when the job says no worker is online AND
 * `GET /trevor/status` agrees: a question with nobody to answer it waits in
 * the queue, and asking every 5 s for an hour would only spend the reader's
 * battery. The caller may resume with a fresh `watchJob` call.
 *
 * Pure of React and of the network: every request comes in through `deps`, so
 * the tests drive it with a fake stream and fake timers.
 */
import type { JobView } from "#/types/generated/JobView";
import type { TrevorStatus } from "#/types/generated/TrevorStatus";
import { readSse } from "./sse";

/** The polling interval the contract's fallback names. */
export const POLL_MS = 5_000;
/** At most this many polls (15 minutes at 5 s), past the backend's 600 s queue TTL. */
export const MAX_POLLS = 180;

export interface IWatchDeps {
    /** The job's SSE body as a `fetch` Response (a server function proxies it with the account's token). */
    openStream: (jobId: string, signal: AbortSignal) => Promise<Response>;
    getJob: (jobId: string) => Promise<JobView>;
    getStatus: () => Promise<TrevorStatus>;
    pollMs?: number;
    maxPolls?: number;
    setTimer?: (fn: () => void, ms: number) => unknown;
    clearTimer?: (handle: unknown) => void;
}

export interface IWatchHandlers {
    /** The answer text so far, assembled in `seq` order. */
    onText: (text: string) => void;
    /** A poll found the job still queued or running. */
    onProgress?: (view: JobView) => void;
    onDone: (view: JobView) => void;
    /** The job itself failed (`status: failed` or a stream `error` event). */
    onJobFailed: (message: string) => void;
    /** A poll request failed in a way retrying will not fix (401, 403, 404), or the poll budget ran out. */
    onRequestError: (err: unknown) => void;
    /** No worker is online: polling stopped, the question is still queued. */
    onOffline: () => void;
    /** Which transport is carrying the job now. */
    onMode?: (mode: "stream" | "poll") => void;
}

export interface IJobWatch {
    stop: () => void;
}

/** Statuses a retry will not change. */
function permanent(err: unknown): boolean {
    const status = err && typeof err === "object" ? (err as { status?: unknown }).status : undefined;
    return status === 401 || status === 403 || status === 404;
}

/** Assembles deltas by `seq`, ignoring repeats, so an out-of-order or repeated delta cannot garble the text. */
export function createDeltaBuffer(): { add: (seq: number, text: string) => string; text: () => string } {
    const parts = new Map<number, string>();
    const join = () =>
        [...parts.entries()]
            .sort((a, b) => a[0] - b[0])
            .map(([, t]) => t)
            .join("");
    return {
        add(seq, text) {
            if (!parts.has(seq)) parts.set(seq, text);
            return join();
        },
        text: join,
    };
}

export function watchJob(jobId: string, handlers: IWatchHandlers, deps: IWatchDeps): IJobWatch {
    const pollMs = deps.pollMs ?? POLL_MS;
    const maxPolls = deps.maxPolls ?? MAX_POLLS;
    const setTimer = deps.setTimer ?? ((fn: () => void, ms: number) => setTimeout(fn, ms));
    const clearTimer = deps.clearTimer ?? ((h: unknown) => clearTimeout(h as ReturnType<typeof setTimeout>));
    const ctrl = new AbortController();
    let stopped = false;
    let timer: unknown = null;
    const deltas = createDeltaBuffer();

    const stop = () => {
        stopped = true;
        ctrl.abort();
        if (timer !== null) clearTimer(timer);
        timer = null;
    };

    /** True when the stream delivered the job's end (done or error). */
    const stream = async (): Promise<boolean> => {
        let terminal = false;
        try {
            const res = await deps.openStream(jobId, ctrl.signal);
            if (stopped || !res.ok || !res.body) return false;
            handlers.onMode?.("stream");
            await readSse(
                res.body,
                (e) => {
                    if (terminal || stopped) return;
                    if (e.event === "delta") {
                        const d = safeJson(e.data) as { seq?: unknown; text?: unknown } | null;
                        if (d && typeof d.text === "string") handlers.onText(deltas.add(Number(d.seq) || 0, d.text));
                    } else if (e.event === "done") {
                        const view = safeJson(e.data) as JobView | null;
                        if (!view || typeof view !== "object") return;
                        terminal = true;
                        handlers.onDone(view);
                        ctrl.abort();
                    } else if (e.event === "error") {
                        const d = safeJson(e.data) as { message?: unknown } | null;
                        terminal = true;
                        handlers.onJobFailed(typeof d?.message === "string" ? d.message : "");
                        ctrl.abort();
                    }
                },
                ctrl.signal,
            );
        } catch {
            // Fall through to polling: a stream that will not open or breaks
            // mid-answer still has a job behind it.
        }
        return terminal;
    };

    const poll = async (n: number): Promise<void> => {
        timer = null;
        if (stopped) return;
        if (n >= maxPolls) {
            handlers.onRequestError(new Error("polling gave up"));
            return;
        }
        let view: JobView;
        try {
            view = await deps.getJob(jobId);
        } catch (err) {
            if (stopped) return;
            if (permanent(err)) {
                handlers.onRequestError(err);
                return;
            }
            timer = setTimer(() => void poll(n + 1), pollMs);
            return;
        }
        if (stopped) return;
        if (view.status === "done") return handlers.onDone(view);
        if (view.status === "failed") return handlers.onJobFailed(view.error ?? "");
        handlers.onProgress?.(view);
        if (!view.workerOnline) {
            const status = await deps.getStatus().catch(() => null);
            if (stopped) return;
            if (status && !status.online) {
                handlers.onOffline();
                return;
            }
        }
        timer = setTimer(() => void poll(n + 1), pollMs);
    };

    void (async () => {
        if (await stream()) return;
        if (stopped) return;
        handlers.onMode?.("poll");
        await poll(0);
    })();

    return { stop };
}

function safeJson(data: string): unknown {
    try {
        return JSON.parse(data) as unknown;
    } catch {
        return null;
    }
}
