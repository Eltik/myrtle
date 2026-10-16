/**
 * Trevor, the story question answerer: the reader's client for contract v1
 * section 4 (`trevor/design/trevor-contract-v1.md`).
 *
 * - Panels and status are public. Panels are per story and per corpus
 *   version: `?v=` naming the active version makes the backend answer them
 *   immutable for a day, so a caller that knows the version (from `/status`)
 *   passes it.
 * - Asking, a job, its stream and feedback need an account. The token is the
 *   httpOnly `site_token` cookie, which only the server can read, so every
 *   authed call is a server function; a signed-out call throws `APIError(401)`
 *   without reaching the backend.
 * - The stream is proxied as a RAW Response (TanStack Start passes a returned
 *   `Response` through untouched, `x-tss-raw`), read in the browser by
 *   `lib/trevor/sse.ts`. A native `EventSource` cannot carry the bearer header.
 *
 * Every non-2xx answer is thrown as the backend's own `APIError` (`parseError`),
 * so `classifyFailure` and `lib/trevor/errors.ts` see its status.
 */
import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getRequest } from "@tanstack/react-start/server";
import { backendFetch } from "#/lib/fetch";
import type { AskRequest } from "#/types/generated/AskRequest";
import type { AskResponse } from "#/types/generated/AskResponse";
import type { FeedbackRequest } from "#/types/generated/FeedbackRequest";
import type { JobView } from "#/types/generated/JobView";
import type { StoryPanels } from "#/types/generated/StoryPanels";
import type { TrevorStatus } from "#/types/generated/TrevorStatus";
import { parseError } from "./_shared";
import { requireSiteToken } from "./_shared.server";

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

export function trevorStatusQueryOptions() {
    return queryOptions({
        queryKey: ["trevor", "status"],
        queryFn: getStatus,
        staleTime: 15 * 1000,
        retry: 1,
    });
}

/** Ask a question. 200 is a cached answer; 202 is the job that will answer. */
export const askTrevorFn = createServerFn({ method: "POST" })
    .inputValidator((data: AskRequest) => data)
    .handler(async ({ data }) => {
        const token = requireSiteToken();
        const body: AskRequest = { question: data.question };
        if (data.horizonStory) body.horizonStory = data.horizonStory;
        if (data.turnstileToken) body.turnstileToken = data.turnstileToken;
        return readJSON<AskResponse>(await backendFetch("/trevor/ask", { method: "POST", bearerToken: token, body: JSON.stringify(body) }));
    });

export function ask(input: { question: string; horizonStory?: string | null; turnstileToken?: string | null }): Promise<AskResponse> {
    return askTrevorFn({ data: { question: input.question, horizonStory: input.horizonStory ?? undefined, turnstileToken: input.turnstileToken ?? undefined } });
}

/** A job's state: the polling fallback. */
export const getTrevorJobFn = createServerFn({ method: "GET" })
    .inputValidator((jobId: string) => jobId)
    .handler(async ({ data: jobId }) => {
        const token = requireSiteToken();
        return readJSON<JobView>(await backendFetch(`/trevor/jobs/${encodeURIComponent(jobId)}`, { bearerToken: token }));
    });

export function getJob(jobId: string): Promise<JobView> {
    return getTrevorJobFn({ data: jobId });
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
    .handler(async ({ data: { answerId, vote, note } }) => {
        const token = requireSiteToken();
        const res = await backendFetch(`/trevor/answers/${encodeURIComponent(answerId)}/feedback`, { method: "POST", bearerToken: token, body: JSON.stringify({ vote, note } satisfies FeedbackRequest) });
        if (!res.ok) throw await parseError(res);
        return true;
    });

export function sendFeedback(answerId: string, vote: -1 | 0 | 1, note: string | null = null): Promise<boolean> {
    return sendTrevorFeedbackFn({ data: { answerId, vote, note } });
}
