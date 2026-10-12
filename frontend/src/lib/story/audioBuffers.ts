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
 *
 * THE `.m4a` SIBLING. Every game music clip (`sound_beta_2/music/**` and the
 * `m_*` clips under `audio/anon`) has an AAC `.m4a` beside its `.ogg`, written
 * by the unpacker (`assets/unpacker/src/export/music_m4a.rs`) for browsers
 * without a Vorbis decoder; iOS Safari is the reported case, unverified on a
 * device. A music url is a list of candidates tried in turn until one
 * decodes: the `.ogg` first wherever the media element claims Vorbis, so
 * Chrome and Firefox fetch exactly what they did before and reach the `.m4a`
 * only if the Vorbis decode rejects; the `.m4a` first where it claims AAC and
 * not Vorbis, so iOS does not spend a fetch on a file it cannot decode. The
 * probe is the media element's, not Web Audio's, so a wrong answer costs one
 * extra fetch and never the sound, because the other file is still tried.
 * The `.m4a` carries ffmpeg's 1024-sample AAC priming in an `elst` edit list;
 * macOS AudioToolbox, which WebKit decodes through, honours it and returns the
 * Vorbis sample count exactly (measured 2026-10-13 on m_bat2_vectorbreak2,
 * lag 0), so the intro-to-loop join is scheduled from the same durations and
 * nothing is trimmed here.
 */

const MUSIC_OGG = /^(?<head>[^?#]*\/(?:sound_beta_2\/music\/[^?#]*|anon\/[^/?#]+\/m_[^/?#]*))\.ogg(?<tail>[?#].*)?$/i;

/** The unpacker's `.m4a` beside a game music `.ogg`, or null for any other url (voices and SFX have none). */
export function m4aSibling(url: string): string | null {
    const m = MUSIC_OGG.exec(url);
    return m?.groups ? `${m.groups.head}.m4a${m.groups.tail ?? ""}` : null;
}

let m4aFirstMemo: boolean | null = null;

/**
 * Whether the media element answers "" for Vorbis and something for AAC. Both
 * are asked because jsdom answers "" to every type, and a test environment must
 * keep the `.ogg`-first order.
 */
export function prefersM4a(): boolean {
    if (m4aFirstMemo !== null) return m4aFirstMemo;
    let out = false;
    try {
        if (typeof document !== "undefined") {
            const el = document.createElement("audio");
            out = el.canPlayType('audio/ogg; codecs="vorbis"') === "" && el.canPlayType('audio/mp4; codecs="mp4a.40.2"') !== "";
        }
    } catch {
        out = false;
    }
    m4aFirstMemo = out;
    return out;
}

/** The urls to try for one request, in order. */
export function candidatesFor(url: string, m4aFirst = prefersM4a()): string[] {
    const alt = m4aSibling(url);
    if (!alt) return [url];
    return m4aFirst ? [alt, url] : [url, alt];
}

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
                // Where every candidate loses, a decode-error outranks a fetch
                // error: a missing `.m4a` beside an undecodable `.ogg` is a codec
                // problem first, in either order.
                // An `.ogg` that will not even FETCH ends the list: its `.m4a`
                // is written from it, so there is no sibling to find, and Chrome
                // keeps exactly one request per missing clip as before.
                let lost: FetchStatus = null;
                for (const candidate of candidatesFor(url)) {
                    let bytes: ArrayBuffer;
                    try {
                        const res = await fetch(candidate);
                        if (!res.ok) {
                            lost = lost === "decode-error" ? lost : "http-error";
                            if (candidate === url) break;
                            continue;
                        }
                        bytes = await res.arrayBuffer();
                    } catch {
                        lost = lost === "decode-error" ? lost : "network-error";
                        if (candidate === url) break;
                        continue;
                    }
                    try {
                        const buf = await ctx.decodeAudioData(bytes);
                        status.set(url, "ok");
                        return buf;
                    } catch {
                        lost = "decode-error";
                    }
                }
                status.set(url, lost);
                onFailure();
                return null;
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
