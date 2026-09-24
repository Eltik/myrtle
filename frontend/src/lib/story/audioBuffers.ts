/**
 * THE DECODE CACHE, one entry per url and never one per use.
 *
 * The corpus is 869 Ogg Vorbis files and a story replays the same cue on every
 * `goTo`, so the fetch and the `decodeAudioData` are memoised by url. The
 * PROMISE is what is cached, not the buffer: two `[playsound]` on the same file
 * in one step must share one fetch, and the second must not start a second
 * decode while the first is still in flight.
 *
 * A failure is cached too, as a resolved null. A browser without Vorbis in Web
 * Audio rejects the decode for every file it will ever be handed, and without
 * the null the media-element fallback would refetch and re-decode on every use.
 * `onFailure` is how the owner learns to set `lastFallback`; the status map is
 * what the dev ledger prints.
 */

/** Where a url got to. `pending` is in flight, `cached` is a hit on a settled entry. */
export type FetchStatus = "cached" | "ok" | "http-error" | "network-error" | "decode-error" | "pending" | null;

export interface IBufferCache {
    /** The decoded buffer, or null where the fetch or the decode lost. */
    get(url: string): Promise<AudioBuffer | null>;
    statusOf(url: string): FetchStatus;
}

/**
 * @param context reads the live context, or null before `arm()` and after dispose.
 * @param onFailure called once per url that fails to fetch or decode.
 */
export function createBufferCache(context: () => AudioContext | null, onFailure: () => void): IBufferCache {
    const buffers = new Map<string, Promise<AudioBuffer | null>>();
    const status = new Map<string, FetchStatus>();

    return {
        get(url: string): Promise<AudioBuffer | null> {
            const cached = buffers.get(url);
            if (cached) return cached;
            const p = (async () => {
                const ctx = context();
                if (!ctx) return null;
                let bytes: ArrayBuffer;
                try {
                    const res = await fetch(url);
                    if (!res.ok) {
                        status.set(url, "http-error");
                        onFailure();
                        return null;
                    }
                    bytes = await res.arrayBuffer();
                } catch {
                    status.set(url, "network-error");
                    onFailure();
                    return null;
                }
                try {
                    const buf = await ctx.decodeAudioData(bytes);
                    status.set(url, "ok");
                    return buf;
                } catch {
                    status.set(url, "decode-error");
                    onFailure();
                    return null;
                }
            })();
            buffers.set(url, p);
            status.set(url, "pending");
            return p;
        },
        statusOf(url: string): FetchStatus {
            return status.get(url) ?? null;
        },
    };
}
