/**
 * Trevor's client (contract v1 section 4, `trevor/design/trevor-contract-v1.md`).
 *
 * Panels and status are public; naming the active version (`?v=`, from
 * `/status`) lets the backend answer panels immutable for a day. Asking, a
 * job, its stream and feedback need the httpOnly `site_token` cookie, which
 * only the server can read, so each is a server function; signed out they
 * throw `APIError(401)` without reaching the backend. The stream is proxied as
 * a raw Response (a native `EventSource` cannot send the bearer header) and
 * read in the browser by `lib/trevor/sse.ts`.
 *
 * An error thrown inside a server function reaches the browser as a bare
 * `Error` without `status` and `kind`, so the authed calls return a refusal as
 * data (`settle`) and the browser rethrows it as the `APIError` it was
 * (`unwrap`); `classifyFailure` and `lib/trevor/errors.ts` then see its status.
 */
import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getRequest } from "@tanstack/react-start/server";
import { backendFetch } from "#/lib/fetch";
import { type IMyJobs, type IPopular, parseMyJobs, parsePopular } from "#/lib/trevor/library";
import type { AskRequest } from "#/types/generated/AskRequest";
import type { AskResponse } from "#/types/generated/AskResponse";
import type { FeedbackRequest } from "#/types/generated/FeedbackRequest";
import type { JobView } from "#/types/generated/JobView";
import type { StoryPanels } from "#/types/generated/StoryPanels";
import type { TrevorMyQuestion } from "#/types/generated/TrevorMyQuestion";
import type { TrevorPopular } from "#/types/generated/TrevorPopular";
import type { TrevorStatus } from "#/types/generated/TrevorStatus";
import { APIError, parseError } from "./_shared";
import { optionalSiteToken, requireSiteToken } from "./_shared.server";

/** A server function's outcome as plain data: the value, or the refusal's status, message and code. */
export type Settled<T> = { ok: true; value: T } | { ok: false; status: number; message: string; code: string | null };

/** Run a server-side call and return an `APIError` as data instead of throwing it across the boundary. */
export async function settle<T>(run: () => Promise<T>): Promise<Settled<T>> {
    try {
        return { ok: true, value: await run() };
    } catch (err) {
        if (err instanceof APIError) return { ok: false, status: err.status, message: err.message, code: err.code };
        throw err;
    }
}

/** The browser side of `settle`: the value, or the refusal rethrown as the `APIError` it was. */
export function unwrap<T>(s: Settled<T>): T {
    if (s.ok) return s.value;
    throw new APIError(s.status, s.message, s.code);
}

/** The response body of a successful request; the backend's error otherwise. */
async function readJSON<T>(res: Response): Promise<T> {
    if (!res.ok) throw await parseError(res);
    return (await res.json()) as T;
}

/** One story's panels, or `null` when it has none (404: unscripted stories, or no active version). */
export const getTrevorPanelsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { storyId: string; version: string | null }) => data)
    .handler(async ({ data: { storyId, version } }) => {
        const query = version ? `?v=${encodeURIComponent(version)}` : "";
        const res = await backendFetch(`/trevor/panels/${encodeURIComponent(storyId)}${query}`);
        if (res.status === 404) return null;
        return readJSON<StoryPanels>(res);
    });

export function getPanels(storyId: string, version: string | null = null): Promise<StoryPanels | null> {
    return getTrevorPanelsFn({ data: { storyId, version } });
}

export function trevorPanelsQueryOptions(storyId: string, version: string | null = null) {
    return queryOptions({
        queryKey: ["trevor", "panels", storyId, version],
        queryFn: () => getPanels(storyId, version),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
        retry: 1,
    });
}

/** Whether Trevor can answer right now. Public; the backend caches it for 15 s. */
export const getTrevorStatusFn = createServerFn({ method: "GET" }).handler(async () => readJSON<TrevorStatus>(await backendFetch("/trevor/status")));

export function getStatus(): Promise<TrevorStatus> {
    return getTrevorStatusFn();
}

/** The status, fetched and polled every 30 s only while `active` (the sheet or the tab on screen). */
export function trevorStatusQueryOptions(active: boolean) {
    return queryOptions({
        queryKey: ["trevor", "status"],
        queryFn: getStatus,
        staleTime: 15 * 1000,
        retry: 1,
        enabled: active,
        refetchInterval: active ? 30_000 : false,
    });
}

/** Ask a question. 200 is a cached answer; 202 is the job that will answer. */
export const askTrevorFn = createServerFn({ method: "POST" })
    .inputValidator((data: AskRequest) => data)
    .handler(({ data }) =>
        settle(async () => {
            const token = requireSiteToken();
            const body: AskRequest = { question: data.question };
            if (data.horizonStory) body.horizonStory = data.horizonStory;
            if (data.turnstileToken) body.turnstileToken = data.turnstileToken;
            // Only spoiler mode is sent: a spoiler-safe ask keeps the body it always had.
            if (data.spoilers === "all") body.spoilers = "all";
            return readJSON<AskResponse>(await backendFetch("/trevor/ask", { method: "POST", bearerToken: token, body: JSON.stringify(body) }));
        }),
    );

