import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { backendFetch } from "#/lib/fetch";
import type { StoryProgress } from "#/lib/story/progress";
import type { StoryProgressResponse } from "#/types/generated/StoryProgressResponse";

/**
 * The account side of the reader's progress: `GET`/`PUT /user/story-progress`.
 *
 * Neither call ever throws across the RPC boundary. Reading is the point of the
 * page and syncing is the convenience beside it, so a dead backend, a stale
 * session or a 400 has to come back as a VALUE the caller can render, not as an
 * exception that unmounts a reader mid-story. `lib/story/sync.ts` turns these
 * results into the state the Progress tab shows.
 */
export type SyncFailure = "offline" | "unauthorized" | "error";

export type PullResult =
    | {
          ok: true;
          progress: StoryProgress | null;
          updatedAt: number | null;
          /** Story ids the GAME says the account has read, imported by the last game-data refresh. */
          gameRead: string[];
          /**
           * Story ids the game's Archive lists that nothing ever played. These
           * are exactly the marks the first import wrongly baked into a v1
           * document, and the merge withdraws them from one.
           */
          gameUnread: string[];
          /** How many of `gameRead` the game's own Archive lists; the rest come from its played-script flags. */
          gameArchived: number;
          /** Unix seconds of that import, null when the account has never refreshed game data. */
          gameSyncedAt: number | null;
      }
    | { ok: false; reason: SyncFailure };

export type PushResult = { ok: true; updatedAt: number | null } | { ok: false; reason: SyncFailure };

/** Map a non-2xx status onto the three states the UI distinguishes. */
function failureFor(status: number): SyncFailure {
    if (status === 401 || status === 403) return "unauthorized";
    // 404 is the shape a backend older than this route answers with, and it is
    // not a signed-out state: the session is fine, the endpoint is not there
    // yet. It reads as a plain failure and retries on the next trigger.
    return "error";
}

/** The response of a pull, either route, in the shape the sync module reads. */
function pulled(body: StoryProgressResponse): PullResult {
    return {
        ok: true,
        progress: (body.progress as StoryProgress | null) ?? null,
        updatedAt: body.updatedAt,
        // A backend older than the import answers none of these fields,
        // and one older than the correction answers every one but
        // `gameUnread`. Reading them through a default keeps either case a
        // no-op: an absent `gameUnread` withdraws nothing, which is what a
        // backend that cannot say what to withdraw should cost.
        gameRead: body.gameRead ?? [],
        gameUnread: body.gameUnread ?? [],
        gameArchived: body.gameArchived ?? 0,
        gameSyncedAt: body.gameSyncedAt ?? null,
    };
}

export const getStoryProgressFn = createServerFn({ method: "GET" }).handler(async (): Promise<PullResult> => {
    const token = getCookie("site_token");
    if (!token) return { ok: false, reason: "unauthorized" };
    try {
        const res = await backendFetch("/user/story-progress", { bearerToken: token });
        if (!res.ok) return { ok: false, reason: failureFor(res.status) };
        return pulled((await res.json()) as StoryProgressResponse);
    } catch {
        return { ok: false, reason: "offline" };
    }
});

/**
 * The pull behind "Sync now": the backend RE-DERIVES the game's verdict from
 * the game data it already stores (the played and Archive marks it imported,
 * and the account's stage records) and answers as the GET does. It never
 * reaches the game server, so it is safe to press with the game open, and a
 * corrected rule reaches the account without a refresh.
 */
export const importStoryProgressFn = createServerFn({ method: "POST" }).handler(async (): Promise<PullResult> => {
    const token = getCookie("site_token");
    if (!token) return { ok: false, reason: "unauthorized" };
    try {
        const res = await backendFetch("/user/story-progress/import", { method: "POST", bearerToken: token });
        if (!res.ok) return { ok: false, reason: failureFor(res.status) };
        return pulled((await res.json()) as StoryProgressResponse);
    } catch {
        return { ok: false, reason: "offline" };
    }
});

export const putStoryProgressFn = createServerFn({ method: "POST" })
    .inputValidator((data: StoryProgress) => data)
    .handler(async ({ data }): Promise<PushResult> => {
        const token = getCookie("site_token");
        if (!token) return { ok: false, reason: "unauthorized" };
        try {
            const res = await backendFetch("/user/story-progress", { method: "PUT", bearerToken: token, body: JSON.stringify(data) });
            if (!res.ok) return { ok: false, reason: failureFor(res.status) };
            const body = (await res.json()) as StoryProgressResponse;
            return { ok: true, updatedAt: body.updatedAt };
        } catch {
            return { ok: false, reason: "offline" };
        }
    });