export async function ask(input: { question: string; horizonStory?: string | null; turnstileToken?: string | null; spoilers?: "all" }): Promise<AskResponse> {
    return unwrap(await askTrevorFn({ data: { question: input.question, horizonStory: input.horizonStory ?? undefined, turnstileToken: input.turnstileToken ?? undefined, ...(input.spoilers ? { spoilers: input.spoilers } : {}) } }));
}

/** A job's state: the polling fallback. */
export const getTrevorJobFn = createServerFn({ method: "GET" })
    .inputValidator((jobId: string) => jobId)
    .handler(({ data: jobId }) =>
        settle(async () => {
            const token = requireSiteToken();
            return readJSON<JobView>(await backendFetch(`/trevor/jobs/${encodeURIComponent(jobId)}`, { bearerToken: token }));
        }),
    );

export async function getJob(jobId: string): Promise<JobView> {
    return unwrap(await getTrevorJobFn({ data: jobId }));
}

/** The client's request signal, so a reader who closes the panel closes the upstream stream too. */
function requestSignal(): AbortSignal | undefined {
    try {
        return getRequest().signal;
    } catch {
        return undefined;
    }
}

/**
 * A job's answer as it is written: the backend's SSE body, passed through.
 * Compression is off on this hop (`backendFetch` asks for identity server
 * side) and the response says `no-transform`, so no layer buffers the deltas.
 */
export const streamTrevorJobFn = createServerFn({ method: "GET" })
    .inputValidator((jobId: string) => jobId)
    .handler(async ({ data: jobId }) => {
        const token = requireSiteToken();
        const upstream = await backendFetch(`/trevor/jobs/${encodeURIComponent(jobId)}/stream`, { bearerToken: token, headers: { accept: "text/event-stream" }, signal: requestSignal() });
        if (!upstream.ok || !upstream.body) throw await parseError(upstream);
        return new Response(upstream.body, {
            status: 200,
            headers: { "content-type": "text/event-stream; charset=utf-8", "cache-control": "no-cache, no-transform", "x-accel-buffering": "no" },
        });
    });

export function streamJob(jobId: string, signal?: AbortSignal): Promise<Response> {
    return streamTrevorJobFn({ data: jobId, signal });
}

/** Vote on an answer (1 up, -1 down, 0 withdraws), optionally with a "this is wrong" note. */
export const sendTrevorFeedbackFn = createServerFn({ method: "POST" })
    .inputValidator((data: { answerId: string } & FeedbackRequest) => data)
    .handler(({ data: { answerId, vote, note } }) =>
        settle(async () => {
            const token = requireSiteToken();
            const res = await backendFetch(`/trevor/answers/${encodeURIComponent(answerId)}/feedback`, { method: "POST", bearerToken: token, body: JSON.stringify({ vote, note } satisfies FeedbackRequest) });
            if (!res.ok) throw await parseError(res);
            return true;
        }),
    );

export async function sendFeedback(answerId: string, vote: -1 | 0 | 1, note: string | null = null): Promise<boolean> {
    return unwrap(await sendTrevorFeedbackFn({ data: { answerId, vote, note } }));
}

/**
 * The most-asked answered questions inside the asker's era (contract v1.1
 * section 5), for the stories tab's chips. Public: the token is sent when
 * there is one, so the backend can place the reader's era; anonymous is era 0
 * and an empty list. `null` when the backend has no such route (404).
 */
export const getTrevorPopularFn = createServerFn({ method: "GET" })
    .inputValidator((limit: number) => limit)
    .handler(async ({ data: limit }) => {
        const res = await backendFetch(`/trevor/popular?limit=${limit}`, { bearerToken: optionalSiteToken() });
        if (res.status === 404) return null;
        return parsePopular(await readJSON<TrevorPopular>(res));
    });

export function getPopular(limit = 8): Promise<IPopular | null> {
    return getTrevorPopularFn({ data: limit });
}

export function trevorPopularQueryOptions(signedIn: boolean, limit = 8) {
    return queryOptions({
        queryKey: ["trevor", "popular", signedIn, limit],
        queryFn: () => getPopular(limit),
        staleTime: 5 * 60 * 1000,
        retry: 1,
    });
}

/** The reader's recent questions ("My questions"), newest first; `null` when the backend has no such route (404). */
export const getTrevorMyJobsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { limit: number; before: string | null }) => data)
    .handler(({ data: { limit, before } }) =>
        settle(async () => {
            const token = requireSiteToken();
            const query = `?limit=${limit}${before ? `&before=${encodeURIComponent(before)}` : ""}`;
            const res = await backendFetch(`/trevor/me/jobs${query}`, { bearerToken: token });
            if (res.status === 404) return null;
            return parseMyJobs(await readJSON<TrevorMyQuestion[]>(res), limit);
        }),
    );

export async function getMyJobs(limit = 20, before: string | null = null): Promise<IMyJobs | null> {
    return unwrap(await getTrevorMyJobsFn({ data: { limit, before } }));
}

export function trevorMyJobsQueryOptions(limit = 20) {
    return queryOptions({
        queryKey: ["trevor", "me", "jobs", limit],
        queryFn: () => getMyJobs(limit),
        staleTime: 15 * 1000,
        retry: 1,
    });
}
